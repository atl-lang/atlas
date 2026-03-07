# Error Message Quality — Atlas Full Audit

**Assessment criteria:**
1. **Localization** — does the error point to the right line and column?
2. **Clarity** — is the message understandable without reading source code?
3. **Actionability** — does it tell you what to do?
4. **AI efficiency** — how many tokens does an AI need to diagnose the error?
5. **Passes to fix** — after seeing the error, how many attempts to fix it?

**Scale:** 1 (useless) → 5 (perfect, single-pass fix)

---

## Error Catalog (from generation session)

| Code | Message | Loc | Clarity | Action | AI passes |
|------|---------|-----|---------|--------|-----------|
| AT1012 | Cannot shadow prelude builtin 'X' | 5 | 5 | 5 | 1 |
| AT2002 | Unknown symbol 'contains' | 5 | 3 | 2 | 2 |
| AT3001 | Condition must be bool, found ? | 5 | 3 | 1 | 3+ |
| AT3001 | Return type mismatch: expected T, found ? | 5 | 3 | 1 | 3+ |
| AT3001 | Argument N type mismatch: expected T, found ? | 5 | 3 | 1 | 3+ |
| AT3003 | Cannot assign to immutable binding | 5 | 4 | 4 | 1 |
| AT3035 | Method not found on Self in default impl | 5 | 3 | 2 | 2 |
| AT0102 | Invalid argument to stdlib function | 5 | 2 | 1 | 3+ |
| AT1000 | Parse error (multi-line struct in array) | 1 | 1 | 1 | 5+ |
| AT4002 | non-Future value used in await | 5 | 4 | 3 | 2 |

---

## Systemic Error Issues

### E-001: Multi-line files report line 1 (pre-existing)
- hydra-v3 found: errors like "Expected ';'" pointing to line 1 col 1 in 200-line files.
- **Status:** Observed once (multi-line struct array). Mostly localization is correct.

### E-002: `?` type in error messages is opaque
- **Most common error encountered:** `Condition must be bool, found ?`
- The `?` gives no information about WHY the type is unknown.
- **Real cause:** match on user-defined enum in function body returns `?`.
- **What AI sees:** "found ?" — no indication of root cause.
- **Passes to diagnose:** 3-5. AI tries type annotations, workarounds, before discovering the typechecker bug.
- **Fix needed:** Error message should say "match on user-defined enum returns unknown type — known limitation" or similar.

### E-003: AT0102 "Invalid argument" has no type info
- Message: `Invalid argument to standard library function in function <main>`
- **Missing:** what type was expected, what type was found.
- **Compared to AT3001:** AT3001 says "expected T, found U" — much more useful.
- **AI passes:** 3+ (must guess which argument and what type mismatch).

### E-004: Error codes not surfaced in `?` cascade
- When one `?` type propagates, all downstream uses produce separate errors.
- A 5-line function body can produce 8+ errors from a single root cause.
- **AI cost:** Must identify the root `?` source among many error lines.
- **Improvement:** Error suppression for cascaded `?` with a single root cause note.

---

## Error Efficiency Findings

| Error encountered | Passes to fix | Root cause of extra passes |
|-------------------|---------------|---------------------------|
| AT1012 prelude shadow | 1 | Perfect message — rename variable |
| AT3003 immutable binding | 1 | Good message — add `mut` |
| AT2002 unknown symbol 'contains' | 2 | No suggestion of correct name |
| AT3001 Condition must be bool, found ? | 3-5 | Root cause is typechecker bug, not code |
| AT3001 Return type mismatch, found ? | 3-5 | Same — `?` cascade from enum match |
| AT0102 Invalid argument | 3+ | No type information in message |
| AT1000 parse error (multi-line struct) | 5+ | Points to wrong location, no hint |

**Worst error class:** `AT3001 found ?` — responsible for ~60% of extra passes.
Root cause is **always** a typechecker limitation (enum match, if/else expr).
Message gives no signal — AI tries type annotations and workarounds before finding the bug.
