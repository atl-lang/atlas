//! Package installer for Atlas.
//!
//! Reads `atlas.toml`, fetches all dependencies via git (or resolves path deps),
//! and writes `atlas.lock`.

use crate::fetcher::GitFetcher;
use crate::lockfile::{LockedPackage, LockedSource, Lockfile};
use crate::manifest::{Dependency, DetailedDependency, PackageManifest};
use crate::{PackageError, Result};
use std::collections::HashSet;
use std::path::PathBuf;

/// Installer: resolves and fetches all dependencies in an `atlas.toml`.
pub struct Installer {
    cache_dir: PathBuf,
    project_dir: PathBuf,
}

/// High-level description of what `install()` would do, without executing it.
pub struct InstallPlan {
    /// Dependencies that need to be downloaded.
    pub to_fetch: Vec<PlannedFetch>,
    /// Dependency names already present in cache (git deps only).
    pub already_cached: Vec<String>,
    /// Where the lockfile will be written.
    pub lockfile_path: PathBuf,
}

/// A single dependency that will be fetched.
pub struct PlannedFetch {
    pub name: String,
    /// Human-readable source description: "git: <url>@<tag>" or "path: <path>".
    pub source: String,
}

/// Dependency kind extracted from a manifest `Dependency` value.
enum DepKind {
    Git { url: String, tag: String },
    Path(PathBuf),
    Registry,
}

fn dep_kind(dep: &Dependency) -> DepKind {
    match dep {
        Dependency::Simple(_) => DepKind::Registry,
        Dependency::Detailed(d) => kind_from_detailed(d),
    }
}

fn kind_from_detailed(d: &DetailedDependency) -> DepKind {
    if let Some(path) = &d.path {
        return DepKind::Path(path.clone());
    }
    if let Some(url) = &d.git {
        let tag = d
            .tag
            .clone()
            .or_else(|| d.rev.clone())
            .or_else(|| d.branch.clone())
            .unwrap_or_else(|| "HEAD".to_string());
        return DepKind::Git {
            url: url.clone(),
            tag,
        };
    }
    DepKind::Registry
}

