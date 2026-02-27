# atlas-package/src/

Atlas package manager. Handles `atlas.toml` manifests, dependency resolution (semver,
conflict detection), lockfiles, registry interaction, and build order computation.

## Files

| File | Role |
|------|------|
| `lib.rs` | Public API, `PackageError` enum, re-exports |
| `manifest.rs` | `PackageManifest`, `Dependency`, `DependencySource`, `Feature`, `VersionConstraint`, `Workspace` |
| `resolver.rs` | `Resolver`, `DependencyGraph`, `Resolution`, `ResolvedPackage`, `ResolverError`, `ResolverResult` |
| `resolver/version_solver.rs` | `VersionSolver` — SAT-style version constraint solver |
| `resolver/graph.rs` | `DependencyGraph` construction and traversal |
| `resolver/conflict.rs` | `Conflict`, `ConflictResolver`, `ConflictingConstraint` — conflict diagnosis |
| `lockfile.rs` | `Lockfile`, `LockedPackage`, `LockedSource`, `LockfileMetadata` — `atlas.lock` read/write |
| `registry/mod.rs` | `Registry` trait, `RegistryManager`, `RegistryError`, `RegistryResult`, `PackageMetadata` |
| `registry/local.rs` | `LocalRegistry` — filesystem-backed package store |
| `registry/remote.rs` | `RemoteRegistry` — HTTP registry client |
| `downloader.rs` | `Downloader` — fetches packages from remote registry, verifies checksums |
| `cache.rs` | `PackageCache` — local disk cache for downloaded packages |
| `build_order.rs` | `BuildOrderComputer`, `BuildOrderResult`, `BuildOrderError` — topological sort for dep graph |
| `validator.rs` | `Validator`, `ValidationError` — validates resolved package set for security/compat |

## Key Types

- `PackageManifest` — parses `atlas.toml`; includes name, version, edition, deps, features, workspace
- `VersionConstraint` — semver ranges: `^`, `~`, `>=`, `=`, `*`
- `Lockfile` — deterministic resolution snapshot; always committed to VCS
- `Resolution` — output of `Resolver::resolve()`: map of package → exact version + source

## Patterns

- `VersionSolver` uses backtracking with unit propagation — not a full SAT solver, but handles
  most real-world conflicts. Complex conflicts fall back to `ConflictResolver` for diagnosis.
- `Lockfile` is the source of truth for builds — never re-resolve if lockfile is present and valid.
- Registry trait is object-safe — `RegistryManager` holds `Box<dyn Registry>` for local/remote mixing.
- Checksums are SHA-256. `Downloader` verifies before extracting to cache.

## Critical Rules

- **Lockfile is immutable during build** — `atlas build` reads the lockfile but never writes it.
  Only `atlas install` / `atlas update` may write the lockfile.
- **No network calls without `allow_network` config** — `RemoteRegistry` checks permissions first.
- Circular dependencies → `PackageError::CircularDependency` (never silently resolved).
