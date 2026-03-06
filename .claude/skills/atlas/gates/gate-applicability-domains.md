# Gate Applicability — Domain-Specific Details (Lazy-Loaded)

**Load when:** Need domain-specific testing/gate guidance beyond the quick reference.

---

## GATE 3 (Parity) — When to Run

**Run:**
- ✅ Runtime features (both interpreter and VM)
- ✅ Stdlib functions (identical behavior in both)
- ✅ VM bytecode changes (must match interpreter)

**Skip:**
- ❌ LSP, CLI, Frontend (no dual-engine)
- ❌ Documentation, tests themselves

**How:** `assert_parity(r#"len("hello")"#, "5");`

---

## GATE V (Versioning) — Event-Driven

Run at exactly two moments:
1. After `fix/` PR merges to main → patch tag check
2. After block AC check phase committed → minor version check

See `gates/gate-versioning.md`. Never ask user — exit criteria are authority.

---

## Domain Testing Patterns

### LSP
- Inline server creation (see `memory/testing-patterns.md`)
- No helper functions for server setup (lifetime issues)
- Check existing tests before writing new ones

### Runtime
- Domain test files (see `memory/testing-patterns.md`)
- No new test files without authorization
- Parity required for all features

### CLI
- Integration tests using `assert_cmd`
- Use `cargo_bin!` macro (not deprecated `cargo_bin()`)
- Test cross-platform paths (use `Path` APIs, not string manipulation)

### Documentation
- Commit directly to feature branch in `atlas-docs/` worktree
- Merge to local main when complete
- Push to GitHub on weekly cadence (no PR needed)

---

## Memory Update Triggers

- ✅ API surprise (undocumented pattern)
- ✅ Architectural decision (new constraint)
- ✅ Bug in existing patterns (fix the docs)
- ✅ Crate-specific behavior (document it)
- ❌ Following existing patterns
- ❌ Phase-specific one-time work
- ❌ Obvious/trivial changes
