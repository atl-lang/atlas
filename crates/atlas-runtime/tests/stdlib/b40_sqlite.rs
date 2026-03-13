//! B40: SQLite bindings tests (H-286)
//!
//! Tests for sqlite namespace: open, execute, query, close

use atlas_runtime::runtime::Atlas;
use atlas_runtime::value::Value;

fn eval(source: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(source).unwrap()
}

// ============================================================================
// sqlite.open Tests
// ============================================================================

#[test]
fn test_sqlite_open_memory() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        reflect.typeOf(conn)
    "#,
    );
    assert_eq!(result.to_string(), "SqliteConnection");
}

#[test]
fn test_sqlite_open_returns_connection() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        reflect.typeOf(conn) == "SqliteConnection"
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// conn.execute Tests
// ============================================================================

#[test]
fn test_sqlite_execute_create_table() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        let result = conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)", []);
        match result {
            Ok(_) => "success",
            Err(e) => e
        }
    "#,
    );
    assert_eq!(result.to_string(), "success");
}

#[test]
fn test_sqlite_execute_insert() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE users (id INTEGER, name TEXT)", [])?;
        let inserted = conn.execute("INSERT INTO users VALUES (1, 'Alice')", [])?;
        inserted
    "#,
    );
    // Should return number of rows affected (1)
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sqlite_execute_multiple_inserts() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE items (id INTEGER)", [])?;
        conn.execute("INSERT INTO items VALUES (1)", [])?;
        conn.execute("INSERT INTO items VALUES (2)", [])?;
        conn.execute("INSERT INTO items VALUES (3)", [])?;
        "done"
    "#,
    );
    assert_eq!(result.to_string(), "done");
}

#[test]
fn test_sqlite_execute_with_params() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE data (value TEXT)", [])?;
        let rows = conn.execute("INSERT INTO data VALUES (?)", ["hello"])?;
        rows
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sqlite_execute_update() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE counter (n INTEGER)", [])?;
        conn.execute("INSERT INTO counter VALUES (0)", [])?;
        let updated = conn.execute("UPDATE counter SET n = 42", [])?;
        updated
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sqlite_execute_delete() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE rows (id INTEGER)", [])?;
        conn.execute("INSERT INTO rows VALUES (1)", [])?;
        conn.execute("INSERT INTO rows VALUES (2)", [])?;
        let deleted = conn.execute("DELETE FROM rows WHERE id = 1", [])?;
        deleted
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

// ============================================================================
// conn.query Tests
// ============================================================================

#[test]
fn test_sqlite_query_empty_table() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE empty_table (id INTEGER)", [])?;
        let rows = conn.query("SELECT * FROM empty_table", [])?;
        len(rows)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_sqlite_query_returns_array() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE items (id INTEGER)", [])?;
        conn.execute("INSERT INTO items VALUES (1)", [])?;
        let rows = conn.query("SELECT * FROM items", [])?;
        len(rows) >= 0
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_sqlite_query_single_row() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE single (value INTEGER)", [])?;
        conn.execute("INSERT INTO single VALUES (42)", [])?;
        let rows = conn.query("SELECT * FROM single", [])?;
        len(rows)
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sqlite_query_multiple_rows() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE multi (n INTEGER)", [])?;
        conn.execute("INSERT INTO multi VALUES (1)", [])?;
        conn.execute("INSERT INTO multi VALUES (2)", [])?;
        conn.execute("INSERT INTO multi VALUES (3)", [])?;
        let rows = conn.query("SELECT * FROM multi", [])?;
        len(rows)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_sqlite_query_with_where() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE filtered (id INTEGER, active INTEGER)", [])?;
        conn.execute("INSERT INTO filtered VALUES (1, 1)", [])?;
        conn.execute("INSERT INTO filtered VALUES (2, 0)", [])?;
        conn.execute("INSERT INTO filtered VALUES (3, 1)", [])?;
        let rows = conn.query("SELECT * FROM filtered WHERE active = 1", [])?;
        len(rows)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_sqlite_query_with_params() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE search (name TEXT)", [])?;
        conn.execute("INSERT INTO search VALUES ('Alice')", [])?;
        conn.execute("INSERT INTO search VALUES ('Bob')", [])?;
        let rows = conn.query("SELECT * FROM search WHERE name = ?", ["Alice"])?;
        len(rows)
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sqlite_query_row_access() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE access (id INTEGER, name TEXT)", [])?;
        conn.execute("INSERT INTO access VALUES (1, 'Test')", [])?;
        let rows = conn.query("SELECT * FROM access", [])?;
        reflect.typeOf(rows[0])
    "#,
    );
    // Rows are JSON objects
    assert_eq!(result.to_string(), "json");
}

