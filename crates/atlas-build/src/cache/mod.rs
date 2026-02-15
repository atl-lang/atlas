//! Build cache infrastructure for incremental compilation

pub mod invalidation;
pub mod metadata;

use crate::error::{BuildError, BuildResult};
pub use invalidation::{
    compute_selective_invalidation, compute_transitive_invalidation, InvalidationReason,
    InvalidationSet,
};
pub use metadata::{ChangeDetector, ChangeType, ChangedFile, FileState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Cache format version
const CACHE_VERSION: &str = "1.0";

/// Default cache size limit (1 GB)
const DEFAULT_SIZE_LIMIT: u64 = 1024 * 1024 * 1024;

/// Stale entry threshold (30 days)
const STALE_THRESHOLD_DAYS: u64 = 30;

/// Build cache for incremental compilation
#[derive(Debug)]
pub struct BuildCache {
    cache_dir: PathBuf,
    metadata: CacheMetadata,
    entries: HashMap<String, CacheEntry>,
    size_limit: u64,
}

/// Cache entry for a compiled module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Module path relative to project root
    pub module_path: PathBuf,
    /// SHA-256 hash of source content
    pub source_hash: String,
    /// Last modified timestamp
    pub timestamp: SystemTime,
    /// Compiled bytecode
    pub bytecode: Vec<u8>,
    /// Module dependencies (module names)
    pub dependencies: Vec<String>,
    /// Compilation duration
    pub compile_time: Duration,
    /// Last accessed time (for LRU)
    pub last_accessed: SystemTime,
}

/// Cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Cache format version
    pub version: String,
    /// Atlas compiler version
    pub atlas_version: String,
    /// Cache creation time
    pub created: SystemTime,
    /// Last update time
    pub last_updated: SystemTime,
    /// Total number of entries
    pub total_entries: usize,
    /// Total cache size in bytes
    pub total_size: u64,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_modules: usize,
    pub cached_modules: usize,
    pub recompiled_modules: usize,
    pub cache_hit_rate: f64,
    pub cache_size_bytes: u64,
    pub cache_entries: usize,
    pub time_saved: Duration,
}

impl BuildCache {
    /// Create a new build cache
    pub fn new(cache_dir: PathBuf) -> BuildResult<Self> {
        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir).map_err(|e| {
            BuildError::BuildFailed(format!("Failed to create cache directory: {}", e))
        })?;

        let metadata = CacheMetadata {
            version: CACHE_VERSION.to_string(),
            atlas_version: env!("CARGO_PKG_VERSION").to_string(),
            created: SystemTime::now(),
            last_updated: SystemTime::now(),
            total_entries: 0,
            total_size: 0,
        };

