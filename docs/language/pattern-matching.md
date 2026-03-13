# Pattern Matching

Pattern matching in Atlas is performed with `match` expressions. A `match` evaluates an expression (the scrutinee), compares it against each arm's pattern in order, and evaluates the first arm whose pattern matches.

`match` is an expression — it produces a value and can appear anywhere an expression is valid.

## Basic Syntax

```atlas
match value {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => fallback,
}
```

Arms are separated by commas (or semicolons). A trailing separator before `}` is optional. Every `match` must have at least one arm.

## Literal Patterns

Match exact values: numbers, strings, booleans, or `null`.

```atlas
let status = 404;

match status {
    200 => console.log("OK"),
    404 => console.log("Not Found"),
    500 => console.log("Server Error"),
    _ => console.log("Unknown"),
}
```

```atlas
let lang = "atlas";

match lang {
    "atlas" => console.log("Atlas"),
    "rust"  => console.log("Rust"),
    _       => console.log("other"),
}
```

```atlas
let flag = true;

match flag {
    true  => console.log("yes"),
    false => console.log("no"),
}
```

## Wildcard Pattern

`_` matches any value and discards it. Use it as the default/catch-all arm:

```atlas
match code {
    0 => console.log("success"),
    _ => console.log("failure"),
}
```

Unlike a variable binding, `_` never introduces a new binding — it is intentionally discarding the value.

## Variable Binding Patterns

A lowercase identifier matches any value and binds it to a new local variable:

```atlas
match result {
    0   => console.log("zero"),
    n   => console.log("got: " + n.toString()),
}
```

`n` is available inside the arm body. Variable bindings are always lowercase — uppercase identifiers are treated as enum variants (see below).

## OR Patterns

Use `|` to match multiple patterns in a single arm:

```atlas
match day {
    "Saturday" | "Sunday" => console.log("weekend"),
    _                     => console.log("weekday"),
}
```

```atlas
match code {
    400 | 401 | 403 => console.log("client error"),
    500 | 502 | 503 => console.log("server error"),
    _               => console.log("other"),
}
```

## Guard Clauses

Append `if condition` after a pattern to add a boolean predicate. The arm only matches if both the pattern matches and the guard evaluates to `true`:

```atlas
match n {
    x if x < 0  => console.log("negative"),
    x if x == 0 => console.log("zero"),
    x            => console.log("positive: " + x.toString()),
}
```

Guards can reference bindings introduced by the pattern:

```atlas
match pair {
    (x, y) if x == y => console.log("equal"),
    (x, y)           => console.log("different"),
}
```

## Constructor Patterns

Match `Ok`, `Err`, and `Some` and extract their inner values:

```atlas
let result: Result<number, string> = Ok(42);

match result {
    Ok(v)  => console.log("value: " + v.toString()),
    Err(e) => console.log("error: " + e),
}
```

```atlas
let opt: Option<string> = Some("hello");

match opt {
    Some(s) => console.log(s),
    None    => console.log("nothing"),
}
```

Nested constructors work too:

```atlas
match outer {
    Ok(Some(x)) => console.log("got: " + x.toString()),
    Ok(None)    => console.log("ok but empty"),
    Err(e)      => console.log("error: " + e),
}
```

## Enum Variant Patterns

Enum variants are matched with either the fully qualified `EnumName::VariantName` syntax or the bare uppercase variant name. Both forms are valid; the typechecker resolves bare names from the scrutinee type.

```atlas
enum Color {
    Red,
    Green,
    Blue,
    Rgb(number, number, number),
}

let c = Color::Rgb(255, 128, 0);

// Fully qualified form
match c {
    Color::Red        => console.log("red"),
    Color::Green      => console.log("green"),
    Color::Blue       => console.log("blue"),
    Color::Rgb(r, g, b) => console.log(`rgb(${r}, ${g}, ${b})`),
}
```

Bare variant names (no `::` prefix) are also accepted when the enum type is clear from context:

```atlas
enum Status {
    Pending,
    Running,
    Done(number),
}

match status {
    Pending    => console.log("waiting"),
    Running    => console.log("in progress"),
    Done(code) => console.log("finished: " + code.toString()),
}
```

