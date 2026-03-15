//! B45 Integration Tests — hermetic git-native package system
//!
//! All tests use local file-protocol git repos in tmpdir. No live network required.
//! Tests cover: install, update, publish validation, lockfile integrity, error paths.

use atlas_package::fetcher::{parse_ls_remote_tags, GitFetcher};
use atlas_package::installer::Installer;
use atlas_package::manifest::PackageManifest;
use atlas_package::validator::Validator;
use atlas_package::{LockedSource, Lockfile};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// ── git helpers ───────────────────────────────────────────────────────────────

/// Create and initialise a bare-enough git repo that can serve as a package source.
fn make_git_package(name: &str, version: &str, tag: &str) -> TempDir {
    let dir = TempDir::new().expect("tmpdir");
    let p = dir.path();

    git(p, &["init"]);
    git(p, &["config", "user.email", "test@example.com"]);
    git(p, &["config", "user.name", "Test"]);

    // Minimal atlas.toml so the package looks real.
    std::fs::write(
        p.join("atlas.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"{version}\"\n\
             description = \"test\"\nlicense = \"MIT\"\n"
        ),
    )
    .expect("write manifest");

    // A source file that exports a constant.
    std::fs::write(
        p.join("index.atl"),
        format!("export const VALUE = \"{name}-{version}\";\n"),
    )
    .expect("write source");

    git(p, &["add", "."]);
    git(p, &["commit", "-m", "initial"]);
    git(p, &["tag", tag, "-m", &format!("release {tag}")]);

    dir
}

/// Add a new tagged commit to an existing repo dir.
fn add_tag(repo_dir: &Path, new_version: &str, new_tag: &str) {
    std::fs::write(
        repo_dir.join("atlas.toml"),
        format!(
            "[package]\nname = \"lib\"\nversion = \"{new_version}\"\n\
             description = \"test\"\nlicense = \"MIT\"\n"
        ),
    )
    .expect("write manifest");
    git(repo_dir, &["add", "."]);
    git(repo_dir, &["commit", "-m", &format!("release {new_tag}")]);
    git(
        repo_dir,
        &["tag", new_tag, "-m", &format!("release {new_tag}")],
    );
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git");
    assert!(
        status.success(),
        "git {:?} failed in {}",
        args,
        dir.display()
    );
}

fn file_url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

fn make_project(dir: &Path, deps_toml: &str) {
    let manifest = format!(
        "[package]\nname = \"my-app\"\nversion = \"0.1.0\"\n\n[dependencies]\n{deps_toml}\n"
    );
    std::fs::write(dir.join("atlas.toml"), manifest).expect("write manifest");
}

fn cache_dir() -> TempDir {
    TempDir::new().expect("cache tmpdir")
}

// ── install tests ─────────────────────────────────────────────────────────────

