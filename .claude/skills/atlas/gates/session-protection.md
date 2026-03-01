# Session Protection — Atlas Delegation Map

**Source:** `~/.claude/CLAUDE.md` session protection rules apply here unchanged.
**Atlas lead = architectural authority, not executor.**

---

## What the Lead Executes Directly

- Architecture decisions and design review
- Reviewing sub-agent output and accepting/rejecting findings
- Updating auto-memory (`decisions/*.md`, `MEMORY.md`)
- Directing the next agent or task
- Small targeted edits (1–2 files, ≤ 15KB each, no git ops)

## What Must Be Delegated

| Work | Agent | Model |
|------|-------|-------|
| GATE -1 (all bash checks, cargo build, security scan) | Task agent | Haiku |
| Git operations (add, commit, push, PR, branch) | Task agent | Haiku |
| File reads spanning > 3 files | Explore agent | Haiku |
| Codebase-wide search (> 3 Glob/Grep rounds needed) | Explore agent | Haiku |
| Multi-file Rust implementation (any phase work) | Task agent | Sonnet |
| Architecture planning for complex multi-file changes | Plan agent | Haiku |
| GATE 7 doc audit | atlas-doc-auditor agent | Sonnet |
| Pre-push CodeRabbit check | Task agent | Haiku |

## Haiku Git Agent Prompt Template

```
You are a git agent for the Atlas compiler project.
Working directory: ~/dev/projects/atlas
Task: [describe the git operation]
Track 1 (non-Rust): direct push to main with [skip ci]
Track 2 (Rust source): commit only — no push, no PR (lead handles PR timing)
Rules: never --no-verify, never --force on main, always conventional commits.
Run the operation and return the commit hash.
```

## Token Budget Triggers (→ delegate immediately)

- Response will exceed ~20k tokens → delegate
- Requires reading > 3 files → Explore agent first
- Requires running > 3 commands → Haiku agent
- Implementation touches > 2 Rust files → Sonnet sub-agent
- Session has completed ≥ 5 tasks → delegate ALL remaining work

## Model Reference

| Model | ID | Use for |
|-------|----|---------|
| Haiku | `claude-haiku-4-5-20251001` | Mechanical: git, bash, searches, reads |
| Sonnet | `claude-sonnet-4-5` | Judgment: Rust implementation, multi-file, merges |
| Opus | `claude-opus-4-5` | Rate-limited Sonnet only — user must confirm |

Sub-agent model family: 4.5 only. Lead agent model = user's Claude Code config.

**See:** `~/.claude/agents/session-ops.md` for full spawn protocol.
