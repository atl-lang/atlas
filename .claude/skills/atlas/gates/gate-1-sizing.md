# GATE 1: Size Estimation (Compiler-Aware)

**Condition:** Dependencies verified

---

## Action

**Step 0 (MANDATORY FIRST):** Pull the current line counts from GATE 0 Step 5. Never estimate
in a vacuum — every projection must start from a real baseline.

1. For each target file, take current line count from GATE 0 Step 5
2. List all functions/features/components needed
3. Estimate lines per item
4. Sum new lines per file
5. Add 20% buffer to new lines
6. Projected final = current + buffered new lines
7. **DECLARE estimate and projection in output**

---

## Estimation Format (MANDATORY)

```
File: src/compiler/parser.rs
Current: 1,240 lines
- Feature 1: ~X lines
- Feature 2: ~Y lines
- Error handling: ~Z lines
New lines: ~N lines
Buffered (×1.2): ~M lines
PROJECTED FINAL: ~(1240 + M) lines
```

---

## Decision (Compiler-Aware)

Decisions are based on **projected final**, not just new lines:

- Projected final < 1,500 → Proceed → GATE 1.5
- Projected final 1,500–2,000 → Warning zone — justify OR plan split → GATE 1.5
- Projected final > 2,000 → **BLOCKING: plan split NOW, before writing a single line**
- Test file projected > 4,000 → **BLOCKING: plan subdirectory migration before adding tests**

**Split obligation:** If a projection hits the blocking threshold, the split plan is
**in-scope for this phase**. It is not follow-up work. Designing the split happens
here, before implementation. Writing a large file and splitting it at GATE 6 wastes
tokens and is disallowed.

**ARCH-EXCEPTION:** If the file legitimately cannot be split, add `// ARCH-EXCEPTION: <reason>`
at the top and document it in the phase summary. See `.claude/rules/atlas-architecture.md`.

---

## Compiler Reality

- Simple modules: 200-600 lines
- Standard modules: 600-1000 lines
- Complex modules: 1000-1500 lines (aim for this ceiling)
- Exception files only: 1500-2000+ lines (VM execute loop, main parser switch — requires ARCH-EXCEPTION)

---

## Split Planning (if needed)

1. Define module boundaries
2. List what goes in each file
3. Estimate lines per file
4. Document split plan

---

**CRITICAL:** Quality over line counts. NEVER simplify compiler logic for arbitrary limits.

---

## Foundation Check (formerly GATE 1.5)

**Before writing any code, verify:**

1. **Existing code audit:** Read existing code FIRST. Check against spec. Fix violations BEFORE adding new code.
2. **Dependencies not stubbed:** Grep for `unimplemented!()`, `todo!()` in dependencies.
3. **Architectural decisions:** Check auto-memory `decisions/*.md` for applicable decisions.
4. **Anti-patterns:** Planning stubs? Single-engine only? Skipping tests? → STOP.

**If issues found:** Fix BEFORE implementation. Don't waste tokens on bad foundations.

---

**Next:** GATE 2
