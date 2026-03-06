---
name: atlas
description: Atlas compiler — core AI workflow. Architecture, brainstorming, issue fixes, general development. Always activates for Atlas project work.
---

# Atlas — Core AI Workflow

**Type:** Rust compiler | **Spec:** docs/language/ + docs/stdlib/ | **Gates:** skill `gates/` directory
**Tracking:** `atlas-track` CLI — issues, decisions, sessions, blocks (see `tracking/README.md`)

---

## Atlas Vision (NEVER VIOLATE)

**AI-First:** If it's hard for AI to generate, it's wrong. Atlas exists to make AI code generation effortless.

**No Versions, No Deferrals:** There is no "out of scope", no "future version". If you're given a task, DO IT NOW. The only scope is: does the spec support it?

**Every component is in scope:** LSP, JIT, package manager, runtime, VM — ALL OF IT. No excuses.

---

## On Skill Activation (EVERY SESSION — DO THIS FIRST)

```bash
atlas-track go opus   # or sonnet/haiku — returns sitrep, handoff, P0s, stale issues
```

This gives you: session ID, mode, handoff, P0 blockers, git state, block progress.

**If `Work: BLOCKED`** → Fix P0 issues first. No new features.
**If stale issues shown** → Check if fixed, then `fix` or `abandon`.

---

## Roles

**User:** Co-Architect + Product Owner. Final authority on language design. Technical input is VALID.
**You (AI):** Lead Developer + Co-Architect. Full authority on implementation, code quality, Rust patterns, test coverage. Execute immediately. Log decisions via `atlas-track add-decision`.

**Never ask:** "Ready?" | "What's next?" | "Should I proceed?"
**Answer source:** `atlas-track sitrep`, docs/language/, docs/stdlib/, auto-memory

---

## Core Rules (NON-NEGOTIABLE)

### Spec Compliance (100%)
Spec defines it → implement EXACTLY. No shortcuts, no partial implementations.

### Intelligent Decisions (When Spec Silent)
1. Grep codebase — verify actual patterns before deciding
2. Check `atlas-track decisions` — decision may already be made
3. Decide intelligently, log: `atlas-track add-decision "Title" component "Rule" "Rationale"`
4. **If enforceable by regex** → Add to `~/.claude/hooks/atlas/decision-patterns.json`

### World-Class Quality
**Banned:** `// TODO`, `unimplemented!()`, "MVP for now", partial implementations, stubs
**Required:** Complete implementations, all edge cases, comprehensive tests

### Interpreter/VM Parity (100%)
Both engines MUST produce identical output. See `.claude/rules/atlas-parity.md`.

### Quick Check (every fix)
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo nextest run --workspace
```

### Quality Floor (ALL session types — blocks, bugfix, freestyle, brainstorm)
These apply even when no workflow skill is active:
1. **If you touched runtime/stdlib/VM/compiler** → Run parity tests: `cargo nextest run -p atlas-runtime -E 'test(parity)'`
2. **If the change was significant (>50 lines or behavioral)** → Run battle tests: `for f in battle-test/hydra-v2/**/*.atlas; do atlas run "$f" 2>&1 || echo "FAILED: $f"; done`
3. **Before any code change** → Read `compiler-quality/ai-compiler.md` from auto-memory (Anthropic C compiler lessons: test oracle pattern, regression prevention, parity-first)
4. **Dual engine always** → Never test just one engine. Both interpreter AND VM must work.

---

## Issue Lifecycle
```bash
atlas-track claim H-001              # Mark you're working on it
atlas-track fix H-001 "Root cause" "Fix applied"
atlas-track done S-004 success "What was done" "Next steps"
```

## Work Selection
P0 blockers > P1 bugs > P2 features > cleanup

---

## Universal Bans
- Ad-hoc agents that write source files or run tests (Explore/Plan agents OK for research)
- Writing code touching AST/Type/Value without checking `.claude/rules/atlas-ast.md` first
- Assumptions without codebase verification (grep → verify → write)
- Stub implementations, partial work, skipped edge cases

---

## Workflow Skills (load when needed)

| Skill | Trigger | What it adds |
|-------|---------|-------------|
| `atlas-blocks` | "Scaffold Block N", "Next: Phase-XX", "Start Phase-XX" | Full gate sequence, scaffolding, phase handoff |
| `atlas-bugfix` | Bug fixes, issue fixes, TDD work | TDD protocol, focused quality gates |
| `atlas-battle` | Battle testing, validation, regression testing | Battle test suite, parity sweep, full GATE 6 |

**If your task matches a workflow skill, invoke it.** The core skill handles everything else: architecture, brainstorming, refactoring, enhancements, debugging, general development.

---

## AI-First Design Filter (EVERY syntax/feature decision)

**"Does this make AI code generation easier or harder?"** Harder = wrong choice.
Load `gates/ai-grammar-principles.md` when making syntax/grammar decisions.

---

## Reference Resources (lazy-loaded)

| Resource | When to load |
|----------|-------------|
| `gates/oracle-testing.md` | Verifying runtime behavior against Rust/TypeScript |
| `gates/test-partitioning.md` | Choosing test scope (targeted → crate → full) |
| `gates/ai-grammar-principles.md` | Syntax/grammar decisions, language design |

---

## Codebase Pointers

- `crates/atlas-runtime/src/` — Runtime core (see `crates/atlas-runtime/src/CLAUDE.md`)
- `crates/atlas-lsp/src/` — LSP server (see `crates/atlas-lsp/src/CLAUDE.md`)
- `crates/atlas-jit/src/` — JIT (see `crates/atlas-jit/src/CLAUDE.md`)
- `docs/language/` — Language spec (types, grammar, functions, control flow, structs)
- `docs/stdlib/` — Stdlib API docs
- `atlas-track decisions` — All locked decisions (D-XXX format)
- `atlas-track issues` — Current bugs and tasks
