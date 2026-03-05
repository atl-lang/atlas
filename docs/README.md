# Atlas Documentation

**Source of Truth:** The codebase. These docs are generated from codebase analysis.

## Documentation Principles

1. **Code is truth** - If docs disagree with code, docs are wrong
2. **Tested examples** - All code examples verified against actual compiler
3. **AI-friendly** - Small files, one concept each, max ~300 lines
4. **No aspirations** - Only document what works TODAY

## Structure

```
docs/
├── language/           # Language syntax and semantics
│   ├── grammar.md      # Actual syntax (from parser)
│   ├── types.md        # Type system
│   ├── structs-enums.md # Structs and enums
│   ├── functions.md     # Functions and closures
│   └── control-flow.md  # If, while, for, match
│
├── stdlib/             # Standard library (actual functions)
│   ├── index.md        # Overview and categories
│   ├── core.md         # print, len, str, typeof
│   ├── array.md        # Array operations
│   ├── hashmap.md      # HashMap operations
│   ├── string.md       # String operations
│   ├── math.md         # Math functions
│   ├── datetime.md     # DateTime operations
│   ├── file.md         # File I/O
│   ├── http.md         # HTTP client
│   ├── json.md         # JSON parsing
│   ├── regex.md        # Regular expressions
│   ├── async.md        # Async/await
│   └── process.md      # Process spawning
│
├── tooling/            # CLI and tools
│   └── cli.md          # atlas check, atlas run
│
└── known-issues.md     # Current limitations (honest)
```

## How These Docs Were Generated

1. Agents audited the actual codebase
2. Extracted real function signatures from `crates/atlas-runtime/src/stdlib/`
3. Tested examples against `atlas check` and `atlas run`
4. Documented only what actually works

## Archived Documentation

Old docs (potentially inaccurate) are in `/docs-archive/` for historical reference only.

## Contributing

To update these docs:
1. Verify against codebase first
2. Test all code examples
3. Keep files under 300 lines
4. One concept per file
