//! Runtime execution API (D-052: unified VM execution)
//!
//! Provides the `Runtime` struct for managing Atlas execution via Compiler+VM.
//! State persists across evaluations.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
//!
//! # Examples
//!
//! ```rust,no_run
//! use atlas_runtime::api::Runtime;
//!
//! let mut runtime = Runtime::new();
//!
//! // Execute code
//! runtime.eval("let x: number = 42;").unwrap();
//!
//! // State persists
//! let result = runtime.eval("x").unwrap();
//! ```

use crate::ast::{
    Expr, Identifier, ImportSpecifier, Item, ObjectEntry, ObjectLiteral, Program, Stmt, VarDecl,
};

/// Emit warnings through the proper diagnostic formatter to stderr (H-196).
/// Replaces raw `eprintln!("{}", diag.to_human_string())` calls.
fn emit_warnings_via_formatter(warnings: &[Diagnostic]) {
    use crate::diagnostic::formatter::DiagnosticFormatter;
    use termcolor::{ColorChoice, StandardStream};
    if warnings.is_empty() {
        return;
    }
    let formatter = DiagnosticFormatter::auto();
    let mut stream = StandardStream::stderr(if std::env::var("NO_COLOR").is_ok() {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    });
    for diag in warnings {
        let _ = formatter.write_diagnostic(&mut stream, diag);
    }
}
use crate::binder::Binder;
use crate::compiler::Compiler;
use crate::diagnostic::error_codes::IMPORT_RESOLUTION_FAILED;
use crate::diagnostic::Diagnostic;
use crate::lexer::Lexer;
use crate::module_loader::ModuleLoader;
use crate::parser::Parser;
use crate::resolver::ModuleResolver;
use crate::security::SecurityContext;
use crate::span::Span;
use crate::typechecker::TypeChecker;
use crate::value::{RuntimeError, Value};
use crate::vm::VM;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

/// Unified error type for runtime evaluation
#[derive(Debug)]
pub enum EvalError {
    /// Lexical or syntax errors
    ParseError(Vec<Diagnostic>),
    /// Type checking errors
    TypeError(Vec<Diagnostic>),
    /// Runtime errors during execution
    RuntimeError(RuntimeError),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::ParseError(diagnostics) => {
                let msgs: Vec<_> = diagnostics.iter().map(|d| d.message.as_str()).collect();
                write!(f, "Parse error: {}", msgs.join("\n  "))
            }
            EvalError::TypeError(diagnostics) => {
                let msgs: Vec<_> = diagnostics.iter().map(|d| d.message.as_str()).collect();
                write!(f, "Type error: {}", msgs.join("\n  "))
            }
            EvalError::RuntimeError(err) => write!(f, "Runtime error: {}", err),
        }
    }
}

impl std::error::Error for EvalError {}

/// Runtime instance managing execution state
///
/// Maintains global variables and function definitions across multiple
/// evaluations. Uses Compiler + VM for execution (D-052: unified execution path).
///
/// # Examples
///
/// ```rust,no_run
/// use atlas_runtime::api::Runtime;
///
/// let mut runtime = Runtime::new();
///
/// // Define a function
/// runtime.eval("fn add(borrow x: number, borrow y: number): number { return x + y; }").unwrap();
///
/// // Call it
/// let result = runtime.eval("add(1, 2)").unwrap();
/// ```
pub struct Runtime {
    /// Global variables and functions (name -> (value, is_mutable))
    /// Replaces interpreter.globals after D-052 unification
    globals: RefCell<HashMap<String, (Value, bool)>>,
    /// Security context for permission checks
    security: SecurityContext,
    /// Execution limits (timeout, memory) for sandbox enforcement
    execution_limits: RefCell<super::config::ExecutionLimits>,
    /// Accumulated bytecode (persists across eval() calls)
    accumulated_bytecode: RefCell<crate::bytecode::Bytecode>,
    /// Output writer for print() (threaded to VM)
    output: crate::stdlib::OutputWriter,
    /// Native function arities (None = variadic)
    native_signatures: RefCell<HashMap<String, Option<usize>>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    /// Create a new runtime with default security (deny-all)
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::Runtime;
    ///
    /// let mut runtime = Runtime::new();
    /// ```
    pub fn new() -> Self {
        let output = crate::stdlib::stdout_writer();
        Self {
            globals: RefCell::new(HashMap::new()),
            security: SecurityContext::new(),
            execution_limits: RefCell::new(super::config::ExecutionLimits::unlimited()),
            accumulated_bytecode: RefCell::new(crate::bytecode::Bytecode::new()),
            output,
            native_signatures: RefCell::new(HashMap::new()),
        }
    }

