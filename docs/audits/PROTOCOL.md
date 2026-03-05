# Audit Protocol

All AI agents conducting audits MUST follow this protocol.

## Directory Structure

```
docs/audits/
├── PROTOCOL.md           # This file
├── INDEX.md              # Master list of all audits
└── {agent}-{date}-{time}/
    ├── SUMMARY.md        # Executive summary (always read first)
    ├── findings.md       # Detailed findings
    ├── {topic}.md        # Topic-specific files if needed
    └── raw/              # Raw data, logs, outputs (optional)
```

## Naming Convention

**Directory:** `{agent}-{YYYY-MM-DD}-{HHMM}/`
- agent: `opus`, `sonnet`, `haiku`, `codex`, `gemini`
- date: ISO format
- time: 24hr, no separators

**Examples:**
- `haiku-2026-03-05-1430/`
- `gemini-2026-03-05-0900/`
- `codex-2026-03-06-2215/`

## Required Files

### SUMMARY.md (MANDATORY)
```markdown
# Audit Summary

**Agent:** {name}
**Date:** {YYYY-MM-DD HH:MM}
**Scope:** {what was audited}
**Duration:** {how long}

## Findings Overview

| Severity | Count |
|----------|-------|
| P0 - Critical | X |
| P1 - High | X |
| P2 - Medium | X |
| P3 - Low | X |

## Critical Issues (P0/P1)

- **{ID}:** {one-line description} → Filed as {issue ID}

## Recommendations

1. {prioritized action}
2. {prioritized action}

## Issues Filed

| Issue ID | Title | Priority |
|----------|-------|----------|
| H-XXX | ... | P0 |
```

### findings.md (MANDATORY)
Detailed findings with evidence. Format:

```markdown
## Finding: {Title}

**Severity:** P0/P1/P2/P3
**Category:** {bug | missing-feature | inconsistency | performance | security}
**Location:** {file:line or component}

### Description
{What's wrong}

### Evidence
{Code snippets, test output, reproduction steps}

### Impact
{What breaks or is blocked}

### Recommendation
{How to fix}

### Issue Filed
{issue ID} or "Not filed - {reason}"
```

## Severity Definitions

| Level | Meaning | Action |
|-------|---------|--------|
| P0 | Blocks core functionality | File immediately, fix before other work |
| P1 | Should exist but doesn't | File immediately, fix after P0s |
| P2 | Works but wrong/suboptimal | File, schedule |
| P3 | Nice to have | File, backlog |

## Filing Issues

**MANDATORY:** All P0 and P1 findings MUST be filed as issues.

```bash
atlas-track add "{title}" P{n} "{description}"
```

Include the issue ID in your SUMMARY.md.

## After Completing Audit

1. Update `docs/audits/INDEX.md` with your audit entry
2. Commit with message: `audit({agent}): {scope} - {X} findings`
3. If P0s found, alert user immediately

## Multi-Agent Audits

For coordinated audits (e.g., Haiku + Gemini battle testing):
- Each agent creates their own directory
- Cross-reference findings: "See also: haiku-2026-03-05-1430/findings.md#issue-name"
- A coordinator (usually Opus) may create a merged summary

## Battle Test Audits

Battle tests in `~/dev/projects/atlas-battle/` produce:
- `audit/FRICTION.md` in each project
- `audit/SESSION.md` session log

After battle test, create audit directory and consolidate findings here.