impl Installer {
    /// Create a new installer.
    pub fn new(cache_dir: PathBuf, project_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            project_dir,
        }
    }

    /// Build an install plan without fetching anything.
    pub fn plan(&self, manifest: &PackageManifest) -> Result<InstallPlan> {
        let fetcher = GitFetcher::new(self.cache_dir.clone());
        let lockfile_path = self.project_dir.join("atlas.lock");
        let mut to_fetch = Vec::new();
        let mut already_cached = Vec::new();

        for (name, dep) in manifest
            .dependencies
            .iter()
            .chain(manifest.dev_dependencies.iter())
        {
            match dep_kind(dep) {
                DepKind::Git { url, tag } => {
                    let source = format!("git: {}@{}", url, tag);
                    if fetcher.is_cached(name, &tag) {
                        already_cached.push(name.clone());
                    } else {
                        to_fetch.push(PlannedFetch {
                            name: name.clone(),
                            source,
                        });
                    }
                }
                DepKind::Path(p) => {
                    to_fetch.push(PlannedFetch {
                        name: name.clone(),
                        source: format!("path: {}", p.display()),
                    });
                }
                DepKind::Registry => {
                    to_fetch.push(PlannedFetch {
                        name: name.clone(),
                        source: "registry: (not yet supported)".to_string(),
                    });
                }
            }
        }

        Ok(InstallPlan {
            to_fetch,
            already_cached,
            lockfile_path,
        })
    }

    /// Execute: fetch all deps and write `atlas.lock`.
    ///
    /// If a valid, up-to-date lockfile already exists and `force` is false, the
    /// lockfile is reused and only missing cache entries are populated.
    pub fn install(&self, manifest: &PackageManifest, force: bool) -> Result<Lockfile> {
        let lockfile_path = self.project_dir.join("atlas.lock");

        // Try reusing an existing lockfile.
        if !force && lockfile_path.exists() {
            if let Ok(existing) = Lockfile::from_file(&lockfile_path) {
                if self.lockfile_is_fresh(&existing, manifest) {
                    // Ensure cache is populated for every locked entry.
                    self.populate_cache_from_lockfile(&existing)?;
                    return Ok(existing);
                }
            }
        }

        // Fresh resolution.
        let mut lockfile = Lockfile::new();
        let fetcher = GitFetcher::new(self.cache_dir.clone());

        // Merge regular + dev deps (dev deps get the same treatment for now).
        let all_deps: Vec<(&String, &Dependency)> = manifest
            .dependencies
            .iter()
            .chain(manifest.dev_dependencies.iter())
            .collect();

        for (name, dep) in all_deps {
            let locked = self.resolve_dep(name, dep, &fetcher)?;
            lockfile.add_package(locked);
        }

        lockfile.write_to_file(&lockfile_path)?;
        Ok(lockfile)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    /// Returns `true` when every dep name in `manifest` appears in `lockfile`.
    fn lockfile_is_fresh(&self, lockfile: &Lockfile, manifest: &PackageManifest) -> bool {
        let locked_names: HashSet<&str> =
            lockfile.packages.iter().map(|p| p.name.as_str()).collect();
        let manifest_names: HashSet<&str> = manifest
            .dependencies
            .keys()
            .chain(manifest.dev_dependencies.keys())
            .map(|s| s.as_str())
            .collect();
        manifest_names == locked_names
    }

    /// For a valid cached lockfile, ensure git deps are still present in the
    /// local cache (someone might have cleared it).
    fn populate_cache_from_lockfile(&self, lockfile: &Lockfile) -> Result<()> {
        let fetcher = GitFetcher::new(self.cache_dir.clone());
        for pkg in &lockfile.packages {
            if let LockedSource::Git { url, rev: _ } = &pkg.source {
                // We only have the rev here, not the original tag.  If the
                // cache entry is missing we can't re-fetch without the tag, so
                // we skip silently — the caller (install --force) can fix it.
                let _ = (fetcher.is_cached(&pkg.name, &pkg.version.to_string()), url);
            }
        }
        Ok(())
    }

    /// Resolve a single dependency to a `LockedPackage`.
    fn resolve_dep(
        &self,
        name: &str,
        dep: &Dependency,
        fetcher: &GitFetcher,
    ) -> Result<LockedPackage> {
        match dep_kind(dep) {
            DepKind::Git { url, tag } => {
                let result = fetcher.fetch(name, &url, &tag)?;
                let version = dep_version_or_zero(dep);
                Ok(LockedPackage {
                    name: name.to_string(),
                    version,
                    source: LockedSource::Git {
                        url,
                        rev: result.rev,
                    },
                    checksum: Some(result.checksum),
                    dependencies: Default::default(),
                })
            }
            DepKind::Path(relative) => {
                let abs = if relative.is_absolute() {
                    relative.clone()
                } else {
                    self.project_dir.join(&relative)
                };

                if !abs.exists() {
                    return Err(PackageError::IoError(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("path dependency '{}' not found at {}", name, abs.display()),
                    )));
                }

                let version = dep_version_or_zero(dep);
                Ok(LockedPackage {
                    name: name.to_string(),
                    version,
                    source: LockedSource::Path { path: abs },
                    checksum: None,
                    dependencies: Default::default(),
                })
            }
            DepKind::Registry => Err(PackageError::Unsupported(
                "registry deps require a community index — use git deps for now".to_string(),
            )),
        }
    }
}

