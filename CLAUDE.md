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

## Git Process (Local-First v2)
- **Local CI first.** All validation via `cargo fmt/clippy/nextest` + `coderabbit` CLI locally.
- **Batch pushes.** Commits accumulate on local main. Push after 5 commits OR 24 hours.
- **No PRs for fixes.** Direct push to main after local CI passes. PRs only for major blocks.
- **Single workspace:** `~/dev/projects/atlas/` — no other worktrees.
- **See `.claude/lazy/git.md`** for full local-first workflow.

## Local CI Commands

```bash
# Quick (every fix)
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo nextest run -p atlas-runtime

# Full (batched — Haiku agent)
coderabbit review --base main --plain
cargo fmt --check && cargo clippy --workspace -- -D warnings
cargo build --workspace && cargo nextest run --workspace
```

Track batch state in `.claude/memory/local-ci.md`.

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
