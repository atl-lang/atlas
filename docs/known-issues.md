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

### H-110 / H-111: Enum Match Typechecker (FIXED 2026-03-07)
**Was:** Matching on user-defined enums inside function bodies returned `?` (unknown type). Double-match on same enum at top level also produced `?` on second match.
**Now:** Enum match correctly resolves to the named enum type. `enum_decls` registry added to TypeChecker.

### H-112: hashMapHas / hashSetHas Return Type (FIXED 2026-03-07)
**Was:** `hashMapHas` and `hashSetHas` returned `any` from the typechecker — couldn't use result in `if` condition.
**Now:** Both return `Type::Bool`.

### H-113: hashSetRemove CoW Semantics (FIXED 2026-03-07)
**Was:** `s = hashSetRemove(s, x)` replaced `s` with `Bool` instead of the updated HashSet — inconsistent with `hashSetAdd`.
**Now:** `hashSetRemove` returns the updated HashSet (CoW, like `hashSetAdd`). Use `hashSetHas` before removing if you need the existence check.

### H-114: Return Inside Match Arm (FIXED 2026-03-07)
**Was:** `return value` inside a match arm body was a parse error.
**Now:** Parser wraps it in an `Expr::Block` — both `return` and expressions work in match arms.

### H-115: If/Else as Expression Type (FIXED 2026-03-07)
**Was:** `if/else` used as an expression returned `?` from the typechecker.
**Now:** Typechecker detects the single-if-with-else-tail-expr pattern in `Expr::Block` and infers the union type.

### H-116: Range Syntax in For-In (FIXED 2026-03-07)
**Was:** `for i in 0..5 { }` and `for i in 1..=5 { }` were not supported.
**Now:** Both interpreter and VM handle range iteration. Root fix: VM's Pop-before-Halt optimization was corrupting for-in cleanup — fixed with a Null+Pop sentinel in `compile_for_in`.

### H-117: Struct Array Fn Param Types (FIXED 2026-03-07)
**Was:** `fn foo(items: MyStruct[])` typed the param as `?[]` — struct type inside array annotation was lost.
**Now:** TypeChecker updates symbol table with resolved param types after struct resolution pass.

### H-120: Enum Tuple Pattern Binding Types (FIXED 2026-03-07)
**Was:** Variables bound inside `Pattern::EnumVariant` (e.g., `Shape::Circle(r)`) were typed `?` — no field type registry existed.
**Now:** `enum_decls` HashMap added to TypeChecker; `enum_variant_field_types()` resolves binding types from the declaration.

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
