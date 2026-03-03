# Atlas v0.3 Migration Guides

**Purpose:** Help migrate existing Atlas code from v0.2 to v0.3 syntax.

Each guide covers one breaking change with before/after examples and
mechanical transformation rules that AI agents can apply.

## Guides

| Guide | Decision | Change |
|-------|----------|--------|
| [var-to-let-mut.md](var-to-let-mut.md) | D-006 | `var` → `let mut` |
| [loops.md](loops.md) | D-007 | C-style for → for-in/while |
| [functions.md](functions.md) | D-008 | Arrow → fn expression |
| [record-literals.md](record-literals.md) | D-009 | `{ }` → `record { }` |

## Migration Order

Apply in this order to avoid conflicts:

1. **var-to-let-mut** — Simple text replacement
2. **loops** — Requires restructuring, may need temp variables
3. **functions** — Requires adding type annotations
4. **record-literals** — Simple prefix addition

## Automated Migration

Future: `atlas migrate v0.2-to-v0.3 <file.atl>` command planned.
For now, use these guides for manual migration.
