//! Code cache management
//!
//! Caches compiled native functions keyed by their bytecode offset.
//! Supports size limits, eviction of cold code, and versioning for
//! invalidation when bytecode changes.

use std::collections::HashMap;

/// Cached entry for a compiled function
#[derive(Debug)]
pub struct CacheEntry {
    /// Native code pointer
    pub code_ptr: *const u8,
    /// Size of native code in bytes
    pub code_size: usize,
    /// Cache version (for invalidation)
    pub version: u64,
    /// Number of times this entry was used
    pub hit_count: u64,
    /// Parameter count of the function
    pub param_count: usize,
}

// Safety: code pointers are read-only after compilation
unsafe impl Send for CacheEntry {}
unsafe impl Sync for CacheEntry {}

/// Code cache for JIT-compiled functions
#[derive(Debug)]
pub struct CodeCache {
    /// Cached functions keyed by bytecode offset
    entries: HashMap<usize, CacheEntry>,
    /// Current cache version (incremented on invalidation)
    version: u64,
    /// Total bytes of cached native code
    total_bytes: usize,
    /// Maximum bytes allowed
    max_bytes: usize,
    /// Total cache hits
    total_hits: u64,
    /// Total cache misses
    total_misses: u64,
}

impl CodeCache {
    /// Create a new code cache with the given size limit
    pub fn new(max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            version: 0,
            total_bytes: 0,
            max_bytes,
            total_hits: 0,
            total_misses: 0,
        }
    }

    /// Look up a compiled function by bytecode offset
    pub fn get(&mut self, offset: usize) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get_mut(&offset) {
            if entry.version == self.version {
                entry.hit_count += 1;
                self.total_hits += 1;
                // Re-borrow immutably
                return self.entries.get(&offset);
            }
        }
        self.total_misses += 1;
        None
    }

    /// Check if an offset is cached (without counting as hit/miss)
    pub fn contains(&self, offset: usize) -> bool {
        self.entries
            .get(&offset)
            .map(|e| e.version == self.version)
            .unwrap_or(false)
    }

    /// Insert a compiled function into the cache
    pub fn insert(
        &mut self,
        offset: usize,
        code_ptr: *const u8,
        code_size: usize,
        param_count: usize,
    ) -> Result<(), CacheFullError> {
        // Check if we need to evict
        if self.total_bytes + code_size > self.max_bytes {
            self.evict_cold(code_size)?;
        }

        self.entries.insert(
            offset,
            CacheEntry {
                code_ptr,
                code_size,
                version: self.version,
                hit_count: 0,
                param_count,
            },
        );
        self.total_bytes += code_size;
        Ok(())
    }

    /// Invalidate all cached entries (bump version)
    pub fn invalidate_all(&mut self) {
        self.version += 1;
    }

    /// Invalidate a specific entry
    pub fn invalidate(&mut self, offset: usize) {
        if let Some(entry) = self.entries.remove(&offset) {
            self.total_bytes = self.total_bytes.saturating_sub(entry.code_size);
        }
    }

    /// Evict cold (least-used) entries to make room for `needed` bytes
    fn evict_cold(&mut self, needed: usize) -> Result<(), CacheFullError> {
        // Sort entries by hit count (ascending) and evict coldest first
        let mut offsets: Vec<(usize, u64)> = self
            .entries
            .iter()
            .map(|(&offset, entry)| (offset, entry.hit_count))
            .collect();
        offsets.sort_by_key(|&(_, hits)| hits);

        let mut freed = 0usize;
        for (offset, _) in offsets {
            if self.total_bytes.saturating_sub(freed) + needed <= self.max_bytes {
                break;
            }
            if let Some(entry) = self.entries.remove(&offset) {
                freed += entry.code_size;
            }
        }

        self.total_bytes = self.total_bytes.saturating_sub(freed);

        if self.total_bytes + needed > self.max_bytes {
            return Err(CacheFullError {
                limit: self.max_bytes,
                used: self.total_bytes,
                needed,
            });
        }
        Ok(())
    }

    /// Number of cached entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total bytes used
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Maximum bytes allowed
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    /// Total cache hits
    pub fn hits(&self) -> u64 {
        self.total_hits
    }

    /// Total cache misses
    pub fn misses(&self) -> u64 {
        self.total_misses
    }

    /// Cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            0.0
        } else {
            self.total_hits as f64 / total as f64
        }
    }

    /// Current cache version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_bytes = 0;
        self.total_hits = 0;
        self.total_misses = 0;
    }
}

