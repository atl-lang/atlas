# GATE -1: Sanity Check

**Purpose:** Full environment audit before any work begins. The Lead Developer owns every decision here — the user is never asked about git state, branch state, or workflow recovery.

---

## Step 0: Clean Build Artifacts (MANDATORY - FIRST ACTION)

> **Delegate to Haiku agent.** Return: disk usage before/after, any errors.

**Prevent disk bloat:** Cargo accumulates GB of build artifacts rapidly. Without cleanup between sessions, `target/` can reach 50GB+.

```bash
du -sh target/ 2>/dev/null || echo "No target directory"
cargo clean
```

**Why:** Fresh build environment at session start prevents accumulation. Especially critical when rules weren't auto-loading correctly (agents were doing more rebuilds than necessary).

**ONE TIME ONLY:** Run once at session start, not during implementation.

---

## Step 1: Workspace State Audit

> **Delegate to Haiku agent.** Return: branch name, uncommitted file list, classification.

```bash
git status --short                        # Uncommitted changes?
git branch --show-current                 # Which branch?
git log main..HEAD --oneline              # Commits not yet on main?
```

Classify and resolve autonomously:

### State: Clean, on main
→ Normal. Proceed to Step 2.

> **Branch cleanup rule:** When deleting stale branches, `block/*` branches are NEVER deleted
> unless their PR has merged to main. `ci/`, `fix/`, `docs/` branches are safe to delete after merge.
> Check `git log --all --oneline | grep block/` before any cleanup.

### State: Uncommitted changes present
→ Inspect every changed file: `git diff` + `git status`
→ **Valid WIP:** stage and commit before starting new work
→ **Stale/accidental:** `git restore .` to discard
→ Rule: changes relate to current task → commit. Unknown → inspect carefully.

### State: On a feature branch with unmerged commits
→ `git log main..HEAD --oneline` + `git diff main`
→ **Work complete** (build + tests pass): push → PR → auto-merge, then continue
→ **Work incomplete:** this is the resumption point — continue here, don't create new branch

### State: Detached HEAD
→ `git checkout main` to return, then reassess.

---

## Step 2: Main CI Health Check (BLOCKING)

> **Delegate to Haiku agent.** Return: last 3 run conclusions, any failing job names.

```bash
gh run list --branch main --limit 3 --json status,conclusion,displayTitle,databaseId \
  -q '.[] | "\(.conclusion // .status)  \(.displayTitle[:60])"'
```

**If the last completed main push CI run shows `failure`:**
→ This is the first priority of the session — fix it before any phase work
→ Check what failed: `gh run view <id> --json jobs -q '.jobs[] | "\(.conclusion)  \(.name)"'`
→ Open a `fix/` or `ci/` branch, fix, PR, wait for green, then proceed

**If runs are `in_progress`:**
→ Note it. If they're coverage runs from a previous session, wait for completion before opening new PRs (merge freeze rule).

**If last completed run is `success`:** proceed.

**Why this is blocking:** Main CI failing silently between sessions is how 8-hour loops happen. Every session starts with a known-green main.

---

## Step 2b: Open PRs Check

```bash
gh pr list --state open
```

→ Open PRs + in-progress CI = **merge freeze in effect** — do not open new PRs until clear
→ Stale PRs with no CI running: rebase and push to restart CI

---

## Step 3: Sync from Remote

> **Delegate to Haiku agent.** Return: sync action taken (pull/rebase/none).

```bash
git fetch origin
git log HEAD..origin/main --oneline       # Is remote ahead of local main?
git log origin/main..HEAD --oneline       # Commits on this branch not yet in main
git log HEAD..origin/$(git branch --show-current) --oneline  # Is remote branch ahead?
```

→ **On main, remote ahead** (PR merged): `git pull origin main`
→ **On block branch, origin/main has new commits:** `git rebase origin/main` — keep the branch current
→ **Remote branch ahead of local** (another session pushed): `git pull --rebase`
→ **All equal:** nothing to do

**Why this matters:** `strict_required_status_checks_policy=true` means auto-merge stalls
if main advanced while the block branch was in progress. Catch it here, not at PR time.

---

## Step 3: Full Build Verification

> **Delegate to Haiku agent.** Return: build success/failure and any error lines.

```bash
cargo build --workspace
```

**BLOCKING.** If this fails, fix it before starting new work.

---

## Step 4: Security Scan

> **Delegate to Haiku agent.** Return: audit findings summary (direct deps only).

```bash
cargo audit
```

→ Vulnerabilities in **direct deps** → STOP, fix or escalate
→ Vulnerabilities in **transitive deps only** → note and continue

