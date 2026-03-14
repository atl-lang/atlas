//! Module resolution
//!
//! Resolves module paths and detects circular dependencies.
//! This is BLOCKER 04-A - syntax and resolution only.
//! Actual loading and execution happens in BLOCKER 04-B, 04-C, 04-D.

// Allow large error variants (Diagnostic) - consistent with rest of codebase
#![allow(clippy::result_large_err)]

use crate::diagnostic::error_codes::{CIRCULAR_DEPENDENCY, INVALID_MODULE_PATH, MODULE_NOT_FOUND};
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Minimal inline lockfile reader (avoids pulling in atlas-package + reqwest)
// ---------------------------------------------------------------------------

/// Minimal subset of atlas.lock needed for bare specifier resolution.
#[derive(Debug, Deserialize)]
struct MinLockfile {
    #[serde(default)]
    packages: Vec<MinLockedPackage>,
}

#[derive(Debug, Deserialize)]
struct MinLockedPackage {
    name: String,
    version: String,
}

impl MinLockfile {
    fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {}", path.display(), e))?;
        toml::from_str(&content).map_err(|e| format!("parse error in {}: {}", path.display(), e))
    }

    fn get_package(&self, name: &str) -> Option<&MinLockedPackage> {
        self.packages.iter().find(|p| p.name == name)
    }
}

/// Module resolver - handles path resolution and circular dependency detection
pub struct ModuleResolver {
    /// Root directory for absolute paths
    root: PathBuf,
    /// Cache of resolved module paths (source path -> absolute path)
    path_cache: HashMap<String, PathBuf>,
    /// Module dependency graph for circular detection
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
}

impl ModuleResolver {
    /// Create a new module resolver with the given root directory
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            path_cache: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Resolve a module path to an absolute file path
    ///
    /// # Arguments
    /// * `source` - The module path from import statement (e.g., "./math", "/src/utils")
    /// * `importing_file` - The file that contains the import statement
    ///
    /// # Returns
    /// The absolute path to the module file, or an error if not found
    pub fn resolve_path(
        &mut self,
        source: &str,
        importing_file: &Path,
        span: Span,
    ) -> Result<PathBuf, Diagnostic> {
        // Check cache first
        let cache_key = format!("{}:{}", importing_file.display(), source);
        if let Some(cached) = self.path_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let base_path = if source.starts_with('/') {
            // Absolute path: resolve from root
            self.resolve_absolute(source)?
        } else if source.starts_with("./") || source.starts_with("../") {
            // Relative path: resolve from importing file's directory
            self.resolve_relative(source, importing_file)?
        } else if !source.contains('/') {
            // Bare package name: 'web', 'http-router', etc.
            return self.resolve_package(source, span);
        } else {
            return Err(INVALID_MODULE_PATH.emit(span)
                .arg("path", source)
                .with_help("use './file' for same directory, '../file' for parent, or '/src/file' for absolute paths")
                .build());
        };

        let candidates = self.build_candidates(&base_path, source);
        let resolved = candidates
            .iter()
            .find(|path| path.exists())
            .cloned()
            .unwrap_or_else(|| candidates[0].clone());

        // Verify file exists
        if !resolved.exists() {
            let label = if candidates.len() == 1 {
                format!("resolved to: {}", resolved.display())
            } else {
                let tried = candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tried: {}", tried)
            };
            return Err(MODULE_NOT_FOUND
                .emit(span)
                .arg("path", source)
                .with_help("check that the file exists and the path is correct")
                .build()
                .with_label(label));
        }

        // Canonicalize to get consistent paths (fixes cycle detection with ./.. components)
        let resolved = resolved.canonicalize().map_err(|e| {
            MODULE_NOT_FOUND
                .emit(span)
                .arg("path", resolved.display().to_string())
                .with_help(format!("OS error: {e}"))
                .build()
                .with_label(format!("path: {}", resolved.display()))
        })?;

        // Cache the resolved path
        self.path_cache.insert(cache_key, resolved.clone());

        Ok(resolved)
    }

