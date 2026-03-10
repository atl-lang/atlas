# Atlas Error Reference

Every Atlas error has a named code (`ATxxxx`), a specific problem description, and concrete fix guidance. This is intentional — Atlas is AI-first, and every error message must be self-correcting without external lookup.

---

## Canonical Error Format (B17 — updated 2026-03-09)

Every Atlas diagnostic renders in this exact format. No Rust internal chrome, no `panicked at`, no anonymous spans.

```
error[AT1020]: return type annotation required on named functions, found `;`
main.atlas:5:1
5: fn greet(name: str) {
   ^ syntax error
help: add `-> void` if the function returns nothing: `fn name(params) -> void { ... }`
note: named functions require explicit return types — closures may omit them
```

### Format Lines

| Line | Format | Meaning |
|------|--------|---------|
| 1 | `{level}[{code}]: {message}` | Level is `error`, `warning`, or `note`. Code is `ATxxxx`. Message names exactly what is wrong. |
| 2 | `{file}:{line}:{col}` | Precise source location. |
| 3 | `{line_number}: {source_snippet}` | The offending source line. |
| 4 | `{indent}^ {label}` | Single `^` anchors the label at the error column. On TTY, the error token is additionally highlighted with an ANSI background span. Label names the error category. |
| 5+ | `help: {text}` | **Actionable fix** — what to write or change. May repeat for multiple distinct fixes. |
| 5+ | `help: did you mean \`X\`?` + diff | **Code-diff suggestion** (H-195) — when Atlas knows exactly what token to replace, it renders a Rust-style `-old / +new` diff immediately after the `help:` line. |
| last | `note: {text}` | **Context/explanation** — why the rule exists or what the rule is. Not a repetition of help. |

Secondary (cascade) diagnostics render as:
```
note[ATxxxx] (secondary): {message}
```

### Help vs Note

- **help**: actionable — tells the developer exactly what to write. Example: `write \`-> void\` if the function returns nothing`.
- **note**: explanatory — explains the rule or why it matters. Example: `named functions require explicit return types — closures may omit them`.

Both are optional. When both are present, `help` comes first. Multiple `help` lines are valid when there are 2+ distinct fixes. Multiple `note` lines are valid when multiple context points apply.

### Empty Snippet

When source is unavailable (e.g. generated code, runtime-only errors), lines 3–4 are omitted. The location line still appears.

---

## Error Code Structure

| Range | Category |
|-------|----------|
| AT0001–AT0007 | Runtime errors (core) |
| AT0102–AT0103, AT0140 | Stdlib errors |
| AT0300–AT0303 | Permission errors |
| AT0400 | I/O errors |
| AT1000–AT1022 | Parser / syntax errors |
| AT2001–AT2014 | Warnings (unused, shadowing, deprecated) |
| AT3001–AT3055 | Typechecker / semantic errors |
| AT4001–AT4010 | Async/Await errors |
| AT5001–AT5008 | Module system errors |
| AT9000, AT9995–AT9999 | Internal errors (bugs) |

---

## AT0xxx — Runtime Errors (Core)

### AT0001 — Type Error
The operation was applied to an incompatible type.

```atlas
let x: number = "hello";  // ✗ AT0001
let x: number = 5.0;      // ✓
```

### AT0002 — Undefined Symbol
A variable or function name is not defined in scope.

```atlas
print(count);              // ✗ AT0002 if 'count' not declared
let count: number = 0.0;
print(count);              // ✓
```

Atlas suggests the closest matching name when available.

### AT0003 — Invalid Arity
Function called with wrong number of arguments. See AT3005 (typechecker) for compile-time checking.

### AT0004 — Invalid Operation
An operation is not valid for the given types.

### AT0005 — Divide By Zero
The divisor evaluated to zero.

```atlas
let result = a / b;        // ✗ AT0005 if b == 0
// ✓ guard before dividing:
if b != 0 {
    let result = a / b;
}
```

### AT0006 — Array Out of Bounds
Index exceeds the array length.

```atlas
let v = arr[99];           // ✗ AT0006 if arr has fewer elements
// ✓ check length first:
if i < len(arr) {
    let v = arr[i];
}
```

### AT0007 — Invalid Numeric Result
An operation produced `NaN` or `Infinity`.

```atlas
let r = sqrt(-1.0);        // produces NaN → AT0007
// ✓ guard inputs: if x >= 0 { sqrt(x) }
```

---

## AT01xx — Stdlib Errors

### AT0102 — Invalid Stdlib Argument
A stdlib function received the wrong argument type or count. The error message includes the full function signature.

```
len(): expected 1 argument, got 0
  Signature: len(value: string | []any) -> number
```

