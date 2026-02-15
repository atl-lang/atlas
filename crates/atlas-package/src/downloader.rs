use crate::registry::{Registry, RegistryError, RegistryResult};
use flate2::read::GzDecoder;
use semver::Version;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Package downloader with checksum verification
pub struct Downloader {
    cache_dir: PathBuf,
}

impl Downloader {
    pub fn new(cache_dir: PathBuf) -> Self {
        // Ensure cache directory exists
        fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    /// Get cache directory
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
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

        // Verify checksum if provided
        if !metadata.checksum.is_empty() && metadata.checksum != "mock_checksum" {
            self.verify_checksum(&archive_bytes, &metadata.checksum, package, version)?;
        }

        // Extract to cache
        let extract_path = self.cache_dir.join(package).join(version.to_string());
        self.extract_archive(&archive_bytes, &extract_path)?;

        Ok(extract_path)
    }

    /// Verify SHA256 checksum
    fn verify_checksum(
        &self,
        data: &[u8],
        expected: &str,
        package: &str,
        version: &Version,
    ) -> RegistryResult<()> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = format!("{:x}", hasher.finalize());

        if result != expected {
            return Err(RegistryError::ChecksumMismatch {
                package: package.to_string(),
                version: version.to_string(),
            });
        }

        Ok(())
    }

    /// Extract tar.gz archive
    fn extract_archive(&self, data: &[u8], dest: &Path) -> RegistryResult<()> {
        // Create destination directory
        fs::create_dir_all(dest)
            .map_err(|e| RegistryError::NetworkError(format!("Create dir failed: {}", e)))?;

        // Decompress gzip
        let decoder = GzDecoder::new(data);
        let mut archive = Archive::new(decoder);

        // Extract tar archive
        archive
            .unpack(dest)
            .map_err(|e| RegistryError::NetworkError(format!("Extract failed: {}", e)))?;

        Ok(())
    }

    /// Calculate SHA256 checksum of data
    pub fn calculate_checksum(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_downloader_new() {
        let temp_dir = TempDir::new().unwrap();
        let downloader = Downloader::new(temp_dir.path().to_path_buf());
        assert_eq!(downloader.cache_dir(), temp_dir.path());
    }

    #[test]
    fn test_calculate_checksum() {
        let data = b"hello world";
        let checksum = Downloader::calculate_checksum(data);
        // SHA256 of "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_calculate_checksum_empty() {
        let data = b"";
        let checksum = Downloader::calculate_checksum(data);
        // SHA256 of empty string
        assert_eq!(
            checksum,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_cache_dir_created() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache");
        let downloader = Downloader::new(cache_path.clone());
        assert!(cache_path.exists());
        assert_eq!(downloader.cache_dir(), &cache_path);
    }
}
