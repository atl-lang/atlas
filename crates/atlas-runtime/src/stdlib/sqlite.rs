//! SQLite database bindings
//!
//! Provides synchronous SQLite database operations:
//! - sqlite.open(path) -> Connection
//! - conn.execute(sql, params) -> Result<number, string>
//! - conn.query(sql, params) -> Result<Row[], string>
//! - conn.close() -> Result<null, string>

use super::stdlib_arity_error;
use crate::json_value::JsonValue;
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use rusqlite::{params_from_iter, Connection, ToSql};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// SQLite Connection Wrapper
// ============================================================================

/// Wrapper around rusqlite::Connection with thread-safe access.
/// SQLite connections are not Send, so we use Mutex for safe access.
pub struct SqliteConnection {
    conn: Mutex<Option<Connection>>,
}

impl SqliteConnection {
    fn new(conn: Connection) -> Self {
        SqliteConnection {
            conn: Mutex::new(Some(conn)),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.conn.lock().map(|g| g.is_none()).unwrap_or(true) // Treat poisoned mutex as closed
    }
}

impl std::fmt::Debug for SqliteConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let closed = self.is_closed();
        write!(
            f,
            "SqliteConnection({})",
            if closed { "closed" } else { "open" }
        )
    }
}

// ============================================================================
// Namespace Functions (sqlite.*)
// ============================================================================

/// Open a SQLite database connection.
///
/// Atlas signature: `sqlite.open(path: string) -> Connection`
/// - ":memory:" creates an in-memory database
/// - File path creates or opens a file-based database
pub fn open(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("sqlite.open", 1, args.len(), span));
    }

    let path = match &args[0] {
        Value::String(s) => s.as_ref().to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "sqlite.open() requires string path argument".to_string(),
                span,
            })
        }
    };

    match Connection::open(&path) {
        Ok(conn) => Ok(Value::SqliteConnection(Arc::new(SqliteConnection::new(
            conn,
        )))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "sqlite.open: failed to open '{}': {}",
            path, e
        )))))),
    }
}

// ============================================================================
// Instance Methods (Connection.*)
// ============================================================================

/// Execute SQL statement (INSERT, UPDATE, DELETE, CREATE, etc.)
///
/// Atlas signature: `conn.execute(sql: string, params: any[]) -> Result<number, string>`
/// Returns the number of rows affected.
pub fn execute(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(stdlib_arity_error("conn.execute", 2, args.len(), span));
    }

    let conn = match &args[0] {
        Value::SqliteConnection(c) => c,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "execute() requires SqliteConnection, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    let sql = match &args[1] {
        Value::String(s) => s.as_ref().to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "execute() requires string SQL argument".to_string(),
                span,
            })
        }
    };

    // Extract params array (default to empty)
    let params = if args.len() == 3 {
        match &args[2] {
            Value::Array(arr) => value_array_to_sql_params(arr.as_slice()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "execute() params must be an array".to_string(),
                    span,
                })
            }
        }
    } else {
        Vec::new()
    };

    // Execute the statement
    let guard = match conn.conn.lock() {
        Ok(g) => g,
        Err(_) => {
            return Ok(Value::Result(Err(Box::new(Value::string(
                "execute: connection lock poisoned".to_string(),
            )))))
        }
    };
    let connection = match guard.as_ref() {
        Some(c) => c,
        None => {
            return Ok(Value::Result(Err(Box::new(Value::string(
                "execute: connection is closed".to_string(),
            )))))
        }
    };

    match connection.execute(&sql, params_from_iter(params.iter())) {
        Ok(rows_affected) => Ok(Value::Result(Ok(Box::new(Value::Number(
            rows_affected as f64,
        ))))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "execute: {}",
            e
        )))))),
    }
}

