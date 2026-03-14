//! Symbol table and name binding

use crate::ast::{ConstDecl, EnumDecl, ImplMethod, StructDecl, TypeAliasDecl, Visibility};
use crate::span::Span;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol type
    pub ty: Type,
    /// Whether the symbol is mutable
    pub mutable: bool,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Declaration location
    pub span: Span,
    /// Whether this symbol is exported (for module system)
    pub exported: bool,
    /// Visibility modifier (pub/private/internal) — B37-P04
    pub visibility: Visibility,
}

/// Symbol classification
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// Variable binding
    Variable,
    /// Function binding
    Function,
    /// Parameter binding
    Parameter,
    /// Builtin function
    Builtin,
    /// Compile-time constant
    Const,
}

/// Symbol table for name resolution
#[derive(Clone, Debug)]
pub struct SymbolTable {
    /// Stack of scopes (innermost last)
    scopes: Vec<HashMap<String, Symbol>>,
    /// Top-level hoisted functions
    functions: HashMap<String, Symbol>,
    /// Type alias declarations (name -> alias)
    type_aliases: HashMap<String, TypeAliasDecl>,
    /// Exported type alias names
    type_alias_exports: HashSet<String>,
    /// Exported struct declarations (name -> StructDecl)
    struct_exports: HashMap<String, StructDecl>,
    /// Exported enum declarations (name -> EnumDecl)
    enum_exports: HashMap<String, EnumDecl>,
    /// Exported inherent impl method signatures: (struct_name, method_name) -> ImplMethod.
    /// Populated after typechecking; imported by consuming modules' typecheckers.
    impl_method_exports: HashMap<(String, String), ImplMethod>,
    /// Compile-time constant declarations (name -> ConstDecl)
    const_decls: HashMap<String, ConstDecl>,
    /// Exported const names
    const_exports: HashSet<String>,
}

