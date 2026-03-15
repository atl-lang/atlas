//! Update dependencies command (atlas update)
//!
//! For git deps: queries remote tags via `git ls-remote`, bumps to the latest
//! semver tag (or latest satisfying the declared version constraint), updates
//! `atlas.lock`, and re-fetches the new version into the local cache.
//!
//! For path deps: always considered up-to-date (no remote version check).
//! Registry deps are not yet supported (D-059: git is the registry).

use anyhow::{bail, Context, Result};
use atlas_package::fetcher::GitFetcher;
use atlas_package::manifest::{Dependency, PackageManifest};
use atlas_package::{LockedPackage, LockedSource, Lockfile};
use semver::{Version, VersionReq};
use std::path::{Path, PathBuf};

// ── public API ────────────────────────────────────────────────────────────────

/// Arguments for the update command.
#[derive(Debug, Clone)]
pub struct UpdateArgs {
    /// Specific packages to update (empty = all).
    pub packages: Vec<String>,
    /// Only update dev dependencies.
    pub dev: bool,
    /// Project directory (defaults to current).
    pub project_dir: PathBuf,
    /// Dry run — print plan, do not modify files.
    pub dry_run: bool,
    /// Verbose output.
    pub verbose: bool,
}

impl Default for UpdateArgs {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            dev: false,
            project_dir: PathBuf::from("."),
            dry_run: false,
            verbose: false,
        }
    }
}

/// Per-package result of an update check.
#[derive(Debug)]
pub struct UpdateResult {
    pub name: String,
    /// Current locked version (None if not in lockfile yet).
    pub old_version: Option<Version>,
    /// Best available version found from remote tags.
    pub new_version: Version,
    /// New tag name (e.g. "v1.2.0").
    pub new_tag: String,
    /// Git URL this dep comes from.
    pub url: String,
    /// True when `new_version > old_version` (or no existing lock entry).
    pub needs_update: bool,
}

// ── entry point ───────────────────────────────────────────────────────────────

