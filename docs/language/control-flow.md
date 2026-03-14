# Atlas Control Flow

Atlas control flow syntax follows Rust structure (no parentheses around conditions, braces required) with TypeScript-style semantics where they overlap.

---

## `if` / `else`

```atlas
if condition {
    // truthy branch
}

if x > 0 {
    console.log("positive");
} else if x < 0 {
    console.log("negative");
} else {
    console.log("zero");
}
```

**No parentheses around conditions.** Writing `if (x > 0) { }` is accepted but the parser emits a warning and asks you to remove them.

**Braces are required.** There is no brace-less single-statement form.

### `if` as an expression

`if` blocks can be used as expressions via tail expressions:

```atlas
let label = if score >= 90 { "A" } else { "B" };
```

Both branches must produce the same type. The tail expression (last expression without `;`) is the branch value.

---

## `match`

Pattern matching over a value. At least one arm is required. Arms are separated by commas or semicolons; a trailing separator before `}` is optional.

```atlas
match value {
    pattern1 => expr1,
    pattern2 => expr2,
    _ => fallback,
}
```

### Pattern types

**Literal patterns:**
```atlas
match n {
    0 => "zero",
    1 => "one",
    _ => "other",
}
```

**OR patterns (`|`):**
```atlas
match status {
    200 | 201 | 204 => "success",
    404             => "not found",
    _               => "error",
}
```

**Enum variant patterns:**
```atlas
match color {
    Color.Red           => "red",
    Color.Rgb(r, g, b)  => "custom",
    _                   => "other",
}
```

**Option / Result patterns:**
```atlas
match result {
    Ok(value) => console.log(value),
    Err(e)    => console.log(e),
}

match opt {
    Some(x) => x * 2,
    None    => 0,
}
```

**Struct patterns:**
```atlas
match point {
    Point { x: 0, y } => console.log("on y-axis"),
    Point { x, y: 0 } => console.log("on x-axis"),
    Point { x, y }    => console.log("at " + x.toString() + ", " + y.toString()),
}
```

**Tuple patterns:**
```atlas
match pair {
    (0, 0) => "origin",
    (x, 0) => "x-axis",
    (0, y) => "y-axis",
    (x, y) => "point",
}
```

**Wildcard (`_`):**
Matches anything, binds nothing.

**Binding patterns:**
A bare identifier in pattern position binds the matched value:
```atlas
match x {
    n if n > 100 => console.log("big: " + n.toString()),
    n            => console.log("small: " + n.toString()),
}
```

### Guard clauses

```atlas
match n {
    x if x > 0 && x < 10 => "single digit positive",
    x if x >= 10          => "ten or more",
    _                     => "non-positive",
}
```

Guards follow the pattern, before `=>`. The bound name from the pattern is in scope in the guard.

### `match` as an expression

`match` produces a value. All arms must produce the same type (or be assignable to a common type):

```atlas
let category = match score {
    s if s >= 90 => "A",
    s if s >= 80 => "B",
    s if s >= 70 => "C",
    _            => "F",
};
```

### `return`, `break`, and `continue` in match arms

`return` is valid as a match arm body to early-exit the enclosing function:

```atlas
match opt {
    None    => return Err("missing value"),
    Some(v) => v,
}
```

`break` and `continue` are valid in match arms inside loops to control the loop without requiring a semicolon:

```atlas
for item in items {
    match item {
        Some(v) => { process(v); },
        None    => continue,  // skip to next iteration
    }
}

for i in 0..100 {
    match classify(i) {
        Invalid => break,     // exit loop entirely
        Valid   => { use(i); },
    }
}
```

[Fixed: 6b78ef40]

---

## `for` / `in`

Iterate over any iterable value (arrays, ranges, maps, sets):

```atlas
for item in collection {
    console.log(item.toString());
}

// Range iteration (exclusive upper bound)
for i in 0..10 {
    console.log(i.toString());
}

// Inclusive range
for i in 0..=10 {
    console.log(i.toString());
}
```

**No parentheses.** `for (x in arr) { }` is accepted with a warning.

