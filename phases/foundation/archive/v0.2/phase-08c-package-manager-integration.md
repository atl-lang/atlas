# Phase 08c: Package Manager - Integration & Documentation

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Foundation phases 08a and 08b must be complete.

**Verification Steps:**

1. Check STATUS.md: Foundation section, phase-08a and phase-08b should be ✅
   ```bash
   grep "phase-08a-package-manager-resolver-core.md" STATUS.md
   grep "phase-08b-package-manager-registry.md" STATUS.md
   ```

2. Verify resolver and registry modules exist:
   ```bash
   ls crates/atlas-package/src/resolver.rs
   ls crates/atlas-package/src/registry.rs
   ls crates/atlas-package/src/downloader.rs
   ls crates/atlas-package/src/cache.rs
   ```

3. Verify test counts from previous phases:
   ```bash
   # Phase-08a should have 35+ tests
   grep "fn test_" crates/atlas-package/tests/resolver_core_tests.rs | wc -l
   # Phase-08b should have 21+ tests
   grep "fn test_" crates/atlas-package/tests/registry_tests.rs | wc -l
   grep "fn test_" crates/atlas-package/tests/downloader_tests.rs | wc -l
   ```

**Expected from phase-08a:**
- Resolver with PubGrub algorithm
- DependencyGraph and VersionSolver
- 35+ tests

**Expected from phase-08b:**
- Registry trait and implementations
- Downloader with checksum verification
- PackageCache with LRU
- 21+ tests

**Decision Tree:**

a) If phase-08a and 08b complete (STATUS.md ✅, all modules exist):
   → Proceed with phase-08c
   → Integrate resolver + registry + lockfile

b) If either phase incomplete:
   → STOP immediately
   → Report which phase is missing
   → Complete missing phase first

**No user questions needed:** Phase completion is verifiable via STATUS.md and file structure.

---

## Objective

Integrate resolver, registry, and lockfile into complete dependency management system - supporting lockfile-based reproducible builds, conflict resolution strategies, build order computation, and comprehensive documentation. Completes the package manager foundation.

## Files

**Create:** `crates/atlas-package/src/resolver/conflict.rs` (~300 lines)
**Create:** `crates/atlas-package/src/build_order.rs` (~250 lines)
**Update:** `crates/atlas-package/src/resolver.rs` (~200 lines - lockfile integration)
**Create:** `docs/dependency-resolution.md` (~800 lines - comprehensive guide)
**Tests:** `crates/atlas-package/tests/integration_tests.rs` (~600 lines)
**Tests:** `crates/atlas-package/tests/build_order_tests.rs` (~300 lines)

**Total:** ~2450 lines

## Dependencies

**Rust Crates:**
- All dependencies from phase-08a and 08b (already in Cargo.toml)

**Phase Dependencies:**
- Resolver from phase-08a
- Registry, Downloader, Cache from phase-08b
- Lockfile from phase-07
- PackageManifest from phase-07

## Implementation

### 1. Conflict Resolution Strategies

Create `crates/atlas-package/src/resolver/conflict.rs`:

```rust
use super::{ResolverError, ResolverResult};
use semver::{Version, VersionReq};
use std::collections::HashMap;

/// Conflict information for reporting
#[derive(Debug, Clone)]
pub struct Conflict {
    pub package: String,
    pub constraints: Vec<ConflictingConstraint>,
}

#[derive(Debug, Clone)]
pub struct ConflictingConstraint {
    pub requirement: VersionReq,
    pub source: String,  // Which package imposed this constraint
}

impl Conflict {
    /// Generate human-readable conflict report
    pub fn report(&self) -> String {
        let mut report = format!("Version conflict for package '{}':\n", self.package);

        for constraint in &self.constraints {
            report.push_str(&format!(
                "  {} requires {}\n",
                constraint.source, constraint.requirement
            ));
        }

        report.push_str("\nPossible solutions:\n");
        report.push_str("  1. Update dependencies to compatible versions\n");
        report.push_str("  2. Use dependency overrides in atlas.toml\n");
        report.push_str("  3. Check for alternative packages\n");

        report
    }
}

/// Conflict detector and resolver
pub struct ConflictResolver {
    /// Detected conflicts
    conflicts: Vec<Conflict>,
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            conflicts: Vec::new(),
        }
    }

    /// Detect conflicts in constraint set
    pub fn detect_conflicts(
        &mut self,
        constraints: &HashMap<String, Vec<(VersionReq, String)>>,
    ) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        for (package, reqs) in constraints {
            if !self.are_constraints_compatible(reqs) {
                conflicts.push(Conflict {
                    package: package.clone(),
                    constraints: reqs
                        .iter()
                        .map(|(req, source)| ConflictingConstraint {
                            requirement: req.clone(),
                            source: source.clone(),
                        })
                        .collect(),
                });
            }
        }

        self.conflicts = conflicts.clone();
        conflicts
    }

    /// Check if constraints are compatible
    fn are_constraints_compatible(&self, constraints: &[(VersionReq, String)]) -> bool {
        // Try to find any version that satisfies all constraints
        // For now, simplified implementation
        // Full implementation would use version solver

        if constraints.len() <= 1 {
            return true;
        }

        // Check for obvious conflicts (e.g., "^1.0" and "^2.0")
        // Implementation in phase
        true
    }

    /// Suggest resolution strategies
    pub fn suggest_resolutions(&self, conflict: &Conflict) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Analyze conflict and suggest fixes
        suggestions.push(format!(
            "Try updating all dependencies of '{}' to latest compatible versions",
            conflict.package
        ));

        // More sophisticated suggestions in implementation
        suggestions
    }
}
```

### 2. Build Order Computation

Create `crates/atlas-package/src/build_order.rs`:

```rust
use crate::resolver::Resolution;
use thiserror::Error;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Error)]
pub enum BuildOrderError {
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Missing dependency: {0}")]
    MissingDependency(String),
}

/// Build order computer using topological sort
pub struct BuildOrderComputer {
    /// Dependency graph: package -> dependencies
    graph: HashMap<String, Vec<String>>,
}

impl BuildOrderComputer {
    pub fn new(resolution: &Resolution) -> Self {
        let mut graph = HashMap::new();

        for (name, package) in &resolution.packages {
            graph.insert(name.clone(), package.dependencies.clone());
        }

        Self { graph }
    }

    /// Compute topological build order
    pub fn compute_build_order(&self) -> Result<Vec<String>, BuildOrderError> {
        // Kahn's algorithm for topological sorting
        let mut in_degree = self.compute_in_degrees();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Start with packages that have no dependencies
        for (package, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(package.clone());
            }
        }

        while let Some(package) = queue.pop_front() {
            result.push(package.clone());

            // Reduce in-degree for dependents
            if let Some(deps) = self.graph.get(&package) {
                for dep in deps {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.graph.len() {
            return Err(BuildOrderError::CircularDependency(
                "Dependency cycle detected".to_string()
            ));
        }

        Ok(result)
    }

    /// Compute in-degrees for all packages
    fn compute_in_degrees(&self) -> HashMap<String, usize> {
        let mut in_degree = HashMap::new();

        // Initialize all packages with 0
        for package in self.graph.keys() {
            in_degree.insert(package.clone(), 0);
        }

        // Count incoming edges
        for deps in self.graph.values() {
            for dep in deps {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        in_degree
    }

    /// Find packages that can be built in parallel
    pub fn parallel_build_groups(&self) -> Result<Vec<Vec<String>>, BuildOrderError> {
        let order = self.compute_build_order()?;
        let mut groups = Vec::new();
        let mut built = HashSet::new();

        while built.len() < order.len() {
            let mut group = Vec::new();

            for package in &order {
                if built.contains(package) {
                    continue;
                }

                // Check if all dependencies are built
                let deps = self.graph.get(package).unwrap();
                if deps.iter().all(|d| built.contains(d)) {
                    group.push(package.clone());
                }
            }

            if group.is_empty() {
                break;
            }

            for package in &group {
                built.insert(package.clone());
            }

            groups.push(group);
        }

        Ok(groups)
    }
}
```

