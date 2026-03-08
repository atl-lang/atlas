//! Atlas runtime API for embedding

use crate::binder::Binder;
use crate::diagnostic::{Diagnostic, StackTraceFrame};
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::module_executor::ModuleExecutor;
use crate::parser::Parser;
use crate::security::SecurityContext;
use crate::span::Span;
use crate::typechecker::TypeChecker;
use crate::value::{RuntimeError, Value};
use std::cell::RefCell;

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, Vec<Diagnostic>>;

/// Atlas runtime instance
///
/// Provides a high-level API for embedding Atlas in host applications.
///
/// # Examples
///
/// ```
/// use atlas_runtime::Atlas;
///
/// let runtime = Atlas::new();
/// let result = runtime.eval("1 + 2");
/// ```
pub struct Atlas {
    /// Interpreter for executing code (using interior mutability)
    interpreter: RefCell<Interpreter>,
    /// Security context for permission checks
    security: SecurityContext,
}

impl Atlas {
    /// Create a new Atlas runtime instance with default (deny-all) security
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::Atlas;
    ///
    /// let runtime = Atlas::new();
    /// ```
    pub fn new() -> Self {
        Self {
            interpreter: RefCell::new(Interpreter::new()),
            security: SecurityContext::new(),
        }
    }

    /// Create a new Atlas runtime instance with custom security context
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::{Atlas, SecurityContext};
    ///
    /// let security = SecurityContext::allow_all();
    /// let runtime = Atlas::new_with_security(security);
    /// ```
    pub fn new_with_security(security: SecurityContext) -> Self {
        Self {
            interpreter: RefCell::new(Interpreter::new()),
            security,
        }
    }

    /// Evaluate Atlas source code
    ///
    /// Returns the result of evaluating the source code, or diagnostics if there are errors.
    ///
    /// # Arguments
    ///
    /// * `source` - Atlas source code to evaluate
    ///
    /// # Examples
    ///
    /// ```
    /// use atlas_runtime::{Atlas, Value};
    ///
    /// let runtime = Atlas::new();
    /// let result = runtime.eval("1 + 2");
    /// match result {
    ///     Ok(Value::Number(n)) => assert_eq!(n, 3.0),
    ///     Err(diagnostics) => panic!("Error: {:?}", diagnostics),
    ///     Ok(_) => panic!("Unexpected value"),
    /// }
    /// ```
    pub fn eval(&self, source: &str) -> RuntimeResult<Value> {
        self.eval_source(source, "<input>")
    }

    fn eval_source(&self, source: &str, file: &str) -> RuntimeResult<Value> {
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
        let mut lexer = Lexer::new(&source_with_semi).with_file(file);
        let (tokens, lex_diagnostics) = lexer.tokenize();

        if !lex_diagnostics.is_empty() {
            return Err(lex_diagnostics);
        }

        // Parse tokens into AST
        let mut parser = Parser::new(tokens);
        let (ast, parse_diagnostics) = parser.parse();

        let parse_errors: Vec<_> = parse_diagnostics
            .into_iter()
            .filter(|d| d.is_error())
            .collect();
        if !parse_errors.is_empty() {
            return Err(parse_errors);
        }

        // Bind symbols
        let mut binder = Binder::new();
        let (mut symbol_table, bind_diagnostics) = binder.bind(&ast);

        if !bind_diagnostics.is_empty() {
            return Err(bind_diagnostics);
        }

        // Type check
        let mut type_checker = TypeChecker::new(&mut symbol_table);
        let type_diagnostics = type_checker.check(&ast);

        // Only fail on errors, not warnings
        let has_errors = type_diagnostics.iter().any(|d| d.is_error());
        if has_errors {
            return Err(type_diagnostics);
        }

        // Print warnings to stderr (they don't block execution)
        for diag in &type_diagnostics {
            if diag.is_warning() {
                eprintln!("{}", diag.to_human_string());
            }
        }

        // Interpret the AST
        let mut interpreter = self.interpreter.borrow_mut();

        match interpreter.eval(&ast, &self.security) {
            Ok(value) => Ok(value),
            Err(runtime_error) => {
                let stack_trace = interpreter.stack_trace_frames(
                    runtime_error.span(),
                    Some((file, source_with_semi.as_str())),
                );
                let function_name = stack_trace.first().map(|frame| frame.function.clone());
                interpreter.reset_call_stack();
                Err(vec![runtime_error_to_diagnostic(
                    runtime_error,
                    stack_trace,
                    function_name,
                )])
            }
        }
    }