### AT0103 — Invalid Index
Array index must be a whole non-negative number (not a float or negative).

```atlas
arr[1.5]   // ✗ AT0103 — fractional index
arr[-1]    // ✗ AT0103 — negative index
arr[1]     // ✓
```

Use `floor(n)` to convert a float index.

---

## AT03xx — Permission Errors

| Code | Description |
|------|-------------|
| AT0300 | Filesystem read/write denied |
| AT0301 | Network access denied |
| AT0302 | Process execution denied |
| AT0303 | Environment variable access denied |
| AT0304 | FFI call denied |

Enable permissions with `--allow-file`, `--allow-network`, `--allow-process`, `--allow-env`.

---

## AT04xx — I/O Errors

### AT0400 — I/O Error
File read/write or network I/O failed at the OS level.

---

## AT05xx — Resource Limits

| Code | Description |
|------|-------------|
| AT0500 | Execution timeout exceeded |
| AT0501 | Memory limit exceeded |

---

## AT1xxx — Parser / Syntax Errors

### AT1000 — Syntax Error
Generic syntax error. Check for typos or missing tokens near the indicated location.

### AT1001 — Unexpected Token
A token appeared where it was not expected.

### AT1002 — Unterminated String
A string literal was opened but not closed.

```atlas
let s = "hello;    // ✗ AT1002 — missing closing "
let s = "hello";   // ✓
```

### AT1003 — Invalid Escape Sequence
An unsupported `\x` escape in a string literal.

Valid escapes: `\n`, `\t`, `\r`, `\\`, `\"`.

### AT1004 — Unterminated Comment
A block comment `/* ... */` was not closed.

### AT1005 — Invalid Number
A number literal has invalid syntax.

### AT1006 — Unexpected EOF
The file ended before a statement or expression was complete. In the REPL, this usually means you opened a block `{` without closing it — type the closing `}`.

### AT1007 — Missing Ownership Annotation
Non-Copy types (arrays, structs, HashMaps) used in a move-sensitive context require an explicit ownership annotation: `own`, `borrow`, or `share`. Bare parameters default to `borrow` (D-040) and do not trigger AT1007.

```atlas
fn bad(own x: []number, y: []number) -> void {
    store(y)   // ✗ AT1007: y is bare (implicit borrow) — cannot escape
}

fn good(own x: []number) -> void { ... }     // ✓ own: moves value into fn
fn good(borrow x: []number) -> void { ... }  // ✓ explicit borrow: read-only
fn good(share x: []number) -> void { ... }   // ✓ share: co-held reference
fn good(x: []number) -> void { ... }         // ✓ bare param: implicit borrow, no annotation needed
```

Choose `own` when the function should take exclusive ownership, `borrow` for read-only access, `share` when both caller and callee hold a valid reference. Primitives (`number`, `string`, `bool`) are Copy — no annotation required.

### AT1008 — Foreign Syntax: echo
`echo` is PHP/shell syntax. Use `print()` in Atlas.

```atlas
echo "hello";      // ✗ AT1008
print("hello");    // ✓
```

### AT1009 — Foreign Syntax: var
`var` is JavaScript syntax. Use `let` in Atlas.

```atlas
var x = 5;         // ✗ AT1009
let x = 5.0;       // ✓
```

### AT1010 — Foreign Syntax: function keyword
`function` is JavaScript/PHP syntax. Use `fn` in Atlas.

```atlas
function greet() { }   // ✗ AT1010
fn greet() -> void { } // ✓
```

### AT1011 — Foreign Syntax: class
`class` is OOP syntax. Atlas uses `struct` for data and `impl` for behavior.

```atlas
class Foo { }          // ✗ AT1011
struct Foo { }         // ✓
```

### AT1012 — Prelude Shadowing
Declaring a variable with the same name as a built-in function shadows it.

### AT1013 — Foreign Syntax: console.log
`console.log` is JavaScript syntax. Use `print()` in Atlas.

```atlas
console.log("hello");  // ✗ AT1013
print("hello");        // ✓
```

### AT1014 — Foreign Syntax: ++ / --
`x++` and `x--` are not valid in Atlas. Use explicit assignment.

```atlas
x++;           // ✗ AT1014
x = x + 1;    // ✓

x--;           // ✗ AT1014
x = x - 1;    // ✓
```

### AT1015 — Foreign Syntax: import X from
ES module `import X from` syntax is not valid. Use Atlas module syntax.

### AT1016 — Invalid Assignment Target: Range Index
Range-based slice assignment is not supported.

```atlas
arr[0..3] = value;   // ✗ AT1016
arr[0] = value;      // ✓ assign to specific index
```