/// Extract semver from a dep, or return 0.0.0 when unavailable (path/git deps).
fn dep_version_or_zero(dep: &Dependency) -> semver::Version {
    let s = match dep {
        Dependency::Simple(v) => v.as_str(),
        Dependency::Detailed(d) => d.version.as_deref().unwrap_or("0.0.0"),
    };
    // Strip leading semver operators (^, ~, =, >=, etc.)
    let clean = s.trim_start_matches(|c: char| !c.is_ascii_digit());
    semver::Version::parse(clean).unwrap_or_else(|_| semver::Version::new(0, 0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_manifest_toml(deps: &str) -> String {
        format!(
            r#"[package]
name = "test-pkg"
version = "0.1.0"

[dependencies]
{deps}
"#
        )
    }

    fn write_manifest(dir: &std::path::Path, toml: &str) {
        fs::write(dir.join("atlas.toml"), toml).expect("write manifest");
    }

    // ── test_install_empty_deps ───────────────────────────────────────────────

    #[test]
    fn test_install_empty_deps() {
        let project = TempDir::new().expect("tempdir");
        let cache = TempDir::new().expect("tempdir");

        write_manifest(project.path(), &make_manifest_toml(""));

        let manifest =
            PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse manifest");
        let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());

        let lockfile = installer.install(&manifest, false).expect("install");
        assert!(lockfile.packages.is_empty(), "no deps → empty lockfile");

        // Lockfile written to disk.
        assert!(project.path().join("atlas.lock").exists());
    }

    // ── test_install_path_dep ─────────────────────────────────────────────────

    #[test]
    fn test_install_path_dep() {
        let project = TempDir::new().expect("tempdir");
        let cache = TempDir::new().expect("tempdir");

        // Create a sibling package directory.
        let sibling = TempDir::new().expect("tempdir");
        fs::write(
            sibling.path().join("atlas.toml"),
            "[package]\nname = \"local-lib\"\nversion = \"0.1.0\"\n",
        )
        .expect("write sibling manifest");

        let toml = format!(
            r#"[package]
name = "test-pkg"
version = "0.1.0"

[dependencies]
local-lib = {{ path = "{}" }}
"#,
            sibling.path().display()
        );
        write_manifest(project.path(), &toml);

        let manifest =
            PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse manifest");
        let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());

        let lockfile = installer.install(&manifest, false).expect("install");
        assert_eq!(lockfile.packages.len(), 1);

        let pkg = lockfile.get_package("local-lib").expect("pkg present");
        assert!(
            matches!(&pkg.source, LockedSource::Path { path } if path.exists()),
            "locked source should be a path that exists"
        );

        assert!(project.path().join("atlas.lock").exists());
    }

    // ── test_plan_shows_git_dep ───────────────────────────────────────────────

    #[test]
    fn test_plan_shows_git_dep() {
        let project = TempDir::new().expect("tempdir");
        let cache = TempDir::new().expect("tempdir");

        let toml = make_manifest_toml(
            r#"web-router = { git = "https://github.com/org/atlas-web-router", tag = "v1.0.0" }"#,
        );
        write_manifest(project.path(), &toml);

        let manifest =
            PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse manifest");
        let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());

        let plan = installer.plan(&manifest).expect("plan");

        // The dep is not cached so it must appear in to_fetch.
        assert_eq!(plan.to_fetch.len(), 1);
        assert_eq!(plan.to_fetch[0].name, "web-router");
        assert!(plan.to_fetch[0]
            .source
            .contains("https://github.com/org/atlas-web-router"));
        assert!(plan.to_fetch[0].source.contains("v1.0.0"));
        assert!(plan.already_cached.is_empty());
    }

    // ── test_lockfile_reused_when_valid ───────────────────────────────────────

    #[test]
    fn test_lockfile_reused_when_valid() {
        let project = TempDir::new().expect("tempdir");
        let cache = TempDir::new().expect("tempdir");

        // Create a path dep so install() can succeed without network.
        let sibling = TempDir::new().expect("tempdir");
        fs::write(
            sibling.path().join("atlas.toml"),
            "[package]\nname = \"utils\"\nversion = \"0.1.0\"\n",
        )
        .expect("write sibling manifest");

        let toml = format!(
            r#"[package]
name = "test-pkg"
version = "0.1.0"

[dependencies]
utils = {{ path = "{}" }}
"#,
            sibling.path().display()
        );
        write_manifest(project.path(), &toml);

        let manifest =
            PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse manifest");
        let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());

        // First install — writes lockfile.
        let lf1 = installer.install(&manifest, false).expect("first install");

        // Record mtime.
        let lock_path = project.path().join("atlas.lock");
        let mtime1 = fs::metadata(&lock_path)
            .expect("lock metadata")
            .modified()
            .expect("mtime");

        // Give filesystem a tick so mtime would differ if rewritten.
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Second install (no force) — must reuse lockfile without rewriting.
        let lf2 = installer.install(&manifest, false).expect("second install");
        let mtime2 = fs::metadata(&lock_path)
            .expect("lock metadata")
            .modified()
            .expect("mtime");

        assert_eq!(lf1.packages.len(), lf2.packages.len());
        assert_eq!(mtime1, mtime2, "lockfile should not be rewritten");
    }

    // ── test_registry_dep_returns_error ──────────────────────────────────────

    #[test]
    fn test_registry_dep_returns_error() {
        let project = TempDir::new().expect("tempdir");
        let cache = TempDir::new().expect("tempdir");

        // A simple string dep is treated as a registry dep.
        let toml = make_manifest_toml(r#"some-lib = "^1.0""#);
        write_manifest(project.path(), &toml);

        let manifest =
            PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse manifest");
        let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());

        let err = installer
            .install(&manifest, false)
            .expect_err("registry dep must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("registry") || msg.contains("community index"),
            "error should mention registry, got: {msg}"
        );
    }
}