    /// Resolve an absolute path (starts with '/')
    fn resolve_absolute(&self, source: &str) -> Result<PathBuf, Diagnostic> {
        // Remove leading '/'
        let relative = &source[1..];
        Ok(self.root.join(relative))
    }

    /// Resolve a relative path (starts with './' or '../')
    fn resolve_relative(&self, source: &str, importing_file: &Path) -> Result<PathBuf, Diagnostic> {
        // Get directory of importing file
        let importing_dir = importing_file.parent().unwrap_or(Path::new("."));
        // Resolve relative to importing directory (canonicalized later in resolve_path)
        Ok(importing_dir.join(source))
    }

    fn build_candidates(&self, base_path: &Path, source: &str) -> Vec<PathBuf> {
        let has_extension = Path::new(source)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some();

        if has_extension {
            vec![base_path.to_path_buf()]
        } else {
            vec![
                base_path.with_extension("atlas"),
                base_path.with_extension("atl"),
            ]
        }
    }

    /// Add a module dependency to the graph
    ///
    /// This is used to track which modules import which, for circular detection.
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.dependencies.entry(from).or_default().push(to);
    }

    /// Check for circular dependencies starting from a given module
    ///
    /// Returns an error if a cycle is detected, with the cycle path for debugging.
    pub fn check_circular(&self, start: &Path, span: Span) -> Result<(), Diagnostic> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        if let Some(cycle) = self.find_cycle(start, &mut visited, &mut path) {
            let cycle_str = cycle
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");

            return Err(CIRCULAR_DEPENDENCY
                .emit(span)
                .arg("cycle", &cycle_str)
                .build()
                .with_label(format!("cycle: {}", cycle_str)));
        }

        Ok(())
    }

    /// Depth-first search to find cycles in the dependency graph
    fn find_cycle(
        &self,
        current: &Path,
        visited: &mut HashSet<PathBuf>,
        path: &mut Vec<PathBuf>,
    ) -> Option<Vec<PathBuf>> {
        let current_buf = current.to_path_buf();

        // If we've seen this node in the current path, we have a cycle
        if path.contains(&current_buf) {
            let cycle_start = path.iter().position(|p| p == &current_buf).unwrap();
            let mut cycle = path[cycle_start..].to_vec();
            cycle.push(current_buf);
            return Some(cycle);
        }

        // If we've visited this node before (but not in current path), skip it
        if visited.contains(&current_buf) {
            return None;
        }

        // Mark as visited and add to current path
        visited.insert(current_buf.clone());
        path.push(current_buf.clone());

        // Check all dependencies
        if let Some(deps) = self.dependencies.get(&current_buf) {
            for dep in deps {
                if let Some(cycle) = self.find_cycle(dep, visited, path) {
                    return Some(cycle);
                }
            }
        }

        // Remove from current path (backtrack)
        path.pop();

        None
    }

    /// Get all dependencies of a module (for debugging/testing)
    pub fn get_dependencies(&self, module: &Path) -> Vec<PathBuf> {
        self.dependencies
            .get(&module.to_path_buf())
            .cloned()
            .unwrap_or_default()
    }

    /// Clear the resolver state (for testing)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.path_cache.clear();
        self.dependencies.clear();
    }

    /// Resolve a bare package specifier (e.g., `'web'`, `'http-router'`) using atlas.lock
    fn resolve_package(&self, name: &str, span: Span) -> Result<PathBuf, Diagnostic> {
        // 1. Find atlas.lock by walking up from project root
        let lockfile_path = self.find_lockfile(span)?;

        // 2. Parse the lockfile
        let lockfile = MinLockfile::from_file(&lockfile_path).map_err(|e| {
            MODULE_NOT_FOUND
                .emit(span)
                .arg("path", name)
                .with_help(format!(
                    "failed to read atlas.lock: {} — run: atlas install",
                    e
                ))
                .build()
        })?;

        // 3. Look up package in lockfile
        let locked_pkg = lockfile.get_package(name).ok_or_else(|| {
            MODULE_NOT_FOUND
                .emit(span)
                .arg("path", name)
                .with_help(format!(
                    "package \"{}\" not found in atlas.lock — run: atlas install",
                    name
                ))
                .build()
        })?;

        // 4. Determine cache dir
        let cache_dir = std::env::var("ATLAS_CACHE_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join(".atlas")
                    .join("cache")
            });

        // 5. Get version from lockfile entry
        let version = locked_pkg.version.to_string();
        let pkg_dir = cache_dir.join(name).join(&version);

        // 6. Try entry point candidates: lib.atlas, index.atlas, mod.atlas
        let candidates = ["lib.atlas", "index.atlas", "mod.atlas"];
        for candidate in &candidates {
            let path = pkg_dir.join(candidate);
            if path.exists() {
                return Ok(path);
            }
        }

        // 7. Package is in lockfile but not in cache
        Err(MODULE_NOT_FOUND
            .emit(span)
            .arg("path", name)
            .with_help(format!(
                "package \"{}\" is in atlas.lock but not in cache — run: atlas install",
                name
            ))
            .build())
    }

    /// Walk up from self.root looking for atlas.lock
    fn find_lockfile(&self, span: Span) -> Result<std::path::PathBuf, Diagnostic> {
        let mut current = self.root.clone();
        loop {
            let candidate = current.join("atlas.lock");
            if candidate.exists() {
                return Ok(candidate);
            }
            if !current.pop() {
                break;
            }
        }
        Err(MODULE_NOT_FOUND
            .emit(span)
            .arg("path", "atlas.lock")
            .with_help("no atlas.lock found — run: atlas install")
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    fn make_resolver(root: &std::path::Path) -> ModuleResolver {
        ModuleResolver::new(root.to_path_buf())
    }

    /// Bare specifier with no atlas.lock present → error containing "atlas install"
    #[test]
    fn test_bare_specifier_no_lockfile() {
        let dir = TempDir::new().expect("tempdir");
        let resolver = make_resolver(dir.path());
        let result = resolver.resolve_package("web", dummy_span());
        assert!(result.is_err());
        let err = result.expect_err("expected error");
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("atlas install"),
            "expected 'atlas install' in error, got: {}",
            msg
        );
    }

    /// Relative imports (./foo) must still resolve through resolve_path normally
    #[test]
    fn test_relative_import_unchanged() {
        let dir = TempDir::new().expect("tempdir");
        // Create a file to resolve to
        let file_path = dir.path().join("foo.atlas");
        fs::write(&file_path, "// test").expect("write foo.atlas");

        let mut resolver = make_resolver(dir.path());
        // Use a fake importing file inside the same dir
        let importing = dir.path().join("main.atlas");
        fs::write(&importing, "// main").expect("write main.atlas");

        let result = resolver.resolve_path("./foo", &importing, dummy_span());
        assert!(result.is_ok(), "relative import failed: {:?}", result.err());
        let resolved = result.expect("resolved path");
        assert!(resolved.ends_with("foo.atlas"));
    }

    /// Absolute imports (/src/foo) must still resolve through resolve_path normally
    #[test]
    fn test_absolute_import_unchanged() {
        let dir = TempDir::new().expect("tempdir");
        let src_dir = dir.path().join("src");
        fs::create_dir_all(&src_dir).expect("create src dir");
        let file_path = src_dir.join("foo.atlas");
        fs::write(&file_path, "// test").expect("write foo.atlas");

        let mut resolver = make_resolver(dir.path());
        let importing = dir.path().join("main.atlas");
        fs::write(&importing, "// main").expect("write main.atlas");

        let result = resolver.resolve_path("/src/foo", &importing, dummy_span());
        assert!(result.is_ok(), "absolute import failed: {:?}", result.err());
        let resolved = result.expect("resolved path");
        assert!(resolved.ends_with("foo.atlas"));
    }
}
