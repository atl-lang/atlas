# Atlas Documentation

**Source of Truth:** The codebase. Docs may lag behind.

## Quick Links

| Doc | What it covers |
|-----|----------------|
| [cli.md](cli.md) | All CLI commands and flags |
| [testing.md](testing.md) | Test framework guide |
| [language/syntax-quickref.md](language/syntax-quickref.md) | Syntax one-pager |
| [stdlib/index.md](stdlib/index.md) | Stdlib namespace reference |

## Documentation Principles

1. **Code is truth** - If docs disagree with code, docs are wrong
2. **User-facing only** - No internal architecture docs (they go stale too fast)
3. **Locked decisions only** - Only document what's decided via `pt decisions`
4. **AI-friendly** - Small files, one concept each

## Structure

```
docs/
├── cli.md                # CLI command reference
├── testing.md            # Test framework guide
├── language/
│   └── syntax-quickref.md  # Syntax one-pager
└── stdlib/
    ├── index.md          # Namespace overview
    └── test.md           # test.* assertions
```

## What We Document vs Don't

| Document | Don't Document |
|----------|----------------|
| CLI commands (stable) | Compiler internals (churning) |
| Stdlib namespaces (B35 locked) | Type system internals |
| Basic syntax (locked) | Ownership model details |
| Test framework (stable) | Architecture decisions |
| Error codes (stable) | Implementation patterns |

## Decisions

All language decisions are tracked via `pt decisions all`. These are the source of truth for why something works a certain way. Key locked decisions:

- D-021: TypeScript method model (namespace.method)
- D-041: Array type syntax (prefix `[]T`)
- D-042: Tuple syntax
- D-046: Function return type uses colon
- D-047: `.atl` file extension
- D-049: Namespace casing convention
- D-052: Single execution path (VM only)

## Archived

Old docs in `/docs-archive/` - historical reference only.
