
# Atlas Git Workflow

**Single workspace:** `~/dev/projects/atlas/` on `main`. No other worktrees.
**100% AI maintained.** proxikal handles GitHub UI only (rulesets, secrets, billing).

---

## Local-First Policy (v2 — March 2026)

All CI and review happens locally. Remote pushes are batched.

### Quick Checks (after every fix)
```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo nextest run --workspace
```

### Full Local CI (batched — Haiku agent)
Trigger: **every 168 hours** 
```bash
coderabbit review --base main --plain   # local review
cargo fmt --check                       # format check
cargo clippy --workspace -- -D warnings # lint
cargo build --workspace                 # build
cargo nextest run --workspace           # full test suite
```

Track state in `.claude/memory/local-ci.md`.

---

## Two-Track Commit Policy

### Track 1 — Direct commit to main (no PR)

Use for ANY change that does not touch Rust source:

| Change type | Examples |
|-------------|---------|
| CI/workflows | `.github/workflows/*.yml` |
| AI workflow | `.claude/**`, `phases/**` |
| Config | `.coderabbit.yaml`, `deny.toml`, `rust-toolchain.toml` |
| Docs | `docs/**`, `**.md` |
| Cargo metadata only | Version bumps, `[package]` fields, no new deps |

```bash
git add <files> && git commit -m "docs(spec): update closure syntax examples"
# Push happens at batch time, not per-commit
```

### Track 2 — Commit to main, batch PR later

For Rust source changes (`crates/**/*.rs`, `Cargo.toml` deps):

```bash
# 1. Quick local checks
cargo fmt --check && cargo clippy --workspace -- -D warnings
cargo nextest run --workspace

# 2. Commit to main locally
git add crates/ && git commit -m "fix(vm): resolve side effect issue"

# 3. DO NOT PUSH YET — accumulate commits
# Push happens at batch time after full local CI
```

---

## Batch Push Workflow

**Check if due:** `git fetch origin && git log origin/main -1 --format="%ci"` — push if 168+ hours ago

```bash
# 1. Full local CI (Haiku agent)
coderabbit review --base main --plain
act -j Build -j Clippy -j Format
cargo nextest run --workspace

# 2. If all pass, push to remote
git push origin main

# 3. Update tracking
# Edit .claude/memory/local-ci.md with timestamp
```

**No PRs for routine fixes.** Direct push to main after local CI validates.

---

## PR Workflow (Blocks Only)

PRs are reserved for major block completions, not individual fixes.

```bash
# Only at block completion:
git checkout -b block/{name}
# ... all block phases ...
git push -u origin block/{name}
gh pr create --title "feat(block-XX): ..." --body "..."
gh pr merge --auto --squash
```

---

## Branch Hygiene

- **At most 2 remote branches:** `main` + `gh-pages`
- Block branches are temporary (created at block end, deleted after merge)
- No `fix/`, `ci/`, `docs/` branches — commit directly to main

---

## Local CI State Tracking

After each full local CI run, record the result inline here or in a scratch note:

```
Last Full Check: <timestamp>
Agent: <opus|sonnet|haiku>
Result: pass|fail
Commits since: <N>
```

**Check if batch push due:** `git fetch origin && git log origin/main -1 --format="%ci"` — push if 168+ hours ago.

---

## Banned

- PRs for individual fixes — batch push instead
- Pushing without local CI validation
- `--force` without `--force-with-lease`
- `--no-verify`
- Creating branches for non-block work
