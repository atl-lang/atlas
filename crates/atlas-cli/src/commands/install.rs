//! Install dependencies command (atlas install)

use anyhow::{bail, Context, Result};
use atlas_package::manifest::PackageManifest;
use std::path::{Path, PathBuf};

/// Arguments for the install command
#[derive(Debug, Clone)]
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

    // Load manifest
    let manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    let _ = (
        &args.packages,
        args.production,
        args.force,
        args.dry_run,
        args.quiet,
        manifest,
    );

    bail!("Package registry not yet implemented. Local dependencies via path in atlas.toml are supported.");
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
        fs::write(&path, manifest).unwrap();
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
        fs::write(&path, manifest).unwrap();
        path
    }

    #[test]
    fn test_install_errors_when_registry_unimplemented() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };

        let err = run(args).unwrap_err();
        assert!(err
            .to_string()
            .contains("Package registry not yet implemented"));
    }

    #[test]
    fn test_install_empty_deps_errors() {
        let temp = TempDir::new().unwrap();
        create_empty_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };

        let err = run(args).unwrap_err();
        assert!(err
            .to_string()
            .contains("Package registry not yet implemented"));
    }

    #[test]
    fn test_install_no_manifest() {
        let temp = TempDir::new().unwrap();

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        assert!(run(args).is_err());
    }
}
