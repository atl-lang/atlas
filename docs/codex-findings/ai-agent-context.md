# AI Agent Context Gaps

Target: project governance / auto-memory
Severity: Medium
Status: Open

## Finding: Decision logs referenced but not present in repo

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/CLAUDE.md:69-72
- /Users/proxikal/dev/projects/atlas/docs/specification/memory-model.md:333
- /Users/proxikal/dev/projects/atlas/STATUS.md:155-159

What/Why:
- Multiple documents reference `memory/decisions/*.md` (auto-memory) and DR-* entries, but the decision logs are explicitly “NOT in repo.” This removes key architectural rationale used to guide AI agents.

Impact:
- AI agents lack authoritative decision context, increasing the chance of regressions or contradictory changes.

Recommendation:
- Mirror the decision logs (or a redacted version) into the repo, or create a canonical, checked-in summary file with all active DR decisions and rationale.
