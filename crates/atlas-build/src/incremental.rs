//! Incremental compilation engine
//!
//! Orchestrates fingerprinting, change detection, dependency tracking,
//! selective recompilation, and build state persistence for fast rebuilds.

use crate::build_order::BuildGraph;
use crate::error::{BuildError, BuildResult};
use crate::fingerprint::{compute_fingerprint, Fingerprint, FingerprintConfig, FingerprintDb};

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Result of incremental analysis: which modules need recompilation
#[derive(Debug)]
pub struct IncrementalPlan {
    /// Modules that need recompilation
    pub recompile: Vec<String>,
    /// Modules that can use cached results
    pub cached: Vec<String>,
    /// Reason each module needs recompilation
    pub reasons: HashMap<String, RecompileReason>,
    /// Total modules in the build
    pub total_modules: usize,
}

impl IncrementalPlan {
    /// Check if any modules need recompilation
    pub fn has_work(&self) -> bool {
        !self.recompile.is_empty()
    }

    /// Get recompile ratio (0.0 = all cached, 1.0 = full rebuild)
    pub fn recompile_ratio(&self) -> f64 {
        if self.total_modules == 0 {
            return 0.0;
        }
        self.recompile.len() as f64 / self.total_modules as f64
    }
}

/// Reason a module needs recompilation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecompileReason {
    /// First time compiling this module
    NoPreviousFingerprint,
    /// Source content changed
    SourceChanged,
    /// A dependency's fingerprint changed
    DependencyChanged(String),
    /// Build configuration changed
    ConfigChanged,
    /// Module was explicitly invalidated
    ManualInvalidation,
}

/// Build state that persists between invocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildState {
    /// Dependency graph edges (module -> dependencies)
    pub dependencies: BTreeMap<String, Vec<String>>,
    /// Module paths
    pub module_paths: BTreeMap<String, PathBuf>,
    /// Last successful build time
    pub last_build_time: Option<SystemTime>,
    /// State format version
    pub version: u32,
}

const BUILD_STATE_VERSION: u32 = 1;

impl BuildState {
    /// Create new empty state
    pub fn new() -> Self {
        Self {
            dependencies: BTreeMap::new(),
            module_paths: BTreeMap::new(),
            last_build_time: None,
            version: BUILD_STATE_VERSION,
        }
    }

    /// Load from disk, returning None if missing/corrupt/incompatible
    pub fn load(path: &Path) -> Option<Self> {
        let data = fs::read_to_string(path).ok()?;
        let state: Self = serde_json::from_str(&data).ok()?;
        if state.version != BUILD_STATE_VERSION {
            return None;
        }
        Some(state)
    }

    /// Save to disk
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, data)
    }

    /// Update from a build graph
    pub fn update_from_graph(&mut self, graph: &BuildGraph) {
        self.dependencies.clear();
        self.module_paths.clear();

        for (name, node) in graph.modules() {
            self.dependencies
                .insert(name.clone(), node.dependencies.clone());
            self.module_paths.insert(name.clone(), node.path.clone());
        }

        self.last_build_time = Some(SystemTime::now());
    }

    /// Detect modules that were added or removed since last build
    pub fn diff_modules(&self, current_modules: &HashSet<String>) -> ModuleDiff {
        let previous: HashSet<String> = self.module_paths.keys().cloned().collect();

        ModuleDiff {
            added: current_modules.difference(&previous).cloned().collect(),
            removed: previous.difference(current_modules).cloned().collect(),
            retained: current_modules.intersection(&previous).cloned().collect(),
        }
    }
}

impl Default for BuildState {
    fn default() -> Self {
        Self::new()
    }
}

/// Difference between previous and current module sets
#[derive(Debug)]
pub struct ModuleDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub retained: Vec<String>,
}

/// The incremental compilation engine
pub struct IncrementalEngine {
    /// Fingerprint database
    fingerprint_db: FingerprintDb,
    /// Build state
    build_state: BuildState,
    /// Fingerprint configuration
    config: FingerprintConfig,
    /// Directory for persistent state
    state_dir: PathBuf,
}

impl IncrementalEngine {
    /// Create a new engine, loading persisted state if available
    pub fn new(state_dir: PathBuf, config: FingerprintConfig) -> Self {
        let fp_path = state_dir.join("fingerprints.json");
        let state_path = state_dir.join("build_state.json");

        let fingerprint_db = FingerprintDb::load(&fp_path).unwrap_or_default();
        let build_state = BuildState::load(&state_path).unwrap_or_default();

        Self {
            fingerprint_db,
            build_state,
            config,
            state_dir,
        }
    }

