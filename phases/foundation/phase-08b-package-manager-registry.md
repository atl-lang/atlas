# Phase 08b: Package Manager - Registry & Downloader

## 🚨 DEPENDENCIES - CHECK BEFORE STARTING

**REQUIRED:** Foundation phase-08a (Resolver Core) must be complete.

**Verification Steps:**

1. Check STATUS.md: Foundation section, phase-08a should be ✅
   ```bash
   grep "phase-08a-package-manager-resolver-core.md" STATUS.md
   ```

2. Verify resolver module exists:
   ```bash
   ls crates/atlas-package/src/resolver.rs
   grep -n "pub struct Resolver" crates/atlas-package/src/resolver.rs
   ```

3. Verify resolver tests pass:
   ```bash
   ls crates/atlas-package/tests/resolver_core_tests.rs
   grep "fn test_" crates/atlas-package/tests/resolver_core_tests.rs | wc -l
   ```
   Should show 35+ test functions.

4. Verify pubgrub dependency exists:
   ```bash
   grep "pubgrub" crates/atlas-package/Cargo.toml
   ```

**Expected from phase-08a:**
- Resolver struct with resolve() method
- DependencyGraph implementation
- VersionSolver with PubGrub integration
- 35+ tests passing
- pubgrub in Cargo.toml

**Decision Tree:**

a) If phase-08a complete (STATUS.md ✅, resolver exists, tests pass):
   → Proceed with phase-08b
   → Implement registry and downloader

b) If phase-08a incomplete:
   → STOP immediately
   → Report: "Foundation phase-08a required before phase-08b"
   → Complete phase-08a first

c) If pubgrub dependency missing:
   → ERROR: Phase-08a incomplete
   → Verify phase-08a acceptance criteria met

**No user questions needed:** Phase-08a completion is verifiable via STATUS.md and file structure.

---

## Objective

Implement registry abstraction for querying package metadata and downloading packages from remote/local sources, with caching, checksum verification, and offline mode support. Enables actual package retrieval for resolved dependencies.

## Files

**Create:** `crates/atlas-package/src/registry.rs` (~600 lines)
**Create:** `crates/atlas-package/src/registry/remote.rs` (~300 lines)
**Create:** `crates/atlas-package/src/registry/local.rs` (~150 lines)
**Create:** `crates/atlas-package/src/downloader.rs` (~400 lines)
**Create:** `crates/atlas-package/src/cache.rs` (~350 lines)
**Update:** `crates/atlas-package/src/lib.rs` (~30 lines - exports)
**Update:** `crates/atlas-package/Cargo.toml` (~20 lines - add reqwest, tar, flate2)
**Tests:** `crates/atlas-package/tests/registry_tests.rs` (~500 lines)
**Tests:** `crates/atlas-package/tests/downloader_tests.rs` (~300 lines)

**Total:** ~2650 lines

## Dependencies

**Rust Crates to ADD:**
- `reqwest` (blocking feature) - HTTP client for downloads
- `tar` - TAR archive extraction
- `flate2` - GZIP compression/decompression
- `sha2` - SHA256 checksums

**Already in Cargo.toml:**
- `semver`, `serde`, `toml`, `thiserror`

**Phase Dependencies:**
- Resolver from phase-08a
- PackageManifest from phase-07

## Implementation

### 1. Add Network Dependencies

Update `crates/atlas-package/Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
semver = { version = "1.0", features = ["serde"] }
thiserror = "2.0"
chrono = "0.4"
pubgrub = "0.2"
# ADD THESE:
reqwest = { version = "0.12", features = ["blocking"] }
tar = "0.4"
flate2 = "1.0"
sha2 = "0.10"
```

### 2. Registry Abstraction

Create `crates/atlas-package/src/registry.rs`:

```rust
use semver::Version;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Checksum mismatch for {package}@{version}")]
    ChecksumMismatch { package: String, version: String },

    #[error("Registry unavailable: {0}")]
    Unavailable(String),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

/// Package metadata from registry
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: Version,
    pub checksum: String,
    pub download_url: String,
    pub dependencies: Vec<String>,
}

/// Registry trait - implemented by remote, local, git registries
pub trait Registry {
    /// Query available versions for a package
    fn query_versions(&self, package: &str) -> RegistryResult<Vec<Version>>;

    /// Get package metadata for specific version
    fn get_metadata(&self, package: &str, version: &Version) -> RegistryResult<PackageMetadata>;

    /// Download package archive
    fn download(&self, package: &str, version: &Version) -> RegistryResult<Vec<u8>>;
}

/// Registry manager - handles multiple registry sources
pub struct RegistryManager {
    registries: Vec<Box<dyn Registry>>,
    cache_enabled: bool,
}

impl RegistryManager {
    pub fn new() -> Self {
        Self {
            registries: Vec::new(),
            cache_enabled: true,
        }
    }

    /// Add registry source
    pub fn add_registry(&mut self, registry: Box<dyn Registry>) {
        self.registries.push(registry);
    }

    /// Query all registries for package versions
    pub fn query_versions(&self, package: &str) -> RegistryResult<Vec<Version>> {
        for registry in &self.registries {
            if let Ok(versions) = registry.query_versions(package) {
                return Ok(versions);
            }
        }
        Err(RegistryError::PackageNotFound(package.to_string()))
    }

    /// Get metadata from first registry that has it
    pub fn get_metadata(&self, package: &str, version: &Version) -> RegistryResult<PackageMetadata> {
        for registry in &self.registries {
            if let Ok(metadata) = registry.get_metadata(package, version) {
                return Ok(metadata);
            }
        }
        Err(RegistryError::PackageNotFound(format!("{}@{}", package, version)))
    }
}
```

### 3. Remote Registry

Create `crates/atlas-package/src/registry/remote.rs`:

```rust
use super::{Registry, RegistryResult, RegistryError, PackageMetadata};
use semver::Version;
use reqwest::blocking::Client;
use std::time::Duration;

/// Remote HTTP registry
pub struct RemoteRegistry {
    base_url: String,
    client: Client,
}

impl RemoteRegistry {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { base_url, client }
    }
}

impl Registry for RemoteRegistry {
    fn query_versions(&self, package: &str) -> RegistryResult<Vec<Version>> {
        let url = format!("{}/packages/{}/versions", self.base_url, package);
        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::PackageNotFound(package.to_string()));
        }

        // Parse version list from JSON response
        // Implementation in phase
        todo!()
    }

    fn get_metadata(&self, package: &str, version: &Version) -> RegistryResult<PackageMetadata> {
        let url = format!("{}/packages/{}/{}", self.base_url, package, version);
        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::PackageNotFound(format!("{}@{}", package, version)));
        }

        // Parse metadata from JSON response
        // Implementation in phase
        todo!()
    }

    fn download(&self, package: &str, version: &Version) -> RegistryResult<Vec<u8>> {
        let metadata = self.get_metadata(package, version)?;
        let response = self.client
            .get(&metadata.download_url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::NetworkError("Download failed".to_string()));
        }

        let bytes = response.bytes()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(bytes)
    }
}
```

### 4. Local Registry (for testing/offline)

Create `crates/atlas-package/src/registry/local.rs`:

```rust
use super::{Registry, RegistryResult, RegistryError, PackageMetadata};
use semver::Version;
use std::path::PathBuf;
use std::fs;

/// Local filesystem registry
pub struct LocalRegistry {
    root: PathBuf,
}

impl LocalRegistry {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl Registry for LocalRegistry {
    fn query_versions(&self, package: &str) -> RegistryResult<Vec<Version>> {
        let package_dir = self.root.join(package);
        if !package_dir.exists() {
            return Err(RegistryError::PackageNotFound(package.to_string()));
        }

        // Read versions from directory structure
        // Implementation in phase
        todo!()
    }

    fn get_metadata(&self, package: &str, version: &Version) -> RegistryResult<PackageMetadata> {
        let metadata_path = self.root.join(package).join(version.to_string()).join("metadata.toml");
        if !metadata_path.exists() {
            return Err(RegistryError::PackageNotFound(format!("{}@{}", package, version)));
        }

        // Read and parse metadata.toml
        // Implementation in phase
        todo!()
    }

    fn download(&self, package: &str, version: &Version) -> RegistryResult<Vec<u8>> {
        let archive_path = self.root.join(package).join(version.to_string()).join("package.tar.gz");
        fs::read(&archive_path)
            .map_err(|_| RegistryError::PackageNotFound(format!("{}@{}", package, version)))
    }
}
```

### 5. Package Downloader

