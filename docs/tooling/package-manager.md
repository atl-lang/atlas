# Atlas Package Manager

Atlas uses `atlas.toml` as its package manifest and `atlas.lock` as its lockfile. The package
manager is implemented in the `atlas-package` crate.

---

## atlas.toml — Manifest Format

The manifest is a TOML file located at the project root. All sections except `[package]` are
optional.

### Minimal manifest

```toml
[package]
name = "my-project"
version = "0.1.0"
```

### Full manifest

```toml
[package]
name = "my-project"
version = "1.2.3"
description = "A short description"
authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
license = "MIT"
repository = "https://github.com/example/my-project"
homepage = "https://my-project.dev"
keywords = ["networking", "async"]
categories = ["web"]

[entry]
main = "src/main.atlas"
lib = "src/lib.atlas"

[dependencies]
json = "^1.0"
http-client = "~2.3.0"
crypto = { version = ">=1.0, <2.0", optional = true }
local-utils = { path = "../local-utils" }
experimental = { git = "https://github.com/example/experimental", branch = "main" }

[dev-dependencies]
test-runner = "0.5"

[build]
optimize = "release"
target = "native"

[[build.scripts]]
name = "codegen"
path = "scripts/codegen.atlas"
phase = "pre-build"
timeout = 30
permissions = ["read", "write"]

[lib]
path = "src/lib.atlas"
name = "my-project-lib"

[[bin]]
name = "my-project"
path = "src/main.atlas"

[[bin]]
name = "my-project-cli"
path = "src/cli.atlas"

[features]
networking = { dependencies = ["http-client"] }
crypto-support = { dependencies = ["crypto"], default = false }

[workspace]
members = ["crates/core", "crates/cli"]
exclude = ["crates/experimental"]

[workspace.dependencies]
json = "^1.0"
```

---

## [package] Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Package name (used as identifier in dependencies) |
| `version` | string | yes | SemVer version (`MAJOR.MINOR.PATCH`) |
| `description` | string | no | Short one-line description |
| `authors` | string[] | no | Author strings, typically `"Name <email>"` |
| `license` | string | no | SPDX license identifier (e.g., `"MIT"`, `"Apache-2.0"`) |
| `repository` | string | no | Source repository URL |
| `homepage` | string | no | Project homepage URL |
| `keywords` | string[] | no | Search keywords |
| `categories` | string[] | no | Registry category tags |

---

## [entry] Fields

| Field | Type | Description |
|---|---|---|
| `main` | string | Path to main executable entry point (relative to project root) |
| `lib` | string | Path to library entry point (relative to project root) |

---

## Dependency Declarations

Dependencies go in `[dependencies]` (runtime) or `[dev-dependencies]` (build and test only).

### Simple form (version constraint only)

```toml
[dependencies]
json = "^1.0"
```

### Detailed form

```toml
[dependencies]
http-client = { version = "~2.3.0" }
crypto = { version = ">=1.0, <2.0", optional = true }
local-utils = { path = "../local-utils" }
git-dep = { git = "https://github.com/example/repo", branch = "main" }
pinned = { git = "https://github.com/example/repo", rev = "abc123" }
tagged = { git = "https://github.com/example/repo", tag = "v1.2.3" }
```

### Detailed dependency fields

| Field | Type | Description |
|---|---|---|
| `version` | string | SemVer version constraint |
| `git` | string | Git repository URL |
| `branch` | string | Git branch (used with `git`) |
| `tag` | string | Git tag (used with `git`) |
| `rev` | string | Exact commit hash (used with `git`) |
| `path` | string | Relative path to local package |
| `registry` | string | Named registry override |
| `optional` | bool | If true, dependency is not included unless a feature enables it |
| `features` | string[] | Feature flags to enable in the dependency |
| `default-features` | bool | Whether to include the dependency's default features |
| `package` | string | Rename: use this as the import name instead of the package name |

---

## Version Constraints

| Syntax | Meaning |
|---|---|
| `"1.2.3"` | Exact version 1.2.3 |
| `"^1.2.3"` | Compatible: `>=1.2.3, <2.0.0` (same major) |
| `"~1.2.3"` | Patch-compatible: `>=1.2.3, <1.3.0` (same major + minor) |
| `">=1.0, <2.0"` | Range expression |
| `"*"` | Any version |

Constraint parsing uses the `semver` crate. Multiple constraints in a range expression must
all be satisfied simultaneously.

---

## [build] Fields

| Field | Type | Description |
|---|---|---|
| `optimize` | string | Optimization level (e.g., `"release"`, `"debug"`) |
| `target` | string | Compilation target (e.g., `"native"`) |
| `scripts` | array | Build script entries (see below) |

### Build script fields (`[[build.scripts]]`)

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | Script identifier |
| `path` | string | no | Path to Atlas script file |
| `shell` | string | no | Shell command to run |
| `phase` | string | yes | When to run: `"pre-build"`, `"post-build"`, etc. |
| `timeout` | number | no | Timeout in seconds |
| `permissions` | string[] | no | Permissions granted to script: `"read"`, `"write"`, `"network"`, etc. |

---

## [lib] Fields

| Field | Type | Description |
|---|---|---|
| `path` | string | Path to library entry file |
| `name` | string | Override library name (defaults to package name) |