    /// Create with empty state (for testing)
    pub fn new_empty(state_dir: PathBuf) -> Self {
        Self::new(state_dir, FingerprintConfig::default())
    }

    /// Analyze the build graph and determine what needs recompilation
    pub fn plan(&self, graph: &BuildGraph) -> BuildResult<IncrementalPlan> {
        let mut recompile = Vec::new();
        let mut cached = Vec::new();
        let mut reasons = HashMap::new();
        let modules = graph.modules();

        // Phase 1: Compute current fingerprints and check direct changes
        let mut current_fingerprints: HashMap<String, Fingerprint> = HashMap::new();
        let mut directly_changed: HashSet<String> = HashSet::new();

        for (name, node) in modules {
            let source =
                fs::read_to_string(&node.path).map_err(|e| BuildError::io(&node.path, e))?;

            // Gather dependency hashes
            let dep_hashes: BTreeMap<String, String> = node
                .dependencies
                .iter()
                .filter_map(|dep| {
                    // Use stored fingerprint hash for dependency
                    self.fingerprint_db
                        .get(dep)
                        .map(|fp| (dep.clone(), fp.hash.clone()))
                })
                .collect();

            let fp = compute_fingerprint(&node.path, &source, dep_hashes, &self.config);

            if self.fingerprint_db.needs_recompile(name, &fp) {
                directly_changed.insert(name.clone());
            }

            current_fingerprints.insert(name.clone(), fp);
        }

        // Phase 2: Propagate invalidation through dependency graph
        let all_invalidated = propagate_invalidation(&directly_changed, modules);

        // Phase 3: Classify modules
        for name in modules.keys() {
            if all_invalidated.contains(name) {
                let reason = if directly_changed.contains(name) {
                    if self.fingerprint_db.get(name).is_none() {
                        RecompileReason::NoPreviousFingerprint
                    } else {
                        RecompileReason::SourceChanged
                    }
                } else {
                    // Find which dependency caused invalidation
                    let dep_cause = find_invalidation_cause(name, &directly_changed, modules);
                    RecompileReason::DependencyChanged(dep_cause)
                };
                reasons.insert(name.clone(), reason);
                recompile.push(name.clone());
            } else {
                cached.push(name.clone());
            }
        }

        Ok(IncrementalPlan {
            recompile,
            cached,
            reasons,
            total_modules: modules.len(),
        })
    }

    /// Record a successful compilation for a module
    pub fn record_compilation(
        &mut self,
        module_name: &str,
        source_path: &Path,
        source_content: &str,
        dependency_hashes: BTreeMap<String, String>,
    ) {
        let fp = compute_fingerprint(source_path, source_content, dependency_hashes, &self.config);
        self.fingerprint_db.insert(module_name.to_string(), fp);
    }

    /// Update build state from the current build graph
    pub fn update_state(&mut self, graph: &BuildGraph) {
        self.build_state.update_from_graph(graph);
    }

    /// Persist all state to disk
    pub fn save(&self) -> BuildResult<()> {
        fs::create_dir_all(&self.state_dir).map_err(|e| {
            BuildError::BuildFailed(format!("Failed to create state directory: {}", e))
        })?;

        self.fingerprint_db
            .save(&self.state_dir.join("fingerprints.json"))
            .map_err(|e| BuildError::BuildFailed(format!("Failed to save fingerprints: {}", e)))?;

        self.build_state
            .save(&self.state_dir.join("build_state.json"))
            .map_err(|e| BuildError::BuildFailed(format!("Failed to save build state: {}", e)))?;

        Ok(())
    }

    /// Get fingerprint database (for reading)
    pub fn fingerprint_db(&self) -> &FingerprintDb {
        &self.fingerprint_db
    }

    /// Get build state (for reading)
    pub fn build_state(&self) -> &BuildState {
        &self.build_state
    }

    /// Get the current fingerprint config
    pub fn config(&self) -> &FingerprintConfig {
        &self.config
    }

    /// Force invalidation of a module
    pub fn invalidate_module(&mut self, module_name: &str) {
        self.fingerprint_db.remove(module_name);
    }

    /// Invalidate all modules (force full rebuild)
    pub fn invalidate_all(&mut self) {
        self.fingerprint_db.clear();
    }

    /// Get the last successful build time
    pub fn last_build_time(&self) -> Option<SystemTime> {
        self.build_state.last_build_time
    }
}

