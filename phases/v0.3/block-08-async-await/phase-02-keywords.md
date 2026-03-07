# Phase 02: Keywords — Lexer + Token

## Dependencies

**Required:** Phase 01 complete (spec authored, AT codes registered)

**Verification:**
```bash
grep "Async\|Await" crates/atlas-runtime/src/token.rs
grep '"async"\|"await"' crates/atlas-runtime/src/lexer/mod.rs
cargo check -p atlas-runtime
```

**If missing:** Complete Phase 01 first — AT code range must be decided before any implementation begins.

---

## Objective

Add `async` and `await` as reserved keywords. They must tokenize correctly, appear in `is_keyword()`, and be rejected as identifiers throughout the language.

---

## Files

**Update:** `crates/atlas-runtime/src/token.rs` (~4 lines — two enum variants, as_str arms, is_keyword arms)
**Update:** `crates/atlas-runtime/src/lexer/mod.rs` (~2 lines — keyword map entries)
**Tests:** `crates/atlas-runtime/tests/frontend_syntax/lexer.rs` (~8 test cases)

**Total new code:** ~14 lines
**Total tests:** ~20 lines (8 test cases)

---

## Dependencies (Components)

- `token.rs` — TokenKind enum (existing)
- `lexer/mod.rs` — keyword dispatch map (existing)

---

## Implementation Notes

**Key patterns to analyze:**
- Examine how `own`, `borrow`, and `shared` were added in Block 2 — follow that exact pattern for both token.rs and lexer/mod.rs
- Verify `is_keyword()` and `as_str()` both have exhaustive match arms — the compiler will warn if missed

**Critical requirements:**
- Both `async` and `await` must return `true` from `is_keyword()`
- Both must produce their respective `TokenKind` variant, not `TokenKind::Identifier`
- `as_str()` must round-trip correctly for both

**Error handling:**
- No new AT codes needed here — keyword rejection is handled at the parser level (Phase 04)

**Integration points:**
- Uses: `token.rs` (existing), `lexer/mod.rs` (existing)
- Creates: two new `TokenKind` variants

---

## Tests (TDD Approach)

**Tokenization:** (4 tests)
1. `async` source text produces `TokenKind::Async`
2. `await` source text produces `TokenKind::Await`
3. `async_fn` (underscore-joined) is NOT a keyword — tokenizes as identifier
4. `awaiting` is NOT a keyword — tokenizes as identifier

**Keyword identity:** (4 tests)
1. `is_keyword` returns true for `async`
2. `is_keyword` returns true for `await`
3. `as_str(TokenKind::Async)` returns `"async"`
4. `as_str(TokenKind::Await)` returns `"await"`

**Minimum test count:** 8 tests

**Parity requirement:** N/A — lexer is shared infrastructure, no parity split.

---

## Acceptance Criteria

- ✅ `TokenKind::Async` and `TokenKind::Await` added to enum
- ✅ Both appear in `is_keyword()` and `as_str()`
- ✅ Keyword map entries in `lexer/mod.rs`
- ✅ 8+ lexer tests pass
- ✅ `cargo check -p atlas-runtime` clean — no new warnings

---

## References

**Decision Logs:** D-030
**Specifications:** docs/language/async.md (syntax section)
**Related phases:** Phase 01 (spec), Phase 03 (AST consumes these tokens via parser), Phase 04 (parser uses TokenKind::Async/Await)
