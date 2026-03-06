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
**Status:** Open (parent issue — most sub-issues now fixed)
Trait parsing, dispatch, default methods, implicit self, and trait objects all work.
**What works now:** `trait X { fn foo(self) -> T; }` (implicit self), default method bodies, `fn process(x: SomeTrait)` trait objects as parameter types, `impl X for MyStruct { ... }` + `instance.foo()`.
**What remains:** method syntax for stdlib (H-065), trait inheritance (H-076), generic traits (H-077).

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

### H-086: Deprecate Anonymous Struct Syntax (FIXED 2026-03-05)
**Was:** `{ x: 1, y: 2 }` accepted as anonymous struct literal — requires fragile 2-token lookahead, ambiguous with blocks.
**Now:** Emits deprecation warning. Use `record { x: 1, y: 2 }` (explicit keyword) or `StructName { x: 1 }` (named instantiation).

### H-080: Enforce No-Parens Condition Syntax (FIXED 2026-03-05)
**Was:** `if (cond)`, `while (cond)`, `for (x in iter)` all accepted silently — inconsistent style.
**Now:** Rust-style enforced: `if cond { }`, `while cond { }`, `for x in iter { }`. Parens emit a deprecation warning.

### H-075: Trait Objects as Parameter Types (FIXED 2026-03-05)
**Was:** `fn process(x: SomeTrait)` didn't work — couldn't use trait names as parameter types.
**Now:** Trait objects work as bounded polymorphism on parameters.

### H-074: Default Method Implementations (FIXED 2026-03-05)
**Was:** Trait methods required `;` — no default bodies.
**Now:** Traits can have default method bodies: `fn foo(self) -> T { default_impl }`.

### H-073: Implicit Self Type Inference (FIXED 2026-03-05)
**Was:** Had to write `fn foo(self: MyType)` — explicit type required on self.
**Now:** Write `fn foo(self)` — type is inferred from the impl block.

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
