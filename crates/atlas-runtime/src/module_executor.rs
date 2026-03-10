//! Module Execution Engine
//!
//! Coordinates module loading and execution for both interpreter and VM.
//! Ensures single evaluation per module with proper dependency order.

use crate::ast::{ImportDecl, ImportSpecifier, Item};
use crate::binder::Binder;
use crate::diagnostic::error_codes::EXPORT_NOT_FOUND;
use crate::diagnostic::Diagnostic;
use crate::interpreter::Interpreter;
use crate::module_loader::{LoadedModule, ModuleLoader, ModuleRegistry};
use crate::resolver::ModuleResolver;
use crate::security::SecurityContext;
use crate::typechecker::TypeChecker;
use crate::value::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Module execution result
pub type ModuleResult<T> = Result<T, Vec<Diagnostic>>;

/// Cache of executed modules and their exports
#[derive(Debug, Clone)]
struct ModuleCache {
    /// Map of module path -> exported symbols (name -> value)
    exports: HashMap<PathBuf, HashMap<String, Value>>,
    /// Map of module path -> type-only export names (struct/enum — no runtime value)
    type_exports: HashMap<PathBuf, std::collections::HashSet<String>>,
}

impl ModuleCache {
    fn new() -> Self {
        Self {
            exports: HashMap::new(),
            type_exports: HashMap::new(),
        }
    }

    fn is_cached(&self, path: &Path) -> bool {
        self.exports.contains_key(path)
    }

    fn store_exports(
        &mut self,
        path: PathBuf,
        exports: HashMap<String, Value>,
        type_only: std::collections::HashSet<String>,
    ) {
        self.type_exports.insert(path.clone(), type_only);
        self.exports.insert(path, exports);
    }

    fn get_exports(&self, path: &Path) -> Option<&HashMap<String, Value>> {
        self.exports.get(path)
    }
}

/// Module executor for interpreter-based execution
///
/// Borrows an interpreter to ensure imports populate the caller's state.
/// See `.claude/memory/patterns/runtime.md` for module execution patterns.
pub struct ModuleExecutor<'a> {
    /// Module loader for resolving and loading dependencies
    loader: ModuleLoader,
    /// Module resolver for path resolution
    resolver: ModuleResolver,
    /// Cache of executed modules
    cache: ModuleCache,
    /// Registry of bound symbol tables for cross-module type checking (H-248)
    type_registry: ModuleRegistry,
    /// Borrowed interpreter instance (ensures imports populate caller's interpreter)
    interpreter: &'a mut Interpreter,
    /// Security context for permission checks
    security: &'a SecurityContext,
}

impl<'a> ModuleExecutor<'a> {
    /// Create a new module executor
    ///
    /// # Arguments
    /// * `interpreter` - Borrowed interpreter to populate with imports
    /// * `security` - Security context for permission checks
    /// * `root` - Project root directory for module resolution
    pub fn new(
        interpreter: &'a mut Interpreter,
        security: &'a SecurityContext,
        root: PathBuf,
    ) -> Self {
        Self {
            loader: ModuleLoader::new(root.clone()),
            resolver: ModuleResolver::new(root),
            cache: ModuleCache::new(),
            type_registry: ModuleRegistry::new(),
            interpreter,
            security,
        }
    }

    /// Execute a module file and all its dependencies
    ///
    /// Loads and executes modules in topological order (dependencies first).
    /// Each module executes exactly once. Returns the entry module's result.
    ///
    /// # Arguments
    /// * `entry_path` - Absolute path to the entry module file
    ///
    /// # Returns
    /// The result value from executing the entry module
    pub fn execute_module(&mut self, entry_path: &Path) -> ModuleResult<Value> {
        // Load all modules in dependency order
        let modules = self.loader.load_module(entry_path)?;

        // Execute each module in order
        let mut last_value = Value::Null;
        for module in modules {
            let result = self.execute_single_module(&module)?;
            // The entry module's result is what we return
            if module.path == entry_path {
                last_value = result;
            }
        }

        Ok(last_value)
    }