#[test]
fn test_install_git_dep_writes_lockfile() {
    let pkg = make_git_package("web-router", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("project tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "web-router = {{ git = \"{}\", tag = \"v1.0.0\" }}",
            file_url(pkg.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    let lockfile = installer.install(&manifest, false).expect("install");

    assert_eq!(lockfile.packages.len(), 1);
    assert!(project.path().join("atlas.lock").exists());
}

#[test]
fn test_install_git_dep_lockfile_has_correct_source() {
    let pkg = make_git_package("utils", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();
    let url = file_url(pkg.path());

    make_project(
        project.path(),
        &format!("utils = {{ git = \"{url}\", tag = \"v1.0.0\" }}"),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    let lockfile = installer.install(&manifest, false).expect("install");

    let pkg_entry = lockfile.get_package("utils").expect("utils in lockfile");
    match &pkg_entry.source {
        LockedSource::Git {
            url: locked_url,
            rev,
            ..
        } => {
            assert_eq!(locked_url, &url);
            assert!(!rev.is_empty(), "rev should be a non-empty commit hash");
        }
        other => panic!("expected Git source, got {:?}", other),
    }
}

#[test]
fn test_install_git_dep_populates_cache() {
    let pkg = make_git_package("logger", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "logger = {{ git = \"{}\", tag = \"v1.0.0\" }}",
            file_url(pkg.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    installer.install(&manifest, false).expect("install");

    // Cache should contain something under the package name.
    let fetcher = GitFetcher::new(cache.path().to_path_buf());
    assert!(
        fetcher.is_cached("logger", "v1.0.0"),
        "cache should be populated after install"
    );
}

#[test]
fn test_install_reuses_lockfile_on_second_call() {
    let pkg = make_git_package("math", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "math = {{ git = \"{}\", tag = \"v1.0.0\" }}",
            file_url(pkg.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());

    installer.install(&manifest, false).expect("first install");
    let lock_path = project.path().join("atlas.lock");
    let mtime1 = std::fs::metadata(&lock_path)
        .expect("meta")
        .modified()
        .expect("mtime");

    std::thread::sleep(std::time::Duration::from_millis(15));
    installer.install(&manifest, false).expect("second install");
    let mtime2 = std::fs::metadata(&lock_path)
        .expect("meta")
        .modified()
        .expect("mtime");

    assert_eq!(
        mtime1, mtime2,
        "lockfile must not be rewritten on second install"
    );
}

#[test]
fn test_install_force_refetches_even_with_lockfile() {
    let pkg = make_git_package("http", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "http = {{ git = \"{}\", tag = \"v1.0.0\" }}",
            file_url(pkg.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());

    installer.install(&manifest, false).expect("first install");

    let lock_path = project.path().join("atlas.lock");
    let mtime1 = std::fs::metadata(&lock_path)
        .expect("meta")
        .modified()
        .expect("mtime");
    std::thread::sleep(std::time::Duration::from_millis(15));

    installer.install(&manifest, true).expect("force install");
    let mtime2 = std::fs::metadata(&lock_path)
        .expect("meta")
        .modified()
        .expect("mtime");

    assert_ne!(mtime1, mtime2, "force must rewrite lockfile");
}

#[test]
fn test_install_missing_git_tag_fails() {
    let pkg = make_git_package("missing-tag", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "missing-tag = {{ git = \"{}\", tag = \"v99.0.0\" }}",
            file_url(pkg.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    let err = installer
        .install(&manifest, false)
        .expect_err("should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("v99.0.0") || msg.contains("not found") || msg.contains("tag"),
        "error should mention the missing tag, got: {msg}"
    );
}

#[test]
fn test_install_empty_deps_creates_empty_lockfile() {
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    std::fs::write(
        project.path().join("atlas.toml"),
        "[package]\nname = \"empty\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
    )
    .expect("write");

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    let lockfile = installer.install(&manifest, false).expect("install");

    assert!(lockfile.packages.is_empty());
    assert!(project.path().join("atlas.lock").exists());
}

#[test]
fn test_install_multiple_git_deps() {
    let pkg_a = make_git_package("pkg-a", "1.0.0", "v1.0.0");
    let pkg_b = make_git_package("pkg-b", "2.0.0", "v2.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "pkg-a = {{ git = \"{}\", tag = \"v1.0.0\" }}\npkg-b = {{ git = \"{}\", tag = \"v2.0.0\" }}",
            file_url(pkg_a.path()),
            file_url(pkg_b.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    let lockfile = installer.install(&manifest, false).expect("install");

    assert_eq!(lockfile.packages.len(), 2);
    assert!(lockfile.get_package("pkg-a").is_some());
    assert!(lockfile.get_package("pkg-b").is_some());
}

#[test]
fn test_install_checksum_set_on_git_dep() {
    let pkg = make_git_package("crypto", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "crypto = {{ git = \"{}\", tag = \"v1.0.0\" }}",
            file_url(pkg.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    let lockfile = installer.install(&manifest, false).expect("install");

    let entry = lockfile.get_package("crypto").expect("in lockfile");
    assert!(
        entry.checksum.is_some(),
        "checksum should be set for git dep"
    );
    let cs = entry.checksum.as_ref().expect("checksum");
    assert_eq!(cs.len(), 64, "SHA-256 hex is 64 chars");
}

// ── install plan tests ────────────────────────────────────────────────────────

#[test]
fn test_plan_identifies_uncached_git_dep() {
    let pkg = make_git_package("planner", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "planner = {{ git = \"{}\", tag = \"v1.0.0\" }}",
            file_url(pkg.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    let plan = installer.plan(&manifest).expect("plan");

    assert_eq!(plan.to_fetch.len(), 1);
    assert_eq!(plan.to_fetch[0].name, "planner");
    assert!(plan.already_cached.is_empty());
}

#[test]
fn test_plan_identifies_cached_git_dep() {
    let pkg = make_git_package("cached-lib", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();
    let url = file_url(pkg.path());

    make_project(
        project.path(),
        &format!("cached-lib = {{ git = \"{url}\", tag = \"v1.0.0\" }}"),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());

    // Pre-populate cache via install.
    installer.install(&manifest, false).expect("install");

    let plan = installer.plan(&manifest).expect("plan after install");
    assert!(
        plan.to_fetch.is_empty(),
        "nothing to fetch — already cached"
    );
    assert!(plan.already_cached.contains(&"cached-lib".to_string()));
}

// ── update tests ──────────────────────────────────────────────────────────────

#[test]
fn test_fetcher_list_remote_tags_local_repo() {
    let pkg = make_git_package("versioned", "1.0.0", "v1.0.0");
    add_tag(pkg.path(), "1.1.0", "v1.1.0");
    add_tag(pkg.path(), "1.2.0", "v1.2.0");

    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());
    let tags = fetcher
        .list_remote_tags(&file_url(pkg.path()))
        .expect("list tags");

    assert!(tags.contains(&"v1.0.0".to_string()));
    assert!(tags.contains(&"v1.1.0".to_string()));
    assert!(tags.contains(&"v1.2.0".to_string()));
}

#[test]
fn test_fetcher_list_remote_tags_sorted() {
    let pkg = make_git_package("sorted", "1.0.0", "v1.0.0");
    add_tag(pkg.path(), "1.2.0", "v1.2.0");
    add_tag(pkg.path(), "1.1.0", "v1.1.0");

    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());
    let tags = fetcher
        .list_remote_tags(&file_url(pkg.path()))
        .expect("list tags");

    // Sorted lexicographically.
    let v_tags: Vec<_> = tags.iter().filter(|t| t.starts_with('v')).collect();
    let mut sorted = v_tags.clone();
    sorted.sort();
    assert_eq!(v_tags, sorted);
}

#[test]
fn test_fetcher_fetch_then_list_tags_hermetic() {
    let pkg = make_git_package("fetch-list", "1.0.0", "v1.0.0");
    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());

    // Fetch first.
    fetcher
        .fetch("fetch-list", &file_url(pkg.path()), "v1.0.0")
        .expect("fetch");

    // List tags still works (no local clone needed).
    let tags = fetcher
        .list_remote_tags(&file_url(pkg.path()))
        .expect("list");
    assert!(tags.contains(&"v1.0.0".to_string()));
}

#[test]
fn test_fetcher_fetch_produces_rev() {
    let pkg = make_git_package("rev-check", "1.0.0", "v1.0.0");
    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());

    let result = fetcher
        .fetch("rev-check", &file_url(pkg.path()), "v1.0.0")
        .expect("fetch");

    // rev should be a 40-char hex SHA.
    assert_eq!(result.rev.len(), 40, "rev should be full SHA-1 hex");
    assert!(
        result.rev.chars().all(|c| c.is_ascii_hexdigit()),
        "rev should be hex"
    );
}

#[test]
fn test_fetcher_idempotent_second_fetch_returns_same_rev() {
    let pkg = make_git_package("idempotent", "1.0.0", "v1.0.0");
    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());
    let url = file_url(pkg.path());

    let r1 = fetcher
        .fetch("idempotent", &url, "v1.0.0")
        .expect("fetch 1");
    let r2 = fetcher
        .fetch("idempotent", &url, "v1.0.0")
        .expect("fetch 2");

    assert_eq!(r1.rev, r2.rev);
    assert_eq!(r1.checksum, r2.checksum);
}

#[test]
fn test_fetcher_different_tags_produce_different_revs() {
    let pkg = make_git_package("two-tags", "1.0.0", "v1.0.0");
    add_tag(pkg.path(), "1.1.0", "v1.1.0");

    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());
    let url = file_url(pkg.path());

    let r1 = fetcher.fetch("two-tags", &url, "v1.0.0").expect("fetch v1");
    let r2 = fetcher
        .fetch("two-tags", &url, "v1.1.0")
        .expect("fetch v1.1");

    assert_ne!(
        r1.rev, r2.rev,
        "different tags should point to different commits"
    );
}

// ── lockfile integrity tests ──────────────────────────────────────────────────

#[test]
fn test_lockfile_roundtrip_with_git_source() {
    let pkg = make_git_package("roundtrip", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();
    let url = file_url(pkg.path());

    make_project(
        project.path(),
        &format!("roundtrip = {{ git = \"{url}\", tag = \"v1.0.0\" }}"),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    installer.install(&manifest, false).expect("install");

    // Re-read lockfile from disk and verify it round-trips correctly.
    let lock_path = project.path().join("atlas.lock");
    let content = std::fs::read_to_string(&lock_path).expect("read lockfile");
    let parsed = Lockfile::from_str(&content).expect("parse lockfile");
    let entry = parsed
        .get_package("roundtrip")
        .expect("roundtrip in lockfile");
    assert!(matches!(entry.source, LockedSource::Git { .. }));
}

#[test]
fn test_lockfile_verify_passes_after_install() {
    let pkg = make_git_package("verified", "1.0.0", "v1.0.0");
    let project = TempDir::new().expect("tmpdir");
    let cache = cache_dir();

    make_project(
        project.path(),
        &format!(
            "verified = {{ git = \"{}\", tag = \"v1.0.0\" }}",
            file_url(pkg.path())
        ),
    );

    let manifest = PackageManifest::from_file(&project.path().join("atlas.toml")).expect("parse");
    let installer = Installer::new(cache.path().to_path_buf(), project.path().to_path_buf());
    let lockfile = installer.install(&manifest, false).expect("install");

    assert!(lockfile.verify().is_ok(), "lockfile should pass verify()");
}

// ── validator publish tests ───────────────────────────────────────────────────

#[test]
fn test_validate_for_publish_valid_manifest() {
    let toml = r#"[package]
name = "my-lib"
version = "1.0.0"
description = "A library"
license = "MIT"

[dependencies]
"#;
    let manifest = PackageManifest::from_str(toml).expect("parse");
    assert!(Validator::validate_for_publish(&manifest).is_ok());
}

#[test]
fn test_validate_for_publish_missing_license_fails() {
    let toml = r#"[package]
name = "my-lib"
version = "1.0.0"
description = "No license"

[dependencies]
"#;
    let manifest = PackageManifest::from_str(toml).expect("parse");
    let errs = Validator::validate_for_publish(&manifest).expect_err("should fail");
    assert!(errs.iter().any(|e| e.to_string().contains("license")));
}

#[test]
fn test_validate_for_publish_missing_description_fails() {
    let toml = r#"[package]
name = "my-lib"
version = "1.0.0"
license = "MIT"

[dependencies]
"#;
    let manifest = PackageManifest::from_str(toml).expect("parse");
    let errs = Validator::validate_for_publish(&manifest).expect_err("should fail");
    assert!(errs.iter().any(|e| e.to_string().contains("description")));
}

#[test]
fn test_validate_for_publish_zero_version_fails() {
    let toml = r#"[package]
name = "my-lib"
version = "0.0.0"
description = "Zero version"
license = "MIT"

[dependencies]
"#;
    let manifest = PackageManifest::from_str(toml).expect("parse");
    let errs = Validator::validate_for_publish(&manifest).expect_err("should fail");
    assert!(errs.iter().any(|e| e.to_string().contains("0.0.0")));
}

#[test]
fn test_validate_for_publish_multiple_errors_returned() {
    // Missing license AND description.
    let toml = "[package]\nname = \"x\"\nversion = \"0.0.0\"\n\n[dependencies]\n";
    let manifest = PackageManifest::from_str(toml).expect("parse");
    let errs = Validator::validate_for_publish(&manifest).expect_err("should fail");
    assert!(
        errs.len() >= 2,
        "should report both license and description missing, got {} errors",
        errs.len()
    );
}

#[test]
fn test_validate_for_publish_invalid_package_name_fails() {
    let toml = r#"[package]
name = "Invalid-Name"
version = "1.0.0"
description = "Bad name"
license = "MIT"

[dependencies]
"#;
    // Manifest parse succeeds; validation should catch the invalid name.
    let manifest = PackageManifest::from_str(toml).expect("parse");
    assert!(Validator::validate_for_publish(&manifest).is_err());
}

// ── parse_ls_remote_tags tests ────────────────────────────────────────────────

#[test]
fn test_parse_ls_remote_tags_mixed_refs() {
    let raw = "abc\trefs/heads/main\ndef\trefs/tags/v1.0.0\nghi\trefs/tags/v1.0.0^{}\n";
    let tags = parse_ls_remote_tags(raw);
    // Heads must not appear; deref entries must be filtered.
    assert_eq!(tags, vec!["v1.0.0"]);
}

#[test]
fn test_parse_ls_remote_tags_multiple_versions() {
    let raw = "a\trefs/tags/v1.0.0\nb\trefs/tags/v2.0.0\nc\trefs/tags/v1.5.0\n";
    let mut tags = parse_ls_remote_tags(raw);
    tags.sort();
    assert_eq!(tags, vec!["v1.0.0", "v1.5.0", "v2.0.0"]);
}

#[test]
fn test_parse_ls_remote_tags_no_tags_returns_empty() {
    let raw = "abc\trefs/heads/main\n";
    let tags = parse_ls_remote_tags(raw);
    assert!(tags.is_empty());
}

// ── fetcher cache tests ───────────────────────────────────────────────────────

#[test]
fn test_is_cached_false_before_fetch() {
    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());
    assert!(!fetcher.is_cached("never-fetched", "v1.0.0"));
}

#[test]
fn test_is_cached_true_after_fetch() {
    let pkg = make_git_package("cache-check", "1.0.0", "v1.0.0");
    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());

    fetcher
        .fetch("cache-check", &file_url(pkg.path()), "v1.0.0")
        .expect("fetch");
    assert!(fetcher.is_cached("cache-check", "v1.0.0"));
}

#[test]
fn test_is_cached_false_for_different_tag() {
    let pkg = make_git_package("tag-check", "1.0.0", "v1.0.0");
    let cache = cache_dir();
    let fetcher = GitFetcher::new(cache.path().to_path_buf());

    fetcher
        .fetch("tag-check", &file_url(pkg.path()), "v1.0.0")
        .expect("fetch");

    // v1.1.0 was never fetched.
    assert!(!fetcher.is_cached("tag-check", "v1.1.0"));
}
