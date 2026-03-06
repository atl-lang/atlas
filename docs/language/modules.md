# Modules

This document reflects the actual parser in `crates/atlas-runtime/src/parser/mod.rs`.

**Import**
```
import { name1, name2 } from "./path";
import * as namespace from "./path";
```
- Named imports destructure specific exports from the target module.
- Namespace imports bring all exports under a single name.
- Paths are relative to the importing file.

Example (tested):
```atlas
import { detect_protocol, parse_header } from "./transport";
import * as config from "./config";

let proto: string = detect_protocol("HTTP/1.1 200 OK");
let val: string = config.get("timeout");
```

**Export**
```
export fn name(params) -> Type { ... }
export let name: Type = value;
export type Name = Type;
export struct Name { ... }
export enum Name { ... }
```
- Any top-level declaration can be exported.
- Only exported items are visible to importers.

Example (tested):
```atlas
export fn greet(name: string) -> string {
    return `Hello {name}`;
}

export type ID = number | string;
export let VERSION: string = "1.0";
```

**Extern (FFI)**
```
extern "library" fn name(params) -> Type;
extern "library" fn name as "symbol"(params) -> Type;
```
- Declares a function implemented in an external native library.
- The optional `as "symbol"` renames the function for FFI binding.

Example:
```atlas
extern "libcrypto" fn sha256(data: string) -> string;
extern "libz" fn compress as "zlib_compress"(data: string) -> string;
```

**fn main() Entry Point**
- If a file defines `fn main()` with zero parameters, it auto-executes after all top-level statements.
- Top-level statements always run first, regardless of `main()` position.
- If no `main()` exists, only top-level statements execute.

**Current limitations:** See `docs/known-issues.md`
