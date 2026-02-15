use semver::Version;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Package cache with LRU eviction
pub struct PackageCache {
    cache_dir: PathBuf,
    max_size_mb: usize,
    /// LRU tracking: package@version -> last access time
    access_times: HashMap<String, SystemTime>,
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

    /// Get cached package path and update LRU
    pub fn get_package(&mut self, package: &str, version: &Version) -> Option<PathBuf> {
        let path = self.package_path(package, version);
        if path.exists() {
            // Update LRU
            let key = format!("{}@{}", package, version);
            self.access_times.insert(key, SystemTime::now());
            Some(path)
        } else {
            None
        }
    }

    /// Add package to cache
    pub fn add_package(&mut self, package: &str, version: &Version, _path: PathBuf) {
        let key = format!("{}@{}", package, version);
        self.access_times.insert(key, SystemTime::now());

        // Check cache size and evict if needed
        self.enforce_cache_limit();
    }

    /// Get package path in cache
    fn package_path(&self, package: &str, version: &Version) -> PathBuf {
        self.cache_dir.join(package).join(version.to_string())
    }

    /// Calculate cache size in MB
    fn cache_size_mb(&self) -> usize {
        let total_bytes = self.calculate_dir_size(&self.cache_dir);
        total_bytes / (1024 * 1024)
    }

    /// Calculate directory size recursively
    fn calculate_dir_size(&self, dir: &PathBuf) -> usize {
        let mut total = 0;

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total += metadata.len() as usize;
                    } else if metadata.is_dir() {
                        total += self.calculate_dir_size(&entry.path());
                    }
                }
            }
        }

        total
    }

    /// Evict oldest packages until under size limit
    fn enforce_cache_limit(&mut self) {
        while self.cache_size_mb() > self.max_size_mb && !self.access_times.is_empty() {
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

        // Parse package@version
        if let Some((package, version_str)) = package_key.split_once('@') {
            if let Ok(version) = version_str.parse::<Version>() {
                let path = self.package_path(package, &version);
                // Delete from filesystem
                fs::remove_dir_all(&path).ok();
            }
        }
    }

    /// Clear entire cache
    pub fn clear(&mut self) {
        fs::remove_dir_all(&self.cache_dir).ok();
        fs::create_dir_all(&self.cache_dir).ok();
        self.access_times.clear();
    }

    /// Get number of cached packages
    pub fn package_count(&self) -> usize {
        self.access_times.len()
    }

    /// Get current cache size in MB
    pub fn current_size_mb(&self) -> usize {
        self.cache_size_mb()
    }

    /// Get max cache size in MB
    pub fn max_size_mb(&self) -> usize {
        self.max_size_mb
    }

    /// Get cache directory
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_new() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);
        assert_eq!(cache.package_count(), 0);
        assert_eq!(cache.max_size_mb(), 100);
    }

    #[test]
    fn test_cache_has_package() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);
        assert!(!cache.has_package("test", &Version::new(1, 0, 0)));
    }

    #[test]
    fn test_cache_add_package() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);

        cache.add_package("test", &Version::new(1, 0, 0), PathBuf::new());
        assert_eq!(cache.package_count(), 1);
    }

    #[test]
    fn test_cache_get_package_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);

        let result = cache.get_package("test", &Version::new(1, 0, 0));
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);

        cache.add_package("test", &Version::new(1, 0, 0), PathBuf::new());
        assert_eq!(cache.package_count(), 1);

        cache.clear();
        assert_eq!(cache.package_count(), 0);
    }

    #[test]
    fn test_package_path() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);

        let path = cache.package_path("test", &Version::new(1, 0, 0));
        assert_eq!(path, temp_dir.path().join("test/1.0.0"));
    }

    #[test]
    fn test_find_oldest_package() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);

        cache.add_package("test1", &Version::new(1, 0, 0), PathBuf::new());
        std::thread::sleep(std::time::Duration::from_millis(10));
        cache.add_package("test2", &Version::new(1, 0, 0), PathBuf::new());

        let oldest = cache.find_oldest_package();
        assert!(oldest.is_some());
        assert_eq!(oldest.unwrap(), "test1@1.0.0");
    }

    #[test]
    fn test_cache_dir() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);
        assert_eq!(cache.cache_dir(), temp_dir.path());
    }
}
