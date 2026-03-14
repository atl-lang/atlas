//! Install dependencies command (atlas install)

use anyhow::{bail, Context, Result};
use atlas_package::installer::Installer;
use atlas_package::manifest::PackageManifest;
use std::path::{Path, PathBuf};

/// Arguments for the install command
#[derive(Debug, Clone)]
#[allow(dead_code)] // packages + production used in P07 (filter by name, skip dev-deps)
pub struct InstallArgs {
    /// Specific packages to install (empty = all from manifest)
    pub packages: Vec<String>,
    /// Only install production dependencies (skip dev)
    pub production: bool,
    /// Force reinstall even if packages exist
    pub force: bool,
    /// Project directory (defaults to current)
    pub project_dir: PathBuf,
    /// Dry run (don't actually install)
    pub dry_run: bool,
    /// Verbose output
    pub verbose: bool,
    /// Quiet output (errors only)
    pub quiet: bool,
}

impl Default for InstallArgs {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            production: false,
            force: false,
            project_dir: PathBuf::from("."),
            dry_run: false,
            verbose: false,
            quiet: false,
        }
    }
}

/// Run the install command
pub fn run(args: InstallArgs) -> Result<()> {
    let manifest_path = find_manifest(&args.project_dir)?;

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    let manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    if manifest.dependencies.is_empty() && manifest.dev_dependencies.is_empty() {
        if !args.quiet {
            println!("Nothing to install.");
        }
        return Ok(());
    }

    let cache_dir = get_cache_dir();
    let project_dir = manifest_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let installer = Installer::new(cache_dir, project_dir);

    if args.dry_run {
        let plan = installer
            .plan(&manifest)
            .context("Failed to build install plan")?;
        println!("Dry run — would fetch {} package(s):", plan.to_fetch.len());
        for p in &plan.to_fetch {
            println!("  {} ({})", p.name, p.source);
        }
        if !plan.already_cached.is_empty() {
            println!("Already cached: {}", plan.already_cached.join(", "));
        }
        return Ok(());
    }

    if !args.quiet {
        println!("Installing packages...");
    }

    let lockfile = installer
        .install(&manifest, args.force)
        .context("Failed to install packages")?;

    if !args.quiet {
        println!("Wrote atlas.lock ({} package(s))", lockfile.packages.len());
    }

    Ok(())
}

fn get_cache_dir() -> PathBuf {
    std::env::var("ATLAS_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".atlas")
                .join("cache")
        })
}

/// Find atlas.toml manifest file
fn find_manifest(start_dir: &Path) -> Result<PathBuf> {
    let mut current = start_dir
        .canonicalize()
        .unwrap_or_else(|_| start_dir.to_path_buf());

    loop {
        let manifest_path = current.join("atlas.toml");
        if manifest_path.exists() {
            return Ok(manifest_path);
        }

        if !current.pop() {
            break;
        }
    }

    bail!(
        "Could not find atlas.toml in {} or any parent directory",
        start_dir.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_manifest(dir: &Path) -> PathBuf {
        let manifest = r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
foo = "^1.0"
bar = "^2.0"

[dev-dependencies]
test-utils = "^0.1"
"#;
        let path = dir.join("atlas.toml");
        fs::write(&path, manifest).expect("write manifest");
        path
    }

    fn create_empty_manifest(dir: &Path) -> PathBuf {
        let manifest = r#"[package]
name = "empty-project"
version = "0.1.0"

[dependencies]

[dev-dependencies]
"#;
        let path = dir.join("atlas.toml");
        fs::write(&path, manifest).expect("write manifest");
        path
    }

    #[test]
    fn test_install_registry_dep_errors() {
        // Simple string deps are registry deps — must fail with helpful message.
        let temp = TempDir::new().expect("tempdir");
        create_test_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };

        let err = run(args).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("registry") || msg.contains("community index") || msg.contains("install"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn test_install_empty_deps_succeeds() {
        let temp = TempDir::new().expect("tempdir");
        create_empty_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };

        // Empty deps → Ok("Nothing to install.")
        assert!(run(args).is_ok());
    }

    #[test]
    fn test_install_no_manifest() {
        let temp = TempDir::new().expect("tempdir");

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        assert!(run(args).is_err());
    }
}