### 3. Lockfile Integration

Update `crates/atlas-package/src/resolver.rs`:

```rust
// Add to existing Resolver impl

use crate::lockfile::Lockfile;

impl Resolver {
    /// Resolve using existing lockfile if available
    pub fn resolve_with_lockfile(
        &mut self,
        manifest: &PackageManifest,
        lockfile: Option<&Lockfile>,
    ) -> ResolverResult<Resolution> {
        // If lockfile exists and is valid, use it
        if let Some(lock) = lockfile {
            if self.lockfile_is_valid(manifest, lock) {
                return self.resolution_from_lockfile(lock);
            }
        }

        // Otherwise, resolve fresh
        self.resolve(manifest)
    }

    /// Check if lockfile matches manifest
    fn lockfile_is_valid(&self, manifest: &PackageManifest, lockfile: &Lockfile) -> bool {
        // Verify all manifest dependencies are in lockfile
        // Verify versions in lockfile satisfy manifest constraints
        // Implementation in phase
        true
    }

    /// Create resolution from lockfile
    fn resolution_from_lockfile(&self, lockfile: &Lockfile) -> ResolverResult<Resolution> {
        let mut packages = HashMap::new();

        for locked_pkg in &lockfile.packages {
            packages.insert(
                locked_pkg.name.clone(),
                ResolvedPackage {
                    name: locked_pkg.name.clone(),
                    version: locked_pkg.version.clone(),
                    dependencies: vec![], // Extract from lockfile
                },
            );
        }

        Ok(Resolution { packages })
    }

    /// Generate lockfile from resolution
    pub fn generate_lockfile(&self, resolution: &Resolution) -> Lockfile {
        use crate::lockfile::{LockedPackage, PackageSource, LockfileMetadata};

        let mut lockfile = Lockfile::new();

        for (name, package) in &resolution.packages {
            lockfile.add_package(LockedPackage {
                name: name.clone(),
                version: package.version.clone(),
                checksum: String::new(), // Would come from registry
                source: PackageSource::Registry {
                    registry: None,
                },
            });
        }

        lockfile
    }
}
```

### 4. Complete Documentation

Create `docs/dependency-resolution.md`:

```markdown
# Dependency Resolution

Atlas uses a PubGrub-based dependency resolver to find compatible package versions.

## Table of Contents

1. [Overview](#overview)
2. [Resolution Algorithm](#resolution-algorithm)
3. [Version Constraints](#version-constraints)
4. [Conflict Resolution](#conflict-resolution)
5. [Build Order](#build-order)
6. [Lockfile Integration](#lockfile-integration)
7. [Registry System](#registry-system)
8. [Caching](#caching)
9. [Offline Mode](#offline-mode)
10. [Examples](#examples)

## Overview

[Comprehensive documentation content - 800 lines total]
[Cover all aspects: resolver, registry, downloader, cache, lockfile]
[Include diagrams, examples, troubleshooting]
[API reference for all public types]
```

## Tests (TDD - Use rstest)

Create `crates/atlas-package/tests/integration_tests.rs`:

**Integration tests:**
1. `test_full_resolution_pipeline` - Manifest → resolver → lockfile
2. `test_resolve_with_existing_lockfile` - Use cached lockfile
3. `test_lockfile_regeneration_on_manifest_change` - Detect drift
4. `test_resolve_download_and_cache` - Full download flow
5. `test_offline_mode_with_cache` - Cache-only resolution
6. `test_parallel_downloads` - Multiple packages downloaded
7. `test_transitive_dependency_resolution` - Multi-level deps
8. `test_diamond_dependency_deduplication` - Shared deps
9. `test_version_conflict_with_resolution_suggestion` - Conflict reporting
10. `test_dependency_override_in_manifest` - Force versions
11. `test_lockfile_checksum_verification` - Integrity check
12. `test_incremental_resolution` - Add new dep, keep existing

