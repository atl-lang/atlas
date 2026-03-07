# Friction Log — Atlas Full Audit

**Methodology:** Sonnet 4.6 generated all Atlas programs (acting as Haiku proxy for this run).
Every friction point, workaround, and failure is recorded. This is the raw signal for language quality.

**Scale:**
- `BLOCKER` — couldn't express the concept at all in Atlas
- `MAJOR` — required significant workaround that degrades readability
- `MINOR` — small adaptation needed; still readable
- `NONE` — expressed naturally, first pass

---

## Domain 01: Primitives

| Issue | Severity | Passes |
|-------|----------|--------|
| No template strings — must use `+` concatenation | MINOR | 1 |
| All 5 programs passed first pass | NONE | 1 |

**Friction count:** 1 minor. **Blocker count:** 0.

---

## Domain 02: Control Flow

| Issue | Severity | Passes |
|-------|----------|--------|
| No range syntax in `for-in` — must build array or use while | MAJOR | 2 |
| `return` inside match arm is syntax error (H-114) | MAJOR | 2 |
| if/else as expression returns `?` — must use imperative rebind (H-115) | MAJOR | 3 |
| `contains` → `includes` (AT2002 — name unknown) | MINOR | 2 |

**Friction count:** 4 (3 major, 1 minor). **Blocker count:** 0.

---

## Domain 03: Functions

| Issue | Severity | Passes |
|-------|----------|--------|
| No `|x|` closure syntax — must use `fn(x: T) -> R { }` | MINOR | 2 |
| All 5 programs passed after syntax correction | NONE | 1-2 |

**Friction count:** 1 minor. **Blocker count:** 0.

---

## Domain 04: Types (Structs/Enums/Option/Result)

| Issue | Severity | Passes |
|-------|----------|--------|
| match on user enum in function body → `?` type (H-110) | BLOCKER | 5+ |
| Workaround: all dispatch at top level only | BLOCKER | — |
| Matching same user enum variable twice → second result `?` (H-111) | BLOCKER | 3+ |
| Multi-line struct array literals fail — must put struct on one line | MAJOR | 2 |
| Struct `T[]` as function parameter types as `?[]` (H-117) | MAJOR | 3 |
| `let x: T = match ...` with user enum fails — must omit type annotation | MINOR | 2 |

**Friction count:** 6 (3 blocker, 2 major, 1 minor). **Blocker count:** 3.

---

## Domain 05: Traits

| Issue | Severity | Passes |
|-------|----------|--------|
| Default methods calling `self.other_method()` → AT3035 | MAJOR | 3 |
| Workaround: override default methods in each impl, no cross-method calls | MAJOR | — |
| Prelude name shadowing (e.g. `push`, `concat`, `pop`) → AT1012 | MINOR | 2 |

**Friction count:** 3 (2 major, 1 minor). **Blocker count:** 0.

---

## Domain 06: Collections

| Issue | Severity | Passes |
|-------|----------|--------|
| `hashMapHas` returns `any` not `bool` (H-112) — if condition fails | MAJOR | 3 |
| Workaround: `str(hashMapHas(m, k)) == "true"` | MAJOR | — |
| `hashSetHas` returns `any` — same issue | MAJOR | 3 |
| `hashSetRemove` CoW semantics differ from `hashSetAdd` (H-113) | MAJOR | 2 |
| `hashMapContains`, `hashSetContains` don't exist — use `hashMapHas`, `hashSetHas` | MINOR | 2 |
| `let mut` required for CoW rebind — missing mut causes AT3003 | MINOR | 1 |
| CoW rebind pattern: result must be rebound (`arr = arrayPush(arr, x)`) | MINOR | 2 |

**Friction count:** 7 (4 major, 3 minor). **Blocker count:** 0.

---

## Domain 07: Error Handling

| Issue | Severity | Passes |
|-------|----------|--------|
| match on user enum in fn body → `?` (H-110) — cannot write fmt_error() | BLOCKER | 5+ |
| Double-match on same user enum → second result `?` (H-111) | BLOCKER | 3+ |
| Workaround: one match per enum var that produces final output string | BLOCKER | — |
| `?` operator — not standard; Result chaining verbose | MINOR | 1 |

**Friction count:** 4 (3 blocker, 1 minor). **Blocker count:** 3.

---

## Domain 08: Async/Await

| Issue | Severity | Passes |
|-------|----------|--------|
| `spawn(fn)` takes 2 args: `spawn(future, null)` — undocumented | MINOR | 2 |
| stdlib names: `futureAll`, `futureRace`, `futureResolve` not `all`, `race` | MINOR | 2 |
| No `|x|` closure syntax — async closures need full fn syntax | MINOR | 1 |
| `readFileAsync`, `writeFileAsync` — names not obvious from stdlib docs | MINOR | 1 |

**Friction count:** 4 (4 minor). **Blocker count:** 0.

---

## Domain 09: Stdlib

| Issue | Severity | Passes |
|-------|----------|--------|
| `sqrt` returns `Result<number,string>` not `number` (H-118) | MAJOR | 2 |
| `indexOf` returns `Option<number>` not `number` (H-118) | MAJOR | 2 |
| `parseJSON` returns `Result<json,string>` not `json` (H-118) | MAJOR | 2 |
| Stdlib uses global fn syntax, not method syntax (G-002) — AI defaults to `.method()` | MAJOR | 3 |
| `contains` → `includes` for strings (AT2002) | MINOR | 1 |

**Friction count:** 5 (4 major, 1 minor). **Blocker count:** 0.

---

## Domain 10: Integration

| Issue | Severity | Passes |
|-------|----------|--------|
| Compound of all above issues — user enum dispatch, CoW, stdlib names | MAJOR | 4+ |
| Struct `T[]` as fn parameter types as `?[]` — forced inlining | MAJOR | 3 |
| All 3 programs eventually passed with workarounds | — | — |

**Friction count:** 2 major per program. **Blocker count:** 0.

---

## Summary Table

| Domain | Blockers | Major | Minor | Passes to compile |
|--------|----------|-------|-------|-------------------|
| 01 Primitives | 0 | 0 | 1 | 1 |
| 02 Control Flow | 0 | 3 | 1 | 2-3 |
| 03 Functions | 0 | 0 | 1 | 1-2 |
| 04 Types | 3 | 2 | 1 | 3-5+ |
| 05 Traits | 0 | 2 | 1 | 2-3 |
| 06 Collections | 0 | 4 | 3 | 2-3 |
| 07 Error Handling | 3 | 0 | 1 | 4-5+ |
| 08 Async | 0 | 0 | 4 | 1-2 |
| 09 Stdlib | 0 | 4 | 1 | 2-3 |
| 10 Integration | 0 | 2 | 0 | 3-4 |
| **TOTAL** | **6** | **17** | **14** | **avg 2.5** |

**Final: 47/47 programs pass. 6 blockers, 17 major frictions.**
