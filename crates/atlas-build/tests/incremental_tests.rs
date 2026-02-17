//! Integration tests for incremental compilation
//!
//! Tests the incremental build system: fingerprinting, change detection,
//! dependency propagation, cache management, build state persistence,
//! and selective recompilation.

use atlas_build::build_order::{BuildGraph, ModuleNode};
use atlas_build::fingerprint::{
    compute_fingerprint, compute_fingerprint_from_content, compute_hash,
    compute_hash_without_comments, quick_check_changed, PlatformInfo,
};
use atlas_build::incremental::BuildState;
use atlas_build::{
    Builder, FingerprintConfig, FingerprintDb, IncrementalEngine, IncrementalPlan, OptLevel,
    RecompileReason,
};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// ─── Test helpers ───

fn create_test_project(files: &[(&str, &str)]) -> (TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();

    fs::create_dir(path.join("src")).unwrap();

    let manifest = r#"
[package]
name = "test-project"
version = "0.1.0"
"#;
    fs::write(path.join("atlas.toml"), manifest).unwrap();

    for (file_path, content) in files {
        let full_path = path.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, content).unwrap();
    }

    let path_str = path.to_string_lossy().to_string();
    (dir, path_str)
}

/// Create a builder with target dir inside the temp project to avoid cross-test interference
fn make_builder(path: &str) -> Builder {
    let target_dir = PathBuf::from(path).join("target/debug");
    Builder::new(path).unwrap().with_target_dir(target_dir)
}

fn simple_main() -> &'static str {
    r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#
}

fn simple_lib() -> &'static str {
    r#"export fn helper() -> number {
    return 10;
}"#
}

// ─── 1. Initial full build ───

#[test]
fn test_initial_build_compiles_all_files() {
    let (_temp, path) = create_test_project(&[
        ("src/main.atlas", simple_main()),
        ("src/lib.atlas", simple_lib()),
    ]);
    let mut builder = make_builder(&path);
    let ctx = builder.build_incremental().unwrap();
    assert_eq!(ctx.stats.total_modules, 2);
}

