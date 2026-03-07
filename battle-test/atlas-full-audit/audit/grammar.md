# Grammar & Syntax Issues — Atlas Full Audit

Issues where Atlas syntax diverges from standard language conventions in ways
that create AI generation friction or violate the AI-first principle.

**AI-first principle:** If a token sequence that would be natural in any of
Go, Rust, TypeScript, or Python causes a parse error in Atlas, it's a bug
(or at minimum, a language design decision to explicitly document).

---

## Known Pre-existing Issues

### G-001: No template strings
- **Natural syntax:** `` `Hello, ${name}!` ``
- **Atlas:** `"Hello, " + name + "!"`
- **Impact:** High friction for AI generation. Every LLM defaults to backtick strings.
- **Token cost:** ~2x token overhead for string construction.
- **Decision needed:** Implement or formally document the exclusion with rationale.

### G-002: Stdlib uses global function syntax, not method syntax
- **Natural syntax:** `arr.push(x)`, `map.get(key)`
- **Atlas:** `arrayPush(arr, x)`, `hashMapGet(map, key)`
- **Impact:** MAJOR. Every AI defaults to method syntax. Requires explicit re-learning per session.
- **D-021 filed.** Not yet implemented.

### G-003: CoW collection rebinding is invisible
- **Problem:** `hashMapPut(m, k, v)` silently drops the result if not rebound.
  AI will frequently write `hashMapPut(scores, "alice", 95);` (no rebind) and get
  silent data loss — no warning, no error.
- **Expected behavior:** Compiler should warn when a CoW-return is discarded.
- **Token cost:** Zero extra tokens (just a rebind), but the *discovery* costs many tokens.

### G-004: for-in doesn't accept ranges
- **Natural syntax:** `for i in 0..10 { }` (Go/Rust)
- **Atlas:** must build array first or use while loop
- **Impact:** MAJOR. Numeric iteration is extremely common.
- **H-116 filed.**

### G-005: Struct variant enums not constructible
- **Natural syntax:** `Shape::Circle { radius: 5.0 }`
- **Atlas:** Only unit and tuple variants work in expressions.
- **Impact:** Forces artificial tuple encoding: `Shape::Circle(5.0)`.

---

## Issues Found During Generation

### G-006: return inside match arm is a syntax error (H-114)
- **Natural syntax (Rust/Go):** `match x { Variant(v) => return v, _ => return default }`
- **Atlas:** parse error. Must use: `let result = match ...; return result;`
- **Impact:** MAJOR. Forced verbosity in every enum function.
- **Filed as H-114 (P1).**

### G-007: if/else as expression returns `?` type (H-115)
- **Natural syntax (all languages):** `let label = if active { "on" } else { "off" };`
- **Atlas:** typechecker returns `?`, then `print(label)` fails.
- **Workaround:** `let mut label = "off"; if active { label = "on"; }`
- **Impact:** MAJOR. Every conditional assignment requires imperative rebind.
- **Filed as H-115 (P1).**

### G-008: match on user-defined enum in function body → `?` type (H-110)
- **Expected:** `fn label(s: Status) -> string { match s { Status::Active => "on", ... } }`
- **Atlas:** typechecker returns `?` for ALL match arms on user enums in function bodies.
- **Workaround:** Do ALL enum dispatch at top level — cannot dispatch inside functions.
- **Impact:** BLOCKER for any idiomatic enum-driven design.
- **Filed as H-110 (P0).**

### G-009: Matching same user enum variable twice → second result `?` (H-111)
- **Expected:** `let kind = match e { ... }; let msg = match e { ... };` both work.
- **Atlas:** After the first match, all subsequent matches on the same variable produce `?`.
- **Workaround:** Extract all needed data in ONE match: `let full = match e { A(m) => "A: " + m, ... }`.
- **Impact:** BLOCKER. Forces severely non-idiomatic single-match patterns.
- **Filed as H-111 (P0).**

### G-010: Prelude name collision is silent until AT1012
- **Problem:** Variables named `pop`, `concat`, `clamp`, `push`, `record`, `is_ok` shadow
  prelude builtins — AT1012 error, no IDE hint.
- **Impact:** MINOR. Caught quickly by error code, but unintuitive.
- **Common collision names:** `pop`, `push`, `concat`, `clamp`, `record`, `is_ok`, `map`, `filter`

### G-011: Default trait methods cannot call other trait methods (AT3035)
- **Natural pattern (Rust):** Default method calls `self.required_method()`.
- **Atlas:** AT3035 — "method not found on Self in default impl".
- **Impact:** MAJOR. Traits can't compose — each impl must re-implement everything.

---

## Issues Found That Go/Rust/TypeScript Have in v1.0

Per CLAUDE.md "See Something, File Something" rule:

| Feature | Status | Filed |
|---------|--------|-------|
| Range syntax in for-in (`0..n`) | Missing | H-116 |
| `return` in match arm | Syntax error | H-114 |
| if/else as expression | Broken | H-115 |
| Enum match in fn body | Broken typechecker | H-110 |
| Method syntax for stdlib | Missing | D-021 |
| Template strings | Missing | G-001 |
