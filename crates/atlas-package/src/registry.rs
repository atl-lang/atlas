use semver::Version;
use thiserror::Error;

pub mod local;
pub mod remote;

pub use local::LocalRegistry;
pub use remote::RemoteRegistry;

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

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

/// Package metadata from registry
#[derive(Debug, Clone, PartialEq)]
pub struct PackageMetadata {
    pub name: String,
    pub version: Version,
    pub checksum: String,
    pub download_url: String,
    pub dependencies: Vec<String>,
}

impl PackageMetadata {
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name,
            version,
            checksum: String::new(),
            download_url: String::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = checksum;
        self
    }

    pub fn with_download_url(mut self, url: String) -> Self {
        self.download_url = url;
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }
}

/// Registry trait - implemented by remote, local, git registries
pub trait Registry: Send + Sync {
    /// Query available versions for a package
    fn query_versions(&self, package: &str) -> RegistryResult<Vec<Version>>;

    /// Get package metadata for specific version
    fn get_metadata(&self, package: &str, version: &Version) -> RegistryResult<PackageMetadata>;

    /// Download package archive
    fn download(&self, package: &str, version: &Version) -> RegistryResult<Vec<u8>>;
}

/// Registry manager - handles multiple registry sources
#[derive(Default)]
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

    /// Enable or disable caching
    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
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
    pub fn get_metadata(
        &self,
        package: &str,
        version: &Version,
    ) -> RegistryResult<PackageMetadata> {
        for registry in &self.registries {
            if let Ok(metadata) = registry.get_metadata(package, version) {
                return Ok(metadata);
            }
        }
        Err(RegistryError::PackageNotFound(format!(
            "{}@{}",
            package, version
        )))
    }

    /// Download from first registry that has the package
    pub fn download(&self, package: &str, version: &Version) -> RegistryResult<Vec<u8>> {
        for registry in &self.registries {
            if let Ok(data) = registry.download(package, version) {
                return Ok(data);
            }
        }
        Err(RegistryError::PackageNotFound(format!(
            "{}@{}",
            package, version
        )))
    }

    /// Get number of registered registries
    pub fn registry_count(&self) -> usize {
        self.registries.len()
    }

    /// Check if cache is enabled
    pub fn is_cache_enabled(&self) -> bool {
        self.cache_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_manager_new() {
        let manager = RegistryManager::new();
        assert_eq!(manager.registry_count(), 0);
        assert!(manager.is_cache_enabled());
    }

    #[test]
    fn test_package_metadata_new() {
        let metadata = PackageMetadata::new("test".to_string(), Version::new(1, 0, 0));
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, Version::new(1, 0, 0));
        assert_eq!(metadata.checksum, "");
    }

    #[test]
    fn test_package_metadata_builder() {
        let metadata = PackageMetadata::new("test".to_string(), Version::new(1, 0, 0))
            .with_checksum("abc123".to_string())
            .with_download_url("https://example.com".to_string())
            .with_dependencies(vec!["dep1".to_string()]);

        assert_eq!(metadata.checksum, "abc123");
        assert_eq!(metadata.download_url, "https://example.com");
        assert_eq!(metadata.dependencies.len(), 1);
    }

    #[test]
    fn test_registry_manager_cache_control() {
        let mut manager = RegistryManager::new();
        assert!(manager.is_cache_enabled());

        manager.set_cache_enabled(false);
        assert!(!manager.is_cache_enabled());

        manager.set_cache_enabled(true);
        assert!(manager.is_cache_enabled());
    }
}
