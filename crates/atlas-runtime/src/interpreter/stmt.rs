//! Statement execution

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

use crate::ast::*;
use crate::interpreter::{ControlFlow, Interpreter, UserFunction};
use crate::value::{FunctionRef, RuntimeError, Value};
use std::collections::{HashMap, HashSet};

impl Interpreter {
    /// Execute a statement
    pub(super) fn eval_statement(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        // Check execution timeout at statement boundaries
        self.check_timeout()?;

        match stmt {
            Stmt::VarDecl(var) => self.eval_var_decl(var),
            Stmt::FunctionDecl(func) => {
                // Nested function declaration
                // Create scoped name to avoid collisions between nested functions
                let scoped_name = format!("{}_{}", func.name.name, self.next_func_id);
                self.next_func_id += 1;

                // Create FunctionRef value
                let func_value = Value::Function(FunctionRef {
                    name: scoped_name, // Internal scoped name for lookup
                    arity: func.params.len(),
                    bytecode_offset: 0, // Not used in interpreter
                    local_count: 0,     // Not used in interpreter
                    param_ownership: vec![],
                    param_names: vec![],
                    return_ownership: None,
                    is_async: func.is_async,
                });

                // Store in current scope (functions are immutable bindings)
                if self.locals.is_empty() {
                    // Global scope (shouldn't happen for nested functions, but handle it)
                    self.globals
                        .insert(func.name.name.clone(), (func_value.clone(), false));
                } else {
                    // Local scope - this is the normal case for nested functions
                    let scope =
                        self.locals
                            .last_mut()
                            .ok_or_else(|| RuntimeError::InternalError {
                                msg: "Missing scope for nested function declaration".to_string(),
                                span: func.span,
                            })?;
                    scope.insert(func.name.name.clone(), (func_value.clone(), false));
                }

                // Snapshot locals for closure capture (named functions can close over outer vars).
                // This matches VM capture-by-value semantics for nested functions.
                let param_names: HashSet<&str> =
                    func.params.iter().map(|p| p.name.name.as_str()).collect();
                let mut captured: HashMap<String, Value> = HashMap::new();
                for scope in self.locals.iter().skip(1) {
                    for (var_name, (value, _mutable)) in scope {
                        if !param_names.contains(var_name.as_str()) {
                            captured
                                .entry(var_name.clone())
                                .or_insert_with(|| value.clone());
                        }
                    }
                }

                // Store function body with scoped internal name
                let scoped_name = match &func_value {
                    Value::Function(func_ref) => func_ref.name.clone(),
                    _ => unreachable!(),
                };
                self.function_bodies.insert(
                    scoped_name,
                    UserFunction {
                        name: func.name.name.clone(),
                        params: func.params.clone(),
                        body: func.body.clone(),
                        captured,
                    },
                );

                Ok(Value::Null)
            }
            Stmt::Assign(assign) => self.eval_assign(assign),
            Stmt::CompoundAssign(compound) => self.eval_compound_assign(compound),
            Stmt::If(if_stmt) => self.eval_if(if_stmt),
            Stmt::While(while_stmt) => self.eval_while(while_stmt),
            Stmt::ForIn(for_in_stmt) => self.eval_for_in(for_in_stmt),
            Stmt::Return(return_stmt) => self.eval_return(return_stmt),
            Stmt::Break(_) => {
                self.control_flow = ControlFlow::Break;
                Ok(Value::Null)
            }
            Stmt::Continue(_) => {
                self.control_flow = ControlFlow::Continue;
                Ok(Value::Null)
            }
            Stmt::Expr(expr_stmt) => self.eval_expr(&expr_stmt.expr),
        }
    }

