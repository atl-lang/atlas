//! Lexical analysis (tokenization)

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// Lexer state for tokenizing source code
pub struct Lexer {
    _placeholder: (),
}

impl Lexer {
    /// Create a new lexer for the given source code
    pub fn new(_source: &str) -> Self {
        Self { _placeholder: () }
    }

    /// Tokenize the source code
    pub fn tokenize(&mut self) -> Result<Vec<Token>, Vec<Diagnostic>> {
        // Placeholder implementation
        Ok(vec![Token {
            kind: TokenKind::Eof,
            span: Span::dummy(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_creation() {
        let mut lexer = Lexer::new("test");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
    }
}
