# Atlas-SPEC.md Restructure Plan

**Goal:** Split 786-line spec into focused files with AI-friendly lazy-loading

**Current state:** 81 references, 25kb, will grow to 100-200kb
**Target state:** ~100-line index + 6-8 focused spec files (~5-15kb each)

---

## New Structure

### Atlas-SPEC.md (INDEX - ~100 lines)
**Purpose:** Quick reference + routing to detailed specs
**AI Instruction:** "Only read specific spec files when needed for your task"

**Content:**
- Project goals (10 lines)
- File format (.atl extension)
- Quick type reference (1-2 lines per type)
- **ROUTING TABLE** - Clear instructions on when to read which spec
- Version/milestone overview

**Example routing:**
```markdown
## Navigation Guide for AI Agents

**CRITICAL:** Do NOT read all spec files. Use this routing:

- **Implementing types?** → Read `docs/specification/types.md`
- **Parser/grammar work?** → Read `docs/specification/syntax.md`
- **Type checking/semantics?** → Read `docs/specification/semantics.md`
- **Stdlib functions?** → Read `docs/api/stdlib.md`
- **Module system?** → Read `docs/specification/modules.md`
- **REPL behavior?** → Read `docs/specification/repl.md`
- **Error handling?** → Read `docs/specification/diagnostics.md`
- **Bytecode/VM?** → Read `docs/specification/bytecode.md`
```

---

## New Specification Files

### 1. docs/specification/types.md (~150 lines)
**From Atlas-SPEC.md:** Lines covering types section
**Content:**
- Primitive types (number, string, bool, null, void)
- Array types syntax
- Function types (full section with examples)
- JSON type (full section with indexing)
- Generic types (syntax, constraints, inference)
- Pattern matching types (when added)
- Typing rules
- Type assignability rules

### 2. docs/specification/syntax.md (~200 lines)
**From Atlas-SPEC.md:** Lexical + Grammar sections
**Content:**
- Lexical structure (whitespace, comments)
- Keywords (complete list)
- Literals (numbers, strings, booleans, null, arrays)
- String escapes
- Expressions (all operators, precedence)
- Statements (all statement forms)
- Grammar (EBNF rules)

### 3. docs/specification/semantics.md (~150 lines)
**Merge:** Atlas-SPEC semantics + existing language-semantics.md
**Content:**
- Execution model
- Evaluation order
- Type checking rules
- Array aliasing semantics
- Numeric edge cases
- Short-circuit evaluation
- Variable scoping rules

### 4. docs/specification/runtime.md (~100 lines)
**From Atlas-SPEC.md:** Runtime + Value model sections
**Content:**
- Runtime model overview
- Value representation
- Memory model (Rc, RefCell usage)
- Function calling conventions
- Stack management

### 5. docs/specification/modules.md (~100 lines)
**From Atlas-SPEC.md:** Module system sections
**Content:**
- Import/export syntax
- Module resolution rules
- Cyclic dependency handling
- Module caching
- Package structure

### 6. docs/specification/repl.md (~80 lines)
**From Atlas-SPEC.md:** REPL sections
**Content:**
- REPL vs file mode differences
- Statement separators
- Multi-line input
- REPL-specific features
- Interactive mode behavior

### 7. docs/specification/bytecode.md (~120 lines)
**From Atlas-SPEC.md:** Bytecode VM + Compiler IR
**Content:**
- Bytecode format
- Instruction set
- Compiler IR design
- VM execution model
- Optimization opportunities

### 8. docs/specification/diagnostics.md
**Already exists** - keep as-is (12kb, comprehensive)

### 9. docs/api/stdlib.md
**Already exists** - keep as-is (8kb, comprehensive)

---

## Reference Updates Required

**81 references to update across:**
- Phase files in phases/
- Atlas skill files in .claude/skills/atlas/
- Documentation in docs/
- Archive files (low priority, can leave as-is)

**Update pattern:**
```markdown
# OLD
See Atlas-SPEC.md for type rules

# NEW
See `docs/specification/types.md` for complete type system spec
```

**Skill file updates:**
- `.claude/skills/atlas/skill.md` - Update spec reference
- `.claude/skills/atlas/gates/gate-0-read-docs.md` - Add routing logic
- `.claude/skills/atlas/gates/gate-1.5-foundation.md` - Update checks
- `.claude/skills/atlas/gates/gate-5-docs.md` - Update doc update rules

---

## AI Lazy-Loading Strategy

**Key principle:** Index provides routing, agents only read what they need

**In phase files:**
```markdown
# OLD (forces reading entire spec)
**Spec:** Atlas-SPEC.md

# NEW (targeted loading)
**Type spec:** docs/specification/types.md
**Grammar spec:** docs/specification/syntax.md
```

**In Atlas skill gate-0:**
```markdown
## What to Read (Selective)

**Type/grammar work:**
1. Read STATUS.md
2. Read Atlas-SPEC.md (index only)
3. Read specific spec from routing table

**Do NOT read all spec files - use the routing table**
```

---

## Execution Plan

### Phase 1: Create new spec files
1. Extract and write types.md
2. Extract and write syntax.md
3. Merge semantics (Atlas-SPEC + language-semantics.md)
4. Extract and write runtime.md
5. Extract and write modules.md
6. Extract and write repl.md
7. Extract and write bytecode.md

### Phase 2: Create index
1. Write new Atlas-SPEC.md with routing table
2. Clear AI instructions on lazy-loading
3. Quick reference for common lookups

### Phase 3: Update references
1. Update atlas skill files (skill.md, gates/)
2. Update phase files in phases/
3. Update docs/ references
4. Update STATUS.md doc map

### Phase 4: Verify
1. Test file sizes (all under 15kb)
2. Grep for broken references
3. Verify routing clarity

---

## Success Criteria

✅ Atlas-SPEC.md under 3kb (index only)
✅ No spec file over 15kb
✅ All 81 references updated or verified
✅ Clear routing for AI agents
✅ Atlas skill enforces lazy-loading
✅ No information loss from original spec
✅ Related content properly merged (e.g., semantics)

---

## Token Savings Estimate

**Current:** 25kb spec × 81 references = potential 2M+ tokens/session waste
**After split:** 3kb index + 5-10kb targeted spec = 90% reduction
**Long-term:** When spec grows to 150kb, savings are 95%+

**This pays for itself in ~10 sessions.**
