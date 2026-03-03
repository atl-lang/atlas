# Atlas Language Design Principles

**Purpose:** Canonical design authority for Atlas grammar and syntax decisions.
**Status:** Authoritative. All grammar changes must align with these principles.
**Last Updated:** 2026-03-03

---

## Source of Truth

These principles derive from Atlas's founding documents:
- `docs/internal/PRD.md` — Non-negotiable requirements
- `README.md` — Vision and "best of all worlds" philosophy
- `docs/internal/V02_LESSONS_LEARNED.md` — Hard-won experience

This document CLARIFIES and ENFORCES those requirements. It does not invent new direction.

---

## Core Principles

### 1. No Ambiguous Syntax

> From PRD: "No ambiguous syntax"

**Meaning:**
- Every token sequence has exactly ONE valid parse
- Parser never backtracks or speculates
- Context should not change meaning of identical syntax

**Violation Example:**
```atlas
// BAD: `{}` means different things based on context
{ }              // Empty block? Empty object? Structural type?
{ x: 5 }         // Object literal (in expression position)
{ let y = 1; }   // Block (statement sequence)
```

**Correct Approach:**
- Distinct syntax for distinct constructs
- Type-prefixed struct literals (like Rust/Go): `Person { name: "Alice" }`
- Bare `{}` is always a block

---

### 2. Cohesion Over Sprawl

> From PRD: "Cohesion over sprawl: Only add features when truly needed and well-designed"

**Meaning:**
- ONE way to express each concept
- No "deprecated but supported" syntax
- Remove redundant paths, don't accumulate them

**Violation Example:**
```atlas
// BAD: Three ways to declare mutable variables
var x = 5;           // Legacy (deprecated but works)
let mut y = 5;       // Recommended
let z = 5; z = 6;    // Wait, is let mutable or not?
```

**Correct Approach:**
- `let` = immutable, `let mut` = mutable
- `var` removed entirely (not deprecated — GONE)

---

### 3. Small Surface Area

> From PRD: "Small surface area: Keep syntax and stdlib focused"

**Meaning:**
- Fewer keywords, fewer operators, fewer special cases
- If Go or Rust removed it, consider if Atlas needs it
- Legacy C syntax is not "proven" — it's baggage

**Violation Example:**
```atlas
// BAD: C-style cruft that adds surface area
for (var i = 0; i < 10; i++) { }    // C-style for
x++;                                  // Increment operator
```

**Correct Approach:**
- `for item in collection { }` + `while condition { }`
- `x += 1` instead of `x++`

---

### 4. Explicit Over Implicit

> From PRD: "Strict typing: No implicit any, no implicit nullable"
> From README: "No implicit behaviors to hallucinate"

**Meaning:**
- Types flow through the system, never become "Unknown" silently
- Conversions require explicit calls
- No truthy/falsy — booleans only

**Violation Example:**
```atlas
// BAD: Type::Unknown satisfies any type
let obj = { x: 1 };     // Typechecks to Unknown
let n: number = obj;    // Silently allowed!
```

**Correct Approach:**
- Object literals have concrete structural types
- `Unknown` is an ERROR state, not a wildcard

---

### 5. Predictable for AI Generation

> From README: "Syntax designed for reliable code generation"
> From V02 Lessons: "Predictable grammar that AI can learn to generate correctly"

**Meaning:**
- Consistent patterns across the language
- No special cases or context-dependent parsing
- What works in one place works everywhere

**Test:** Can an AI generate valid Atlas code by pattern-matching on examples?
If the answer requires "well, except when..." — the grammar is wrong.

---

## Inheritance from Other Languages

> From README: "Cherry-picks proven ideas from the best languages"

| Source | What Atlas Takes | What Atlas Rejects |
|--------|------------------|-------------------|
| **Rust** | `let`/`let mut`, Result/Option, `for-in`, struct literals | Lifetime syntax, implicit derefs |
| **Go** | Clear errors, simple concurrency model | `var` keyword, `:=` short declaration |
| **TypeScript** | Strict typing, type inference | `any`, implicit coercion, arrow functions |
| **Python** | Readable syntax, low ceremony | Significant whitespace, dynamic typing |

**Key insight:** "Proven" means proven to be CLEAR, not proven to be POPULAR.
C-style `for(;;)` is popular but unclear. Rust's `for-in` is clearer.

---

## Decision Authority

When evaluating grammar changes:

1. **Does it violate any principle above?** → Reject
2. **Does it add surface area without clear benefit?** → Reject
3. **Is there already a way to do this?** → Use existing way
4. **How do Rust/Go handle this?** → Default to their approach unless Atlas has specific reason to differ

---

## Enforcement

- All grammar changes require a decision record in `decisions/`
- Decision records must reference which principle(s) apply
- "AI-first" is not a justification — cite specific principles
- Spec documents track implementation, design docs guide it

---

## References

- PRD: `/docs/internal/PRD.md`
- README: `/README.md`
- Lessons: `/docs/internal/V02_LESSONS_LEARNED.md`
- Decisions: `/docs/language-design/rationale/`
