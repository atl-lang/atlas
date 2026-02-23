# GATE -1: Sanity Check

**Purpose:** Full environment audit before any work begins. The Lead Developer owns every decision here — the user is never asked about git state, branch state, or workflow recovery.

---

## Step 1: Workspace State Audit (ALWAYS FIRST)

```bash
git status --short                        # Uncommitted changes?
git branch --show-current                 # Which branch?
git log main..HEAD --oneline              # Commits not yet on main?
```

Classify and resolve autonomously:

### State: Clean, on main
→ Normal. Proceed to Step 2.

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

```bash
cargo build --workspace
```

**BLOCKING.** If this fails, fix it before starting new work.

---

## Step 4: Security Scan

```bash
cargo audit
```

→ Vulnerabilities in **direct deps** → STOP, fix or escalate
→ Vulnerabilities in **transitive deps only** → note and continue

---

## Step 4b: Integrity Spot-Check (BLOCKING)

Runs every session. Catches drift, broken references, and missing files before they compound.

```bash
# 1. MEMORY.md line count — must stay ≤ 50
wc -l ~/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/MEMORY.md

# 2. All always-on rule files referenced in MEMORY.md must exist on disk
ls .claude/rules/atlas-git.md .claude/rules/atlas-comms.md \
   .claude/rules/atlas-architecture.md .claude/rules/atlas-ci.md \
   .claude/rules/atlas-testing.md .claude/rules/atlas-parity.md \
   .claude/rules/atlas-syntax.md .claude/rules/atlas-ast.md \
   .claude/rules/atlas-typechecker.md .claude/rules/atlas-interpreter.md \
   .claude/rules/atlas-vm.md 2>&1 | grep "No such file"

# 3. CI test gate — must include pull_request
grep -c "pull_request" .github/workflows/ci.yml

# 4. CI path filter — .claude/ must be excluded from code detection
grep -c "\.claude" .github/workflows/ci.yml

# 5. actionlint present — workflow protection layer
grep -c "actionlint" .github/workflows/ci.yml
```

| Check | Pass | Fail → Action |
|-------|------|---------------|
| MEMORY.md ≤ 50 lines | number ≤ 50 | **BLOCKING** — split/archive before any work |
| Rule files exist | no output | **BLOCKING** — missing file = broken governance; create or restore before proceeding |
| pull_request ≥ 1 | count ≥ 1 | CI drifted — open `ci/` fix branch before phase work |
| .claude excluded ≥ 1 | count ≥ 1 | Path filter gap — open `ci/` fix before any `.claude/` PR |
| actionlint ≥ 1 | count ≥ 1 | Workflow protection drifted — open `ci/` fix branch |

**Cost: 5 tool calls, ~0 context. Every check is BLOCKING if it fails.**

### Required Tools

These must be present. Install autonomously — never ask the user.

```bash
which actionlint  || brew install actionlint
which cargo-audit || cargo install cargo-audit --locked
which cargo-nextest || cargo install cargo-nextest --locked
```

### If you modified a `.github/workflows/` file this session

Run actionlint locally to verify YAML validity before committing:
```bash
# Install once: brew install actionlint  (or go install github.com/rhysd/actionlint/cmd/actionlint@latest)
actionlint .github/workflows/ci.yml
actionlint .github/workflows/bench.yml
```

Also apply Layer 2 + Layer 3 from `atlas-ci.md` — read it if you haven't.

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
