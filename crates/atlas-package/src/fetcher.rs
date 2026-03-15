//! Git-based package fetcher for Atlas.
//!
//! Atlas uses a Go-style package system: git repos are the registry.
//! Packages are cached locally under `~/atlas/pkg/<host>/<org>/<name>@<tag>/`,
//! namespaced by git host + org to prevent name collisions across orgs.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Fetches Atlas packages from git repositories.
pub struct GitFetcher {
    cache_dir: PathBuf,
}

/// Result of a successful package fetch.
pub struct FetchResult {
    /// Path where the package was extracted: `<cache_dir>/<name>/<tag>/`
    pub path: PathBuf,
    /// Resolved commit SHA (from `git rev-parse HEAD`)
    pub rev: String,
    /// SHA-256 of the fetched directory tree
    pub checksum: String,
}

/// Errors produced by [`GitFetcher`].
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("git command failed: {0}")]
    GitError(String),

    #[error("tag '{tag}' not found in {url}")]
    TagNotFound { url: String, tag: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Derive a namespaced cache subpath from a git URL and tag.
///
/// Mirrors Go's module cache layout: `<host>/<org>/<name>@<tag>`.
///
/// Examples:
/// - `https://github.com/atl-pkg/argus` + `v0.1.0` → `github.com/atl-pkg/argus@v0.1.0`
/// - `https://gitlab.com/myorg/utils`   + `v2.0.0` → `gitlab.com/myorg/utils@v2.0.0`
/// - Fallback (unrecognised URL):                   → `<name>@<tag>`
pub fn url_to_cache_subpath(url: &str, name: &str, tag: &str) -> PathBuf {
    // Strip scheme (https://, http://, git://, ssh://git@, etc.)
    let stripped = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("git://")
        .trim_start_matches("ssh://")
        // Handle `git@github.com:org/repo` SSH shorthand → `github.com/org/repo`
        .replace(':', "/")
        .trim_start_matches("git@")
        .to_string();

    // Strip trailing `.git` suffix
    let stripped = stripped.trim_end_matches(".git");

    if stripped.is_empty() || !stripped.contains('/') {
        // Unrecognised format — fall back to name@tag
        return PathBuf::from(format!("{}@{}", name, tag));
    }

    PathBuf::from(format!("{}@{}", stripped, tag))
}

impl GitFetcher {
    /// Create a new fetcher rooted at `pkg_dir` (e.g. `~/atlas/pkg/`).
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Return the canonical cache path for a package at a given URL + tag.
    ///
    /// Layout: `<pkg_dir>/<host>/<org>/<name>@<tag>/`
    fn package_path(&self, url: &str, name: &str, tag: &str) -> PathBuf {
        self.cache_dir.join(url_to_cache_subpath(url, name, tag))
    }

    /// Check whether the package directory already exists and is non-empty.
    pub fn is_cached(&self, url: &str, name: &str, tag: &str) -> bool {
        let path = self.package_path(url, name, tag);
        if !path.exists() {
            return false;
        }
        // Non-empty: at least one entry inside the directory
        std::fs::read_dir(&path)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    }

    /// Fetch a package from a git repo at a specific tag.
    ///
    /// Idempotent: if the package is already cached the existing data is returned
    /// without re-cloning.
    pub fn fetch(&self, name: &str, url: &str, tag: &str) -> Result<FetchResult, FetchError> {
        let target = self.package_path(url, name, tag);

        if self.is_cached(url, name, tag) {
            // Re-read cached rev and recompute checksum
            let rev = self.read_rev(&target)?;
            let checksum = compute_dir_checksum(&target)?;
            return Ok(FetchResult {
                path: target,
                rev,
                checksum,
            });
        }

        // Create target directory
        std::fs::create_dir_all(&target)?;

        // Clone at the specific tag with minimal history
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                tag,
                url,
                target.to_str().unwrap_or("."),
            ])
            .status()
            .map_err(|e| FetchError::GitError(format!("failed to spawn git clone: {}", e)))?;

        if !status.success() {
            // Clean up the empty directory we created
            let _ = std::fs::remove_dir_all(&target);
            return Err(FetchError::TagNotFound {
                url: url.to_string(),
                tag: tag.to_string(),
            });
        }

        let rev = self.read_rev(&target)?;
        let checksum = compute_dir_checksum(&target)?;

        Ok(FetchResult {
            path: target,
            rev,
            checksum,
        })
    }

    /// Read the resolved HEAD commit SHA from an already-cloned directory.
    fn read_rev(&self, dir: &Path) -> Result<String, FetchError> {
        let output = Command::new("git")
            .args(["-C", dir.to_str().unwrap_or("."), "rev-parse", "HEAD"])
            .output()
            .map_err(|e| FetchError::GitError(format!("failed to spawn git rev-parse: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FetchError::GitError(format!(
                "rev-parse failed: {}",
                stderr.trim()
            )));
        }

        let rev = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(rev)
    }

    /// List available tags from a remote repository.
    ///
    /// Runs `git ls-remote --tags <url>` — no local clone required.
    pub fn list_remote_tags(&self, url: &str) -> Result<Vec<String>, FetchError> {
        let output = Command::new("git")
            .args(["ls-remote", "--tags", url])
            .output()
            .map_err(|e| FetchError::GitError(format!("failed to spawn git ls-remote: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FetchError::GitError(format!(
                "ls-remote failed for {}: {}",
                url,
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let tags = parse_ls_remote_tags(&stdout);
        Ok(tags)
    }
}

/// Parse the output of `git ls-remote --tags` into a sorted list of tag names.
///
/// Each line looks like:
/// ```text
/// abc123\trefs/tags/v1.0.0
/// abc123\trefs/tags/v1.0.0^{}
/// ```
/// Entries ending with `^{}` are dereferenced tag objects and are filtered out.
pub fn parse_ls_remote_tags(output: &str) -> Vec<String> {
    let mut tags: Vec<String> = output
        .lines()
        .filter_map(|line| {
            let ref_part = line.split('\t').nth(1)?;
            let tag = ref_part.strip_prefix("refs/tags/")?;
            // Skip dereferenced tag objects
            if tag.ends_with("^{}") {
                return None;
            }
            Some(tag.to_string())
        })
        .collect();
    tags.sort();
    tags
}

/// Walk `dir`, collect all files sorted by relative path, hash all content with SHA-256.
fn compute_dir_checksum(dir: &Path) -> Result<String, FetchError> {
    let mut hasher = Sha256::new();

    let mut entries: Vec<PathBuf> = WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        // Exclude .git internals from the content hash
        .filter(|e| !e.path().components().any(|c| c.as_os_str() == ".git"))
        .map(|e| e.path().to_path_buf())
        .collect();

    entries.sort();

    for path in entries {
        // Feed relative path into hash so renames are detected
        if let Ok(rel) = path.strip_prefix(dir) {
            hasher.update(rel.to_string_lossy().as_bytes());
        }
        let contents = std::fs::read(&path)?;
        hasher.update(&contents);
    }

    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // ── url_to_cache_subpath ──────────────────────────────────────────────────

    #[test]
    fn test_url_to_cache_subpath_https() {
        let path = url_to_cache_subpath("https://github.com/atl-pkg/argus", "argus", "v0.1.0");
        assert_eq!(path, PathBuf::from("github.com/atl-pkg/argus@v0.1.0"));
    }

    #[test]
    fn test_url_to_cache_subpath_strips_git_suffix() {
        let path = url_to_cache_subpath("https://github.com/atl-pkg/argus.git", "argus", "v1.0.0");
        assert_eq!(path, PathBuf::from("github.com/atl-pkg/argus@v1.0.0"));
    }

    #[test]
    fn test_url_to_cache_subpath_ssh() {
        let path = url_to_cache_subpath("git@github.com:atl-pkg/web", "web", "v0.2.0");
        assert_eq!(path, PathBuf::from("github.com/atl-pkg/web@v0.2.0"));
    }

    #[test]
    fn test_url_to_cache_subpath_gitlab() {
        let path = url_to_cache_subpath("https://gitlab.com/myorg/utils", "utils", "v2.0.0");
        assert_eq!(path, PathBuf::from("gitlab.com/myorg/utils@v2.0.0"));
    }

    #[test]
    fn test_url_to_cache_subpath_different_orgs_no_collision() {
        let a = url_to_cache_subpath("https://github.com/atl-pkg/router", "router", "v1.0.0");
        let b = url_to_cache_subpath("https://github.com/other-org/router", "router", "v1.0.0");
        assert_ne!(a, b, "different orgs must produce different cache paths");
    }

    // ── is_cached ────────────────────────────────────────────────────────────

    const TEST_URL: &str = "https://github.com/atl-pkg/my-lib";

    #[test]
    fn test_is_cached_false_when_empty() {
        let tmp = TempDir::new().expect("tempdir");
        let fetcher = GitFetcher::new(tmp.path().to_path_buf());
        assert!(!fetcher.is_cached(TEST_URL, "my-lib", "v1.0.0"));
    }

    #[test]
    fn test_is_cached_false_for_empty_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let fetcher = GitFetcher::new(tmp.path().to_path_buf());
        let subpath = url_to_cache_subpath(TEST_URL, "my-lib", "v1.0.0");
        std::fs::create_dir_all(tmp.path().join(&subpath)).expect("create dir");
        assert!(!fetcher.is_cached(TEST_URL, "my-lib", "v1.0.0"));
    }

    #[test]
    fn test_is_cached_true_when_populated() {
        let tmp = TempDir::new().expect("tempdir");
        let fetcher = GitFetcher::new(tmp.path().to_path_buf());
        let subpath = url_to_cache_subpath(TEST_URL, "my-lib", "v1.0.0");
        let pkg_dir = tmp.path().join(&subpath);
        std::fs::create_dir_all(&pkg_dir).expect("create dir");
        std::fs::write(
            pkg_dir.join("atlas.toml"),
            b"[package]\nname = \"my-lib\"\n",
        )
        .expect("write file");
        assert!(fetcher.is_cached(TEST_URL, "my-lib", "v1.0.0"));
    }

    // ── parse_ls_remote_tags ──────────────────────────────────────────────────

    #[test]
    fn test_list_remote_tags_parse_basic() {
        let raw = "\
abc123\trefs/tags/v1.0.0\n\
def456\trefs/tags/v1.1.0\n\
ghi789\trefs/tags/v2.0.0\n";
        let tags = parse_ls_remote_tags(raw);
        assert_eq!(tags, vec!["v1.0.0", "v1.1.0", "v2.0.0"]);
    }

    #[test]
    fn test_list_remote_tags_parse_filters_deref() {
        // Lines ending with ^{} are dereferenced annotated tags — must be excluded
        let raw = "\
abc123\trefs/tags/v1.0.0\n\
abc123\trefs/tags/v1.0.0^{}\n\
def456\trefs/tags/v1.1.0\n\
def456\trefs/tags/v1.1.0^{}\n";
        let tags = parse_ls_remote_tags(raw);
        assert_eq!(tags, vec!["v1.0.0", "v1.1.0"]);
    }

    #[test]
    fn test_list_remote_tags_parse_empty() {
        let tags = parse_ls_remote_tags("");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_list_remote_tags_parse_sorted() {
        // Output order from git is not guaranteed; result must be sorted
        let raw = "\
zzz\trefs/tags/v3.0.0\n\
aaa\trefs/tags/v1.0.0\n\
mmm\trefs/tags/v2.0.0\n";
        let tags = parse_ls_remote_tags(raw);
        assert_eq!(tags, vec!["v1.0.0", "v2.0.0", "v3.0.0"]);
    }

    // ── FetchError display ────────────────────────────────────────────────────

    #[test]
    fn test_fetch_error_display_git_error() {
        let e = FetchError::GitError("exit code 128".to_string());
        assert_eq!(e.to_string(), "git command failed: exit code 128");
    }

    #[test]
    fn test_fetch_error_display_tag_not_found() {
        let e = FetchError::TagNotFound {
            url: "https://github.com/example/lib".to_string(),
            tag: "v99.0.0".to_string(),
        };
        assert_eq!(
            e.to_string(),
            "tag 'v99.0.0' not found in https://github.com/example/lib"
        );
    }

    #[test]
    fn test_fetch_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let e = FetchError::Io(io_err);
        assert!(e.to_string().starts_with("io error:"));
    }

    // ── compute_dir_checksum ──────────────────────────────────────────────────

    #[test]
    fn test_checksum_deterministic() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path();
        std::fs::write(dir.join("a.atlas"), b"let x = 1").expect("write");
        std::fs::write(dir.join("b.atlas"), b"let y = 2").expect("write");

        let c1 = compute_dir_checksum(dir).expect("checksum");
        let c2 = compute_dir_checksum(dir).expect("checksum");
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_checksum_changes_on_content_change() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path();
        let file = dir.join("main.atlas");
        std::fs::write(&file, b"let x = 1").expect("write");
        let c1 = compute_dir_checksum(dir).expect("checksum");

        std::fs::write(&file, b"let x = 2").expect("write");
        let c2 = compute_dir_checksum(dir).expect("checksum");

        assert_ne!(c1, c2);
    }
}
