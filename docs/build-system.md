# Atlas Build System

**Status:** Implemented (Phases 11a–11c)
**Last Updated:** 2026-02-15

## Overview

Atlas ships a first-class build system covering target discovery, dependency ordering, incremental compilation, caching, profiles, and build scripts. It is implemented in `crates/atlas-build` and integrated with the CLI.

## Components
- `builder.rs` – orchestration of the full pipeline
- `targets.rs` – target definitions (library, binary, bytecode, test, bench)
- `build_order.rs` – topological sort + parallel-ready grouping
- `cache/` – incremental cache, change detection, invalidation
- `profile.rs` – build profiles (dev, release, custom) with overrides
- `script.rs` – build scripts execution (pre/post hooks)
- `output.rs` – formatted build output and diagnostics

## Pipeline
```
Manifest → Target discovery → Dependency graph → Build order → Incremental cache → Compile → Link → Artifacts
```

### Target Discovery
- Reads `atlas.toml` to identify library/binary/test/bench targets.
- Library: requires `src/lib.atlas` entry; Binary: `src/main.atlas`.

### Dependency Graph & Ordering
- Parses imports to build module graph.
- Uses Kahn topological sort; cycles are hard errors with diagnostic spans.
- Parallel-ready grouping of independent modules.

### Incremental & Cache
- Content hashes + mtimes tracked in `target/cache/`.
- Invalidation propagates through dependency graph; unchanged modules are loaded from cache.

### Compilation & Linking
- Pipeline: Lex → Parse → Bind → TypeCheck → Compile to bytecode.
- Links module bytecode into per-target artifacts.

### Profiles
- Profiles defined in `atlas.toml` under `[profile.*]` with flags for optimization, debug info, warnings, lint levels, benchmarks.
- Defaults: `dev`, `release`; custom profiles supported.

### Build Scripts
- Optional `[package.scripts]` hooks: `prebuild`, `postbuild`.
- Scripts run in sandboxed environment respecting security policy.
- Failures abort the build with structured diagnostics.

### Output
- Structured, colorized output summarizing phases, timings, cache hits/misses, and artifacts.

## CLI Integration
- `atlas build` uses `atlas-build` crate internally.
- Flags: `--profile <name>`, `--target <kind>`, `--no-cache` (force rebuild), `--script <hook>` to run hooks only.

## Configuration (atlas.toml)
```toml
[profile.dev]
opt_level = 0
debug = true

[profile.release]
opt_level = 3
debug = false

[package.scripts]
prebuild = "scripts/prebuild.sh"
postbuild = "scripts/postbuild.sh"
```

## Acceptance Checklist
- Targets discovered from `atlas.toml`
- Dependency cycles reported with spans
- Incremental rebuilds reuse cache; changing a file recompiles dependents only
- Profiles override defaults; build scripts executed with diagnostics
- CLI shows cache stats, timings, artifacts

## References
- Implementation: `crates/atlas-build/*`
- Tests: `crates/atlas-build/tests/`
- Benchmarks: N/A (planned in performance phases)