#[test]
fn test_initial_build_single_file() {
    let (_temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    let ctx = builder.build_incremental().unwrap();
    assert_eq!(ctx.stats.total_modules, 1);
}

#[test]
fn test_initial_build_creates_artifacts() {
    let (_temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    let ctx = builder.build_incremental().unwrap();
    assert!(!ctx.artifacts.is_empty());
}

#[test]
fn test_initial_build_three_files() {
    let (_temp, path) = create_test_project(&[
        ("src/main.atlas", simple_main()),
        ("src/lib.atlas", simple_lib()),
        (
            "src/utils.atlas",
            r#"export fn mul(a: number, b: number) -> number { return a * b; }"#,
        ),
    ]);
    let mut builder = make_builder(&path);
    let ctx = builder.build_incremental().unwrap();
    assert_eq!(ctx.stats.total_modules, 3);
}

// ─── 2. No-change rebuild ───

#[test]
fn test_rebuild_no_changes() {
    let (_temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    builder.build_incremental().unwrap();
    let result = builder.build_incremental();
    assert!(result.is_ok());
}

#[test]
fn test_rebuild_no_changes_preserves_module_count() {
    let (_temp, path) = create_test_project(&[
        ("src/main.atlas", simple_main()),
        ("src/lib.atlas", simple_lib()),
    ]);
    let mut builder = make_builder(&path);
    builder.build_incremental().unwrap();
    let ctx = builder.build_incremental().unwrap();
    assert_eq!(ctx.stats.total_modules, 2);
}

// ─── 3. Change one file ───

#[test]
fn test_change_one_file_rebuilds() {
    let (temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    builder.build_incremental().unwrap();

    thread::sleep(Duration::from_millis(10));
    fs::write(
        temp.path().join("src/main.atlas"),
        r#"fn main() -> void { let x: number = 43; print(x); }"#,
    )
    .unwrap();

    let result = builder.build_incremental();
    assert!(result.is_ok());
}

#[test]
fn test_change_lib_not_main() {
    let (temp, path) = create_test_project(&[
        ("src/main.atlas", simple_main()),
        ("src/lib.atlas", simple_lib()),
    ]);
    let mut builder = make_builder(&path);
    builder.build_incremental().unwrap();

    thread::sleep(Duration::from_millis(10));
    fs::write(
        temp.path().join("src/lib.atlas"),
        r#"export fn helper() -> number { return 20; }"#,
    )
    .unwrap();

    let ctx = builder.build_incremental().unwrap();
    assert_eq!(ctx.stats.total_modules, 2);
}

// ─── 4. Dependency change propagation ───

#[test]
fn test_dependency_change_propagates() {
    let dir = tempfile::tempdir().unwrap();
    let a_path = dir.path().join("a.atlas");
    let b_path = dir.path().join("b.atlas");
    fs::write(&a_path, "fn a() {}").unwrap();
    fs::write(&b_path, "fn b() {}").unwrap();

    let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
    engine.record_compilation("a", &a_path, "fn a() {}", BTreeMap::new());

    let mut dep_hashes = BTreeMap::new();
    dep_hashes.insert("a".to_string(), compute_hash("fn a() {}"));
    engine.record_compilation("b", &b_path, "fn b() {}", dep_hashes);

    // Change a
    fs::write(&a_path, "fn a() { 42 }").unwrap();

    let mut graph = BuildGraph::new();
    graph.add_module(ModuleNode::new("a", a_path));
    graph.add_module(ModuleNode::new("b", b_path).with_dependencies(vec!["a".to_string()]));

    let plan = engine.plan(&graph).unwrap();
    assert!(plan.recompile.contains(&"a".to_string()));
    // b should also be invalidated because its dep hash changed
    assert!(plan.recompile.contains(&"b".to_string()));
}

// ─── 5. Fingerprint detection ───

#[test]
fn test_fingerprint_detects_content_change() {
    let config = FingerprintConfig::default();
    let fp1 = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &config);
    let fp2 = compute_fingerprint_from_content("fn main() { 42 }", BTreeMap::new(), &config);
    assert_ne!(fp1.hash, fp2.hash);
}

#[test]
fn test_fingerprint_stable_for_same_content() {
    let config = FingerprintConfig::default();
    let fp1 = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &config);
    let fp2 = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &config);
    assert_eq!(fp1.hash, fp2.hash);
}

#[test]
fn test_fingerprint_includes_deps() {
    let config = FingerprintConfig::default();
    let fp1 = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &config);

    let mut deps = BTreeMap::new();
    deps.insert("lib".to_string(), "hash123".to_string());
    let fp2 = compute_fingerprint_from_content("fn main() {}", deps, &config);
    assert_ne!(fp1.hash, fp2.hash);
}

#[test]
fn test_fingerprint_includes_optimization() {
    let c1 = FingerprintConfig {
        optimization: "O0".to_string(),
        ..Default::default()
    };
    let c2 = FingerprintConfig {
        optimization: "O2".to_string(),
        ..Default::default()
    };
    let fp1 = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &c1);
    let fp2 = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &c2);
    assert_ne!(fp1.hash, fp2.hash);
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

// ─── 6. Cache hit reuses artifact ───

#[test]
fn test_cache_hit_on_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("main.atlas");
    fs::write(&src, "fn main() {}").unwrap();

    let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
    engine.record_compilation("main", &src, "fn main() {}", BTreeMap::new());

    let mut graph = BuildGraph::new();
    graph.add_module(ModuleNode::new("main", src));

    let plan = engine.plan(&graph).unwrap();
    assert!(plan.cached.contains(&"main".to_string()));
    assert!(plan.recompile.is_empty());
}

// ─── 7. Cache miss recompiles ───

#[test]
fn test_cache_miss_on_new_module() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("main.atlas");
    fs::write(&src, "fn main() {}").unwrap();

    let engine = IncrementalEngine::new_empty(dir.path().to_path_buf());

    let mut graph = BuildGraph::new();
    graph.add_module(ModuleNode::new("main", src));

    let plan = engine.plan(&graph).unwrap();
    assert!(plan.recompile.contains(&"main".to_string()));
    assert!(matches!(
        plan.reasons.get("main"),
        Some(RecompileReason::NoPreviousFingerprint)
    ));
}

