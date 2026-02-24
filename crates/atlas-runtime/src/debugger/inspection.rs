//! Variable inspection and expression evaluation for the Atlas debugger.
//!
//! Provides scoped variable collection, formatted output, watch expressions,
//! and hover evaluation for the debugger.

use crate::debugger::protocol::Variable;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::security::SecurityContext;
use crate::value::Value;
use crate::vm::VM;

// ── VariableScope ────────────────────────────────────────────────────────────

/// The scope a variable belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableScope {
    /// Local variable in the current function frame.
    Local,
    /// Global variable.
    Global,
}

// ── ScopedVariable ───────────────────────────────────────────────────────────

/// A variable with scope information.
#[derive(Debug, Clone, PartialEq)]
pub struct ScopedVariable {
    /// Protocol-level variable (name, value, type_name).
    pub variable: Variable,
    /// Scope the variable belongs to.
    pub scope: VariableScope,
}

impl ScopedVariable {
    /// Create a new scoped variable.
    pub fn new(variable: Variable, scope: VariableScope) -> Self {
        Self { variable, scope }
    }
}

// ── Inspector ────────────────────────────────────────────────────────────────

/// Inspects VM state to collect variables and evaluate expressions.
pub struct Inspector {
    /// Watch expressions that are evaluated on each pause.
    watch_expressions: Vec<String>,
    /// Maximum depth for formatting nested values.
    max_format_depth: usize,
}

impl Inspector {
    /// Create a new inspector.
    pub fn new() -> Self {
        Self {
            watch_expressions: Vec::new(),
            max_format_depth: 3,
        }
    }

    /// Set the maximum format depth for nested values.
    pub fn set_max_format_depth(&mut self, depth: usize) {
        self.max_format_depth = depth;
    }

    /// Get the maximum format depth.
    pub fn max_format_depth(&self) -> usize {
        self.max_format_depth
    }

    /// Add a watch expression.
    pub fn add_watch(&mut self, expression: String) {
        if !self.watch_expressions.contains(&expression) {
            self.watch_expressions.push(expression);
        }
    }

    /// Remove a watch expression.
    pub fn remove_watch(&mut self, expression: &str) -> bool {
        let len_before = self.watch_expressions.len();
        self.watch_expressions.retain(|e| e != expression);
        self.watch_expressions.len() < len_before
    }

    /// Clear all watch expressions.
    pub fn clear_watches(&mut self) {
        self.watch_expressions.clear();
    }

    /// Get all watch expressions.
    pub fn watch_expressions(&self) -> &[String] {
        &self.watch_expressions
    }

    /// Collect variables with scope information from a VM frame.
    pub fn collect_scoped_variables(&self, vm: &VM, frame_index: usize) -> Vec<ScopedVariable> {
        let mut vars = Vec::new();

        // Locals from the requested frame
        for (slot, value) in vm.get_locals_for_frame(frame_index) {
            vars.push(ScopedVariable::new(
                Variable::new(
                    format!("local_{slot}"),
                    format_value_with_depth(value, self.max_format_depth),
                    value.type_name(),
                ),
                VariableScope::Local,
            ));
        }

        // Globals
        for (name, value) in vm.get_global_variables() {
            vars.push(ScopedVariable::new(
                Variable::new(
                    name.clone(),
                    format_value_with_depth(value, self.max_format_depth),
                    value.type_name(),
                ),
                VariableScope::Global,
            ));
        }

        vars.sort_by(|a, b| a.variable.name.cmp(&b.variable.name));
        vars
    }

    /// Collect only local variables for a frame.
    pub fn collect_locals(&self, vm: &VM, frame_index: usize) -> Vec<Variable> {
        vm.get_locals_for_frame(frame_index)
            .into_iter()
            .map(|(slot, value)| {
                Variable::new(
                    format!("local_{slot}"),
                    format_value_with_depth(value, self.max_format_depth),
                    value.type_name(),
                )
            })
            .collect()
    }

    /// Collect only global variables.
    pub fn collect_globals(&self, vm: &VM) -> Vec<Variable> {
        let mut vars: Vec<Variable> = vm
            .get_global_variables()
            .iter()
            .map(|(name, value)| {
                Variable::new(
                    name.clone(),
                    format_value_with_depth(value, self.max_format_depth),
                    value.type_name(),
                )
            })
            .collect();
        vars.sort_by(|a, b| a.name.cmp(&b.name));
        vars
    }

