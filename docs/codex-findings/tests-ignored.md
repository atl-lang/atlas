# Ignored Tests and Deadlock Notes

Target: atlas-runtime tests
Severity: Medium
Status: Open

## Finding: Ignored hashset tests reference old Arc<Mutex> deadlocks

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/tests/collections.rs:515-560

What/Why:
- Two tests are ignored due to Arc<Mutex> self-deadlock scenarios. The runtime has moved to CoW value types, so the reason for ignoring may no longer apply.

Impact:
- Potential behavior regressions remain untested; AI agents may assume coverage that does not exist.

Recommendation:
- Re-evaluate and re-enable these tests under the CoW model. If the issue is still valid, update the ignore reason to the current root cause.
