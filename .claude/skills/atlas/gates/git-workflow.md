# Git Workflow

**Full rules:** `.claude/lazy/git.md`
**Single workspace:** `~/dev/projects/atlas/` — no other worktrees.

## Local-First CI (v2)

All validation happens locally. Remote pushes are batched.

### Quick Checks (every fix)
```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo nextest run -p atlas-runtime
```

### Full Local CI (Haiku agent — batched)
```bash
coderabbit review --base main --plain
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo build --workspace
cargo nextest run --workspace
```

**Trigger:** 5 commits OR 24 hours — track in `.claude/memory/local-ci.md`

---

## Commit Flow

### Track 1 — Non-Rust (commit directly)
```bash
git add <files> && git commit -m "docs(spec): update examples"
```

### Track 2 — Rust source (commit + quick checks)
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings
cargo nextest run -p atlas-runtime
git add crates/ && git commit -m "fix(vm): description"
```

Both tracks commit to `main` locally. Push happens at batch time.

---

## Batch Push (Daily)

When 5+ commits accumulated or 24h elapsed:

```bash
# 1. Haiku agent runs full local CI
coderabbit review --base main --plain
act -j Build -j Clippy -j Format
cargo nextest run --workspace

# 2. Push if all pass
git push origin main

# 3. Update .claude/memory/local-ci.md timestamp
```

---

## Block PR Workflow (Major Features Only)

PRs reserved for block completions:

```bash
git checkout -b block/{name}
# ... implement all phases ...
git push -u origin block/{name}
gh pr create --title "feat(block-XX): ..." --body "$(cat <<'EOF'
## Summary
...

## Test plan
...

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
gh pr merge --auto --squash

# After merge
git checkout main && git pull origin main
git branch -d block/{name}
```

---

## Haiku Agent Prompt (for batched CI)

```
Run full local CI in ~/dev/projects/atlas:

1. coderabbit review --base main --plain
2. act -j Build -j Clippy -j Format --container-architecture linux/amd64
3. cargo nextest run --workspace

Return: pass/fail status + any error output. Do not fix — report only.
```

---

## Banned

- PRs for individual fixes
- Pushing without local CI
- Remote branches except main/gh-pages/block/*
- `--force` on main
- `--no-verify`
