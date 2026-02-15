use atlas_package::{Downloader, PackageCache};
use semver::Version;
use tempfile::TempDir;

// ==================================================================
// DOWNLOADER TESTS
// ==================================================================

#[test]
fn test_downloader_new() {
    let temp_dir = TempDir::new().unwrap();
    let downloader = Downloader::new(temp_dir.path().to_path_buf());
    assert_eq!(downloader.cache_dir(), temp_dir.path());
    assert!(temp_dir.path().exists());
}

#[test]
fn test_downloader_calculate_checksum() {
    let data = b"hello world";
    let checksum = Downloader::calculate_checksum(data);
    // SHA256 of "hello world"
    assert_eq!(
        checksum,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_downloader_checksum_deterministic() {
    let data = b"test data";
    let checksum1 = Downloader::calculate_checksum(data);
    let checksum2 = Downloader::calculate_checksum(data);
    assert_eq!(checksum1, checksum2);
}

#[test]
fn test_downloader_empty_checksum() {
    let data = b"";
    let checksum = Downloader::calculate_checksum(data);
    // SHA256 of empty string
    assert_eq!(
        checksum,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

// ==================================================================
// CACHE TESTS
// ==================================================================

#[test]
fn test_cache_new() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);
    assert_eq!(cache.package_count(), 0);
    assert_eq!(cache.max_size_mb(), 100);
    assert!(temp_dir.path().exists());
}

#[test]
fn test_cache_has_package_false() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);
    assert!(!cache.has_package("test", &Version::new(1, 0, 0)));
}

#[test]
fn test_cache_add_package() {
    let temp_dir = TempDir::new().unwrap();
    let mut cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);

    cache.add_package(
        "test",
        &Version::new(1, 0, 0),
        temp_dir.path().to_path_buf(),
    );
    assert_eq!(cache.package_count(), 1);
}

#[test]
fn test_cache_add_multiple_packages() {
    let temp_dir = TempDir::new().unwrap();
    let mut cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);

    cache.add_package(
        "test1",
        &Version::new(1, 0, 0),
        temp_dir.path().to_path_buf(),
    );
    cache.add_package(
        "test2",
        &Version::new(1, 0, 0),
        temp_dir.path().to_path_buf(),
    );
    cache.add_package(
        "test3",
        &Version::new(2, 0, 0),
        temp_dir.path().to_path_buf(),
    );

    assert_eq!(cache.package_count(), 3);
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

    cache.add_package(
        "test1",
        &Version::new(1, 0, 0),
        temp_dir.path().to_path_buf(),
    );
    cache.add_package(
        "test2",
        &Version::new(1, 0, 0),
        temp_dir.path().to_path_buf(),
    );
    assert_eq!(cache.package_count(), 2);

    cache.clear();
    assert_eq!(cache.package_count(), 0);
}

#[test]
fn test_cache_lru_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let mut cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);

    cache.add_package(
        "test1",
        &Version::new(1, 0, 0),
        temp_dir.path().to_path_buf(),
    );
    std::thread::sleep(std::time::Duration::from_millis(10));
    cache.add_package(
        "test2",
        &Version::new(1, 0, 0),
        temp_dir.path().to_path_buf(),
    );

    // Both should be tracked
    assert_eq!(cache.package_count(), 2);
}

#[test]
fn test_cache_max_size() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PackageCache::new(temp_dir.path().to_path_buf(), 500);
    assert_eq!(cache.max_size_mb(), 500);
}

#[test]
fn test_cache_current_size() {
    let temp_dir = TempDir::new().unwrap();
    let cache = PackageCache::new(temp_dir.path().to_path_buf(), 100);
    // Empty cache should be 0 MB
    assert_eq!(cache.current_size_mb(), 0);
}

#[test]
fn test_cache_dir_access() {
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().to_path_buf();
    let cache = PackageCache::new(cache_path.clone(), 100);
    assert_eq!(cache.cache_dir(), &cache_path);
}
