# Intentional Stubs

**Purpose:** Track features intentionally stubbed for future versions.

**Rule:** Stubs are BANNED except when documented here with user approval.

---

## How This Works

**If agent wants to stub:**
1. GATE 1.5 catches it (anti-pattern detection)
2. Agent asks user: "Should I stub this for [future version]?"
3. If user approves: Document here, implement stub
4. If user denies: Implement fully or split phase

**Before using any feature:**
1. GATE 1.5 Check 2: Read this file
2. If dependency is listed here: STOP, can't build on stub
3. Report to user: "Feature X depends on stubbed Y (v0.3 scope)"

---

## Currently Stubbed Features

(None - v0.2 starts with clean slate)

---

## Template for New Stubs

```markdown
## [Feature Name] (v0.X scope)
**Added:** 2026-XX-XX
**Location:** src/path/to/file.rs
**Functions/Modules:**
- function_name() - returns unimplemented!()
- module_name::* - all stubbed

**Reason:** [Why stubbed - design not ready, future scope, etc.]
**Version:** v0.X
**Dependencies:** [What features depend on this]
**Don't use until:** [Condition for implementation]

**User approved:** [Date and reason]
```

---

## GATE 1.5 Integration

**Check 2: Dependency Verification includes stub check:**

```
About to implement: HashMap with custom hasher

Check: Does hash function exist?
→ grep -r "fn hash" src/
→ FOUND in src/stdlib.rs

Check: Is hash function stubbed?
→ grep "hash" docs/reference/intentional-stubs.md
→ NOT FOUND (not stubbed)

✓ Safe to use
```

**If stubbed:**
```
About to implement: JIT-optimized loop

Check: Does JIT exist?
→ grep -r "fn jit_compile" src/
→ FOUND in src/vm/jit.rs

Check: Is JIT stubbed?
→ grep "jit" docs/reference/intentional-stubs.md
→ FOUND: "JIT Compilation (v0.3 scope) - intentionally stubbed"

⚠️ DEPENDENCY IS STUBBED

Can't implement feature that depends on stubbed JIT (v0.3 scope).

Options:
1. Wait for v0.3
2. Implement without JIT optimization
3. Implement JIT now (if user approves out-of-scope work)

Which approach?
```

---

## Anti-Stub Enforcement

**GATE 1.5 Check 5: Anti-Pattern Detection**

If agent plans to stub:
```
⚠️ PLANNING TO STUB

Your plan includes: "Stub jit_compile() for now"

COMPILER_PRINCIPLES.md: "No TODOs or stubs" (BANNED)

Options:
1. Implement fully now
2. Split into smaller phase
3. Document as intentional stub for v0.X (requires user approval)

Should I get user approval for intentional stub?
```

---

## Keeping This Clean

**Add stubs:**
- Only with user approval
- Only for future versions
- Document fully (why, when, dependencies)

**Remove stubs:**
- When implemented, delete entry
- Update "Currently Stubbed Features" count

**Audit:**
- Every 20 phases: Review this file
- Remove implemented stubs
- Verify remaining stubs still valid