    /// Evaluate an expression in the context of visible variables.
    ///
    /// Injects variable bindings as `let` statements before the expression.
    pub fn evaluate_expression(&self, expression: &str, variables: &[Variable]) -> EvalResult {
        let mut snippet = String::new();
        for var in variables {
            if is_valid_identifier(&var.name) {
                if let Some(lit) = value_to_atlas_literal(&var.type_name, &var.value) {
                    snippet.push_str(&format!("let {} = {};\n", var.name, lit));
                }
            }
        }
        snippet.push_str(expression);
        let trimmed = expression.trim();
        if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
            snippet.push(';');
        }

        let tokens = Lexer::new(&snippet).tokenize().0;
        let (ast, errors) = Parser::new(tokens).parse();
        if !errors.is_empty() {
            return EvalResult::Error(format!("parse error: {:?}", errors[0]));
        }

        let mut interp = Interpreter::new();
        let security = SecurityContext::allow_all();
        match interp.eval(&ast, &security) {
            Ok(value) => EvalResult::Success {
                value: format_value_with_depth(&value, self.max_format_depth),
                type_name: value.type_name().to_string(),
            },
            Err(e) => EvalResult::Error(format!("{:?}", e)),
        }
    }

    /// Evaluate all watch expressions and return results.
    pub fn evaluate_watches(&self, variables: &[Variable]) -> Vec<WatchResult> {
        self.watch_expressions
            .iter()
            .map(|expr| {
                let result = self.evaluate_expression(expr, variables);
                WatchResult {
                    expression: expr.clone(),
                    result,
                }
            })
            .collect()
    }

    /// Quick hover evaluation: just look up a variable by name.
    pub fn hover(&self, name: &str, variables: &[Variable]) -> Option<Variable> {
        variables.iter().find(|v| v.name == name).cloned()
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}

// ── EvalResult ───────────────────────────────────────────────────────────────

/// Result of evaluating an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalResult {
    /// Successful evaluation.
    Success { value: String, type_name: String },
    /// Evaluation error.
    Error(String),
}

// ── WatchResult ──────────────────────────────────────────────────────────────

/// Result of a watch expression evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct WatchResult {
    /// The watch expression.
    pub expression: String,
    /// The evaluation result.
    pub result: EvalResult,
}

// ── Formatting helpers ───────────────────────────────────────────────────────

/// Format a `Value` for display with depth control.
pub fn format_value_with_depth(value: &Value, max_depth: usize) -> String {
    format_value_recursive(value, 0, max_depth)
}

fn format_value_recursive(value: &Value, depth: usize, max_depth: usize) -> String {
    if depth > max_depth {
        return "...".to_string();
    }
    match value {
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::String(s) => format!("\"{}\"", s.as_ref()),
        Value::Array(arr) => {
            if depth >= max_depth {
                return format!("[{} items]", arr.len());
            }
            let items: Vec<String> = arr
                .as_slice()
                .iter()
                .take(10)
                .map(|v| format_value_recursive(v, depth + 1, max_depth))
                .collect();
            if arr.len() > 10 {
                format!("[{}, ... +{} more]", items.join(", "), arr.len() - 10)
            } else {
                format!("[{}]", items.join(", "))
            }
        }
        Value::HashMap(m) => {
            format!("{{HashMap, {} entries}}", m.inner().len())
        }
        Value::HashSet(s) => {
            format!("{{HashSet, {} items}}", s.inner().len())
        }
        Value::Queue(q) => {
            format!("[Queue, {} items]", q.inner().len())
        }
        Value::Stack(s) => {
            format!("[Stack, {} items]", s.inner().len())
        }
        Value::Function(f) => format!("<fn {}>", f.name),
        _ => format!("{:?}", value),
    }
}

/// Check if a string is a valid Atlas identifier.
fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Try to produce an Atlas literal from type_name + display value.
fn value_to_atlas_literal(type_name: &str, display: &str) -> Option<String> {
    match type_name {
        "number" => {
            display.parse::<f64>().ok()?;
            Some(display.to_string())
        }
        "bool" => Some(display.to_string()),
        "null" => Some("null".to_string()),
        "string" => Some(display.to_string()),
        _ => None,
    }
}
