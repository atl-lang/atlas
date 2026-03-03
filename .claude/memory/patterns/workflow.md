# Workflow Architectural Patterns

Decisions about the AI development workflow, gate chain, governance, and CI.

---

## P-W04: No Inline Tests in Source Files (2026-02-23)

**Decision:** `#[cfg(test)]` blocks are banned from `src/` files. All tests live in `crates/atlas-runtime/tests/`.

**Rationale:** Inline tests inflate source file line counts and force agents to load test code when working on implementation. `vm/mod.rs` has ~1,400 lines of inline tests contributing to its 4,393-line total. The separation is not just style — it's a token cost that compounds every session.

**Migration:** Existing inline tests are noted as cleanup items, not mid-phase refactors.

---

## P-W05: CI Path Filter Must Cover All Changed File Types (2026-02-23)

**Decision:** Before opening any PR, verify that the CI `paths-filter` exclusion list covers the file types being changed. A docs-only PR that triggers Rust CI is a waste of runner time and merge freeze budget.

**Background:** PR #146 was opened with `.claude/` changes but `.claude/**` was not in the path filter exclusions. The full Rust matrix would have run. PR was closed, CI fix (`!.claude/**`) landed as PR #147 first.

**Enforced at:** GATE -1 spot-check should verify path filter covers active change types. Currently partial — GATE -1 checks CI structure drift but not per-PR path coverage. Full enforcement is a future hardening item.

---

## P-W06: ROADMAP.md Required at Scaffolding Time (2026-02-23)

**Decision:** Any scaffolding session (Scaffold Block N) must read `ROADMAP.md` before producing the block kickoff doc.

**Rationale:** Gate 0 selective reading table had no entry for scaffolding. An agent scaffolding Block 7 (JIT) with no awareness of the long-term trajectory (systems language, no GC, v0.3 is the permanent foundation) could make block-level decisions that conflict with the 5-year direction.

**Status:** Pending — Gate 0 scaffolding path needs one line added. Tracked as workflow hardening item.

---

## P-W08: Side Branches Must Always Fork from main (2026-02-23)

**Decision:** `fix/`, `ci/`, and `docs/` branches MUST be created from `main`, never from `block/*`.

**Rationale:** Creating a side branch from `block/closures` carries all block commits into the
rebase. When the side branch is deleted (e.g. closed PR, redo), any files that only existed on
that branch are permanently lost. This caused `atlas-architecture.md` to be created, referenced
in MEMORY.md, and then silently deleted when PR #146 was closed — leaving a broken always-on
rule reference. The GATE -1 existence check catches the symptom; this rule prevents the cause.

**Enforced at:** `atlas-git.md` commit cadence section.

---

## P-W06: Versioning Model — Capability Milestones, Not Development Velocity (2026-02-23)

**Decision:** Atlas uses capability milestones (Zig-inspired) for version tagging. Version numbers do not track commit count, block count, or `fix/` PR merges. A version tag represents a meaningful architectural capability that an external observer could distinguish.

**Rationale:** Atlas builds at AI pace (~3 blocks/week). Auto-tagging every fix/ PR or block completion would reach `v0.10+` before any human used the language, making version numbers meaningless. Rust takes 3-4 months per minor; Zig is still at 0.x after 9 years. Atlas must pace version numbers to capability, not velocity.

**Authoritative table:** `gates/gate-versioning.md` — the version-to-block map is the contract. Every agent runs GATE V from that table. Do not deviate from it.

**Patch tags:** Only for bugs confirmed present in an already-tagged version. A `fix/` PR on in-development code is not a patch release.

**Workspace version:** Always matches the last tag. All crates use `version.workspace = true`. `atlas-build` and `atlas-config` were converted to workspace version on 2026-02-23.

**Enforced at:** GATE V (`gates/gate-versioning.md`), skill.md GATE V bullets, ROADMAP.md version table.

---

## P-W07: PRD Must Reflect Current Atlas Identity (2026-02-23)

**Decision:** `docs/internal/PRD.md` must be updated to reflect Atlas as a systems language, not "REPL-first scripting language."

**Background:** PRD line 10 says "REPL-first programming language." ROADMAP says systems language replacing C/Rust/Zig. Every agent that loads the PRD gets the wrong mental model, which affects diagnostic message quality, API design decisions, and scope judgment.

**Status:** Pending — PRD update scheduled as standalone docs session.

---

## P-W08: No Test-to-Production Ratio Tracking (2026-02-23)

**Decision:** Do NOT add lines-of-test / lines-of-production ratio to STATUS.md or any gate check.

**Rationale:** Line counts are noisy signal. Coverage floors (Codecov, per-crate floors in `atlas-ci.md`) already catch "production code without tests" directly and more accurately. A ratio metric adds busywork without catching the underlying failure mode better than the existing tools.

**What covers this instead:** Per-crate Codecov floors enforced at merge time. The "no new test files" rule limits test bloat. These are sufficient.
