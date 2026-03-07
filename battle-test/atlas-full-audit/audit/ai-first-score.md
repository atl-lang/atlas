# AI-First Compliance Score — Atlas Full Audit

**Model used for generation:** claude-sonnet-4-6 (Haiku proxy — programs generated via AI)
**Scoring:** 0–10 per dimension. 10 = AI generates correct code on first pass, zero friction.

---

## Dimension Scores

| Dimension | Score | Notes |
|-----------|-------|-------|
| Syntax intuitiveness | 6/10 | `let`, `fn`, `match` all natural; no templates, no ranges |
| Token efficiency | 7/10 | Concise overall; CoW rebind adds ~10% overhead |
| Error diagnostic quality | 4/10 | `found ?` cascade is the #1 cost driver — opaque root cause |
| Stdlib discoverability | 4/10 | Global fn names not guessable; `arrayPush` vs `.push()` |
| Type system clarity | 5/10 | Primitives/Option/Result good; user enum match broken |
| Async model | 8/10 | Clean syntax; `futureAll`/`futureRace` names are friction |
| Collection patterns | 6/10 | CoW rebind logical once known; `hashMapHas` returning `any` is bad |
| Error handling | 6/10 | Result/Option match works; user enum dispatch broken |
| Struct/enum usage | 5/10 | Construction natural; match dispatch blocked in functions |
| Integration | 6/10 | Multi-feature programs work; compound frictions add up |

**Overall AI-First Score: 57/100**

---

## Pass Analysis

| Domain | First-pass success | Corrections needed | Blocker hit |
|--------|--------------------|--------------------|-------------|
| 01 Primitives | 5/5 | template strings | No |
| 02 Control Flow | 2/5 | no ranges, return-in-match, if/else expr | No |
| 03 Functions | 4/5 | closure syntax | No |
| 04 Types | 1/6 | enum match in fn body, struct arrays | Yes (H-110) |
| 05 Traits | 2/4 | default method cross-call, prelude names | No |
| 06 Collections | 2/5 | hashMapHas any, hashSetRemove semantics | No |
| 07 Error Handling | 1/4 | enum dispatch broken | Yes (H-110) |
| 08 Async | 4/5 | stdlib names | No |
| 09 Stdlib | 2/5 | undocumented return types, method names | No |
| 10 Integration | 1/3 | compound of all above | Yes (H-110) |

**First-pass success rate: 24/47 (51%)**

---

## Final Verdict

### Top 3 Strengths for AI Generation
1. **Async/await model** — clean `async fn` + `await` syntax; parity between engines; concurrent builtins exist
2. **Result/Option handling** — `match r { Ok(v) => ..., Err(e) => ... }` is natural and works correctly
3. **Basic syntax** — `let`, `fn`, `struct`, `for-in`, `while` are all conventional and readable

### Top 3 Blockers for AI Generation
1. **User enum match in function body returns `?` (H-110)** — fundamental typechecker bug that blocks ALL idiomatic enum dispatch. Every dispatch function requires top-level inlining.
2. **Double-match on same user enum → `?` (H-111)** — even at top level, can only match a user enum once. Forces non-idiomatic single-match encoding.
3. **Stdlib global function names (G-002)** — AI defaults to method syntax every session. `arrayPush(arr, x)` vs `arr.push(x)` is re-learned per conversation.

### Compared to Go/Rust/TypeScript

**Where Atlas wins:**
- Cleaner async syntax than Go (no goroutines/channels to explain)
- More concise than Rust for basic programs (no lifetimes, no borrowing ceremony)
- Strongly typed vs TypeScript's optional typing — fewer runtime surprises for AI

**Where Atlas loses:**
- Enum match is broken in functions — Rust gets this right
- No range syntax — Go/Rust both have `0..n`
- if/else as expression broken — Rust, Kotlin, Scala all get this right
- Stdlib method syntax missing — TypeScript/Go/Python all have this
- Error messages for `?` type far less informative than rustc

**Overall:** With H-110 + H-111 fixed, Atlas would likely score 72+/100.
Those two bugs alone account for ~80% of the friction in domains 04, 07, 10.

---

## Recommendations for Atlas v0.4 (Priority Ordered)

1. **Fix typechecker: user enum match in function body (H-110)** — single highest-impact fix.
   Unblocks idiomatic dispatch patterns in all domains.

2. **Fix typechecker: double-match on user enum at top level (H-111)** — second highest.
   Forces severely non-idiomatic code even when H-110 workaround is applied.

3. **Implement stdlib method syntax (D-021)** — `arr.push(x)`, `map.get(k)`, etc.
   Every AI generation session re-learns this. Highest token-cost friction overall.

4. **Fix: hashMapHas/hashSetHas return bool (H-112)** — trivial fix, high friction.

5. **Add range syntax in for-in (H-116)** — `for i in 0..10` is expected by every AI.

6. **Fix: if/else as expression (H-115)** — standard in all modern languages.

7. **Fix: return inside match arm (H-114)** — standard in Rust, Go, Kotlin.

8. **Document: sqrt/indexOf/parseJSON return types (H-118)** — low-effort, high value.
