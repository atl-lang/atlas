//! Atlas build system infrastructure
//!
//! Provides build orchestration for Atlas projects including:
//! - Build pipeline management
//! - Multiple build targets (library, binary, bytecode, test)
//! - Dependency resolution and building
//! - Parallel compilation
//! - Incremental compilation (phase-11b)
//! - Build profiles and configuration (phase-11c)
//! - Build scripts with sandboxing (phase-11c)
//! - Progress reporting and output formatting (phase-11c)

pub mod binary_emit;
pub mod build_order;
pub mod builder;
pub mod cache;
pub mod error;
pub mod fingerprint;
pub mod incremental;
pub mod module_resolver;
pub mod output;
pub mod profile;
pub mod script;
pub mod targets;

// Re-export main types
pub use build_order::{BuildGraph, ModuleNode};
pub use builder::{BuildConfig, BuildContext, BuildStats, Builder, OptLevel};
pub use cache::{BuildCache, CacheEntry, CacheMetadata, CacheStats};
pub use error::{BuildError, BuildResult};
pub use fingerprint::{
    compute_fingerprint, compute_hash, Fingerprint, FingerprintConfig, FingerprintDb, PlatformInfo,
};
pub use incremental::{
    BuildState, IncrementalEngine, IncrementalPlan, IncrementalStats, RecompileReason,
};
pub use output::{BuildProgress, BuildSummary, ErrorFormatter, OutputMode};
pub use profile::{
    DependencyProfile, ManifestProfileConfig, Profile, ProfileConfig, ProfileManager,
};
pub use script::{
    BuildScript, ScriptContext, ScriptExecutor, ScriptKind, ScriptPhase, ScriptResult,
};
pub use targets::{ArtifactMetadata, BuildArtifact, BuildTarget, TargetKind};

// Re-export atlas-package types for convenience
pub use atlas_package::manifest::PackageManifest;

/// Validate package state before compilation.
///
/// Returns `Ok(())` if safe to proceed.
/// Returns `Err` with an actionable message if validation fails.
/// Skips silently if no `atlas.toml` is found or no dependencies are declared.
pub fn validate_packages(project_dir: &std::path::Path) -> Result<(), String> {
    let manifest_path = project_dir.join("atlas.toml");
    if !manifest_path.exists() {
        return Ok(());
    }

    let manifest = atlas_package::manifest::PackageManifest::from_file(&manifest_path)
        .map_err(|e| format!("Failed to parse atlas.toml: {e}"))?;

    if manifest.dependencies.is_empty() {
        return Ok(());
    }

    let lockfile_path = project_dir.join("atlas.lock");
    if !lockfile_path.exists() {
        return Err("atlas.lock not found — run: atlas install".into());
    }

    let lockfile = atlas_package::lockfile::Lockfile::from_file(&lockfile_path)
        .map_err(|e| format!("Failed to parse atlas.lock: {e}"))?;

    // Compare dep names in manifest vs lockfile
    let manifest_names: std::collections::BTreeSet<&str> =
        manifest.dependencies.keys().map(String::as_str).collect();
    let locked_names: std::collections::BTreeSet<&str> =
        lockfile.packages.iter().map(|p| p.name.as_str()).collect();

    if manifest_names != locked_names {
        let missing: Vec<&str> = manifest_names.difference(&locked_names).copied().collect();
        let extra: Vec<&str> = locked_names.difference(&manifest_names).copied().collect();
        return Err(format!(
            "atlas.lock is out of date (missing: [{}], extra: [{}]) — run: atlas install",
            missing.join(", "),
            extra.join(", ")
        ));
    }

    // Determine cache root: ATLAS_CACHE_DIR env var or ~/.atlas/cache/
    let cache_root = if let Ok(val) = std::env::var("ATLAS_CACHE_DIR") {
        std::path::PathBuf::from(val)
    } else {
        dirs::home_dir()
            .ok_or_else(|| "Could not determine home directory for package cache".to_string())?
            .join(".atlas")
            .join("cache")
    };

    // Check each locked package's cache dir exists
    let mut missing_pkgs: Vec<String> = Vec::new();
    for pkg in &lockfile.packages {
        let pkg_cache = cache_root.join(&pkg.name).join(pkg.version.to_string());
        if !pkg_cache.exists() {
            missing_pkgs.push(format!("{}@{}", pkg.name, pkg.version));
        }
    }

    if !missing_pkgs.is_empty() {
        return Err(format!(
            "packages not installed: [{}] — run: atlas install",
            missing_pkgs.join(", ")
        ));
    }

    Ok(())
}