/// Query SQL statement (SELECT)
///
/// Atlas signature: `conn.query(sql: string, params: any[]) -> Result<Row[], string>`
/// Returns array of rows, where each row is a HashMap.
pub fn query(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(stdlib_arity_error("conn.query", 2, args.len(), span));
    }

    let conn = match &args[0] {
        Value::SqliteConnection(c) => c,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "query() requires SqliteConnection, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    let sql = match &args[1] {
        Value::String(s) => s.as_ref().to_string(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "query() requires string SQL argument".to_string(),
                span,
            })
        }
    };

    // Extract params array (default to empty)
    let params = if args.len() == 3 {
        match &args[2] {
            Value::Array(arr) => value_array_to_sql_params(arr.as_slice()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "query() params must be an array".to_string(),
                    span,
                })
            }
        }
    } else {
        Vec::new()
    };

    // Execute the query
    let guard = match conn.conn.lock() {
        Ok(g) => g,
        Err(_) => {
            return Ok(Value::Result(Err(Box::new(Value::string(
                "query: connection lock poisoned".to_string(),
            )))))
        }
    };
    let connection = match guard.as_ref() {
        Some(c) => c,
        None => {
            return Ok(Value::Result(Err(Box::new(Value::string(
                "query: connection is closed".to_string(),
            )))))
        }
    };

    let mut stmt = match connection.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "query: {}",
                e
            ))))))
        }
    };

    // Get column names
    let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

    // Execute and collect rows
    let rows_result = stmt.query_map(params_from_iter(params.iter()), |row| {
        let mut obj: HashMap<String, JsonValue> = HashMap::new();
        for (idx, name) in column_names.iter().enumerate() {
            let json_val = sql_value_to_json_value(row, idx);
            obj.insert(name.clone(), json_val);
        }
        Ok(obj)
    });

    match rows_result {
        Ok(rows) => {
            let mut result: Vec<Value> = Vec::new();
            for row_result in rows {
                match row_result {
                    Ok(row_obj) => {
                        result.push(Value::JsonValue(Arc::new(JsonValue::Object(row_obj))));
                    }
                    Err(e) => {
                        return Ok(Value::Result(Err(Box::new(Value::string(format!(
                            "query row error: {}",
                            e
                        ))))))
                    }
                }
            }
            Ok(Value::Result(Ok(Box::new(Value::array(result)))))
        }
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "query: {}",
            e
        )))))),
    }
}

/// Close a SQLite connection.
///
/// Atlas signature: `conn.close() -> Result<null, string>`
pub fn close(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("conn.close", 1, args.len(), span));
    }

    let conn = match &args[0] {
        Value::SqliteConnection(c) => c,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "close() requires SqliteConnection, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    let mut guard = match conn.conn.lock() {
        Ok(g) => g,
        Err(_) => {
            return Ok(Value::Result(Err(Box::new(Value::string(
                "close: connection lock poisoned".to_string(),
            )))))
        }
    };
    if guard.is_none() {
        return Ok(Value::Result(Err(Box::new(Value::string(
            "close: connection already closed".to_string(),
        )))));
    }

    // Drop the connection
    *guard = None;
    Ok(Value::Result(Ok(Box::new(Value::Null))))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert Atlas Value array to SQL parameter values.
/// Supports: null, bool, number, string.
fn value_array_to_sql_params(arr: &[Value]) -> Vec<Box<dyn ToSql>> {
    arr.iter()
        .map(|v| -> Box<dyn ToSql> {
            match v {
                Value::Null => Box::new(rusqlite::types::Null),
                Value::Bool(b) => Box::new(*b),
                Value::Number(n) => {
                    // Check if it's an integer
                    if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                        Box::new(*n as i64)
                    } else {
                        Box::new(*n)
                    }
                }
                Value::String(s) => Box::new(s.as_ref().to_string()),
                // For other types, convert to string representation
                other => Box::new(format!("{}", other)),
            }
        })
        .collect()
}

/// Convert SQLite row value to JsonValue.
fn sql_value_to_json_value(row: &rusqlite::Row, idx: usize) -> JsonValue {
    // Try each type in order of likelihood
    if let Ok(v) = row.get::<_, i64>(idx) {
        return JsonValue::Number(v as f64);
    }
    if let Ok(v) = row.get::<_, f64>(idx) {
        return JsonValue::Number(v);
    }
    if let Ok(v) = row.get::<_, String>(idx) {
        return JsonValue::String(v);
    }
    if let Ok(v) = row.get::<_, bool>(idx) {
        return JsonValue::Bool(v);
    }
    if let Ok(v) = row.get::<_, Vec<u8>>(idx) {
        // Binary data as base64 string
        return JsonValue::String(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &v,
        ));
    }
    // NULL or unknown type
    JsonValue::Null
}
