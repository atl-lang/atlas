//! Module fingerprinting for incremental compilation
//!
//! A fingerprint uniquely identifies the compilation state of a module.
//! It combines source content hash, dependency fingerprints, compiler version,
//! platform info, and build configuration to determine when recompilation is needed.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::SystemTime;
use std::{env, fs};

/// Compiler version used in fingerprint computation
const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A fingerprint that uniquely identifies a module's compilation state
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fingerprint {
    /// Combined hash of all inputs
    pub hash: String,
    /// Source content hash
    pub source_hash: String,
    /// Dependency fingerprint hashes (sorted by name for determinism)
    pub dependency_hashes: BTreeMap<String, String>,
    /// Compiler version
    pub compiler_version: String,
    /// Target platform
    pub platform: PlatformInfo,
    /// Build configuration hash
    pub config_hash: String,
    /// Source file modification time
    pub mtime: Option<SystemTime>,
    /// Source file size
    pub file_size: u64,
}

/// Platform information included in fingerprints
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

impl PlatformInfo {
    /// Detect current platform
    pub fn current() -> Self {
        Self {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
        }
    }

    /// Create platform info for testing
    #[cfg(test)]
    pub fn test() -> Self {
        Self {
            os: "test-os".to_string(),
            arch: "test-arch".to_string(),
        }
    }

    fn to_hash_input(&self) -> String {
        format!("{}:{}", self.os, self.arch)
    }
}

/// Build configuration relevant to fingerprinting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    /// Optimization level string
    pub optimization: String,
    /// Whether to ignore comment-only changes
    pub ignore_comments: bool,
    /// Environment variables that affect compilation
    pub env_vars: BTreeMap<String, String>,
}

impl Default for FingerprintConfig {
    fn default() -> Self {
        Self {
            optimization: "O0".to_string(),
            ignore_comments: false,
            env_vars: BTreeMap::new(),
        }
    }
}

impl FingerprintConfig {
    /// Compute hash of this configuration
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.optimization.as_bytes());
        hasher.update(if self.ignore_comments { b"1" } else { b"0" });
        for (k, v) in &self.env_vars {
            hasher.update(k.as_bytes());
            hasher.update(b"=");
            hasher.update(v.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

/// Fingerprint database for tracking module fingerprints across builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintDb {
    /// Module name -> fingerprint
    fingerprints: BTreeMap<String, Fingerprint>,
    /// Compiler version when DB was created
    pub compiler_version: String,
    /// Platform when DB was created
    pub platform: PlatformInfo,
}

impl FingerprintDb {
    /// Create new empty database
    pub fn new() -> Self {
        Self {
            fingerprints: BTreeMap::new(),
            compiler_version: COMPILER_VERSION.to_string(),
            platform: PlatformInfo::current(),
        }
    }

    /// Load from disk
    pub fn load(path: &Path) -> Option<Self> {
        let data = fs::read_to_string(path).ok()?;
        let db: Self = serde_json::from_str(&data).ok()?;

        // Invalidate if compiler version changed
        if db.compiler_version != COMPILER_VERSION {
            return None;
        }

        // Invalidate if platform changed
        if db.platform != PlatformInfo::current() {
            return None;
        }

        Some(db)
    }

    /// Save to disk
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, data)
    }

    /// Get fingerprint for a module
    pub fn get(&self, module_name: &str) -> Option<&Fingerprint> {
        self.fingerprints.get(module_name)
    }

    /// Store fingerprint for a module
    pub fn insert(&mut self, module_name: String, fingerprint: Fingerprint) {
        self.fingerprints.insert(module_name, fingerprint);
    }

    /// Remove fingerprint for a module
    pub fn remove(&mut self, module_name: &str) -> Option<Fingerprint> {
        self.fingerprints.remove(module_name)
    }

    /// Get all module names
    pub fn module_names(&self) -> impl Iterator<Item = &String> {
        self.fingerprints.keys()
    }

    /// Number of tracked modules
    pub fn len(&self) -> usize {
        self.fingerprints.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }

    /// Clear all fingerprints
    pub fn clear(&mut self) {
        self.fingerprints.clear();
    }

    /// Check if a module needs recompilation by comparing fingerprints
    pub fn needs_recompile(&self, module_name: &str, current: &Fingerprint) -> bool {
        match self.get(module_name) {
            Some(stored) => stored.hash != current.hash,
            None => true, // No stored fingerprint = needs compile
        }
    }
}

