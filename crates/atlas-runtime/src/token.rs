//! Token types for lexical analysis

use crate::span::Span;

/// Token type produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,
    /// Source location
    pub span: Span,
}

/// Classification of token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    /// Integer literal
    Integer(i64),
    /// Float literal
    Float(f64),
    /// String literal
    String(String),
    /// Boolean literal (true/false)
    Bool(bool),

    // Keywords
    /// `let` keyword
    Let,
    /// `fun` keyword
    Fun,
    /// `if` keyword
    If,
    /// `else` keyword
    Else,
    /// `while` keyword
    While,
    /// `return` keyword
    Return,
    /// `null` keyword
    Null,

    // Identifiers
    /// Identifier name
    Ident(String),

    // Operators
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `==`
    EqualEqual,
    /// `!=`
    BangEqual,
    /// `<`
    Less,
    /// `<=`
    LessEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `=`
    Equal,

    // Delimiters
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `;`
    Semicolon,

    /// End of file
    Eof,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token {
            kind: TokenKind::Integer(42),
            span: Span::new(0, 2),
        };
        assert_eq!(token.kind, TokenKind::Integer(42));
    }
}
