use atlas_package::{LocalRegistry, PackageMetadata, Registry, RegistryManager, RemoteRegistry};
use semver::Version;
use std::fs;
use tempfile::TempDir;

// ==================================================================
// LOCAL REGISTRY TESTS
// ==================================================================

#[test]
fn test_local_registry_query_versions() {
    let temp_dir = TempDir::new().unwrap();
    let registry_root = temp_dir.path().to_path_buf();

    // Create mock package structure
    let pkg_dir = registry_root.join("test-package");
    fs::create_dir_all(pkg_dir.join("1.0.0")).unwrap();
    fs::create_dir_all(pkg_dir.join("1.1.0")).unwrap();
    fs::create_dir_all(pkg_dir.join("2.0.0")).unwrap();

    let registry = LocalRegistry::new(registry_root);
    let versions = registry.query_versions("test-package").unwrap();

    assert_eq!(versions.len(), 3);
    assert!(versions.contains(&Version::new(1, 0, 0)));
    assert!(versions.contains(&Version::new(1, 1, 0)));
    assert!(versions.contains(&Version::new(2, 0, 0)));
}

#[test]
fn test_local_registry_get_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let registry_root = temp_dir.path().to_path_buf();

    // Create mock package with metadata
    let version_dir = registry_root.join("test-package/1.0.0");
    fs::create_dir_all(&version_dir).unwrap();

    let metadata_content = r#"
name = "test-package"
checksum = "abc123"
"#;
    fs::write(version_dir.join("metadata.toml"), metadata_content).unwrap();

    let registry = LocalRegistry::new(registry_root);
    let metadata = registry
        .get_metadata("test-package", &Version::new(1, 0, 0))
        .unwrap();

    assert_eq!(metadata.name, "test-package");
    assert_eq!(metadata.version, Version::new(1, 0, 0));
    assert_eq!(metadata.checksum, "abc123");
}

#[test]
fn test_local_registry_download() {
    let temp_dir = TempDir::new().unwrap();
    let registry_root = temp_dir.path().to_path_buf();

    // Create mock package archive
    let version_dir = registry_root.join("test-package/1.0.0");
    fs::create_dir_all(&version_dir).unwrap();

    let archive_content = b"mock archive data";
    fs::write(version_dir.join("package.tar.gz"), archive_content).unwrap();

    let registry = LocalRegistry::new(registry_root);
    let data = registry
        .download("test-package", &Version::new(1, 0, 0))
        .unwrap();

    assert_eq!(data, archive_content);
}

#[test]
fn test_local_registry_package_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let registry = LocalRegistry::new(temp_dir.path().to_path_buf());

    let result = registry.query_versions("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_local_registry_version_sorting() {
    let temp_dir = TempDir::new().unwrap();
    let registry_root = temp_dir.path().to_path_buf();

    // Create versions in random order
    let pkg_dir = registry_root.join("test-package");
    fs::create_dir_all(pkg_dir.join("2.0.0")).unwrap();
    fs::create_dir_all(pkg_dir.join("1.0.0")).unwrap();
    fs::create_dir_all(pkg_dir.join("1.5.0")).unwrap();

    let registry = LocalRegistry::new(registry_root);
    let versions = registry.query_versions("test-package").unwrap();

    // Verify sorted
    assert_eq!(versions[0], Version::new(1, 0, 0));
    assert_eq!(versions[1], Version::new(1, 5, 0));
    assert_eq!(versions[2], Version::new(2, 0, 0));
}

// ==================================================================
// REMOTE REGISTRY TESTS
// ==================================================================

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

// ==================================================================
// REGISTRY MANAGER TESTS
// ==================================================================

#[test]
fn test_registry_manager_new() {
    let manager = RegistryManager::new();
    assert_eq!(manager.registry_count(), 0);
    assert!(manager.is_cache_enabled());
}

#[test]
fn test_registry_manager_add_registry() {
    let mut manager = RegistryManager::new();
    let temp_dir = TempDir::new().unwrap();
    let registry = Box::new(LocalRegistry::new(temp_dir.path().to_path_buf()));

    manager.add_registry(registry);
    assert_eq!(manager.registry_count(), 1);
}