#[test]
fn test_cache_miss_on_changed_content() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("main.atlas");
    fs::write(&src, "fn main() {}").unwrap();

    let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
    engine.record_compilation("main", &src, "fn main() {}", BTreeMap::new());

    // Change content
    fs::write(&src, "fn main() { 42 }").unwrap();

    let mut graph = BuildGraph::new();
    graph.add_module(ModuleNode::new("main", src));

    let plan = engine.plan(&graph).unwrap();
    assert!(plan.recompile.contains(&"main".to_string()));
}

// ─── 8. Parallel incremental build (recompile groups) ───

#[test]
fn test_parallel_independent_modules() {
    let (_temp, path) = create_test_project(&[
        ("src/main.atlas", simple_main()),
        ("src/lib.atlas", simple_lib()),
    ]);
    let mut builder = make_builder(&path);
    builder = builder.with_parallel(true);
    let ctx = builder.build_incremental().unwrap();
    assert_eq!(ctx.stats.total_modules, 2);
}

// ─── 9. Incremental type checking ───

#[test]
fn test_type_error_detected_incrementally() {
    let (temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    builder.build_incremental().unwrap();

    // Introduce type error
    thread::sleep(Duration::from_millis(10));
    fs::write(
        temp.path().join("src/main.atlas"),
        r#"fn main() -> number { return "not a number"; }"#,
    )
    .unwrap();

    let result = builder.build_incremental();
    assert!(result.is_err());
}

// ─── 10. State persistence ───

#[test]
fn test_state_persists_across_builder_instances() {
    let (_temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);

    {
        let mut b = make_builder(&path);
        b.build_incremental().unwrap();
    }
    {
        let mut b = make_builder(&path);
        let result = b.build_incremental();
        assert!(result.is_ok());
    }
}

#[test]
fn test_fingerprint_db_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("fp.json");
    let config = FingerprintConfig::default();

    let fp = compute_fingerprint_from_content("fn main() {}", BTreeMap::new(), &config);

    {
        let mut db = FingerprintDb::new();
        db.insert("main".to_string(), fp.clone());
        db.save(&db_path).unwrap();
    }

    let db = FingerprintDb::load(&db_path).unwrap();
    assert_eq!(db.len(), 1);
    assert_eq!(db.get("main").unwrap().hash, fp.hash);
}

#[test]
fn test_build_state_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join("state.json");

    let mut state = BuildState::new();
    state
        .dependencies
        .insert("a".to_string(), vec!["b".to_string()]);
    state
        .module_paths
        .insert("a".to_string(), PathBuf::from("a.atlas"));
    state.save(&state_path).unwrap();

    let loaded = BuildState::load(&state_path).unwrap();
    assert_eq!(loaded.dependencies.len(), 1);
    assert_eq!(loaded.module_paths.len(), 1);
}

// ─── Fingerprint edge cases ───

#[test]
fn test_comment_only_change_ignored() {
    let v1 = "let x = 1;\nlet y = 2;";
    let v2 = "let x = 1; // added comment\nlet y = 2;";
    let h1 = compute_hash_without_comments(v1);
    let h2 = compute_hash_without_comments(v2);
    assert_eq!(h1, h2);
}

#[test]
fn test_multiline_comment_change_ignored() {
    // Same-line block comment removal
    let v1 = "let x = 1;  let y = 2;";
    let v2 = "let x = 1; /* comment */ let y = 2;";
    let h1 = compute_hash_without_comments(v1);
    let h2 = compute_hash_without_comments(v2);
    assert_eq!(h1, h2);
}

#[test]
fn test_code_change_not_ignored() {
    let h1 = compute_hash_without_comments("let x = 1;");
    let h2 = compute_hash_without_comments("let x = 2;");
    assert_ne!(h1, h2);
}