/// Propagate invalidation from directly changed modules through the dependency graph.
/// Returns all modules that need recompilation (directly changed + transitive dependents).
fn propagate_invalidation(
    directly_changed: &HashSet<String>,
    modules: &HashMap<String, crate::build_order::ModuleNode>,
) -> HashSet<String> {
    let mut invalidated = directly_changed.clone();

    // Build reverse dependency map (module -> dependents)
    let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();
    for (name, node) in modules {
        for dep in &node.dependencies {
            reverse_deps
                .entry(dep.clone())
                .or_default()
                .push(name.clone());
        }
    }

    // BFS propagation
    let mut queue: Vec<String> = directly_changed.iter().cloned().collect();
    while let Some(module) = queue.pop() {
        if let Some(dependents) = reverse_deps.get(&module) {
            for dependent in dependents {
                if invalidated.insert(dependent.clone()) {
                    queue.push(dependent.clone());
                }
            }
        }
    }

    invalidated
}

/// Find which dependency caused a module to be invalidated
fn find_invalidation_cause(
    module: &str,
    directly_changed: &HashSet<String>,
    modules: &HashMap<String, crate::build_order::ModuleNode>,
) -> String {
    if let Some(node) = modules.get(module) {
        for dep in &node.dependencies {
            if directly_changed.contains(dep) {
                return dep.clone();
            }
        }
        // Transitive - return first dependency
        if let Some(dep) = node.dependencies.first() {
            return dep.clone();
        }
    }
    "unknown".to_string()
}

/// Incremental build statistics
#[derive(Debug, Clone)]
pub struct IncrementalStats {
    /// Total modules in the project
    pub total_modules: usize,
    /// Modules recompiled this build
    pub recompiled: usize,
    /// Modules served from cache
    pub from_cache: usize,
    /// Time spent on analysis
    pub analysis_time: Duration,
    /// Time spent on compilation
    pub compilation_time: Duration,
    /// Estimated time saved vs full rebuild
    pub time_saved: Duration,
    /// Whether this was a full rebuild
    pub was_full_rebuild: bool,
}

