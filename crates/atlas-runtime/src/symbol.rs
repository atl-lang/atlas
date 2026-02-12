//! Symbol table and name binding

use crate::span::Span;
use std::collections::HashMap;

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Declaration location
    pub span: Span,
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
}

/// Symbol table for name resolution
pub struct SymbolTable {
    /// Stack of scopes
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
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
    pub fn define(&mut self, symbol: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(symbol.name.clone(), symbol);
        }
    }

    /// Look up a symbol in all scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        table.define(Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            span: Span::dummy(),
        });
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }
}
