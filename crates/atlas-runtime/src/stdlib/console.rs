//! Console namespace stdlib functions
//!
//! Implements `console.log()`, `console.println()`, `console.print()`,
//! `console.error()`, `console.warn()`, and `console.debug()`.
//!
//! - log / println → println to stdout (via OutputWriter)
//! - print         → print without newline to stdout (via OutputWriter)
//! - error         → eprintln to stderr
//! - warn          → eprintln with "WARN: " prefix to stderr
//! - debug         → eprintln with "DEBUG: " prefix to stderr

use crate::span::Span;
use crate::stdlib::OutputWriter;
use crate::value::{RuntimeError, Value};
use std::io::Write;

/// Format multiple args as space-separated display strings.
fn format_args(args: &[Value]) -> String {
    args.iter()
        .map(|v| v.to_display_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// console.log(...) — println to stdout via OutputWriter.
pub fn console_log(
    args: &[Value],
    span: Span,
    output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    let msg = format_args(args);
    let mut w = output.lock().map_err(|_| RuntimeError::TypeError {
        msg: "output lock poisoned".into(),
        span,
    })?;
    writeln!(w, "{}", msg).map_err(|_| RuntimeError::TypeError {
        msg: "write failed".into(),
        span,
    })?;
    Ok(Value::Null)
}

/// console.println(...) — alias for console.log.
pub fn console_println(
    args: &[Value],
    span: Span,
    output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    console_log(args, span, output)
}

/// console.print(...) — print without newline to stdout via OutputWriter.
pub fn console_print(
    args: &[Value],
    span: Span,
    output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    let msg = format_args(args);
    let mut w = output.lock().map_err(|_| RuntimeError::TypeError {
        msg: "output lock poisoned".into(),
        span,
    })?;
    write!(w, "{}", msg).map_err(|_| RuntimeError::TypeError {
        msg: "write failed".into(),
        span,
    })?;
    Ok(Value::Null)
}

/// console.error(...) — eprintln to stderr.
pub fn console_error(
    args: &[Value],
    _span: Span,
    _output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    let msg = format_args(args);
    eprintln!("{}", msg);
    Ok(Value::Null)
}

/// console.warn(...) — eprintln with "WARN: " prefix to stderr.
pub fn console_warn(
    args: &[Value],
    _span: Span,
    _output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    let msg = format_args(args);
    eprintln!("WARN: {}", msg);
    Ok(Value::Null)
}

/// console.debug(...) — eprintln with "DEBUG: " prefix to stderr.
pub fn console_debug(
    args: &[Value],
    _span: Span,
    _output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    let msg = format_args(args);
    eprintln!("DEBUG: {}", msg);
    Ok(Value::Null)
}