---

## [[bin]] Fields

| Field | Type | Description |
|---|---|---|
| `name` | string | Binary name |
| `path` | string | Path to entry file |

---

## [features] Fields

Each key is a feature name:

```toml
[features]
networking = { dependencies = ["http-client"], default = false }
```

| Field | Type | Description |
|---|---|---|
| `dependencies` | string[] | Dependency names that this feature enables |
| `default` | bool | Whether this feature is enabled by default |

---

## [workspace] Fields

| Field | Type | Description |
|---|---|---|
| `members` | string[] | Glob patterns or paths to workspace member packages |
| `exclude` | string[] | Paths to exclude from membership |
| `dependencies` | table | Shared dependency versions for all workspace members |

---

## atlas.lock — Lockfile Format

The lockfile (`atlas.lock`) is generated automatically and must be committed to version control.
It is TOML format, version 1.

```toml
version = 1

[metadata]
generated_at = "2026-03-13T12:00:00Z"
atlas_version = "0.3.0"

[[packages]]
name = "json"
version = "1.4.2"
checksum = "sha256:abc123..."

[packages.source]
type = "registry"

[[packages]]
name = "http-client"
version = "2.3.1"

[packages.source]
type = "registry"
registry = "https://registry.atlas-lang.dev"

[packages.dependencies]
json = "1.4.2"

[[packages]]
name = "local-utils"
version = "0.2.0"

[packages.source]
type = "path"
path = "../local-utils"

[[packages]]
name = "experimental"
version = "0.1.0"

[packages.source]
type = "git"
url = "https://github.com/example/experimental"
rev = "abc123def456789"
```

### Lockfile fields

**Top-level**

| Field | Type | Description |
|---|---|---|
| `version` | integer | Lockfile format version (currently `1`) |
| `packages` | array | Resolved package entries |
| `metadata.generated_at` | string | ISO 8601 timestamp |
| `metadata.atlas_version` | string | Atlas version that generated the lockfile |

**Per package (`[[packages]]`)**

| Field | Type | Description |
|---|---|---|
| `name` | string | Package name |
| `version` | string | Exact resolved SemVer version |
| `checksum` | string | SHA-256 checksum for integrity verification (omitted for path/git) |
| `source.type` | string | `"registry"`, `"git"`, or `"path"` |
| `source.registry` | string | Registry URL (optional; omitted for default registry) |
| `source.url` | string | Git repository URL (for `type = "git"`) |
| `source.rev` | string | Resolved commit hash (for `type = "git"`) |
| `source.path` | string | Local path (for `type = "path"`) |
| `dependencies` | table | Direct dependencies: name → exact version |

---

## Dependency Resolution

The resolver (`Resolver`) implements a constraint satisfaction approach using the PubGrub
algorithm via `VersionSolver`. Conflicts are diagnosed by `ConflictResolver`.

**Resolution process:**

1. Parse `atlas.toml` manifest
2. If `atlas.lock` is present and valid (all manifest constraints satisfied by locked versions),
   use the lockfile directly — no re-resolution
3. If lockfile is missing or stale, run fresh constraint solving:
   - Add all manifest dependency constraints
   - Query registry for available versions
   - Find maximum version satisfying all constraints for each package
   - Compute topological build order (via `DependencyGraph`)
   - Generate new `atlas.lock`

**Lockfile validity check:** The resolver verifies that every dependency in `[dependencies]` has
a locked entry whose exact version satisfies the manifest constraint. Integrity is also verified
(no duplicate package names, format version check).

**Circular dependency detection:** Detected during topological sort. Results in
`PackageError::CircularDependency` — never silently resolved.

---

## Registry

The `Registry` trait is implemented by:

- `LocalRegistry` — filesystem-backed package store
- `RemoteRegistry` — HTTP registry client

`RegistryManager` holds multiple `Box<dyn Registry>` sources and queries them in order.
Caching is enabled by default. Checksums are SHA-256, verified by `Downloader` before
extracting to cache. Network requests require `allow_network` configuration.

---

## Package Manager CLI Commands

All commands operate on the `atlas.toml` in the current directory unless a path is specified.

```bash
# Initialize a new package
atlas init [name]
atlas init --lib [name]

# Add a dependency
atlas add json
atlas add json@^1.0
atlas add json --dev

# Remove a dependency
atlas remove json

# Install all dependencies (writes atlas.lock)
atlas install

# Update dependencies to newest versions satisfying constraints
atlas update
atlas update json          # update specific package only

# Build the project (reads atlas.lock, never writes it)
atlas build
atlas build --release

# Run the project
atlas run
atlas run -- arg1 arg2

# Run tests
atlas test

# Publish to registry
atlas publish

# Show dependency tree
atlas tree
```

---

## Critical Rules

- **`atlas.lock` is immutable during `atlas build`** — the build command reads the lockfile but
  never writes it. Only `atlas install` and `atlas update` may write the lockfile.
- **Always commit `atlas.lock`** — it is the source of truth for reproducible builds.
- **No network without `allow_network`** — `RemoteRegistry` enforces this at the API level.
- **Circular dependencies are fatal** — they produce an error, not a silently skipped node.
