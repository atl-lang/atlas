# Git Workflow Quick Reference

**Canonical source:** `.claude/lazy/git.md` — read for full policy details.

---

## Quick Commands

### After Every Fix
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo nextest run --workspace
```

### Batch Push Check (168h policy)
```bash
git fetch origin && git log origin/main -1 --format="%ci"
```

### Keep Branch Current
```bash
git rebase origin/main
```

---

## Two-Track Summary

| Track | What | Action |
|-------|------|--------|
| 1 | Non-Rust (docs, CI, .claude/, config) | Commit to main, push at batch time |
| 2 | Rust source (crates/**/*.rs) | Quick checks → commit to main, push at batch time |

**PRs:** Block completions only. Individual fixes go direct to main.

---

## Banned

- `--force` on main
- `--no-verify`
- PRs for individual fixes
- Pushing without local CI validation

**Full details:** `.claude/lazy/git.md`