    /// Evaluate an Atlas source file
    ///
    /// Reads and evaluates the Atlas source code from the specified file path.
    /// If the file contains imports, uses the module system to load dependencies.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Atlas source file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use atlas_runtime::Atlas;
    ///
    /// let runtime = Atlas::new();
    /// let result = runtime.eval_file("program.atlas");
    /// ```
    pub fn eval_file(&self, path: &str) -> RuntimeResult<Value> {
        use std::path::Path;

        let file_path = Path::new(path);

        // Get absolute path
        let abs_path = file_path.canonicalize().map_err(|e| {
            vec![
                Diagnostic::error(format!("Failed to resolve path: {}", e), Span::dummy())
                    .with_file(path),
            ]
        })?;

        // Check filesystem read permission
        self.security
            .check_filesystem_read(&abs_path)
            .map_err(|_| {
                vec![runtime_error_to_diagnostic(
                    RuntimeError::FilesystemPermissionDenied {
                        operation: "file read".to_string(),
                        path: abs_path.display().to_string(),
                        span: Span::dummy(),
                    },
                    Vec::new(),
                    None,
                )]
            })?;

        // Quick check: does the file contain imports?
        // If so, use module executor. If not, use simple eval.
        let source = std::fs::read_to_string(&abs_path).map_err(|e| {
            vec![
                Diagnostic::error(format!("Failed to read file: {}", e), Span::dummy())
                    .with_file(abs_path.display().to_string()),
            ]
        })?;

        // Check if source contains "import {" or "import *"
        if source.contains("import {") || source.contains("import *") {
            // Use module executor for multi-file programs
            let root = abs_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();

            let mut interpreter = self.interpreter.borrow_mut();
            let mut executor = ModuleExecutor::new(&mut interpreter, &self.security, root);
            executor.execute_module(&abs_path)
        } else {
            // Simple single-file program - use regular eval
            let file_display = abs_path.display().to_string();
            self.eval_source(&source, &file_display)
        }
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a RuntimeError to a Diagnostic
pub(crate) fn runtime_error_to_diagnostic(
    error: RuntimeError,
    stack_trace: Vec<StackTraceFrame>,
    function_name: Option<String>,
) -> Diagnostic {
    // Map runtime errors to their corresponding diagnostic codes from Atlas-SPEC.md
    // Extract span from error (all RuntimeError variants now include span)
    let span = error.span();

    let (code, message) = match &error {
        RuntimeError::DivideByZero { .. } => (
            "AT0005",
            "division by zero: the divisor evaluated to 0".to_string(),
        ),
        RuntimeError::OutOfBounds { .. } => (
            "AT0006",
            "array index out of bounds: index exceeds array length".to_string(),
        ),
        RuntimeError::InvalidNumericResult { .. } => (
            "AT0007",
            "invalid numeric result: operation produced NaN or Infinity".to_string(),
        ),
        RuntimeError::InvalidIndex { .. } => (
            "AT0103",
            "invalid index: array indices must be whole numbers (not fractions or negatives)"
                .to_string(),
        ),
        // Use the detailed msg field (includes function signature from P05)
        RuntimeError::InvalidStdlibArgument { msg, .. } => ("AT0102", msg.clone()),
        RuntimeError::TypeError { msg, .. } => ("AT0001", format!("type error: {}", msg)),
        RuntimeError::UndefinedVariable { name, .. } => {
            ("AT0002", format!("undefined variable '{}': variable is not in scope", name))
        }
        RuntimeError::UnknownFunction { name, .. } => {
            ("AT0002", format!("unknown function '{}': not defined or not in scope", name))
        }
        // VM-specific errors
        RuntimeError::UnknownOpcode { .. } => (
            "AT9998",
            "unknown bytecode opcode: this is a compiler bug; please report it".to_string(),
        ),
        RuntimeError::StackUnderflow { .. } => (
            "AT9997",
            "stack underflow: more values popped than pushed — this is a compiler bug; please report it".to_string(),
        ),
        // Permission errors
        RuntimeError::FilesystemPermissionDenied {
            operation, path, ..
        } => (
            "AT0300",
            format!("Permission denied: {} access to {}", operation, path),
        ),
        RuntimeError::NetworkPermissionDenied { host, .. } => (
            "AT0301",
            format!("Permission denied: network access to {}", host),
        ),
        RuntimeError::ProcessPermissionDenied { command, .. } => (
            "AT0302",
            format!("Permission denied: process execution of {}", command),
        ),
        RuntimeError::EnvironmentPermissionDenied { var, .. } => (
            "AT0303",
            format!("Permission denied: environment variable {}", var),
        ),
        RuntimeError::IoError { message, .. } => ("AT0400", message.clone()),
        RuntimeError::UnhashableType { type_name, .. } => (
            "AT0140",
            format!(
                "Cannot hash type {} - only number, string, bool, null are hashable",
                type_name
            ),
        ),
        RuntimeError::Timeout { elapsed, limit } => (
            "AT0500",
            format!(
                "Execution timeout: {:?} elapsed, limit was {:?}",
                elapsed, limit
            ),
        ),
        RuntimeError::FfiPermissionDenied { function } => (
            "AT0304",
            format!("Permission denied: FFI call to {}", function),
        ),
        RuntimeError::MemoryLimitExceeded {
            requested,
            used,
            limit,
        } => (
            "AT0501",
            format!(
                "Memory limit exceeded: attempted to allocate {} bytes, limit is {} bytes (used: {} bytes)",
                requested, limit, used
            ),
        ),
        RuntimeError::InternalError { msg, .. } => ("AT9995", format!("Internal error: {}", msg)),
    };

    let message = if let Some(function_name) = function_name {
        format!("{} in function {}", message, function_name)
    } else {
        message
    };

    let help = match &error {
        RuntimeError::DivideByZero { .. } => {
            "guard against zero before dividing:\n  if divisor != 0 { result = dividend / divisor }"
        }
        RuntimeError::OutOfBounds { .. } => {
            "check array length before indexing:\n  if i < len(arr) { value = arr[i] }\n  or use arr.get(i) which returns null when out of bounds"
        }
        RuntimeError::InvalidNumericResult { .. } => {
            "ensure inputs to math operations are finite numbers.\n  Use isFinite(n) to check before using the result."
        }
        RuntimeError::InvalidIndex { .. } => {
            "array indices must be whole non-negative numbers.\n  Use floor(n) to convert a float, or check i >= 0 before indexing."
        }
        RuntimeError::InvalidStdlibArgument { .. } => {
            // The msg field already contains full context (including signature from P05)
            "check the function signature shown above for correct argument types and count"
        }
        RuntimeError::UndefinedVariable { name, .. } => {
            // help is included in the message for dynamic content
            let _ = name; // used in message
            "declare the variable with 'let name = value' before using it"
        }
        RuntimeError::UnknownFunction { .. } => {
            "check spelling — or import the function from its module"
        }
        RuntimeError::FilesystemPermissionDenied { .. } => {
            "enable file permissions with --allow-file or adjust security settings"
        }
        RuntimeError::NetworkPermissionDenied { .. } => {
            "enable network permissions with --allow-network or adjust security settings"
        }
        RuntimeError::ProcessPermissionDenied { .. } => {
            "enable process permissions with --allow-process or adjust security settings"
        }
        RuntimeError::EnvironmentPermissionDenied { .. } => {
            "enable environment permissions with --allow-env or adjust security settings"
        }
        RuntimeError::UnknownOpcode { .. } | RuntimeError::StackUnderflow { .. } => {
            "this is a bug in the Atlas compiler; please report it at https://github.com/anthropics/atlas/issues"
        }
        RuntimeError::InternalError { .. } => {
            "this is a bug in the runtime; please report it"
        }
        _ => "check the error message above for details",
    };

    Diagnostic::error_with_code(code, message, span)
        .with_stack_trace(stack_trace)
        .with_help(help)
}
