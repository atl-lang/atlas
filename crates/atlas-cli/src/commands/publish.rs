//! Publish package command (atlas publish)
//!
//! Publishing in Atlas is git-native (D-059): there is no central archive server.
//! `atlas publish` validates, creates a local version tag, and guides the user to
//! push it.  Consumers then depend on the package via:
//!   `{ git = "<repo-url>", tag = "v1.2.3" }`
//!
//! Optionally, if ATLAS_INDEX_TOKEN is set, package metadata is POST-ed to a
//! community index for discoverability — non-blocking, best-effort only.

use anyhow::{bail, Context, Result};
use atlas_package::manifest::PackageManifest;
use atlas_package::validator::Validator;
use std::path::{Path, PathBuf};
use std::process::Command;

// ── public API ────────────────────────────────────────────────────────────────

/// Arguments for the publish command.
#[derive(Debug, Clone)]
pub struct PublishArgs {
    /// Project directory (defaults to current).
    pub project_dir: PathBuf,
    /// Reserved: registry URL (unused — publishing is git-native per D-059).
    #[allow(dead_code)]
    pub registry: Option<String>,
    /// Skip all validation checks.
    pub no_verify: bool,
    /// Perform all checks but do not create a tag.
    pub dry_run: bool,
    /// Allow publishing with uncommitted changes in the working tree.
    pub allow_dirty: bool,
    /// Verbose output.
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

// ── entry point ───────────────────────────────────────────────────────────────

/// Run the publish command.
pub fn run(args: PublishArgs) -> Result<()> {
    let manifest_path = find_manifest(&args.project_dir)?;
    let project_dir = manifest_path
        .parent()
        .context("atlas.toml has no parent directory")?;

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    let manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    let pkg_name = &manifest.package.name;
    let version = &manifest.package.version;
    let tag = format!("v{}", version);

    // ── 1. Validate ──────────────────────────────────────────────────────────
    if !args.no_verify {
        if let Err(errors) = Validator::validate_for_publish(&manifest) {
            eprintln!("error: package validation failed:");
            for e in &errors {
                eprintln!("  - {}", e);
            }
            bail!("Validation failed with {} error(s)", errors.len());
        }
        if args.verbose {
            println!("{} Validation passed", green_check());
        }
    }

    // ── 2. Check git working tree ─────────────────────────────────────────────
    if !args.allow_dirty {
        check_clean_working_tree(project_dir)?;
        if args.verbose {
            println!("{} Working tree is clean", green_check());
        }
    }

    // ── 3. Get remote URL (origin) ────────────────────────────────────────────
    let remote_url = get_remote_url(project_dir).unwrap_or_default();

    // ── 4. Verify tag does not exist on remote ────────────────────────────────
    if !remote_url.is_empty() {
        check_tag_not_on_remote(&remote_url, &tag)?;
        if args.verbose {
            println!("{} Tag {} not yet on remote", green_check(), tag);
        }
    }

    // ── 5. Verify tag does not exist locally ─────────────────────────────────
    check_tag_not_local(project_dir, &tag)?;

    // ── 6. Print plan / dry-run ───────────────────────────────────────────────
    println!("Publishing {} {}", pkg_name, tag);

    if args.dry_run {
        println!("\n[Dry run] Would create tag: {}", tag);
        println!("No changes made.");
        return Ok(());
    }

    // ── 7. Create local tag ───────────────────────────────────────────────────
    create_local_tag(project_dir, &tag, pkg_name, version)?;
    println!("{} Created tag {}", green_check(), tag);

    // ── 8. Guide user ─────────────────────────────────────────────────────────
    println!("\nNext step — push the tag to make it available:");
    println!("  git push origin {}", tag);

    if !remote_url.is_empty() {
        println!("\nConsumers can then depend on this package with:");
        println!(
            "  {} = {{ git = \"{}\", tag = \"{}\" }}",
            pkg_name, remote_url, tag
        );
    } else {
        println!("\nConsumers can depend on this package with:");
        println!(
            "  {} = {{ git = \"<your-repo-url>\", tag = \"{}\" }}",
            pkg_name, tag
        );
    }

    // ── 9. Optional community index POST (best-effort) ────────────────────────
    if let Ok(token) = std::env::var("ATLAS_INDEX_TOKEN") {
        post_to_community_index(&token, pkg_name, version, &remote_url, args.verbose);
    }

    Ok(())
}

// ── git helpers ───────────────────────────────────────────────────────────────

/// Fail if the git working tree has uncommitted changes.
fn check_clean_working_tree(project_dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args([
            "-C",
            &project_dir.to_string_lossy(),
            "status",
            "--porcelain",
        ])
        .output()
        .context("Failed to run git status")?;

    if !output.status.success() {
        bail!("git status failed — is this a git repository?");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        bail!(
            "Working tree has uncommitted changes.\n\
             Commit or stash your changes, or use --allow-dirty to skip this check.\n\
             Changes:\n{}",
            stdout.trim_end()
        );
    }

    Ok(())
}

/// Return the URL of the `origin` remote, or an error if not set.
fn get_remote_url(project_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args([
            "-C",
            &project_dir.to_string_lossy(),
            "remote",
            "get-url",
            "origin",
        ])
        .output()
        .context("Failed to run git remote get-url")?;

    if !output.status.success() {
        bail!("No 'origin' remote configured");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Fail if the tag already exists on the remote.
fn check_tag_not_on_remote(remote_url: &str, tag: &str) -> Result<()> {
    let full_ref = format!("refs/tags/{}", tag);
    let output = Command::new("git")
        .args(["ls-remote", "--tags", remote_url, &full_ref])
        .output()
        .context("Failed to run git ls-remote")?;

    if !output.status.success() {
        // ls-remote failure is non-fatal: remote might be unreachable.
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        bail!(
            "Tag '{}' already exists on remote '{}'.\n\
             Bump the version in atlas.toml before publishing.",
            tag,
            remote_url
        );
    }

    Ok(())
}

/// Fail if the tag already exists locally.
fn check_tag_not_local(project_dir: &Path, tag: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["-C", &project_dir.to_string_lossy(), "tag", "--list", tag])
        .output()
        .context("Failed to run git tag --list")?;

    if !output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        bail!(
            "Tag '{}' already exists locally.\n\
             Bump the version in atlas.toml before publishing.",
            tag
        );
    }

    Ok(())
}

/// Create an annotated local git tag.
fn create_local_tag(
    project_dir: &Path,
    tag: &str,
    pkg_name: &str,
    version: &semver::Version,
) -> Result<()> {
    let message = format!("atlas publish {} v{}", pkg_name, version);
    let status = Command::new("git")
        .args([
            "-C",
            &project_dir.to_string_lossy(),
            "tag",
            tag,
            "-m",
            &message,
        ])
        .status()
        .context("Failed to run git tag")?;

    if !status.success() {
        bail!("git tag '{}' failed", tag);
    }

    Ok(())
}

// ── community index ───────────────────────────────────────────────────────────

/// POST minimal package metadata to a community index for discoverability.
///
/// Non-blocking: failures are logged but do not abort the publish.
/// Only runs when ATLAS_INDEX_TOKEN is set.
fn post_to_community_index(
    token: &str,
    name: &str,
    version: &semver::Version,
    repo_url: &str,
    verbose: bool,
) {
    // Build a minimal JSON payload.
    let payload = format!(
        r#"{{"name":"{name}","version":"{version}","repository":"{repo_url}"}}"#,
        name = name,
        version = version,
        repo_url = repo_url,
    );

    // Use curl if available — avoids pulling in an HTTP client dep for a
    // best-effort, optional side-channel.
    let index_url = std::env::var("ATLAS_INDEX_URL")
        .unwrap_or_else(|_| "https://index.atlaslang.dev/api/packages".to_string());

    let status = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-d",
            &payload,
            &index_url,
        ])
        .output();

    match status {
        Ok(out) => {
            let code = String::from_utf8_lossy(&out.stdout);
            if verbose {
                println!("Community index: HTTP {}", code.trim());
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("Community index: skipped ({})", e);
            }
        }
    }
}