#[test]
fn test_hash_deterministic() {
    let h1 = compute_hash("fn main() {}");
    let h2 = compute_hash("fn main() {}");
    assert_eq!(h1, h2);
}

#[test]
fn test_hash_different_content() {
    let h1 = compute_hash("fn a() {}");
    let h2 = compute_hash("fn b() {}");
    assert_ne!(h1, h2);
}

#[test]
fn test_quick_check_missing_file() {
    let fp =
        compute_fingerprint_from_content("test", BTreeMap::new(), &FingerprintConfig::default());
    assert!(quick_check_changed(
        &fp,
        &PathBuf::from("/nonexistent/file.atlas")
    ));
}

#[test]
fn test_fingerprint_env_vars_affect_hash() {
    let c1 = FingerprintConfig::default();
    let mut vars = BTreeMap::new();
    vars.insert("ATLAS_DEBUG".to_string(), "1".to_string());
    let c2 = FingerprintConfig {
        env_vars: vars,
        ..Default::default()
    };

    let fp1 = compute_fingerprint_from_content("test", BTreeMap::new(), &c1);
    let fp2 = compute_fingerprint_from_content("test", BTreeMap::new(), &c2);
    assert_ne!(fp1.hash, fp2.hash);
}

#[test]
fn test_fingerprint_dep_order_independent() {
    let config = FingerprintConfig::default();
    let mut d1 = BTreeMap::new();
    d1.insert("a".to_string(), "ha".to_string());
    d1.insert("b".to_string(), "hb".to_string());

    let mut d2 = BTreeMap::new();
    d2.insert("b".to_string(), "hb".to_string());
    d2.insert("a".to_string(), "ha".to_string());

    let fp1 = compute_fingerprint_from_content("test", d1, &config);
    let fp2 = compute_fingerprint_from_content("test", d2, &config);
    assert_eq!(fp1.hash, fp2.hash);
}

// ─── Incremental engine ───

#[test]
fn test_engine_invalidate_module() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("test.atlas");
    fs::write(&src, "fn test() {}").unwrap();

    let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
    engine.record_compilation("test", &src, "fn test() {}", BTreeMap::new());
    assert_eq!(engine.fingerprint_db().len(), 1);

    engine.invalidate_module("test");
    assert_eq!(engine.fingerprint_db().len(), 0);
}

#[test]
fn test_engine_invalidate_all() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("test.atlas");
    fs::write(&src, "fn test() {}").unwrap();

    let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
    engine.record_compilation("a", &src, "fn a() {}", BTreeMap::new());
    engine.record_compilation("b", &src, "fn b() {}", BTreeMap::new());
    assert_eq!(engine.fingerprint_db().len(), 2);

    engine.invalidate_all();
    assert!(engine.fingerprint_db().is_empty());
}

#[test]
fn test_engine_save_and_reload() {
    let dir = tempfile::tempdir().unwrap();
    let state_dir = dir.path().join("state");
    let src = dir.path().join("test.atlas");
    fs::write(&src, "fn test() {}").unwrap();

    {
        let mut engine = IncrementalEngine::new(state_dir.clone(), FingerprintConfig::default());
        engine.record_compilation("test", &src, "fn test() {}", BTreeMap::new());
        engine.save().unwrap();
    }

    let engine = IncrementalEngine::new(state_dir, FingerprintConfig::default());
    assert_eq!(engine.fingerprint_db().len(), 1);
}

#[test]
fn test_engine_plan_cold_build() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("main.atlas");
    fs::write(&src, "fn main() {}").unwrap();

    let engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
    let mut graph = BuildGraph::new();
    graph.add_module(ModuleNode::new("main", src));

    let plan = engine.plan(&graph).unwrap();
    assert_eq!(plan.recompile.len(), 1);
    assert_eq!(plan.cached.len(), 0);
    assert_eq!(plan.total_modules, 1);
}

