# Atlas Skill - REBUILT CORRECTLY ✅

**Date:** 2026-02-13
**Status:** Production ready
**Result:** Compiler-aware skill for entire project lifespan

---

## What Was Wrong (First Version)

❌ **250-line limit** - Copied from EchoDev (app). Would force code simplification.
❌ **Phase-only workflow** - No support for bugs, refactoring, debugging, maintenance.
❌ **Empty domains/ folder** - Wrong concept, not used.
❌ **Not compiler-aware** - Treated compiler like an app.

**You were right:** This would produce bad code and wouldn't last beyond v0.2.

---

## What's Correct Now (Rebuilt Version)

✅ **Compiler-aware line limits** - 1000 line soft target, justified exceptions for complex modules
✅ **Quality over metrics** - Explicitly states: NEVER simplify code for line counts
✅ **5 workflow types** - Phase execution, bug fix, refactoring, debugging, feature addition
✅ **Project lifespan** - Works for v0.2, v0.3, bugs, maintenance, years from now
✅ **Realistic expectations** - Embraces compiler complexity

---

## Final Structure

```
.claude/skills/atlas/
├── skill.md (244 lines) - Main skill, compiler-aware
└── workflows/
    ├── phase-execution.md - v0.2 structured development
    ├── bug-fix.md - TDD approach for bugs
    ├── refactoring.md - Code cleanup without behavior changes
    ├── debugging.md - Systematic investigation
    └── feature-addition.md - Post-v0.2 features
```

