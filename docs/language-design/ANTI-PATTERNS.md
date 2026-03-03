# Atlas Anti-Patterns

**Purpose:** Document grammar mistakes made in v0.1-v0.2 so they are never repeated.
**Status:** Authoritative warning list for all AI agents.
**Last Updated:** 2026-03-03

---

## Why This Document Exists

Atlas v0.1-v0.2 accumulated grammar debt despite having clear principles in the PRD.
AI agents added features without checking if they violated stated goals.
This document names specific mistakes so they cannot be repeated.

---

## Anti-Pattern 1: "Deprecated But Supported"

**What happened:**
```atlas
var x = 5;  // Marked deprecated, still fully functional
```

**Why it's wrong:**
- Doubles the syntax surface area
- AI agents see both patterns in code, generate both randomly
- "Deprecated" becomes permanent because nothing enforces removal

**The fix:** Remove, don't deprecate. If syntax is wrong, delete it.

**Principle violated:** Cohesion Over Sprawl

---

## Anti-Pattern 2: Multiple Paths to Same Outcome

**What happened:**
```atlas
// Three ways to make a mutable variable
var x = 5;
let mut y = 5;

// Two ways to write anonymous functions
fn(x: number) -> number { return x * 2; }
(x) => x * 2

// Two loop syntaxes
for (var i = 0; i < 10; i++) { }
for item in items { }
```

**Why it's wrong:**
- AI must learn multiple patterns for identical semantics
- Code review becomes "which style did they use?" not "is it correct?"
- Generation variance increases error rate

**The fix:** Pick ONE syntax per concept. Remove alternatives.

**Principle violated:** Cohesion Over Sprawl, Small Surface Area

---

## Anti-Pattern 3: Context-Dependent Syntax

**What happened:**
```atlas
// Same tokens, different meanings
{ key: value }     // Object literal (expression context)
{ let x = 1; x }   // Block expression (statement context)
{ field: type }    // Structural type (type context)
```

**Why it's wrong:**
- Parser must backtrack or speculate
- AI cannot predict parse result from tokens alone
- Same sequence generates different AST based on surroundings

**The fix:** Distinct syntax for distinct constructs.
- Struct/object literals: `TypeName { field: value }`
- Blocks: `{ statements }`
- Structural types: Define via `type` declaration

**Principle violated:** No Ambiguous Syntax

---

## Anti-Pattern 4: Legacy C Syntax

**What happened:**
```atlas
for (var i = 0; i < 10; i++) { }  // C-style for
x++;                               // Increment
--y;                               // Decrement
```

**Why it's wrong:**
- Three-part for loops are error-prone (off-by-one)
- `++`/`--` add operators with statement-only semantics
- None of this is clearer than alternatives

**The fix:**
- `for item in collection` for iteration
- `while condition` for conditional loops
- `x += 1` for increment

**Principle violated:** Small Surface Area, Predictable for AI

---

## Anti-Pattern 5: Type System Escape Hatches

**What happened:**
```rust
// In typechecker
Expr::ObjectLiteral(_) => Type::Unknown  // Returns Unknown
// In is_assignable_to
(Type::Unknown, _) => true  // Unknown fits anything!
```

**Why it's wrong:**
- Object literals silently satisfy any expected type
- Type errors hidden, surface at runtime
- AI learns that type mismatches are acceptable

**The fix:**
- Object literals have concrete structural types
- `Unknown` indicates ERROR, not flexibility
- Flowing `Unknown` into concrete position = compile error

**Principle violated:** Explicit Over Implicit

---

## Anti-Pattern 6: Spec Follows Implementation

**What happened:**
- `syntax.md` header: "Living document — reflects current implementation"
- Every feature added → spec updated to document it
- Spec became changelog, not design authority

**Why it's wrong:**
- No guard against bad grammar decisions
- AI agents read spec, assume everything in it is correct
- Mistakes get canonized

**The fix:**
- Design documents (`/docs/language-design/`) define what SHOULD exist
- Spec documents (`/docs/specification/`) track what DOES exist
- Design docs are authoritative; spec docs are descriptive

**Principle violated:** PRD principle "Spec-first: Define behavior before implementing"

---

## Anti-Pattern 7: "AI-First" as Blank Check

**What happened:**
- Decisions justified as "AI-first" without defining what that means
- Added features AI agents were actually confused by
- V02 Lessons explicitly noted the gap: "needs more concrete language design guidance"

**Why it's wrong:**
- "AI-first" became meaningless buzzword
- No concrete criteria to evaluate decisions
- Contradictory choices both claimed "AI-first" justification

**The fix:**
- "AI-first" is NOT a principle — it's the goal
- Cite concrete principles: "No Ambiguous Syntax", "Cohesion Over Sprawl"
- Test: "Can AI reliably generate this?" with actual generation attempts

**Principle violated:** All of them, by bypassing them

---

## Red Flags for Future Decisions

Reject any proposal that:

1. Adds a second way to do something already possible
2. Requires "but in this context it means..."
3. Is justified only as "AI-first" without citing specific principles
4. Marks existing syntax "deprecated" instead of removing it
5. Introduces backtracking or speculation in parser
6. Adds operators/keywords from C that Rust/Go removed

---

## References

- Grammar audit: `/docs/codex-findings/atlas-language-issues-advanced.md`
- Lessons learned: `/docs/internal/V02_LESSONS_LEARNED.md`
- Design principles: `/docs/language-design/PRINCIPLES.md`
