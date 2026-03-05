# Known Issues

Current limitations in Atlas. This is the honest truth.

**Source of truth:** `atlas-track issues` — this file is a quick reference.
**Full details:** `atlas-track issue H-XXX`
**Search:** `atlas-track search "keyword"`

## P0 - Critical Blockers

### H-069: Closure Global Mutations
**Status:** In Progress
**Problem:** Closures passed as function parameters don't persist mutations to global mutable arrays/state.
**Workaround:** Avoid callback-based patterns. Use imperative style instead of `describe(fn() { ... })`.

## P1 - Open Issues

### H-070: Trait System Incomplete
**Status:** Open (parent issue)
Trait parsing and basic dispatch work. Missing: implicit self inference (H-073), default methods (H-074), trait objects as parameter types (H-075), method syntax for stdlib (H-065).
**What works now:** `trait X { fn foo(self: X) -> T; }` + `impl X for MyStruct { ... }` + `instance.foo()`.
**What doesn't:** `fn foo(self)` (must write `self: Type`), `fn process(x: SomeTrait)`, default method bodies.

### H-065: Stdlib Method Syntax
**Status:** Open
Stdlib uses global functions (`arrayPush(arr, x)`) not method syntax (`arr.push(x)`).
**Workaround:** Use `arrayPush()`, `hashMapGet()`, `len()` etc. See `docs/stdlib/`.

## P2 - Tracked

- **H-041:** Param.type_ref cleanup (parser)
- **H-040:** Structural type syntax evaluation (typechecker)
- **H-076:** Trait inheritance (typechecker)
- **H-077:** Generic traits (typechecker)

## Recently Fixed

### H-072: Struct Trait Dispatch (FIXED 2026-03-05)
**Was:** `impl Greetable for Person` registered but `p.greet()` failed — typechecker couldn't map struct types.
**Now:** Trait method dispatch works on all struct types.

### H-071: Diagnostic Shows Struct Names (FIXED 2026-03-05)
**Was:** Error said `Type '{ name: string }'` instead of `Type 'Person'`.
**Now:** Errors use the declared struct name.

### H-068: fn main() Entry Point (FIXED 2026-03-05)
**Was:** `fn main() { ... }` didn't execute.
**Now:** Zero-arg `fn main()` auto-executes after top-level statements.

### H-067: .atl Extension (FIXED 2026-03-05)
**Was:** `.atl` files didn't execute.
**Now:** Both `.atlas` and `.atl` work.

### H-066: Struct Field Access (FIXED 2026-03-05)
**Was:** Struct field access returned `?` instead of declared type.
**Now:** `item.id` correctly returns `number` if declared as such.

### H-064: HashMap Generic Enforcement (FIXED 2026-03-04)
**Was:** `HashMap<K,V>` generics were cosmetic.
**Now:** Type annotations are enforced on all HashMap operations.

### H-062: Array<T> vs T[] (FIXED 2026-03-04)
**Was:** `Array<T>` and `T[]` were separate types.
**Now:** Unified — both are interchangeable.

## Reporting Issues

Found a bug? Document it in battle test `audit/FRICTION.md` or report to the Atlas repo.
