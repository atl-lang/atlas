# Archived Workflow Decisions (2026-02-23)

Archived from decisions/workflow.md — stable, fully encoded in rule files.

---

## DR-W01: File Size Limits (2026-02-23)

**Decision:** Source files hard cap 2,000 lines (warn at 1,500). Test files hard cap 4,000 lines (warn at 3,000).

**Rationale:** At Atlas token burn rate (~500M/day), unbounded file growth compounds every session. AI agents on cold start have no mental model of a 14k-line file — they miss existing tests, duplicate logic, and make inconsistent decisions.

**ARCH-EXCEPTION protocol:** Files that cannot be split must carry `// ARCH-EXCEPTION: <reason>` at the top.

**Enforced at:** GATE 0, GATE 1, `atlas-architecture.md`.

---

## DR-W02: Subagent Policy (2026-02-23)

**Decision:** Tiered agent policy. Explore/Plan (haiku) allowed for > 3 Glob/Grep searches. `atlas-doc-auditor` always allowed at GATE 7. Code-writing agents banned.

**Enforced at:** `atlas-architecture.md`.

---

## DR-W03: Branch Hygiene — Max 3 Remote Branches (2026-02-23)

**Decision:** Maximum 3 remote branches: `main` + `gh-pages` + 1 active work branch.

**Enforced at:** GATE -1, `atlas-git.md`.