    /// Execute a single module
    ///
    /// If the module is already cached, skip execution.
    /// Otherwise, bind + typecheck (with cross-module type context), then execute.
    ///
    /// H-248: typechecking now runs here so `atlas run` and `atlas build` catch
    /// struct field errors, type mismatches, and missing names at compile time —
    /// not at runtime.
    fn execute_single_module(&mut self, module: &LoadedModule) -> ModuleResult<Value> {
        // Skip if already executed
        if self.cache.is_cached(&module.path) {
            return Ok(Value::Null);
        }

        // --- Compile-time phase (H-248) ---
        // Bind using the registry so cross-module struct/enum types are in scope.
        let mut binder = Binder::new();
        let (mut symbol_table, bind_diags) =
            binder.bind_with_modules(&module.ast, &module.path, &self.type_registry);
        let bind_has_errors = bind_diags.iter().any(|d| d.is_error());

        // Run typechecker regardless of bind errors — collect all diagnostics together.
        let mut type_checker = TypeChecker::new(&mut symbol_table);
        let type_diags = type_checker.check(&module.ast);
        let type_has_errors = type_diags.iter().any(|d| d.is_error());

        if bind_has_errors || type_has_errors {
            let mut all = bind_diags;
            all.extend(type_diags);
            return Err(all);
        }

        // Store this module's symbol table so downstream modules can see its exported types.
        self.type_registry
            .register(module.path.clone(), symbol_table);

        // Process imports - inject imported symbols into interpreter globals
        for import in &module.imports {
            self.process_import(import, &module.path)?;
        }

        // Set module path for the interpreter (enables inline import resolution if needed)
        self.interpreter.set_module_path(Some(module.path.clone()));

        // Execute the module
        let result = self
            .interpreter
            .eval(&module.ast, self.security)
            .map_err(|e| {
                let stack_trace = self.interpreter.stack_trace_frames(e.span(), None);
                let function_name = stack_trace.first().map(|frame| frame.function.clone());
                self.interpreter.reset_call_stack();
                vec![crate::runtime::runtime_error_to_diagnostic(
                    e,
                    stack_trace,
                    function_name,
                )]
            })?;

        // Extract and cache exports
        let (exports, type_only) = self.extract_exports(module);
        self.cache
            .store_exports(module.path.clone(), exports, type_only);

        Ok(result)
    }

    /// Process an import declaration
    ///
    /// Resolves the module path, retrieves cached exports, and injects
    /// imported symbols into the interpreter's globals.
    fn process_import(&mut self, import: &ImportDecl, current_path: &Path) -> ModuleResult<()> {
        // Resolve the import path relative to current module
        let import_path = self
            .resolver
            .resolve_path(&import.source, current_path, import.span)
            .map_err(|e| vec![e])?;

        // Get cached exports (module should already be executed due to topological order)
        // Clone both maps to avoid borrow conflicts in the loop below.
        let exports = self
            .cache
            .get_exports(&import_path)
            .ok_or_else(|| {
                vec![Diagnostic::error(
                    format!(
                        "Module not yet executed: {}. This indicates a bug in topological sorting.",
                        import_path.display()
                    ),
                    import.span,
                )]
            })?
            .clone();
        let type_only = self
            .cache
            .type_exports
            .get(&import_path)
            .cloned()
            .unwrap_or_default();

        // Process import specifiers
        for specifier in &import.specifiers {
            match specifier {
                ImportSpecifier::Named { name, span } => {
                    if let Some(value) = exports.get(&name.name) {
                        // Runtime value — inject into interpreter globals
                        self.interpreter
                            .define_global(name.name.clone(), value.clone());
                    } else if type_only.contains(&name.name) {
                        // Type-only export (struct/enum/type alias) — no runtime value needed
                    } else {
                        return Err(vec![EXPORT_NOT_FOUND
                            .emit(*span)
                            .arg("name", &name.name)
                            .arg("module", "this module")
                            .build()]);
                    }
                }
                ImportSpecifier::Namespace { alias, span: _ } => {
                    use crate::stdlib::collections::hash::HashKey;
                    use crate::stdlib::collections::hashmap::AtlasHashMap;
                    use crate::value::ValueHashMap;

                    let mut atlas_map = AtlasHashMap::new();
                    for (name, value) in &exports {
                        let key = HashKey::String(std::sync::Arc::new(name.clone()));
                        atlas_map.insert(key, value.clone());
                    }
                    self.interpreter.define_global(
                        alias.name.clone(),
                        Value::HashMap(ValueHashMap::from_atlas(atlas_map)),
                    );
                }
            }
        }

        Ok(())
    }

    /// Extract exports from an executed module
    ///
    /// Examines the module's AST to find exported items and retrieves
    /// their values from the interpreter's globals.
    fn extract_exports(
        &self,
        module: &LoadedModule,
    ) -> (HashMap<String, Value>, std::collections::HashSet<String>) {
        let mut exports = HashMap::new();
        let mut type_only = std::collections::HashSet::new();

        for item in &module.ast.items {
            if let Item::Export(export_decl) = item {
                match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => {
                        if let Some((value, _mutable)) =
                            self.interpreter.globals.get(&func.name.name)
                        {
                            exports.insert(func.name.name.clone(), value.clone());
                        }
                    }
                    crate::ast::ExportItem::Variable(var) => {
                        if let Some((value, _mutable)) =
                            self.interpreter.globals.get(&var.name.name)
                        {
                            exports.insert(var.name.name.clone(), value.clone());
                        }
                    }
                    crate::ast::ExportItem::TypeAlias(alias) => {
                        type_only.insert(alias.name.name.clone());
                    }
                    crate::ast::ExportItem::Struct(s) => {
                        type_only.insert(s.name.name.clone());
                    }
                    crate::ast::ExportItem::Enum(e) => {
                        type_only.insert(e.name.name.clone());
                    }
                }
            }
        }

        (exports, type_only)
    }
}
