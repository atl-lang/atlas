//! Cache invalidation logic for incremental builds

use std::collections::{HashMap, HashSet};

/// Invalidation reason
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidationReason {
    /// Source file content changed
    SourceChanged,
    /// Dependency changed
    DependencyChanged(String),
    /// Compiler version changed
    CompilerVersionChanged,
    /// Build configuration changed
    ConfigurationChanged,
    /// Manual invalidation
    Manual,
}

/// Invalidation set - modules that need recompilation
#[derive(Debug)]
pub struct InvalidationSet {
    /// Set of module names to invalidate
    modules: HashSet<String>,
    /// Reason for each invalidation
    reasons: HashMap<String, InvalidationReason>,
}

impl InvalidationSet {
    /// Create a new empty invalidation set
    pub fn new() -> Self {
        Self {
            modules: HashSet::new(),
            reasons: HashMap::new(),
        }
    }

    /// Add a module to the invalidation set
    pub fn invalidate(&mut self, module: String, reason: InvalidationReason) {
        self.modules.insert(module.clone());
        self.reasons.insert(module, reason);
    }

    /// Check if a module is invalidated
    pub fn is_invalidated(&self, module: &str) -> bool {
        self.modules.contains(module)
    }

    /// Get all invalidated modules
    pub fn modules(&self) -> &HashSet<String> {
        &self.modules
    }

    /// Get invalidation reason for a module
    pub fn reason(&self, module: &str) -> Option<&InvalidationReason> {
        self.reasons.get(module)
    }

    /// Number of invalidated modules
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Check if invalidation set is empty
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Clear the invalidation set
    pub fn clear(&mut self) {
        self.modules.clear();
        self.reasons.clear();
    }
}

impl Default for InvalidationSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute transitive invalidation set
///
/// Given a set of directly changed modules and a dependency graph,
/// compute all modules that need to be recompiled (including transitive dependencies).
pub fn compute_transitive_invalidation(
    changed_modules: &HashSet<String>,
    dependencies: &HashMap<String, Vec<String>>,
) -> InvalidationSet {
    let mut invalidation = InvalidationSet::new();

    // Add all directly changed modules
    for module in changed_modules {
        invalidation.invalidate(module.clone(), InvalidationReason::SourceChanged);
    }

    // Propagate invalidation to dependents (fixed-point iteration)
    loop {
        let mut added_any = false;

        for (module, deps) in dependencies {
            if invalidation.is_invalidated(module) {
                continue; // Already invalidated
            }

            // Check if any dependency is invalidated
            for dep in deps {
                if invalidation.is_invalidated(dep) {
                    invalidation.invalidate(
                        module.clone(),
                        InvalidationReason::DependencyChanged(dep.clone()),
                    );
                    added_any = true;
                    break;
                }
            }
        }

        if !added_any {
            break; // Fixed point reached
        }
    }

    invalidation
}

/// Build reverse dependency graph (module -> modules that depend on it)
pub fn build_reverse_dependencies(
    dependencies: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();

    for (module, deps) in dependencies {
        for dep in deps {
            reverse_deps
                .entry(dep.clone())
                .or_default()
                .push(module.clone());
        }
    }

    reverse_deps
}

