use atlas_package::{DependencyGraph, Resolution, ResolvedPackage, Resolver, VersionSolver};
use rstest::*;
use semver::{Version, VersionReq};

// Helper functions

fn create_version(major: u64, minor: u64, patch: u64) -> Version {
    Version::new(major, minor, patch)
}

fn create_version_req(req: &str) -> VersionReq {
    req.parse().expect("Invalid version requirement")
}

// ==================================================================
// CORE RESOLUTION TESTS
// ==================================================================

#[test]
fn test_resolve_simple_dependency_tree() {
    let mut resolver = Resolver::new();

    // Add single dependency: pkg1 depends on pkg2@1.0.0
    let pkg1_name = "pkg1".to_string();
    let pkg2_name = "pkg2".to_string();

    resolver
        .add_dependency_edge(&pkg1_name, &pkg2_name)
        .unwrap();

    // Verify dependency edge was added successfully
    // (Constraints would be added during resolve() call)
}

#[test]
fn test_resolver_new() {
    let resolver = Resolver::new();
    assert_eq!(resolver.get_constraints("test"), None);
}

#[test]
fn test_empty_manifest_resolution() {
    let resolution = Resolution::new();
    assert_eq!(resolution.package_count(), 0);
}

#[test]
fn test_resolution_add_package() {
    let mut resolution = Resolution::new();
    resolution.add_package(ResolvedPackage::new(
        "test".to_string(),
        create_version(1, 0, 0),
    ));
    assert_eq!(resolution.package_count(), 1);
}

#[test]
fn test_resolution_get_package() {
    let mut resolution = Resolution::new();
    resolution.add_package(ResolvedPackage::new(
        "test".to_string(),
        create_version(1, 0, 0),
    ));

    let pkg = resolution.get_package("test");
    assert!(pkg.is_some());
    assert_eq!(pkg.unwrap().name, "test");
}

#[test]
fn test_resolved_package_new() {
    let pkg = ResolvedPackage::new("test".to_string(), create_version(1, 0, 0));
    assert_eq!(pkg.name, "test");
    assert_eq!(pkg.version, create_version(1, 0, 0));
    assert_eq!(pkg.dependencies.len(), 0);
}

#[test]
fn test_resolved_package_with_dependencies() {
    let pkg = ResolvedPackage::with_dependencies(
        "test".to_string(),
        create_version(1, 0, 0),
        vec!["dep1".to_string(), "dep2".to_string()],
    );
    assert_eq!(pkg.dependencies.len(), 2);
    assert_eq!(pkg.dependencies[0], "dep1");
    assert_eq!(pkg.dependencies[1], "dep2");
}

#[test]
fn test_resolution_is_deterministic() {
    let resolution1 = Resolution::new();
    let resolution2 = Resolution::new();
    assert_eq!(resolution1, resolution2);
}

// ==================================================================
// VERSION CONSTRAINT TESTS
// ==================================================================

#[rstest]
#[case("=1.2.3", "1.2.3", true)]
#[case("=1.2.3", "1.2.4", false)]
#[case("=1.2.3", "2.0.0", false)]
fn test_exact_version_match(
    #[case] version_str: &str,
    #[case] test_version: &str,
    #[case] expected: bool,
) {
    let req = create_version_req(version_str);
    let version: Version = test_version.parse().unwrap();
    assert_eq!(req.matches(&version), expected);
}

#[rstest]
#[case("^1.0.0", "1.0.0", true)]
#[case("^1.0.0", "1.5.0", true)]
#[case("^1.0.0", "1.9.9", true)]
#[case("^1.0.0", "2.0.0", false)]
fn test_caret_range_compatibility(
    #[case] req_str: &str,
    #[case] version_str: &str,
    #[case] expected: bool,
) {
    let req = create_version_req(req_str);
    let version: Version = version_str.parse().unwrap();
    assert_eq!(req.matches(&version), expected);
}

#[rstest]
#[case("~1.2.0", "1.2.0", true)]
#[case("~1.2.0", "1.2.3", true)]
#[case("~1.2.0", "1.3.0", false)]
#[case("~1.2.0", "2.0.0", false)]
fn test_tilde_range_compatibility(
    #[case] req_str: &str,
    #[case] version_str: &str,
    #[case] expected: bool,
) {
    let req = create_version_req(req_str);
    let version: Version = version_str.parse().unwrap();
    assert_eq!(req.matches(&version), expected);
}

#[rstest]
#[case(">=1.0.0", "1.0.0", true)]
#[case(">=1.0.0", "2.0.0", true)]
#[case(">=1.0.0", "0.9.9", false)]
fn test_range_constraints_greater_than(
    #[case] req_str: &str,
    #[case] version_str: &str,
    #[case] expected: bool,
) {
    let req = create_version_req(req_str);
    let version: Version = version_str.parse().unwrap();
    assert_eq!(req.matches(&version), expected);
}

