//! Parsing (tokens to AST)

use crate::ast::Program;
use crate::diagnostic::Diagnostic;
use crate::token::Token;

/// Parser state for building AST from tokens
pub struct Parser {
    _placeholder: (),
}

impl Parser {
    /// Create a new parser for the given tokens
    pub fn new(_tokens: Vec<Token>) -> Self {
        Self { _placeholder: () }
    }

    /// Parse tokens into an AST
    pub fn parse(&mut self) -> Result<Program, Vec<Diagnostic>> {
        // Placeholder implementation
        Ok(Program { items: Vec::new() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let mut parser = Parser::new(Vec::new());
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 0);
    }
}
