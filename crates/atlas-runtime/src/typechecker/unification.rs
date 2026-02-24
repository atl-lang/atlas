//! Advanced unification algorithm for type inference
//!
//! Provides constraint-based type inference with:
//! - Constraint accumulation and batch solving
//! - Occurs check to prevent infinite types
//! - Structural unification for compound types
//! - Constraint-aware unification respecting bounds
//! - Backtracking unification for union types
//! - Detailed, actionable error messages

use crate::types::{Type, TypeParamDef};
use std::collections::HashMap;

// ============================================================================
// Types and Errors
// ============================================================================

/// A type constraint to be solved
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Two types must be equal
    Equal(Type, Type),
    /// A type must be assignable to another
    Assignable { from: Type, to: Type },
    /// A type must satisfy a bound
    Bound { ty: Type, bound: Type },
}

/// Error produced during unification
#[derive(Debug, Clone, PartialEq)]
pub enum UnificationError {
    /// Types cannot be unified
    Mismatch { expected: Type, found: Type },
    /// Occurs check failed: type variable would create infinite type
    InfiniteType { var: String, ty: Type },
    /// A bound constraint was violated
    ConstraintViolation {
        ty: Type,
        bound: Type,
        detail: String,
    },
    /// A constraint could not be solved
    Unsolvable { detail: String },
}

impl UnificationError {
    /// Human-readable message for this error
    pub fn message(&self) -> String {
        match self {
            Self::Mismatch { expected, found } => format!(
                "type mismatch: expected {}, found {}",
                expected.display_name(),
                found.display_name()
            ),
            Self::InfiniteType { var, ty } => format!(
                "infinite type: '{}' cannot equal {}",
                var,
                ty.display_name()
            ),
            Self::ConstraintViolation { ty, bound, detail } => format!(
                "'{}' does not satisfy constraint '{}': {}",
                ty.display_name(),
                bound.display_name(),
                detail
            ),
            Self::Unsolvable { detail } => format!("unsolvable constraint: {}", detail),
        }
    }
}

// ============================================================================
// Unification Engine
// ============================================================================

/// Advanced unification engine
///
/// Accumulates type constraints and solves them in batch.
/// Supports backtracking for union types and constraint-aware binding.
pub struct UnificationEngine {
    /// Accumulated constraints to solve
    constraints: Vec<Constraint>,
    /// Current substitutions: type variable name -> concrete type
    substitutions: HashMap<String, Type>,
    /// Bounds for named type parameters
    bounds: HashMap<String, Type>,
    /// Counter for generating fresh type variable IDs
    next_var_id: u32,
}