**No C-style `for` loop.** Atlas has only `for item in iterable { }`. There is no `for (let i = 0; i < n; i++)` syntax — use `for i in 0..n { }` with a range instead.

---

## `while`

```atlas
while condition {
    // body
}

let mut i = 0;
while i < 10 {
    console.log(i.toString());
    i += 1;
}
```

**No parentheses around condition.** Same as `if`.

**No `loop` keyword.** Infinite loops use `while true { }`:

```atlas
while true {
    let input = console.readLine();
    if input == "quit" { break; }
    process(input);
}
```

---

## `break` and `continue`

```atlas
for i in 0..100 {
    if i == 50 { break; }      // exit loop entirely
    if i % 2 == 0 { continue; } // skip even numbers
    console.log(i.toString());
}
```

Both require a trailing semicolon. There is no labeled break/continue in the current version.

---

## `return`

Exit a function immediately, optionally with a value:

```atlas
fn find(arr: number[], target: number): Option<number> {
    for i in 0..arr.length() {
        if arr[i] == target {
            return Some(i);   // early return
        }
    }
    return None;
}
```

`return;` with no value is valid in `void` functions:

```atlas
fn process(input: string): void {
    if input == "" { return; }
    // ...
}
```

`return` requires a trailing semicolon.

---

## Early Return Patterns

Early return on error is the idiomatic Atlas pattern:

```atlas
fn parse_and_process(raw: string): Result<string, string> {
    let value = Json.parse(raw);
    if value.isErr() {
        return Err("parse failed");
    }

    let data = value.unwrap();
    // continue with data...
    return Ok(data.toString());
}
```

### The `?` operator

`?` propagates errors automatically. In a function returning `Result<T, E>`, appending `?` to a `Result`-producing expression unwraps the `Ok` value or returns the `Err` immediately:

```atlas
fn divide(a: number, b: number): Result<number, string> {
    if b == 0 { return Err("division by zero"); }
    Ok(a / b)
}

fn compute(x: number, y: number): Result<number, string> {
    let result = divide(x, y)?;  // returns Err early if divide returns Err
    return Ok(result * 2);
}
```

---

## `defer`

`defer` schedules cleanup to run when the enclosing scope exits, in LIFO order:

```atlas
fn open_and_process(path: string): Result<string, string> {
    let file = fs.open(path)?;
    defer { file.close(); }
    defer { console.log("done"); }

    // On exit (normal or early return): "done" prints first, then file.close()
    return Ok(file.readAll());
}

// Single-expression form (requires trailing semicolon)
defer resource.close();
```

---

## Blocks as Expressions

Any `{ }` block can be used as an expression via a tail expression (last expression without `;`):

```atlas
let result = {
    let a = compute_a();
    let b = compute_b();
    a + b   // tail expression — this is the value of the block
};
```

This works inside function bodies, `if` branches, `match` arms, and standalone statement position.

---

## Operator Precedence Reference

From lowest to highest:

| Level | Operators |
|---|---|
| Range | `..` `..=` |
| Or | `\|\|` |
| And | `&&` |
| Equality | `==` `!=` |
| Comparison | `<` `<=` `>` `>=` |
| Term | `+` `-` |
| Factor | `*` `/` `%` |
| Unary | `!` `-` (prefix) |
| Call | `()` `[]` `.` |

---

## Gotchas

**Parentheses around conditions emit warnings.** `if (x)` and `for (x in arr)` are parsed but generate a diagnostic asking you to remove the parens.

**`break` and `continue` require semicolons** in statement position. Exception: when used directly as a match arm body (e.g., `None => break`), no semicolon is needed.

**No `loop` keyword.** Use `while true { }` for infinite loops.

**No labeled break.** You cannot break to an outer loop by label in the current version.

**`match` arms need a separator.** Each arm must be followed by `,` or `;`. A trailing separator before `}` is optional.

**`match` must have at least one arm.** An empty `match { }` is a compile error.

**`return` in a void function still needs the semicolon.** `return;` not just `return`.

**`if` without `else` in expression position.** If you use `if` as an expression, both branches must exist and produce the same type. `if cond { val }` without an `else` is only valid as a statement.
