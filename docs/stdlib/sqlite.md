# sqlite

SQLite database bindings for Atlas. Provides synchronous access to SQLite databases — both file-based and in-memory.

SQLite operations use Atlas's `Result<T, string>` type for error handling. All connection methods return errors as strings describing what went wrong.

## Opening a Connection

```atlas
fn sqlite.open(path: string): Connection
```

Opens a SQLite database at the given path. Use `":memory:"` for an in-memory database that exists only for the lifetime of the connection.

**Parameters:**
- `path` — file path to the database, or `":memory:"` for in-memory

**Returns:** A `Connection` value. On failure, returns a `Result<_, string>` error.

```atlas
let conn = sqlite.open("app.db");
let mem  = sqlite.open(":memory:");
```

Note: `sqlite.open` either returns a live `Connection` directly on success, or a `Result` error on failure. Check for error before using.

---

## Connection Methods

All methods take the connection as their first argument.

### `conn.execute`

```atlas
fn conn.execute(sql: string, params: any[]): Result<number, string>
fn conn.execute(sql: string): Result<number, string>
```

Executes a non-query SQL statement (INSERT, UPDATE, DELETE, CREATE, DROP, etc.).

**Parameters:**
- `sql` — SQL statement, may contain `?` parameter placeholders
- `params` — optional array of values bound to `?` placeholders in order

**Returns:** `Ok(rowsAffected)` on success, `Err(message)` on failure.

Supported parameter types: `null`, `bool`, `number`, `string`. Integer numbers are sent as `i64`; floats as `f64`. Other types are coerced to their string representation.

```atlas
let conn = sqlite.open(":memory:");

// Create table
let r = conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)");

// Insert with parameters
let r2 = conn.execute(
    "INSERT INTO users (name, age) VALUES (?, ?)",
    ["Alice", 30]
);

match r2 {
    Ok(n) => console.log("Inserted " + n.toString() + " row(s)"),
    Err(e) => console.error("Insert failed: " + e),
}
```

---

### `conn.query`

```atlas
fn conn.query(sql: string, params: any[]): Result<Row[], string>
fn conn.query(sql: string): Result<Row[], string>
```

Executes a SELECT statement and returns all matching rows.

**Parameters:**
- `sql` — SQL SELECT statement, may contain `?` placeholders
- `params` — optional array of bound values

**Returns:** `Ok(rows)` where each row is an object with column names as keys, or `Err(message)`.

Column value types:
- Integer columns → `number`
- Float columns → `number`
- Text columns → `string`
- Boolean columns → `bool`
- BLOB columns → `string` (base64-encoded)
- NULL → `null`

```atlas
let conn = sqlite.open("app.db");

let result = conn.query("SELECT id, name, age FROM users WHERE age > ?", [25]);

match result {
    Ok(rows) => {
        for row in rows {
            console.log(row.name + " is " + row.age.toString());
        }
    },
    Err(e) => console.error("Query failed: " + e),
}
```

```atlas
// Query with no parameters
let all = conn.query("SELECT * FROM users");

match all {
    Ok(rows) => console.log("Found " + rows.length().toString() + " users"),
    Err(e)   => console.error(e),
}
```

---

### `conn.close`

```atlas
fn conn.close(): Result<null, string>
```

Closes the connection and releases the underlying file handle. After closing, any further calls on this connection return an error.

**Returns:** `Ok(null)` on success, `Err(message)` if already closed or on failure.

```atlas
let conn = sqlite.open("app.db");
// ... use conn ...
let r = conn.close();
match r {
    Ok(_)  => console.log("Connection closed"),
    Err(e) => console.error("Close failed: " + e),
}
```

Calling `conn.close()` on an already-closed connection returns `Err("close: connection already closed")`.

---

## Full Example

```atlas
fn setupDatabase(): void {
    let conn = sqlite.open(":memory:");

    conn.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price REAL)");

    conn.execute("INSERT INTO products (name, price) VALUES (?, ?)", ["Widget", 9.99]);
    conn.execute("INSERT INTO products (name, price) VALUES (?, ?)", ["Gadget", 24.99]);
    conn.execute("INSERT INTO products (name, price) VALUES (?, ?)", ["Doohickey", 4.49]);

    let cheap = conn.query("SELECT name, price FROM products WHERE price < ?", [10.0]);
    match cheap {
        Ok(rows) => {
            for row in rows {
                console.log(row.name + " costs $" + row.price.toString());
            }
        },
        Err(e) => console.error("Query error: " + e),
    }

    conn.close();
}
```

---

## Error Handling Notes

- `sqlite.open` propagates I/O and path errors as `Result` errors.
- `conn.execute` and `conn.query` wrap SQL syntax errors, constraint violations, and type mismatches as `Err(string)`.
- Using a closed connection returns `Err("execute: connection is closed")` or `Err("query: connection is closed")`.
- The connection is protected by a mutex; a poisoned lock produces an error message indicating lock failure.
