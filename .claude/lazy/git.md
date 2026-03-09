# Atlas Git Workflow

**Single workspace:** `~/dev/projects/atlas/`
**100% AI maintained.** proxikal handles GitHub UI only (rulesets, secrets, billing).

---

## Branch Policy (MANDATORY — enforced by hook)

**main is protected.** Only docs, config, CI, and AI workflow files may be committed directly to main.

| Work type | Branch name | Example |
|-----------|-------------|---------|
| Bug fix | `fix/H-XXX` | `fix/H-178` |
| Block phase | `block/B-XX-name` | `block/B14-error-quality` |
| Feature | `feat/name` | `feat/impl-blocks` |
| Refactor | `refactor/name` | `refactor/parser-errors` |

**Allowed directly on main (no branch):**
- `docs/**`, `**.md` files
- `.claude/**` (AI workflow, skills, hooks, memory)
- `.github/**`, CI config
- `Cargo.toml` package metadata only (not dep changes)

**Rust source (`.rs`) and Cargo.toml dep changes MUST be on a branch.**
The hook `enforce-branch-policy.sh` will block the commit if you try.
Architect override only: `touch /tmp/atlas-branch-unlock`

---

## Starting Work — Always Branch First

```bash
# Bug fix
git checkout -b fix/H-XXX

# Block (create at scaffold time, per atlas-blocks skill)
git checkout -b block/B-XX-name

# Feature
git checkout -b feat/short-name
```

**Check you are NOT on main before writing any Rust code:**
```bash
git branch --show-current   # must NOT be "main"
```

---

## During Development (on feature branch)

```bash
# After every meaningful change
cargo fmt --check && cargo clippy --workspace -- -D warnings
git add crates/ && git commit -m "fix(vm): description"
# DO NOT PUSH YET — local-first policy
```

---

## Merging Back to Main

When the work is complete and all gates passed:

```bash
# 1. Make sure main is up to date
git fetch origin
git checkout main
git pull origin main

# 2. Merge with a merge commit (preserves branch history)
git merge --no-ff fix/H-XXX -m "merge: fix/H-XXX into main"

# 3. Delete the branch
git branch -d fix/H-XXX

# 4. Do NOT push yet — batch push policy (see below)
```

**For block branches** — merge only at block completion after all phases done and AC verified:
```bash
git merge --no-ff block/B-XX-name -m "feat(B-XX): merge block/B-XX-name — <what shipped>"
git branch -d block/B-XX-name
```

---

## Stale Branch Protocol (AI responsibility)

Run this audit at session start if `pt go` shows unmerged branches:

```bash
# List branches with unmerged commits
git branch --no-merged main

# For each stale branch, assess:
git log main..fix/H-XXX --oneline   # commits not yet in main
git diff main...fix/H-XXX --stat    # files changed

# Decision tree:
# 1. Has unmerged commits AND work is complete → merge to main (see above)
# 2. Has unmerged commits AND work is in progress → leave it, note in handoff
# 3. Has NO unmerged commits (already merged) → safe to delete: git branch -d <name>
# 4. Has commits that conflict with main direction (abandoned) → ask architect before deleting
```

**Never delete a branch with unmerged commits without architect approval.**
The Stop hook warns about unmerged branches — treat it as a blocker to resolve before session end.

---

## Batch Push Workflow

**Check if due:** `git fetch origin && git log origin/main -1 --format="%ci"` — push if 168+ hours ago

```bash
# 1. Full local CI
cargo fmt --check && cargo clippy --workspace -- -D warnings
cargo nextest run --workspace

# 2. If all pass
git push origin main

# 3. Update tracking
pt mark-ci-pass "local CI: fmt+clippy+nextest"
```

**No PRs for routine fixes.** Direct push to main after local CI passes.

---

## PR Workflow (Major Block Completions Only)

PRs are for external visibility on major blocks, not routine work.

```bash
git push -u origin block/B-XX-name
gh pr create --title "feat(B-XX): Block name — what shipped" --body "..."
gh pr merge --auto --squash
git branch -d block/B-XX-name
```

---

## Quick Checks (During Development)

```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings
# During development: targeted tests only (no nextest --workspace)
cargo nextest run -p atlas-runtime -E 'test(exact_test_name)'
```

---

## Banned

- Committing `.rs` files directly to main — hook blocks this, architect override only
- PRs for individual fixes — batch push instead
- Pushing without local CI validation
- `--force` without `--force-with-lease`
- `--no-verify`
- Deleting branches with unmerged commits without architect approval
- `git branch -D` (force delete) — banned by block-destructive-git.sh hook