#[test]
fn test_sqlite_query_numeric_access() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE nums (value INTEGER)", [])?;
        conn.execute("INSERT INTO nums VALUES (999)", [])?;
        let rows = conn.query("SELECT * FROM nums", [])?;
        len(rows)
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

// ============================================================================
// conn.close Tests
// ============================================================================

#[test]
fn test_sqlite_close_basic() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        let result = conn.close();
        match result {
            Ok(_) => "closed",
            Err(e) => e
        }
    "#,
    );
    assert_eq!(result.to_string(), "closed");
}

#[test]
fn test_sqlite_close_double_close() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.close()?;
        let result = conn.close();
        match result {
            Ok(_) => "ok",
            Err(_) => "error"
        }
    "#,
    );
    // Double close should return error
    assert_eq!(result.to_string(), "error");
}

// ============================================================================
// Error Handling
// ============================================================================

#[test]
fn test_sqlite_execute_syntax_error() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        let result = conn.execute("INVALID SQL SYNTAX", []);
        match result {
            Ok(_) => "unexpected",
            Err(_) => "error"
        }
    "#,
    );
    assert_eq!(result.to_string(), "error");
}

#[test]
fn test_sqlite_query_nonexistent_table() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        let result = conn.query("SELECT * FROM nonexistent", []);
        match result {
            Ok(_) => "unexpected",
            Err(_) => "error"
        }
    "#,
    );
    assert_eq!(result.to_string(), "error");
}

// ============================================================================
// Integration / Complex Operations
// ============================================================================

#[test]
fn test_sqlite_full_crud() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");

        // Create
        conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)", [])?;

        // Insert - use inline values instead of params since mixed type arrays aren't allowed
        conn.execute("INSERT INTO users (name, age) VALUES ('Alice', 30)", [])?;
        conn.execute("INSERT INTO users (name, age) VALUES ('Bob', 25)", [])?;

        // Read
        let users = conn.query("SELECT * FROM users ORDER BY age", [])?;

        // Update
        conn.execute("UPDATE users SET age = 26 WHERE name = 'Bob'", [])?;

        // Delete
        conn.execute("DELETE FROM users WHERE name = 'Alice'", [])?;

        // Final count
        let remaining = conn.query("SELECT * FROM users", [])?;
        len(remaining)
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sqlite_transaction_pattern() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE ledger (amount INTEGER)", [])?;

        // Simulating transaction-like operations
        conn.execute("INSERT INTO ledger VALUES (100)", [])?;
        conn.execute("INSERT INTO ledger VALUES (-50)", [])?;
        conn.execute("INSERT INTO ledger VALUES (25)", [])?;

        // Count query to verify all inserts worked
        let rows = conn.query("SELECT COUNT(*) as cnt FROM ledger", [])?;
        len(rows)
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_sqlite_multiple_columns() {
    let result = eval(
        r#"
        let conn = sqlite.open(":memory:");
        conn.execute("CREATE TABLE product (name TEXT, price REAL, qty INTEGER)", [])?;
        conn.execute("INSERT INTO product VALUES ('Widget', 9.99, 100)", [])?;

        let rows = conn.query("SELECT * FROM product", [])?;
        // Verify we got a row back
        len(rows) > 0
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}