impl IncrementalStats {
    /// Cache hit rate as a percentage
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_modules == 0 {
            return 0.0;
        }
        self.from_cache as f64 / self.total_modules as f64 * 100.0
    }

    /// Format as a human-readable summary
    pub fn summary(&self) -> String {
        if self.was_full_rebuild {
            format!(
                "Full rebuild: {} modules in {:.2}s",
                self.total_modules,
                self.compilation_time.as_secs_f64()
            )
        } else {
            format!(
                "Incremental: {}/{} recompiled ({:.0}% cached) in {:.2}s (saved ~{:.2}s)",
                self.recompiled,
                self.total_modules,
                self.cache_hit_rate(),
                self.compilation_time.as_secs_f64(),
                self.time_saved.as_secs_f64()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_order::ModuleNode;
    use std::path::PathBuf;

    fn make_graph(modules: Vec<(&str, Vec<&str>)>) -> BuildGraph {
        let mut graph = BuildGraph::new();
        for (name, deps) in modules {
            let node = ModuleNode::new(name, PathBuf::from(format!("{}.atlas", name)))
                .with_dependencies(deps.into_iter().map(String::from).collect());
            graph.add_module(node);
        }
        graph
    }

    #[test]
    fn test_propagate_invalidation_single() {
        let graph = make_graph(vec![("main", vec![])]);
        let mut changed = HashSet::new();
        changed.insert("main".to_string());

        let result = propagate_invalidation(&changed, graph.modules());
        assert_eq!(result.len(), 1);
        assert!(result.contains("main"));
    }

    #[test]
    fn test_propagate_invalidation_chain() {
        let graph = make_graph(vec![("a", vec!["b"]), ("b", vec!["c"]), ("c", vec![])]);
        let mut changed = HashSet::new();
        changed.insert("c".to_string());

        let result = propagate_invalidation(&changed, graph.modules());
        assert_eq!(result.len(), 3);
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(result.contains("c"));
    }

    #[test]
    fn test_propagate_invalidation_partial() {
        // a -> b, c -> d  (two independent chains)
        let graph = make_graph(vec![
            ("a", vec!["b"]),
            ("b", vec![]),
            ("c", vec!["d"]),
            ("d", vec![]),
        ]);
        let mut changed = HashSet::new();
        changed.insert("b".to_string());

        let result = propagate_invalidation(&changed, graph.modules());
        assert_eq!(result.len(), 2);
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(!result.contains("c"));
        assert!(!result.contains("d"));
    }

    #[test]
    fn test_propagate_invalidation_diamond() {
        // a -> [b, c] -> d
        let graph = make_graph(vec![
            ("a", vec!["b", "c"]),
            ("b", vec!["d"]),
            ("c", vec!["d"]),
            ("d", vec![]),
        ]);
        let mut changed = HashSet::new();
        changed.insert("d".to_string());

        let result = propagate_invalidation(&changed, graph.modules());
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_build_state_new() {
        let state = BuildState::new();
        assert!(state.dependencies.is_empty());
        assert!(state.module_paths.is_empty());
        assert!(state.last_build_time.is_none());
    }

    #[test]
    fn test_build_state_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");

        let mut state = BuildState::new();
        state
            .dependencies
            .insert("main".to_string(), vec!["lib".to_string()]);
        state
            .module_paths
            .insert("main".to_string(), PathBuf::from("src/main.atlas"));
        state.last_build_time = Some(SystemTime::now());
        state.save(&path).unwrap();

        let loaded = BuildState::load(&path).unwrap();
        assert_eq!(loaded.dependencies.len(), 1);
        assert_eq!(loaded.module_paths.len(), 1);
        assert!(loaded.last_build_time.is_some());
    }

    #[test]
    fn test_build_state_diff_modules() {
        let mut state = BuildState::new();
        state
            .module_paths
            .insert("a".to_string(), PathBuf::from("a.atlas"));
        state
            .module_paths
            .insert("b".to_string(), PathBuf::from("b.atlas"));

        let mut current = HashSet::new();
        current.insert("b".to_string());
        current.insert("c".to_string());

        let diff = state.diff_modules(&current);
        assert_eq!(diff.added, vec!["c"]);
        assert_eq!(diff.removed, vec!["a"]);
        assert_eq!(diff.retained, vec!["b"]);
    }

    #[test]
    fn test_incremental_plan_no_work() {
        let plan = IncrementalPlan {
            recompile: vec![],
            cached: vec!["a".to_string(), "b".to_string()],
            reasons: HashMap::new(),
            total_modules: 2,
        };
        assert!(!plan.has_work());
        assert_eq!(plan.recompile_ratio(), 0.0);
    }

    #[test]
    fn test_incremental_plan_full_rebuild() {
        let plan = IncrementalPlan {
            recompile: vec!["a".to_string(), "b".to_string()],
            cached: vec![],
            reasons: HashMap::new(),
            total_modules: 2,
        };
        assert!(plan.has_work());
        assert_eq!(plan.recompile_ratio(), 1.0);
    }

    #[test]
    fn test_incremental_plan_partial() {
        let plan = IncrementalPlan {
            recompile: vec!["a".to_string()],
            cached: vec!["b".to_string()],
            reasons: HashMap::new(),
            total_modules: 2,
        };
        assert!(plan.has_work());
        assert_eq!(plan.recompile_ratio(), 0.5);
    }

    #[test]
    fn test_incremental_engine_new_empty() {
        let dir = tempfile::tempdir().unwrap();
        let engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
        assert!(engine.fingerprint_db().is_empty());
        assert!(engine.last_build_time().is_none());
    }

    #[test]
    fn test_incremental_engine_record_and_save() {
        let dir = tempfile::tempdir().unwrap();
        let state_dir = dir.path().join("state");

        let file_path = dir.path().join("test.atlas");
        fs::write(&file_path, "fn test() {}").unwrap();

        let mut engine = IncrementalEngine::new_empty(state_dir.clone());
        engine.record_compilation("test", &file_path, "fn test() {}", BTreeMap::new());
        engine.save().unwrap();

        // Reload and verify
        let engine2 = IncrementalEngine::new_empty(state_dir);
        assert_eq!(engine2.fingerprint_db().len(), 1);
    }

    #[test]
    fn test_incremental_engine_invalidate() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.atlas");
        fs::write(&file_path, "fn test() {}").unwrap();

        let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
        engine.record_compilation("test", &file_path, "fn test() {}", BTreeMap::new());
        assert_eq!(engine.fingerprint_db().len(), 1);

        engine.invalidate_module("test");
        assert_eq!(engine.fingerprint_db().len(), 0);
    }

    #[test]
    fn test_incremental_engine_invalidate_all() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.atlas");
        fs::write(&file_path, "fn test() {}").unwrap();

        let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
        engine.record_compilation("a", &file_path, "fn a() {}", BTreeMap::new());
        engine.record_compilation("b", &file_path, "fn b() {}", BTreeMap::new());
        assert_eq!(engine.fingerprint_db().len(), 2);

        engine.invalidate_all();
        assert!(engine.fingerprint_db().is_empty());
    }

    #[test]
    fn test_incremental_stats_cache_hit_rate() {
        let stats = IncrementalStats {
            total_modules: 10,
            recompiled: 3,
            from_cache: 7,
            analysis_time: Duration::from_millis(5),
            compilation_time: Duration::from_millis(100),
            time_saved: Duration::from_millis(200),
            was_full_rebuild: false,
        };
        assert_eq!(stats.cache_hit_rate(), 70.0);
    }

    #[test]
    fn test_incremental_stats_summary_full() {
        let stats = IncrementalStats {
            total_modules: 5,
            recompiled: 5,
            from_cache: 0,
            analysis_time: Duration::from_millis(1),
            compilation_time: Duration::from_millis(500),
            time_saved: Duration::ZERO,
            was_full_rebuild: true,
        };
        let summary = stats.summary();
        assert!(summary.contains("Full rebuild"));
        assert!(summary.contains("5 modules"));
    }

    #[test]
    fn test_incremental_stats_summary_incremental() {
        let stats = IncrementalStats {
            total_modules: 10,
            recompiled: 2,
            from_cache: 8,
            analysis_time: Duration::from_millis(1),
            compilation_time: Duration::from_millis(50),
            time_saved: Duration::from_millis(200),
            was_full_rebuild: false,
        };
        let summary = stats.summary();
        assert!(summary.contains("Incremental"));
        assert!(summary.contains("2/10"));
    }

    #[test]
    fn test_recompile_reason_variants() {
        assert_eq!(
            RecompileReason::SourceChanged,
            RecompileReason::SourceChanged
        );
        assert_ne!(
            RecompileReason::SourceChanged,
            RecompileReason::ConfigChanged
        );
        assert_eq!(
            RecompileReason::DependencyChanged("x".to_string()),
            RecompileReason::DependencyChanged("x".to_string())
        );
    }

    #[test]
    fn test_incremental_engine_plan_cold() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("main.atlas");
        fs::write(&src, "fn main() {}").unwrap();

        let engine = IncrementalEngine::new_empty(dir.path().to_path_buf());

        let mut graph = BuildGraph::new();
        graph.add_module(ModuleNode::new("main", src));

        let plan = engine.plan(&graph).unwrap();
        assert_eq!(plan.recompile.len(), 1);
        assert_eq!(plan.cached.len(), 0);
        assert!(matches!(
            plan.reasons.get("main"),
            Some(RecompileReason::NoPreviousFingerprint)
        ));
    }

    #[test]
    fn test_incremental_engine_plan_warm() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("main.atlas");
        fs::write(&src, "fn main() {}").unwrap();

        let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());

        // Record compilation
        engine.record_compilation("main", &src, "fn main() {}", BTreeMap::new());

        let mut graph = BuildGraph::new();
        graph.add_module(ModuleNode::new("main", src));

        let plan = engine.plan(&graph).unwrap();
        assert_eq!(plan.recompile.len(), 0);
        assert_eq!(plan.cached.len(), 1);
    }

    #[test]
    fn test_incremental_engine_plan_after_change() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("main.atlas");
        fs::write(&src, "fn main() {}").unwrap();

        let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
        engine.record_compilation("main", &src, "fn main() {}", BTreeMap::new());

        // Change source
        fs::write(&src, "fn main() { 42 }").unwrap();

        let mut graph = BuildGraph::new();
        graph.add_module(ModuleNode::new("main", src));

        let plan = engine.plan(&graph).unwrap();
        assert_eq!(plan.recompile.len(), 1);
        assert!(matches!(
            plan.reasons.get("main"),
            Some(RecompileReason::SourceChanged)
        ));
    }

    #[test]
    fn test_build_state_update_from_graph() {
        let mut state = BuildState::new();
        let mut graph = BuildGraph::new();

        let a =
            ModuleNode::new("a", PathBuf::from("a.atlas")).with_dependencies(vec!["b".to_string()]);
        let b = ModuleNode::new("b", PathBuf::from("b.atlas"));
        graph.add_module(a);
        graph.add_module(b);

        state.update_from_graph(&graph);

        assert_eq!(state.dependencies.len(), 2);
        assert_eq!(state.module_paths.len(), 2);
        assert!(state.last_build_time.is_some());
    }

    #[test]
    fn test_module_diff_empty_previous() {
        let state = BuildState::new();
        let mut current = HashSet::new();
        current.insert("a".to_string());

        let diff = state.diff_modules(&current);
        assert_eq!(diff.added.len(), 1);
        assert!(diff.removed.is_empty());
        assert!(diff.retained.is_empty());
    }
}
