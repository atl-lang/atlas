# Phase 05: Generic Call-Site Inference

**Block:** 5 (Type Inference)
**Depends on:** Phase 02 + Phase 03 complete

## Goal

`identity(42)` should work without `identity::<number>(42)`. The typechecker infers `T=number`
from the argument type.

## Current state

The monomorphizer (`src/typechecker/generics.rs`) already mangles generic calls. Call-site
type argument inference is the missing piece — when type args are omitted, infer them from
the argument types.

## Implementation

In `typechecker/expr.rs`, `check_call_against_signature`:

When `call.type_args` is empty AND the function has type params, attempt inference:

```rust
if call.type_args.is_empty() && !type_params.is_empty() {
    // Infer type args from argument types
    let inferred_args = self.infer_type_args_from_call(call, type_params, param_types)?;
    // Proceed with inferred_args as if they were explicit
}
```

`infer_type_args_from_call`: for each type param `T`, find the first function param that
uses `T` in its type, look at the corresponding call argument type, bind `T = arg_type`.

## Scope boundary

Only positional inference from direct param-to-arg matching. No constraint propagation,
no unification across multiple params. If inference is ambiguous, require explicit annotation.

## Examples that must work

```atlas
fn identity<T>(x: T) -> T { return x; }
identity(42);           // T=number, returns number
identity("hello");      // T=string, returns string

fn first<T>(arr: T[]) -> T { return arr[0]; }
first([1, 2, 3]);       // T=number
```

## Error for ambiguous inference

```atlas
fn pair<T, U>(x: T, y: U) -> T { return x; }
pair(1, "a");   // T=number, U=string — both inferrable ✓
```

If a type param appears ONLY in the return type (not in params), require explicit annotation:
AT3051 "cannot infer type argument `T` — add explicit type annotation `<T>`".

## Acceptance Criteria

- [ ] `identity(42)` infers `T=number` without explicit type arg
- [ ] `first([1,2,3])` infers `T=number`
- [ ] AT3051 emitted when type param is return-only (cannot infer from args)
- [ ] Existing explicit `identity::<number>(42)` still works
- [ ] Minimum 6 new tests
