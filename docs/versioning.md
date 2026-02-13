# Atlas Versioning Policy

## Scheme
Atlas follows Semantic Versioning 2.0.0: `MAJOR.MINOR.PATCH`

**Current Version:** 0.1.0

## Rules
- **MAJOR:** Breaking changes to syntax, semantics, or diagnostics
- **MINOR:** New features that are backward compatible
- **PATCH:** Bug fixes only

## Component Versioning

Atlas tracks versions for multiple components to ensure stability and compatibility:

| Component | Constant | Current | Location |
|-----------|----------|---------|----------|
| Runtime | `VERSION` | 0.1.0 | `crates/atlas-runtime/src/lib.rs` |
| AST | `AST_VERSION` | 1 | `crates/atlas-runtime/src/ast.rs` |
| Diagnostics | `DIAG_VERSION` | 1 | `crates/atlas-runtime/src/diagnostic.rs` |
| Bytecode | `BYTECODE_VERSION` | 1 | `crates/atlas-runtime/src/bytecode/mod.rs` |
| Typecheck Dump | `TYPECHECK_VERSION` | 1 | `crates/atlas-runtime/src/typecheck_dump.rs` |

### Version Increment Rules

**Runtime VERSION (MAJOR.MINOR.PATCH):**
- Follows semantic versioning for the entire Atlas language
- Bumped via `Cargo.toml` workspace version

**Component Versions (Integer):**
- **AST_VERSION:** Increment when AST structure changes (breaks serialization)
- **DIAG_VERSION:** Increment when diagnostic schema changes (breaks tooling)
- **BYTECODE_VERSION:** Increment when bytecode format changes (breaks .abc files)
- **TYPECHECK_VERSION:** Increment when typecheck dump format changes (breaks IDE integration)

## Stability Guarantees

### v0.x (Pre-1.0)
- No stability guarantees
- Breaking changes allowed in MINOR versions
- Diagnostic codes may change
- Bytecode format may change

### v1.x (Post-1.0)
- **Syntax:** No breaking changes without MAJOR bump
- **Semantics:** Behavior changes require MAJOR bump
- **Diagnostics:** Error code changes require MAJOR bump
- **Bytecode:** Format changes require MAJOR bump
- **API:** Public API changes follow semver strictly

## Minimum Supported Rust Version (MSRV)

**Current MSRV:** 1.70

- MSRV bumps require MINOR version increment
- Documented in RELEASE.md
- Tested in CI

## Deprecation Policy

Features marked as deprecated:
1. Emit warnings for one MINOR version
2. Removed in next MAJOR version
3. Documented in CHANGELOG.md

## Version Checking

Tools can check component versions before processing:
```rust
use atlas_runtime::{AST_VERSION, BYTECODE_VERSION, DIAG_VERSION};

// Verify compatibility
assert_eq!(ast.ast_version, AST_VERSION);
assert_eq!(bytecode.version, BYTECODE_VERSION);
```