/// Error when the cache is full and cannot fit new code
#[derive(Debug)]
pub struct CacheFullError {
    pub limit: usize,
    pub used: usize,
    pub needed: usize,
}

impl std::fmt::Display for CacheFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "code cache full: limit={}, used={}, needed={}",
            self.limit, self.used, self.needed
        )
    }
}

impl std::error::Error for CacheFullError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = CodeCache::new(1024);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.max_bytes(), 1024);
        assert_eq!(cache.total_bytes(), 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = CodeCache::new(1024);
        let fake_ptr = 0x1000 as *const u8;
        cache.insert(42, fake_ptr, 64, 0).unwrap();
        assert_eq!(cache.len(), 1);
        assert!(cache.contains(42));
        assert!(!cache.contains(99));

        let entry = cache.get(42).unwrap();
        assert_eq!(entry.code_ptr, fake_ptr);
        assert_eq!(entry.code_size, 64);
    }

    #[test]
    fn test_cache_hit_miss_tracking() {
        let mut cache = CodeCache::new(1024);
        let fake_ptr = 0x1000 as *const u8;
        cache.insert(42, fake_ptr, 64, 0).unwrap();

        // Hit
        assert!(cache.get(42).is_some());
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 0);

        // Miss
        assert!(cache.get(99).is_none());
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);

        assert!((cache.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = CodeCache::new(1024);
        let fake_ptr = 0x1000 as *const u8;
        cache.insert(42, fake_ptr, 64, 0).unwrap();
        assert!(cache.contains(42));

        cache.invalidate(42);
        assert!(!cache.contains(42));
        assert_eq!(cache.total_bytes(), 0);
    }

    #[test]
    fn test_cache_invalidate_all() {
        let mut cache = CodeCache::new(1024);
        let fake_ptr = 0x1000 as *const u8;
        cache.insert(42, fake_ptr, 64, 0).unwrap();
        cache.insert(84, fake_ptr, 64, 0).unwrap();

        cache.invalidate_all();
        // Entries still exist but version mismatch means they won't be found
        assert!(!cache.contains(42));
        assert!(!cache.contains(84));
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = CodeCache::new(128);
        let fake_ptr = 0x1000 as *const u8;

        // Fill cache
        cache.insert(1, fake_ptr, 64, 0).unwrap();
        cache.insert(2, fake_ptr, 64, 0).unwrap();
        assert_eq!(cache.total_bytes(), 128);

        // Use entry 2 (make it hot)
        cache.get(2);

        // Insert another — should evict entry 1 (coldest)
        cache.insert(3, fake_ptr, 64, 0).unwrap();
        assert!(!cache.contains(1)); // evicted
        assert!(cache.contains(3));
    }

    #[test]
    fn test_cache_full_error() {
        let mut cache = CodeCache::new(64);
        let fake_ptr = 0x1000 as *const u8;

        // Fill it
        cache.insert(1, fake_ptr, 64, 0).unwrap();
        // Try to insert more than max — should evict entry 1 and succeed
        let result = cache.insert(2, fake_ptr, 64, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = CodeCache::new(1024);
        let fake_ptr = 0x1000 as *const u8;
        cache.insert(1, fake_ptr, 64, 0).unwrap();
        cache.get(1);
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.total_bytes(), 0);
        assert_eq!(cache.hits(), 0);
    }

    #[test]
    fn test_cache_version() {
        let cache = CodeCache::new(1024);
        assert_eq!(cache.version(), 0);
    }
}
