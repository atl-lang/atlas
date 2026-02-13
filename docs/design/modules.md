# Module System Design

**Status:** Approved Design (v0.2)
**Last Updated:** 2026-02-13

---

## Objective

Add module system to Atlas for multi-file programs. Enables imports, exports, namespace management, and proper code organization. Foundation for package management.

---

## Syntax

### Export Declarations

**Export function:**
```atlas
export fn add(a: number, b: number) -> number {
    return a + b;
}
```

**Export variable:**
```atlas
export let PI = 3.14159;
export var counter = 0;
```

**Multiple exports:**
```atlas
export fn add(a: number, b: number) -> number { return a + b; }
export fn subtract(a: number, b: number) -> number { return a - b; }
export let ZERO = 0;
```

### Import Declarations

**Named imports:**
```atlas
import { add, subtract } from "./math";
import { PI } from "./constants";
```

**Namespace import:**
```atlas
import * as math from "./math";

// Usage: math.add(2, 3)
```

**Mixed imports (v0.3+):**
```atlas
// NOT in v0.2:
import defaultExport, { named1, named2 } from "./module";
```

---

## Module Resolution

### File Extensions

**Atlas modules:** `.atl` files

**Resolution rules:**
1. Exact path with extension: `import from "./math.atl"` → `./math.atl`
2. Path without extension: `import from "./math"` → `./math.atl`
3. If not found, error

**No automatic index.atl resolution in v0.2**

### Path Types

**Relative paths:**
```atlas
import { x } from "./sibling";        // Same directory
import { y } from "../parent";        // Parent directory
import { z } from "../../grandparent"; // Two levels up
```

**Absolute paths (from project root):**
```atlas
import { config } from "/src/config";
// Resolves from project root (where atlas.toml is)
```

**Package imports (v0.3+):**
```atlas
// NOT in v0.2:
import { http } from "http-client";  // From packages
```

---

## Module Semantics

### Module Evaluation

**Single evaluation:** Each module evaluated exactly once, first time imported

**Caching:** Module exports cached by absolute path

**Initialization order:** Topological sort based on dependency graph

**Example:**
```
// main.atl imports a.atl and b.atl
// a.atl imports c.atl
// b.atl imports c.atl

Evaluation order: c.atl → a.atl → b.atl → main.atl
c.atl executed ONCE even though imported twice
```

### Circular Dependencies

**Detection:** Required (compile error, not runtime)

**Example (ERROR):**
```atlas
// a.atl
import { b_func } from "./b";
export fn a_func() { return b_func(); }

// b.atl
import { a_func } from "./a";  // ERROR: Circular dependency
export fn b_func() { return a_func(); }
```

**Error message:** "Circular dependency detected: a.atl → b.atl → a.atl"

### Export Visibility

**Exports only:** Only explicitly exported items visible to importers

**Private by default:** All non-exported items are module-private

**Example:**
```atlas
// math.atl
fn internal_helper() { return 42; }  // Private

export fn add(a: number, b: number) -> number {
    return a + b;  // Public
}

// main.atl
import { add } from "./math";
add(2, 3);           // ✅ OK
internal_helper();   // ❌ Error: not exported
```

### Namespace Objects (import * as)

**Creates object with all exports:**
```atlas
import * as math from "./math";

// math is object-like namespace
math.add(2, 3);
math.PI;
```

**Type:** Special namespace type (not regular object)

**Immutable:** Cannot reassign properties

---

## Grammar (EBNF Extension)

```ebnf
(* Module declarations at top level *)
program        = { module_item } ;
module_item    = export_decl | import_decl | decl_or_stmt ;

(* Export declarations *)
export_decl    = "export" ( fn_decl | var_decl ) ;

(* Import declarations *)
import_decl    = "import" import_clause "from" string ";" ;
import_clause  = named_imports | namespace_import ;
named_imports  = "{" import_specifiers "}" ;
import_specifiers = import_specifier { "," import_specifier } ;
import_specifier  = ident ;
namespace_import  = "*" "as" ident ;

(* No default imports in v0.2 *)
```

---

## Module Loading

### Module Registry