impl Default for FingerprintDb {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a fingerprint for a source file
pub fn compute_fingerprint(
    source_path: &Path,
    source_content: &str,
    dependency_hashes: BTreeMap<String, String>,
    config: &FingerprintConfig,
) -> Fingerprint {
    let source_hash = if config.ignore_comments {
        compute_hash_without_comments(source_content)
    } else {
        compute_hash(source_content)
    };

    let platform = PlatformInfo::current();
    let config_hash = config.hash();

    // Read file metadata
    let (mtime, file_size) = fs::metadata(source_path)
        .map(|m| (m.modified().ok(), m.len()))
        .unwrap_or((None, 0));

    // Compute combined hash
    let combined_hash = compute_combined_hash(
        &source_hash,
        &dependency_hashes,
        COMPILER_VERSION,
        &platform,
        &config_hash,
    );

    Fingerprint {
        hash: combined_hash,
        source_hash,
        dependency_hashes,
        compiler_version: COMPILER_VERSION.to_string(),
        platform,
        config_hash,
        mtime,
        file_size,
    }
}

/// Compute fingerprint from content string (for testing or when path not available)
pub fn compute_fingerprint_from_content(
    source_content: &str,
    dependency_hashes: BTreeMap<String, String>,
    config: &FingerprintConfig,
) -> Fingerprint {
    let source_hash = if config.ignore_comments {
        compute_hash_without_comments(source_content)
    } else {
        compute_hash(source_content)
    };

    let platform = PlatformInfo::current();
    let config_hash = config.hash();

    let combined_hash = compute_combined_hash(
        &source_hash,
        &dependency_hashes,
        COMPILER_VERSION,
        &platform,
        &config_hash,
    );

    Fingerprint {
        hash: combined_hash,
        source_hash,
        dependency_hashes,
        compiler_version: COMPILER_VERSION.to_string(),
        platform,
        config_hash,
        mtime: None,
        file_size: source_content.len() as u64,
    }
}

/// Quick check: has the file changed based on mtime + size?
pub fn quick_check_changed(stored: &Fingerprint, path: &Path) -> bool {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return true, // Can't read = changed
    };

    let current_mtime = meta.modified().ok();
    let current_size = meta.len();

    // If mtime and size match, probably unchanged
    if stored.mtime == current_mtime && stored.file_size == current_size {
        return false;
    }

    true
}

/// Compute SHA-256 hash of content
pub fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Compute hash of content with comments stripped and trailing whitespace normalized
pub fn compute_hash_without_comments(content: &str) -> String {
    let stripped = strip_comments(content);
    // Normalize trailing whitespace on each line to avoid false differences
    let normalized: String = stripped
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    compute_hash(&normalized)
}