/// Run the update command.
pub fn run(args: UpdateArgs) -> Result<()> {
    let manifest_path = find_manifest(&args.project_dir)?;
    let project_dir = manifest_path
        .parent()
        .context("atlas.toml has no parent directory")?;
    let lockfile_path = project_dir.join("atlas.lock");

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    let manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    let existing_lockfile = if lockfile_path.exists() {
        Some(Lockfile::from_file(&lockfile_path).context("Failed to read atlas.lock")?)
    } else {
        None
    };

    // Build the set of dep names we should check.
    let target_names = resolve_target_names(&args, &manifest)?;

    if target_names.is_empty() {
        println!("No dependencies to update.");
        return Ok(());
    }

    println!("Checking for updates...");

    let cache_dir = get_cache_dir();
    let fetcher = GitFetcher::new(cache_dir);

    // Check each dep for updates.
    let mut updates: Vec<UpdateResult> = Vec::new();

    for name in &target_names {
        let dep = manifest
            .dependencies
            .get(name)
            .or_else(|| manifest.dev_dependencies.get(name))
            .expect("dep in target_names must exist in manifest");

        match check_dep_for_update(
            name,
            dep,
            existing_lockfile.as_ref(),
            &fetcher,
            args.verbose,
        )? {
            Some(result) => updates.push(result),
            None => {
                if args.verbose {
                    println!("  {} {} (path dep — skipped)", green_check(), name);
                }
            }
        }
    }

    // Report.
    let needs_update: Vec<&UpdateResult> = updates.iter().filter(|u| u.needs_update).collect();

    if needs_update.is_empty() {
        println!("\n{} All packages are up to date.", green_check());
        return Ok(());
    }

    println!("\nUpdates available:");
    for u in &needs_update {
        let old = u
            .old_version
            .as_ref()
            .map(|v| format!("v{}", v))
            .unwrap_or_else(|| "(new)".to_string());
        println!("  {} {} {} → {}", arrow_up(), u.name, old, u.new_tag);
    }

    if args.verbose {
        let current: Vec<&UpdateResult> = updates.iter().filter(|u| !u.needs_update).collect();
        for u in current {
            println!("  {} {} {} (current)", green_check(), u.name, u.new_tag);
        }
    }

    if args.dry_run {
        println!("\n[Dry run] Would update {} package(s)", needs_update.len());
        return Ok(());
    }

    // Apply updates: re-fetch and update lockfile.
    let mut lockfile = existing_lockfile.unwrap_or_else(Lockfile::new);

    for u in &needs_update {
        println!("  Fetching {} {}...", u.name, u.new_tag);
        let fetch_result = fetcher
            .fetch(&u.name, &u.url, &u.new_tag)
            .with_context(|| format!("Failed to fetch {} at {}", u.name, u.new_tag))?;

        lockfile.add_package(LockedPackage {
            name: u.name.clone(),
            version: u.new_version.clone(),
            source: LockedSource::Git {
                url: u.url.clone(),
                rev: fetch_result.rev,
                tag: Some(u.new_tag.clone()),
            },
            checksum: Some(fetch_result.checksum),
            dependencies: Default::default(),
        });
    }

    lockfile.write_to_file(&lockfile_path)?;

    println!(
        "\n{} Updated {} package{}",
        green_check(),
        needs_update.len(),
        if needs_update.len() == 1 { "" } else { "s" }
    );

    Ok(())
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Determine which dep names to check, validating any explicitly-named packages.
fn resolve_target_names(args: &UpdateArgs, manifest: &PackageManifest) -> Result<Vec<String>> {
    if !args.packages.is_empty() {
        for pkg in &args.packages {
            if !manifest.dependencies.contains_key(pkg)
                && !manifest.dev_dependencies.contains_key(pkg)
            {
                bail!("Package '{}' not found in dependencies", pkg);
            }
        }
        return Ok(args.packages.clone());
    }

    // All deps (or dev-only when --dev is set).
    if args.dev {
        Ok(manifest.dev_dependencies.keys().cloned().collect())
    } else {
        let mut all: Vec<String> = manifest.dependencies.keys().cloned().collect();
        all.extend(manifest.dev_dependencies.keys().cloned());
        Ok(all)
    }
}

/// Check a single dep for available updates.
///
/// Returns `None` for path/registry deps (no remote version to compare).
fn check_dep_for_update(
    name: &str,
    dep: &Dependency,
    lockfile: Option<&Lockfile>,
    fetcher: &GitFetcher,
    verbose: bool,
) -> Result<Option<UpdateResult>> {
    let detailed = match dep {
        Dependency::Simple(_) => return Ok(None), // registry dep — skip
        Dependency::Detailed(d) => d,
    };

    // Path dep — no remote check.
    if detailed.path.is_some() {
        return Ok(None);
    }

    // Must be a git dep.
    let url = match &detailed.git {
        Some(u) => u.clone(),
        None => return Ok(None), // registry dep — skip
    };

    // Optional semver constraint from the `version` field.
    let req: Option<VersionReq> = detailed
        .version
        .as_deref()
        .and_then(|v| VersionReq::parse(v).ok());

    if verbose {
        println!("  Querying tags for {} ({})", name, url);
    }

    let raw_tags = fetcher
        .list_remote_tags(&url)
        .with_context(|| format!("Failed to list remote tags for '{}'", name))?;

    // Filter to semver tags (with optional 'v' prefix), apply constraint.
    let best = best_semver_tag(&raw_tags, req.as_ref());

    let (new_tag, new_version) = match best {
        Some(pair) => pair,
        None => {
            if verbose {
                println!("  {} {} — no semver tags found", arrow_up(), name);
            }
            return Ok(None);
        }
    };

    let old_version = lockfile
        .and_then(|lf| lf.get_package(name))
        .map(|p| p.version.clone());

    let needs_update = old_version
        .as_ref()
        .map(|old| &new_version > old)
        .unwrap_or(true);

    Ok(Some(UpdateResult {
        name: name.to_string(),
        old_version,
        new_version,
        new_tag,
        url,
        needs_update,
    }))
}

/// From a list of raw tag strings, find the highest semver tag satisfying `req`
/// (or the global maximum if no constraint). Returns `(tag_string, version)`.
fn best_semver_tag(tags: &[String], req: Option<&VersionReq>) -> Option<(String, Version)> {
    let mut candidates: Vec<(String, Version)> = tags
        .iter()
        .filter_map(|tag| {
            let stripped = tag.strip_prefix('v').unwrap_or(tag);
            let v = Version::parse(stripped).ok()?;
            if let Some(r) = req {
                if !r.matches(&v) {
                    return None;
                }
            }
            Some((tag.clone(), v))
        })
        .collect();

    // Sort descending, pick first (highest).
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates.into_iter().next()
}

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

fn get_cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ATLAS_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    let root = std::env::var("ATLAS_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("atlas")
        });
    root.join("pkg")
}

fn green_check() -> &'static str {
    "\u{2713}"
}