---

## Step 4b: Integrity Spot-Check (BLOCKING)

> **Delegate to Haiku agent.** Return: pass/fail for each check in the table below.

Runs every session. Catches drift, broken references, and missing files before they compound.

```bash
# 1. MEMORY.md line count — must stay ≤ 55
wc -l .claude/memory/MEMORY.md

# 2. All lazy-load and pattern-triggered rule files must exist on disk
ls .claude/lazy/git.md .claude/lazy/comms.md .claude/lazy/architecture.md \
   .claude/rules/atlas-ci.md .claude/rules/atlas-testing.md \
   .claude/rules/atlas-parity.md .claude/rules/atlas-syntax.md \
   .claude/rules/atlas-ast.md .claude/rules/atlas-typechecker.md \
   .claude/rules/atlas-interpreter.md .claude/rules/atlas-vm.md 2>&1 | grep "No such file"

# 3. CI test gate — must include pull_request
grep -c "pull_request" .github/workflows/ci.yml

# 4. CI path filter — .claude/ must be excluded from code detection
grep -c "\.claude" .github/workflows/ci.yml

# 5. actionlint present — workflow protection layer
grep -c "actionlint" .github/workflows/ci.yml

# 6. Pre-push hook active
git config core.hooksPath
```

### Step 4c: Pre-PR Path Filter Verification (before opening any PR)

Before opening a PR, verify the changed file types are covered by the CI path exclusion list.

```bash
# List extensions of files changed vs main
git diff --name-only main...HEAD | sed 's/.*\.//' | sort -u

# Compare against exclusions in ci.yml
grep -A20 "paths:" .github/workflows/ci.yml | grep "^\s*- '!"
```

**Rule:** Every extension/directory you changed must appear in the exclusion list (prefixed with `!`) if it should be treated as docs-only. If a new file type would trigger Rust CI unexpectedly, add it to the exclusion list before opening the PR.

**Example gap that bit us:** `.claude/**` changes were not in the exclusion list — full Rust matrix would have run on a pure workflow PR (DR-W05). The check here catches this class of bug at PR open time, not post-merge.

| Check | Pass | Fail → Action |
|-------|------|---------------|
| MEMORY.md ≤ 50 lines | number ≤ 50 | **BLOCKING** — split/archive before any work |
| Rule files exist | no output | **BLOCKING** — missing file = broken governance; create or restore before proceeding |
| pull_request ≥ 1 | count ≥ 1 | CI drifted — direct push fix to main |
| .claude excluded ≥ 1 | count ≥ 1 | Path filter gap — direct push fix to main |
| actionlint ≥ 1 | count ≥ 1 | Workflow protection drifted — direct push fix to main |
| hooksPath = .githooks | `.githooks` | **BLOCKING** — `git config core.hooksPath .githooks` before any work |

**Cost: 5 tool calls, ~0 context. Every check is BLOCKING if it fails.**

### Required Tools

These must be present. Install autonomously — never ask the user.

```bash
which actionlint    || brew install actionlint
which cargo-audit   || cargo install cargo-audit --locked
which cargo-nextest || cargo install cargo-nextest --locked
```

### Pre-push Hook (BLOCKING if missing)

The pre-push hook in `.githooks/pre-push` runs automatically on every push and catches:
- actionlint errors on workflow file changes
- cargo fmt failures on Rust changes
- cargo check failures on Cargo.toml/deny.toml changes

Verify it is active:
```bash
git config core.hooksPath    # must output: .githooks
ls .githooks/pre-push        # must exist
```

If missing or not configured — fix before doing any work:
```bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-push
```

This is why workflow errors are caught in seconds locally, not after a full CI run on GitHub.

---

## Step 5: Branch Setup

```bash
git checkout -b block/{name}   # One branch per block — all phases committed here
```

If resuming an existing block branch (Step 1 State 2): skip, continue on existing branch.
See `git-workflow.md` for full branch naming convention (`feat/`, `fix/`, `ci/`, `docs/`).

---

## Step 6: Phase Evaluation

1. **Read phase blockers:** Check `🚨 BLOCKERS` section in phase file
2. **Verify dependencies:** Check spec → check codebase → decide autonomously
3. **Evaluate scope:** Version scope? Dependencies met? Parity impact?

---

## Decision Authority

**Lead Developer decides autonomously:**
- All git state resolution
- All build failures (fix them)
- Resume vs new branch decisions

**Architect is informed, not consulted:**
- Significant unexpected state → note once, handle it. Never block on user response.

---

**If concerns found:** Present with evidence, act. Don't ask.
**If no concerns:** Proceed to GATE 0.
