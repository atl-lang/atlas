---
paths:
  - "**"
---

# Atlas Git Workflow

**Single workspace:** `~/dev/projects/atlas/` on `main`. No other worktrees.
**100% AI maintained.** proxikal handles GitHub UI only (rulesets, secrets, billing).

---

## Two-Track Push Policy

### Track 1 — Direct push to main (no PR, no CI wait)

Use for ANY change that does not touch Rust source, AND for pure refactors/splits of Rust test files where behavior is unchanged and locally verified:

| Change type | Examples |
|-------------|---------|
| CI/workflows | `.github/workflows/*.yml` |
| AI workflow | `.claude/**`, `phases/**` |
| Config | `.coderabbit.yaml`, `deny.toml`, `rust-toolchain.toml` |
| Docs | `docs/**`, `**.md`, `STATUS.md`, `ROADMAP.md` |
| Cargo metadata only | Version bumps, `[package]` fields, no new deps |
| Pure Rust refactors | File splits, renames, moves — zero logic change, full local test suite green (`cargo nextest run -p <crate>`) |

```bash
git add <files> && git commit -m "docs(spec): update closure syntax examples [skip ci]" && git push origin main
```

**`[skip ci]` is MANDATORY on every direct push** — appended to a normal conventional commit message. The message describes the change fully. `[skip ci]` is just a trailer. GitHub skips ALL workflows. No branch. No PR. No waiting. Ever.

```
# Correct — descriptive message, [skip ci] as trailer
docs(runtime): update test table line counts [skip ci]
ci(coderabbit): disable blocking reviews [skip ci]
chore(status): mark phase-05 complete [skip ci]

# Wrong — message is not the trailer
[skip ci]
skip ci
ci: skip
```

### Track 2 — PR + CI required (no exceptions)

Use for ANY change touching Rust source:

- `crates/**/*.rs`
- `crates/**/Cargo.toml` when adding/changing dependencies

```bash
git checkout block/{name}   # already on block branch
# ... implement ...
git add crates/ && git commit -m "feat: ..."
# PR opens at block completion, not per-phase
```

---

## Emergency Bypass (Rust source — rare, strict criteria)

Force-pushing Rust source bypasses CI. Only propose this when ALL of the following are true:

1. **CI failure is infrastructure, not code** — flaky runner, GitHub outage, unrelated test rot
2. **The change is trivially safe** — typo fix, comment, unreachable dead code removal
3. **Blocked > 30 minutes** with no CI fix in sight
4. **Explicitly flag it:** "This qualifies for emergency bypass because [reason]. Confirm?"

**Never propose bypass because:**
- CI is slow
- The change "seems obviously correct"
- We've been waiting a while
- It's just a one-liner

When in doubt: wait for CI. The bar is high intentionally.

---

## Block Branch Workflow

All phase commits live on `block/{name}`. PR opens only at block completion (final AC check phase).

```
block/closures:
  scaffold commit
  phase-01 commit → phase-12 commit
  ← PR opened here, CI runs once, auto-squash merges
```

**CRITICAL:** `fix/`, `ci/`, `docs/` branches MUST be created from `main`, never from `block/`.
For non-code fixes mid-block: don't branch at all — direct push to main (Track 1).

---

## Branch Hygiene

- **At most 3 remote branches:** `main` + `gh-pages` + 1 active `block/` branch
- Track 1 changes never create branches
- After every PR merge: `git remote prune origin`

### Session-start audit (GATE -1)
```bash
git branch -r | grep -v "HEAD\|dependabot"   # main + gh-pages [+ block/name]
gh pr list                                     # 0 or 1 open PR
git remote prune origin
```

---

## Banned

- PRs for Track 1 changes — direct push instead, always
- Merge commits (`--no-ff`)
- `--force` without `--force-with-lease`
- `--no-verify`
- Branching `fix/`/`ci/`/`docs/` off `block/` branches
- Proposing emergency bypass without meeting all 4 criteria above