#[cfg(test)]
mod package_validation_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_manifest(dir: &std::path::Path, with_dep: bool) {
        let content = if with_dep {
            "[package]\nname = \"test-pkg\"\nversion = \"0.1.0\"\n\n[dependencies]\nfoo = \"1.0\"\n"
        } else {
            "[package]\nname = \"test-pkg\"\nversion = \"0.1.0\"\n"
        };
        fs::write(dir.join("atlas.toml"), content).expect("write atlas.toml");
    }

    fn make_lockfile(dir: &std::path::Path, pkg_name: &str) {
        let content = format!(
            "version = 1\n\n[[packages]]\nname = \"{pkg_name}\"\nversion = \"1.0.0\"\n\n[packages.source]\ntype = \"registry\"\n"
        );
        fs::write(dir.join("atlas.lock"), content).expect("write atlas.lock");
    }

    #[test]
    fn test_no_atlas_toml_skips() {
        let tmp = TempDir::new().expect("tempdir");
        let result = validate_packages(tmp.path());
        assert!(result.is_ok(), "should skip when no atlas.toml");
    }

    #[test]
    fn test_no_deps_skips() {
        let tmp = TempDir::new().expect("tempdir");
        make_manifest(tmp.path(), false);
        let result = validate_packages(tmp.path());
        assert!(result.is_ok(), "should skip when no dependencies");
    }

    #[test]
    fn test_missing_lockfile_errors() {
        let tmp = TempDir::new().expect("tempdir");
        make_manifest(tmp.path(), true);
        let result = validate_packages(tmp.path());
        assert!(result.is_err(), "should error when lockfile missing");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("atlas install"),
            "error should mention atlas install, got: {msg}"
        );
    }

    #[test]
    fn test_stale_lockfile_errors() {
        let tmp = TempDir::new().expect("tempdir");
        // atlas.toml has "foo", lockfile has "bar"
        make_manifest(tmp.path(), true);
        make_lockfile(tmp.path(), "bar");
        let result = validate_packages(tmp.path());
        assert!(result.is_err(), "should error when lockfile is stale");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("atlas install"),
            "error should mention atlas install, got: {msg}"
        );
    }

    #[test]
    fn test_missing_cache_errors() {
        let tmp = TempDir::new().expect("tempdir");
        make_manifest(tmp.path(), true);
        make_lockfile(tmp.path(), "foo");
        // Cache dir does not exist
        let cache_dir = TempDir::new().expect("cache tempdir");
        // Point to a non-existent subdir
        let fake_cache = cache_dir.path().join("nonexistent");
        std::env::set_var("ATLAS_CACHE_DIR", &fake_cache);
        let result = validate_packages(tmp.path());
        std::env::remove_var("ATLAS_CACHE_DIR");
        assert!(result.is_err(), "should error when cache is missing");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("atlas install"),
            "error should mention atlas install, got: {msg}"
        );
    }

    #[test]
    fn test_valid_state_passes() {
        let tmp = TempDir::new().expect("tempdir");
        make_manifest(tmp.path(), true);
        make_lockfile(tmp.path(), "foo");
        // Create the cache dir for foo@1.0.0
        let cache_dir = TempDir::new().expect("cache tempdir");
        let pkg_cache = cache_dir.path().join("foo").join("1.0.0");
        fs::create_dir_all(&pkg_cache).expect("create cache dir");
        std::env::set_var("ATLAS_CACHE_DIR", cache_dir.path());
        let result = validate_packages(tmp.path());
        std::env::remove_var("ATLAS_CACHE_DIR");
        assert!(result.is_ok(), "should pass when state is valid");
    }
}