**No domains/ folder** - Wrong concept, removed.
**workflows/ not gates/** - More descriptive, covers all work types.

---

## Line Limits (Corrected)

**Guideline:** 1000 lines soft target
**Reality:** Compiler modules are 600-2000 lines
**Approach:** Quality and correctness over arbitrary counts

| Component | Actual Size | Justification |
|-----------|-------------|---------------|
| VM | 1972 lines | Complex state machine, instruction dispatch |
| Bytecode | 1421 lines | Instruction encoding, serialization |
| Lexer | 908 lines | Tokenization, Unicode handling |
| Compiler | 869 lines | Code generation |
| AST | 671 lines | Node types, traversal |

**Key rule from skill:**
> **Simplifying code to meet line limits** → Quality over line count. NEVER dumb down compiler logic for arbitrary limits.

**This prevents the code quality reduction you saw in EchoDev.**

---

## Workflow Types

### 1. Phase Execution (`workflows/phase-execution.md`)
**When:** v0.2 structured development via phase files
**Gates:** 9-gate workflow (Read phase → BLOCKERS → Tests → Implementation → Parity → Quality → Docs → STATUS)

### 2. Bug Fix (`workflows/bug-fix.md`)
**When:** Fixing reported bugs
**Approach:** TDD (test first, then fix), both engines

### 3. Refactoring (`workflows/refactoring.md`)
**When:** Code cleanup, optimization, restructuring
**Rule:** Existing tests must still pass (no behavior changes)

### 4. Debugging (`workflows/debugging.md`)
**When:** Investigating issues, root cause analysis
**Approach:** Systematic (reproduce → hypothesize → narrow → root cause)

### 5. Feature Addition (`workflows/feature-addition.md`)
**When:** Adding features outside phase system (post-v0.2)
**Approach:** Similar to phases but more flexible

**This covers the entire project lifespan, not just v0.2 phases.**

---

## Key Differences from EchoDev

| Aspect | EchoDev | Atlas (Correct) |
|--------|---------|-----------------|
| **Project type** | TypeScript app | Rust compiler |
| **Line limits** | 200-330 lines | 1000 line soft target, justified exceptions |
| **Complexity** | Tiers (simple/standard/critical) | Workflows (phase/bug/refactor/debug/feature) |
| **File structure** | patterns/ | workflows/ |
| **Approach** | Line count enforcement | Quality over metrics |
| **Testing** | Lean (skip UI tests) | Comprehensive (parity required) |
| **Lifespan** | App maintenance | Compiler evolution (decades) |

**Atlas is adapted FOR a compiler, not blindly copied from an app.**

---

## Compiler Complexity Reality

**From skill.md:**

> **Atlas is not a CRUD app:**
> - Lexer handles Unicode, complex tokenization
> - Parser does recursive descent with error recovery
> - Typechecker performs constraint-based inference
> - Bytecode compiler generates optimized instructions
> - VM executes complex state machine
> - Interpreter provides REPL with instant feedback

> **This means:**
> - Files will be larger than typical app code
> - Complex algorithms are necessary
> - Trade-offs favor correctness over simplicity
> - Quality matters more than arbitrary metrics

> **Focus on:**
> - Correct implementation of language semantics
> - Clear separation of compiler phases
> - Comprehensive test coverage
> - Interpreter/VM parity
> - Following language specification exactly

> **Don't:**
> - Simplify compiler logic to meet line limits
> - Skip complexity for easier code
> - Cut corners on edge cases
> - Compromise correctness for speed

**This prevents AI agents from dumbing down compiler code.**

---

## Project Lifespan Coverage

### v0.2 Development (Now)
- Use **Phase Execution** workflow
- Structured, gate-based development
- Follow phase files in `phases/`

### Bug Reports (Anytime)
- Use **Bug Fix** workflow
- TDD approach
- Both engines fixed

### Code Quality (Ongoing)
- Use **Refactoring** workflow
- Incremental improvements
- Tests still pass

### Investigating Issues (Anytime)
- Use **Debugging** workflow
- Systematic analysis
- Root cause identification

### Post-v0.2 Features (Future)
- Use **Feature Addition** workflow
- More flexible than phases
- Still maintain standards

### Years From Now
- **All workflows still apply**
- Skill doesn't expire after v0.2
- Adapts to project evolution

---

## Comparison to EchoDev Pattern

**What Atlas DOES mirror from EchoDev:**
✅ Skill = workflow router, docs = source of truth
✅ Doc-driven evolution (docs grow, skill stays stable)
✅ References project docs (not hardcoded)
✅ Compact skill file (~244 lines)
✅ Workflow-specific pattern files
✅ Build commands documented
✅ Universal rules enforced

**What Atlas DOESN'T copy (correctly adapted):**
✅ Line limits (compiler-aware, not app-aware)
✅ Workflow types (task-based, not tier-based)
✅ Quality emphasis (correctness > metrics)
✅ Testing approach (comprehensive parity, not lean)
✅ Long-term thinking (decades, not sprints)

**Result:** Mirrors EchoDev's STRUCTURE but adapted FOR Atlas's NEEDS.

---

## Verification

```bash
# Skill exists
ls .claude/skills/atlas/skill.md

# Workflows exist
ls .claude/skills/atlas/workflows/
# phase-execution.md
# bug-fix.md
# refactoring.md
# debugging.md
# feature-addition.md

# Correct structure (no domains/)
ls .claude/skills/atlas/domains  # Does not exist ✓

# Realistic line limits in skill.md
grep "1000 lines" .claude/skills/atlas/skill.md  # Found ✓
grep "250" .claude/skills/atlas/skill.md  # Not found ✓
```

---

## How To Use

### Invoke skill:
```
Use the atlas skill for [task description]
```

### Skill will:
1. Ask you to declare workflow type
2. Route to appropriate workflow file
3. Enforce Atlas-specific rules
4. Reference correct docs
5. Ensure quality standards

### Example:
```
User: "Use atlas skill to fix the parser bug with array literals"
Skill: Identifies as Bug Fix workflow
Skill: Routes to workflows/bug-fix.md
Skill: TDD approach, parity check, quality gates
```

---

## Success Criteria - All Met

✅ Removed wrong 250-line limit
✅ Added compiler-aware line limits (1000 soft, justified exceptions)
✅ Emphasized quality over metrics
✅ Created 5 workflow types (covers entire lifespan)
✅ Removed empty domains/ folder
✅ Renamed to workflows/ (more descriptive)
✅ Mirrors EchoDev structure but adapted for compiler
✅ Works for v0.2, post-v0.2, bugs, maintenance, years from now
✅ Prevents code simplification for arbitrary limits
✅ Comprehensive, not just phases

---

## The Bottom Line

**First version:** Blindly copied EchoDev. Would produce bad compiler code.
**Rebuilt version:** Adapted FOR Atlas. Embraces compiler complexity. Lasts forever.

**You caught it before it caused problems. Good instinct.**

**Now:** Atlas skill is correct, compiler-aware, and will last the entire project lifespan.

---

**Atlas Skill: Compiler-First. Quality-First. Built For Decades. ✅**
