# Phase 08a: Package Manager - Resolver Core (PubGrub Algorithm)

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Foundation phase-07 (Package Manifest) must be complete.

**Verification Steps:**

1. Check STATUS.md: Foundation section, phase-07 should be ✅
   ```bash
   grep "phase-07-package-manifest.md" STATUS.md
   ```

2. Verify atlas-package crate exists:
   ```bash
   ls crates/atlas-package/src/lib.rs
   ls crates/atlas-package/src/manifest.rs
   ls crates/atlas-package/src/lockfile.rs
   ```

3. Verify PackageManifest struct exists:
   ```bash
   grep -n "pub struct PackageManifest" crates/atlas-package/src/manifest.rs
   ```

4. Verify semver dependency exists:
   ```bash
   grep "semver" crates/atlas-package/Cargo.toml
   ```

**Expected from phase-07:**
- atlas-package crate created
- PackageManifest struct for atlas.toml parsing
- Lockfile (atlas.lock) structure
- Dependency specifications (semver, git, path)
- semver dependency in Cargo.toml

**Decision Tree:**

a) If phase-07 complete (STATUS.md ✅, files exist):
   → Proceed with phase-08a
   → Implement PubGrub resolver core

b) If phase-07 incomplete:
   → STOP immediately
   → Report: "Foundation phase-07 required before phase-08a"
   → Complete phase-07 first

c) If semver dependency missing:
   → ERROR: Phase-07 incomplete
   → Verify phase-07 acceptance criteria met

**No user questions needed:** Phase-07 completion is verifiable via STATUS.md and file structure.

---

## Objective

Implement the core dependency resolution algorithm using PubGrub (constraint satisfaction solver) - resolving version constraints, building dependency graphs, and detecting conflicts. This is the foundation for automated dependency management.

## Files

**Create:** `crates/atlas-package/src/resolver.rs` (~800 lines)
**Create:** `crates/atlas-package/src/resolver/graph.rs` (~300 lines)
**Create:** `crates/atlas-package/src/resolver/version_solver.rs` (~400 lines)
**Update:** `crates/atlas-package/src/lib.rs` (~50 lines - exports)
**Update:** `crates/atlas-package/Cargo.toml` (~10 lines - add pubgrub dependency)
**Tests:** `crates/atlas-package/tests/resolver_core_tests.rs` (~600 lines)

**Total:** ~2160 lines

## Dependencies

**Rust Crates:**
- `semver` (already in Cargo.toml from phase-07)
- `pubgrub` - PubGrub algorithm implementation (ADD to Cargo.toml)
- `thiserror` (already in Cargo.toml)

**Phase Dependencies:**
- PackageManifest from phase-07
- Dependency struct from phase-07
- semver types from phase-07

## Implementation

### 1. Add pubgrub Dependency

Update `crates/atlas-package/Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
semver = { version = "1.0", features = ["serde"] }
thiserror = "2.0"
chrono = "0.4"
pubgrub = "0.2"  # ADD THIS
```

### 2. Resolver Core Structure

Create `crates/atlas-package/src/resolver.rs`:

```rust
use crate::manifest::{PackageManifest, Dependency};
use semver::{Version, VersionReq};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("No version of package '{package}' satisfies constraints: {constraints}")]
    NoSatisfyingVersion {
        package: String,
        constraints: String,
    },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),
}

pub type ResolverResult<T> = Result<T, ResolverError>;

/// Core dependency resolver using PubGrub algorithm
pub struct Resolver {
    /// Dependency graph being built
    graph: DependencyGraph,

    /// Version constraints for each package
    constraints: HashMap<String, Vec<VersionReq>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            constraints: HashMap::new(),
        }
    }

    /// Resolve dependencies from a manifest
    pub fn resolve(&mut self, manifest: &PackageManifest) -> ResolverResult<Resolution> {
        // Add root package constraints
        for (name, dep) in &manifest.dependencies {
            self.add_constraint(name, dep)?;
        }

        // Run PubGrub algorithm
        self.solve()
    }

    /// Add version constraint for a package
    fn add_constraint(&mut self, package: &str, dep: &Dependency) -> ResolverResult<()> {
        // Implementation in phase
        todo!()
    }

    /// Run constraint solver to find compatible versions
    fn solve(&mut self) -> ResolverResult<Resolution> {
        // Implementation in phase
        todo!()
    }
}

/// Resolved dependency set with exact versions
#[derive(Debug, Clone)]
pub struct Resolution {
    /// Resolved packages with exact versions
    pub packages: HashMap<String, ResolvedPackage>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<String>,
}
```