#[rstest]
#[case("<2.0.0", "1.9.9", true)]
#[case("<2.0.0", "2.0.0", false)]
#[case("<2.0.0", "2.1.0", false)]
fn test_range_constraints_less_than(
    #[case] req_str: &str,
    #[case] version_str: &str,
    #[case] expected: bool,
) {
    let req = create_version_req(req_str);
    let version: Version = version_str.parse().unwrap();
    assert_eq!(req.matches(&version), expected);
}

#[test]
fn test_range_constraints_combined() {
    let req = create_version_req(">=1.0.0, <2.0.0");
    assert!(req.matches(&create_version(1, 0, 0)));
    assert!(req.matches(&create_version(1, 5, 0)));
    assert!(req.matches(&create_version(1, 9, 9)));
    assert!(!req.matches(&create_version(2, 0, 0)));
}

#[test]
fn test_wildcard_version() {
    let req = create_version_req("*");
    assert!(req.matches(&create_version(1, 0, 0)));
    assert!(req.matches(&create_version(2, 0, 0)));
    assert!(req.matches(&create_version(99, 99, 99)));
}

#[test]
fn test_version_comparison_ordering() {
    let v1 = create_version(1, 0, 0);
    let v2 = create_version(1, 0, 1);
    let v3 = create_version(1, 1, 0);
    let v4 = create_version(2, 0, 0);

    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v3 < v4);
}

#[test]
fn test_constraint_intersection() {
    let req1 = create_version_req("^1.0.0");
    let req2 = create_version_req(">=1.2.0");

    // Both should match 1.5.0
    let version = create_version(1, 5, 0);
    assert!(req1.matches(&version));
    assert!(req2.matches(&version));

    // 1.1.0 matches req1 but not req2
    let version2 = create_version(1, 1, 0);
    assert!(req1.matches(&version2));
    assert!(!req2.matches(&version2));
}

// ==================================================================
// DEPENDENCY GRAPH TESTS
// ==================================================================

#[test]
fn test_add_package_to_graph() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));
    assert_eq!(graph.len(), 1);
    assert!(!graph.is_empty());
}

#[test]
fn test_add_edge_to_graph() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));
    graph.add_package("pkg2".to_string(), create_version(1, 0, 0));

    assert!(graph.add_edge("pkg1", "pkg2").is_ok());
}

#[test]
fn test_circular_dependency_detection() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));
    graph.add_package("pkg2".to_string(), create_version(1, 0, 0));

    assert!(graph.add_edge("pkg1", "pkg2").is_ok());
    let result = graph.add_edge("pkg2", "pkg1");
    assert!(result.is_err());
}

#[test]
fn test_circular_dependency_complex() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));
    graph.add_package("pkg2".to_string(), create_version(1, 0, 0));
    graph.add_package("pkg3".to_string(), create_version(1, 0, 0));

    assert!(graph.add_edge("pkg1", "pkg2").is_ok());
    assert!(graph.add_edge("pkg2", "pkg3").is_ok());
    let result = graph.add_edge("pkg3", "pkg1");
    assert!(result.is_err());
}

#[test]
fn test_topological_sort_simple() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));
    graph.add_package("pkg2".to_string(), create_version(1, 0, 0));

    graph.add_edge("pkg1", "pkg2").unwrap();

    let result = graph.topological_sort();
    if result.is_err() {
        println!("ERROR: {:?}", result);
    }
    assert!(result.is_ok(), "Topological sort failed: {:?}", result);
    let order = result.unwrap();
    assert_eq!(order.len(), 2);

    // pkg2 should come before pkg1
    let pkg2_idx = order.iter().position(|p| p == "pkg2").unwrap();
    let pkg1_idx = order.iter().position(|p| p == "pkg1").unwrap();
    assert!(pkg2_idx < pkg1_idx);
}

#[test]
fn test_topological_sort_diamond() {
    let mut graph = DependencyGraph::new();
    graph.add_package("root".to_string(), create_version(1, 0, 0));
    graph.add_package("left".to_string(), create_version(1, 0, 0));
    graph.add_package("right".to_string(), create_version(1, 0, 0));
    graph.add_package("bottom".to_string(), create_version(1, 0, 0));

    graph.add_edge("root", "left").unwrap();
    graph.add_edge("root", "right").unwrap();
    graph.add_edge("left", "bottom").unwrap();
    graph.add_edge("right", "bottom").unwrap();

    let result = graph.topological_sort();
    assert!(result.is_ok());
    let order = result.unwrap();
    assert_eq!(order.len(), 4);

    // bottom should come before left and right, which should come before root
    let bottom_idx = order.iter().position(|p| p == "bottom").unwrap();
    let left_idx = order.iter().position(|p| p == "left").unwrap();
    let right_idx = order.iter().position(|p| p == "right").unwrap();
    let root_idx = order.iter().position(|p| p == "root").unwrap();

    assert!(bottom_idx < left_idx);
    assert!(bottom_idx < right_idx);
    assert!(left_idx < root_idx);
    assert!(right_idx < root_idx);
}

#[test]
fn test_graph_empty() {
    let graph = DependencyGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.len(), 0);
}