/// Compute selective invalidation (only affected modules)
///
/// More efficient than transitive invalidation for large dependency graphs.
/// Uses reverse dependency graph to propagate changes upward.
pub fn compute_selective_invalidation(
    changed_modules: &HashSet<String>,
    dependencies: &HashMap<String, Vec<String>>,
) -> InvalidationSet {
    let mut invalidation = InvalidationSet::new();
    let reverse_deps = build_reverse_dependencies(dependencies);

    // BFS from changed modules through reverse dependencies
    let mut queue: Vec<String> = changed_modules.iter().cloned().collect();
    let mut visited: HashSet<String> = changed_modules.clone();

    // Add all changed modules to invalidation set
    for module in changed_modules {
        invalidation.invalidate(module.clone(), InvalidationReason::SourceChanged);
    }

    while let Some(module) = queue.pop() {
        // Get all modules that depend on this one
        if let Some(dependents) = reverse_deps.get(&module) {
            for dependent in dependents {
                if !visited.contains(dependent) {
                    visited.insert(dependent.clone());
                    queue.push(dependent.clone());
                    invalidation.invalidate(
                        dependent.clone(),
                        InvalidationReason::DependencyChanged(module.clone()),
                    );
                }
            }
        }
    }

    invalidation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalidation_set_new() {
        let set = InvalidationSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_invalidation_set_invalidate() {
        let mut set = InvalidationSet::new();
        set.invalidate("module_a".to_string(), InvalidationReason::SourceChanged);

        assert!(!set.is_empty());
        assert_eq!(set.len(), 1);
        assert!(set.is_invalidated("module_a"));
        assert!(!set.is_invalidated("module_b"));
    }

    #[test]
    fn test_invalidation_set_reason() {
        let mut set = InvalidationSet::new();
        set.invalidate("module_a".to_string(), InvalidationReason::SourceChanged);

        let reason = set.reason("module_a").unwrap();
        assert_eq!(*reason, InvalidationReason::SourceChanged);
    }

    #[test]
    fn test_compute_transitive_invalidation_single() {
        let mut changed = HashSet::new();
        changed.insert("module_a".to_string());

        let dependencies = HashMap::new();

        let invalidation = compute_transitive_invalidation(&changed, &dependencies);

        assert_eq!(invalidation.len(), 1);
        assert!(invalidation.is_invalidated("module_a"));
    }

    #[test]
    fn test_compute_transitive_invalidation_chain() {
        // Dependency chain: C -> B -> A (A depends on B, B depends on C)
        let mut changed = HashSet::new();
        changed.insert("module_c".to_string());

        let mut dependencies = HashMap::new();
        dependencies.insert("module_a".to_string(), vec!["module_b".to_string()]);
        dependencies.insert("module_b".to_string(), vec!["module_c".to_string()]);

        let invalidation = compute_transitive_invalidation(&changed, &dependencies);

        // C changed, so B and A should be invalidated too
        assert_eq!(invalidation.len(), 3);
        assert!(invalidation.is_invalidated("module_c"));
        assert!(invalidation.is_invalidated("module_b"));
        assert!(invalidation.is_invalidated("module_a"));
    }

    #[test]
    fn test_compute_transitive_invalidation_diamond() {
        // Diamond: D -> B -> A, D -> C -> A
        let mut changed = HashSet::new();
        changed.insert("module_d".to_string());

        let mut dependencies = HashMap::new();
        dependencies.insert(
            "module_a".to_string(),
            vec!["module_b".to_string(), "module_c".to_string()],
        );
        dependencies.insert("module_b".to_string(), vec!["module_d".to_string()]);
        dependencies.insert("module_c".to_string(), vec!["module_d".to_string()]);

        let invalidation = compute_transitive_invalidation(&changed, &dependencies);

        // D changed, so B, C, and A should all be invalidated
        assert_eq!(invalidation.len(), 4);
        assert!(invalidation.is_invalidated("module_d"));
        assert!(invalidation.is_invalidated("module_b"));
        assert!(invalidation.is_invalidated("module_c"));
        assert!(invalidation.is_invalidated("module_a"));
    }

    #[test]
    fn test_build_reverse_dependencies() {
        // A depends on B and C
        let mut dependencies = HashMap::new();
        dependencies.insert(
            "module_a".to_string(),
            vec!["module_b".to_string(), "module_c".to_string()],
        );

        let reverse = build_reverse_dependencies(&dependencies);

        assert_eq!(reverse.get("module_b").unwrap(), &vec!["module_a"]);
        assert_eq!(reverse.get("module_c").unwrap(), &vec!["module_a"]);
    }

    #[test]
    fn test_compute_selective_invalidation() {
        // Chain: C -> B -> A
        let mut changed = HashSet::new();
        changed.insert("module_c".to_string());

        let mut dependencies = HashMap::new();
        dependencies.insert("module_a".to_string(), vec!["module_b".to_string()]);
        dependencies.insert("module_b".to_string(), vec!["module_c".to_string()]);

        let invalidation = compute_selective_invalidation(&changed, &dependencies);

        assert_eq!(invalidation.len(), 3);
        assert!(invalidation.is_invalidated("module_c"));
        assert!(invalidation.is_invalidated("module_b"));
        assert!(invalidation.is_invalidated("module_a"));
    }

    #[test]
    fn test_selective_invalidation_independent() {
        // A -> B, C -> D (independent chains)
        let mut changed = HashSet::new();
        changed.insert("module_b".to_string());

        let mut dependencies = HashMap::new();
        dependencies.insert("module_a".to_string(), vec!["module_b".to_string()]);
        dependencies.insert("module_c".to_string(), vec!["module_d".to_string()]);

        let invalidation = compute_selective_invalidation(&changed, &dependencies);

        // Only A and B should be invalidated, not C or D
        assert_eq!(invalidation.len(), 2);
        assert!(invalidation.is_invalidated("module_b"));
        assert!(invalidation.is_invalidated("module_a"));
        assert!(!invalidation.is_invalidated("module_c"));
        assert!(!invalidation.is_invalidated("module_d"));
    }
}