**Stores loaded modules:**
```rust
struct ModuleRegistry {
    modules: HashMap<PathBuf, Module>,  // Absolute path → Module
    loading: HashSet<PathBuf>,           // Currently loading (for cycle detection)
}

struct Module {
    path: PathBuf,
    ast: Program,
    exports: HashMap<String, Symbol>,
    dependencies: Vec<PathBuf>,
    initialized: bool,
}
```

### Loading Algorithm

**High-level:**
1. Parse import statement → get module path
2. Resolve to absolute path
3. Check registry cache → if cached, return
4. Check loading set → if in set, circular dependency error
5. Add to loading set
6. Read and parse module file
7. Recursively load dependencies
8. Remove from loading set
9. Add to registry
10. Return module

**Pseudocode:**
```
load_module(path):
    abs_path = resolve_path(path)

    if abs_path in registry:
        return registry[abs_path]

    if abs_path in loading:
        error("Circular dependency")

    loading.add(abs_path)

    source = read_file(abs_path)
    ast = parse(source)

    for import in ast.imports:
        load_module(import.path)

    loading.remove(abs_path)
    registry[abs_path] = Module { ast, ... }

    return registry[abs_path]
```

### Execution Order

**Topological sort:** Execute modules in dependency order

**Example:**
```
Dependencies:
main.atl → [a.atl, b.atl]
a.atl → [c.atl]
b.atl → [c.atl]

Topological order: c.atl, a.atl, b.atl, main.atl
```

**Algorithm:** Depth-first search with visited tracking

---

## Type System Integration

### Cross-Module Type Checking

**Exports have types:**
```atlas
// math.atl
export fn add(a: number, b: number) -> number { ... }

// main.atl
import { add } from "./math";
// Type checker knows: add: (number, number) -> number
```

**Import type checking:**
- Verify imported name exists in module
- Verify imported name is exported
- Import symbol type from export

### Symbol Table per Module

**Each module has own scope:**
```atlas
// a.atl
let x = 10;          // Module-private
export let y = 20;   // Exported

// b.atl
let x = 30;          // Different x, no conflict
import { y } from "./a";
```

**No global namespace pollution**

### Type References

**Imported types available:**
```atlas
// types.atl (v0.3+ when user-defined types exist)
export struct Point { x: number, y: number }

// main.atl
import { Point } from "./types";
let p: Point = { x: 0, y: 0 };
```

**v0.2:** Only value imports (functions, variables)
**v0.3+:** Type imports when user-defined types added

---

## Implementation Strategy

### Phase 1: Syntax & Resolution (1 week)
- Add ImportDecl, ExportDecl to AST
- Parser for import/export syntax
- Path resolution algorithm
- Circular dependency detection
- Tests: parsing, resolution

### Phase 2: Loading & Caching (1 week)
- Module registry
- Module loader
- Dependency graph
- Topological sort
- Tests: loading, caching, initialization order

### Phase 3: Type System Integration (1 week)
- Binder: cross-module symbol resolution
- Type checker: validate imports/exports
- Symbol table per module
- Tests: cross-module type checking

### Phase 4: Runtime Implementation (1 week)
- Interpreter: module execution
- VM: module linking
- Module initialization
- Tests: runtime execution, parity

**Total:** 4 weeks (matches BLOCKER 04 estimate)

---

## Examples

### Basic Module

**math.atl:**
```atlas
export fn add(a: number, b: number) -> number {
    return a + b;
}

export fn subtract(a: number, b: number) -> number {
    return a - b;
}

export let PI = 3.14159;
```

**main.atl:**
```atlas
import { add, PI } from "./math";

let result = add(2, 3);
print(str(result));  // 5

let circle_area = PI * 2 * 2;
print(str(circle_area));  // ~12.566
```

### Namespace Import

**math.atl:** (same as above)

**main.atl:**
```atlas
import * as math from "./math";

let sum = math.add(10, 20);
let diff = math.subtract(10, 5);
print(str(math.PI));
```

### Multi-File Structure

```
project/
├── atlas.toml
├── main.atl
├── math/
│   ├── basic.atl
│   └── advanced.atl
└── utils/
    └── helpers.atl
```

