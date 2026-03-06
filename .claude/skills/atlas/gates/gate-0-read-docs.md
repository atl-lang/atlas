# GATE 0: Environment Prep + Read Docs

**Condition:** Starting any task

---

## Step 1: Clean Build Artifacts

```bash
cargo clean
```
**ONE TIME ONLY** at task start. Prevents GB-level artifact accumulation.

---

## Step 2: Read Docs (Selective Reading)

1. **ALWAYS:** Run `atlas-track sitrep`
2. **IF scaffolding session:** Also read `ROADMAP.md` and `docs/internal/V03_PLAN.md`
3. **IF structured development:** Read complete development plan (phase file)
4. **SELECTIVE:** Read ONLY the spec files your task needs (routing below)
5. **CHECK EXISTING CODE:** Before writing tests, read existing test files in the target crate

### Spec Routing (DO NOT read all specs)

| Task area | Read |
|-----------|------|
| Types/generics | `docs/language/types.md` |
| Parser/grammar | `docs/language/grammar.md` |
| Functions | `docs/language/functions.md` |
| Control flow | `docs/language/control-flow.md` |
| Structs/enums | `docs/language/structs-enums.md` |
| Traits/impl | `docs/language/traits.md` |
| Modules/import/export | `docs/language/modules.md` |
| Stdlib | `docs/stdlib/index.md` → specific module |
| Path operations | `docs/stdlib/path.md` |
| CLI/tooling | `docs/tooling/cli.md` |

### Implementation Patterns (As Needed)

- Code patterns: auto-memory `patterns.md` (index → load specific `patterns/*.md`)
- Domain verification: auto-memory `domain-prereqs.md`
- Decisions: `atlas-track decisions`
- Testing: `.claude/rules/atlas-testing.md` (auto-loaded)

---

## Lazy Loading Rules

**DO:** Use routing table, read ONLY relevant specs.
**DON'T:** Read all spec files at once, skip routing, guess which spec.
**Token savings:** 80-95% (5-15kb instead of 150kb)

---

## Step 3-5: Load IF needed

| Step | File | Load when |
|------|------|-----------|
| Dependency Check | `gates/gate-0-deps.md` | Block/phase work (`atlas-blocks` skill) |
| Domain Verification | auto-memory `domain-prereqs.md` | Writing code touching AST/Type/Value |
| Arch Health Check | `gates/gate-0-arch.md` | About to write/modify source files |

---

**BLOCKING:** Cannot proceed without understanding current state and requirements.
**Next:** GATE 1
