# Audit Index

Master list of all audits conducted on Atlas.

## Active Audits

| Date | Agent | Scope | P0s | P1s | Status |
|------|-------|-------|-----|-----|--------|
| 2026-03-05 | Gemini | Doc coverage | 0 | 4 | Complete |
| 2026-03-05 | Codex | AST implementation | 0 | 0 | Complete |

## Audit Reports

### 2026-03-05: Documentation Audit (Gemini + Codex)
- **Scope:** Doc accuracy and coverage vs codebase
- **Findings:** 109+ issues in original docs, all fixed
- **Output:** `docs/stdlib/*.md` updated (AST_STATUS.md since removed as stale)

### Pending Battle Tests
Projects in `~/dev/projects/atlas-battle/` ready for audit:

| Project | Stresses | Assigned | Status |
|---------|----------|----------|--------|
| atlas-parse | CSV/INI/TOML, regex, nested data | - | Ready |
| atlas-metrics | floating-point, datetime, aggregation | - | Ready |
| atlas-sieve | numeric perf, tight loops, CoW | - | Ready |
| atlas-graph | recursion, pattern matching, enums | - | Ready |
| atlas-router | async middleware, HTTP handlers | - | Ready |
| atlas-github-bot | HTTP server, webhooks, JSON | - | Ready |
| atlas-scraper | rate limiting, concurrent HTTP | - | Ready |
| atlas-log-parser | file watchers, regex, aggregation | - | Ready |
| atlas-task-runner | process spawning, env vars | - | Ready |
| atlas-url-shortener | concurrent HashMap, persistence | - | Ready |
| atlas-json-validator | recursive validation, deep traversal | - | Ready |
| atlas-file-sync | file watchers, directory recursion | - | Ready |
| atlas-discord-bot | WebSockets, concurrent state | - | Ready |

## Completed Battle Tests

| Project | Agent | Date | Issues Found |
|---------|-------|------|--------------|
| atlas-test | Sonnet | 2026-03 | H-069 (closure/global) |
| atlas-todo | Haiku | 2026-03 | H-066 (struct fields) |

## How to Add an Audit

1. Follow `PROTOCOL.md`
2. Create directory: `{agent}-{date}-{time}/`
3. Add entry to this index
4. File all P0/P1 issues via `atlas-track`