    /// Create a new runtime with custom security context
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::Runtime;
    /// use atlas_runtime::SecurityContext;
    ///
    /// let security = SecurityContext::allow_all();
    /// let mut runtime = Runtime::new_with_security(security);
    /// ```
    pub fn new_with_security(security: SecurityContext) -> Self {
        let output = crate::stdlib::stdout_writer();
        Self {
            globals: RefCell::new(HashMap::new()),
            security,
            execution_limits: RefCell::new(super::config::ExecutionLimits::unlimited()),
            accumulated_bytecode: RefCell::new(crate::bytecode::Bytecode::new()),
            output,
            native_signatures: RefCell::new(HashMap::new()),
        }
    }

    /// Create a new runtime with configuration
    ///
    /// Converts RuntimeConfig into appropriate SecurityContext settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::{Runtime, RuntimeConfig};
    ///
    /// let config = RuntimeConfig::sandboxed();
    /// let mut runtime = Runtime::from_config(config);
    /// ```
    pub fn from_config(config: super::config::RuntimeConfig) -> Self {
        // Create security context based on config flags
        // IMPORTANT: allow_io and allow_network are SEPARATE permissions
        let mut security = SecurityContext::new(); // Deny-all by default

        if config.allow_io {
            // Grant filesystem permissions only (not network)
            #[cfg(not(windows))]
            {
                security.grant_filesystem_read(std::path::Path::new("/"), true);
                security.grant_filesystem_write(std::path::Path::new("/"), true);
            }
            #[cfg(windows)]
            {
                let system_drive =
                    std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
                let root = format!("{}\\", system_drive);
                security.grant_filesystem_read(std::path::Path::new(&root), true);
                security.grant_filesystem_write(std::path::Path::new(&root), true);
            }
            // Also grant process and environment access with IO
            security.grant_process("*");
            security.grant_environment("*");
        }

        if config.allow_network {
            // Grant network permissions separately
            security.grant_network("*");
        }

        // Create execution limits from config (timeout enforcement)
        let execution_limits = super::config::ExecutionLimits::from_config(&config);

        let output = config.output.clone();
        Self {
            globals: RefCell::new(HashMap::new()),
            security,
            execution_limits: RefCell::new(execution_limits),
            accumulated_bytecode: RefCell::new(crate::bytecode::Bytecode::new()),
            output,
            native_signatures: RefCell::new(HashMap::new()),
        }
    }

    /// Create a sandboxed runtime with restrictive defaults
    ///
    /// Disables IO and network operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::Runtime;
    ///
    /// let mut runtime = Runtime::sandboxed();
    /// // Attempts to use IO operations will fail
    /// ```
    pub fn sandboxed() -> Self {
        Self::from_config(super::config::RuntimeConfig::sandboxed())
    }

