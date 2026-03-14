# GATE -1 — Integrity & Security Checks (Lazy-Loaded)

**Load when:** Called from GATE -1 Step 3

---

## Security Scan

```bash
cargo audit
```

- Vulnerabilities in **direct deps** → STOP, fix or escalate
- Vulnerabilities in **transitive deps only** → note and continue

---

## Integrity Spot-Check

> **Delegate to Haiku agent.** Return: pass/fail for each check.

```bash
# 1. MEMORY.md line count — must stay ≤ 55
wc -l /Users/proxikal/.claude/projects/-Users-proxikal-dev-projects-atlas/memory/MEMORY.md

# 2. All lazy-load and rule files must exist
ls .claude/lazy/git.md .claude/lazy/comms.md .claude/lazy/architecture.md \
   .claude/rules/atlas-ci.md .claude/rules/atlas-testing.md \
   .claude/rules/atlas-parity.md .claude/rules/atlas-syntax.md \
   .claude/rules/atlas-ast.md .claude/rules/atlas-typechecker.md \
   .claude/rules/atlas-vm.md 2>&1 | grep "No such file"

# 3. CI test gate — must include pull_request
grep -c "pull_request" .github/workflows/ci.yml

# 4. .claude/ excluded from code detection
grep -c "\.claude" .github/workflows/ci.yml

# 5. actionlint present
grep -c "actionlint" .github/workflows/ci.yml

# 6. Pre-push hook active
git config core.hooksPath
```

| Check | Pass | Fail → Action |
|-------|------|---------------|
| MEMORY.md ≤ 55 lines | ≤ 55 | **BLOCKING** — split/archive |
| Rule files exist | no output | **BLOCKING** — create or restore |
| pull_request ≥ 1 | ≥ 1 | CI drifted — direct push fix |
| .claude excluded ≥ 1 | ≥ 1 | Path filter gap — direct push fix |
| actionlint ≥ 1 | ≥ 1 | Workflow protection drifted |
| hooksPath = .githooks | `.githooks` | **BLOCKING** — `git config core.hooksPath .githooks` |

---

## Pre-PR Path Filter Verification

Before opening any PR, verify changed file types are covered by CI path exclusions:

```bash
git diff --name-only main...HEAD | sed 's/.*\.//' | sort -u
grep -A20 "paths:" .github/workflows/ci.yml | grep "^\s*- '!"
```

Every extension you changed must be in the exclusion list if docs-only.

---

## Required Tools (install autonomously)

```bash
which actionlint    || brew install actionlint
which cargo-audit   || cargo install cargo-audit --locked
which cargo-nextest || cargo install cargo-nextest --locked
```

---

## Pre-push Hook (BLOCKING if missing)

```bash
git config core.hooksPath    # must output: .githooks
ls .githooks/pre-push        # must exist
```

If missing:
```bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-push
```
