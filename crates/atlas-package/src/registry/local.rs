use super::{PackageMetadata, Registry, RegistryError, RegistryResult};
use semver::Version;
use std::fs;
use std::path::PathBuf;

/// Local filesystem registry
pub struct LocalRegistry {
    root: PathBuf,
}

impl LocalRegistry {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get root directory
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get package directory
    fn package_dir(&self, package: &str) -> PathBuf {
        self.root.join(package)
    }

    /// Get version directory
    fn version_dir(&self, package: &str, version: &Version) -> PathBuf {
        self.package_dir(package).join(version.to_string())
    }

    /// Get metadata file path
    fn metadata_path(&self, package: &str, version: &Version) -> PathBuf {
        self.version_dir(package, version).join("metadata.toml")
    }

    /// Get archive file path
    fn archive_path(&self, package: &str, version: &Version) -> PathBuf {
        self.version_dir(package, version).join("package.tar.gz")
    }
}

impl Registry for LocalRegistry {
    fn query_versions(&self, package: &str) -> RegistryResult<Vec<Version>> {
        let package_dir = self.package_dir(package);
        if !package_dir.exists() {
            return Err(RegistryError::PackageNotFound(package.to_string()));
        }

        let mut versions = Vec::new();

        // Read version directories
        for entry in fs::read_dir(&package_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(version) = entry.file_name().to_string_lossy().parse::<Version>() {
                    versions.push(version);
                }
            }
        }

        if versions.is_empty() {
            return Err(RegistryError::PackageNotFound(package.to_string()));
        }

        // Sort versions
        versions.sort();

        Ok(versions)
    }

    fn get_metadata(&self, package: &str, version: &Version) -> RegistryResult<PackageMetadata> {
        let metadata_path = self.metadata_path(package, version);
        if !metadata_path.exists() {
            return Err(RegistryError::PackageNotFound(format!(
                "{}@{}",
                package, version
            )));
        }

        // Read and parse metadata.toml
        let content = fs::read_to_string(&metadata_path)?;
        let metadata_toml: toml::Value =
            toml::from_str(&content).map_err(|e| RegistryError::ParseError(e.to_string()))?;

        // Extract metadata fields
        let name = metadata_toml
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(package)
            .to_string();

        let checksum = metadata_toml
            .get("checksum")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let download_url = format!("file://{}", self.archive_path(package, version).display());

        Ok(PackageMetadata::new(name, version.clone())
            .with_checksum(checksum)
            .with_download_url(download_url))
    }

    fn download(&self, package: &str, version: &Version) -> RegistryResult<Vec<u8>> {
        let archive_path = self.archive_path(package, version);
        if !archive_path.exists() {
            return Err(RegistryError::PackageNotFound(format!(
                "{}@{}",
                package, version
            )));
        }

        fs::read(&archive_path).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_registry_new() {
        let registry = LocalRegistry::new(PathBuf::from("/tmp/registry"));
        assert_eq!(registry.root(), &PathBuf::from("/tmp/registry"));
    }

    #[test]
    fn test_package_dir() {
        let registry = LocalRegistry::new(PathBuf::from("/tmp/registry"));
        let package_dir = registry.package_dir("test-package");
        assert_eq!(package_dir, PathBuf::from("/tmp/registry/test-package"));
    }

    #[test]
    fn test_version_dir() {
        let registry = LocalRegistry::new(PathBuf::from("/tmp/registry"));
        let version_dir = registry.version_dir("test-package", &Version::new(1, 0, 0));
        assert_eq!(
            version_dir,
            PathBuf::from("/tmp/registry/test-package/1.0.0")
        );
    }

    #[test]
    fn test_metadata_path() {
        let registry = LocalRegistry::new(PathBuf::from("/tmp/registry"));
        let metadata_path = registry.metadata_path("test-package", &Version::new(1, 0, 0));
        assert_eq!(
            metadata_path,
            PathBuf::from("/tmp/registry/test-package/1.0.0/metadata.toml")
        );
    }

    #[test]
    fn test_archive_path() {
        let registry = LocalRegistry::new(PathBuf::from("/tmp/registry"));
        let archive_path = registry.archive_path("test-package", &Version::new(1, 0, 0));
        assert_eq!(
            archive_path,
            PathBuf::from("/tmp/registry/test-package/1.0.0/package.tar.gz")
        );
    }
}