/// Strip single-line (//) and multi-line (/* */) comments from source
fn strip_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = '"';

    while i < len {
        // Handle string literals (don't strip comments inside strings)
        if !in_string && (chars[i] == '"' || chars[i] == '\'') {
            in_string = true;
            string_char = chars[i];
            result.push(chars[i]);
            i += 1;
            continue;
        }

        if in_string {
            if chars[i] == '\\' && i + 1 < len {
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if chars[i] == string_char {
                in_string = false;
            }
            result.push(chars[i]);
            i += 1;
            continue;
        }

        // Single-line comment
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            // Skip to end of line
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Multi-line comment
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < len {
                i += 2; // Skip */
            }
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Compute combined hash from all fingerprint components
fn compute_combined_hash(
    source_hash: &str,
    dependency_hashes: &BTreeMap<String, String>,
    compiler_version: &str,
    platform: &PlatformInfo,
    config_hash: &str,
) -> String {
    let mut hasher = Sha256::new();

    hasher.update(b"source:");
    hasher.update(source_hash.as_bytes());

    hasher.update(b"deps:");
    for (name, hash) in dependency_hashes {
        hasher.update(name.as_bytes());
        hasher.update(b"=");
        hasher.update(hash.as_bytes());
        hasher.update(b";");
    }

    hasher.update(b"compiler:");
    hasher.update(compiler_version.as_bytes());

    hasher.update(b"platform:");
    hasher.update(platform.to_hash_input().as_bytes());

    hasher.update(b"config:");
    hasher.update(config_hash.as_bytes());

    format!("{:x}", hasher.finalize())
}

/// Compute a dependency hash from a module path (for use in dependency_hashes maps)
pub fn compute_dependency_hash(source_path: &Path) -> Option<String> {
    let content = fs::read_to_string(source_path).ok()?;
    Some(compute_hash(&content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_deterministic() {
        let hash1 = compute_hash("fn main() {}");
        let hash2 = compute_hash("fn main() {}");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_different_content() {
        let hash1 = compute_hash("fn main() {}");
        let hash2 = compute_hash("fn main() { 42 }");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_strip_single_line_comments() {
        let source = "let x = 1; // comment\nlet y = 2;";
        let stripped = strip_comments(source);
        assert_eq!(stripped, "let x = 1; \nlet y = 2;");
    }

    #[test]
    fn test_strip_multi_line_comments() {
        let source = "let x = 1; /* block\ncomment */ let y = 2;";
        let stripped = strip_comments(source);
        assert_eq!(stripped, "let x = 1;  let y = 2;");
    }

    #[test]
    fn test_no_strip_comments_in_strings() {
        let source = r#"let x = "hello // world";"#;
        let stripped = strip_comments(source);
        assert_eq!(stripped, source);
    }

    #[test]
    fn test_comment_only_change_ignored() {
        let v1 = "let x = 1;\nlet y = 2;";
        let v2 = "let x = 1; // added comment\nlet y = 2;";
        let h1 = compute_hash_without_comments(v1);
        let h2 = compute_hash_without_comments(v2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_code_change_detected_with_comment_ignore() {
        let v1 = "let x = 1;";
        let v2 = "let x = 2;";
        let h1 = compute_hash_without_comments(v1);
        let h2 = compute_hash_without_comments(v2);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_fingerprint_config_default() {
        let config = FingerprintConfig::default();
        assert_eq!(config.optimization, "O0");
        assert!(!config.ignore_comments);
    }

    #[test]
    fn test_fingerprint_config_hash_deterministic() {
        let config = FingerprintConfig::default();
        let h1 = config.hash();
        let h2 = config.hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_fingerprint_config_hash_differs_on_opt() {
        let c1 = FingerprintConfig {
            optimization: "O0".to_string(),
            ..Default::default()
        };
        let c2 = FingerprintConfig {
            optimization: "O2".to_string(),
            ..Default::default()
        };
        assert_ne!(c1.hash(), c2.hash());
    }

    #[test]
    fn test_fingerprint_from_content() {
        let fp = compute_fingerprint_from_content(
            "fn main() {}",
            BTreeMap::new(),
            &FingerprintConfig::default(),
        );
        assert!(!fp.hash.is_empty());
        assert!(!fp.source_hash.is_empty());
        assert_eq!(fp.compiler_version, COMPILER_VERSION);
    }

    #[test]
    fn test_fingerprint_changes_with_deps() {
        let config = FingerprintConfig::default();
        let fp1 = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &config);

        let mut deps = BTreeMap::new();
        deps.insert("dep".to_string(), "abc123".to_string());
        let fp2 = compute_fingerprint_from_content("fn main() {}", deps, &config);

        assert_ne!(fp1.hash, fp2.hash);
    }

    #[test]
    fn test_fingerprint_db_new() {
        let db = FingerprintDb::new();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_fingerprint_db_insert_get() {
        let mut db = FingerprintDb::new();
        let fp = compute_fingerprint_from_content(
            "fn main() {}",
            BTreeMap::new(),
            &FingerprintConfig::default(),
        );
        db.insert("main".to_string(), fp.clone());

        assert_eq!(db.len(), 1);
        assert_eq!(db.get("main").unwrap().hash, fp.hash);
    }

    #[test]
    fn test_fingerprint_db_needs_recompile() {
        let mut db = FingerprintDb::new();
        let config = FingerprintConfig::default();

        let fp1 = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &config);
        db.insert("main".to_string(), fp1.clone());

        // Same fingerprint - no recompile needed
        assert!(!db.needs_recompile("main", &fp1));

        // Different fingerprint - needs recompile
        let fp2 = compute_fingerprint_from_content("fn main() { 42 }", BTreeMap::new(), &config);
        assert!(db.needs_recompile("main", &fp2));

        // Unknown module - needs compile
        assert!(db.needs_recompile("unknown", &fp1));
    }

    #[test]
    fn test_fingerprint_db_remove() {
        let mut db = FingerprintDb::new();
        let fp = compute_fingerprint_from_content(
            "fn main() {}",
            BTreeMap::new(),
            &FingerprintConfig::default(),
        );
        db.insert("main".to_string(), fp);
        assert_eq!(db.len(), 1);

        db.remove("main");
        assert_eq!(db.len(), 0);
        assert!(db.get("main").is_none());
    }

    #[test]
    fn test_fingerprint_db_clear() {
        let mut db = FingerprintDb::new();
        let config = FingerprintConfig::default();
        db.insert(
            "a".to_string(),
            compute_fingerprint_from_content("a", BTreeMap::new(), &config),
        );
        db.insert(
            "b".to_string(),
            compute_fingerprint_from_content("b", BTreeMap::new(), &config),
        );
        assert_eq!(db.len(), 2);

        db.clear();
        assert!(db.is_empty());
    }

    #[test]
    fn test_fingerprint_db_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("fingerprints.json");

        let config = FingerprintConfig::default();
        let fp = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &config);

        // Save
        {
            let mut db = FingerprintDb::new();
            db.insert("main".to_string(), fp.clone());
            db.save(&db_path).unwrap();
        }

        // Load
        {
            let db = FingerprintDb::load(&db_path).unwrap();
            assert_eq!(db.len(), 1);
            assert_eq!(db.get("main").unwrap().hash, fp.hash);
        }
    }

    #[test]
    fn test_platform_info_current() {
        let info = PlatformInfo::current();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn test_fingerprint_env_vars() {
        let c1 = FingerprintConfig {
            env_vars: BTreeMap::new(),
            ..Default::default()
        };
        let mut vars = BTreeMap::new();
        vars.insert("ATLAS_DEBUG".to_string(), "1".to_string());
        let c2 = FingerprintConfig {
            env_vars: vars,
            ..Default::default()
        };
        assert_ne!(c1.hash(), c2.hash());
    }

    #[test]
    fn test_quick_check_missing_file() {
        let fp = compute_fingerprint_from_content(
            "test",
            BTreeMap::new(),
            &FingerprintConfig::default(),
        );
        assert!(quick_check_changed(
            &fp,
            Path::new("/nonexistent/file.atlas")
        ));
    }

    #[test]
    fn test_fingerprint_with_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.atlas");
        fs::write(&file, "fn test() {}").unwrap();

        let fp = compute_fingerprint(
            &file,
            "fn test() {}",
            BTreeMap::new(),
            &FingerprintConfig::default(),
        );
        assert!(!fp.hash.is_empty());
        assert!(fp.mtime.is_some());
        assert!(fp.file_size > 0);
    }

    #[test]
    fn test_combined_hash_order_independent_of_insertion() {
        // BTreeMap is always sorted, so this tests determinism
        let mut deps1 = BTreeMap::new();
        deps1.insert("a".to_string(), "hash_a".to_string());
        deps1.insert("b".to_string(), "hash_b".to_string());

        let mut deps2 = BTreeMap::new();
        deps2.insert("b".to_string(), "hash_b".to_string());
        deps2.insert("a".to_string(), "hash_a".to_string());

        let config = FingerprintConfig::default();
        let fp1 = compute_fingerprint_from_content("test", deps1, &config);
        let fp2 = compute_fingerprint_from_content("test", deps2, &config);
        assert_eq!(fp1.hash, fp2.hash);
    }
}
