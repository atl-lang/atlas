//! Integration tests for build cache management

use atlas_build::BuildCache;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_cache_new_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let _cache = BuildCache::new(cache_dir.clone()).unwrap();

    assert!(cache_dir.exists());
    assert!(cache_dir.is_dir());
}

#[test]
fn test_cache_load_creates_if_missing() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let _cache = BuildCache::load(&cache_dir).unwrap();

    assert!(cache_dir.exists());
}

#[test]
fn test_cache_store_and_retrieve() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    let source_file = temp_dir.path().join("test.atlas");

    // Create source file
    fs::write(&source_file, "fn main() {}").unwrap();

    let mut cache = BuildCache::new(cache_dir).unwrap();

    // Store module
    let module_name = "test";
    let bytecode = vec![1, 2, 3, 4];
    cache
        .store(
            module_name,
            source_file.clone(),
            "fn main() {}",
            bytecode.clone(),
            vec![],
            Duration::from_millis(10),
        )
        .unwrap();

    // Retrieve module
    let retrieved = cache.get(module_name, &source_file).unwrap();

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), bytecode);
}

#[test]
fn test_cache_invalidate_removes_entry() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    let source_file = temp_dir.path().join("test.atlas");

    fs::write(&source_file, "fn main() {}").unwrap();

    let mut cache = BuildCache::new(cache_dir).unwrap();

    // Store module
    cache
        .store(
            "test",
            source_file.clone(),
            "fn main() {}",
            vec![1, 2, 3],
            vec![],
            Duration::from_millis(10),
        )
        .unwrap();

    // Invalidate
    cache.invalidate("test").unwrap();

    // Should not be retrievable
    let retrieved = cache.get("test", &source_file).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_cache_clear_removes_all() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    let source_file = temp_dir.path().join("test.atlas");

    fs::write(&source_file, "fn main() {}").unwrap();

    let mut cache = BuildCache::new(cache_dir).unwrap();

    // Store multiple modules
    for i in 0..3 {
        cache
            .store(
                &format!("test_{}", i),
                source_file.clone(),
                "fn main() {}",
                vec![1, 2, 3],
                vec![],
                Duration::from_millis(10),
            )
            .unwrap();
    }

    // Clear cache
    cache.clear().unwrap();

    // Should be empty
    for i in 0..3 {
        let retrieved = cache.get(&format!("test_{}", i), &source_file).unwrap();
        assert!(retrieved.is_none());
    }
}

#[test]
fn test_cache_stats() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");
    let cache = BuildCache::new(cache_dir).unwrap();

    let stats = cache.stats();

    assert_eq!(stats.cache_entries, 0);
    assert_eq!(stats.cache_size_bytes, 0);
}
