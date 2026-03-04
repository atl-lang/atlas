//! Publish package command (atlas publish)

use anyhow::{bail, Context, Result};
use atlas_package::manifest::PackageManifest;
use std::path::{Path, PathBuf};

/// Arguments for the publish command
#[derive(Debug, Clone)]
pub struct PublishArgs {
    /// Project directory (defaults to current)
    pub project_dir: PathBuf,
    /// Registry to publish to
    pub registry: Option<String>,
    /// Skip all validation checks
    pub no_verify: bool,
    /// Perform all checks but don't actually publish
    pub dry_run: bool,
    /// Allow publishing with dirty git state
    pub allow_dirty: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for PublishArgs {
    fn default() -> Self {
        Self {
            project_dir: PathBuf::from("."),
            registry: None,
            no_verify: false,
            dry_run: false,
            allow_dirty: false,
            verbose: false,
        }
    }
}

/// Run the publish command
pub fn run(args: PublishArgs) -> Result<()> {
    let manifest_path = find_manifest(&args.project_dir)?;

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    // Load manifest
    let manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    let _ = (
        &args.registry,
        args.no_verify,
        args.dry_run,
        args.allow_dirty,
        manifest,
    );

    bail!("Package publishing not yet implemented.");
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

    fn create_test_project(dir: &Path) {
        // Create manifest
        let manifest = r#"[package]
name = "test-package"
version = "1.0.0"
description = "A test package"
authors = ["Test Author <test@example.com>"]
license = "MIT"

[dependencies]
"#;
        fs::write(dir.join("atlas.toml"), manifest).unwrap();

        // Create src directory and main file
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.atl"), "fn main() { print(\"hello\") }").unwrap();
    }

    #[test]
    fn test_publish_errors_when_not_implemented() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            dry_run: true,
            ..Default::default()
        };

        let err = run(args).unwrap_err();
        assert!(err
            .to_string()
            .contains("Package publishing not yet implemented"));
    }

    #[test]
    fn test_publish_no_manifest() {
        let temp = TempDir::new().unwrap();

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        assert!(run(args).is_err());
    }
}
