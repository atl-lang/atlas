use super::{PackageMetadata, Registry, RegistryError, RegistryResult};
use reqwest::blocking::Client;
use semver::Version;
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

    /// Create with custom timeout
    pub fn with_timeout(base_url: String, timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { base_url, client }
    }

    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Registry for RemoteRegistry {
    fn query_versions(&self, package: &str) -> RegistryResult<Vec<Version>> {
        let url = format!("{}/packages/{}/versions", self.base_url, package);
        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::PackageNotFound(package.to_string()));
        }

        // For now, return mock versions
        // In a real implementation, this would parse JSON response
        Ok(vec![
            Version::new(1, 0, 0),
            Version::new(1, 1, 0),
            Version::new(2, 0, 0),
        ])
    }

    fn get_metadata(&self, package: &str, version: &Version) -> RegistryResult<PackageMetadata> {
        let url = format!("{}/packages/{}/{}", self.base_url, package, version);
        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::PackageNotFound(format!(
                "{}@{}",
                package, version
            )));
        }

        // For now, return mock metadata
        // In a real implementation, this would parse JSON response
        Ok(PackageMetadata::new(package.to_string(), version.clone())
            .with_checksum("mock_checksum".to_string())
            .with_download_url(format!(
                "{}/downloads/{}/{}",
                self.base_url, package, version
            )))
    }

    fn download(&self, package: &str, version: &Version) -> RegistryResult<Vec<u8>> {
        let metadata = self.get_metadata(package, version)?;
        let response = self
            .client
            .get(&metadata.download_url)
            .send()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::NetworkError("Download failed".to_string()));
        }

        let bytes = response
            .bytes()
            .map_err(|e| RegistryError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_registry_new() {
        let registry = RemoteRegistry::new("https://registry.example.com".to_string());
        assert_eq!(registry.base_url(), "https://registry.example.com");
    }

    #[test]
    fn test_remote_registry_with_timeout() {
        let registry = RemoteRegistry::with_timeout("https://registry.example.com".to_string(), 10);
        assert_eq!(registry.base_url(), "https://registry.example.com");
    }
}
