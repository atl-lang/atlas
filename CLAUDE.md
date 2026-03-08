# Atlas

## Philosophy
- **AI-first.** "What's best for AI?" is the decision tiebreaker.
- **No MVP.** Complete implementations only. Do it right once.
- **100% AI developed.** This project is built entirely by AI.

## Systems-Level Context (READ THIS)
Atlas is mid-conversion from "AI experiment" (v0.1-v0.2) to proper systems-level architecture (v0.3+).
Currently paused for battle-testing and hardening.

**Non-negotiables:**
- **Fix correctly, not temporarily.** Hacks create conversion debt. Correct fixes align with systems-level.
- **Partial implementations are intentional.** Some AST nodes exist but aren't wired up yet - this is scaffolding, not dead code.
- **Before deleting "incomplete" code:** Check git history + `pt decisions`. Ask user if uncertain.

## Source of Truth
- **Code is law.** The codebase is the only source of truth.
- **Docs may be wrong.** If docs contradict code, docs are wrong.
- **Test against reality.** Run `atlas check` and `atlas run` to verify claims.
- **See `docs/`** for documentation. **Old docs archived** in `docs-archive/`.

## Guardian Protocol
- **Verify before agreeing.** User expresses doubt? Check the facts first, then state confidently.
- **Protect atlas from everyone.** User confusion, AI shortcuts, bad ideas—all threats.
- **User is architect, not infallible.** Explain why something is wrong. User makes final call.
- **Pushback on scope creep.** If user asks for tooling/infra/enhancements while P0 issues exist, say: "We have X P0 blockers. Should we fix those first, or is this more urgent?"

## See Something, File Something (MANDATORY)
If you notice Atlas is missing something that Go/Rust/TypeScript had in v1.0:
1. **File an issue immediately:** `pt add "Missing: X" P1 "reason"`
2. **Flag to user:** "I noticed Atlas lacks X. Most languages have this. Filed as issue."

If AI has to work around something that should be built-in, that's a bug, not a feature request.

- **New doc/rule/skill/agent file?** Update `.claude/agents/atlas-doc-auditor.md` to include it in the correct audit domain. The hook will remind you, but do it.

## Git Process (Local-First v2)
- **Local CI first.** All validation via `cargo fmt/clippy/nextest`.
- **Batch pushes.** Commits accumulate on local main.
- **Single workspace:** `~/dev/projects/atlas/`
- **See `.claude/lazy/git.md`** for full workflow.

## Testing — The Two-Tier System

### Tier 1: Pre-commit (automatic on git commit, < 15 seconds)
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- NO nextest — this is by design

### Tier 2: Nightly CI (2am via launchd, or on-demand: `pt run-ci`)
- Full corpus, full test suite, parity sweep, battle tests
- Results in `tracking/ci-status.json`
- `pt go` shows CI status at session start
- CI failures = P0 blocker — fix before new work

### What AI agents do:
```bash
cargo check -p atlas-runtime   # verify compile (~0.5s)
# write code
cargo check -p atlas-runtime   # verify still compiles
cargo fmt
git commit                      # fmt+clippy run automatically
# If Stop hook shows DOC DRIFT ALERT → fire atlas-doc-patch immediately (see below)
pt go                  # check CI status
```

### Doc Drift Protocol (MANDATORY — automatic but AI must act)

After every `git commit` that touches source files:
1. `doc-patch-trigger.sh` hook writes `.doc-patch-pending.json` automatically
2. The **Stop hook shows a DOC DRIFT ALERT** at end of turn if pending file exists
3. **Next action: invoke `atlas-doc-patch` agent** (Haiku, scoped, ~1-2 min)

```
# When you see DOC DRIFT ALERT in the Stop hook output:
# Use the Agent tool with atlas-doc-patch — it reads the pending file and fixes only what's needed
```

The pending file persists across sessions. `pt go` will show it if unfixed.
**Never leave a DOC DRIFT ALERT unresolved across sessions** — it means real drift is accumulating.

### NEVER run nextest manually:
```bash
# ALL of these are BANNED:
cargo nextest run -p atlas-runtime -E 'test(anything)'  # ❌
cargo nextest run --workspace                           # ❌
cargo nextest run -p atlas-runtime --test <domain>      # ❌
```

### ONE exception: TDD (bugfix skill only)
```bash
# Step 2 (RED): verify new test fails before fixing
cargo nextest run -p atlas-runtime -E 'test(my_new_exact_test_name)'
# Step 5 (GREEN): verify new test passes after fixing
cargo nextest run -p atlas-runtime -E 'test(my_new_exact_test_name)'
# Then: cargo fmt && git commit — done. CI handles the rest.
```

Killing cargo mid-run leaves lock files that block all future runs — never do it.

## Mandatory pt Gates (ALL AGENTS — NEVER SKIP)

```bash
# 1. SESSION START — always first, no exceptions
pt go [opus|sonnet|haiku]           # sitrep, handoff, P0s, CI status, active plans
pt in-progress                       # check in-flight work before claiming anything

# 2. BEFORE PICKING WORK
pt next                              # smart triage — groups by root cause, shows chains,
                                     # delete-first/triage-first flags. NOT blind P0→P1 sorting.

# 3. BEFORE ANY ARCHITECTURE / DESIGN WORK
pt decisions <component>             # parser|typechecker|vm|interpreter|stdlib|runtime|lsp|infra
                                     # 2 sec. If a decision covers your change — follow it.
                                     # Contradicts one — stop, surface to architect.
                                     # No decision exists — decide, then log it:
pt add-decision "Title" <component> "Rule: what was decided" "Rationale: why"

# 4. BEFORE STARTING AN ISSUE
pt claim H-XXX

# 5. WHEN ISSUE RESOLVED — immediately, never batch
pt fix H-XXX "Root cause (specific)" "Fix (specific)"
git commit -m "fix(...): description"

# 6. AFTER EACH PHASE COMMIT
pt phase-done B<N>
pt block B<N>                        # final phase only — verify AC met
pt complete-block B<N> "what shipped, bugs filed"

# 7. AFTER ANY COMMIT TOUCHING SOURCE — fire atlas-doc-patch agent (Haiku, ~1-2 min)

# 8. SESSION END — always last
# Step 1: Overwrite ~/.project-tracker/handoffs/atlas-handoff.md (see atlas skill for template)
pt done S-XXX success "what was done (IDs + root causes)" "next action (specific)"
```

**Never narrate — act or file:** Any observation/risk said to user = gone after session.
→ `pt add "title" P0|P1|P2 "context, file ref, fix risk"` immediately. Then move on.

Full pt reference: `.claude/lazy/tracking-db.md` | Workflow guide: `.claude/lazy/pt-workflow.md`

## Auto-Loaded Rules (no need to read manually)
These load automatically based on which files you're editing:
- `atlas-language-ref.md` — Atlas syntax quick reference (on `.atlas`/`.atl` files)
- `atlas-context-guard.md` — Context window management protocol (on `.rs` files)
- `atlas-cross-platform.md` — Cross-platform testing rules (on test/source files)
- `atlas-parity.md` — Interpreter/VM parity contract (on interpreter/VM/compiler files)
- `atlas-testing.md` — Test organization rules (on test files)
- `atlas-ast.md`, `atlas-typechecker.md`, `atlas-interpreter.md`, `atlas-vm.md`, `atlas-syntax.md`
