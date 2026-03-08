# B12 Error Audit — Quality Scoring

**Generated:** 2026-03-08 | **Block:** B12-P01 | **Purpose:** Drive P02–P08 implementation priority

## Quality Scale

| Score | Meaning | Example |
|-------|---------|---------|
| 1 | Pure generic fallback — no context, no fix | "check your syntax for typos" |
| 2 | Context present but no fix shown | "Ensure types match. Use explicit conversions." |
| 3 | Specific problem stated, no fix shown | `Unknown extern type 'X'. Valid types: CInt, CLong...` |
| 4 | Specific problem + partial fix shown | "Use `-> void` for functions that return nothing" |
| 5 | AT1007-level: named code + specific problem + Atlas fix with inline example | AT1007 + multi-line ownership example |

---

## Group A — Parser (`parser/mod.rs`, `parser/stmt.rs`, `parser/expr.rs`)

| File | Line | Message | Help | Score |
|------|------|---------|------|-------|
| parser/stmt.rs | 30 | Import statements are not supported in Atlas v0.1 | none | 2 |
| parser/stmt.rs | 159 | Invalid assignment target | none | 1 |
| parser/stmt.rs | 171 | Invalid assignment target | none | 1 |
| parser/stmt.rs | 178 | Invalid assignment target | none | 1 |
| parser/stmt.rs | 188 | Invalid assignment target | none | 1 |
| parser/mod.rs | 145 | expected `fn` after `async` | none | 2 |
| parser/mod.rs | 364 | Return type annotation required on named functions. Use `-> void`... | none | 3 |
| parser/mod.rs | 500 | Expected variable declaration after 'export' | none | 2 |
| parser/mod.rs | 507 | Expected 'fn', 'let', or 'type' after 'export' | none | 2 |
| parser/mod.rs | 637 | Expected parameter name after ownership annotation '{kw}' | none | 2 |
| parser/mod.rs | 660 | Parameter '{param_name}' is missing ownership annotation | multi-line example | **5** |
| parser/mod.rs | 681 | Expected ':' after parameter name | none | 2 |
| parser/mod.rs | 1017 | Unknown extern type '{other}'. Valid types: CInt, CLong... | none | 3 |
| parser/expr.rs | 48 | Expected expression | none | 1 |
| parser/expr.rs | 564 | Expected expression inside '[]' | none | 2 |
| parser/expr.rs | 591 | Inclusive range requires an end expression | none | 2 |
| parser/expr.rs | 623 | Inclusive range requires an end expression | none | 2 |
| parser/expr.rs | 844 | Structural type must include at least one member | none | 2 |
| parser/expr.rs | 870 | Unexpected empty type group | none | 1 |
| parser/expr.rs | 897 | Expected '->' after function type parameters | none | 2 |
| parser/expr.rs | 918 | Generic type requires at least one type argument | none | 2 |
| parser/expr.rs | 1088 | Anonymous struct literal requires at least one field | none | 2 |
| parser/expr.rs | 1184 | Expected ',' or ';' after match arm | none | 2 |
| parser/expr.rs | 1194 | Match expression must have at least one arm | none | 2 |
| parser/expr.rs | 1400 | Expected pattern | none | 1 |
| parser/mod.rs | 1093 | check your syntax for typos or missing tokens | none | 1 |

**Parser total:** 26 sites | Score-1: 6 | Score-2: 16 | Score-3: 2 | Score-4: 0 | Score-5: 1 | **Avg: 1.8**

### Critical Parser Pattern Findings

1. **4 duplicate "Invalid assignment target" errors** (stmt.rs:159, 171, 178, 188) — same generic message for 4 distinct cases (index-of-range, member-of-call, member-bad-base, other invalid lvalue). Must be split into distinct codes.

2. **Zero `.with_help()` calls in parser** — error_codes.rs defines help for AT1000–AT1006 but the parser never attaches it. Help text is defined but unreachable.

3. **All parser errors use AT9999 (generic)** — except AT1007. `E_GENERIC / E_BAD_NUMBER / E_MISSING_SEMI / E_MISSING_BRACE / E_UNEXPECTED / E_RESERVED` constants are defined but unused.

---

## Group B — Typechecker (`typechecker/mod.rs`, `typechecker/expr.rs`, `binder.rs`)

| File | Line | Message | Help | Score |
|------|------|---------|------|-------|
| binder.rs | 933 | declare '{}' with 'let' or 'const' before assigning to it | none | 2 |
| binder.rs | 977 | declare '{}' before using it, or check for typos | none | 2 |

**Typechecker direct emission total:** 2 sites | All score-2

> **Major gap:** Typechecker type mismatches, arity errors, and struct field errors do NOT emit diagnostics directly. They surface through RuntimeError enum only — meaning no help text, no error codes, no fix suggestions reach the user at compile time. AT3001–AT3005 codes are defined in error_codes.rs but never emitted.

---

## Group C — Runtime (`interpreter/mod.rs`, `vm/mod.rs`, `vm/dispatch.rs`)

RuntimeError variants are emitted with context fields (file, variable name, type) but no help text:

