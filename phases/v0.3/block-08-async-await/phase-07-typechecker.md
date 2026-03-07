# Phase 07: Typechecker

## Dependencies

**Required:** Phase 03 (AST), Phase 01 (AT codes registered)

**Verification:**
```bash
grep "AT4001\|AT4002\|is_async\|TypeRef::Future" crates/atlas-runtime/src/typechecker/
cargo check -p atlas-runtime
```

---

## Objective

Implement all async/await type rules: async fn return type inference/validation, await-in-async enforcement, Future<T> unwrapping via await, and all AT4001–AT4010 error emissions.

---

## Files

**Update:** `crates/atlas-runtime/src/typechecker/mod.rs`
  - `check_function`: detect `is_async`, wrap inferred/declared return as `Future<T>`
  - Track async context in typechecker state (`in_async_context: bool`)
**Update:** `crates/atlas-runtime/src/typechecker/expr.rs`
  - `Expr::Await`: verify operand is `Future<T>`, yield type `T`; emit AT4001 if not in async/top-level; emit AT4002 if operand not Future
  - `Expr::Call` on async fn: result type is `Future<T>`, not `T`
**Update:** `crates/atlas-runtime/src/typechecker/inference.rs`
  - `infer_return_type`: async fn body infers `T`, return type becomes `Future<T>`
**Tests:** `crates/atlas-runtime/tests/typesystem/` (+25 test cases)

**Total new code:** ~120 lines typechecker, ~100 lines tests
**Total tests:** ~25 test cases

---

## Implementation Notes

**Async context tracking:** Thread a boolean through the check context — `in_async: bool`. Set to `true` when entering `async fn` body. `Expr::Await` checks this flag.

**Top-level await:** Top-level is always considered an async context (like Python `asyncio.run`). The typechecker must know if it's at top-level scope.

**Return type rules:**
```
async fn foo() -> number   →  actual return type: Future<number>
async fn foo()             →  inferred return type: Future<T> where T = inferred body type
fn foo() -> Future<number> →  explicit, treated identically to async fn → number
```

**`await` type rules:**
```
await expr  where expr: Future<T>  →  yields T
await expr  where expr: T (not Future)  →  AT4002
await expr  outside async fn  →  AT4001
```

**Call type of async fn:**
```
async fn foo() -> string
foo()  →  type: Future<string>  (not string — call returns the future)
await foo()  →  type: string
```

---

## Tests

**Valid async programs:** (10 tests)
1. `async fn` return type is `Future<T>` when called
2. `await` on `Future<T>` yields `T`
3. `async fn` with no return type — inferred as `Future<void>`
4. `let f: Future<number> = async_fn()` — explicit Future annotation
5. `return await other_async()` inside async fn
6. Top-level `await` accepted without async fn wrapper
7. `await foo() + 1` — await result used in arithmetic
8. Chained: `await (await f)` — nested awaits
9. async fn with `?` operator on `Future<Result<T,E>>`
10. async method in impl block

**Error cases — AT codes:** (10 tests)
1. AT4001: `await` outside async fn (non-top-level)
2. AT4002: `await 42` — non-Future
3. AT4002: `await "hello"` — non-Future string
4. AT4003: return type mismatch in async fn
5. AT4006: `async fn main()` → error
6. AT4009: `async` on anonymous fn → error (reserved)
7. AT4005: Future<T> used as T without await (warning)
8. AT4010: await inside sync for loop
9. AT4008: Future<number> vs Future<string> type mismatch
10. AT4004: async fn passed to sync fn param

**Regression:** (5 tests)
1. Non-async functions unaffected — is_async=false has no type change
2. Regular `fn -> string` return type unchanged
3. Existing error codes unaffected

**Minimum test count:** 25 tests

---

## Acceptance Criteria

- ✅ async fn calls return `Future<T>` type
- ✅ `await` on Future<T> yields T
- ✅ AT4001 emitted for await-outside-async
- ✅ AT4002 emitted for await-non-future
- ✅ AT4006 emitted for async main
- ✅ Top-level await accepted
- ✅ Return type inference works for async fn
- ✅ 25+ typechecker tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030
**Spec:** docs/language/async.md (type system + semantics sections)
**Related phases:** Phase 04 (parser), Phase 08 (compiler uses type info), Phase 09/10 (engines)