    /// Evaluate a variable declaration
    fn eval_var_decl(&mut self, var: &VarDecl) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(&var.init)?;
        let scope = self
            .locals
            .last_mut()
            .ok_or_else(|| RuntimeError::InternalError {
                msg: "Missing scope for variable declaration".to_string(),
                span: var.span,
            })?;
        // Store with mutability flag from the declaration
        scope.insert(var.name.name.clone(), (value, var.mutable));
        Ok(Value::Null)
    }

    /// Evaluate an assignment
    fn eval_assign(&mut self, assign: &Assign) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(&assign.value)?;

        match &assign.target {
            AssignTarget::Name(id) => {
                self.set_variable(&id.name, value, assign.span)?;
                // H-177/H-178: assignment rebinds the variable — clear consumed flag.
                // x = f(own x) pattern: value moved in, returned, rebound — now valid again.
                for scope in self.consumed_locals.iter_mut() {
                    scope.remove(&id.name);
                }
            }
            AssignTarget::Index {
                target,
                index,
                span,
            } => {
                let idx_val = self.eval_expr(index)?;
                self.assign_at_index(target, idx_val, value, *span)?;
            }
            AssignTarget::Member {
                target,
                member,
                span,
            } => {
                self.assign_at_member(target, member, value, *span)?;
            }
        }

        Ok(Value::Null)
    }

    /// Evaluate a compound assignment (+=, -=, *=, /=, %=)
    ///
    /// H-004 fix: For index targets (arr[idx] += val), we evaluate target and index
    /// exactly once to avoid side effects being triggered multiple times.
    fn eval_compound_assign(&mut self, compound: &CompoundAssign) -> Result<Value, RuntimeError> {
        // For index targets, evaluate target and index once upfront and cache the values.
        // This prevents side effects from being evaluated twice (H-004).
        let cached_index_parts: Option<(Value, Value, crate::span::Span)> =
            if let AssignTarget::Index {
                target,
                index,
                span,
            } = &compound.target
            {
                let arr_val = self.eval_expr(target.as_ref())?;
                let idx_val = self.eval_expr(index.as_ref())?;
                Some((arr_val, idx_val, *span))
            } else {
                None
            };

        // Get current value
        let current = match &compound.target {
            AssignTarget::Name(id) => self.get_variable(&id.name, compound.span)?,
            AssignTarget::Index { .. } => {
                // Use cached values
                let (ref arr_val, ref idx_val, span) =
                    cached_index_parts
                        .as_ref()
                        .ok_or_else(|| RuntimeError::InternalError {
                            msg: "Missing cached index parts for compound assignment".to_string(),
                            span: compound.span,
                        })?;
                self.get_array_element(arr_val.clone(), idx_val.clone(), *span)?
            }
            AssignTarget::Member {
                target,
                member,
                span,
            } => self.get_member_value(target, member, *span)?,
        };

        // Get the value to apply
        let value = self.eval_expr(&compound.value)?;

        // Perform the operation
        let result = match (&current, &value) {
            (Value::Number(a), Value::Number(b)) => {
                let res = match compound.op {
                    CompoundOp::AddAssign => a + b,
                    CompoundOp::SubAssign => a - b,
                    CompoundOp::MulAssign => a * b,
                    CompoundOp::DivAssign => {
                        if b == &0.0 {
                            return Err(RuntimeError::DivideByZero {
                                span: compound.span,
                            });
                        }
                        a / b
                    }
                    CompoundOp::ModAssign => {
                        if b == &0.0 {
                            return Err(RuntimeError::DivideByZero {
                                span: compound.span,
                            });
                        }
                        a % b
                    }
                };

                if res.is_nan() || res.is_infinite() {
                    return Err(RuntimeError::InvalidNumericResult {
                        span: compound.span,
                    });
                }

                Value::Number(res)
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Compound assignment requires numbers".to_string(),
                    span: compound.span,
                })
            }
        };

        // Store the result
        match &compound.target {
            AssignTarget::Name(id) => {
                self.set_variable(&id.name, result, compound.span)?;
            }
            AssignTarget::Index { target, span, .. } => {
                // Use cached index value (not re-evaluated)
                let (_, idx_val, _) =
                    cached_index_parts.ok_or_else(|| RuntimeError::InternalError {
                        msg: "Missing cached index parts for compound assignment".to_string(),
                        span: compound.span,
                    })?;
                self.assign_at_index(target, idx_val, result, *span)?;
            }
            AssignTarget::Member {
                target,
                member,
                span,
            } => {
                self.assign_at_member(target, member, result, *span)?;
            }
        }

        Ok(Value::Null)
    }

    /// Evaluate an if statement
    fn eval_if(&mut self, if_stmt: &IfStmt) -> Result<Value, RuntimeError> {
        let condition = self.eval_expr(&if_stmt.cond)?;

        if condition.is_truthy() {
            self.eval_block(&if_stmt.then_block)
        } else if let Some(else_block) = &if_stmt.else_block {
            self.eval_block(else_block)
        } else {
            Ok(Value::Null)
        }
    }

    /// Evaluate a while loop
    fn eval_while(&mut self, while_stmt: &WhileStmt) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Null;

        loop {
            let condition = self.eval_expr(&while_stmt.cond)?;

            if !condition.is_truthy() {
                break;
            }

            last_value = self.eval_block(&while_stmt.body)?;

            match self.control_flow {
                ControlFlow::Break => {
                    self.control_flow = ControlFlow::None;
                    break;
                }
                ControlFlow::Continue => {
                    self.control_flow = ControlFlow::None;
                    continue;
                }
                ControlFlow::Return(_) => {
                    // Propagate return up
                    break;
                }
                ControlFlow::None => {}
            }
        }

        Ok(last_value)
    }

    /// Evaluate a for-in loop
    fn eval_for_in(&mut self, for_in_stmt: &ForInStmt) -> Result<Value, RuntimeError> {
        // Evaluate the iterable expression to get the array
        let iterable = self.eval_expr(&for_in_stmt.iterable)?;

        // Extract array elements (H-116: also accept ranges)
        let elements = match &iterable {
            Value::Array(arr) => arr.iter().cloned().collect::<Vec<_>>(),
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                let start_n = start.unwrap_or(0.0) as i64;
                let end_n = end.ok_or_else(|| RuntimeError::TypeError {
                    msg: "for-in range requires an end bound".to_string(),
                    span: for_in_stmt.iterable.span(),
                })? as i64;
                let iter: Box<dyn Iterator<Item = i64>> = if *inclusive {
                    Box::new(start_n..=end_n)
                } else {
                    Box::new(start_n..end_n)
                };
                iter.map(|i| Value::Number(i as f64)).collect()
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("for-in requires an array, found {}", iterable.type_name()),
                    span: for_in_stmt.iterable.span(),
                });
            }
        };

        // Push new scope for loop variable
        self.push_scope();

        let mut last_value = Value::Null;

        // Iterate over each element
        for element in elements {
            // Bind loop variable to current element (loop variables are mutable)
            let scope = self
                .locals
                .last_mut()
                .ok_or_else(|| RuntimeError::InternalError {
                    msg: "Missing scope for loop variable binding".to_string(),
                    span: for_in_stmt.span,
                })?;
            scope.insert(for_in_stmt.variable.name.clone(), (element, true));

            // Execute body
            last_value = self.eval_block(&for_in_stmt.body)?;

            // Handle control flow
            match self.control_flow {
                ControlFlow::Break => {
                    self.control_flow = ControlFlow::None;
                    break;
                }
                ControlFlow::Continue => {
                    self.control_flow = ControlFlow::None;
                    // Continue to next iteration
                }
                ControlFlow::Return(_) => {
                    // Propagate return up
                    break;
                }
                ControlFlow::None => {}
            }
        }

        self.pop_scope();
        Ok(last_value)
    }

    /// Evaluate a return statement
    fn eval_return(&mut self, return_stmt: &ReturnStmt) -> Result<Value, RuntimeError> {
        let value = if let Some(expr) = &return_stmt.value {
            self.eval_expr(expr)?
        } else {
            Value::Null
        };

        self.control_flow = ControlFlow::Return(value.clone());
        Ok(value)
    }

    /// Evaluate a block with support for tail expressions (implicit returns)
    pub(super) fn eval_block(&mut self, block: &Block) -> Result<Value, RuntimeError> {
        self.push_scope();

        let last_stmt_index = block.statements.len().saturating_sub(1);
        let use_last_stmt_value = block.tail_expr.is_none();
        let mut last_stmt_value: Option<Value> = None;

        // Execute all statements
        for (idx, stmt) in block.statements.iter().enumerate() {
            let value = self.eval_statement(stmt)?;

            // Check for control flow
            if self.control_flow != ControlFlow::None {
                self.pop_scope();
                return Ok(Value::Null);
            }

            if use_last_stmt_value && idx == last_stmt_index {
                last_stmt_value = Some(value);
            }
        }

        // Evaluate tail expression if present (implicit return)
        let result = if let Some(tail) = &block.tail_expr {
            self.eval_expr(tail)?
        } else if let Some(value) = last_stmt_value {
            value
        } else {
            Value::Null
        };

        self.pop_scope();
        Ok(result)
    }
}
