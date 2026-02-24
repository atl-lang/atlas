//! Flow-sensitive type tracking
//!
//! Tracks how types change through control flow:
//! - Type refinement after assignments (immutable: precise, mutable: conservative)
//! - Type narrowing in conditional branches
//! - Type widening at merge points (join from branches)
//! - Fixpoint iteration for loops
//! - Impossible branch detection (Never type)

use crate::types::Type;
use std::collections::HashMap;

// ============================================================================
// FlowState: Type state at a point in control flow
// ============================================================================

/// Per-variable type metadata
#[derive(Debug, Clone)]
pub struct TypeEntry {
    /// Current type of this variable
    pub ty: Type,
    /// Whether the variable is declared mutable (`mut x`)
    pub mutable: bool,
    /// Whether the type was narrowed from a wider declared type
    pub narrowed: bool,
}

/// Type state at a single point in control flow
#[derive(Debug, Clone, Default)]
pub struct FlowState {
    types: HashMap<String, TypeEntry>,
}

impl FlowState {
    /// Create an empty flow state
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a variable with an initial type
    pub fn set(&mut self, name: String, ty: Type, mutable: bool) {
        self.types.insert(
            name,
            TypeEntry {
                ty,
                mutable,
                narrowed: false,
            },
        );
    }

    /// Narrow a variable's type in this state
    pub fn narrow(&mut self, name: &str, narrowed_ty: Type) {
        if let Some(entry) = self.types.get_mut(name) {
            entry.ty = narrowed_ty;
            entry.narrowed = true;
        }
    }

    /// Get the full type entry for a variable
    pub fn get(&self, name: &str) -> Option<&TypeEntry> {
        self.types.get(name)
    }

    /// Get just the type for a variable
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name).map(|e| &e.ty)
    }

    /// Whether a variable's type is the Never type (impossible branch)
    pub fn is_impossible(&self, name: &str) -> bool {
        self.types
            .get(name)
            .map(|e| e.ty.normalized() == Type::Never)
            .unwrap_or(false)
    }

    /// Refine a variable's type after an assignment.
    ///
    /// - Immutable: precise update (the declared type is replaced by the assigned type)
    /// - Mutable: conservative widening (LUB of old and new type)
    pub fn refine_after_assignment(&mut self, name: &str, new_type: Type) {
        if let Some(entry) = self.types.get_mut(name) {
            if entry.mutable {
                entry.ty = widen_types(&entry.ty, &new_type);
                entry.narrowed = false;
            } else {
                entry.ty = new_type;
                entry.narrowed = true;
            }
        }
    }

    /// Iterate over all (name, type) pairs in this state
    pub fn all_types(&self) -> impl Iterator<Item = (&str, &Type)> {
        self.types.iter().map(|(k, v)| (k.as_str(), &v.ty))
    }

    /// Number of tracked variables
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Whether the state is empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

// ============================================================================
// Merge / Widen operations
// ============================================================================

/// Widen two types to a common supertype.
///
/// - Same type → returns that type
/// - Unknown → returns the other type (Unknown is the bottom type for inference)
/// - Never → returns the other type (Never is the empty type)
/// - Otherwise → forms a union
pub fn widen_types(a: &Type, b: &Type) -> Type {
    let a_norm = a.normalized();
    let b_norm = b.normalized();

    if a_norm == b_norm {
        return a_norm;
    }
    if a_norm == Type::Unknown {
        return b_norm;
    }
    if b_norm == Type::Unknown {
        return a_norm;
    }
    if a_norm == Type::Never {
        return b_norm;
    }
    if b_norm == Type::Never {
        return a_norm;
    }

    // Array covariance: widen element types
    if let (Type::Array(ea), Type::Array(eb)) = (&a_norm, &b_norm) {
        return Type::Array(Box::new(widen_types(ea, eb)));
    }

    Type::union(vec![a.clone(), b.clone()])
}

/// Merge two flow states at a join point (end of if-else).
///
/// Variables in both states get their types widened.
/// Variables only in one state retain their type (the other branch was unreachable).
pub fn merge_flow_states(then_state: &FlowState, else_state: &FlowState) -> FlowState {
    let mut merged = FlowState::new();

    for (name, then_entry) in &then_state.types {
        if let Some(else_entry) = else_state.types.get(name) {
            let merged_ty = widen_types(&then_entry.ty, &else_entry.ty);
            let mutable = then_entry.mutable || else_entry.mutable;
            merged.types.insert(
                name.clone(),
                TypeEntry {
                    ty: merged_ty,
                    mutable,
                    narrowed: false,
                },
            );
        } else {
            merged.types.insert(name.clone(), then_entry.clone());
        }
    }

    for (name, else_entry) in &else_state.types {
        if !then_state.types.contains_key(name) {
            merged.types.insert(name.clone(), else_entry.clone());
        }
    }

    merged
}

