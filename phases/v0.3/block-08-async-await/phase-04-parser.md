# Phase 04: Parser

## Dependencies

**Required:** Phase 03 (AST nodes), Phase 02 (keywords)

**Verification:**
```bash
grep "parse_async\|TokenKind::Async\|TokenKind::Await" crates/atlas-runtime/src/parser/mod.rs
cargo check -p atlas-runtime
```

---

## Objective

Wire the parser to produce `async` function declarations and `await` expressions from the token stream. Enforce syntactic constraints (await position).

---

## Files

**Update:** `crates/atlas-runtime/src/parser/mod.rs`
  - Function declaration parsing: detect `async` prefix, set `is_async: true`
  - `impl` method parsing: same `async` prefix detection
  - Expression parsing: `await <expr>` → `Expr::Await`
  - TypeRef parsing: `Future<T>` → `TypeRef::Future`
  - Top-level `await` at statement level
**Tests:** `crates/atlas-runtime/tests/frontend_syntax/parser_basics.rs` (+20 cases)
        `crates/atlas-runtime/tests/frontend_integration/integration_part_1.rs` (+5 cases)

**Total new code:** ~60 lines parser, ~80 lines tests
**Total tests:** ~25 test cases

---

## Implementation Notes

**`async fn` parsing:**
```
peek TokenKind::Async → consume → expect TokenKind::Fn → parse_function_decl with is_async: true
```

**`await` expression:**
```
peek TokenKind::Await → consume → parse_expr(precedence) → Expr::Await { expr, span }
```
`await` has high precedence (like unary prefix ops). `await foo.bar()` = `await (foo.bar())`.

**`Future<T>` in TypeRef:**
```
"Future" identifier → peek `<` → parse generic arg → TypeRef::Future { inner }
```

**Top-level await:** Parser must allow `Stmt::Expr(Expr::Await { .. })` at top-level without error.

**Syntactic enforcement NOT in parser** (leave to typechecker):
- await-outside-async is a type error, not a parse error — parser accepts it everywhere

---

## Tests

**Parse: async fn** (6 tests)
1. `async fn foo() -> void { }` → FunctionDecl { is_async: true }
2. `async fn fetch(url: string) -> string { }` — with params
3. `async fn` in impl block → MethodDecl { is_async: true }
4. `async fn` with generic params `<T>`
5. `fn foo()` (non-async) → is_async: false (regression guard)
6. `async` alone (not followed by fn) → parse error

**Parse: await expr** (8 tests)
1. `await foo()` → Expr::Await wrapping Call
2. `let x = await bar()` → let binding
3. `await foo().method()` — method chain after await
4. `await await foo()` — nested await
5. Top-level `await foo()` as statement
6. `await` without expression → parse error
7. `return await foo()` — in return stmt
8. `await foo() + 1` — await in binary expr (precedence)

**Parse: Future<T> type** (4 tests)
1. `let f: Future<number>` → TypeRef::Future
2. `fn foo() -> Future<string>` — explicit return type
3. `Future<Result<number, string>>` — nested generics
4. `Future` alone (no angle bracket) — treated as named type, not Future<T>

**Parse: error cases** (7 tests)
1. `async async fn` → error
2. `async 42` → error (not fn)
3. Regression: existing non-async tests still parse correctly

**Minimum test count:** 25 tests

---

## Acceptance Criteria

- ✅ `async fn` parses with `is_async: true`
- ✅ `await expr` parses to `Expr::Await`
- ✅ `Future<T>` parses to `TypeRef::Future`
- ✅ Top-level await permitted syntactically
- ✅ All parse error cases produce correct diagnostics
- ✅ 25+ parser tests pass
- ✅ `cargo check -p atlas-runtime` clean

---

## References

**Decision Logs:** D-030
**Spec:** docs/language/async.md (syntax section)
**Related phases:** Phase 03 (AST), Phase 07 (typechecker enforces semantics)
