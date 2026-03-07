# Phase 02: Keywords — Lexer + Token

## Dependencies

**Required:** Phase 01 (spec complete, AT codes registered)

**Verification:**
```bash
grep "Async\|Await" crates/atlas-runtime/src/token.rs
grep '"async"\|"await"' crates/atlas-runtime/src/lexer/mod.rs
cargo check -p atlas-runtime
```

---

## Objective

Add `async` and `await` as reserved keywords to the lexer and token system. They must be rejected as identifiers and correctly tokenized in all positions.

---

## Files

**Update:** `crates/atlas-runtime/src/token.rs` (+4 lines — two variants + as_str + is_keyword)
**Update:** `crates/atlas-runtime/src/lexer/mod.rs` (+2 lines — keyword map entries)
**Tests:** `crates/atlas-runtime/tests/frontend_syntax/lexer.rs` (+8 test cases)

**Total new code:** ~14 lines
**Total tests:** ~20 lines (8 test cases)

---

## Implementation Notes

**Pattern:** Follow existing keyword additions (e.g., `own`, `borrow`, `shared` from B2):
- Add `TokenKind::Async` and `TokenKind::Await` variants
- Add to `is_keyword()` match arm
- Add to `as_str()` match arm: `"async"` and `"await"`
- Add to keyword map in `lexer/mod.rs`: `"async" => TokenKind::Async`, `"await" => TokenKind::Await`

**Critical:** These must be rejected as identifiers — verify `is_keyword()` returns `true` for both.

---

## Tests

**Lexer tokenization:** (4 tests)
1. `async` tokenizes as `TokenKind::Async`
2. `await` tokenizes as `TokenKind::Await`
3. `async` rejected as variable name (parser error, but lexer produces Async token)
4. `await` rejected as variable name

**Keyword identity:** (4 tests)
1. `is_keyword("async")` returns true
2. `is_keyword("await")` returns true
3. `as_str(TokenKind::Async)` returns `"async"`
4. `as_str(TokenKind::Await)` returns `"await"`

**Minimum test count:** 8 tests

---

## Acceptance Criteria

- ✅ `TokenKind::Async` and `TokenKind::Await` added
- ✅ Both appear in `is_keyword()` and `as_str()`
- ✅ Keyword map entries in lexer
- ✅ 8+ lexer tests pass
- ✅ `cargo check -p atlas-runtime` clean
- ✅ No clippy warnings

---

## References

**Decision Logs:** D-030
**Spec:** docs/language/async.md
**Related phases:** Phase 01 (AT codes), Phase 03 (AST uses these tokens), Phase 04 (Parser)
