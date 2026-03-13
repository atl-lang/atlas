# Foreign Function Interface (FFI)

The Atlas FFI lets Atlas programs call functions in C shared libraries. It uses statically-typed `extern` declarations, a direct function-pointer calling convention, automatic type marshaling, and an optional callback mechanism for C code to call back into Atlas.

## Overview

FFI in Atlas has three layers:

1. **Declaration** — an `extern fn` declaration in source tells the compiler the library name, function signature, and C types.
2. **Loading** — the runtime locates and loads the shared library at program start.
3. **Calling** — when the extern function is called, arguments are marshaled from Atlas values to C types, the foreign function is invoked via a direct function pointer cast, and the result is marshaled back.

## Declaring Extern Functions

Syntax:

```atlas
extern fn functionName(param: c_type, ...): c_return_type from "library"
```

The library name is a short name (e.g. `"m"` for libm) or an absolute path. The runtime resolves it to the platform-specific filename automatically.

Examples:

```atlas
// Call sqrt from libm
extern fn sqrt(x: c_double): c_double from "m"

// Call strlen from libc
extern fn strlen(s: c_char_ptr): c_int from "c"

// Call a custom library function
extern fn processData(input: c_int, scale: c_double): c_double from "mylib"
```

After declaring, call the function like a regular Atlas function:

```atlas
let root = sqrt(9.0);       // returns 3.0 as number
let len = strlen("hello");  // returns 5 as number
```

The optional `symbol` field allows the Atlas name to differ from the C symbol:

```atlas
extern fn atlasName(...): c_int from "lib" // symbol matches fn name by default
```

## C Type System

Atlas FFI uses a fixed set of C-compatible types. These are the only permitted types in `extern` declarations.

| Atlas keyword | C type | Atlas runtime type |
|---------------|--------|--------------------|
| `c_int` | `int` (i32) | `number` |
| `c_long` | `long` (i64 on 64-bit) | `number` |
| `c_double` | `double` (f64) | `number` |
| `c_char_ptr` | `char*` (null-terminated) | `string` |
| `c_void` | `void` | `null` |
| `c_bool` | `_Bool` / `uint8_t` | `bool` |

All numeric C types map to Atlas `number`. Strings map to Atlas `string`. Void returns map to `null`. The marshaling layer validates ranges and rejects values that do not fit (e.g. an Atlas `number` with a fractional part cannot be marshaled to `c_int`).

## Marshaling

Marshaling happens automatically at the FFI boundary. The rules are:

**Atlas → C:**
- `number` → `c_int`: truncates to i32, rejects non-integers and out-of-range values
- `number` → `c_long`: truncates to i64, rejects non-integers and out-of-range values
- `number` → `c_double`: passes f64 directly
- `string` → `c_char_ptr`: creates a null-terminated C string; the memory lives until the call returns
- `bool` → `c_bool`: `true` becomes `1`, `false` becomes `0`
- `null` → `c_void`: no-op

**C → Atlas:**
- `c_int` / `c_long` / `c_double` → `number`: widened to f64
- `c_char_ptr` → `string`: reads the null-terminated string as UTF-8; panics on null pointer or invalid UTF-8
- `c_bool` → `bool`: non-zero becomes `true`
- `c_void` → `null`

Strings passed to C (`c_char_ptr`) must not contain embedded null bytes. Strings returned from C must be valid null-terminated UTF-8.

## Library Loading

The runtime loads shared libraries using platform-specific search paths and naming conventions.

**Search order:**
1. Current working directory
2. Platform system paths:
   - Linux: `/usr/lib`, `/usr/local/lib`, `/lib`, plus multiarch paths (`/usr/lib/x86_64-linux-gnu`, etc.)
   - macOS: `/usr/lib`, `/usr/local/lib`, `/opt/homebrew/lib`
   - Windows: `C:\Windows\System32`

**Naming convention:** The library name in the `extern` declaration is the short name. The runtime applies the platform prefix and extension automatically:
- Linux: `"m"` → `libm.so.6`
- macOS: `"m"` → `libm.dylib`
- Windows: `"m"` → `m.dll`

