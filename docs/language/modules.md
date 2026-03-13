# Modules

Atlas uses TypeScript's module system: explicit `import`/`export` at the file level, no global namespace, no implicit sharing between files.

---

## Overview

Each `.atl` file is a module. Names are private by default — nothing is visible across file boundaries unless explicitly exported. The runtime loads dependencies in topological order, detecting circular imports at load time.

**File extensions:** `.atl` (primary), `.atlas` (also accepted)

---

## Import Syntax

### Named Imports

```atlas
import { add, subtract } from "./math";
import { User, createUser } from "./models/user";
```

Named imports bind specific exported names from the target module. The name inside `{ }` must match the exported name exactly.

Trailing commas are allowed:

```atlas
import {
    add,
    subtract,
    multiply,
} from "./math";
```

### Namespace Imports

```atlas
import * as math from "./math";

let result = math.add(1, 2);
```

`import * as ns` brings all exports from a module into a single namespace object. Access with dot notation: `ns.name`.

### Import Rules

- The module path is a **string literal** (with quotes).
- Relative paths: `"./sibling"`, `"../parent"`, `"./subdirectory/file"`.
- Imports require a trailing semicolon.
- `import` is **top-level only** — writing `import` inside a function or block body produces a diagnostic.

---

## Export Syntax

Prefix any top-level declaration with `export` to make it accessible from other modules.

### Exported Functions

```atlas
export fn add(x: number, y: number): number {
    return x + y;
}

export async fn fetchData(url: string): Future<string> {
    // ...
}
```

### Exported Variables

```atlas
export let VERSION: string = "1.0.0";
export let mut counter: number = 0;
```

### Exported Type Aliases

```atlas
export type UserId = number;
export type Callback<T> = fn(T): void;
```

### Exported Constants

```atlas
export const MAX_CONNECTIONS = 100;
export const BASE_URL: string = "https://api.example.com";
```

### Exported Structs

```atlas
export struct User {
    id: number,
    name: string,
    email: string,
}
```

### Exported Enums

```atlas
export enum Status {
    Active,
    Inactive,
    Pending,
}
```

### What Can Be Exported

Valid `export` targets:

| Syntax | Exported Item |
|--------|--------------|
| `export fn ...` | Named function |
| `export async fn ...` | Async function |
| `export let ...` | Variable |
| `export const ...` | Compile-time constant |
| `export type ...` | Type alias |
| `export struct ...` | Struct declaration |
| `export enum ...` | Enum declaration |

Traits and impl blocks cannot be exported directly with the `export` keyword.

---

## Visibility Modifiers

All declarations carry a visibility level independent of the module system:

| Modifier | Scope |
|----------|-------|
| `pub` | Accessible from any module |
| `private` | File-private (default) |
| `internal` | Module-internal (all files in the same module) |

The default visibility when no keyword is written is `private`.

```atlas
pub fn publicHelper(): void { }       // any module can import this
fn privateHelper(): void { }          // same file only (default)
internal fn moduleHelper(): void { }  // same module only
```

`pub` is a visibility declaration on the item itself. `export` registers the item in the module system. In practice, `export fn foo()` automatically treats the function as public for external callers.

---

## Module Loading

The module loader:
1. Parses the entry module and extracts its import declarations.
2. Resolves each import path relative to the importing file.
3. Recursively loads all dependencies.
4. Detects circular imports and reports them as errors.
5. Returns all modules in topological order (dependencies initialized first).

Parse errors are collected across **all** modules in one pass — the loader does not stop at the first file with errors. A single `atlas run` surfaces all parse errors in the import graph simultaneously.

### Circular Import Detection

Circular imports produce a compile-time error and halt compilation. The loader uses DFS to find cycles before executing any module.

---

## Full Example

```atlas
// math.atl
export fn add(x: number, y: number): number {
    return x + y;
}

export fn multiply(x: number, y: number): number {
    return x * y;
}

export const PI: number = 3.14159265358979;
```

```atlas
// geometry.atl
import { multiply, PI } from "./math";

export fn circleArea(radius: number): number {
    return multiply(PI, multiply(radius, radius));
}
```

```atlas
// main.atl
import { circleArea } from "./geometry";
import * as math from "./math";

let area = circleArea(5.0);
console.log(area.toString());
console.log(math.PI.toString());
```

---

## Gotchas

**No re-exports.** Atlas does not support `export { x } from "./other"` syntax. To re-export, import and re-declare.

**No default exports.** There is no `export default`. All exports are named.

**No renaming on import.** `import { add as plus }` is not supported. The imported name must match the export name.

**No dynamic imports.** `import()` call syntax does not exist. All imports are static and resolved at load time.

**Import order does not matter.** The module loader handles dependency ordering through topological sort.

**Named import mismatch is a compile error.** Importing a name that does not exist in the target module's exports fails at compile/check time, not at runtime.
