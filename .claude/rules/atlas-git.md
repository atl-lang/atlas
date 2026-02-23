---
paths:
  - "**"
---

# Atlas Git Workflow

**Single workspace:** `~/dev/projects/atlas/` on `main`. No other worktrees.

## GitHub Branch Protection (main)

- **PRs required** — direct push to `main` is rejected
- **No merge commits** — linear history only (squash)
- **CI gate** — "CI Success" check must pass
- **Auto-merge** — use `gh pr merge --auto --squash`; merges when CI passes

## PR Workflow

```bash
# 1. Start from clean main
git checkout main && git pull origin main
git checkout -b feat/short-description

# 2. Do work, commit
git add <files> && git commit -m "feat: description"

# 3. Rebase on latest main BEFORE push (strict CI policy requires up-to-date branch)
git fetch origin && git rebase origin/main

# 4. Push + PR
git push -u origin feat/short-description
gh pr create --title "title" --body "body"
gh pr merge --auto --squash

# 5. After merge: sync and clean up
git checkout main && git pull origin main
git branch -d feat/short-description
```

> **Why:** `strict_required_status_checks_policy=true` means CI won't run (and auto-merge stalls) if any commit landed on main after the branch was created. Always rebase immediately before push.

## Branch Naming

```
block/{name}                # e.g. block/trait-system — ONE branch per block
feat/{short-description}    # standalone features outside the block plan
fix/{short-description}     # blocking bug fixes (may PR immediately)
ci/{short-description}      # CI/infra changes
docs/{short-description}    # docs-only changes
```

## Commit Cadence — ONE PR PER BLOCK

All scaffold commits, phase execution commits, and spec/STATUS updates for a block
live on the **same branch** (`block/{name}`). The PR is opened only when the block's
final AC check phase is complete.

```
block/trait-system branch:
  scaffold commit
  phase-01 commit
  phase-02 commit
  ...
  phase-18 commit (spec + AC check)
  ← PR opened here, auto-merged
```

**Exception:** Blocking fixes or critical CI changes may PR immediately on a `fix/`
or `ci/` branch. These are the ONLY valid reasons to PR before block completion.

## CI Push Discipline

**Every force-push resets CI from scratch.** Each push cancels the running CI and starts over. On slow runners (windows, tarpaulin) that's 10+ wasted minutes per push.

**Rule: batch all fixes before pushing.** When CodeRabbit or CI leaves feedback:
1. Read ALL pending review comments first
2. Fix everything in one commit
3. Push once

Never push to address one comment while others are pending.

## Branch Hygiene (MANDATORY)

Stale branches are a safety hazard — they cause confusion, conflicts, and accidental overwrites.

### Rules
- **At most 3 remote branches:** `main` + `gh-pages` (permanent) + 1 active branch (block/fix/ci/docs)
- **Never leave a PR open and unattended.** If a `fix/` or `ci/` branch is opened, it must merge or be closed before the next session ends
- **Prune after every merge:** `git remote prune origin` after any PR merges to sync local refs
- **No orphan branches.** A branch with no open PR and no active work must be deleted immediately

### Session-start audit (GATE -1)
Run at the start of every session:
```bash
git branch -r | grep -v "HEAD\|dependabot"   # should show: origin/main + origin/gh-pages + origin/<active-branch>
gh pr list                                      # should show 0 or 1 open PR
git remote prune origin                         # prune any stale tracking refs
```

If more than 1 PR is open or more than 2 remote branches exist → **stop and audit before doing any work.**

### When a `fix/` or `ci/` branch is needed
1. Create it, do the work, push, PR, set auto-merge
2. **Do not switch back to the block branch until CI passes and PR merges**
3. After merge: `git checkout block/<name> && git remote prune origin`

## Banned

- `git push origin main` directly
- Merge commits (`--no-ff`)
<<<<<<< HEAD
- `--force` on main (use `--force-with-lease` only when rebasing your own branch)
- `--no-verify`
- Force-pushing to address review comments one at a time
- Leaving branches open across sessions without a tracking note in STATUS.md