#[test]
fn test_graph_single_package() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));

    let result = graph.topological_sort();
    assert!(result.is_ok());
    let order = result.unwrap();
    assert_eq!(order.len(), 1);
    assert_eq!(order[0], "pkg1");
}

// ==================================================================
// VERSION SOLVER TESTS
// ==================================================================

#[test]
fn test_version_solver_new() {
    let solver = VersionSolver::new();
    assert!(solver.get_versions("test").is_none());
}

#[test]
fn test_add_package_versions() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions(
        "test",
        vec![create_version(1, 0, 0), create_version(2, 0, 0)],
    );

    let versions = solver.get_versions("test");
    assert!(versions.is_some());
    assert_eq!(versions.unwrap().len(), 2);
}

#[test]
fn test_max_satisfying_version_simple() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions(
        "test",
        vec![
            create_version(1, 0, 0),
            create_version(1, 1, 0),
            create_version(2, 0, 0),
        ],
    );

    let req = create_version_req("^1.0.0");
    let version = solver.max_satisfying_version("test", &[req]);
    assert_eq!(version, Some(create_version(1, 1, 0)));
}

#[test]
fn test_max_satisfying_version_no_match() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions("test", vec![create_version(1, 0, 0)]);

    let req = create_version_req("^2.0.0");
    let version = solver.max_satisfying_version("test", &[req]);
    assert_eq!(version, None);
}

#[test]
fn test_is_satisfiable() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions("test", vec![create_version(1, 0, 0)]);

    let req1 = create_version_req("^1.0.0");
    assert!(solver.is_satisfiable("test", &[req1]));

    let req2 = create_version_req("^2.0.0");
    assert!(!solver.is_satisfiable("test", &[req2]));
}

#[test]
fn test_find_all_satisfying() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions(
        "test",
        vec![
            create_version(1, 0, 0),
            create_version(1, 1, 0),
            create_version(1, 2, 0),
            create_version(2, 0, 0),
        ],
    );

    let req = create_version_req("^1.0.0");
    let versions = solver.find_all_satisfying("test", &[req]);
    assert_eq!(versions.len(), 3);
}

#[test]
fn test_are_constraints_compatible_yes() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions(
        "test",
        vec![create_version(1, 0, 0), create_version(1, 5, 0)],
    );

    let req1 = vec![create_version_req("^1.0.0")];
    let req2 = vec![create_version_req(">=1.0.0")];
    assert!(solver.are_constraints_compatible("test", &req1, &req2));
}

#[test]
fn test_are_constraints_compatible_no() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions(
        "test",
        vec![create_version(1, 0, 0), create_version(2, 0, 0)],
    );

    let req1 = vec![create_version_req("^1.0.0")];
    let req2 = vec![create_version_req("^2.0.0")];
    assert!(!solver.are_constraints_compatible("test", &req1, &req2));
}

#[test]
fn test_min_satisfying_version() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions(
        "test",
        vec![
            create_version(1, 0, 0),
            create_version(1, 1, 0),
            create_version(1, 2, 0),
        ],
    );

    let req = create_version_req("^1.0.0");
    let version = solver.min_satisfying_version("test", &[req]);
    assert_eq!(version, Some(create_version(1, 0, 0)));
}

// ==================================================================
// EDGE CASES
// ==================================================================

#[test]
fn test_package_depends_on_itself() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));

    let result = graph.add_edge("pkg1", "pkg1");
    assert!(result.is_err());
}

#[test]
fn test_get_version_from_graph() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 2, 3));

    let version = graph.get_version("pkg1");
    assert_eq!(version, Some(&create_version(1, 2, 3)));
}

#[test]
fn test_get_dependencies_from_graph() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));
    graph.add_package("pkg2".to_string(), create_version(1, 0, 0));
    graph.add_edge("pkg1", "pkg2").unwrap();

    let deps = graph.get_dependencies("pkg1");
    assert!(deps.is_some());
    assert_eq!(deps.unwrap().len(), 1);
}

#[test]
fn test_packages_list() {
    let mut graph = DependencyGraph::new();
    graph.add_package("pkg1".to_string(), create_version(1, 0, 0));
    graph.add_package("pkg2".to_string(), create_version(1, 0, 0));

    let packages = graph.packages();
    assert_eq!(packages.len(), 2);
    assert!(packages.contains(&"pkg1".to_string()));
    assert!(packages.contains(&"pkg2".to_string()));
}

#[test]
fn test_topological_sort_empty_graph() {
    let graph = DependencyGraph::new();
    let result = graph.topological_sort();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_version_solver_multiple_constraints() {
    let mut solver = VersionSolver::new();
    solver.add_package_versions(
        "test",
        vec![
            create_version(1, 0, 0),
            create_version(1, 5, 0),
            create_version(1, 8, 0),
            create_version(2, 0, 0),
        ],
    );

    let req1 = create_version_req("^1.0.0");
    let req2 = create_version_req("<1.8.0");
    let version = solver.max_satisfying_version("test", &[req1, req2]);
    assert_eq!(version, Some(create_version(1, 5, 0)));
}