### 3. Dependency Graph

Create `crates/atlas-package/src/resolver/graph.rs`:

```rust
use semver::Version;
use std::collections::{HashMap, HashSet};
use super::ResolverError;

/// Dependency graph tracking package relationships
pub struct DependencyGraph {
    /// Adjacency list: package -> dependencies
    edges: HashMap<String, HashSet<String>>,

    /// Package versions
    versions: HashMap<String, Version>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            versions: HashMap::new(),
        }
    }

    /// Add package to graph
    pub fn add_package(&mut self, name: String, version: Version) {
        self.versions.insert(name.clone(), version);
        self.edges.entry(name).or_insert_with(HashSet::new);
    }

    /// Add dependency edge
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<(), ResolverError> {
        // Check for circular dependencies
        if self.would_create_cycle(from, to) {
            return Err(ResolverError::CircularDependency(
                format!("{} -> {}", from, to)
            ));
        }

        self.edges
            .entry(from.to_string())
            .or_insert_with(HashSet::new)
            .insert(to.to_string());

        Ok(())
    }

    /// Check if adding edge would create cycle
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // DFS to detect cycle
        // Implementation in phase
        false
    }

    /// Get topological sort (build order)
    pub fn topological_sort(&self) -> Result<Vec<String>, ResolverError> {
        // Kahn's algorithm for topological sorting
        // Implementation in phase
        todo!()
    }
}
```

### 4. Version Solver (PubGrub Integration)

Create `crates/atlas-package/src/resolver/version_solver.rs`:

```rust
use pubgrub::{
    solver::resolve,
    type_aliases::SelectedDependencies,
    package::Package,
    version::Version as PubGrubVersion,
};
use semver::{Version, VersionReq};
use std::collections::HashMap;

/// Wrapper for PubGrub version solving
pub struct VersionSolver {
    /// Available package versions
    available_versions: HashMap<String, Vec<Version>>,
}

impl VersionSolver {
    pub fn new() -> Self {
        Self {
            available_versions: HashMap::new(),
        }
    }

    /// Register available versions for a package
    pub fn add_package_versions(&mut self, package: &str, versions: Vec<Version>) {
        self.available_versions.insert(package.to_string(), versions);
    }

    /// Solve version constraints using PubGrub
    pub fn solve(
        &self,
        constraints: &HashMap<String, Vec<VersionReq>>,
    ) -> Result<HashMap<String, Version>, String> {
        // Convert to PubGrub types and solve
        // Implementation in phase
        todo!()
    }

    /// Find maximum version satisfying constraints
    pub fn max_satisfying_version(
        &self,
        package: &str,
        constraints: &[VersionReq],
    ) -> Option<Version> {
        let versions = self.available_versions.get(package)?;

        versions.iter()
            .filter(|v| constraints.iter().all(|req| req.matches(v)))
            .max()
            .cloned()
    }
}
```

### 5. Update lib.rs

Update `crates/atlas-package/src/lib.rs`:

```rust
mod manifest;
mod lockfile;
mod validator;
pub mod resolver;  // ADD THIS

pub use manifest::*;
pub use lockfile::*;
pub use validator::*;
pub use resolver::{Resolver, Resolution, ResolvedPackage, ResolverError};  // ADD THIS
```

## Tests (TDD - Use rstest)

Create `crates/atlas-package/tests/resolver_core_tests.rs`:

**Core resolution tests:**
1. `test_resolve_simple_dependency_tree` - Single-level deps
2. `test_resolve_transitive_dependencies` - Multi-level deps
3. `test_find_maximum_compatible_versions` - Version selection
4. `test_version_conflict_detected` - Conflicting constraints
5. `test_no_satisfying_version_error` - Impossible constraints
6. `test_circular_dependency_error` - Cycle detection
7. `test_empty_manifest_resolution` - No deps case
8. `test_multiple_constraints_same_package` - Constraint merging
9. `test_pre_release_version_handling` - Pre-release semantics
10. `test_resolution_is_deterministic` - Same input = same output

**Version constraint tests:**
11. `test_exact_version_match` - "1.2.3"
12. `test_caret_range_compatibility` - "^1.2.3"
13. `test_tilde_range_compatibility` - "~1.2.3"
14. `test_range_constraints_greater_than` - ">=1.0.0"
15. `test_range_constraints_less_than` - "<2.0.0"
16. `test_range_constraints_combined` - ">=1.0.0, <2.0.0"
17. `test_wildcard_version` - "*"
18. `test_pre_release_version_semantics` - "1.0.0-alpha"
19. `test_version_comparison_ordering` - Version ordering
20. `test_constraint_intersection` - Multiple constraints

**Dependency graph tests:**
21. `test_add_package_to_graph` - Node addition
22. `test_add_edge_to_graph` - Edge addition
23. `test_circular_dependency_detection` - Simple cycle
24. `test_circular_dependency_complex` - Multi-node cycle
25. `test_topological_sort_simple` - Linear deps
26. `test_topological_sort_diamond` - Diamond deps
27. `test_graph_empty` - No packages
28. `test_graph_single_package` - One package, no deps

**Conflict detection tests:**
29. `test_simple_version_conflict` - Direct conflict
30. `test_transitive_conflict` - Indirect conflict
31. `test_conflict_error_message_clarity` - Error contains constraint sources
32. `test_multiple_conflicts_reported` - All conflicts shown

**Edge cases:**
33. `test_package_depends_on_itself` - Self-dependency error
34. `test_malformed_version_constraint` - Invalid constraint
35. `test_empty_constraint_set` - No constraints for package
36. `test_many_packages_performance` - Large dep graph (100+ packages)

**Minimum test count:** 35+ tests

## Integration Points

- **Uses:** PackageManifest from phase-07
- **Uses:** Dependency from phase-07
- **Uses:** semver crate
- **Creates:** Resolver core engine
- **Creates:** DependencyGraph
- **Creates:** VersionSolver
- **Output:** Resolution with exact versions
- **Next:** Phase-08b will add registry and downloader

## Acceptance Criteria

- [ ] PubGrub dependency added to Cargo.toml
- [ ] Resolver struct implements core resolution
- [ ] DependencyGraph tracks dependencies
- [ ] VersionSolver integrates PubGrub
- [ ] Circular dependency detection works
- [ ] Version conflict detection works
- [ ] Transitive dependencies resolved
- [ ] Maximum compatible versions selected
- [ ] Topological sort produces build order
- [ ] 35+ tests pass (target: 40+ for thoroughness)
- [ ] All tests use rstest for parameterization
- [ ] No clippy warnings
- [ ] `cargo test -p atlas-package` passes
- [ ] `cargo check -p atlas-package` passes
- [ ] Code follows Rust best practices
- [ ] Error messages are clear and actionable

## Notes

**Phase split rationale:**
- Phase-08a: Resolver core (this phase) - constraint solving, no I/O
- Phase-08b: Registry + downloader - network I/O, caching
- Phase-08c: Integration + advanced features - lockfile, build order, docs

**Why split here:**
- Resolver core is complex algorithm work (PubGrub)
- Can be tested without network I/O
- Clean separation: algorithm vs I/O vs integration

**After completion:**
- Resolver can find compatible versions from in-memory data
- No registry queries yet (phase-08b)
- No package downloading yet (phase-08b)
- No lockfile integration yet (phase-08c)