**Conflict resolution tests:**
13. `test_detect_simple_conflict` - Direct version conflict
14. `test_detect_transitive_conflict` - Indirect conflict
15. `test_conflict_report_formatting` - Human-readable errors
16. `test_suggest_conflict_resolutions` - Resolution suggestions
17. `test_no_conflict_compatible_constraints` - Compatible deps

**Build order tests (in `build_order_tests.rs`):**
18. `test_topological_sort_linear` - Sequential deps
19. `test_topological_sort_diamond` - Diamond pattern
20. `test_parallel_build_groups` - Identify parallelizable builds
21. `test_circular_dependency_error` - Cycle detection
22. `test_build_order_empty_graph` - No packages
23. `test_build_order_single_package` - One package

**Lockfile integration tests:**
24. `test_lockfile_valid_for_manifest` - Validation
25. `test_lockfile_invalid_triggers_reresolution` - Stale lockfile
26. `test_generate_lockfile_from_resolution` - Lockfile creation
27. `test_lockfile_preserves_structure` - Format stability

**Minimum test count:** 27+ tests (target: 30+ for comprehensive coverage)

## Integration Points

- **Uses:** Resolver from phase-08a
- **Uses:** Registry, Downloader, Cache from phase-08b
- **Uses:** Lockfile from phase-07
- **Uses:** PackageManifest from phase-07
- **Creates:** Complete package manager system
- **Creates:** Build order computation
- **Creates:** Conflict resolution strategies
- **Output:** Production-ready dependency management
- **Enables:** CLI package manager commands (future phase)

## Acceptance Criteria

- [ ] Conflict detection and reporting implemented
- [ ] ConflictResolver suggests resolutions
- [ ] BuildOrderComputer uses topological sort
- [ ] Parallel build groups identified
- [ ] Circular dependency detection works
- [ ] Lockfile integration complete
- [ ] Resolver uses lockfile when valid
- [ ] Lockfile regenerated on manifest change
- [ ] Full resolution pipeline works (manifest → resolution → lockfile)
- [ ] Offline mode supported (cache-only)
- [ ] `docs/dependency-resolution.md` comprehensive (800+ lines)
- [ ] 27+ tests pass (target: 30+)
- [ ] All tests use rstest where appropriate
- [ ] No clippy warnings
- [ ] `cargo test -p atlas-package` passes (all phase-08a/b/c tests)
- [ ] `cargo check -p atlas-package` passes
- [ ] Code follows Rust best practices
- [ ] Error messages are clear and actionable
- [ ] Documentation includes examples and troubleshooting

## Notes

**Phase-08 Complete:**

After phase-08c, the complete package manager foundation is ready:
- ✅ Dependency resolution (PubGrub algorithm)
- ✅ Registry abstraction (remote, local, git)
- ✅ Package downloading and caching
- ✅ Checksum verification
- ✅ Lockfile integration
- ✅ Conflict resolution
- ✅ Build order computation
- ✅ Comprehensive documentation

**What's Next:**

The package manager core is complete, but CLI integration is deferred:
- CLI/phase-05 will add `atlas add`, `atlas update`, `atlas lock` commands
- Build system (foundation/phase-11) will use resolver for builds

**Total Phase-08 Test Count:**
- Phase-08a: 35+ tests (resolver core)
- Phase-08b: 21+ tests (registry + downloader)
- Phase-08c: 27+ tests (integration)
- **Total: 83+ tests** for complete package manager

**Documentation Complete:**
- `docs/package-manifest.md` (from phase-07) - manifest format
- `docs/dependency-resolution.md` (this phase) - resolver internals
- Together: complete package manager documentation