### AT1017 — Invalid Assignment Target: Method Call Result
You cannot assign to the result of a method call.

```atlas
obj.method() = value;  // ✗ AT1017
// ✓ store result in a variable first:
let mut result = obj.method();
result = value;
```

### AT1018 — Invalid Assignment Target: Member of Non-Addressable
Cannot assign to a member of a non-addressable expression.

```atlas
f().field = value;     // ✗ AT1018 — f() is not addressable
let mut r = f();
r.field = value;       // ✓
```

### AT1019 — Invalid Assignment Target
Expression is not a valid assignment target.

Valid targets: variables, array indices, struct fields.

```atlas
len("hi") = 5;         // ✗ AT1019
x = 5;                 // ✓
arr[i] = 5.0;          // ✓
obj.field = 5.0;       // ✓
```

### AT1020 — Missing Semicolon
A statement requires a terminating semicolon.

```atlas
let x = 5              // ✗ AT1020 — missing `;`
let x = 5;             // ✓
print("hello")         // ✗ AT1020
print("hello");        // ✓
```

### AT1021 — Missing Closing Delimiter
A block, list, or expression is missing its closing delimiter (`}`, `]`, or `)`).

```atlas
if x > 0 {
    print("positive")  // ✗ AT1021 — missing closing `}`

if x > 0 {
    print("positive")
}                      // ✓
```

### AT1022 — Reserved Keyword Used as Identifier
A reserved keyword cannot be used as an identifier.

```atlas
let fn = 5.0;         // ✗ AT1022 — fn is reserved
let func = 5.0;       // ✓

struct import { }      // ✗ AT1022 — import is reserved
struct Container { }   // ✓
```

---

## AT2xxx — Warnings

Warnings do not stop compilation. Fix them for cleaner code.

| Code | Warning |
|------|---------|
| AT2001 | Unused variable |
| AT2002 | Unreachable code |
| AT2003 | Duplicate declaration |
| AT2004 | Unused function |
| AT2005 | Variable shadowing |
| AT2006 | Constant condition (always true/false) |
| AT2007 | Unnecessary type annotation |
| AT2008 | Unused import |
| AT2009 | Deprecated type alias |
| AT2010 | `own` on primitive type (unnecessary) |
| AT2011 | `borrow` on already-shared value |
| AT2012 | Trying to borrow and then pass as own |
| AT2013 | Move-type requires ownership annotation |
| AT2014 | Deprecated `var` keyword |

---

## AT3xxx — Typechecker / Semantic Errors

### AT3001 — Type Error
General type mismatch — expected type X, found type Y.

### AT3002 — Binary Operator Type Error
Operator cannot be applied to the given operand types.

```atlas
let r: number = 5.0 + "hi";    // ✗ AT3002
let r: string = "a" + "b";     // ✓ string concatenation
let r: number = 5.0 + 3.0;     // ✓
```

### AT3003 — Immutable Assignment
Assigning to a `let` binding (not `let mut`).

```atlas
let x = 5.0;
x = 10.0;          // ✗ AT3003
let mut x = 5.0;
x = 10.0;          // ✓
```

### AT3004 — Missing Return
A function with a non-void return type is missing a return statement on some path.

### AT3005 — Arity Mismatch
Function called with wrong number of arguments (compile-time check).

### AT3006 — Not Callable
Trying to call a value that is not a function.

### AT3010 — Invalid Index Type
Index must be a `number`, not another type.

### AT3011 — Not Indexable
Type does not support index access `[]`.

### AT3020–AT3027 — Match Errors

| Code | Description |
|------|-------------|
| AT3020 | Empty match — no arms |
| AT3021 | Match arm type mismatch |
| AT3022 | Pattern type mismatch |
| AT3023 | Constructor arity mismatch |
| AT3024 | Unknown constructor |
| AT3025 | Unsupported pattern type |
| AT3026 | Array pattern type mismatch |
| AT3027 | Non-exhaustive match |

### AT3028 — Non-Shared to Shared
Passing a non-shared value where `share` semantics are required.

### AT3029 — Impl Already Exists
A trait implementation for this type already exists.

### AT3030–AT3037 — Trait System Errors

| Code | Description |
|------|-------------|
| AT3030 | Trait redefines built-in (Copy, Move, Drop, Display, Debug) |
| AT3031 | Trait already defined in scope |
| AT3032 | Trait not found (referenced in `impl` but not declared) |
| AT3033 | `impl` block missing a required method |
| AT3034 | `impl` method signature does not match trait |
| AT3035 | Type does not implement required trait |
| AT3036 | Copy type required (non-Copy type provided) |
| AT3037 | Generic type argument does not satisfy trait bound |
| AT3056 | Inherent impl block for unknown type |
| AT3057 | Duplicate method in inherent impl |
| AT3058 | Self receiver is not the first parameter |
| AW3059 | Inherent method shadows trait method (warning, inherent wins — D-037) |