Absolute paths bypass the search and naming logic.

Libraries are cached after the first load. Calling the same library's functions multiple times incurs no repeated loading cost.

## Calling Convention

The FFI uses direct function pointer casts rather than a dynamic FFI library. Because Atlas `extern` declarations carry complete type information at parse time, the runtime can transmute the raw symbol address to a typed `extern "C" fn` pointer and call it directly. This approach is fast and requires no code generation at runtime.

The currently supported call signatures are:

| Signature | Example use |
|-----------|-------------|
| `() -> c_int` | `getpid()` |
| `() -> c_long` | Long-returning zero-arg functions |
| `() -> c_double` | `drand48()` |
| `() -> c_void` | `void` zero-arg functions |
| `(c_int) -> c_int` | `abs(n)` |
| `(c_double) -> c_double` | `sqrt(x)`, `fabs(x)` |
| `(c_char_ptr) -> c_int` | `strlen(s)`, `puts(s)` |
| `(c_int, c_int) -> c_int` | `min(a, b)` |
| `(c_double, c_double) -> c_double` | `pow(x, y)`, `atan2(y, x)` |

Signatures outside this set produce a `UnsupportedSignature` error at runtime. New signatures must be added in `crates/atlas-runtime/src/ffi/caller.rs`.

## Callbacks (C Calling Atlas)

The callback system allows C code to call back into Atlas functions. This is the pattern used by event-driven C APIs (e.g. `qsort`, GUI toolkits, audio APIs).

A callback handle wraps an Atlas closure and provides two pointers that C code needs:

- **Trampoline** — a real `extern "C"` function pointer with stable address
- **Context** — an opaque pointer encoding the closure

C code must call the trampoline with the context as its first argument:

```c
// C side (conceptual)
result = trampoline(context, arg1, arg2);
```

On the Atlas side, creating a callback is an internal runtime operation. The `CallbackHandle` owns the closure and frees it when dropped.

Supported callback signatures:

| Parameters → Return | Notes |
|--------------------|-------|
| `() -> c_int` | |
| `(c_double) -> c_double` | |
| `(c_double, c_double) -> c_double` | |
| `(c_int) -> c_int` | |
| `(c_int, c_int) -> c_int` | |
| `(c_long) -> c_long` | |
| `() -> c_void` | |
| `(c_int) -> c_void` | |

## Safety

FFI is inherently unsafe. The Atlas FFI isolates all unsafe code inside `crates/atlas-runtime/src/ffi/` and wraps it in safe APIs. The key invariants are:

- **String lifetimes:** C strings created during argument marshaling are tracked in a `MarshalContext` and freed when the call returns. Never pass the pointer to a C function that stores it beyond the call.
- **Null pointers:** The marshaling layer checks for null `c_char_ptr` returns and produces a `MarshalError::NullPointer` rather than dereferencing.
- **Range validation:** Numeric narrowing (`number` → `c_int`) rejects values outside the target type's range.
- **Library trust:** Loading a dynamic library executes its initialization code in the same process. Only load trusted libraries.
- **Signature correctness:** The caller is responsible for ensuring the declared `extern` signature matches the actual C function. A mismatch is undefined behavior.

The security policy `ResourceType::FFI` controls whether FFI is permitted in a given sandbox. By default all I/O including FFI requires an explicit grant.

## Complete Example

```atlas
// Bind sqrt and pow from libm
extern fn sqrt(x: c_double): c_double from "m"
extern fn pow(base: c_double, exp: c_double): c_double from "m"

fn hypotenuse(a: number, b: number): number {
    return sqrt(pow(a, 2.0) + pow(b, 2.0));
}

console.log(hypotenuse(3.0, 4.0).toString());  // 5
```

```atlas
// Bind strlen from libc
extern fn strlen(s: c_char_ptr): c_int from "c"

let len = strlen("hello");
console.log(len.toString());  // 5
```
