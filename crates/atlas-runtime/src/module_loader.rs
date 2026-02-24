//! Module Loading and Caching
//!
//! Loads module files, builds dependency graphs, and returns modules in topological order.
//! This is BLOCKER 04-B - loading and caching only.
//! Type checking happens in BLOCKER 04-C.

use crate::ast::{ImportDecl, Item, Program};
use crate::diagnostic::Diagnostic;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::resolver::ModuleResolver;
use crate::span::Span;
use crate::symbol::SymbolTable;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

/// A loaded module with its AST and metadata
#[derive(Debug, Clone)]
pub struct LoadedModule {
    /// Absolute path to the module file
    pub path: PathBuf,
    /// Parsed AST
    pub ast: Program,
    /// List of exported names (for validation in 04-C)
    pub exports: Vec<String>,
    /// List of import declarations (for dependency tracking)
    pub imports: Vec<ImportDecl>,
}

/// Registry of bound modules with their symbol tables
///
/// Used during binding and type checking to resolve cross-module references.
/// This is BLOCKER 04-C - cross-module type checking.
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    /// Map of module path -> symbol table
    modules: HashMap<PathBuf, SymbolTable>,
}

impl ModuleRegistry {
    /// Create a new empty module registry
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Register a module's symbol table
    pub fn register(&mut self, path: PathBuf, symbol_table: SymbolTable) {
        self.modules.insert(path, symbol_table);
    }

    /// Get a module's symbol table
    pub fn get(&self, path: &Path) -> Option<&SymbolTable> {
        self.modules.get(path)
    }

    /// Get a mutable reference to a module's symbol table
    pub fn get_mut(&mut self, path: &Path) -> Option<&mut SymbolTable> {
        self.modules.get_mut(path)
    }

