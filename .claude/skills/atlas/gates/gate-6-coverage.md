# GATE 6 — Coverage Floors (Lazy-Loaded)

**Load when:** Coverage check fails on CI, or verifying coverage expectations.

---

Coverage is enforced by Codecov on CI — don't run tarpaulin locally (too slow).

| Crate | Minimum floor | If below → STOP |
|-------|--------------|-----------------|
| `atlas-runtime` | 70% | Fix before merge |
| `atlas-cli` | 50% | Fix before merge |
| `atlas-formatter` | 60% | Fix before merge |
| `atlas-lsp` | 40% | Fix before merge |
| `atlas-jit` | 25% | Fix before merge |
| `atlas-config` | 60% | Fix before merge |
| `atlas-build` | 40% | Fix before merge |
| `atlas-package` | 40% | Fix before merge |

**Patch coverage floor:** 80% of new lines must be covered. Unreachable/dead code paths are the only valid exception.

**The local check is:** Did I write tests for every code path I added? If yes, CI will confirm. If no, fix now.