/// Compute the widened state for a loop fixpoint iteration.
///
/// Compares pre-loop state with post-loop state:
/// - If types are stable, `fixpoint = true`
/// - Otherwise, widen changed types and return `fixpoint = false`
pub fn widen_loop_state(pre: &FlowState, post: &FlowState) -> (FlowState, bool) {
    let mut widened = FlowState::new();
    let mut reached_fixpoint = true;

    for (name, pre_entry) in &pre.types {
        let new_ty = if let Some(post_entry) = post.types.get(name) {
            let pre_norm = pre_entry.ty.normalized();
            let post_norm = post_entry.ty.normalized();
            if pre_norm == post_norm {
                pre_entry.ty.clone()
            } else {
                reached_fixpoint = false;
                widen_types(&pre_entry.ty, &post_entry.ty)
            }
        } else {
            pre_entry.ty.clone()
        };

        widened.types.insert(
            name.clone(),
            TypeEntry {
                ty: new_ty,
                mutable: pre_entry.mutable,
                narrowed: false,
            },
        );
    }

    // New variables introduced in the loop body
    for (name, post_entry) in &post.types {
        if !pre.types.contains_key(name) {
            widened.types.insert(name.clone(), post_entry.clone());
        }
    }

    (widened, reached_fixpoint)
}

// ============================================================================
// FlowSensitiveTracker: Main tracking structure
// ============================================================================

/// Tracks type state through scopes and control flow
pub struct FlowSensitiveTracker {
    /// Stack of flow states (scope nesting)
    scope_stack: Vec<FlowState>,
    /// Maximum fixpoint iterations before giving up
    max_iterations: usize,
}

impl FlowSensitiveTracker {
    pub fn new() -> Self {
        Self {
            scope_stack: vec![FlowState::new()],
            max_iterations: 10,
        }
    }

    /// Enter a new scope, inheriting the current state
    pub fn enter_scope(&mut self) {
        let snapshot = self.current().clone();
        self.scope_stack.push(snapshot);
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    /// Get the current flow state (immutable)
    pub fn current(&self) -> &FlowState {
        self.scope_stack.last().expect("scope stack never empty")
    }

    /// Get the current flow state (mutable)
    fn current_mut(&mut self) -> &mut FlowState {
        self.scope_stack
            .last_mut()
            .expect("scope stack never empty")
    }

    /// Declare a variable with its initial type
    pub fn declare(&mut self, name: String, ty: Type, mutable: bool) {
        self.current_mut().set(name, ty, mutable);
    }

    /// Get the type of a variable, searching all enclosing scopes
    pub fn get_type(&self, name: &str) -> Option<&Type> {
        for state in self.scope_stack.iter().rev() {
            if let Some(ty) = state.get_type(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Narrow a variable's type in the current scope
    pub fn narrow(&mut self, name: &str, ty: Type) {
        self.current_mut().narrow(name, ty);
    }

    /// Refine a variable's type after an assignment
    pub fn refine_after_assignment(&mut self, name: &str, new_type: Type) {
        self.current_mut().refine_after_assignment(name, new_type);
    }

    /// Check whether a branch is impossible for a variable (type is Never)
    pub fn is_impossible(&self, name: &str) -> bool {
        self.current().is_impossible(name)
    }

    /// Take a snapshot of the current flow state
    pub fn take_snapshot(&self) -> FlowState {
        self.current().clone()
    }

    /// Restore a previously taken snapshot as the current state
    pub fn restore_snapshot(&mut self, state: FlowState) {
        *self.current_mut() = state;
    }

    /// Merge two branch states (for if-else join)
    pub fn merge_branches(&mut self, then_state: FlowState, else_state: FlowState) {
        let merged = merge_flow_states(&then_state, &else_state);
        self.restore_snapshot(merged);
    }

    /// Run a loop body and compute the fixpoint state.
    ///
    /// The closure `body` mutates this tracker to reflect what the loop body does.
    /// We iterate until the type state stabilizes (fixpoint).
    pub fn compute_loop_fixpoint<F>(&mut self, mut body: F) -> FlowState
    where
        F: FnMut(&mut FlowSensitiveTracker),
    {
        let pre_state = self.take_snapshot();
        let mut current = pre_state.clone();

        for _ in 0..self.max_iterations {
            self.restore_snapshot(current.clone());
            body(self);
            let post_state = self.take_snapshot();
            let (widened, fixpoint) = widen_loop_state(&current, &post_state);
            if fixpoint {
                self.restore_snapshot(widened.clone());
                return widened;
            }
            current = widened;
        }

        // Did not converge within max_iterations — return conservative widened state
        self.take_snapshot()
    }
}

impl Default for FlowSensitiveTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================