// ── filesystem helpers ────────────────────────────────────────────────────────

/// Find atlas.toml by walking up from `start_dir`.
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

fn green_check() -> &'static str {
    "\u{2713}"
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_manifest(dir: &Path, content: &str) {
        fs::write(dir.join("atlas.toml"), content).expect("test");
    }

    fn init_git_repo(dir: &Path) {
        Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "init"])
            .output()
            .expect("test");
        Command::new("git")
            .args([
                "-C",
                &dir.to_string_lossy(),
                "config",
                "user.email",
                "test@example.com",
            ])
            .output()
            .expect("test");
        Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "config", "user.name", "Test"])
            .output()
            .expect("test");
    }

    fn commit_all(dir: &Path) {
        Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "add", "."])
            .output()
            .expect("test");
        Command::new("git")
            .args([
                "-C",
                &dir.to_string_lossy(),
                "commit",
                "-m",
                "init",
                "--allow-empty",
            ])
            .output()
            .expect("test");
    }

    // ── find_manifest ─────────────────────────────────────────────────────────

    #[test]
    fn test_find_manifest_missing_errors() {
        let temp = TempDir::new().expect("test");
        assert!(find_manifest(temp.path()).is_err());
    }

    #[test]
    fn test_find_manifest_found() {
        let temp = TempDir::new().expect("test");
        write_manifest(
            temp.path(),
            "[package]\nname = \"x\"\nversion = \"1.0.0\"\n",
        );
        assert!(find_manifest(temp.path()).is_ok());
    }

    // ── dry_run ───────────────────────────────────────────────────────────────

    #[test]
    fn test_dry_run_validates_and_exits_without_tag() {
        let temp = TempDir::new().expect("test");
        write_manifest(
            temp.path(),
            r#"[package]
name = "my-lib"
version = "1.0.0"
description = "Test library"
license = "MIT"

[dependencies]
"#,
        );
        init_git_repo(temp.path());
        commit_all(temp.path());

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            dry_run: true,
            allow_dirty: true, // skip dirty check for simplicity
            ..Default::default()
        };

        // Should succeed — dry-run path does not call git tag.
        run(args).expect("test");

        // Verify no tag was created.
        let out = Command::new("git")
            .args([
                "-C",
                &temp.path().to_string_lossy(),
                "tag",
                "--list",
                "v1.0.0",
            ])
            .output()
            .expect("test");
        assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
    }

    // ── validation ────────────────────────────────────────────────────────────

    #[test]
    fn test_missing_license_fails_validation() {
        let temp = TempDir::new().expect("test");
        write_manifest(
            temp.path(),
            r#"[package]
name = "my-lib"
version = "1.0.0"
description = "No license here"

[dependencies]
"#,
        );

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            allow_dirty: true,
            ..Default::default()
        };

        let err = run(args).unwrap_err();
        assert!(err.to_string().contains("Validation failed"));
    }

    #[test]
    fn test_missing_description_fails_validation() {
        let temp = TempDir::new().expect("test");
        write_manifest(
            temp.path(),
            r#"[package]
name = "my-lib"
version = "1.0.0"
license = "MIT"

[dependencies]
"#,
        );

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            allow_dirty: true,
            ..Default::default()
        };

        let err = run(args).unwrap_err();
        assert!(err.to_string().contains("Validation failed"));
    }

    #[test]
    fn test_no_verify_skips_validation() {
        let temp = TempDir::new().expect("test");
        // Missing license + description — normally fails validation.
        write_manifest(
            temp.path(),
            "[package]\nname = \"x\"\nversion = \"0.0.0\"\n\n[dependencies]\n",
        );
        init_git_repo(temp.path());
        commit_all(temp.path());

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            no_verify: true,
            dry_run: true,
            allow_dirty: true,
            ..Default::default()
        };

        // --no-verify + --dry-run: validation is skipped, no tag created.
        run(args).expect("test");
    }

    // ── git tag guards ────────────────────────────────────────────────────────

    #[test]
    fn test_local_tag_already_exists_errors() {
        let temp = TempDir::new().expect("test");
        write_manifest(
            temp.path(),
            r#"[package]
name = "my-lib"
version = "1.0.0"
description = "Test"
license = "MIT"

[dependencies]
"#,
        );
        init_git_repo(temp.path());
        commit_all(temp.path());

        // Pre-create the v1.0.0 tag.
        Command::new("git")
            .args([
                "-C",
                &temp.path().to_string_lossy(),
                "tag",
                "v1.0.0",
                "-m",
                "pre-existing",
            ])
            .output()
            .expect("test");

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            allow_dirty: true,
            ..Default::default()
        };

        let err = run(args).unwrap_err();
        assert!(
            err.to_string().contains("already exists"),
            "unexpected error: {}",
            err
        );
    }

    // ── no_manifest ───────────────────────────────────────────────────────────

    #[test]
    fn test_no_manifest_errors() {
        let temp = TempDir::new().expect("test");
        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };
        assert!(run(args).is_err());
    }
}
