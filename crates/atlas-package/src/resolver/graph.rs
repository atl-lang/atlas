use semver::Version;
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),
}

pub type GraphResult<T> = Result<T, GraphError>;

/// Dependency graph tracking package relationships
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Adjacency list: package -> dependencies
    edges: HashMap<String, HashSet<String>>,

    /// Package versions
    versions: HashMap<String, Version>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            versions: HashMap::new(),
        }
    }

    /// Add package to graph
    pub fn add_package(&mut self, name: String, version: Version) {
        self.versions.insert(name.clone(), version);
        self.edges.entry(name).or_default();
    }

    /// Add dependency edge (from depends on to)
    pub fn add_edge(&mut self, from: &str, to: &str) -> GraphResult<()> {
        // Ensure both packages exist in graph
        if !self.edges.contains_key(from) {
            self.edges.insert(from.to_string(), HashSet::new());
        }
        if !self.edges.contains_key(to) {
            self.edges.insert(to.to_string(), HashSet::new());
        }

        // Check for circular dependencies before adding edge
        if self.would_create_cycle(from, to) {
            return Err(GraphError::CircularDependency(format!(
                "{} -> {}",
                from, to
            )));
        }

        self.edges.get_mut(from).unwrap().insert(to.to_string());

        Ok(())
    }

    /// Check if adding edge would create cycle
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // If from == to, it's a self-loop (cycle)
        if from == to {
            return true;
        }

        // Check if there's a path from 'to' to 'from'
        // If yes, adding 'from -> to' would create a cycle
        self.has_path(to, from)
    }

    /// Check if there's a path from start to end using DFS
    fn has_path(&self, start: &str, end: &str) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(current) = stack.pop() {
            if current == end {
                return true;
            }

            if visited.contains(current) {
                continue;
            }

            visited.insert(current);

            if let Some(deps) = self.edges.get(current) {
                for dep in deps {
                    if !visited.contains(dep.as_str()) {
                        stack.push(dep);
                    }
                }
            }
        }

        false
    }

    /// Get topological sort (build order) using Kahn's algorithm
    pub fn topological_sort(&self) -> GraphResult<Vec<String>> {
        if self.edges.is_empty() {
            return Ok(Vec::new());
        }

        let mut in_degree = self.compute_in_degrees();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Start with packages that have no dependencies (in-degree = 0)
        for (package, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(package.clone());
            }
        }

        while let Some(package) = queue.pop_front() {
            result.push(package.clone());

            // For each dependent of this package, reduce its in-degree
            for (node, deps) in &self.edges {
                if deps.contains(&package) {
                    if let Some(degree) = in_degree.get_mut(node) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(node.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.edges.len() {
            return Err(GraphError::CircularDependency(
                "Dependency cycle detected in graph".to_string(),
            ));
        }

        Ok(result)
    }

    /// Compute in-degrees for all packages
    /// In-degree = number of dependencies this package has
    fn compute_in_degrees(&self) -> HashMap<String, usize> {
        let mut in_degree = HashMap::new();

        // For each package, count how many dependencies it has
        for (package, deps) in &self.edges {
            in_degree.insert(package.clone(), deps.len());
        }

        in_degree
    }

    /// Get dependencies for a package
    pub fn get_dependencies(&self, package: &str) -> Option<&HashSet<String>> {
        self.edges.get(package)
    }

    /// Get version for a package
    pub fn get_version(&self, package: &str) -> Option<&Version> {
        self.versions.get(package)
    }

    /// Get all packages in graph
    pub fn packages(&self) -> Vec<String> {
        self.edges.keys().cloned().collect()
    }

    /// Get number of packages
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.len(), 0);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_add_package() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));
        assert_eq!(graph.len(), 1);
        assert!(!graph.is_empty());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));
        graph.add_package("pkg2".to_string(), Version::new(1, 0, 0));

        assert!(graph.add_edge("pkg1", "pkg2").is_ok());
    }

    #[test]
    fn test_circular_dependency_self_loop() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));

        let result = graph.add_edge("pkg1", "pkg1");
        assert!(result.is_err());
        assert!(matches!(result, Err(GraphError::CircularDependency(_))));
    }

    #[test]
    fn test_circular_dependency_simple_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));
        graph.add_package("pkg2".to_string(), Version::new(1, 0, 0));

        assert!(graph.add_edge("pkg1", "pkg2").is_ok());
        let result = graph.add_edge("pkg2", "pkg1");
        assert!(result.is_err());
        assert!(matches!(result, Err(GraphError::CircularDependency(_))));
    }

    #[test]
    fn test_circular_dependency_complex_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));
        graph.add_package("pkg2".to_string(), Version::new(1, 0, 0));
        graph.add_package("pkg3".to_string(), Version::new(1, 0, 0));

        assert!(graph.add_edge("pkg1", "pkg2").is_ok());
        assert!(graph.add_edge("pkg2", "pkg3").is_ok());
        let result = graph.add_edge("pkg3", "pkg1");
        assert!(result.is_err());
        assert!(matches!(result, Err(GraphError::CircularDependency(_))));
    }

    #[test]
    fn test_topological_sort_empty() {
        let graph = DependencyGraph::new();
        let result = graph.topological_sort();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_topological_sort_single_package() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));

        let result = graph.topological_sort();
        assert!(result.is_ok());
        let order = result.unwrap();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], "pkg1");
    }

    #[test]
    fn test_topological_sort_linear() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));
        graph.add_package("pkg2".to_string(), Version::new(1, 0, 0));
        graph.add_package("pkg3".to_string(), Version::new(1, 0, 0));

        graph.add_edge("pkg1", "pkg2").unwrap();
        graph.add_edge("pkg2", "pkg3").unwrap();

        let result = graph.topological_sort();
        assert!(result.is_ok());
        let order = result.unwrap();
        assert_eq!(order.len(), 3);

        // pkg3 should come before pkg2, pkg2 before pkg1
        let pkg3_idx = order.iter().position(|p| p == "pkg3").unwrap();
        let pkg2_idx = order.iter().position(|p| p == "pkg2").unwrap();
        let pkg1_idx = order.iter().position(|p| p == "pkg1").unwrap();

        assert!(pkg3_idx < pkg2_idx);
        assert!(pkg2_idx < pkg1_idx);
    }

    #[test]
    fn test_topological_sort_diamond() {
        let mut graph = DependencyGraph::new();
        graph.add_package("root".to_string(), Version::new(1, 0, 0));
        graph.add_package("left".to_string(), Version::new(1, 0, 0));
        graph.add_package("right".to_string(), Version::new(1, 0, 0));
        graph.add_package("bottom".to_string(), Version::new(1, 0, 0));

        // root depends on left and right
        graph.add_edge("root", "left").unwrap();
        graph.add_edge("root", "right").unwrap();
        // both left and right depend on bottom
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
    fn test_get_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));
        graph.add_package("pkg2".to_string(), Version::new(1, 0, 0));
        graph.add_edge("pkg1", "pkg2").unwrap();

        let deps = graph.get_dependencies("pkg1");
        assert!(deps.is_some());
        assert_eq!(deps.unwrap().len(), 1);
        assert!(deps.unwrap().contains("pkg2"));
    }

    #[test]
    fn test_get_version() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 2, 3));

        let version = graph.get_version("pkg1");
        assert!(version.is_some());
        assert_eq!(version.unwrap(), &Version::new(1, 2, 3));
    }

    #[test]
    fn test_packages() {
        let mut graph = DependencyGraph::new();
        graph.add_package("pkg1".to_string(), Version::new(1, 0, 0));
        graph.add_package("pkg2".to_string(), Version::new(1, 0, 0));

        let packages = graph.packages();
        assert_eq!(packages.len(), 2);
        assert!(packages.contains(&"pkg1".to_string()));
        assert!(packages.contains(&"pkg2".to_string()));
    }
}
