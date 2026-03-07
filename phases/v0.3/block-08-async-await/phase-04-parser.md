# Phase 04: Parser

## Dependencies

**Required:** Phase 03 complete (AST nodes exist — FunctionDecl.is_async, Expr::Await, TypeRef::Future)

**Verification:**
```bash
grep "TokenKind::Async\|TokenKind::Await" crates/atlas-runtime/src/parser/mod.rs
cargo check -p atlas-runtime
```

**If missing:** AST nodes must exist before the parser can construct them.

---

## Objective

Wire the parser to produce async function declarations and await expressions from the token stream. Enforce syntactic well-formedness; leave semantic enforcement (await-outside-async) to the typechecker.

---

## Files

**Update:** `crates/atlas-runtime/src/parser/mod.rs` (~60 lines)
**Tests:** `crates/atlas-runtime/tests/frontend_syntax/parser_basics.rs` (~20 test cases)
**Tests:** `crates/atlas-runtime/tests/frontend_integration/integration_part_1.rs` (~5 test cases)

**Total new code:** ~60 lines parser, ~80 lines tests
**Total tests:** ~25 test cases

---

## Dependencies (Components)

- `parser/mod.rs` — function declaration parser, expression parser, type parser (existing)
- `token.rs` — TokenKind::Async, TokenKind::Await (Phase 02)
- `ast.rs` — FunctionDecl.is_async, Expr::Await, TypeRef::Future (Phase 03)

---

## Implementation Notes

**Key patterns to analyze:**
- Find where `TokenKind::Fn` is matched in function declaration parsing — `async` is a prefix that appears before `fn`
- Find where unary prefix expressions are parsed — `await` follows the same prefix-operator pattern
- Find where named types are parsed in the TypeRef parser — `Future<T>` is parsed as a generic named type

**Critical requirements:**
- `async fn` requires `async` immediately followed by `fn` — any other token after `async` is a parse error
- `await` binds tighter than binary operators but looser than method calls and field access — same precedence level as other unary prefix operators
- `Future<T>` in type position is parsed as a generic named type using the existing generic type parsing path; `TypeRef::Future` is constructed only when the name is exactly `"Future"` with one type argument
- Top-level `await expr` as a statement must be accepted syntactically — the typechecker decides if it is valid
- `await` on the right-hand side of a `let` binding must be accepted

**Error handling:**
- `async` not followed by `fn` → parse error with clear message ("expected `fn` after `async`")
- `await` with no following expression → parse error ("expected expression after `await`")
- No AT codes from parser — AT4001 and AT4002 are typechecker errors

**Integration points:**
- Uses: TokenKind::Async, TokenKind::Await (Phase 02)
- Constructs: FunctionDecl { is_async: true }, Expr::Await, TypeRef::Future (Phase 03)

---

## Tests (TDD Approach)

**Parse: async fn declarations** (6 tests)
1. `async fn foo() -> void` parses with `is_async: true`
2. `async fn fetch(url: string) -> string` — with typed parameters
3. `async fn` inside an impl block → `MethodDecl { is_async: true }`
4. `async fn` with generic type parameters
5. Non-async `fn foo()` still parses with `is_async: false` (regression guard)
6. `async 42` → parse error

**Parse: await expressions** (8 tests)
1. `await foo()` → Expr::Await wrapping a call expression
2. `let x = await bar()` → await in let binding
3. `await foo().method()` — method chain resolves after await
4. `await await foo()` — nested await (syntactically valid)
5. Top-level `await foo()` as a statement
6. `return await foo()` — await in return statement
7. `await foo() + 1` — await result used in binary expression
8. `await` with no expression → parse error

**Parse: Future<T> type** (4 tests)
1. `let f: Future<number>` → TypeRef::Future with inner TypeRef::Number
2. `-> Future<string>` as an explicit function return type
3. `Future<Result<number, string>>` — nested generics
4. `Future` alone without angle brackets → TypeRef::Named, not TypeRef::Future

**Parse: error cases** (7 tests)
1. `async async fn` → error
2. `async` not followed by fn → error
3. `await` with no following expression → error
4. Existing non-async function tests still parse correctly (regression)
5. async fn with no body → error
6. await in type position → error
7. `async fn main()` → parse succeeds (typechecker emits AT4006, not parser)

**Minimum test count:** 25 tests

**Parity requirement:** N/A — parser produces AST, both engines consume the same AST.

---

## Acceptance Criteria

- ✅ `async fn` parses to `FunctionDecl { is_async: true }`
- ✅ `await expr` parses to `Expr::Await`
- ✅ `Future<T>` in type position parses to `TypeRef::Future`
- ✅ Top-level await accepted syntactically
- ✅ All parse error cases produce correct messages
- ✅ 25+ parser tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030
**Specifications:** docs/language/async.md (syntax section)
**Related phases:** Phase 03 (AST nodes), Phase 05 (Value::Future), Phase 07 (typechecker enforces await-position semantics)