**math/basic.atl:**
```atlas
export fn add(a: number, b: number) -> number {
    return a + b;
}
```

**main.atl:**
```atlas
import { add } from "./math/basic";
// Relative paths work across directories
```

---

## Limitations (v0.2)

### No Default Exports

**Cannot export default value:**
```atlas
// NOT in v0.2:
export default function main() { ... }
```

**v0.3+:** Default exports if justified

### No Re-Exports

**Cannot re-export from another module:**
```atlas
// NOT in v0.2:
export { add, subtract } from "./math";
```

**Workaround:** Import then export separately
```atlas
import { add, subtract } from "./math";
export fn add_wrapper(a: number, b: number) -> number {
    return add(a, b);
}
```

**v0.3+:** Re-export syntax

### No Dynamic Imports

**All imports static (at top level):**
```atlas
// NOT in v0.2:
if (condition) {
    import { x } from "./conditional";  // Error
}

let module = import("./dynamic");  // Error
```

**v0.3+:** Dynamic import() if needed

### No Package Imports

**Only file paths, not package names:**
```atlas
// NOT in v0.2:
import { http } from "http-client";
```

**v0.2:** Relative/absolute paths only
**v0.3+:** Package resolution (BLOCKER 05 + Foundation Phase 7-8)

### No Type-Only Imports

**No distinction between value and type imports:**
```atlas
// NOT in v0.2:
import type { Point } from "./types";
```

**v0.2:** Only value imports (no user-defined types yet)
**v0.3+:** Type imports when needed

---

## Rationale

### Why ES Module Syntax?

**Alternatives considered:**
1. **CommonJS** (Node.js)
   - `require()` and `module.exports`
   - Cons: Dynamic, harder to analyze

2. **Rust modules**
   - `mod`, `use`, explicit file tree
   - Cons: More complex, less familiar

3. **ES Modules** (JavaScript) ✅ **CHOSEN**
   - `import`/`export` syntax
   - Pros: Familiar, static analysis, explicit

**Decision:** ES modules are widely known, explicit, and enable static analysis (tree-shaking, etc.)

### Why Circular Detection?

**Prevent subtle bugs:**
```atlas
// Without detection, infinite loop at initialization
// a.atl
import { b } from "./b";
export let a = b + 1;

// b.atl
import { a } from "./a";
export let b = a + 1;
```

**Better to fail at compile time than runtime**

### Why Single Evaluation?

**Consistency:** Module state consistent across all importers

**Example:**
```atlas
// counter.atl
export var count = 0;
export fn increment() { count = count + 1; }

// a.atl
import { count, increment } from "./counter";
increment();  // count is now 1

// b.atl
import { count } from "./counter";
// Sees count = 1 (same module instance)
```

**Alternative (per-import evaluation):** Each import gets separate copy → confusing

---

## Project Root Detection

**Project root:** Directory containing `atlas.toml`

**Algorithm:**
1. Start from current file's directory
2. Walk up directory tree
3. Find first `atlas.toml`
4. That directory is project root

**Absolute paths resolve from project root:**
```atlas
// In /home/user/project/src/sub/file.atl
import { x } from "/src/utils";
// Resolves to: /home/user/project/src/utils.atl
```

---

## Verification

**After implementation, verify:**
1. ✅ Parse import/export syntax
2. ✅ Path resolution works (relative, absolute)
3. ✅ Module loading works
4. ✅ Circular dependencies detected
5. ✅ Single evaluation enforced
6. ✅ Exports visible to importers
7. ✅ Non-exports are private
8. ✅ Cross-module type checking works
9. ✅ Interpreter executes modules correctly
10. ✅ VM executes modules correctly
11. ✅ 100% parity between engines

---

## References

**Inspiration:**
- ES Modules: Syntax, static structure
- Rust: Explicit module system, path clarity
- Python: Simple imports, clear errors
- TypeScript: Type checking across modules

**Atlas Philosophy:**
- Explicit over implicit (no magic path resolution)
- Static analysis friendly (no dynamic imports in v0.2)
- AI-optimized (clear error messages for missing modules)

---

**Design Status:** ✅ Complete and approved
**Ready for Implementation:** Yes (BLOCKER 04)
