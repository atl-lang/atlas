# GATE -1: Sanity Check

**Purpose:** Full environment audit before any work begins. Lead Developer owns every decision — user is never asked about git state or workflow recovery.

---

## Step 0: Clean Build Artifacts (MANDATORY - FIRST ACTION)

> **Delegate to Haiku agent.** Return: disk usage before/after, any errors.

```bash
du -sh target/ 2>/dev/null || echo "No target directory"
cargo clean
```

**ONE TIME ONLY** at session start. Prevents 50GB+ accumulation.

---

## Step 1: Workspace State Audit

> **Delegate to Haiku agent.** Return: branch name, uncommitted file list, classification.

```bash
git status --short
git branch --show-current
git log main..HEAD --oneline
```

| State | Resolution |
|-------|-----------|
| Clean, on main | Proceed to Step 2 |
| Uncommitted changes | Inspect → commit WIP or `git restore .` for stale |
| Feature branch with unmerged commits | Resume if incomplete; push→PR if complete |
| Detached HEAD | `git checkout main`, reassess |

> `block/*` branches are NEVER deleted unless PR merged to main.

---

## Step 2: Local CI Health Check (BLOCKING)

> **Delegate to Haiku agent.** Return: build/test/clippy pass/fail.

```bash
cargo build --workspace
cargo nextest run --workspace
cargo clippy --workspace -- -D warnings
```

**If any fail:** Fix before any phase work. Broken build = 8-hour loops.

---

## Step 3: Security + Integrity (BLOCKING)

> **Delegate to Haiku agent.** See `gates/gate-minus1-checks.md` for full checklist.

Quick version:
```bash
cargo audit                                          # Direct dep vulns = STOP
git config core.hooksPath                            # Must be .githooks
wc -l /Users/proxikal/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/MEMORY.md  # Must be ≤ 55
```

**Full integrity checks (file existence, CI config, pre-push hook):** `gates/gate-minus1-checks.md`

---

## Step 4: Branch Setup

```bash
git checkout -b block/{name}   # One branch per block
```

If resuming existing block branch: skip, continue on existing branch.
See `git-workflow.md` for branch naming (`feat/`, `fix/`, `ci/`, `docs/`).

---

## Step 5: Phase Evaluation

1. Read phase blockers (`🚨 BLOCKERS` section)
2. Verify dependencies: spec → codebase → decide autonomously
3. Evaluate scope: version scope? Dependencies met? Parity impact?

---

## Decision Authority

- **Lead Developer decides autonomously:** git state, build failures, resume vs new branch
- **Architect informed, not consulted:** significant unexpected state → note once, handle it

**If concerns:** Present with evidence, act. Don't ask.
**If clean:** Proceed to GATE 0.