### AT3040 — Closure Captures Borrow
A closure captures a `borrow` parameter, which may escape the parameter's lifetime. Use `share` or `own` instead.

### AT3050 — Cannot Infer Return Type
The return type cannot be inferred because return expressions have incompatible types.

### AT3051 — Cannot Infer Type Argument
A generic type parameter cannot be inferred from context.

### AT3052 — Inferred Type Incompatible
The inferred return type is incompatible with how the function is used.

### AT3053 — Use After Own
A value was used after ownership was transferred.

```atlas
fn consume(own x: []number) -> void { }
let arr = [1.0, 2.0];
consume(arr);
print(arr);    // ✗ AT3053 — arr was moved into consume()
```

Options: pass `share arr` to keep the reference, or clone before moving.

### AT3054 — Borrow Escape
A `borrow` reference escapes the scope of the borrowed value. Only fires when the parameter was explicitly annotated with `borrow` in source. Bare parameters (`fn f(x: T)`) default to `borrow` per D-040 but do not trigger AT3054 — returning a bare param is always valid.

### AT3055 — Share Violation
Invalid use of `share` semantics.

---

## AT4xxx — Async/Await Errors

| Code | Description |
|------|-------------|
| AT4001 | `await` used outside `async` context |
| AT4002 | `await` applied to non-Future type |
| AT4003 | Async function return type mismatch |
| AT4004 | Async function passed where sync function expected |
| AT4005 | Future result used without `await` |
| AT4006 | `async fn main()` is forbidden |
| AT4007 | Task spawn in synchronous context |
| AT4008 | Future type mismatch in composition |
| AT4009 | Async closures not supported |
| AT4010 | `await` in synchronous loop |

---

## AT5xxx — Module System Errors

| Code | Description |
|------|-------------|
| AT5001 | Invalid module path |
| AT5002 | Module not found |
| AT5003 | Circular module dependency |
| AT5004 | Export not found |
| AT5005 | Import resolution failed |
| AT5006 | Module not exported (private) |
| AT5007 | Namespace import not supported |
| AT5008 | Duplicate export name |

---

## AT9xxx — Internal Errors

These indicate bugs in the Atlas compiler. Please report them.

| Code | Description |
|------|-------------|
| AT9995 | Internal error |
| AT9997 | Stack underflow (VM compiler bug) |
| AT9998 | Unknown opcode (VM compiler bug) |

Report at: https://github.com/anthropics/atlas/issues

---

## Contributor Guide: Adding a New Error Code (B17)

All Atlas error codes are backed by a `DiagnosticDescriptor` constant in
`crates/atlas-runtime/src/diagnostic/error_codes.rs`. Adding a new error requires three steps:

### 1. Declare the descriptor constant

```rust
pub const MY_ERROR: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT1099",
    level: DiagnosticLevel::Error,
    title: "Short title (shown in atlas explain)",
    message_template: "description with `{name}` named holes",
    static_help: Some("what the developer should do to fix this"),
    static_note: None,           // optional context note
    domain: DiagnosticDomain::Parser,  // or Typechecker, Runtime, etc.
};
```

Rules:
- `static_help` is **mandatory** — every code must have actionable guidance
- No embedded `\n` in `static_help` or `static_note`
- `message_template` uses `{name}` holes filled at call sites via `.arg("name", value)`

### 2. Add to DESCRIPTOR_REGISTRY

At the bottom of `error_codes.rs`, add `&MY_ERROR` to the `DESCRIPTOR_REGISTRY` slice.
This makes it available via `atlas explain ATxxxx` and the coverage tests.

### 3. Emit at the call site

```rust
use crate::diagnostic::error_codes::MY_ERROR;

// Builder chain — returns Diagnostic
let diag = MY_ERROR.emit(span)
    .arg("name", the_name)           // fills {name} hole
    .with_help("extra context here") // additive after static_help
    .build();
```

The render path is unified — `DiagnosticFormatter::write_diagnostic()` is the single renderer.
`Diagnostic::to_human_string()` delegates to it. Do not add render logic elsewhere.

---

## Design: Self-Correcting Errors

Atlas error messages follow the AT1007 quality bar:
1. **Named code** — `[AT1007]` — unique, searchable, stable
2. **Specific problem** — what went wrong and where
3. **Concrete fix** — Atlas syntax showing the correct way

This means you can fix Atlas errors by reading the error output alone. No documentation lookup required. This is intentional for AI code generation: a model can generate valid Atlas just by reading error messages.