Create `crates/atlas-package/src/downloader.rs`:

```rust
use crate::registry::{Registry, RegistryResult, RegistryError};
use semver::Version;
use sha2::{Sha256, Digest};
use tar::Archive;
use flate2::read::GzDecoder;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;

/// Package downloader with checksum verification
pub struct Downloader {
    cache_dir: PathBuf,
}

impl Downloader {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Download and extract package
    pub fn download_package(
        &self,
        registry: &dyn Registry,
        package: &str,
        version: &Version,
    ) -> RegistryResult<PathBuf> {
        // Get metadata for checksum
        let metadata = registry.get_metadata(package, version)?;

        // Download archive
        let archive_bytes = registry.download(package, version)?;

        // Verify checksum
        self.verify_checksum(&archive_bytes, &metadata.checksum)?;

        // Extract to cache
        let extract_path = self.cache_dir.join(package).join(version.to_string());
        self.extract_archive(&archive_bytes, &extract_path)?;

        Ok(extract_path)
    }

    /// Verify SHA256 checksum
    fn verify_checksum(&self, data: &[u8], expected: &str) -> RegistryResult<()> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = format!("{:x}", hasher.finalize());

        if result != expected {
            return Err(RegistryError::ChecksumMismatch {
                package: "unknown".to_string(),
                version: "unknown".to_string(),
            });
        }

        Ok(())
    }

    /// Extract tar.gz archive
    fn extract_archive(&self, data: &[u8], dest: &Path) -> RegistryResult<()> {
        fs::create_dir_all(dest)
            .map_err(|e| RegistryError::NetworkError(format!("Create dir failed: {}", e)))?;

        let decoder = GzDecoder::new(data);
        let mut archive = Archive::new(decoder);

        archive.unpack(dest)
            .map_err(|e| RegistryError::NetworkError(format!("Extract failed: {}", e)))?;

        Ok(())
    }
}
```

### 6. Package Cache

Create `crates/atlas-package/src/cache.rs`:

```rust
use semver::Version;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

/// Package cache with LRU eviction
pub struct PackageCache {
    cache_dir: PathBuf,
    max_size_mb: usize,
    /// LRU tracking: package -> last access time
    access_times: HashMap<String, std::time::SystemTime>,
}

impl PackageCache {
    pub fn new(cache_dir: PathBuf, max_size_mb: usize) -> Self {
        fs::create_dir_all(&cache_dir).ok();
        Self {
            cache_dir,
            max_size_mb,
            access_times: HashMap::new(),
        }
    }

    /// Check if package is cached
    pub fn has_package(&self, package: &str, version: &Version) -> bool {
        self.package_path(package, version).exists()
    }

    /// Get cached package path
    pub fn get_package(&mut self, package: &str, version: &Version) -> Option<PathBuf> {
        let path = self.package_path(package, version);
        if path.exists() {
            // Update LRU
            let key = format!("{}@{}", package, version);
            self.access_times.insert(key, std::time::SystemTime::now());
            Some(path)
        } else {
            None
        }
    }

    /// Add package to cache
    pub fn add_package(&mut self, package: &str, version: &Version, path: PathBuf) {
        let key = format!("{}@{}", package, version);
        self.access_times.insert(key, std::time::SystemTime::now());

        // Check cache size and evict if needed
        self.enforce_cache_limit();
    }

    /// Get package path in cache
    fn package_path(&self, package: &str, version: &Version) -> PathBuf {
        self.cache_dir.join(package).join(version.to_string())
    }

    /// Calculate cache size in MB
    fn cache_size_mb(&self) -> usize {
        // Walk cache_dir and sum file sizes
        // Implementation in phase
        0
    }

    /// Evict oldest packages until under size limit
    fn enforce_cache_limit(&mut self) {
        while self.cache_size_mb() > self.max_size_mb {
            if let Some(oldest) = self.find_oldest_package() {
                self.evict_package(&oldest);
            } else {
                break;
            }
        }
    }

    /// Find least recently used package
    fn find_oldest_package(&self) -> Option<String> {
        self.access_times
            .iter()
            .min_by_key(|(_, time)| *time)
            .map(|(pkg, _)| pkg.clone())
    }

    /// Remove package from cache
    fn evict_package(&mut self, package_key: &str) {
        // Remove from access_times
        self.access_times.remove(package_key);

        // Delete from filesystem
        // Implementation in phase
    }

    /// Clear entire cache
    pub fn clear(&mut self) {
        fs::remove_dir_all(&self.cache_dir).ok();
        fs::create_dir_all(&self.cache_dir).ok();
        self.access_times.clear();
    }
}
```