#[test]
fn test_registry_manager_multiple_sources() {
    let mut manager = RegistryManager::new();

    let temp_dir1 = TempDir::new().unwrap();
    let registry1 = Box::new(LocalRegistry::new(temp_dir1.path().to_path_buf()));

    let temp_dir2 = TempDir::new().unwrap();
    let registry2 = Box::new(LocalRegistry::new(temp_dir2.path().to_path_buf()));

    manager.add_registry(registry1);
    manager.add_registry(registry2);

    assert_eq!(manager.registry_count(), 2);
}

#[test]
fn test_registry_manager_first_match_wins() {
    let mut manager = RegistryManager::new();

    // Create first registry with package
    let temp_dir1 = TempDir::new().unwrap();
    let pkg_dir = temp_dir1.path().join("test-package");
    fs::create_dir_all(pkg_dir.join("1.0.0")).unwrap();
    let registry1 = Box::new(LocalRegistry::new(temp_dir1.path().to_path_buf()));

    // Create second registry (empty)
    let temp_dir2 = TempDir::new().unwrap();
    let registry2 = Box::new(LocalRegistry::new(temp_dir2.path().to_path_buf()));

    manager.add_registry(registry1);
    manager.add_registry(registry2);

    // Should find package in first registry
    let versions = manager.query_versions("test-package").unwrap();
    assert_eq!(versions.len(), 1);
}

#[test]
fn test_registry_manager_fallback() {
    let mut manager = RegistryManager::new();

    // First registry doesn't have package
    let temp_dir1 = TempDir::new().unwrap();
    let registry1 = Box::new(LocalRegistry::new(temp_dir1.path().to_path_buf()));

    // Second registry has package
    let temp_dir2 = TempDir::new().unwrap();
    let pkg_dir = temp_dir2.path().join("test-package");
    fs::create_dir_all(pkg_dir.join("1.0.0")).unwrap();
    let registry2 = Box::new(LocalRegistry::new(temp_dir2.path().to_path_buf()));

    manager.add_registry(registry1);
    manager.add_registry(registry2);

    // Should find package in second registry
    let versions = manager.query_versions("test-package").unwrap();
    assert_eq!(versions.len(), 1);
}

#[test]
fn test_registry_manager_cache_control() {
    let mut manager = RegistryManager::new();
    assert!(manager.is_cache_enabled());

    manager.set_cache_enabled(false);
    assert!(!manager.is_cache_enabled());
}

#[test]
fn test_registry_manager_package_not_found() {
    let manager = RegistryManager::new();
    let result = manager.query_versions("nonexistent");
    assert!(result.is_err());
}

// ==================================================================
// PACKAGE METADATA TESTS
// ==================================================================

#[test]
fn test_package_metadata_new() {
    let metadata = PackageMetadata::new("test".to_string(), Version::new(1, 0, 0));
    assert_eq!(metadata.name, "test");
    assert_eq!(metadata.version, Version::new(1, 0, 0));
    assert_eq!(metadata.checksum, "");
    assert_eq!(metadata.download_url, "");
}

#[test]
fn test_package_metadata_builder_pattern() {
    let metadata = PackageMetadata::new("test".to_string(), Version::new(1, 0, 0))
        .with_checksum("abc123".to_string())
        .with_download_url("https://example.com/pkg.tar.gz".to_string())
        .with_dependencies(vec!["dep1".to_string(), "dep2".to_string()]);

    assert_eq!(metadata.checksum, "abc123");
    assert_eq!(metadata.download_url, "https://example.com/pkg.tar.gz");
    assert_eq!(metadata.dependencies.len(), 2);
}

#[test]
fn test_package_metadata_equality() {
    let meta1 = PackageMetadata::new("test".to_string(), Version::new(1, 0, 0))
        .with_checksum("abc".to_string());

    let meta2 = PackageMetadata::new("test".to_string(), Version::new(1, 0, 0))
        .with_checksum("abc".to_string());

    assert_eq!(meta1, meta2);
}
