//! Abstract Syntax Tree (AST) definitions

use crate::span::Span;

/// Top-level program
#[derive(Debug, Clone)]
pub struct Program {
    /// List of statements
    pub statements: Vec<Stmt>,
}

/// Statement node
#[derive(Debug, Clone)]
pub struct Stmt {
    /// Statement kind
    pub kind: StmtKind,
    /// Source location
    pub span: Span,
}

/// Statement kinds
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Expression statement
    Expr(Expr),
    /// Variable declaration
    Let {
        name: String,
        type_annotation: Option<String>,
        initializer: Option<Expr>,
    },
    /// Placeholder for future statement types
    Placeholder,
}

/// Expression node
#[derive(Debug, Clone)]
pub struct Expr {
    /// Expression kind
    pub kind: ExprKind,
    /// Source location
    pub span: Span,
}

/// Expression kinds
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Integer literal
    Integer(i64),
    /// String literal
    String(String),
    /// Boolean literal
    Bool(bool),
    /// Null literal
    Null,
    /// Variable reference
    Ident(String),
    /// Placeholder for future expression types
    Placeholder,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_creation() {
        let expr = Expr {
            kind: ExprKind::Integer(42),
            span: Span::dummy(),
        };
        assert!(matches!(expr.kind, ExprKind::Integer(42)));
    }
}