fn arrow_up() -> &'static str {
    "\u{2191}"
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_package::{LockedSource, Lockfile};
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    // ── best_semver_tag ───────────────────────────────────────────────────────

    #[test]
    fn test_best_semver_tag_no_constraint_picks_latest() {
        let tags = vec![
            "v1.0.0".to_string(),
            "v1.2.0".to_string(),
            "v1.1.0".to_string(),
        ];
        let (tag, ver) = best_semver_tag(&tags, None).unwrap();
        assert_eq!(tag, "v1.2.0");
        assert_eq!(ver, Version::new(1, 2, 0));
    }

    #[test]
    fn test_best_semver_tag_with_constraint() {
        let tags = vec![
            "v1.0.0".to_string(),
            "v1.5.0".to_string(),
            "v2.0.0".to_string(),
        ];
        let req = VersionReq::parse("^1").unwrap();
        let (tag, ver) = best_semver_tag(&tags, Some(&req)).unwrap();
        assert_eq!(tag, "v1.5.0");
        assert_eq!(ver, Version::new(1, 5, 0));
    }

    #[test]
    fn test_best_semver_tag_filters_non_semver() {
        let tags = vec![
            "nightly".to_string(),
            "latest".to_string(),
            "v1.0.0".to_string(),
        ];
        let (tag, _) = best_semver_tag(&tags, None).unwrap();
        assert_eq!(tag, "v1.0.0");
    }

    #[test]
    fn test_best_semver_tag_empty_returns_none() {
        assert!(best_semver_tag(&[], None).is_none());
    }

    #[test]
    fn test_best_semver_tag_all_non_semver_returns_none() {
        let tags = vec!["nightly".to_string(), "edge".to_string()];
        assert!(best_semver_tag(&tags, None).is_none());
    }

    #[test]
    fn test_best_semver_tag_no_v_prefix() {
        let tags = vec!["1.0.0".to_string(), "1.2.0".to_string()];
        let (tag, ver) = best_semver_tag(&tags, None).unwrap();
        assert_eq!(tag, "1.2.0");
        assert_eq!(ver, Version::new(1, 2, 0));
    }

    // ── resolve_target_names ──────────────────────────────────────────────────

    #[test]
    fn test_resolve_target_names_all_deps() {
        let temp = TempDir::new().unwrap();
        let manifest_str = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
foo = { git = "https://github.com/x/foo", tag = "v1.0.0" }

[dev-dependencies]
bar = { git = "https://github.com/x/bar", tag = "v1.0.0" }
"#;
        fs::write(temp.path().join("atlas.toml"), manifest_str).unwrap();
        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        let args = UpdateArgs::default();
        let names = resolve_target_names(&args, &manifest).unwrap();
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_resolve_target_names_dev_only() {
        let temp = TempDir::new().unwrap();
        let manifest_str = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
foo = { git = "https://github.com/x/foo", tag = "v1.0.0" }

[dev-dependencies]
bar = { git = "https://github.com/x/bar", tag = "v1.0.0" }
"#;
        fs::write(temp.path().join("atlas.toml"), manifest_str).unwrap();
        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        let args = UpdateArgs {
            dev: true,
            ..Default::default()
        };
        let names = resolve_target_names(&args, &manifest).unwrap();
        assert!(!names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_resolve_target_names_specific_package() {
        let temp = TempDir::new().unwrap();
        let manifest_str = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
foo = { git = "https://github.com/x/foo", tag = "v1.0.0" }
bar = { git = "https://github.com/x/bar", tag = "v1.0.0" }
"#;
        fs::write(temp.path().join("atlas.toml"), manifest_str).unwrap();
        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        let args = UpdateArgs {
            packages: vec!["foo".to_string()],
            ..Default::default()
        };
        let names = resolve_target_names(&args, &manifest).unwrap();
        assert_eq!(names, vec!["foo".to_string()]);
    }

    #[test]
    fn test_resolve_target_names_unknown_package_errors() {
        let temp = TempDir::new().unwrap();
        let manifest_str = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
foo = { git = "https://github.com/x/foo", tag = "v1.0.0" }
"#;
        fs::write(temp.path().join("atlas.toml"), manifest_str).unwrap();
        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        let args = UpdateArgs {
            packages: vec!["nonexistent".to_string()],
            ..Default::default()
        };
        assert!(resolve_target_names(&args, &manifest).is_err());
    }

    // ── dry_run_does_not_write ────────────────────────────────────────────────

    #[test]
    fn test_dry_run_does_not_create_lockfile() {
        let temp = TempDir::new().unwrap();
        // Empty deps — "No dependencies to update" path.
        let manifest_str = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
"#;
        fs::write(temp.path().join("atlas.toml"), manifest_str).unwrap();

        let args = UpdateArgs {
            project_dir: temp.path().to_path_buf(),
            dry_run: true,
            ..Default::default()
        };
        run(args).unwrap();
        assert!(!temp.path().join("atlas.lock").exists());
    }

    // ── find_manifest ─────────────────────────────────────────────────────────

    #[test]
    fn test_find_manifest_missing_errors() {
        let temp = TempDir::new().unwrap();
        assert!(find_manifest(temp.path()).is_err());
    }

    #[test]
    fn test_find_manifest_finds_it() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("atlas.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert!(find_manifest(temp.path()).is_ok());
    }

    // ── update_result ─────────────────────────────────────────────────────────

    #[test]
    fn test_update_result_needs_update_when_new_greater() {
        let result = UpdateResult {
            name: "foo".to_string(),
            old_version: Some(Version::new(1, 0, 0)),
            new_version: Version::new(1, 1, 0),
            new_tag: "v1.1.0".to_string(),
            url: "https://github.com/x/foo".to_string(),
            needs_update: true,
        };
        assert!(result.needs_update);
    }

    #[test]
    fn test_update_result_no_update_when_equal() {
        let result = UpdateResult {
            name: "foo".to_string(),
            old_version: Some(Version::new(1, 0, 0)),
            new_version: Version::new(1, 0, 0),
            new_tag: "v1.0.0".to_string(),
            url: "https://github.com/x/foo".to_string(),
            needs_update: false,
        };
        assert!(!result.needs_update);
    }
}