### 7. Update lib.rs

Update `crates/atlas-package/src/lib.rs`:

```rust
mod manifest;
mod lockfile;
mod validator;
pub mod resolver;
pub mod registry;     // ADD THIS
pub mod downloader;   // ADD THIS
pub mod cache;        // ADD THIS

pub use manifest::*;
pub use lockfile::*;
pub use validator::*;
pub use resolver::{Resolver, Resolution, ResolvedPackage, ResolverError};
pub use registry::{Registry, RegistryManager, RegistryError, PackageMetadata};  // ADD THIS
pub use downloader::Downloader;  // ADD THIS
pub use cache::PackageCache;     // ADD THIS
```

## Tests (TDD - Use rstest)

Create `crates/atlas-package/tests/registry_tests.rs`:

**Registry tests:**
1. `test_local_registry_query_versions` - List versions from filesystem
2. `test_local_registry_get_metadata` - Read metadata.toml
3. `test_local_registry_download` - Read package.tar.gz
4. `test_local_registry_package_not_found` - Missing package error
5. `test_registry_manager_multiple_sources` - Fallback between registries
6. `test_registry_manager_first_match_wins` - Priority order
7. `test_remote_registry_network_error` - HTTP error handling
8. `test_remote_registry_timeout` - Request timeout

**Downloader tests (in `downloader_tests.rs`):**
9. `test_download_and_extract_package` - Full download flow
10. `test_checksum_verification_success` - Valid checksum
11. `test_checksum_verification_failure` - Invalid checksum
12. `test_extract_tar_gz_archive` - Archive extraction
13. `test_download_to_cache_directory` - Correct paths

**Cache tests:**
14. `test_cache_has_package` - Check presence
15. `test_cache_get_package` - Retrieve cached
16. `test_cache_add_package` - Add new package
17. `test_cache_lru_eviction` - LRU policy
18. `test_cache_size_limit_enforced` - Max size respected
19. `test_cache_clear` - Clear all
20. `test_cache_oldest_package_found` - LRU tracking
21. `test_offline_mode_uses_cache` - Cache-only operation

**Minimum test count:** 21+ tests (target: 25+ for edge cases)

## Integration Points

- **Uses:** Resolver from phase-08a
- **Uses:** PackageManifest from phase-07
- **Creates:** Registry abstraction
- **Creates:** RemoteRegistry, LocalRegistry
- **Creates:** Downloader with checksum verification
- **Creates:** PackageCache with LRU eviction
- **Output:** Complete package retrieval system
- **Next:** Phase-08c will integrate with lockfile and build order

## Acceptance Criteria

- [ ] reqwest, tar, flate2, sha2 dependencies added to Cargo.toml
- [ ] Registry trait implemented
- [ ] RemoteRegistry supports HTTP downloads
- [ ] LocalRegistry supports filesystem packages
- [ ] RegistryManager handles multiple sources
- [ ] Downloader verifies checksums (SHA256)
- [ ] Downloader extracts tar.gz archives
- [ ] PackageCache implements LRU eviction
- [ ] PackageCache enforces size limits
- [ ] Cache supports offline mode
- [ ] Network errors handled gracefully
- [ ] 21+ tests pass (target: 25+)
- [ ] All tests use rstest where appropriate
- [ ] No clippy warnings
- [ ] `cargo test -p atlas-package` passes
- [ ] `cargo check -p atlas-package` passes
- [ ] Code follows Rust best practices
- [ ] Error messages are clear and actionable

## Notes

**Phase split rationale:**
- Phase-08a: Resolver core - algorithm work
- Phase-08b: Registry + downloader (this phase) - network I/O
- Phase-08c: Integration - tie everything together

**After completion:**
- Packages can be downloaded from registries
- Checksums verified for integrity
- Cache reduces redundant downloads
- Offline mode supported
- Still need: lockfile integration, build order (phase-08c)

**Testing note:**
- Use mock HTTP servers for remote registry tests (or local registry for simplicity)
- Use tempfile for cache tests
- Focus on error handling (network failures, checksum mismatches)
