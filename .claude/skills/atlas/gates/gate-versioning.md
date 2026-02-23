# GATE V: Versioning

**Purpose:** Decide autonomously when to advance the version and tag a release.
No user input required. The rules are the intelligence.

---

## The Versioning Model (Zig-inspired, Atlas-adapted)

Atlas uses **capability milestones**, not development velocity, to drive version numbers.

**Rule:** A version tag represents something an observer outside the project could meaningfully
distinguish. Blocks and phases are internal development tracking. They do not auto-trigger tags.

**Why not "every fix/ PR gets a patch tag":**
Rust bumps version ~quarterly. Zig takes months per minor. Atlas at AI pace (3 blocks/week)
would hit `v0.10+` before a human ever ran the language. The version number would be meaningless.
Patch tags only make sense when a named version is already published and a confirmed bug was
found in it — not for every internal fix/ PR.

---

## Version-to-Milestone Map (THE AUTHORITATIVE TABLE)

When an agent runs GATE V, this table is the decision. No other interpretation.

| Tag | Capability Milestone | Blocks Completed | Status |
|-----|----------------------|-----------------|--------|
| `v0.1.0` | Working interpreter + VM + CLI + stdlib foundation | Pre-Block-1 baseline | ✅ Tagged |
| `v0.2.0` | CoW memory model + ownership syntax + trait system | Blocks 1–3 | ✅ Tagged |
| `v0.3.0` | Closures + type inference + error handling + JIT + async + quick wins | Blocks 4–9 (all v0.3 blocks) | ⬜ In progress |
| `v0.4.0` | Compile-time ownership verification + trait objects + user generics | v0.4 plan blocks | ⬜ Future |
| `v0.5.0` | AOT native compilation (Cranelift backend) | v0.5 plan blocks | ⬜ Future |
| `v1.0.0` | Stability commitment: syntax locked, stdlib stable, security audited | All v0.9 stabilization | ⬜ Future |

**Update this table** when a new version plan is scaffolded. The table is the contract.

---

## When This Gate Runs

| Event | Action |
|-------|--------|
| Block AC check phase committed | Run minor version check (Step 1 below) |
| Bug confirmed in a *tagged* version | Run patch version check (Step 2 below) |
| All blocks in a version plan complete | Run minor version advance (Step 1, then tag) |

**Does NOT run:**
- Every `fix/` PR merged (unless the fix is for a confirmed bug in a tagged version)
- Every phase commit
- Every block completion (only on the *final block* of a version plan)

---

## Step 1: Minor Version Check (after final block of a version plan)

### 1a. Confirm which version plan just completed

```bash
ls docs/internal/V*_PLAN.md | sort | tail -1
```

### 1b. Read the Exit Criteria section — verify every item

**Never assume. Always verify against codebase.**

| Criterion type | How to verify |
|----------------|---------------|
| Block N complete | STATUS.md shows ✅ AND final AC phase committed |
| Test count ≥ X | `cargo nextest run --workspace 2>&1 \| grep "tests run"` |
| No Arc<Mutex<Value>> | `grep -r "Arc<Mutex<" crates/ --include="*.rs" \| grep -v test \| grep -v "//"` |
| Spec updated | Grep for expected sections in `docs/specification/` |
| Clippy/fmt clean | CI green on latest main commit |

### 1c. Decision

- **ALL criteria ✅** → Advance version (Step 1d)
- **ANY criteria ❌** → Do NOT advance. Log blockers in STATUS.md under "Version Gate Blockers". Continue building.

### 1d. On minor version advance

```bash
# 1. Determine new version from the table above
# 2. Update workspace version
# Cargo.toml: version = "X.Y.0"

# 3. Tag and push
git tag vX.Y.0
git push origin vX.Y.0

# 4. Update STATUS.md version field
# 5. Update ROADMAP.md: mark vX.Y.Z complete in the table above, set next version ⬜ → 🔨
# 6. Update decisions/workflow.md: log the version advance with DR-VXX
# 7. Commit STATUS.md + ROADMAP.md on main via direct push (Track 1)
```

---

## Step 2: Patch Version Check (bug in a tagged version)

**Trigger:** A `fix/` PR that corrects behavior that was wrong in a *currently tagged release.*

### Is this a patch-worthy fix?

Ask: "Was this bug present in `v0.X.Y`?" (the last tag)

```bash
git tag --sort=-version:refname | head -3   # see tagged versions
git log vX.Y.Z..HEAD --oneline              # commits since last tag
```

- Bug existed in tagged version AND the fix changes observable behavior → **patch tag**
- Bug is new (introduced after last tag, never in a tagged version) → **no tag, just commit**
- Fix is CI, docs, workflow, or tooling → **no tag ever**

### On patch version advance

```bash
# Increment patch only: v0.2.0 → v0.2.1
# Update workspace version in Cargo.toml
git tag vX.Y.Z+1
git push origin vX.Y.Z+1
# Update STATUS.md version field
# Commit via Track 1 direct push
```

---

## Rules (Non-Negotiable)

- **Never ask the user.** The table and checklist are the decision.
- **Never advance on block count alone.** Exit criteria must ALL verify against actual codebase.
- **Tags are permanent.** Never re-tag. Verify before pushing.
- **No version exists without a tag.** STATUS.md version = last git tag. If they diverge, STATUS.md is wrong.
- **Version table is the contract.** If the table says `v0.3.0` requires Blocks 4–9, then `v0.3.0` does not tag until all 9 blocks are complete. No exceptions.
- **Workspace version must match the last tag.** After every version advance, Cargo.toml workspace version is updated to match. All crates using `version.workspace = true` stay in sync automatically.

---

## Current State (Update on every version advance)

**Last tag:** `v0.2.0` (2026-02-23)
**Workspace version:** `0.2.0`
**Next tag:** `v0.3.0` — requires all 9 v0.3 blocks complete + exit criteria verified
**Blocks remaining:** 4, 5, 6, 7, 8, 9 (Block 4 in progress)
