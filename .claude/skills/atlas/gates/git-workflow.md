# Git Workflow

**Rule:** AI handles entire git lifecycle autonomously. User involvement: none.

## Branch Naming

```
phase/{category}-{number}   # e.g. phase/ownership-01
fix/{description}           # e.g. fix/parser-float
feat/{description}          # e.g. feat/array-slice
ci/{description}            # e.g. ci/optimize-workflows
```

## Start of Phase

```bash
git rebase main                          # sync home branch to local main
git checkout -b phase/{category}-{n}    # create feature branch
```

## During Phase (multi-part)

```bash
cargo build --workspace                  # must pass before committing
cargo nextest run -p atlas-runtime       # must be 100%
git add -A && git commit -m "feat(phase-XX): Part A — description"
```

## End of Phase

```bash
# 1. Final verification
cargo build --workspace
cargo nextest run -p atlas-runtime
cargo clippy -p atlas-runtime -- -D warnings
cargo fmt --check -p atlas-runtime

# 2. Commit
git add -A && git commit -m "feat(phase-XX): Description

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

# 3. Merge to local main
git checkout main
git merge --no-ff phase/{category}-{n} -m "feat(phase-XX): Description"
git branch -d phase/{category}-{n}

# 4. Sync ALL worktree home branches
git -C /Users/proxikal/dev/projects/atlas-dev rebase main
git -C /Users/proxikal/dev/projects/atlas-docs rebase main
```

## Push to GitHub

Only when user says "push to GitHub":
```bash
git push origin main   # from main worktree only
```

## Banned

- PRs for normal phase/doc work
- Pushing on every phase
- Working directly on `main`
- Leaving uncommitted changes at session end
- `git checkout` a branch that lives in another worktree (use `git -C` instead)