| Variant | Code | Guidance | Score |
|---------|------|----------|-------|
| TypeError | AT0001 | message only | 2 |
| UndefinedVariable | AT0002 | message only | 2 |
| DivideByZero | AT0005 | no message | 1 |
| OutOfBounds | AT0006 | no message | 1 |
| InvalidNumericResult | AT0007 | no message | 1 |
| InvalidStdlibArgument | AT0102 | message only | 2 |
| UnhashableType | AT0140 | includes type name | 3 |
| Permission (AT0300–AT0303) | AT0300–AT0303 | includes path/host | 3 |
| IoError | AT0400 | generic message | 1 |
| StackUnderflow | AT9997 | no message | 1 |
| UnknownOpcode | AT9998 | no message | 1 |
| InternalError | AT9995 | message only | 2 |

**Runtime total:** 12 variants | Score-1: 6 | Score-2: 4 | Score-3: 2 | **Avg: 1.8**

---

## Group D — Stdlib (`stdlib/*.rs`)

**Zero direct Diagnostic or `.with_help()` calls** across all 20+ stdlib modules. All stdlib errors surface through `RuntimeError::InvalidStdlibArgument` with a generic message string — no help text, no error codes, no function signature shown.

**Stdlib total:** 0 diagnostic sites | **All errors are score-1 at point of emission**

---

## Summary Counts

| Group | Sites | Score-1 | Score-2 | Score-3 | Score-4 | Score-5 | Avg |
|-------|-------|---------|---------|---------|---------|---------|-----|
| Parser | 26 | 6 | 16 | 2 | 0 | 1 | 1.8 |
| Typechecker | 2 | 0 | 2 | 0 | 0 | 0 | 2.0 |
| Runtime | 12 | 6 | 4 | 2 | 0 | 0 | 1.8 |
| Stdlib | 0 | — | — | — | — | — | 1.0* |
| **Total** | **40** | **12** | **22** | **4** | **0** | **1** | **1.8** |

*Stdlib errors are all score-1 at the RuntimeError emission layer.

---

## Top 15 Priority Fix Sites (Score 1–2, Ordered by Impact)

| Rank | File | Issue | Target Phase |
|------|------|-------|-------------|
| 1 | parser/stmt.rs:159,171,178,188 | 4 duplicate "Invalid assignment target" — must split into distinct codes | P02 |
| 2 | parser/mod.rs:1093 | "check your syntax for typos" — canonical score-1 generic | P02 |
| 3 | parser/expr.rs:48 | "Expected expression" — zero context | P02 |
| 4 | parser/expr.rs:1400 | "Expected pattern" — zero context | P02 |
| 5 | All parser sites | Zero `.with_help()` calls despite registry defining AT1000–AT1006 | P02 |
| 6 | RuntimeError::DivideByZero | No message at all — AT0005 emits nothing | P07 |
| 7 | RuntimeError::OutOfBounds | No message — AT0006 emits nothing | P07 |
| 8 | RuntimeError::IoError | "I/O error: {msg}" — too generic, needs split | P07 |
| 9 | Typechecker: no arity errors | AT3005 defined but never emitted with help | P04 |
| 10 | Typechecker: no type mismatch | AT3001 defined but never emitted with help | P04 |
| 11 | binder.rs:977 | "declare '{}' before using it, or check for typos" — no Atlas fix | P03 |
| 12 | Stdlib: all modules | No Diagnostic calls — help never reaches user | P05 |
| 13 | parser/expr.rs:591,623 | Duplicate "Inclusive range requires end" — no fix shown | P02 |
| 14 | RuntimeError::StackUnderflow | AT9997 — no message at all | P07 |
| 15 | parser/mod.rs:364 | Return type annotation error — score-3, needs `.with_help()` wiring | P02 |

---

## Architectural Findings (Drive P02–P08 Design)

### Finding 1: Help Registry Is Wired But Never Called
`error_codes.rs` defines help text for AT1000–AT1006 (parser errors) and AT3001–AT3005 (typechecker errors). The registry works. But **no call site in parser or typechecker calls `.with_help()`**. Fix: create `emit_parser_error(code, msg, span)` helper that auto-attaches help from the registry.

### Finding 2: Typechecker Emits Nothing — Errors Are All Runtime
Type mismatches, arity errors, struct field errors — none emit a `Diagnostic` at check time. They either panic or surface as RuntimeError. This means users see runtime errors for what should be compile-time type errors. P04 must introduce typechecker-time Diagnostic emission.

### Finding 3: Stdlib Is a Black Box for Errors
20+ stdlib modules, zero help text. Every stdlib error is `RuntimeError::InvalidStdlibArgument("some string")`. P05 must create a pattern (helper macro or function) that wraps stdlib errors with function signature context.

### Finding 4: AT1007 Is the Only Gold Standard
The entire codebase has exactly **one score-5 error**: AT1007 (missing ownership annotation). It is the template. Every other error in B12 should be upgraded to match its structure: named code → specific problem → inline Atlas fix with example.

### Finding 5: Cross-Language Patterns Not Yet Detected
Parser has no detection for: `echo`, `console.log`, `var`, `function`, `class`, `x++`, `${x}`. These are the highest-frequency AI mistakes. P02 adds them as token-level detection in the parser.
