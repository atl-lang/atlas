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

/// Convert a git URL + tag into the H-411 namespaced cache subpath.
/// Mirrors `resolver_url_to_cache_subpath` in atlas-runtime — kept in sync manually.
/// `https://github.com/atl-pkg/web` + `v0.1.0` → `github.com/atl-pkg/web@v0.1.0`
fn git_url_to_subpath(url: &str, name: &str, tag: &str) -> std::path::PathBuf {
    let stripped = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("git://")
        .trim_start_matches("ssh://")
        .replace(':', "/")
        .trim_start_matches("git@")
        .to_string();
    let stripped = stripped.trim_end_matches(".git");
    if stripped.is_empty() || !stripped.contains('/') {
        return std::path::PathBuf::from(format!("{}@{}", name, tag));
    }
    std::path::PathBuf::from(format!("{}@{}", stripped, tag))
}

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

    // Determine pkg store root: ATLAS_CACHE_DIR (compat) → ATLAS_HOME/pkg → ~/atlas/pkg
    // Mirrors the H-411 layout used by atlas-runtime's module resolver.
    let pkg_root = if let Ok(val) = std::env::var("ATLAS_CACHE_DIR") {
        std::path::PathBuf::from(val)
    } else {
        let atlas_home = std::env::var("ATLAS_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("atlas")
            });
        atlas_home.join("pkg")
    };

    // Check each locked package is present on disk.
    let mut missing_pkgs: Vec<String> = Vec::new();
    for pkg in &lockfile.packages {
        let pkg_present = match &pkg.source {
            // Path deps: verify the path itself exists — no caching needed.
            atlas_package::LockedSource::Path { path } => path.exists(),
            // Git deps: H-411 namespaced layout — <pkg_root>/<host>/<org>/<name>@<tag>
            atlas_package::LockedSource::Git { url, tag, .. } => {
                let version_str = pkg.version.to_string();
                let cache_key = tag.as_deref().unwrap_or(&version_str).to_string();
                let pkg_dir = pkg_root.join(git_url_to_subpath(url, &pkg.name, &cache_key));
                pkg_dir.exists()
            }
            // Registry deps: <pkg_root>/<name>/<version>
            _ => {
                let cache_key = pkg.version.to_string();
                pkg_root.join(&pkg.name).join(&cache_key).exists()
            }
        };
        if !pkg_present {
            let label = match &pkg.source {
                atlas_package::LockedSource::Git { tag: Some(t), .. } => {
                    format!("{}@{}", pkg.name, t)
                }
                atlas_package::LockedSource::Path { path } => {
                    format!("{} (path: {})", pkg.name, path.display())
                }
                _ => format!("{}@{}", pkg.name, pkg.version),
            };
            missing_pkgs.push(label);
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
        // Create the pkg store dir for foo@1.0.0 (registry dep: <pkg_root>/<name>/<version>)
        let pkg_root = TempDir::new().expect("pkg root tempdir");
        let pkg_dir = pkg_root.path().join("foo").join("1.0.0");
        fs::create_dir_all(&pkg_dir).expect("create pkg dir");
        std::env::set_var("ATLAS_CACHE_DIR", pkg_root.path());
        let result = validate_packages(tmp.path());
        std::env::remove_var("ATLAS_CACHE_DIR");
        assert!(result.is_ok(), "should pass when state is valid");
    }

    #[test]
    fn test_path_dep_existing_path_passes() {
        let tmp = TempDir::new().expect("tempdir");
        // atlas.toml depends on "mypkg"
        let content = "[package]\nname = \"test-pkg\"\nversion = \"0.1.0\"\n\n[dependencies]\nmypkg = { path = \"../mypkg\" }\n";
        fs::write(tmp.path().join("atlas.toml"), content).expect("write atlas.toml");

        // Create a sibling directory as the path dep
        let sibling = TempDir::new().expect("sibling tempdir");
        let abs_path = sibling
            .path()
            .to_str()
            .expect("tempdir path is valid UTF-8")
            .to_string();

        // Write lockfile with path source
        let lock = format!(
            "version = 1\n\n[[packages]]\nname = \"mypkg\"\nversion = \"0.1.0\"\n\n[packages.source]\ntype = \"path\"\npath = \"{abs_path}\"\n"
        );
        fs::write(tmp.path().join("atlas.lock"), lock).expect("write atlas.lock");

        let result = validate_packages(tmp.path());
        assert!(result.is_ok(), "should pass for path dep that exists");
    }

    #[test]
    fn test_path_dep_missing_path_errors() {
        let tmp = TempDir::new().expect("tempdir");
        let content = "[package]\nname = \"test-pkg\"\nversion = \"0.1.0\"\n\n[dependencies]\nmypkg = { path = \"../mypkg\" }\n";
        fs::write(tmp.path().join("atlas.toml"), content).expect("write atlas.toml");

        let lock = "version = 1\n\n[[packages]]\nname = \"mypkg\"\nversion = \"0.1.0\"\n\n[packages.source]\ntype = \"path\"\npath = \"/nonexistent/path/mypkg\"\n";
        fs::write(tmp.path().join("atlas.lock"), lock).expect("write atlas.lock");

        let result = validate_packages(tmp.path());
        assert!(
            result.is_err(),
            "should error for path dep that doesn't exist"
        );
    }
}
