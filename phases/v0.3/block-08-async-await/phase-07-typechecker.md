# Phase 07: Typechecker

## Dependencies

**Required:** Phase 06 complete (opcodes defined; full AST, parser, and Value::Future all stable)

**Verification:**
```bash
grep "AT4001\|AT4002\|is_async\|TypeRef::Future" crates/atlas-runtime/src/typechecker/
cargo check -p atlas-runtime
```

**If missing:** The typechecker depends on stable AST nodes (Phase 03), parser (Phase 04), and Value::Future type information (Phase 05) — all implied by Phase 06 being complete.

---

## Objective

Implement all async/await type rules: return type inference and validation for async fn, await-position enforcement, Future<T> unwrapping semantics, and emission of AT4001–AT4010 diagnostics.

---

## Files

**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~60 lines)
**Update:** `crates/atlas-runtime/src/typechecker/expr.rs` (~40 lines)
**Update:** `crates/atlas-runtime/src/typechecker/inference.rs` (~20 lines)
**Tests:** `crates/atlas-runtime/tests/typesystem/` (~25 test cases, correct subfile per domain)

**Total new code:** ~120 lines typechecker, ~100 lines tests
**Total tests:** ~25 test cases

---

## Dependencies (Components)

- `typechecker/mod.rs` — function checking, context state (existing)
- `typechecker/expr.rs` — expression type resolution (existing)
- `typechecker/inference.rs` — return type inference (existing)
- AT4001–AT4010 from `error_codes.rs` (Phase 01)
- `Expr::Await`, `TypeRef::Future`, `FunctionDecl.is_async` (Phase 03)

---

## Implementation Notes

**Key patterns to analyze:**
- Find how `check_function` currently tracks context state (e.g., whether we are inside a loop for `break` validation) — use the same mechanism to track `in_async_context: bool`
- Examine `infer_return_type()` in `inference.rs` to understand where the async fn wrapping (`Future<T>`) must be applied to the inferred body type
- Check how call-site type resolution works in `expr.rs` — calling an async fn must produce `Future<T>`, not `T`

**Critical requirements:**
- Async context flag: set to `true` when entering an `async fn` body; top-level scope is always considered an async context
- `Expr::Await` type rule: the operand must have type `Future<T>`; the expression yields type `T`; if operand is not `Future<T>` emit AT4002; if not in async context emit AT4001
- Async fn return type: if declared `-> T` (not `-> Future<T>`), the typechecker treats the actual return type as `Future<T>` — the declared annotation describes the resolved type, and the wrapping is implicit
- Call site type: calling an async fn produces `Future<T>` — the caller must explicitly await to get `T`
- AT4006: `main` function must not be declared `async`
- AT4009: async anonymous functions are not supported — emit this code if `async` appears on an anonymous function

**Error handling:**
- AT4001: await outside async fn and outside top-level
- AT4002: await on non-Future value
- AT4003: body return type incompatible with declared return for async fn
- AT4005: Future value used as T without await (warning — do not fail compilation)
- AT4006: async main
- AT4008: Future<T> and Future<U> type mismatch at assignment or parameter site
- AT4009: async anonymous fn

**Integration points:**
- Uses: all Phase 01–06 artifacts
- Creates: type annotations on call sites (for compiler use in Phase 08)
- Consumed by: compiler (Phase 08), LSP (Phase 14)

---

## Tests (TDD Approach)

**Valid async programs** (10 tests)
1. Calling an async fn produces type `Future<T>` at the call site
2. `await` on `Future<number>` produces type `number`
3. `async fn` with no return annotation infers `Future<void>`
4. `let f: Future<number> = async_fn()` — explicit Future annotation accepted
5. `return await other_async()` inside an async fn body
6. Top-level `await` accepted without an async wrapper
7. `await foo() + 1` — await result used in arithmetic
8. `async fn` in an impl block — method typing correct
9. Async fn with `?` operator on `Future<Result<T, E>>`
10. Non-async fn unaffected — existing return type rules unchanged (regression)

**Error cases** (10 tests)
1. AT4001: `await` inside a non-async fn body
2. AT4002: `await 42` — non-Future literal
3. AT4002: `await "hello"` — non-Future string
4. AT4003: return type mismatch in async fn body
5. AT4005: Future value used as T (should produce a warning, not an error)
6. AT4006: `async fn main()` rejected
7. AT4008: assigning `Future<number>` where `Future<string>` is expected
8. AT4009: `async` on an anonymous function
9. AT4001: `await` in a nested sync closure inside an async fn
10. AT4002: `await null`

**Regression** (5 tests)
1. `fn foo() -> string` return type unchanged by async changes
2. Regular function call still produces `T`, not `Future<T>`
3. Existing AT3xxx codes still fire correctly on non-async programs
4. Type inference for non-async fn unaffected
5. impl block methods without async work as before

**Minimum test count:** 25 tests

**Parity requirement:** Typechecker runs once on the AST before both engines execute — parity is automatically satisfied for type errors.

---

## Acceptance Criteria

- ✅ Async fn calls produce `Future<T>` type at call sites
- ✅ `await` on `Future<T>` resolves to `T`
- ✅ AT4001 emitted for await-outside-async
- ✅ AT4002 emitted for await-non-future
- ✅ AT4006 emitted for async main
- ✅ AT4009 emitted for async anonymous functions
- ✅ Top-level await accepted
- ✅ Return type inference correct for async fn
- ✅ 25+ typechecker tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030
**Specifications:** docs/language/async.md (type system and semantics sections)
**Related phases:** Phase 06 (all prior phases), Phase 08 (compiler uses type annotations from typechecker)