#[test]
fn test_engine_plan_warm_no_changes() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("main.atlas");
    fs::write(&src, "fn main() {}").unwrap();

    let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
    engine.record_compilation("main", &src, "fn main() {}", BTreeMap::new());

    let mut graph = BuildGraph::new();
    graph.add_module(ModuleNode::new("main", src));

    let plan = engine.plan(&graph).unwrap();
    assert_eq!(plan.recompile.len(), 0);
    assert_eq!(plan.cached.len(), 1);
}

#[test]
fn test_engine_plan_after_content_change() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("main.atlas");
    fs::write(&src, "fn main() {}").unwrap();

    let mut engine = IncrementalEngine::new_empty(dir.path().to_path_buf());
    engine.record_compilation("main", &src, "fn main() {}", BTreeMap::new());

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

// ─── Build state ───

#[test]
fn test_build_state_update_from_graph() {
    let mut state = BuildState::new();
    let mut graph = BuildGraph::new();
    graph.add_module(
        ModuleNode::new("a", PathBuf::from("a.atlas")).with_dependencies(vec!["b".to_string()]),
    );
    graph.add_module(ModuleNode::new("b", PathBuf::from("b.atlas")));

    state.update_from_graph(&graph);
    assert_eq!(state.dependencies.len(), 2);
    assert_eq!(state.module_paths.len(), 2);
    assert!(state.last_build_time.is_some());
}

#[test]
fn test_build_state_diff_added() {
    let state = BuildState::new();
    let mut current = HashSet::new();
    current.insert("a".to_string());
    let diff = state.diff_modules(&current);
    assert_eq!(diff.added.len(), 1);
    assert!(diff.removed.is_empty());
}

#[test]
fn test_build_state_diff_removed() {
    let mut state = BuildState::new();
    state
        .module_paths
        .insert("a".to_string(), PathBuf::from("a.atlas"));
    let current = HashSet::new();
    let diff = state.diff_modules(&current);
    assert!(diff.added.is_empty());
    assert_eq!(diff.removed.len(), 1);
}

#[test]
fn test_build_state_diff_retained() {
    let mut state = BuildState::new();
    state
        .module_paths
        .insert("a".to_string(), PathBuf::from("a.atlas"));
    let mut current = HashSet::new();
    current.insert("a".to_string());
    let diff = state.diff_modules(&current);
    assert!(diff.added.is_empty());
    assert!(diff.removed.is_empty());
    assert_eq!(diff.retained.len(), 1);
}

// ─── IncrementalPlan ───

#[test]
fn test_plan_has_work() {
    let plan = IncrementalPlan {
        recompile: vec!["a".to_string()],
        cached: vec![],
        reasons: Default::default(),
        total_modules: 1,
    };
    assert!(plan.has_work());
}

#[test]
fn test_plan_no_work() {
    let plan = IncrementalPlan {
        recompile: vec![],
        cached: vec!["a".to_string()],
        reasons: Default::default(),
        total_modules: 1,
    };
    assert!(!plan.has_work());
}

#[test]
fn test_plan_recompile_ratio_full() {
    let plan = IncrementalPlan {
        recompile: vec!["a".to_string(), "b".to_string()],
        cached: vec![],
        reasons: Default::default(),
        total_modules: 2,
    };
    assert_eq!(plan.recompile_ratio(), 1.0);
}

#[test]
fn test_plan_recompile_ratio_half() {
    let plan = IncrementalPlan {
        recompile: vec!["a".to_string()],
        cached: vec!["b".to_string()],
        reasons: Default::default(),
        total_modules: 2,
    };
    assert_eq!(plan.recompile_ratio(), 0.5);
}

#[test]
fn test_plan_recompile_ratio_zero() {
    let plan = IncrementalPlan {
        recompile: vec![],
        cached: vec!["a".to_string()],
        reasons: Default::default(),
        total_modules: 1,
    };
    assert_eq!(plan.recompile_ratio(), 0.0);
}

#[test]
fn test_plan_recompile_ratio_empty() {
    let plan = IncrementalPlan {
        recompile: vec![],
        cached: vec![],
        reasons: Default::default(),
        total_modules: 0,
    };
    assert_eq!(plan.recompile_ratio(), 0.0);
}

// ─── FingerprintDb ───

