# Structure Reference (Archived 2026-02)

Stable reference material moved from patterns.md.

---

## Multi-Crate Structure

```
crates/
├── atlas-runtime/    # Core: lexer, parser, compiler, interpreter, VM, stdlib
├── atlas-cli/        # CLI binary (atlas run, atlas fmt, atlas test)
├── atlas-formatter/  # Code formatter (comments, indentation, config)
├── atlas-config/     # Configuration system (manifest, project, global)
├── atlas-build/      # Build system (incremental, caching)
├── atlas-package/    # Package manager (resolver, lockfile, downloader)
├── atlas-jit/        # JIT compilation foundation
└── atlas-lsp/        # Language server protocol
```

Most work happens in `atlas-runtime`. Other crates depend on it.

---

## Stdlib Module Organization

```
stdlib/
├── mod.rs           # is_builtin() + call_builtin() registry
├── string.rs        # String functions
├── array.rs         # Array functions
├── math.rs          # Math functions
├── json.rs          # JSON functions
├── io.rs            # I/O functions
├── fs.rs            # File system functions
├── path.rs          # Path manipulation
├── regex.rs         # Regex functions
├── datetime.rs      # Date/time functions
├── http.rs          # HTTP functions
├── process.rs       # Process management
├── test.rs          # Testing assertions
├── types.rs         # Type utilities
├── reflect.rs       # Reflection API
├── collections/     # Collection types
│   ├── hashmap.rs
│   ├── hashset.rs
│   ├── queue.rs
│   └── stack.rs
├── compression/     # Compression modules
└── async_*.rs       # Async primitives
```
