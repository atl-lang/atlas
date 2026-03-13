# console

The `console` namespace provides output functions for writing to stdout and stderr. All methods accept any number of arguments of any type, format them as space-separated display strings, and write them to the appropriate stream.

All `console` methods return `null`.

---

## Argument Formatting

Multiple arguments are joined with a single space between them. Each value is converted using its display representation:

- `number` → decimal (e.g., `42`, `3.14`)
- `string` → bare string (no quotes)
- `bool` → `true` or `false`
- `null` → `null`
- `array` → `[elem, elem, ...]`
- `Option` → `Some(value)` or `None`
- `Result` → `Ok(value)` or `Err(value)`

---

## stdout Methods

### `console.log`

```atlas
fn console.log(...args: any[]): null
```

Prints arguments as a space-separated line to stdout, followed by a newline. This is the primary output function for general program output.

```atlas
console.log("Hello, Atlas!");
console.log("Count:", 42, "done:", true);
// Output:
// Hello, Atlas!
// Count: 42 done: true
```

---

### `console.println`

```atlas
fn console.println(...args: any[]): null
```

Alias for `console.log`. Identical behavior — prints to stdout with a trailing newline.

```atlas
console.println("Same as log");
```

---

### `console.print`

```atlas
fn console.print(...args: any[]): null
```

Prints arguments to stdout **without** a trailing newline. Useful for building output incrementally or prompts.

```atlas
console.print("Loading");
console.print(".");
console.print(".");
console.print(".");
console.log("done");
// Output: Loading...done
```

---

## stderr Methods

The following methods write to stderr. They do not go through the `OutputWriter` abstraction — they write directly to the process stderr stream. This means they appear even when stdout is captured (e.g., in test harnesses).

### `console.error`

```atlas
fn console.error(...args: any[]): null
```

Prints arguments to stderr with a trailing newline. No prefix is added.

```atlas
console.error("Something went wrong:", 404);
// stderr: Something went wrong: 404
```

---

### `console.warn`

```atlas
fn console.warn(...args: any[]): null
```

Prints arguments to stderr with a `WARN: ` prefix and trailing newline.

```atlas
console.warn("Deprecated function called");
// stderr: WARN: Deprecated function called
```

---

### `console.debug`

```atlas
fn console.debug(...args: any[]): null
```

Prints arguments to stderr with a `DEBUG: ` prefix and trailing newline. Use during development to trace execution; remove or gate behind a flag before production.

```atlas
console.debug("user object:", user);
// stderr: DEBUG: user object: { ... }
```

---

## Summary Table

| Method | Stream | Newline | Prefix |
|--------|--------|---------|--------|
| `console.log` | stdout | yes | none |
| `console.println` | stdout | yes | none |
| `console.print` | stdout | no | none |
| `console.error` | stderr | yes | none |
| `console.warn` | stderr | yes | `WARN: ` |
| `console.debug` | stderr | yes | `DEBUG: ` |

---

## Examples

```atlas
fn greet(borrow name: string): void {
    console.log("Hello,", name + "!");
}

fn divide(borrow a: number, borrow b: number): Result<number, string> {
    if b == 0 {
        console.error("Division by zero attempted");
        return Err("division by zero");
    }
    return Ok(a / b);
}

fn main(): void {
    greet("Atlas");

    let result = divide(10, 2);
    match result {
        Ok(v)  => console.log("Result:", v),
        Err(e) => console.error("Failed:", e),
    }

    console.warn("This feature is experimental");
    console.debug("Internal state check passed");
}
```

Output (stdout):
```
Hello, Atlas!
Result: 5
```

Output (stderr):
```
WARN: This feature is experimental
DEBUG: Internal state check passed
```