    /// Evaluate Atlas source code
    ///
    /// Runs the full compilation pipeline (lex → parse → bind → typecheck → execute)
    /// and returns the result value. State (globals, functions) persists across calls.
    ///
    /// # Arguments
    ///
    /// * `source` - Atlas source code to evaluate
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - Result of evaluation
    /// * `Err(EvalError)` - Parse, type, or runtime error
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::Runtime;
    ///
    /// let mut runtime = Runtime::new();
    /// let result = runtime.eval("1 + 2").unwrap();
    /// ```
    pub fn eval(&mut self, source: &str) -> Result<Value, EvalError> {
        // For REPL-style usage, if the source doesn't end with a semicolon,
        // treat it as an expression statement by appending one
        let source = source.trim();
        let source_with_semi =
            if !source.is_empty() && !source.ends_with(';') && !source.ends_with('}') {
                format!("{};", source)
            } else {
                source.to_string()
            };

        // Lex the source code
        let mut lexer = Lexer::new(&source_with_semi);
        let (tokens, lex_diagnostics) = lexer.tokenize();

        if !lex_diagnostics.is_empty() {
            return Err(EvalError::ParseError(lex_diagnostics));
        }

        // Parse tokens into AST
        let mut parser = Parser::new(tokens);
        let (ast, parse_diagnostics) = parser.parse();

        let parse_errors: Vec<_> = parse_diagnostics
            .into_iter()
            .filter(|d| d.is_error())
            .collect();
        if !parse_errors.is_empty() {
            return Err(EvalError::ParseError(parse_errors));
        }

        // Create initial symbol table with registered globals
        let mut initial_symbol_table = crate::symbol::SymbolTable::new();
        {
            let globals = self.globals.borrow();
            for (name, (value, is_mutable)) in globals.iter() {
                // Determine symbol kind based on value type
                let kind = match value {
                    Value::NativeFunction(_) | Value::Function(_) => {
                        crate::symbol::SymbolKind::Function
                    }
                    _ => crate::symbol::SymbolKind::Variable,
                };

                let ty = match value {
                    Value::NativeFunction(_) => self
                        .native_signatures
                        .borrow()
                        .get(name)
                        .and_then(|arity| {
                            arity.as_ref().map(|count| crate::types::Type::Function {
                                type_params: Vec::new(),
                                params: vec![crate::types::Type::any_placeholder(); *count],
                                return_type: Box::new(crate::types::Type::any_placeholder()),
                            })
                        })
                        .unwrap_or(crate::types::Type::Unknown),
                    _ => crate::types::Type::Unknown,
                };

                // Create symbol with placeholder type (runtime values don't have compile-time types)
                let symbol = crate::symbol::Symbol {
                    name: name.clone(),
                    ty,
                    mutable: *is_mutable,
                    kind: kind.clone(),
                    span: crate::span::Span::dummy(),
                    exported: false,
                    visibility: crate::ast::Visibility::Private,
                };

                // Add to initial symbol table
                if kind == crate::symbol::SymbolKind::Function {
                    let _ = initial_symbol_table.define_function(symbol);
                } else {
                    let _ = initial_symbol_table.define(symbol);
                }
            }
        }

        // Bind symbols with pre-populated symbol table
        let mut binder = Binder::with_symbol_table(initial_symbol_table);
        let (mut symbol_table, bind_diagnostics) = binder.bind(&ast);

        if !bind_diagnostics.is_empty() {
            return Err(EvalError::ParseError(bind_diagnostics));
        }

        // Type check
        let mut type_checker = TypeChecker::new(&mut symbol_table);
        let type_diagnostics = type_checker.check(&ast);

        // Only fail on errors, not warnings. Return ALL diagnostics (errors + warnings)
        // so the CLI can emit them in one pass via the proper formatter (H-196).
        let has_errors = type_diagnostics.iter().any(|d| d.is_error());
        if has_errors {
            return Err(EvalError::TypeError(type_diagnostics));
        }

        // Collect typecheck warnings — emit after execution alongside runtime warnings.
        let typecheck_warnings: Vec<crate::diagnostic::Diagnostic> = type_diagnostics
            .into_iter()
            .filter(|d| d.is_warning())
            .collect();

        // Start execution timer and prepare limits for sharing
        let execution_limits = {
            let mut limits = self.execution_limits.borrow_mut();
            limits.start();
            std::sync::Arc::new(limits.clone())
        };

        // Compile AST to bytecode (D-052: unified VM execution path)
        let mut compiler = Compiler::new();
        let new_bytecode = match compiler.compile(&ast) {
            Ok(bc) => bc,
            Err(diagnostics) => return Err(EvalError::ParseError(diagnostics)),
        };

        // Get the start offset of new code (before appending)
        let new_code_start = self.accumulated_bytecode.borrow().instructions.len();

        // Append to accumulated bytecode
        self.accumulated_bytecode.borrow_mut().append(new_bytecode);

        // Create VM with the accumulated bytecode
        let accumulated = self.accumulated_bytecode.borrow().clone();
        let mut vm = VM::new(accumulated);
        vm.set_output_writer(self.output.clone());

        // Set execution limits for timeout enforcement
        if execution_limits.is_active() {
            vm.set_execution_limits(execution_limits);
        }

        // Set IP to start of new code (so we don't re-execute old code)
        vm.set_ip(new_code_start);

        // Copy runtime globals to VM (for natives and other complex types)
        {
            let globals = self.globals.borrow();
            for (name, (value, _mutable)) in globals.iter() {
                vm.set_global(name.clone(), value.clone());
            }
        }

        // Load extern function declarations (FFI bindings)
        vm.load_extern_declarations(&ast)
            .map_err(EvalError::RuntimeError)?;

        let result = match vm.run(&self.security) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Ok(Value::Null),
            Err(e) => Err(EvalError::RuntimeError(e)),
        };