impl SymbolTable {
    /// Create a new symbol table (no bare globals — use namespace.method() or value.method())
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            type_aliases: HashMap::new(),
            type_alias_exports: HashSet::new(),
            struct_exports: HashMap::new(),
            enum_exports: HashMap::new(),
            impl_method_exports: HashMap::new(),
            const_decls: HashMap::new(),
            const_exports: HashSet::new(),
        }
    }
    /// Define a type alias in the current module
    pub fn define_type_alias(
        &mut self,
        alias: TypeAliasDecl,
    ) -> Result<(), Box<(String, Option<TypeAliasDecl>)>> {
        if let Some(existing) = self.type_aliases.get(&alias.name.name) {
            return Err(Box::new((
                format!("Type alias '{}' already defined", alias.name.name),
                Some(existing.clone()),
            )));
        }
        self.type_aliases.insert(alias.name.name.clone(), alias);
        Ok(())
    }

    /// Look up a type alias by name
    pub fn get_type_alias(&self, name: &str) -> Option<&TypeAliasDecl> {
        self.type_aliases.get(name)
    }

    /// Get all type aliases
    pub fn type_aliases(&self) -> &HashMap<String, TypeAliasDecl> {
        &self.type_aliases
    }

    /// Mark a type alias as exported
    pub fn mark_type_alias_exported(&mut self, name: &str) -> bool {
        if self.type_aliases.contains_key(name) {
            self.type_alias_exports.insert(name.to_string());
            true
        } else {
            false
        }
    }

    /// Define a compile-time constant in the current module
    pub fn define_const(
        &mut self,
        decl: ConstDecl,
    ) -> Result<(), Box<(String, Option<ConstDecl>)>> {
        if let Some(existing) = self.const_decls.get(&decl.name.name) {
            return Err(Box::new((
                format!("Constant '{}' already defined", decl.name.name),
                Some(existing.clone()),
            )));
        }
        // Also register as a Symbol so lookup() finds it
        let symbol = Symbol {
            name: decl.name.name.clone(),
            ty: Type::Unknown, // Will be resolved during typechecking
            mutable: false,
            kind: SymbolKind::Const,
            span: decl.name.span,
            exported: false,
            visibility: Visibility::Private,
        };
        // Add to global scope (scopes[0])
        if let Some(global) = self.scopes.first_mut() {
            global.insert(decl.name.name.clone(), symbol);
        }
        self.const_decls.insert(decl.name.name.clone(), decl);
        Ok(())
    }

    /// Look up a const declaration by name
    pub fn get_const(&self, name: &str) -> Option<&ConstDecl> {
        self.const_decls.get(name)
    }

    /// Get all const declarations
    pub fn const_decls(&self) -> &HashMap<String, ConstDecl> {
        &self.const_decls
    }

    /// Mark a const as exported
    pub fn mark_const_exported(&mut self, name: &str) -> bool {
        if self.const_decls.contains_key(name) {
            self.const_exports.insert(name.to_string());
            true
        } else {
            false
        }
    }

    /// Add an exported struct declaration
    pub fn add_struct_export(&mut self, decl: StructDecl) {
        self.struct_exports.insert(decl.name.name.clone(), decl);
    }

    /// Add an exported enum declaration
    pub fn add_enum_export(&mut self, decl: EnumDecl) {
        self.enum_exports.insert(decl.name.name.clone(), decl);
    }

    /// Get all exported struct declarations
    pub fn get_struct_exports(&self) -> &HashMap<String, StructDecl> {
        &self.struct_exports
    }

    /// Get all exported enum declarations
    pub fn get_enum_exports(&self) -> &HashMap<String, EnumDecl> {
        &self.enum_exports
    }

    /// Add an exported inherent impl method (populated after typechecking).
    pub fn add_impl_method_export(
        &mut self,
        struct_name: String,
        method_name: String,
        method: ImplMethod,
    ) {
        self.impl_method_exports
            .insert((struct_name, method_name), method);
    }

    /// Get all exported impl method signatures.
    pub fn get_impl_method_exports(&self) -> &HashMap<(String, String), ImplMethod> {
        &self.impl_method_exports
    }

    /// Get exported type aliases
    pub fn get_type_alias_exports(&self) -> HashMap<String, TypeAliasDecl> {
        self.type_alias_exports
            .iter()
            .filter_map(|name| {
                self.type_aliases
                    .get(name)
                    .cloned()
                    .map(|alias| (name.clone(), alias))
            })
            .collect()
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a symbol in the current scope
    /// Returns Err with existing symbol if symbol already exists in current scope
    pub fn define(&mut self, symbol: Symbol) -> Result<(), Box<(String, Option<Symbol>)>> {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(existing) = scope.get(&symbol.name) {
                return Err(Box::new((
                    format!("Symbol '{}' is already defined in this scope", symbol.name),
                    Some(existing.clone()),
                )));
            }
            scope.insert(symbol.name.clone(), symbol);
            Ok(())
        } else {
            Err(Box::new(("No scope to define symbol in".to_string(), None)))
        }
    }

    /// Define a top-level function (hoisted)
    /// Returns Err with existing symbol if function already exists
    pub fn define_function(&mut self, symbol: Symbol) -> Result<(), Box<(String, Option<Symbol>)>> {
        if let Some(existing) = self.functions.get(&symbol.name) {
            return Err(Box::new((
                format!("Function '{}' is already defined", symbol.name),
                Some(existing.clone()),
            )));
        }
        self.functions.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Define a scoped function (nested function, not hoisted)
    ///
    /// This defines a function in the current scope on the stack, rather than
    /// in the global functions table. Nested functions are not hoisted and
    /// follow normal lexical scoping rules.
    ///
    /// Returns Err with existing symbol if name already exists in current scope
    pub fn define_scoped_function(
        &mut self,
        symbol: Symbol,
    ) -> Result<(), Box<(String, Option<Symbol>)>> {
        // Define in current scope (not global functions HashMap)
        // This allows nested functions to shadow outer functions and follow
        // lexical scoping rules
        self.define(symbol)
    }

    /// Look up a symbol in all scopes (innermost first, then functions)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Check local scopes first (innermost to outermost)
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }

        // Check top-level functions (hoisted)
        self.functions.get(name)
    }

    /// Look up a symbol mutably in all scopes (innermost first, then functions)
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        // Check local scopes first (innermost to outermost)
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                return scope.get_mut(name);
            }
        }

        // Check top-level functions (hoisted)
        self.functions.get_mut(name)
    }

    /// Look up a symbol mutably in the current (innermost) scope only.
    /// Returns `None` if the symbol exists only in an outer scope or not at all.
    pub fn lookup_current_scope_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.scopes.last_mut()?.get_mut(name)
    }

    /// Returns true if the symbol is defined in the current (innermost) scope.
    pub fn is_defined_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last().is_some_and(|s| s.contains_key(name))
    }

    /// Check if a name is a prelude builtin
    pub fn is_prelude_builtin(&self, name: &str) -> bool {
        if let Some(symbol) = self.functions.get(name) {
            symbol.kind == SymbolKind::Builtin
        } else {
            false
        }
    }

    /// Check if we're currently in the global scope
    pub fn is_global_scope(&self) -> bool {
        self.scopes.len() == 1
    }

    /// Get all symbols from all scopes and functions
    /// Returns a vector of all symbols in the table
    /// Collect all in-scope names for typo suggestion (edit-distance engine input).
    /// Includes variables, user functions, and builtins — O(n) over the scope stack.
    pub fn all_names_for_suggestion(&self) -> Vec<&str> {
        let mut names = Vec::new();
        for scope in &self.scopes {
            for name in scope.keys() {
                names.push(name.as_str());
            }
        }
        for name in self.functions.keys() {
            names.push(name.as_str());
        }
        names
    }

    pub fn all_symbols(&self) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        // Collect from all scopes
        for scope in &self.scopes {
            for symbol in scope.values() {
                symbols.push(symbol.clone());
            }
        }

        // Collect from functions (excluding builtins for cleaner output)
        for symbol in self.functions.values() {
            if symbol.kind != SymbolKind::Builtin {
                symbols.push(symbol.clone());
            }
        }

        symbols
    }

    /// Merge another symbol table into this one (for REPL state persistence)
    ///
    /// Adds new symbols from the other table to the top-level scope.
    /// Overwrites existing symbols with the same name.
    /// Does not merge nested scopes (only top-level scope and functions).
    pub fn merge(&mut self, other: SymbolTable) {
        // Merge top-level scope (index 0)
        if let Some(other_top_scope) = other.scopes.first() {
            if let Some(self_top_scope) = self.scopes.first_mut() {
                for (name, symbol) in other_top_scope {
                    self_top_scope.insert(name.clone(), symbol.clone());
                }
            }
        }

        // Merge functions (overwrite existing)
        for (name, symbol) in other.functions {
            // Don't overwrite builtins
            if symbol.kind != SymbolKind::Builtin {
                self.functions.insert(name, symbol);
            }
        }
    }

    /// Get all top-level symbols from this symbol table (including private ones)
    ///
    /// Used for cross-file visibility checking (H-313). When a symbol is not
    /// exported, we check if it exists as private to give a better error message.
    pub fn get_all_top_level_symbols(&self) -> HashMap<String, Symbol> {
        let mut symbols = HashMap::new();

        // Check top-level scope
        if let Some(top_scope) = self.scopes.first() {
            for (name, symbol) in top_scope {
                symbols.insert(name.clone(), symbol.clone());
            }
        }

        // Check top-level functions
        for (name, symbol) in &self.functions {
            if symbol.kind != SymbolKind::Builtin {
                symbols.insert(name.clone(), symbol.clone());
            }
        }

        symbols
    }

    /// Get all exported symbols from this symbol table
    ///
    /// Returns symbols marked as exported (for module system)
    pub fn get_exports(&self) -> HashMap<String, Symbol> {
        let mut exports = HashMap::new();

        // Check top-level scope for exported symbols
        if let Some(top_scope) = self.scopes.first() {
            for (name, symbol) in top_scope {
                if symbol.exported {
                    exports.insert(name.clone(), symbol.clone());
                }
            }
        }

        // Check top-level functions for exported symbols
        for (name, symbol) in &self.functions {
            if symbol.exported && symbol.kind != SymbolKind::Builtin {
                exports.insert(name.clone(), symbol.clone());
            }
        }

        exports
    }

    /// Mark a symbol as exported
    ///
    /// Used by binder when processing export declarations
    pub fn mark_exported(&mut self, name: &str) -> bool {
        // Check top-level scope first
        if let Some(top_scope) = self.scopes.first_mut() {
            if let Some(symbol) = top_scope.get_mut(name) {
                symbol.exported = true;
                return true;
            }
        }

        // Check top-level functions
        if let Some(symbol) = self.functions.get_mut(name) {
            symbol.exported = true;
            return true;
        }

        false
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