        Ok(Self {
            cache_dir,
            metadata,
            entries: HashMap::new(),
            size_limit: DEFAULT_SIZE_LIMIT,
        })
    }

    /// Load existing cache from disk
    pub fn load(cache_dir: &Path) -> BuildResult<Self> {
        let metadata_path = cache_dir.join("metadata.json");

        if !metadata_path.exists() {
            // No existing cache, create new
            return Self::new(cache_dir.to_path_buf());
        }

        // Load metadata
        let metadata_json = fs::read_to_string(&metadata_path).map_err(|e| {
            BuildError::BuildFailed(format!("Failed to read cache metadata: {}", e))
        })?;

        let metadata: CacheMetadata = serde_json::from_str(&metadata_json).map_err(|e| {
            BuildError::BuildFailed(format!("Failed to parse cache metadata: {}", e))
        })?;

        // Validate cache version
        if metadata.version != CACHE_VERSION {
            // Incompatible cache version, create new
            return Self::new(cache_dir.to_path_buf());
        }

        // Validate Atlas version (for safety, invalidate on compiler upgrade)
        if metadata.atlas_version != env!("CARGO_PKG_VERSION") {
            // Compiler version changed, create new cache
            return Self::new(cache_dir.to_path_buf());
        }

        // Load all cache entries
        let mut entries = HashMap::new();
        let modules_dir = cache_dir.join("modules");

        if modules_dir.exists() {
            for entry_result in fs::read_dir(&modules_dir).map_err(|e| {
                BuildError::BuildFailed(format!("Failed to read cache entries: {}", e))
            })? {
                let entry = entry_result.map_err(|e| {
                    BuildError::BuildFailed(format!("Failed to read cache entry: {}", e))
                })?;

                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let entry_json = fs::read_to_string(&path).map_err(|e| {
                        BuildError::BuildFailed(format!("Failed to read cache entry: {}", e))
                    })?;

                    let cache_entry: CacheEntry =
                        serde_json::from_str(&entry_json).map_err(|e| {
                            BuildError::BuildFailed(format!("Failed to parse cache entry: {}", e))
                        })?;

                    let module_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    entries.insert(module_name, cache_entry);
                }
            }
        }

        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
            metadata,
            entries,
            size_limit: DEFAULT_SIZE_LIMIT,
        })
    }

    /// Store a compiled module in the cache
    pub fn store(
        &mut self,
        module_name: &str,
        module_path: PathBuf,
        source_content: &str,
        bytecode: Vec<u8>,
        dependencies: Vec<String>,
        compile_time: Duration,
    ) -> BuildResult<()> {
        // Compute source hash
        let source_hash = Self::compute_hash(source_content);

        // Get file timestamp
        let timestamp = fs::metadata(&module_path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        let entry = CacheEntry {
            module_path,
            source_hash,
            timestamp,
            bytecode,
            dependencies,
            compile_time,
            last_accessed: SystemTime::now(),
        };

        // Check size limit
        let entry_size = entry.bytecode.len() as u64;
        while self.metadata.total_size + entry_size > self.size_limit && !self.entries.is_empty() {
            self.evict_lru()?;
        }

        // Persist entry to disk
        self.persist_entry(module_name, &entry)?;

        // Update metadata
        self.metadata.total_size += entry_size;
        self.metadata.total_entries = self.entries.len() + 1;
        self.metadata.last_updated = SystemTime::now();

        // Add to cache
        self.entries.insert(module_name.to_string(), entry);

        Ok(())
    }

    /// Retrieve a cached module if valid
    pub fn get(&mut self, module_name: &str, source_path: &Path) -> BuildResult<Option<Vec<u8>>> {
        let entry = match self.entries.get_mut(module_name) {
            Some(e) => e,
            None => return Ok(None), // Cache miss
        };

        // Validate source file exists
        if !source_path.exists() {
            return Ok(None);
        }

        // Validate timestamp
        let current_timestamp = fs::metadata(source_path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        if current_timestamp != entry.timestamp {
            // Timestamp changed, validate hash
            let source_content = fs::read_to_string(source_path).map_err(|e| {
                BuildError::BuildFailed(format!("Failed to read source file: {}", e))
            })?;

            let current_hash = Self::compute_hash(&source_content);
            if current_hash != entry.source_hash {
                // Content actually changed
                return Ok(None);
            }
        }

        // Cache hit - update last accessed time
        entry.last_accessed = SystemTime::now();

        Ok(Some(entry.bytecode.clone()))
    }

    /// Invalidate a cache entry
    pub fn invalidate(&mut self, module_name: &str) -> BuildResult<()> {
        if let Some(entry) = self.entries.remove(module_name) {
            self.metadata.total_size -= entry.bytecode.len() as u64;
            self.metadata.total_entries = self.entries.len();

            // Remove from disk
            let entry_path = self
                .cache_dir
                .join("modules")
                .join(format!("{}.json", module_name));
            if entry_path.exists() {
                fs::remove_file(entry_path).ok();
            }

            let bytecode_path = self
                .cache_dir
                .join("modules")
                .join(format!("{}.bc", module_name));
            if bytecode_path.exists() {
                fs::remove_file(bytecode_path).ok();
            }
        }

        Ok(())
    }

    /// Clear all cache entries
    pub fn clear(&mut self) -> BuildResult<()> {
        self.entries.clear();
        self.metadata.total_size = 0;
        self.metadata.total_entries = 0;

        // Remove all cache files
        let modules_dir = self.cache_dir.join("modules");
        if modules_dir.exists() {
            fs::remove_dir_all(&modules_dir)
                .map_err(|e| BuildError::BuildFailed(format!("Failed to clear cache: {}", e)))?;
        }

        Ok(())
    }

    /// Clean stale cache entries (not accessed in STALE_THRESHOLD_DAYS)
    pub fn clean_stale(&mut self) -> BuildResult<usize> {
        let now = SystemTime::now();
        let threshold = Duration::from_secs(STALE_THRESHOLD_DAYS * 24 * 60 * 60);

        let mut removed = Vec::new();

        for (name, entry) in &self.entries {
            if let Ok(elapsed) = now.duration_since(entry.last_accessed) {
                if elapsed > threshold {
                    removed.push(name.clone());
                }
            }
        }

        for name in &removed {
            self.invalidate(name)?;
        }

        Ok(removed.len())
    }

    /// Save cache to disk
    pub fn save(&self) -> BuildResult<()> {
        // Save metadata
        let metadata_json = serde_json::to_string_pretty(&self.metadata).map_err(|e| {
            BuildError::BuildFailed(format!("Failed to serialize cache metadata: {}", e))
        })?;

        let metadata_path = self.cache_dir.join("metadata.json");
        fs::write(&metadata_path, metadata_json).map_err(|e| {
            BuildError::BuildFailed(format!("Failed to write cache metadata: {}", e))
        })?;

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache_hit_rate = if self.metadata.total_entries > 0 {
            self.entries.len() as f64 / self.metadata.total_entries as f64
        } else {
            0.0
        };

        CacheStats {
            total_modules: self.metadata.total_entries,
            cached_modules: self.entries.len(),
            recompiled_modules: 0, // Updated during build
            cache_hit_rate,
            cache_size_bytes: self.metadata.total_size,
            cache_entries: self.entries.len(),
            time_saved: Duration::from_secs(0), // Updated during build
        }
    }

    /// Compute SHA-256 hash of content
    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Persist cache entry to disk
    fn persist_entry(&self, module_name: &str, entry: &CacheEntry) -> BuildResult<()> {
        let modules_dir = self.cache_dir.join("modules");
        fs::create_dir_all(&modules_dir).map_err(|e| {
            BuildError::BuildFailed(format!("Failed to create modules directory: {}", e))
        })?;

        // Save entry metadata
        let entry_json = serde_json::to_string_pretty(entry).map_err(|e| {
            BuildError::BuildFailed(format!("Failed to serialize cache entry: {}", e))
        })?;

        let entry_path = modules_dir.join(format!("{}.json", module_name));
        fs::write(&entry_path, entry_json)
            .map_err(|e| BuildError::BuildFailed(format!("Failed to write cache entry: {}", e)))?;

        Ok(())
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) -> BuildResult<()> {
        // Find LRU entry
        let lru_name = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(name, _)| name.clone());

        if let Some(name) = lru_name {
            self.invalidate(&name)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compute_hash() {
        let content = "fn main() { print(42); }";
        let hash1 = BuildCache::compute_hash(content);
        let hash2 = BuildCache::compute_hash(content);
        assert_eq!(hash1, hash2);

        let different = "fn main() { print(43); }";
        let hash3 = BuildCache::compute_hash(different);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_new_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = BuildCache::new(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(cache.metadata.version, CACHE_VERSION);
        assert_eq!(cache.metadata.total_entries, 0);
        assert_eq!(cache.metadata.total_size, 0);
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create and save cache
        {
            let cache = BuildCache::new(temp_dir.path().to_path_buf()).unwrap();
            cache.save().unwrap();
        }

        // Load cache
        let loaded = BuildCache::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.metadata.version, CACHE_VERSION);
    }
}
