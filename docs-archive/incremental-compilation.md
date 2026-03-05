# Incremental Compilation

Atlas implements incremental compilation to dramatically improve rebuild times by only recompiling modules that have changed or whose dependencies have changed.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 Builder                          │
│  build_incremental()                             │
│                                                  │
│  ┌──────────────┐  ┌──────────────────────────┐ │
│  │ BuildGraph   │  │ IncrementalEngine        │ │
│  │              │──│                          │ │
│  │ - modules    │  │ - FingerprintDb          │ │
│  │ - deps       │  │ - BuildState             │ │
│  │ - build order│  │ - FingerprintConfig      │ │
│  └──────────────┘  └──────────────────────────┘ │
│                           │                      │
│                    ┌──────┴──────┐               │
│                    │ plan()      │               │
│                    │             │               │
│                    ▼             ▼               │
│              ┌──────────┐ ┌──────────┐          │
│              │ Recompile│ │ Cached   │          │
│              └──────────┘ └──────────┘          │
└─────────────────────────────────────────────────┘
```

## Components

### Fingerprint (`fingerprint.rs`)

A fingerprint uniquely identifies a module's compilation state by combining:

- **Source content hash** (SHA-256)
- **Dependency fingerprint hashes** (sorted by name for determinism)
- **Compiler version** (invalidates on upgrade)
- **Platform info** (OS + architecture)
- **Build configuration hash** (optimization level, env vars)

```rust
Fingerprint {
    hash: "combined_sha256",
    source_hash: "sha256_of_source",
    dependency_hashes: {"dep_name": "dep_fingerprint_hash"},
    compiler_version: "0.2.0",
    platform: PlatformInfo { os: "macos", arch: "aarch64" },
    config_hash: "sha256_of_config",
    mtime: Some(SystemTime),
    file_size: 1234,
}
```

#### Comment-Only Change Detection

When `FingerprintConfig.ignore_comments` is enabled, single-line (`//`) and multi-line (`/* */`) comments are stripped before hashing. This prevents unnecessary recompilation when only comments change.

#### Quick Check

Before computing a full fingerprint, a quick mtime + file size check can eliminate unchanged files without reading content.

### FingerprintDb (`fingerprint.rs`)

Persistent database mapping module names to their last-known fingerprints. Stored as JSON at `<target>/incremental/fingerprints.json`.

- Invalidates automatically on compiler version change
- Invalidates on platform change
- Supports insert, get, remove, clear operations

### IncrementalEngine (`incremental.rs`)

The orchestration layer that ties fingerprinting, dependency tracking, and build state together.

**Key operation: `plan(graph) -> IncrementalPlan`**

1. Compute current fingerprints for all modules
2. Compare against stored fingerprints to find directly changed modules
3. Propagate invalidation through reverse dependency graph (BFS)
4. Classify modules as "recompile" or "cached"

**IncrementalPlan** contains:
- `recompile`: modules that need compilation
- `cached`: modules that can skip compilation
- `reasons`: why each module needs recompilation (source changed, dependency changed, no previous fingerprint)
- `total_modules`: for computing ratios

### BuildState (`incremental.rs`)

Persisted state tracking:
- Dependency graph edges
- Module paths
- Last successful build time
- Format version (for forward compatibility)

Stored at `<target>/incremental/build_state.json`.

### Build Cache (`cache/mod.rs`)

Caches compiled bytecode per module with:
- SHA-256 content hash for validation
- Timestamp for quick checks
- LRU eviction when size limit exceeded (default 1GB)
- Stale entry cleanup (30 day threshold)
- Cache version + compiler version validation

## Build Flow

```
build_incremental()
├── Discover source files
├── Build dependency graph
├── Validate graph (no cycles, all deps exist)
├── Initialize IncrementalEngine (load persisted state)
├── engine.plan(graph) → IncrementalPlan
│   ├── Compute fingerprints for all modules
│   ├── Compare with stored fingerprints
│   ├── Propagate invalidation via reverse deps
│   └── Return {recompile, cached, reasons}
├── For each module:
│   ├── If in recompile set: compile, record fingerprint, store in cache
│   └── If cached: use cached result
├── engine.update_state(graph)
├── engine.save() + cache.save()
├── Create build targets
├── Link artifacts
└── Return BuildContext with stats
```

## Invalidation Propagation

When module C changes:
```
A depends on B
B depends on C    ← C changed

Invalidation: C → B → A (all recompiled)
```

Uses BFS through reverse dependency graph for efficient propagation.

## Configuration

```rust
FingerprintConfig {
    optimization: "O2",          // Opt level in fingerprint
    ignore_comments: false,      // Skip comment-only changes
    env_vars: {"KEY": "VALUE"},  // Env vars in fingerprint
}
```

## Performance Characteristics

| Scenario | Behavior |
|----------|----------|
| Clean build (no cache) | Full compilation, populate fingerprints |
| No changes | Only fingerprint comparison, skip compilation |
| Single file change | Recompile changed file + dependents |
| Dependency change | Cascade recompilation through dep graph |
| Compiler upgrade | Full rebuild (fingerprint DB invalidated) |
| Platform change | Full rebuild (platform mismatch) |
| Config change | Recompile affected modules |

## File Layout

```
<target>/
├── incremental/
│   ├── fingerprints.json    # FingerprintDb
│   └── build_state.json     # BuildState
├── cache/
│   ├── metadata.json        # CacheMetadata
│   └── modules/
│       ├── main.json        # CacheEntry per module
│       └── lib.json
└── bin/
    └── project.abc          # Build artifacts
```

## API

### Builder

```rust
// Full build
builder.build()?;

// Incremental build (uses fingerprinting + caching)
builder.build_incremental()?;
```

### IncrementalEngine

```rust
let engine = IncrementalEngine::new(state_dir, config);

// Analyze what needs recompilation
let plan = engine.plan(&graph)?;

// After compilation, record result
engine.record_compilation("module", &path, &source, dep_hashes);

// Persist state
engine.save()?;

// Force recompilation
engine.invalidate_module("module");
engine.invalidate_all();
```

### FingerprintDb

```rust
let mut db = FingerprintDb::new();
db.insert("module".to_string(), fingerprint);
db.needs_recompile("module", &current_fingerprint);
db.save(&path)?;
let db = FingerprintDb::load(&path);
```

## Testing

61+ integration tests covering:
- Initial full builds
- No-change rebuilds
- Single-file change detection
- Dependency propagation
- Fingerprint correctness
- Cache hits/misses
- Build state persistence
- Comment-only change detection
- Configuration-aware fingerprinting
- Engine lifecycle (create, plan, record, save, reload)
- Builder integration
- Edge cases (empty projects, type errors, optimization levels)

Plus 138+ unit tests in the library covering all components.
