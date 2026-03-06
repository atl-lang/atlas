# Gate Applicability Matrix

**Purpose:** Determine which gates to execute for your task.

---

## Decision Tree (Start Here)

| Working on... | Run these gates | Skip |
|---------------|----------------|------|
| **Docs only** | -1, 0, 5, 7 | 1, 2, 3, 4, 6 |
| **LSP/CLI/Frontend** | -1, 0, 1, 2, 4, 5, 6, 7 | 3 (no parity) |
| **Runtime/Stdlib/VM** | ALL gates (-1 through 7) | Nothing — parity required |
| **Small fix (< 50 lines)** | -1, 0, 2, 4, 6, 7 | 1 (no sizing), maybe 5 |

---

## Quick Reference Matrix

| Gate | Runtime | LSP | CLI | VM | Frontend | Docs-Only |
|------|---------|-----|-----|-----|----------|-----------|
| -1 Sanity | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 0 Read Docs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 1 Sizing | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 2 Implement | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| 3 Parity | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| 4 Quality | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| 5 Docs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 6 Testing | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| 7 Memory | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| V Versioning | — event-driven, not part of sequence — |

---

## Domain-Specific Notes

**Load `gates/gate-applicability-domains.md` for detailed domain testing patterns.**

Quick pointers:
- **LSP:** Inline server creation, no helpers (lifetime issues). See `memory/testing-patterns.md`
- **Runtime:** Domain files, parity required. See `memory/testing-patterns.md`
- **CLI:** Integration tests via `assert_cmd`, use `cargo_bin!` macro

---

**Rule of thumb:** When in doubt, run the gate. Skipping is optimization, not requirement.
