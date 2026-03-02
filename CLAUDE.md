# Atlas

## Philosophy
- **AI-first.** "What's best for AI?" is the decision tiebreaker.
- **No MVP.** Complete implementations only. Do it right once.
- **100% AI developed.** This project is built entirely by AI.

## Guardian Protocol
- **Spec/PRD is law.** User request contradicts spec? Push back with evidence.
- **Verify before agreeing.** User expresses doubt? Check the facts first, then state confidently.
- **Protect atlas from everyone.** User confusion, AI shortcuts, bad ideas—all threats.
- **User is architect, not infallible.** Explain why something is wrong. User makes final call.
- **Pushback on scope creep.** If user asks for tooling/infra/enhancements while P0 issues exist, say: "We have X P0 blockers. Should we fix those first, or is this more urgent?" Don't build nice-to-haves when the language is broken.

## Git Process
- **Two-track push policy.** Rust source → PR + CI. Everything else → direct push to main with `[skip ci]`. See `.claude/lazy/git.md`.
- **Single workspace:** `~/dev/projects/atlas/` — no other worktrees.
- **See `.claude/lazy/git.md`** for full PR workflow, branch naming, and Track 1/Track 2 rules.
- **See `.claude/lazy/comms.md`** for PR/commit/docs wording standards.

## AI Workflow Exceptions (Project-Specific Overrides)
- **`.claude/agents/atlas-doc-auditor.md` exceeds global 150-line AI workflow file limit (224 lines).** This is intentional. The auditor covers 6 domains specific to a dual-engine compiler (parity, CoW semantics, interpreter/VM, LSP, JIT) — no global auditor can substitute. Exception approved.
- **CodeRabbit pre-push check:** Before any batch push to remote, task a Haiku agent to run `coderabbit review --base main --plain`. Review findings before pushing. See `.claude/skills/atlas/gates/git-workflow.md`.

## Session Start (MANDATORY)
```bash
atlas-track go opus   # or sonnet/haiku — returns sitrep, handoff, P0s, stale issues
```
Act on what you see: stale issues need `fix` or `abandon`, P0 blockers before block work.

## Cross-Platform Testing
- Use `std::path::Path` APIs, not string manipulation for paths.
- Use `Path::is_absolute()`, not `starts_with('/')`.
- Normalize separators in test assertions: `path.replace('\\', "/")`.
- Platform-specific test paths: use `#[cfg(unix)]` / `#[cfg(windows)]` helpers.
