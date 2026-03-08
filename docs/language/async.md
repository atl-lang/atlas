# Async / Await

This document is the authoritative specification for Atlas async/await.
All B8 phases implement exactly what is described here — **spec is law**.

Concurrency model: **multi-threaded tokio runtime** (D-030).

---

## Overview

Atlas async/await provides structured concurrency built on Tokio.
An `async fn` is a function that may suspend and resume — it always returns
`Future<T>` at the language level, regardless of whether the return annotation
says `T` or `Future<T>`.

```atlas
async fn fetch(url: string) -> string {
    let response = await http.get(url);
    return response.body;
}
```

Top-level `await` is allowed without any `async` wrapper — the runtime
executes it via `block_on` on a multi-threaded Tokio runtime.

```atlas
// top-level script — no async fn required
let result = await fetch("https://example.com");
print(result);
```

---

## Syntax

### async fn Declaration

```
async fn name<T>(param: Type, ...) -> ReturnType { ... }
```

- The `async` keyword precedes `fn`.
- Return type annotation is **required** (same rule as sync functions).
- Write the inner return type — `-> string`, not `-> Future<string>`.
  The compiler automatically wraps it in `Future<T>`.
- Writing `-> Future<T>` explicitly is also accepted and means the same thing
  (the wrapper is not double-applied).
- `async fn` may appear at module scope, inside structs (as methods), and
  as closures are **not** supported in B8 (AT4009).

Example:
```atlas
async fn add_async(a: number, b: number) -> number {
    return a + b;
}
```

### await Expression

```
await expr
```

- `await` is a **prefix keyword expression**, not a function call.
- `expr` must have type `Future<T>`; the result type is `T`.
- Applying `await` to a non-`Future` value is a type error (AT4002).
- `await` is valid **only** inside an `async fn` body or at the **top level**
  of a script. Using it anywhere else (e.g. inside a sync helper function
  called from an async context) raises AT4001.

Example:
```atlas
async fn run() -> number {
    let f: Future<number> = async_compute();
    let n = await f;           // n: number
    return n;
}
```

### await + ? (Error Propagation)

When the future resolves to `Result<T, E>`, `?` may be chained immediately:

```atlas
async fn load(path: string) -> Result<string, string> {
    let content = await readFile(path)?;   // unwrap Ok or early-return Err
    return Ok(content);
}
```

The `?` applies to the resolved `T`; order of evaluation is:
1. `await` suspends until `Future<Result<T, E>>` resolves.
2. `?` unwraps `Ok(T)` or propagates `Err(E)`.

---

## Type System

### Future\<T\>

`Future<T>` is a **first-class named type** in Atlas.

- An `async fn` that declares `-> T` has the external call-site type `Future<T>`.
- `Future<T>` values may be stored in variables, passed to functions, and
  returned from other async functions.
- A `Future<T>` value **must** be awaited before its inner `T` is accessible.
  Using a `Future<T>` as if it were `T` without awaiting generates AT4005
  (warning, not error, to allow explicit passing of futures).

```atlas
let f: Future<number> = add_async(1, 2);   // not yet resolved
let n: number = await f;                    // resolved
```

### Type Compatibility

- `Future<T>` and `Future<U>` where `T ≠ U` are incompatible (AT4008).
- An `async fn` cannot be passed where a sync `fn(A) -> B` parameter is
  expected, because the call-site type differs (AT4004).

---

## Concurrency Primitives

These are stdlib functions that work with futures.

### spawn

```atlas
fn spawn<T>(f: Future<T>) -> Future<T>
```

Spawns `f` as an independent Tokio task on the multi-threaded runtime.
Returns a new `Future<T>` that resolves when the task completes.

```atlas
async fn main_work() -> void {
    let task_a = spawn(compute_a());
    let task_b = spawn(compute_b());
    let a = await task_a;
    let b = await task_b;
    print(a + b);
    return;
}
```