impl UnificationEngine {
    /// Create a new unification engine
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            substitutions: HashMap::new(),
            bounds: HashMap::new(),
            next_var_id: 0,
        }
    }

    /// Create a fresh type variable (named `?hint<id>`)
    pub fn fresh_var(&mut self, hint: &str) -> Type {
        let id = self.next_var_id;
        self.next_var_id += 1;
        Type::TypeParameter {
            name: format!("?{}{}", hint, id),
        }
    }

    /// Declare a bound for a named type parameter
    pub fn add_bound(&mut self, param: &str, bound: Type) {
        self.bounds.insert(param.to_string(), bound);
    }

    /// Add a constraint: `a` must equal `b`
    pub fn constrain_equal(&mut self, a: Type, b: Type) {
        self.constraints.push(Constraint::Equal(a, b));
    }

    /// Add a constraint: `from` must be assignable to `to`
    pub fn constrain_assignable(&mut self, from: Type, to: Type) {
        self.constraints.push(Constraint::Assignable { from, to });
    }

    /// Add a bound constraint: `ty` must satisfy `bound`
    pub fn constrain_bound(&mut self, ty: Type, bound: Type) {
        self.constraints.push(Constraint::Bound { ty, bound });
    }

    /// Solve all accumulated constraints, returning any errors
    pub fn solve(&mut self) -> Vec<UnificationError> {
        let constraints = std::mem::take(&mut self.constraints);
        let mut errors = Vec::new();
        for constraint in constraints {
            if let Err(e) = self.solve_one(constraint) {
                errors.push(e);
            }
        }
        errors
    }

    /// Simplify constraints by applying current substitutions and removing trivial ones
    pub fn simplify(&mut self) {
        let constraints = std::mem::take(&mut self.constraints);
        self.constraints = constraints
            .into_iter()
            .map(|c| match c {
                Constraint::Equal(a, b) => Constraint::Equal(self.apply(&a), self.apply(&b)),
                Constraint::Bound { ty, bound } => Constraint::Bound {
                    ty: self.apply(&ty),
                    bound: self.apply(&bound),
                },
                Constraint::Assignable { from, to } => Constraint::Assignable {
                    from: self.apply(&from),
                    to: self.apply(&to),
                },
            })
            .filter(|c| !is_trivially_satisfied(c))
            .collect();
    }

    /// Solve a single constraint
    #[allow(clippy::result_large_err)]
    fn solve_one(&mut self, constraint: Constraint) -> Result<(), UnificationError> {
        match constraint {
            Constraint::Equal(a, b) => self.unify(a, b),
            Constraint::Assignable { from, to } => {
                let from_applied = self.apply(&from);
                let to_applied = self.apply(&to);
                if from_applied.is_assignable_to(&to_applied) {
                    Ok(())
                } else {
                    self.unify(from, to)
                }
            }
            Constraint::Bound { ty, bound } => {
                let ty_applied = self.apply(&ty);
                let bound_applied = self.apply(&bound);
                if ty_applied.is_assignable_to(&bound_applied) {
                    Ok(())
                } else {
                    Err(UnificationError::ConstraintViolation {
                        ty: ty_applied.clone(),
                        bound: bound_applied.clone(),
                        detail: format!(
                            "{} does not satisfy {}",
                            ty_applied.display_name(),
                            bound_applied.display_name()
                        ),
                    })
                }
            }
        }
    }

    /// Core unification: make types `a` and `b` equal
    #[allow(clippy::result_large_err)]
    pub fn unify(&mut self, a: Type, b: Type) -> Result<(), UnificationError> {
        let a = self.apply(&a).normalized();
        let b = self.apply(&b).normalized();

        match (&a, &b) {
            // Equal types trivially unify
            _ if a == b => Ok(()),

            // Type variable unifies with anything
            (Type::TypeParameter { name }, _) => {
                let name = name.clone();
                self.bind(name, b)
            }
            (_, Type::TypeParameter { name }) => {
                let name = name.clone();
                self.bind(name, a)
            }

            // Unknown is compatible with everything (error recovery)
            (Type::Unknown, _) | (_, Type::Unknown) => Ok(()),

            // Arrays: unify element types
            (Type::Array(ea), Type::Array(eb)) => {
                let ea = *ea.clone();
                let eb = *eb.clone();
                self.unify(ea, eb)
            }

            // Functions: unify parameter types and return types
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                    ..
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                    ..
                },
            ) => {
                if p1.len() != p2.len() {
                    return Err(UnificationError::Mismatch {
                        expected: a.clone(),
                        found: b.clone(),
                    });
                }
                let pairs: Vec<(Type, Type)> = p1
                    .iter()
                    .zip(p2.iter())
                    .map(|(x, y)| (x.clone(), y.clone()))
                    .collect();
                for (pa, pb) in pairs {
                    self.unify(pa, pb)?;
                }
                let ra = *r1.clone();
                let rb = *r2.clone();
                self.unify(ra, rb)
            }

            // Generic types: same name, unify each type argument
            (
                Type::Generic {
                    name: n1,
                    type_args: args1,
                },
                Type::Generic {
                    name: n2,
                    type_args: args2,
                },
            ) => {
                if n1 != n2 || args1.len() != args2.len() {
                    return Err(UnificationError::Mismatch {
                        expected: a.clone(),
                        found: b.clone(),
                    });
                }
                let pairs: Vec<(Type, Type)> = args1
                    .iter()
                    .zip(args2.iter())
                    .map(|(x, y)| (x.clone(), y.clone()))
                    .collect();
                for (arg_a, arg_b) in pairs {
                    self.unify(arg_a, arg_b)?;
                }
                Ok(())
            }

            // Structural types: unify matching member types
            (Type::Structural { members: m1 }, Type::Structural { members: m2 }) => {
                let m2_clone = m2.clone();
                for req in &m2_clone {
                    if let Some(found) = m1.iter().find(|m| m.name == req.name) {
                        let found_ty = found.ty.clone();
                        let req_ty = req.ty.clone();
                        self.unify(found_ty, req_ty)?;
                    }
                }
                Ok(())
            }

            // Union type (right side): try to unify with any member via backtracking
            (other, Type::Union(members)) => {
                let other = other.clone();
                let members = members.clone();
                self.unify_with_union(other, members)
            }
            // Union type (left side): try to unify with any member via backtracking
            (Type::Union(members), other) => {
                let other = other.clone();
                let members = members.clone();
                self.unify_with_union(other, members)
            }

            // Alias: unify against the target type
            (Type::Alias { target, .. }, other) => {
                let t = *target.clone();
                let o = other.clone();
                self.unify(t, o)
            }
            (other, Type::Alias { target, .. }) => {
                let o = other.clone();
                let t = *target.clone();
                self.unify(o, t)
            }

            // Incompatible types
            _ => Err(UnificationError::Mismatch {
                expected: a,
                found: b,
            }),
        }
    }

    /// Try to unify a type with any member of a union (backtracking)
    #[allow(clippy::result_large_err)]
    fn unify_with_union(&mut self, ty: Type, members: Vec<Type>) -> Result<(), UnificationError> {
        // First try exact structural match
        for member in &members {
            let mut probe = UnificationEngine::new();
            probe.substitutions = self.substitutions.clone();
            probe.bounds = self.bounds.clone();
            if probe.unify(ty.clone(), member.clone()).is_ok() {
                // Commit this branch's substitutions
                self.substitutions = probe.substitutions;
                return Ok(());
            }
        }
        // No member matched
        Err(UnificationError::Mismatch {
            expected: Type::Union(members),
            found: ty,
        })
    }

    /// Bind a type variable to a concrete type
    #[allow(clippy::result_large_err)]
    fn bind(&mut self, var: String, ty: Type) -> Result<(), UnificationError> {
        // If already bound, unify the existing and new types
        if let Some(existing) = self.substitutions.get(&var).cloned() {
            return self.unify(existing, ty);
        }

        // Occurs check: prevent circular types like T = Option<T>
        if self.occurs_in(&var, &ty) {
            return Err(UnificationError::InfiniteType { var, ty });
        }

        // Check declared bound if any
        if let Some(bound) = self.bounds.get(&var).cloned() {
            let ty_norm = ty.normalized();
            if !ty_norm.is_assignable_to(&bound) {
                return Err(UnificationError::ConstraintViolation {
                    ty: ty_norm,
                    bound,
                    detail: format!("bound not satisfied for '{}'", var),
                });
            }
        }

        self.substitutions.insert(var, ty);
        Ok(())
    }

    /// Apply current substitutions to a type (fully resolved)
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::TypeParameter { name } => {
                if let Some(sub) = self.substitutions.get(name) {
                    self.apply(sub)
                } else {
                    ty.clone()
                }
            }
            Type::Array(elem) => Type::Array(Box::new(self.apply(elem))),
            Type::Function {
                type_params,
                params,
                return_type,
            } => Type::Function {
                type_params: type_params
                    .iter()
                    .map(|tp| TypeParamDef {
                        name: tp.name.clone(),
                        bound: tp.bound.as_ref().map(|b| Box::new(self.apply(b))),
                        trait_bounds: tp.trait_bounds.clone(),
                    })
                    .collect(),
                params: params.iter().map(|p| self.apply(p)).collect(),
                return_type: Box::new(self.apply(return_type)),
            },
            Type::Generic { name, type_args } => Type::Generic {
                name: name.clone(),
                type_args: type_args.iter().map(|a| self.apply(a)).collect(),
            },
            Type::Alias {
                name,
                type_args,
                target,
            } => Type::Alias {
                name: name.clone(),
                type_args: type_args.iter().map(|a| self.apply(a)).collect(),
                target: Box::new(self.apply(target)),
            },
            Type::Structural { members } => Type::Structural {
                members: members
                    .iter()
                    .map(|m| crate::types::StructuralMemberType {
                        name: m.name.clone(),
                        ty: self.apply(&m.ty),
                    })
                    .collect(),
            },
            Type::Union(members) => Type::union(members.iter().map(|m| self.apply(m)).collect()),
            Type::Intersection(members) => {
                Type::intersection(members.iter().map(|m| self.apply(m)).collect())
            }
            _ => ty.clone(),
        }
    }

    /// Occurs check: does variable `var` appear free in `ty`?
    fn occurs_in(&self, var: &str, ty: &Type) -> bool {
        match ty {
            Type::TypeParameter { name } => {
                if name == var {
                    return true;
                }
                // Transitively check through substitutions
                if let Some(sub) = self.substitutions.get(name) {
                    return self.occurs_in(var, sub);
                }
                false
            }
            Type::Array(elem) => self.occurs_in(var, elem),
            Type::Function {
                params,
                return_type,
                ..
            } => params.iter().any(|p| self.occurs_in(var, p)) || self.occurs_in(var, return_type),
            Type::Generic { type_args, .. } => type_args.iter().any(|a| self.occurs_in(var, a)),
            Type::Alias {
                type_args, target, ..
            } => type_args.iter().any(|a| self.occurs_in(var, a)) || self.occurs_in(var, target),
            Type::Structural { members } => members.iter().any(|m| self.occurs_in(var, &m.ty)),
            Type::Union(members) | Type::Intersection(members) => {
                members.iter().any(|m| self.occurs_in(var, m))
            }
            _ => false,
        }
    }

    /// Get the substitution for a named type variable
    pub fn get_substitution(&self, var: &str) -> Option<&Type> {
        self.substitutions.get(var)
    }

    /// Get all current substitutions
    pub fn substitutions(&self) -> &HashMap<String, Type> {
        &self.substitutions
    }

    /// Get the names of type variables that have not been solved
    pub fn unsolved_vars<'a>(&'a self, vars: &'a [String]) -> Vec<&'a str> {
        vars.iter()
            .filter(|v| !self.substitutions.contains_key(*v))
            .map(|v| v.as_str())
            .collect()
    }

    /// Number of pending constraints
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }
}

impl Default for UnificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn is_trivially_satisfied(constraint: &Constraint) -> bool {
    match constraint {
        Constraint::Equal(a, b) => a.normalized() == b.normalized(),
        Constraint::Assignable { from, to } => from.is_assignable_to(to),
        Constraint::Bound { .. } => false,
    }
}

// ============================================================================
// Tests
// ============================================================================
