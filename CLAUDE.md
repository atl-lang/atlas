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
- **Before deleting "incomplete" code:** Check git history + `atlas-track decisions`. Ask user if uncertain.
- **See `docs/known-issues.md`** for current bugs and limitations.

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
1. **File an issue immediately:** `atlas-track add "Missing: X" P1 "reason"`
2. **Flag to user:** "I noticed Atlas lacks X. Most languages have this. Filed as issue."

If AI has to work around something that should be built-in, that's a bug, not a feature request.

- **New doc/rule/skill/agent file?** Update `.claude/agents/atlas-doc-auditor.md` to include it in the correct audit domain. The hook will remind you, but do it.

## Git Process (Local-First v2)
- **Local CI first.** All validation via `cargo fmt/clippy/nextest`.
- **Batch pushes.** Commits accumulate on local main.
- **Single workspace:** `~/dev/projects/atlas/`
- **See `.claude/lazy/git.md`** for full workflow.

## Testing Strategy (CRITICAL)
```bash
# DURING DEVELOPMENT — two commands only:
cargo check -p atlas-runtime                                        # verify compile, ~0.5s
cargo nextest run -p atlas-runtime -E 'test(exact_test_name)'      # ONE test by exact name

# BANNED — these compile ALL test binaries and cause 5-20 min hangs:
# cargo nextest run -p atlas-runtime -E 'test(interpreter)'   ❌
# cargo nextest run -p atlas-runtime -E 'test(stdlib)'        ❌
# cargo nextest run -p atlas-runtime --test <any_domain>      ❌
# cargo nextest run -p atlas-runtime                          ❌
# cargo nextest run --workspace                               ❌

# NEVER run full suite manually.
# The pre-commit Guardian hook (.githooks/pre-commit) runs full suite + parity on every commit.
# Killing cargo mid-run leaves lock files that block all future runs — never do it.
```

## Session Start (MANDATORY)
```bash
atlas-track go opus   # or sonnet/haiku — returns sitrep, handoff, P0s, stale issues
```

## Auto-Loaded Rules (no need to read manually)
These load automatically based on which files you're editing:
- `atlas-language-ref.md` — Atlas syntax quick reference (on `.atlas`/`.atl` files)
- `atlas-context-guard.md` — Context window management protocol (on `.rs` files)
- `atlas-cross-platform.md` — Cross-platform testing rules (on test/source files)
- `atlas-parity.md` — Interpreter/VM parity contract (on interpreter/VM/compiler files)
- `atlas-testing.md` — Test organization rules (on test files)
- `atlas-ast.md`, `atlas-typechecker.md`, `atlas-interpreter.md`, `atlas-vm.md`, `atlas-syntax.md`