    /// Check if a module is registered
    pub fn contains(&self, path: &Path) -> bool {
        self.modules.contains_key(path)
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Module loader - loads files, builds dependency graphs, performs topological sort
pub struct ModuleLoader {
    /// Module resolver for path resolution
    resolver: ModuleResolver,
    /// Cache of loaded modules (by absolute path)
    cache: HashMap<PathBuf, LoadedModule>,
    /// Dependency graph (module -> its dependencies)
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
    /// Modules currently being loaded (for cycle detection during loading)
    loading: HashSet<PathBuf>,
}

impl ModuleLoader {
    /// Create a new module loader with the given project root
    pub fn new(root: PathBuf) -> Self {
        Self {
            resolver: ModuleResolver::new(root),
            cache: HashMap::new(),
            dependencies: HashMap::new(),
            loading: HashSet::new(),
        }
    }

    /// Load a module and all its dependencies
    ///
    /// Returns modules in topological order (dependencies first).
    ///
    /// # Arguments
    /// * `entry_point` - Absolute path to the entry module
    ///
    /// # Returns
    /// List of modules in initialization order, or diagnostics if errors occurred
    pub fn load_module(
        &mut self,
        entry_point: &Path,
    ) -> Result<Vec<LoadedModule>, Vec<Diagnostic>> {
        // Load the entry module and all dependencies recursively
        self.load_recursive(entry_point)?;

        // Check for circular dependencies
        self.resolver
            .check_circular(entry_point, Span::dummy())
            .map_err(|e| vec![e])?;

        // Return modules in topological order
        let ordered = self.topological_sort(entry_point)?;

        // Convert paths to loaded modules
        let modules = ordered
            .into_iter()
            .map(|path| {
                self.cache
                    .get(&path)
                    .expect("module should exist in cache after loading")
                    .clone()
            })
            .collect();

        Ok(modules)
    }

    /// Recursively load a module and its dependencies
    fn load_recursive(&mut self, module_path: &Path) -> Result<(), Vec<Diagnostic>> {
        let abs_path = module_path.to_path_buf();

        // Check cache - if already loaded, skip
        if self.cache.contains_key(&abs_path) {
            return Ok(());
        }

        // Check if currently being loaded (circular dependency)
        if self.loading.contains(&abs_path) {
            return Err(vec![Diagnostic::error_with_code(
                "AT5003",
                "Circular dependency detected",
                Span::dummy(),
            )
            .with_label(format!("module: {}", abs_path.display()))
            .with_help(
                "Refactor to remove circular dependencies between modules".to_string(),
            )]);
        }

        // Mark as currently loading
        self.loading.insert(abs_path.clone());

        // Load and parse the module file
        let loaded = self.load_and_parse(&abs_path)?;

        // Extract dependencies from imports (deduplicate)
        let mut deps = Vec::new();
        let mut seen_deps = HashSet::new();

        for import in &loaded.imports {
            // Resolve import path relative to current module
            let dep_path = self
                .resolver
                .resolve_path(&import.source, &abs_path, import.span)
                .map_err(|e| vec![e])?;

            // Skip if already processed (multiple imports from same module)
            if !seen_deps.insert(dep_path.clone()) {
                continue;
            }

            deps.push(dep_path.clone());

            // Add to resolver's dependency graph
            self.resolver
                .add_dependency(abs_path.clone(), dep_path.clone());

            // Recursively load the dependency
            self.load_recursive(&dep_path)?;
        }

        // Store dependencies in our graph
        self.dependencies.insert(abs_path.clone(), deps);

        // Cache the loaded module
        self.cache.insert(abs_path.clone(), loaded);

        // Remove from loading set (done loading)
        self.loading.remove(&abs_path);

        Ok(())
    }

    /// Load and parse a single module file
    fn load_and_parse(&self, path: &Path) -> Result<LoadedModule, Vec<Diagnostic>> {
        // Read file contents
        let source = fs::read_to_string(path).map_err(|e| {
            vec![Diagnostic::error_with_code(
                "AT5002",
                format!("Failed to read module file: {}", e),
                Span::dummy(),
            )
            .with_label(format!("path: {}", path.display()))
            .with_help("ensure the file exists and you have read permissions")]
        })?;

        // Lex
        let mut lexer = Lexer::new(&source);
        let (tokens, lex_diags) = lexer.tokenize();
        if !lex_diags.is_empty() {
            return Err(lex_diags);
        }

        // Parse
        let mut parser = Parser::new(tokens);
        let (ast, parse_diags) = parser.parse();
        if !parse_diags.is_empty() {
            return Err(parse_diags);
        }

        // Extract exports and imports
        let mut exports = Vec::new();
        let mut imports = Vec::new();

        for item in &ast.items {
            match item {
                Item::Export(export_decl) => {
                    let name = match &export_decl.item {
                        crate::ast::ExportItem::Function(func) => func.name.name.clone(),
                        crate::ast::ExportItem::Variable(var) => var.name.name.clone(),
                        crate::ast::ExportItem::TypeAlias(alias) => alias.name.name.clone(),
                    };
                    exports.push(name);
                }
                Item::Import(import_decl) => {
                    imports.push(import_decl.clone());
                }
                _ => {}
            }
        }

        Ok(LoadedModule {
            path: path.to_path_buf(),
            ast,
            exports,
            imports,
        })
    }

    /// Perform topological sort to get initialization order
    ///
    /// Returns modules in dependency order (dependencies before dependents).
    /// Uses Kahn's algorithm.
    /// Only includes modules reachable from the entry point.
    fn topological_sort(&self, entry: &Path) -> Result<Vec<PathBuf>, Vec<Diagnostic>> {
        // First, find all modules reachable from entry using DFS
        let reachable = self.find_reachable(entry);

        // Build in-degree map (count of incoming edges) for reachable nodes only
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();

        // Initialize in-degrees for reachable nodes
        for node in &reachable {
            in_degree.insert(node.clone(), 0);
        }

        // Calculate in-degrees (only for reachable nodes)
        for from in &reachable {
            if let Some(deps) = self.dependencies.get(from) {
                for _dep in deps {
                    if reachable.contains(_dep) {
                        *in_degree
                            .get_mut(from)
                            .expect("in_degree should contain all reachable nodes") += 1;
                    }
                }
            }
        }

        // Queue of nodes with no incoming edges
        let mut queue: VecDeque<PathBuf> = VecDeque::new();
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        // Process nodes in topological order
        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(node.clone());

            // For each dependent of this node (in reachable set)
            for from in &reachable {
                if let Some(deps) = self.dependencies.get(from) {
                    if deps.contains(&node) {
                        // Decrease in-degree
                        let degree = in_degree
                            .get_mut(from)
                            .expect("in_degree should contain all reachable nodes");
                        *degree -= 1;

                        // If no more dependencies, add to queue
                        if *degree == 0 {
                            queue.push_back(from.clone());
                        }
                    }
                }
            }
        }

        // Check if all reachable nodes were processed (no cycles)
        if sorted.len() != reachable.len() {
            return Err(vec![Diagnostic::error_with_code(
                "AT5003",
                "Circular dependency detected during topological sort",
                Span::dummy(),
            )
            .with_help("refactor your modules to remove circular imports - modules cannot import each other in a cycle")]);
        }

        Ok(sorted)
    }

    /// Find all modules reachable from a given entry point using DFS
    fn find_reachable(&self, entry: &Path) -> HashSet<PathBuf> {
        let mut reachable = HashSet::new();
        let mut stack = vec![entry.to_path_buf()];

        while let Some(node) = stack.pop() {
            if reachable.insert(node.clone()) {
                // If this is a new node, explore its dependencies
                if let Some(deps) = self.dependencies.get(&node) {
                    for dep in deps {
                        stack.push(dep.clone());
                    }
                }
            }
        }

        reachable
    }

    /// Get a loaded module from cache
    pub fn get_module(&self, path: &Path) -> Option<&LoadedModule> {
        self.cache.get(path)
    }

    /// Clear all caches (for testing)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.cache.clear();
        self.dependencies.clear();
        self.loading.clear();
        self.resolver.clear();
    }
}