Unit variants (no arguments) simply match by name. Tuple variants bind their positional fields to new variables.

## Struct Patterns

Match struct values by naming their fields. Shorthand binds a field to a variable with the same name; the long form provides a sub-pattern:

```atlas
struct Point {
    x: number,
    y: number,
}

let p = Point { x: 3, y: 4 };

// Shorthand — binds x and y
match p {
    Point { x, y } => console.log(`(${x}, ${y})`),
}

// Long form with sub-patterns
match p {
    Point { x: 0, y } => console.log(`on y-axis at ${y}`),
    Point { x, y: 0 } => console.log(`on x-axis at ${x}`),
    Point { x, y }    => console.log(`(${x}, ${y})`),
}
```

Anonymous record patterns (without a type name) are also valid when the scrutinee is a record:

```atlas
match record {
    { name, age } => console.log(`${name} is ${age}`),
}
```

## Tuple Patterns

Destructure tuple values positionally:

```atlas
let pair: (number, string) = (1, "one");

match pair {
    (0, s) => console.log("zero: " + s),
    (n, s) => console.log(n.toString() + ": " + s),
}
```

Nested tuples:

```atlas
match triple {
    (0, 0, 0) => console.log("origin"),
    (x, 0, 0) => console.log(`on x-axis at ${x}`),
    (x, y, z) => console.log(`(${x}, ${y}, ${z})`),
}
```

## Array Patterns

Match arrays by their contents. The pattern must match the array length exactly:

```atlas
match items {
    []        => console.log("empty"),
    [x]       => console.log("single: " + x.toString()),
    [x, y]    => console.log("pair"),
    _         => console.log("longer"),
}
```

## Nested Patterns

Patterns compose arbitrarily:

```atlas
enum Message {
    Move { x: number, y: number },
    Color(number, number, number),
    Quit,
}

match msg {
    Message::Move { x: 0, y } => console.log(`move to y=${y} on x=0`),
    Message::Move { x, y }    => console.log(`move to (${x}, ${y})`),
    Message::Color(r, g, b)   => console.log(`color rgb(${r}, ${g}, ${b})`),
    Message::Quit              => console.log("quit"),
}
```

Struct patterns with guards:

```atlas
match user {
    User { name, age } if age >= 18 => console.log(`${name} is an adult`),
    User { name, age }              => console.log(`${name} is a minor`),
}
```

## Match as an Expression

`match` produces a value. Assign it directly:

```atlas
let label = match code {
    200 => "OK",
    404 => "Not Found",
    _   => "Unknown",
};
```

Return it from a function:

```atlas
fn describe(n: number): string {
    match n {
        0 => "zero",
        x if x < 0 => "negative",
        _ => "positive",
    }
}
```

The `return` keyword is also valid in match arm bodies:

```atlas
fn classify(n: number): string {
    match n {
        0 => return "zero",
        _ => return "nonzero",
    }
}
```

## Exhaustiveness

The typechecker requires match expressions to be exhaustive — every possible value must be covered. For enums, every variant must appear in at least one arm. For open types like `number` and `string`, a wildcard `_` or variable binding arm satisfies exhaustiveness.

Missing arms produce a compile error, not a runtime panic.

## Summary of Pattern Kinds

| Pattern | Syntax | Description |
|---------|--------|-------------|
| Literal | `42`, `"hi"`, `true`, `null` | Match an exact value |
| Wildcard | `_` | Match anything, discard |
| Variable | `x` (lowercase) | Match anything, bind to `x` |
| OR | `a \| b` | Match either pattern |
| Guard | `pattern if cond` | Match pattern and condition |
| Constructor | `Ok(x)`, `Some(v)`, `None` | Unwrap Result/Option |
| Enum variant | `Color::Red`, `Pending(msg)` | Match enum variant |
| Struct | `Point { x, y }` | Match struct fields |
| Tuple | `(a, b)` | Match tuple elements positionally |
| Array | `[a, b]` | Match array by contents and length |
