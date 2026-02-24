# Phase 03: Typechecker — Return Type Inference

**Block:** 5 (Type Inference)
**Depends on:** Phase 01 + Phase 02 complete

## Goal

When `func.return_type.is_none()`, infer the return type from the function body using
`infer_return_type` from `inference.rs`. Wire this into `check_function`.

## Implementation

In `typechecker/mod.rs`, `check_function`:

```rust
let return_type = match &func.return_type {
    Some(type_ref) => self.resolve_type_ref(type_ref),
    None => {
        // Infer from body — runs before body type-checking begins
        let inferred = crate::typechecker::inference::infer_return_type(&func.body);
        inferred.ty  // Type::Unknown if cannot be determined
    }
};
self.current_function_return_type = Some(return_type.clone());
```

The inferred type is then used for return statement validation — `return x` inside the
function is checked against the inferred return type, same as explicit annotations.

## Error message for inference failure

When `infer_return_type` returns `Type::Unknown` and a `return x` statement exists:
emit AT3050 "cannot infer return type — add an explicit `-> T` annotation".

## Scope boundary

Only return type inference for named functions. Anonymous fn return type is already handled
in `check_anon_fn` (Block 4). Do not merge those paths.

## Examples that must work

```atlas
fn double(x: number) { return x * 2; }     // infers -> number
fn greet(name: string) { return "hi " + name; }  // infers -> string
fn noop() { }                               // infers -> void
```

## Acceptance Criteria

- [ ] `fn f(x: number) { return x * 2; }` typechecks without AT3001 error
- [ ] `fn f() { return 42; }` infers return type `number`
- [ ] `fn f() { }` (no return) infers return type `void`
- [ ] AT3050 emitted when return type genuinely cannot be inferred
- [ ] Both engines execute inferred-return-type functions correctly
- [ ] Minimum 6 new tests