#[test]
fn test_fingerprint_db_needs_recompile_unknown() {
    let db = FingerprintDb::new();
    let fp =
        compute_fingerprint_from_content("test", BTreeMap::new(), &FingerprintConfig::default());
    assert!(db.needs_recompile("unknown", &fp));
}

#[test]
fn test_fingerprint_db_needs_recompile_same() {
    let mut db = FingerprintDb::new();
    let config = FingerprintConfig::default();
    let fp = compute_fingerprint_from_content("test", BTreeMap::new(), &config);
    db.insert("test".to_string(), fp.clone());
    assert!(!db.needs_recompile("test", &fp));
}

#[test]
fn test_fingerprint_db_needs_recompile_different() {
    let mut db = FingerprintDb::new();
    let config = FingerprintConfig::default();
    let fp1 = compute_fingerprint_from_content("v1", BTreeMap::new(), &config);
    db.insert("test".to_string(), fp1);
    let fp2 = compute_fingerprint_from_content("v2", BTreeMap::new(), &config);
    assert!(db.needs_recompile("test", &fp2));
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
fn test_fingerprint_db_remove() {
    let mut db = FingerprintDb::new();
    let config = FingerprintConfig::default();
    db.insert(
        "a".to_string(),
        compute_fingerprint_from_content("a", BTreeMap::new(), &config),
    );
    db.remove("a");
    assert!(db.is_empty());
}

// ─── Builder integration ───

#[test]
fn test_incremental_build_with_errors() {
    let (temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    builder.build_incremental().unwrap();

    thread::sleep(Duration::from_millis(10));
    fs::write(
        temp.path().join("src/main.atlas"),
        r#"fn main() -> void { let x: number = // missing }"#,
    )
    .unwrap();

    assert!(builder.build_incremental().is_err());
}

#[test]
fn test_library_target_incremental() {
    let (_temp, path) = create_test_project(&[("src/lib.atlas", simple_lib())]);
    let mut builder = make_builder(&path);
    let ctx = builder.build_incremental().unwrap();
    assert_eq!(ctx.stats.total_modules, 1);
}

#[test]
fn test_binary_target_incremental() {
    let (_temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    let ctx = builder.build_incremental().unwrap();
    assert!(!ctx.artifacts.is_empty());
}

#[test]
fn test_incremental_with_optimization() {
    let (_temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    builder = builder.with_optimization(OptLevel::O2);
    let result = builder.build_incremental();
    assert!(result.is_ok());
}

#[test]
fn test_build_then_incremental_same_result() {
    let (_temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);

    // Full build
    let mut b1 = make_builder(&path);
    let ctx1 = b1.build().unwrap();

    // Incremental build
    let mut b2 = make_builder(&path);
    let ctx2 = b2.build_incremental().unwrap();

    assert_eq!(ctx1.stats.total_modules, ctx2.stats.total_modules);
}

#[test]
fn test_cold_cache_incremental_build() {
    let (_temp, path) = create_test_project(&[("src/main.atlas", simple_main())]);
    let mut builder = make_builder(&path);
    let ctx = builder.build_incremental().unwrap();
    assert_eq!(ctx.stats.total_modules, 1);
}

// ─── Platform info ───

#[test]
fn test_platform_info_current() {
    let info = PlatformInfo::current();
    assert!(!info.os.is_empty());
    assert!(!info.arch.is_empty());
}

// ─── RecompileReason ───

#[test]
fn test_recompile_reason_source_changed() {
    assert_eq!(
        RecompileReason::SourceChanged,
        RecompileReason::SourceChanged
    );
}

#[test]
fn test_recompile_reason_dep_changed() {
    let r = RecompileReason::DependencyChanged("lib".to_string());
    assert_eq!(r, RecompileReason::DependencyChanged("lib".to_string()));
    assert_ne!(r, RecompileReason::SourceChanged);
}

#[test]
fn test_recompile_reason_no_previous() {
    assert_ne!(
        RecompileReason::NoPreviousFingerprint,
        RecompileReason::SourceChanged
    );
}
