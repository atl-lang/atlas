# atlas-build/src/

Atlas build system. Orchestrates compilation of Atlas projects: dependency ordering,
parallel builds, incremental compilation, build profiles, and build scripts.

## Files

| File | Role |
|------|------|
| `lib.rs` | Public API and re-exports |
| `builder.rs` | `Builder`, `BuildConfig`, `BuildContext`, `BuildStats`, `OptLevel` — core build orchestrator |
| `build_order.rs` | `BuildGraph`, `ModuleNode` — topological sort of module dependencies |
| `targets.rs` | `BuildTarget`, `TargetKind`, `BuildArtifact`, `ArtifactMetadata` — target types (binary, library, bytecode, test) |
| `incremental.rs` | `IncrementalEngine`, `BuildState`, `IncrementalPlan`, `IncrementalStats`, `RecompileReason` |
| `fingerprint.rs` | `Fingerprint`, `FingerprintDb`, `FingerprintConfig`, `PlatformInfo` — content hashing for cache invalidation |
| `cache/mod.rs` | `BuildCache`, `CacheEntry`, `CacheMetadata`, `CacheStats` |
| `cache/invalidation.rs` | Cache invalidation rules |
| `cache/metadata.rs` | Cache metadata persistence |
| `profile.rs` | `Profile`, `ProfileConfig`, `ProfileManager`, `DependencyProfile`, `ManifestProfileConfig` — dev/release/custom profiles |
| `script.rs` | `BuildScript`, `ScriptExecutor`, `ScriptContext`, `ScriptKind`, `ScriptPhase`, `ScriptResult` — sandboxed build scripts |
| `module_resolver.rs` | Module path resolution during build |
| `output.rs` | `BuildProgress`, `BuildSummary`, `ErrorFormatter`, `OutputMode` — progress reporting |
| `error.rs` | `BuildError`, `BuildResult` |

## Key Invariants

- Build order is a DAG — cycles are detected and returned as `BuildError::CircularDependency`.
- Incremental engine uses content fingerprints (SHA-256), not timestamps.
- Build scripts run in a sandbox — they cannot access the network or write outside `$OUT_DIR`.
- `OptLevel` controls codegen: `Debug` (no opts), `Release` (full), `Size` (size-optimized).

## Patterns

- `Builder` is the entry point — callers construct `BuildConfig` and call `Builder::build()`.
- `IncrementalEngine` queries `FingerprintDb` to determine which modules need recompilation.
- Progress is reported via `BuildProgress` (callback-based, not stdout directly).

## Critical Rules

- **Build scripts are sandboxed** — never execute them without going through `ScriptExecutor`.
- **No interpreter/VM calls during build** — build system only invokes the compiler pipeline
  (`atlas-runtime::Compiler`), not the interpreter or VM.
- Parallel compilation uses Rayon — keep shared state behind `Arc<Mutex<>>` (not CoW, build-time only).