        // Collect and emit all warnings via the proper formatter (H-196)
        let mut all_warnings = typecheck_warnings;
        all_warnings.extend(vm.take_runtime_warnings());
        emit_warnings_via_formatter(&all_warnings);

        // Copy VM globals back to runtime for persistence across eval() calls
        // Note: VM doesn't track mutability, so we default to mutable for copied-back values
        {
            let mut globals = self.globals.borrow_mut();
            for (name, value) in vm.get_globals() {
                globals.insert(name.clone(), (value.clone(), true));
            }
        }

        result
    }

    /// Evaluate an Atlas file with full module support
    ///
    /// Loads the file and all its dependencies in topological order,
    /// executes them, and returns the entry module's result. Imports
    /// are properly resolved and executed.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Atlas file to execute
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - Result of execution
    /// * `Err(EvalError)` - Parse, type, or runtime error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use atlas_runtime::api::Runtime;
    /// use std::path::Path;
    ///
    /// let mut runtime = Runtime::new();
    /// let result = runtime.eval_file(Path::new("main.atlas")).unwrap();
    /// ```
    pub fn eval_file(&mut self, path: &Path) -> Result<Value, EvalError> {
        // Determine project root from file path
        let project_root = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        // D-052: unified VM execution path
        {
            // Step 1: Load all modules in dependency order
            let mut loader = ModuleLoader::new(project_root.clone());
            let modules = loader.load_module(path).map_err(EvalError::ParseError)?;

            // Step 2: Compile ALL modules to bytecode in dependency order
            // Modules are already in topological order (dependencies first),
            // so when module B imports from A, A's code will have already run
            // and defined its globals before B tries to access them.
            let mut combined_bytecode = crate::bytecode::Bytecode::new();

            let exports_by_path: HashMap<std::path::PathBuf, Vec<String>> = modules
                .iter()
                .map(|module| (module.path.clone(), module.exports.clone()))
                .collect();
            let mut resolver = ModuleResolver::new(project_root.clone());

            // Pass 1: Run binder + typechecker to populate AST annotations (type_tag, etc.)
            // Without this, MemberExpr.type_tag is None and compile_member falls back to
            // the GetField structural path, which fails for builtin namespaces (console, etc.).
            let mut module_registry = crate::module_loader::ModuleRegistry::new();
            let mut expanded_modules: Vec<Program> = Vec::new();
            for module in &modules {
                let expanded = self
                    .expand_namespace_imports(module, &exports_by_path, &mut resolver)
                    .map_err(|diag| EvalError::ParseError(vec![*diag]))?;
                let mut binder = Binder::new();
                binder.set_resolved_imports(module.resolved_imports.clone());
                let (mut symbol_table, _) =
                    binder.bind_with_modules(&expanded, &module.path, &module_registry);
                let mut type_checker = TypeChecker::new(&mut symbol_table);
                let _ = type_checker.check(&expanded); // populate annotations; ignore errors
                module_registry.register(module.path.clone(), symbol_table);
                expanded_modules.push(expanded);
            }

            for (i, (module, expanded)) in modules.iter().zip(expanded_modules.iter()).enumerate() {
                let is_last = i == modules.len() - 1;

                // Compile this module (AST already has type_tag annotations from Pass 1)
                let mut compiler = Compiler::new();
                compiler.register_imported_enums(&module.imports, &module.path, &module_registry);
                let mut module_bytecode =
                    compiler.compile(expanded).map_err(EvalError::ParseError)?;

                // Strip trailing Halt from non-final modules
                // (compiler adds Halt at end of each module, but we need
                // continuous execution until the entry module completes)
                if !is_last && !module_bytecode.instructions.is_empty() {
                    // Check if last instruction is Halt (0xFF)
                    if module_bytecode.instructions.last() == Some(&0xFF) {
                        module_bytecode.instructions.pop();
                        // Also remove corresponding debug info if present
                        if let Some(last_debug) = module_bytecode.debug_info.last() {
                            if last_debug.instruction_offset == module_bytecode.instructions.len() {
                                module_bytecode.debug_info.pop();
                            }
                        }
                    }
                }

                // Append to combined bytecode (this adjusts function offsets)
                combined_bytecode.append(module_bytecode);
            }

            // Step 3: Create VM and run combined bytecode
            let mut vm = VM::new(combined_bytecode);
            vm.set_output_writer(self.output.clone());

            // Load extern function declarations from all modules (FFI bindings)
            for module in &modules {
                vm.load_extern_declarations(&module.ast)
                    .map_err(EvalError::RuntimeError)?;
            }

            // Step 4: Execute via VM
            match vm.run(&self.security) {
                Ok(Some(value)) => Ok(value),
                Ok(None) => Ok(Value::Null),
                Err(e) => Err(EvalError::RuntimeError(e)),
            }
        }
    }

    /// Load an Atlas file into the runtime for subsequent `eval()` calls
    ///
    /// This method loads a file with full import resolution (like `eval_file`),
    /// but persists the defined functions and globals so they can be called
    /// via subsequent `eval()` calls. This is useful for test frameworks that
    /// need to load a test file and then call specific test functions.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Atlas file to load
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File loaded successfully, functions available for `eval()`
    /// * `Err(EvalError)` - Parse, type, or runtime error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use atlas_runtime::api::Runtime;
    /// use std::path::Path;
    ///
    /// let mut runtime = Runtime::new();
    /// // Load file with imports resolved
    /// runtime.load_file(Path::new("tests/math.test.atl")).unwrap();
    /// // Call a function defined in the file
    /// runtime.eval("test_add()").unwrap();
    /// ```
    pub fn load_file(&mut self, path: &Path) -> Result<(), EvalError> {
        // Determine project root from file path
        let project_root = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        // Step 1: Load all modules in dependency order
        let mut loader = ModuleLoader::new(project_root.clone());
        let modules = loader.load_module(path).map_err(EvalError::ParseError)?;

        let exports_by_path: HashMap<std::path::PathBuf, Vec<String>> = modules
            .iter()
            .map(|module| (module.path.clone(), module.exports.clone()))
            .collect();
        let mut resolver = ModuleResolver::new(project_root.clone());

        // Build module registry for cross-module import resolution
        let mut module_registry = crate::module_loader::ModuleRegistry::new();

        // ═══════════════════════════════════════════════════════════════════════
        // PASS 1: Bind and typecheck ALL modules — collect ALL errors.
        // This ensures type_tags are set for namespace calls like test.assert().
        // ═══════════════════════════════════════════════════════════════════════
        let mut all_errors: Vec<Diagnostic> = Vec::new();
        let mut expanded_modules: Vec<Program> = Vec::new();

        for module in &modules {
            // Expand namespace imports (import * as foo)
            let expanded =
                match self.expand_namespace_imports(module, &exports_by_path, &mut resolver) {
                    Ok(e) => e,
                    Err(diag) => {
                        all_errors.push(*diag);
                        expanded_modules.push(Program { items: vec![] });
                        continue;
                    }
                };

            // Bind symbols with cross-module import support
            let mut binder = Binder::new();
            binder.set_resolved_imports(module.resolved_imports.clone());
            let (mut symbol_table, bind_diags) =
                binder.bind_with_modules(&expanded, &module.path, &module_registry);
            let bind_errors: Vec<_> = bind_diags
                .iter()
                .filter(|d| d.is_error())
                .cloned()
                .collect();
            all_errors.extend(bind_errors);

            // Type-check even if bind had errors — collect ALL diagnostics
            let mut type_checker = TypeChecker::new(&mut symbol_table);
            let type_diags = type_checker.check(&expanded);
            let type_errors: Vec<_> = type_diags
                .iter()
                .filter(|d| d.is_error())
                .cloned()
                .collect();
            all_errors.extend(type_errors);

            // Register this module's symbol table for subsequent imports
            module_registry.register(module.path.clone(), symbol_table);
            expanded_modules.push(expanded);
        }

        // If ANY module had errors, return ALL errors now (before compilation)
        if !all_errors.is_empty() {
            return Err(EvalError::ParseError(all_errors));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // PASS 2: Compile ALL modules to bytecode in dependency order.
        // ═══════════════════════════════════════════════════════════════════════
        let mut combined_bytecode = crate::bytecode::Bytecode::new();

        for (i, expanded) in expanded_modules.iter().enumerate() {
            let is_last = i == modules.len() - 1;

            // Compile this module
            let mut compiler = Compiler::new();
            let mut module_bytecode = compiler.compile(expanded).map_err(EvalError::ParseError)?;

            // Strip trailing Halt from non-final modules
            if !is_last
                && !module_bytecode.instructions.is_empty()
                && module_bytecode.instructions.last() == Some(&0xFF)
            {
                module_bytecode.instructions.pop();
                if let Some(last_debug) = module_bytecode.debug_info.last() {
                    if last_debug.instruction_offset == module_bytecode.instructions.len() {
                        module_bytecode.debug_info.pop();
                    }
                }
            }

            combined_bytecode.append(module_bytecode);
        }

        // Strip trailing Halt from final module too (we need to continue execution)
        if !combined_bytecode.instructions.is_empty()
            && combined_bytecode.instructions.last() == Some(&0xFF)
        {
            combined_bytecode.instructions.pop();
        }

        // Step 3: Get start offset and append to accumulated bytecode
        let new_code_start = self.accumulated_bytecode.borrow().instructions.len();
        self.accumulated_bytecode
            .borrow_mut()
            .append(combined_bytecode);

        // Step 4: Create VM with accumulated bytecode, set IP to new code
        let accumulated = self.accumulated_bytecode.borrow().clone();
        let mut vm = VM::new(accumulated);
        vm.set_output_writer(self.output.clone());
        vm.set_ip(new_code_start);

        // Copy runtime globals to VM (for natives and other complex types)
        {
            let globals = self.globals.borrow();
            for (name, (value, _mutable)) in globals.iter() {
                vm.set_global(name.clone(), value.clone());
            }
        }

        // Load extern function declarations from all modules (FFI bindings)
        for module in &modules {
            vm.load_extern_declarations(&module.ast)
                .map_err(EvalError::RuntimeError)?;
        }

        // Step 5: Execute the loaded code
        if let Err(e) = vm.run(&self.security) {
            return Err(EvalError::RuntimeError(e));
        }

        // Step 6: Copy VM globals back to runtime for subsequent eval() calls
        {
            let mut globals = self.globals.borrow_mut();
            for (name, value) in vm.get_globals() {
                globals.insert(name.clone(), (value.clone(), true));
            }
        }

        Ok(())
    }

    fn expand_namespace_imports(
        &self,
        module: &crate::module_loader::LoadedModule,
        exports_by_path: &HashMap<std::path::PathBuf, Vec<String>>,
        resolver: &mut ModuleResolver,
    ) -> Result<Program, Box<Diagnostic>> {
        let mut items = Vec::new();

        for item in &module.ast.items {
            items.push(item.clone());

            let Item::Import(import_decl) = item else {
                continue;
            };

            for specifier in &import_decl.specifiers {
                let ImportSpecifier::Namespace { alias, span } = specifier else {
                    continue;
                };

                let import_path = resolver
                    .resolve_path(&import_decl.source, &module.path, import_decl.span)
                    .map_err(Box::new)?;
                let exports = exports_by_path.get(&import_path).ok_or_else(|| {
                    Box::new(
                        IMPORT_RESOLUTION_FAILED
                            .emit(import_decl.span)
                            .arg("path", &import_decl.source)
                            .arg("detail", "module not found")
                            .build()
                            .with_label("namespace import"),
                    )
                })?;

                let mut entries = Vec::with_capacity(exports.len());
                for name in exports {
                    let ident = Identifier {
                        name: name.clone(),
                        span: *span,
                    };
                    entries.push(ObjectEntry {
                        key: ident.clone(),
                        value: Expr::Identifier(ident),
                        span: *span,
                    });
                }

                let obj = ObjectLiteral {
                    entries,
                    span: *span,
                };
                let var = VarDecl {
                    mutable: false,
                    uses_deprecated_var: false,
                    name: alias.clone(),
                    type_ref: None,
                    init: Expr::ObjectLiteral(obj),
                    span: *span,
                    needs_drop: std::cell::RefCell::new(None),
                };
                items.push(Item::Statement(Stmt::VarDecl(var)));
            }
        }

        Ok(Program { items })
    }

    /// Call an Atlas function by name with arguments
    ///
    /// Looks up the function in global scope and executes it with provided arguments.
    /// Both user-defined and builtin functions can be called.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name to call
    /// * `args` - Vector of argument values
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - Function return value
    /// * `Err(EvalError)` - Runtime error (function not found, arity mismatch, etc.)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use atlas_runtime::api::Runtime;
    /// use atlas_runtime::value::Value;
    ///
    /// let mut runtime = Runtime::new();
    /// runtime.eval("fn add(borrow x: number, borrow y: number): number { x + y }").ok();
    ///
    /// let result = runtime.call("add", vec![Value::Number(1.0), Value::Number(2.0)]);
    /// ```
    pub fn call(&mut self, name: &str, args: Vec<Value>) -> Result<Value, EvalError> {
        // Build a source string that calls the function
        // This is a simple approach that leverages existing eval infrastructure
        let mut args_code = Vec::with_capacity(args.len());
        for value in &args {
            let rendered = match value {
                Value::Number(n) => n.to_string(),
                Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                _ => {
                    return Err(EvalError::RuntimeError(RuntimeError::TypeError {
                        msg: "Unsupported argument type for call()".to_string(),
                        span: Span::dummy(),
                    }))
                }
            };
            args_code.push(rendered);
        }
        let args_code = args_code.join(", ");

        let call_source = format!("{}({})", name, args_code);
        self.eval(&call_source)
    }

    /// Set a global variable
    ///
    /// Creates or updates a global variable in the runtime state.
    /// The variable will be accessible in subsequent evaluations.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `value` - Value to assign
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::Runtime;
    /// use atlas_runtime::Value;
    ///
    /// let mut runtime = Runtime::new();
    /// runtime.set_global("x", Value::Number(42.0));
    ///
    /// let result = runtime.eval("x").unwrap();
    /// ```
    pub fn set_global(&mut self, name: &str, value: Value) {
        // Store in runtime globals - eval() copies these to VM before execution
        let mut globals = self.globals.borrow_mut();
        globals.insert(name.to_string(), (value, true));
    }

    /// Get a global variable
    ///
    /// Retrieves the current value of a global variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    ///
    /// # Returns
    ///
    /// * `Some(Value)` - Variable value if it exists
    /// * `None` - Variable not found
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::Runtime;
    /// use atlas_runtime::Value;
    ///
    /// let mut runtime = Runtime::new();
    /// runtime.eval("let x: number = 42;").expect("eval failed");
    ///
    /// let value = runtime.get_global("x");
    /// ```
    pub fn get_global(&self, name: &str) -> Option<Value> {
        // Read from runtime globals - eval() copies VM globals back here after execution
        let globals = self.globals.borrow();
        globals.get(name).map(|(v, _)| v.clone())
    }

    /// Register a native function with fixed arity
    ///
    /// Registers a Rust closure as a callable function in Atlas code. The function
    /// will be available globally and can be called like any Atlas function.
    ///
    /// The function's arity (argument count) is validated automatically - calls with
    /// the wrong number of arguments will result in a runtime error.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name (how it will be called from Atlas)
    /// * `arity` - Required number of arguments
    /// * `implementation` - Rust closure implementing the function
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::Runtime;
    /// use atlas_runtime::value::{Value, RuntimeError};
    /// use atlas_runtime::span::Span;
    ///
    /// let mut runtime = Runtime::new();
    ///
    /// // Register a native "add" function
    /// runtime.register_function("add", 2, |args| {
    ///     let a = match &args[0] {
    ///         Value::Number(n) => *n,
    ///         _ => return Err(RuntimeError::TypeError {
    ///             msg: "Expected number".to_string(),
    ///             span: Span::dummy()
    ///         }),
    ///     };
    ///     let b = match &args[1] {
    ///         Value::Number(n) => *n,
    ///         _ => return Err(RuntimeError::TypeError {
    ///             msg: "Expected number".to_string(),
    ///             span: Span::dummy()
    ///         }),
    ///     };
    ///     Ok(Value::Number(a + b))
    /// });
    ///
    /// // Call from Atlas code
    /// let result = runtime.eval("add(10, 20)");
    /// ```
    pub fn register_function<F>(&mut self, name: &str, arity: usize, implementation: F)
    where
        F: Fn(&[Value]) -> Result<Value, RuntimeError> + Send + Sync + 'static,
    {
        let native_fn = match crate::api::native::NativeFunctionBuilder::new(name)
            .with_arity(arity)
            .with_implementation(implementation)
            .build()
        {
            Ok(value) => value,
            Err(err) => {
                let msg = format!("Failed to build native function '{}': {}", name, err);
                let fallback: crate::value::NativeFn = std::sync::Arc::new(move |_args| {
                    Err(RuntimeError::InternalError {
                        msg: msg.clone(),
                        span: Span::dummy(),
                    })
                });
                Value::NativeFunction(fallback)
            }
        };

        self.native_signatures
            .borrow_mut()
            .insert(name.to_string(), Some(arity));
        self.set_global(name, native_fn);
    }

    /// Register a variadic native function
    ///
    /// Registers a Rust closure as a callable function that accepts any number of arguments.
    /// The implementation is responsible for validating the argument count and types.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name (how it will be called from Atlas)
    /// * `implementation` - Rust closure implementing the function
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::api::Runtime;
    /// use atlas_runtime::value::{Value, RuntimeError};
    /// use atlas_runtime::span::Span;
    ///
    /// let mut runtime = Runtime::new();
    ///
    /// // Register a variadic "sum" function
    /// runtime.register_variadic("sum", |args| {
    ///     let mut total = 0.0;
    ///     for arg in args {
    ///         match arg {
    ///             Value::Number(n) => total += n,
    ///             _ => return Err(RuntimeError::TypeError {
    ///                 msg: "All arguments must be numbers".to_string(),
    ///                 span: Span::dummy()
    ///             }),
    ///         }
    ///     }
    ///     Ok(Value::Number(total))
    /// });
    ///
    /// // Call with any number of arguments
    /// let result = runtime.eval("sum(1, 2, 3, 4, 5)");
    /// ```
    pub fn register_variadic<F>(&mut self, name: &str, implementation: F)
    where
        F: Fn(&[Value]) -> Result<Value, RuntimeError> + Send + Sync + 'static,
    {
        let native_fn = match crate::api::native::NativeFunctionBuilder::new(name)
            .variadic()
            .with_implementation(implementation)
            .build()
        {
            Ok(value) => value,
            Err(err) => {
                let msg = format!("Failed to build native function '{}': {}", name, err);
                let fallback: crate::value::NativeFn = std::sync::Arc::new(move |_args| {
                    Err(RuntimeError::InternalError {
                        msg: msg.clone(),
                        span: Span::dummy(),
                    })
                });
                Value::NativeFunction(fallback)
            }
        };

        self.native_signatures
            .borrow_mut()
            .insert(name.to_string(), None);
        self.set_global(name, native_fn);
    }
}
