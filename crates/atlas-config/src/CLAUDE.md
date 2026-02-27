# atlas-config/src/

Atlas configuration system. Handles project manifests (`atlas.toml`), global user config,
and configuration loading/merging for the CLI and build system.

## Files

| File | Role |
|------|------|
| `lib.rs` | Public API, `ConfigError` enum, re-exports. Config hierarchy: global → project → env vars → CLI flags |
| `loader.rs` | `ConfigLoader` — walks directory tree to find `atlas.toml`, merges sources |
| `project.rs` | `ProjectConfig`, `PackageConfig`, `DependencySpec`, `TargetConfig` — parsed from `atlas.toml` |
| `manifest.rs` | `Manifest` — package-focused view of `ProjectConfig`; used by `atlas-package` for dependency resolution |
| `global.rs` | `GlobalConfig` — user-level config at `~/.atlas/config.toml` |
| `security.rs` | `SecurityConfig` — permission declarations in `atlas.toml` (`[security]` table) |

## Key Types

- `ProjectConfig` — full `atlas.toml` parse: package metadata, deps, targets, security
- `Manifest` — subset of `ProjectConfig` for package manager use
- `DependencySpec` — supports semver strings, path deps, git deps
- `ConfigError` — `NotFound`, `IoError`, `TomlParseError`, `InvalidVersion`, `InvalidValue`, `CircularDependency`

## Patterns

- All config loading validates before returning — `validate()` is always called on load.
- TOML parsing uses `toml` crate. Field renames use `#[serde(rename = "...")]`.
- `loader.rs` searches parent directories up to filesystem root for `atlas.toml` (workspace support).
- `security.rs` maps permission names to `atlas-runtime::security::Permission` variants.

## Critical Rules

- **No runtime dependency allowed here** (other than `atlas-runtime::security` for Permission types).
  atlas-config is a leaf crate — it must not pull in interpreter/VM/compiler.
- Config validation must be total — invalid configs must never silently produce defaults.