`spawn` requires an active Tokio runtime. Calling it in a sync context with
no runtime raises AT4007.

### all

```atlas
fn all<T>(futures: Future<T>[]) -> Future<T[]>
```

Waits for **all** futures to resolve and returns their results in order.
If any future panics or fails, `all` propagates the first failure.

```atlas
let results = await all([fetch("a"), fetch("b"), fetch("c")]);
```

### race

```atlas
fn race<T>(futures: Future<T>[]) -> Future<T>
```

Returns the result of the **first** future to resolve.
Remaining futures are cancelled.

```atlas
let winner = await race([slow_path(), fast_path()]);
```

---

## Restrictions and Forbidden Patterns

| Pattern | Diagnostic |
|---------|------------|
| `await` outside async fn and outside top-level | AT4001 (error) |
| `await non_future_value` | AT4002 (error) |
| async fn body returns incompatible type | AT4003 (error) |
| `async fn` passed as sync fn argument | AT4004 (error) |
| `Future<T>` used as `T` without `await` | AT4005 (warning) |
| `async fn main()` | AT4006 (error) — use top-level await instead |
| `spawn(...)` in sync context with no runtime | AT4007 (error) |
| `Future<T>` used where `Future<U>` expected, T ≠ U | AT4008 (error) |
| `async fn(params) { ... }` (async closure) | AT4009 (error) — not yet supported |
| `await` inside sync `for` loop body | AT4010 (error) — ambiguous evaluation order |

---

## Error Messages

### AT4001 — await outside async context

```
error[AT4001]: `await` used outside of an async function or top-level scope
  --> example.atlas:5:5
   |
 5 |     let x = await some_future();
   |             ^^^^^ not inside an async fn or top-level
   |
   = hint: Move this into an `async fn`, or restructure so the `await`
           appears at the top level of the script.
```

### AT4002 — await applied to non-Future

```
error[AT4002]: `await` applied to a non-Future value
  --> example.atlas:3:13
   |
 3 |     let x = await 42;
   |             ^^^^^^^^^ type `number` is not `Future<_>`
   |
   = hint: Only values of type `Future<T>` can be awaited.
```

### AT4006 — async main forbidden

```
error[AT4006]: `main` function cannot be declared `async`
  --> example.atlas:1:1
   |
 1 | async fn main() -> void { ... }
   | ^^^^^ `async` on `main` is forbidden
   |
   = hint: Use top-level `await` instead — the Atlas runtime wraps
           the entire script in `block_on` automatically.
```

---

## Examples

### Basic async/await

```atlas
async fn double_async(n: number) -> number {
    return n * 2;
}

let result = await double_async(21);
print(result);  // 42
```

### Concurrent tasks

```atlas
async fn fetch_data(id: number) -> string {
    await sleep(0.1);
    return "data_" + id;
}

let tasks = [spawn(fetch_data(1)), spawn(fetch_data(2)), spawn(fetch_data(3))];
let results = await all(tasks);
print(results);  // ["data_1", "data_2", "data_3"]
```

### Error propagation

```atlas
async fn read_config(path: string) -> Result<string, string> {
    let contents = await readFile(path)?;
    return Ok(contents);
}

let cfg = await read_config("config.atlas")?;
print(cfg);
```

### race for timeout

```atlas
async fn with_timeout(f: Future<string>, secs: number) -> Result<string, string> {
    fn make_timeout(s: Future<void>) -> Future<Result<string, string>> {
        return s.then(fn(_) -> Result<string, string> { return Err("timeout"); });
    }
    fn make_ok(s: Future<string>) -> Future<Result<string, string>> {
        return s.then(fn(v: string) -> Result<string, string> { return Ok(v); });
    }
    return await race([make_ok(f), make_timeout(sleep(secs))]);
}
```
> **Note:** The `|>` pipeline operator is not yet implemented. Use named helper functions or chained method calls instead.
