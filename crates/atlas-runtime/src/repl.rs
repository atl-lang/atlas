//! REPL core logic (UI-agnostic)

use crate::diagnostic::Diagnostic;
use crate::value::Value;

/// REPL result type
pub enum ReplResult {
    /// Successful evaluation
    Value(Value),
    /// Diagnostics (errors/warnings)
    Diagnostics(Vec<Diagnostic>),
    /// Empty result (no output)
    Empty,
}

/// REPL core state
pub struct ReplCore {
    _placeholder: (),
}

impl ReplCore {
    /// Create a new REPL core
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    /// Evaluate a line of input
    pub fn eval_line(&mut self, _input: &str) -> ReplResult {
        // Placeholder implementation
        ReplResult::Empty
    }

    /// Reset REPL state
    pub fn reset(&mut self) {
        // Placeholder implementation
    }
}

impl Default for ReplCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let mut repl = ReplCore::new();
        let result = repl.eval_line("1 + 1");
        assert!(matches!(result, ReplResult::Empty));
    }
}
