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

## Banned

- `git push origin main` directly
- Merge commits (`--no-ff`)
- `--force` on main (use `--force-with-lease` only when rebasing)
- `--no-verify`
- Force-pushing to address review comments one at a time
