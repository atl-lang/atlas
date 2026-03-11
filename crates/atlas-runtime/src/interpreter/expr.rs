//! Expression evaluation

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

use crate::ast::*;
use crate::interpreter::{ControlFlow, Interpreter, UserFunction};
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::sync::Arc;

impl Interpreter {
    /// Evaluate an expression
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit, _) => Ok(self.eval_literal(lit)),
            Expr::TemplateString { parts, span } => self.eval_template_string(parts, *span),
            Expr::Identifier(id) => self.get_variable(&id.name, id.span),
            Expr::Binary(binary) => self.eval_binary(binary),
            Expr::Unary(unary) => self.eval_unary(unary),
            Expr::Call(call) => self.eval_call(call),
            Expr::Index(index) => self.eval_index(index),
            Expr::ArrayLiteral(arr) => self.eval_array_literal(arr),
            Expr::Group(group) => self.eval_expr(&group.expr),
            Expr::Match(match_expr) => self.eval_match(match_expr),
            Expr::Member(member) => self.eval_member(member),
            Expr::Try(try_expr) => self.eval_try(try_expr),
            Expr::AnonFn {
                params, body, span, ..
            } => self.eval_anon_fn(params, body, *span),
            Expr::Block(block) => self.eval_block(block),
            Expr::ObjectLiteral(obj) => self.eval_object_literal(obj),
            Expr::StructExpr(struct_expr) => self.eval_struct_expr(struct_expr),
            Expr::Range {
                start,
                end,
                inclusive,
                span,
            } => self.eval_range(start, end, *inclusive, *span),
            Expr::EnumVariant(ev) => self.eval_enum_variant(ev),
            Expr::TupleLiteral { elements, .. } => {
                let mut vals = Vec::with_capacity(elements.len());
                for elem in elements {
                    vals.push(self.eval_expr(elem)?);
                    if self.control_flow != ControlFlow::None {
                        return Ok(Value::Null);
                    }
                }
                Ok(Value::Tuple(Arc::new(vals)))
            }
            Expr::Await { expr, span } => {
                let val = self.eval_expr(expr)?;
                match val {
                    Value::Future(future) => {
                        match future.get_state() {
                            crate::async_runtime::FutureState::Resolved(v) => Ok(v),
                            crate::async_runtime::FutureState::Rejected(e) => {
                                Err(RuntimeError::TypeError {
                                    msg: format!("Awaited future was rejected: {}", e),
                                    span: *span,
                                })
                            }
                            crate::async_runtime::FutureState::Pending => {
                                // Pending futures from stdlib async I/O: block until resolved.
                                // For interpreter-level async fn bodies this state won't occur
                                // because bodies are evaluated eagerly.
                                Err(RuntimeError::TypeError {
                                    msg: "Cannot await a pending future in the interpreter — use the VM for full async concurrency".to_string(),
                                    span: *span,
                                })
                            }
                        }
                    }
                    other => Err(RuntimeError::TypeError {
                        msg: format!(
                            "AT4002: await operand must be Future, got {}",
                            other.type_name()
                        ),
                        span: *span,
                    }),
                }
            }
        }
    }

    fn eval_range(
        &mut self,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        let start_val = match start {
            Some(expr) => match self.eval_expr(expr)? {
                Value::Number(n) => Some(n),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Range bound must be number".to_string(),
                        span,
                    })
                }
            },
            None => None,
        };

        let end_val = match end {
            Some(expr) => match self.eval_expr(expr)? {
                Value::Number(n) => Some(n),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Range bound must be number".to_string(),
                        span,
                    })
                }
            },
            None => None,
        };

        if inclusive && end_val.is_none() {
            return Err(RuntimeError::TypeError {
                msg: "Inclusive range requires an end bound".to_string(),
                span,
            });
        }

        Ok(Value::Range {
            start: start_val,
            end: end_val,
            inclusive,
        })
    }

    /// Evaluate a struct instantiation expression
    ///
    /// For now, struct expressions evaluate to a HashMap value
    fn eval_struct_expr(
        &mut self,
        struct_expr: &crate::ast::StructExpr,
    ) -> Result<Value, RuntimeError> {
        use crate::stdlib::collections::hash::HashKey;
        use crate::stdlib::collections::hashmap::AtlasHashMap;
        use crate::value::ValueHashMap;

        let mut atlas_map = AtlasHashMap::new();

        for field in &struct_expr.fields {
            // Field name is always a string (from identifier)
            let key = HashKey::String(Arc::new(field.name.name.clone()));
            // Evaluate the field value expression
            let value = self.eval_expr(&field.value)?;
            atlas_map.insert(key, value);
        }

        let map = ValueHashMap::from_atlas(atlas_map);
        self.register_struct_type(&map, &struct_expr.name.name);
        Ok(Value::HashMap(map))
    }

    /// Evaluate an enum variant expression
    fn eval_enum_variant(
        &mut self,
        ev: &crate::ast::EnumVariantExpr,
    ) -> Result<Value, RuntimeError> {
        // Evaluate any arguments
        let data = if let Some(args) = &ev.args {
            let mut data = Vec::with_capacity(args.len());
            for arg in args {
                data.push(self.eval_expr(arg)?);
            }
            data
        } else {
            Vec::new()
        };

        // Create the enum value
        Ok(Value::EnumValue {
            enum_name: ev.enum_name.name.clone(),
            variant_name: ev.variant_name.name.clone(),
            data,
        })
    }

    /// Evaluate a literal
    pub(super) fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::string(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Null => Value::Null,
        }
    }

    fn eval_template_string(
        &mut self,
        parts: &[TemplatePart],
        span: Span,
    ) -> Result<Value, RuntimeError> {
        let mut result = String::new();

        for part in parts {
            match part {
                TemplatePart::Literal(text) => {
                    result.push_str(text);
                }
                TemplatePart::Expression(expr) => {
                    let value = self.eval_expr(expr)?;
                    if self.control_flow != ControlFlow::None {
                        return Ok(value);
                    }
                    let string_value = crate::stdlib::types::to_string(&[value], span)?;
                    if let Value::String(s) = string_value {
                        result.push_str(s.as_ref());
                    }
                }
            }
        }

        self.track_memory(Self::estimate_string_size(&result))?;
        Ok(Value::string(result))
    }

    /// Evaluate a binary expression
    fn eval_binary(&mut self, binary: &BinaryExpr) -> Result<Value, RuntimeError> {
        // Short-circuit evaluation for && and ||
        if binary.op == BinaryOp::And {
            let left = self.eval_expr(&binary.left)?;
            if let Value::Bool(false) = left {
                return Ok(Value::Bool(false));
            }
            if let Value::Bool(true) = left {
                let right = self.eval_expr(&binary.right)?;
                if let Value::Bool(b) = right {
                    return Ok(Value::Bool(b));
                }
            }
            return Err(RuntimeError::TypeError {
                msg: "Expected bool for &&".to_string(),
                span: binary.span,
            });
        }

        if binary.op == BinaryOp::Or {
            let left = self.eval_expr(&binary.left)?;
            if let Value::Bool(true) = left {
                return Ok(Value::Bool(true));
            }
            if let Value::Bool(false) = left {
                let right = self.eval_expr(&binary.right)?;
                if let Value::Bool(b) = right {
                    return Ok(Value::Bool(b));
                }
            }
            return Err(RuntimeError::TypeError {
                msg: "Expected bool for ||".to_string(),
                span: binary.span,
            });
        }

        // Regular binary operations
        let left = self.eval_expr(&binary.left)?;
        // If ? operator triggered early return, stop evaluating
        if self.control_flow != ControlFlow::None {
            return Ok(left);
        }
        let right = self.eval_expr(&binary.right)?;
        if self.control_flow != ControlFlow::None {
            return Ok(right);
        }

        match binary.op {
            BinaryOp::Add => match (&left, &right) {
                (Value::Number(a), Value::Number(b)) => {
                    let result = a + b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult { span: binary.span });
                    }
                    Ok(Value::Number(result))
                }
                (Value::String(a), Value::String(b)) => {
                    let result = format!("{}{}", a, b);
                    // Track memory for the new concatenated string
                    self.track_memory(Self::estimate_string_size(&result))?;
                    Ok(Value::string(result))
                }
                (Value::Array(a), Value::Array(b)) => {
                    let mut elements = Vec::with_capacity(a.len() + b.len());
                    elements.extend_from_slice(a.as_slice());
                    elements.extend_from_slice(b.as_slice());
                    let estimated_size = Self::estimate_array_size(&elements);
                    self.track_memory(estimated_size)?;
                    Ok(Value::array(elements))
                }
                _ => Err(RuntimeError::TypeError {
                    msg: "Invalid operands for +".to_string(),
                    span: binary.span,
                }),
            },
            BinaryOp::Sub => self.numeric_binary_op(left, right, |a, b| a - b, binary.span),
            BinaryOp::Mul => self.numeric_binary_op(left, right, |a, b| a * b, binary.span),
            BinaryOp::Div => {
                if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivideByZero { span: binary.span });
                    }
                    let result = a / b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult { span: binary.span });
                    }
                    Ok(Value::Number(result))
                } else {
                    Err(RuntimeError::TypeError {
                        msg: "Expected numbers for /".to_string(),
                        span: binary.span,
                    })
                }
            }
            BinaryOp::Mod => {
                if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                    if *b == 0.0 {
                        return Err(RuntimeError::DivideByZero { span: binary.span });
                    }
                    let result = a % b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult { span: binary.span });
                    }
                    Ok(Value::Number(result))
                } else {
                    Err(RuntimeError::TypeError {
                        msg: "Expected numbers for %".to_string(),
                        span: binary.span,
                    })
                }
            }
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::Ne => Ok(Value::Bool(left != right)),
            BinaryOp::Lt => self.numeric_comparison(left, right, |a, b| a < b, binary.span),
            BinaryOp::Le => self.numeric_comparison(left, right, |a, b| a <= b, binary.span),
            BinaryOp::Gt => self.numeric_comparison(left, right, |a, b| a > b, binary.span),
            BinaryOp::Ge => self.numeric_comparison(left, right, |a, b| a >= b, binary.span),
            BinaryOp::And | BinaryOp::Or => {
                // Already handled above
                unreachable!()
            }
        }
    }

    /// Helper for numeric binary operations
    fn numeric_binary_op<F>(
        &self,
        left: Value,
        right: Value,
        op: F,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        if let (Value::Number(a), Value::Number(b)) = (left, right) {
            let result = op(a, b);
            if result.is_nan() || result.is_infinite() {
                return Err(RuntimeError::InvalidNumericResult { span });
            }
            Ok(Value::Number(result))
        } else {
            Err(RuntimeError::TypeError {
                msg: "Expected numbers".to_string(),
                span,
            })
        }
    }

    /// Helper for numeric comparisons
    fn numeric_comparison<F>(
        &self,
        left: Value,
        right: Value,
        op: F,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        if let (Value::Number(a), Value::Number(b)) = (left, right) {
            Ok(Value::Bool(op(a, b)))
        } else {
            Err(RuntimeError::TypeError {
                msg: "Expected numbers for comparison".to_string(),
                span,
            })
        }
    }

    /// Evaluate a unary expression
    fn eval_unary(&mut self, unary: &UnaryExpr) -> Result<Value, RuntimeError> {
        let operand = self.eval_expr(&unary.expr)?;

        match unary.op {
            UnaryOp::Negate => {
                if let Value::Number(n) = operand {
                    Ok(Value::Number(-n))
                } else {
                    Err(RuntimeError::TypeError {
                        msg: "Expected number for -".to_string(),
                        span: unary.span,
                    })
                }
            }
            UnaryOp::Not => {
                if let Value::Bool(b) = operand {
                    Ok(Value::Bool(!b))
                } else {
                    Err(RuntimeError::TypeError {
                        msg: "Expected bool for !".to_string(),
                        span: unary.span,
                    })
                }
            }
        }
    }

    /// Evaluate a function call
    pub(super) fn eval_call(&mut self, call: &CallExpr) -> Result<Value, RuntimeError> {
        // Bare user-defined enum variant constructor: `Unknown(raw)`, `Quit` without EnumName::.
        // Check BEFORE evaluating callee to avoid "undefined variable" errors for variant names.
        // Skip stdlib constructors (Ok, Err, Some, None) — they have their own Value types.
        if let crate::ast::Expr::Identifier(id) = call.callee.as_ref() {
            let is_stdlib_ctor = matches!(id.name.as_str(), "Ok" | "Err" | "Some" | "None");
            if !is_stdlib_ctor {
                if let Some((enum_name, arity)) = self.enum_variants.get(&id.name).cloned() {
                    if arity > 0 {
                        let mut data = Vec::new();
                        for arg in &call.args {
                            data.push(self.eval_expr(arg)?);
                            if self.control_flow != ControlFlow::None {
                                return Ok(match &self.control_flow {
                                    ControlFlow::Return(v) => v.clone(),
                                    _ => Value::Null,
                                });
                            }
                        }
                        return Ok(Value::EnumValue {
                            enum_name,
                            variant_name: id.name.clone(),
                            data,
                        });
                    }
                }
            }
        }

        // Evaluate callee as ANY expression (enables first-class functions)
        let callee_value = self.eval_expr(&call.callee)?;

        // Check for early return from callee evaluation
        if self.control_flow != ControlFlow::None {
            // Propagate control flow (e.g., from ? operator)
            return Ok(match &self.control_flow {
                ControlFlow::Return(v) => v.clone(),
                _ => Value::Null,
            });
        }

        // Evaluate arguments, checking for control flow after each
        let mut args = Vec::new();
        for arg in &call.args {
            let val = self.eval_expr(arg)?;

            // Check for early return from argument evaluation (e.g., ? operator)
            if self.control_flow != ControlFlow::None {
                return Ok(match &self.control_flow {
                    ControlFlow::Return(v) => v.clone(),
                    _ => Value::Null,
                });
            }

            args.push(val);
        }

        self.invoke_callee(callee_value, args, call.span, Some(&call.args))
    }

    fn invoke_callee(
        &mut self,
        callee_value: Value,
        args: Vec<Value>,
        span: crate::span::Span,
        arg_exprs: Option<&[Expr]>,
    ) -> Result<Value, RuntimeError> {
        match callee_value {
            Value::Builtin(ref name) => {
                // Check for array intrinsics (callback-based functions)
                match name.as_ref() {
                    "map" => return self.intrinsic_map(&args, span),
                    "filter" => return self.intrinsic_filter(&args, span),
                    "reduce" => return self.intrinsic_reduce(&args, span),
                    "forEach" | "for_each" => return self.intrinsic_for_each(&args, span),
                    "find" => return self.intrinsic_find(&args, span),
                    "findIndex" | "find_index" => return self.intrinsic_find_index(&args, span),
                    "flatMap" | "flat_map" => return self.intrinsic_flat_map(&args, span),
                    "some" => return self.intrinsic_some(&args, span),
                    "every" => return self.intrinsic_every(&args, span),
                    "sort" => return self.intrinsic_sort(&args, span),
                    "sortBy" | "sort_by" => return self.intrinsic_sort_by(&args, span),
                    "result_map" => return self.intrinsic_result_map(&args, span),
                    "result_map_err" => return self.intrinsic_result_map_err(&args, span),
                    "result_and_then" => return self.intrinsic_result_and_then(&args, span),
                    "result_or_else" => return self.intrinsic_result_or_else(&args, span),
                    "hashMapForEach" | "hash_map_for_each" => {
                        return self.intrinsic_hashmap_for_each(&args, span)
                    }
                    "hashMapMap" | "hash_map_map" => {
                        return self.intrinsic_hashmap_map(&args, span)
                    }
                    "hashMapFilter" | "hash_map_filter" => {
                        return self.intrinsic_hashmap_filter(&args, span)
                    }
                    "hashSetForEach" | "hash_set_for_each" => {
                        return self.intrinsic_hashset_for_each(&args, span)
                    }
                    "hashSetMap" | "hash_set_map" => {
                        return self.intrinsic_hashset_map(&args, span)
                    }
                    "hashSetFilter" | "hash_set_filter" => {
                        return self.intrinsic_hashset_filter(&args, span)
                    }
                    "regexReplaceWith" | "regex_replace_with" => {
                        return self.intrinsic_regex_replace_with(&args, span)
                    }
                    "regexReplaceAllWith" | "regex_replace_all_with" => {
                        return self.intrinsic_regex_replace_all_with(&args, span)
                    }
                    "assertThrows" | "assert_throws" => {
                        return self.intrinsic_assert_throws(&args, span)
                    }
                    "assertNoThrow" | "assert_no_throw" => {
                        return self.intrinsic_assert_no_throw(&args, span)
                    }
                    _ => {}
                }

                // Stdlib builtin dispatch
                let security =
                    self.current_security
                        .as_ref()
                        .ok_or_else(|| RuntimeError::InternalError {
                            msg: "Security context not set".to_string(),
                            span,
                        })?;
                let result =
                    crate::stdlib::call_builtin(name, &args, span, security, &self.output_writer)?;
                // CoW write-back: collection mutation builtins return the new collection
                // but the caller's variable still holds the old value. Write it back.
                if let Some(arg_exprs) = arg_exprs {
                    self.apply_cow_writeback(name, result, arg_exprs, span)
                } else {
                    Ok(result)
                }
            }
            Value::Function(func_ref) => {
                // Extern function - check if it's an FFI function
                if let Some(extern_fn) = self.extern_functions.get(&func_ref.name) {
                    // Check FFI permission before calling extern function
                    if let Some(ref security) = self.current_security {
                        // FFI requires process execution permission (it's running native code)
                        if security.check_process(&func_ref.name).is_err() {
                            return Err(RuntimeError::FfiPermissionDenied {
                                function: func_ref.name.clone(),
                            });
                        }
                    }
                    // Call the extern function using FFI
                    return unsafe { extern_fn.call(&args) }.map_err(|e| RuntimeError::TypeError {
                        msg: format!("FFI call error: {}", e),
                        span,
                    });
                }

                // User-defined function - look up body
                if let Some(func) = self.function_bodies.get(&func_ref.name).cloned() {
                    // In debug mode, mark caller bindings consumed for `own` parameters.
                    // Only applies when the argument is a direct variable reference —
                    // literals and expression results have no binding to consume.
                    #[cfg(debug_assertions)]
                    if let Some(arg_exprs) = arg_exprs {
                        for (param, arg_expr) in func.params.iter().zip(arg_exprs.iter()) {
                            if param.ownership == Some(crate::ast::OwnershipAnnotation::Own) {
                                if let Expr::Identifier(id) = arg_expr {
                                    self.mark_consumed(&id.name);
                                }
                            }
                        }
                    }
                    let result = self.call_user_function(&func, args, span)?;
                    // Async fn: wrap the result in an immediately-resolved Future.
                    // The interpreter evaluates async bodies eagerly; the Future wrapper
                    // preserves the Value::Future contract so `await` can unwrap it.
                    if func_ref.is_async {
                        let future = crate::async_runtime::AtlasFuture::resolved(result);
                        return Ok(Value::Future(Arc::new(future)));
                    }
                    return Ok(result);
                }

                Err(RuntimeError::UnknownFunction {
                    name: func_ref.name.clone(),
                    span,
                })
            }
            Value::NativeFunction(native_fn) => {
                // Call the native Rust closure
                native_fn(&args)
            }
            // None() is a valid call that returns Option::None (zero-arg constructor)
            Value::Option(None) if args.is_empty() => Ok(Value::Option(None)),
            _ => Err(RuntimeError::TypeError {
                msg: format!("Cannot call non-function type {}", callee_value.type_name()),
                span,
            }),
        }
    }

    /// Evaluate a member expression (method call)
    ///
    /// Desugars method calls to stdlib function calls:
    ///   value.method(args) → Type_method(value, args)
    pub(super) fn eval_member(&mut self, member: &MemberExpr) -> Result<Value, RuntimeError> {
        // 1. Evaluate target expression
        let target_value = self.eval_expr(&member.target)?;

        if member.args.is_none() {
            return Self::get_member_from_value(target_value, &member.member, member.span);
        }

        // 1b. Check for trait/inherent dispatch (user-defined impl methods).
        // The typechecker annotates `trait_dispatch` when a method is resolved.
        // Empty trait_name = inherent dispatch (D-037): __impl__TypeName__MethodName
        // Non-empty trait_name = trait dispatch:        __impl__TypeName__TraitName__MethodName
        if let Some((type_name, trait_name)) = member.trait_dispatch.borrow().clone() {
            let dispatch_type = if type_name.is_empty() {
                self.struct_name_for_value(&target_value)
                    .unwrap_or_else(|| target_value.type_name())
                    .to_string()
            } else {
                type_name
            };
            let mangled_name = if trait_name.is_empty() {
                format!("__impl__{}__{}", dispatch_type, member.member.name)
            } else {
                format!(
                    "__impl__{}__{}__{}",
                    dispatch_type, trait_name, member.member.name
                )
            };
            // Build argument list: receiver first (self), then method args
            let mut args = vec![target_value];
            if let Some(method_args) = &member.args {
                for arg in method_args {
                    args.push(self.eval_expr(arg)?);
                }
            }
            let func = self
                .function_bodies
                .get(&mangled_name)
                .cloned()
                .ok_or_else(|| RuntimeError::TypeError {
                    msg: format!(
                        "Trait method '{}' not found (impl not registered for this type)",
                        member.member.name
                    ),
                    span: member.span,
                })?;
            return self.call_user_function(&func, args, member.span);
        }

        // 2. Build desugared function name via shared dispatch table.
        // Prefer the static TypeTag set by the typechecker; fall back to dynamic dispatch
        // from the runtime value when the typechecker couldn't infer the type (e.g. `array`
        // annotation resolves to Unknown in some paths, or the typechecker is not run).
        let dynamic_tag = match &target_value {
            Value::Array(_) => Some(crate::method_dispatch::TypeTag::Array),
            Value::HttpResponse(_) => Some(crate::method_dispatch::TypeTag::HttpResponse),
            Value::ProcessOutput(_) => Some(crate::method_dispatch::TypeTag::ProcessOutput),
            Value::String(_) => Some(crate::method_dispatch::TypeTag::String),
            Value::HashMap(_) => Some(crate::method_dispatch::TypeTag::HashMap),
            Value::HashSet(_) => Some(crate::method_dispatch::TypeTag::HashSet),
            Value::Queue(_) => Some(crate::method_dispatch::TypeTag::Queue),
            Value::Stack(_) => Some(crate::method_dispatch::TypeTag::Stack),
            Value::Option(_) => Some(crate::method_dispatch::TypeTag::Option),
            Value::Result(_) => Some(crate::method_dispatch::TypeTag::Result),
            Value::Future(_) => Some(crate::method_dispatch::TypeTag::FutureValue),
            Value::ChannelSender(_) => Some(crate::method_dispatch::TypeTag::ChannelSender),
            Value::ChannelReceiver(_) => Some(crate::method_dispatch::TypeTag::ChannelReceiver),
            Value::AsyncMutex(_) => Some(crate::method_dispatch::TypeTag::AsyncMutexValue),
            // Static namespace sentinels: Value::Builtin("__ns__Json") etc.
            Value::Builtin(name) if name.starts_with("__ns__") => {
                crate::method_dispatch::namespace_type_tag(&name["__ns__".len()..])
            }
            _ => None,
        };
        let type_tag = member.type_tag.get().or(dynamic_tag);
        if let Some(type_tag) = type_tag {
            // For HashMap targets, check if the member is a callable field first.
            // This handles namespace imports (`import * as ns`) which are stored as
            // Value::HashMap but whose fields may be user-defined functions — not stdlib methods.
            if matches!(type_tag, crate::method_dispatch::TypeTag::HashMap) && member.args.is_some()
            {
                let key = crate::stdlib::collections::hash::HashKey::String(Arc::new(
                    member.member.name.clone(),
                ));
                if let Value::HashMap(ref map) = target_value {
                    if let Some(field_val) = map.get(&key).cloned() {
                        if matches!(
                            field_val,
                            Value::Function(_) | Value::Closure(_) | Value::NativeFunction(_)
                        ) {
                            let mut args = Vec::new();
                            if let Some(method_args) = &member.args {
                                for arg in method_args {
                                    args.push(self.eval_expr(arg)?);
                                }
                            }
                            return self.invoke_callee(field_val, args, member.span, None);
                        }
                    }
                }
            }

            let func_name = crate::method_dispatch::resolve_method(type_tag, &member.member.name)
                .ok_or_else(|| RuntimeError::TypeError {
                msg: format!("No method '{}' on type {:?}", member.member.name, type_tag),
                span: member.span,
            })?;

            // 3. Build argument list.
            // For static namespaces (Json/Math/Env), target is a sentinel — not a receiver.
            // For instance methods, target is arg[0].
            let is_ns = matches!(
                type_tag,
                crate::method_dispatch::TypeTag::JsonNs
                    | crate::method_dispatch::TypeTag::MathNs
                    | crate::method_dispatch::TypeTag::EnvNs
                    | crate::method_dispatch::TypeTag::FileNs
                    | crate::method_dispatch::TypeTag::ProcessNs
                    | crate::method_dispatch::TypeTag::DateTimeNs
                    | crate::method_dispatch::TypeTag::PathNs
                    | crate::method_dispatch::TypeTag::HttpNs
                    | crate::method_dispatch::TypeTag::NetNs
                    | crate::method_dispatch::TypeTag::CryptoNs
                    | crate::method_dispatch::TypeTag::RegexNs
                    | crate::method_dispatch::TypeTag::IoNs
                    | crate::method_dispatch::TypeTag::GzipNs
                    | crate::method_dispatch::TypeTag::TarNs
                    | crate::method_dispatch::TypeTag::ZipNs
                    | crate::method_dispatch::TypeTag::TaskNs
                    | crate::method_dispatch::TypeTag::SyncNs
                    | crate::method_dispatch::TypeTag::FutureNs
                    | crate::method_dispatch::TypeTag::TestNs
            );
            let mut args = if is_ns {
                Vec::new()
            } else {
                vec![target_value]
            };
            if let Some(method_args) = &member.args {
                for arg in method_args {
                    args.push(self.eval_expr(arg)?);
                }
            }

            // 4a. Callback-based intrinsics (map, filter, reduce, etc.) must go through
            //     invoke_callee so the interpreter can execute the callback.
            if crate::method_dispatch::is_callback_intrinsic(&func_name) {
                return self.invoke_callee(
                    Value::Builtin(func_name.into()),
                    args,
                    member.span,
                    None,
                );
            }

            // 4. Call stdlib function
            let security =
                self.current_security
                    .as_ref()
                    .ok_or_else(|| RuntimeError::InternalError {
                        msg: "Security context not set".to_string(),
                        span: member.span,
                    })?;
            let result = crate::stdlib::call_builtin(
                &func_name,
                &args,
                member.span,
                security,
                &self.output_writer,
            )?;

            // 5. CoW write-back: if the method mutates the receiver, update the receiver variable.
            //    Only possible when the target is a simple identifier (not a complex expression).
            if let Expr::Identifier(id) = member.target.as_ref() {
                if crate::method_dispatch::is_array_mutating_collection(&func_name) {
                    // Push/unshift/reverse: result IS the new array — write it back
                    self.force_set_collection(&id.name, result.clone());
                    return Ok(result);
                }
                if crate::method_dispatch::is_array_mutating_pair(&func_name) {
                    // Pop/shift: result is [extracted_value, new_array] — write back new_array, return extracted
                    if let Value::Array(ref arr) = result {
                        let s = arr.as_slice();
                        if s.len() == 2 {
                            let extracted = s[0].clone();
                            let new_arr = s[1].clone();
                            self.force_set_collection(&id.name, new_arr);
                            return Ok(extracted);
                        }
                    }
                    return Ok(result);
                }
                if crate::method_dispatch::is_collection_mutating_simple(&func_name) {
                    // HashMap.put/clear, HashSet.add/remove/clear, Queue.enqueue/clear,
                    // Stack.push/clear: result IS the new collection — write it back
                    self.force_set_collection(&id.name, result.clone());
                    return Ok(result);
                }
                if crate::method_dispatch::is_collection_mutating_pair(&func_name) {
                    // HashMap.remove, Queue.dequeue, Stack.pop:
                    // result is [extracted_value, new_collection] — write back collection, return extracted
                    if let Value::Array(ref arr) = result {
                        let s = arr.as_slice();
                        if s.len() == 2 {
                            let extracted = s[0].clone();
                            let new_col = s[1].clone();
                            self.force_set_collection(&id.name, new_col);
                            return Ok(extracted);
                        }
                    }
                    return Ok(result);
                }
            }

            Ok(result)
        } else {
            let callee = Self::get_member_from_value(target_value, &member.member, member.span)?;
            let mut args = Vec::new();
            if let Some(method_args) = &member.args {
                for arg in method_args {
                    args.push(self.eval_expr(arg)?);
                }
            }
            self.invoke_callee(callee, args, member.span, member.args.as_deref())
        }
    }

    /// Evaluate try expression (error propagation operator ?)
    ///
    /// Unwraps Ok value or returns Err early from current function
    pub(super) fn eval_try(&mut self, try_expr: &TryExpr) -> Result<Value, RuntimeError> {
        let value = self.eval_expr(&try_expr.expr)?;

        match value {
            Value::Result(Ok(inner)) => {
                // Unwrap Ok value
                Ok(*inner)
            }
            Value::Result(Err(err)) => {
                // Propagate error by early return
                let err_result = Value::Result(Err(err));
                self.control_flow = ControlFlow::Return(err_result.clone());
                Ok(err_result)
            }
            Value::Option(Some(inner)) => {
                // Unwrap Some value
                Ok(*inner)
            }
            Value::Option(None) => {
                // Propagate None by early return
                let none_val = Value::Option(None);
                self.control_flow = ControlFlow::Return(none_val.clone());
                Ok(none_val)
            }
            _ => {
                // Type checker should prevent this, but handle gracefully
                Err(RuntimeError::TypeError {
                    msg: "? operator requires Result<T, E> or Option<T> type".to_string(),
                    span: try_expr.span,
                })
            }
        }
    }

    /// Call a user-defined function
    pub(super) fn call_user_function(
        &mut self,
        func: &UserFunction,
        args: Vec<Value>,
        call_span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        // Check arity
        if args.len() != func.params.len() {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Function {} expects {} arguments, got {}",
                    func.name,
                    func.params.len(),
                    args.len()
                ),
                span: call_span,
            });
        }

        self.push_call_frame(func.name.clone(), call_span);

        // Push new scope for function
        self.push_scope();

        // Inject captured outer-scope snapshots (anonymous functions only).
        // These shadow live scope so that outer `var` mutations after closure
        // creation are invisible inside, matching VM snapshot semantics.
        // Parameters bound below will shadow any same-named captured value.
        if !func.captured.is_empty() {
            let scope = self
                .locals
                .last_mut()
                .ok_or_else(|| RuntimeError::InternalError {
                    msg: "Missing scope for variable declaration".to_string(),
                    span: call_span,
                })?;
            for (var_name, value) in &func.captured {
                scope.insert(var_name.clone(), (value.clone(), true));
            }
        }

        // Bind parameters (parameters are mutable)
        for (param, arg) in func.params.iter().zip(args.iter()) {
            // Debug-mode ownership enforcement for `share` parameters.
            #[cfg(debug_assertions)]
            {
                use crate::ast::OwnershipAnnotation;
                match &param.ownership {
                    Some(OwnershipAnnotation::Share) => {
                        if !matches!(arg, Value::SharedValue(_)) {
                            // Must pop scope before returning — we already pushed it.
                            self.pop_scope();
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "ownership violation: parameter '{}' expects share<T> but received {}",
                                    param.name.name,
                                    arg.type_name()
                                ),
                                span: call_span,
                            });
                        }
                    }
                    Some(ann @ OwnershipAnnotation::Own)
                    | Some(ann @ OwnershipAnnotation::Borrow) => {
                        if matches!(arg, Value::SharedValue(_)) {
                            let ann_str = match ann {
                                OwnershipAnnotation::Own => "own",
                                OwnershipAnnotation::Borrow => "borrow",
                                OwnershipAnnotation::Share => unreachable!(),
                            };
                            self.runtime_warnings.push(
                                crate::diagnostic::error_codes::SHARE_PASSED_TO_NON_SHARE
                                    .emit(param.name.span)
                                    .arg("inner", "T")
                                    .arg("annotation", ann_str)
                                    .arg("name", &param.name.name)
                                    .build(),
                            );
                        }
                    }
                    None => {}
                }
            }
            let scope = self
                .locals
                .last_mut()
                .ok_or_else(|| RuntimeError::InternalError {
                    msg: "Missing scope for assignment".to_string(),
                    span: call_span,
                })?;
            scope.insert(param.name.name.clone(), (arg.clone(), true));
        }

        // Execute function body
        let mut result = Value::Null;
        let mut had_explicit_return = false;
        for stmt in &func.body.statements {
            result = self.eval_statement(stmt)?;

            // Check for return
            if let ControlFlow::Return(val) = &self.control_flow {
                result = val.clone();
                self.control_flow = ControlFlow::None;
                had_explicit_return = true;
                break;
            }
        }

        // Evaluate tail expression if present (implicit return)
        // Skip if we already had an explicit return statement
        if !had_explicit_return {
            if let Some(tail) = &func.body.tail_expr {
                result = self.eval_expr(tail)?;
                // H-178: if the tail expr (e.g. a match) internally executed a `return`
                // statement, ControlFlow::Return carries the real value — use it.
                if let ControlFlow::Return(val) = &self.control_flow {
                    result = val.clone();
                    self.control_flow = ControlFlow::None;
                }
            }
        }

        self.pop_scope();
        self.pop_call_frame();
        Ok(result)
    }

    /// Evaluate array indexing
    fn eval_index(&mut self, index: &IndexExpr) -> Result<Value, RuntimeError> {
        let target = self.eval_expr(&index.target)?;
        match &index.index {
            IndexValue::Single(expr) => {
                let idx = self.eval_expr(expr)?;
                match target {
                    Value::Array(arr) => match idx {
                        Value::Number(n) => {
                            let index_val = n as i64;
                            if n.fract() != 0.0 || n < 0.0 {
                                return Err(RuntimeError::InvalidIndex { span: index.span });
                            }

                            if index_val >= 0 && (index_val as usize) < arr.len() {
                                Ok(arr[index_val as usize].clone())
                            } else {
                                Err(RuntimeError::OutOfBounds { span: index.span })
                            }
                        }
                        Value::Range {
                            start,
                            end,
                            inclusive,
                        } => {
                            let start = start.unwrap_or(0.0);
                            let mut end_val = end.unwrap_or(arr.len() as f64);
                            if inclusive && end.is_some() {
                                end_val += 1.0;
                            }
                            crate::stdlib::array::slice(arr.as_slice(), start, end_val, index.span)
                        }
                        _ => Err(RuntimeError::InvalidIndex { span: index.span }),
                    },
                    Value::String(s) => {
                        if let Value::Number(n) = idx {
                            let index_val = n as i64;
                            if n.fract() != 0.0 || n < 0.0 {
                                return Err(RuntimeError::InvalidIndex { span: index.span });
                            }

                            let chars: Vec<char> = s.chars().collect();
                            if index_val >= 0 && (index_val as usize) < chars.len() {
                                Ok(Value::string(chars[index_val as usize].to_string()))
                            } else {
                                Err(RuntimeError::OutOfBounds { span: index.span })
                            }
                        } else {
                            Err(RuntimeError::InvalidIndex { span: index.span })
                        }
                    }
                    Value::JsonValue(json) => {
                        // JSON indexing with string or number, returns JsonValue
                        let result = match idx {
                            Value::String(key) => json.index_str(key.as_ref()),
                            Value::Number(n) => json.index_num(n),
                            _ => {
                                return Err(RuntimeError::TypeError {
                                    msg: "JSON index must be string or number".to_string(),
                                    span: index.span,
                                })
                            }
                        };
                        Ok(Value::JsonValue(Arc::new(result)))
                    }
                    _ => Err(RuntimeError::TypeError {
                        msg: "Cannot index non-array/string/json".to_string(),
                        span: index.span,
                    }),
                }
            }
        }
    }

    /// Evaluate array literal
    fn eval_array_literal(
        &mut self,
        arr: &crate::ast::ArrayLiteral,
    ) -> Result<Value, RuntimeError> {
        let elements: Result<Vec<Value>, _> =
            arr.elements.iter().map(|e| self.eval_expr(e)).collect();
        let elements = elements?;

        // Track memory allocation before creating the array
        let estimated_size = Self::estimate_array_size(&elements);
        self.track_memory(estimated_size)?;

        Ok(Value::array(elements))
    }

    /// Evaluate object literal: `record { key: value, key2: value2 }`
    fn eval_object_literal(
        &mut self,
        obj: &crate::ast::ObjectLiteral,
    ) -> Result<Value, RuntimeError> {
        use crate::stdlib::collections::hash::HashKey;
        use crate::stdlib::collections::hashmap::AtlasHashMap;
        use crate::value::ValueHashMap;

        let mut atlas_map = AtlasHashMap::new();

        for entry in &obj.entries {
            // Key is always a string (from identifier)
            let key = HashKey::String(Arc::new(entry.key.name.clone()));
            // Evaluate the value expression
            let value = self.eval_expr(&entry.value)?;
            atlas_map.insert(key, value);
        }

        Ok(Value::HashMap(ValueHashMap::from_atlas(atlas_map)))
    }

    /// Evaluate match expression
    fn eval_match(&mut self, match_expr: &crate::ast::MatchExpr) -> Result<Value, RuntimeError> {
        // Evaluate scrutinee
        let scrutinee = self.eval_expr(&match_expr.scrutinee)?;

        // Try each arm in order
        for arm in &match_expr.arms {
            // Try to match pattern against scrutinee
            if let Some(bindings) = self.try_match_pattern(&arm.pattern, &scrutinee) {
                // Pattern matched! Create new scope and bind variables
                self.push_scope();

                // Bind pattern variables (pattern bindings are immutable - they're destructured values)
                for (name, value) in &bindings {
                    let scope =
                        self.locals
                            .last_mut()
                            .ok_or_else(|| RuntimeError::InternalError {
                                msg: "Missing scope for assignment".to_string(),
                                span: arm.span,
                            })?;
                    scope.insert(name.clone(), (value.clone(), false));
                }

                // Check guard if present — guard failure means try next arm
                if let Some(guard_expr) = &arm.guard {
                    let guard_result = self.eval_expr(guard_expr)?;
                    if guard_result != Value::Bool(true) {
                        self.pop_scope();
                        continue; // Guard failed — try next arm
                    }
                }

                // Evaluate arm body with bindings in scope
                let result = self.eval_expr(&arm.body)?;

                // Pop scope (remove bindings)
                self.pop_scope();

                // Return result
                return Ok(result);
            }
        }

        // No pattern matched - this should be prevented by exhaustiveness checking
        // but provide a fallback error just in case
        Err(RuntimeError::TypeError {
            msg: "Non-exhaustive pattern match - no arm matched".to_string(),
            span: match_expr.span,
        })
    }

    /// Try to match a pattern against a value
    /// Returns Some(bindings) if match succeeds, None if match fails
    fn try_match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
        match pattern {
            // Literal patterns: must match exactly
            Pattern::Literal(lit, _) => {
                let pattern_value = self.eval_literal(lit);
                if self.values_equal(&pattern_value, value) {
                    Some(Vec::new()) // Match, no bindings
                } else {
                    None // No match
                }
            }

            // Wildcard: matches anything, no bindings
            Pattern::Wildcard(_) => Some(Vec::new()),

            // Variable: matches anything, binds to name
            Pattern::Variable(id) => Some(vec![(id.name.clone(), value.clone())]),

            // Constructor patterns: Some(x), None, Ok(x), Err(e)
            Pattern::Constructor { name, args, .. } => {
                self.try_match_constructor(name, args, value)
            }

            // Array patterns: [x, y, z]
            Pattern::Array { elements, .. } => self.try_match_array(elements, value),

            // Tuple patterns: (x, y, z)
            Pattern::Tuple { elements, .. } => {
                if let Value::Tuple(elems) = value {
                    if elements.len() != elems.len() {
                        return None;
                    }
                    let mut all_bindings = Vec::new();
                    for (pat, val) in elements.iter().zip(elems.iter()) {
                        if let Some(bindings) = self.try_match_pattern(pat, val) {
                            all_bindings.extend(bindings);
                        } else {
                            return None;
                        }
                    }
                    Some(all_bindings)
                } else {
                    None
                }
            }

            // OR patterns: try each sub-pattern, return first match
            Pattern::Or(alternatives, _) => {
                for alt in alternatives {
                    if let Some(bindings) = self.try_match_pattern(alt, value) {
                        return Some(bindings);
                    }
                }
                None
            }

            // Enum variant patterns: State::Running, Color::Rgb(r, g, b)
            Pattern::EnumVariant {
                enum_name,
                variant_name,
                args,
                ..
            } => {
                // Must match an EnumValue with same enum_name and variant_name
                if let Value::EnumValue {
                    enum_name: val_enum,
                    variant_name: val_variant,
                    data,
                } = value
                {
                    // Check enum and variant names match
                    if enum_name.name != *val_enum || variant_name.name != *val_variant {
                        return None;
                    }

                    // Check argument count matches
                    if args.len() != data.len() {
                        return None;
                    }

                    // Recursively match arguments
                    let mut all_bindings = Vec::new();
                    for (pattern, val) in args.iter().zip(data.iter()) {
                        if let Some(bindings) = self.try_match_pattern(pattern, val) {
                            all_bindings.extend(bindings);
                        } else {
                            return None;
                        }
                    }
                    Some(all_bindings)
                } else {
                    None
                }
            }

            // Bare variant patterns: Running, Pending(msg) — infer enum from value.
            // Only the variant_name is checked; the enum_name is not required at the call site.
            Pattern::BareVariant { name, args, .. } => {
                if let Value::EnumValue {
                    variant_name: val_variant,
                    data,
                    ..
                } = value
                {
                    if name.name != *val_variant {
                        return None;
                    }
                    if args.len() != data.len() {
                        return None;
                    }
                    let mut all_bindings = Vec::new();
                    for (pattern, val) in args.iter().zip(data.iter()) {
                        if let Some(bindings) = self.try_match_pattern(pattern, val) {
                            all_bindings.extend(bindings);
                        } else {
                            return None;
                        }
                    }
                    Some(all_bindings)
                } else {
                    None
                }
            }
        }
    }

    /// Try to match constructor pattern
    fn try_match_constructor(
        &self,
        name: &crate::ast::Identifier,
        args: &[Pattern],
        value: &Value,
    ) -> Option<Vec<(String, Value)>> {
        match name.name.as_str() {
            "Some" => {
                // Match Option::Some
                if let Value::Option(Some(inner)) = value {
                    if args.len() != 1 {
                        return None; // Type checker should prevent this
                    }
                    self.try_match_pattern(&args[0], inner)
                } else {
                    None
                }
            }
            "None" => {
                // Match Option::None
                if let Value::Option(None) = value {
                    if args.is_empty() {
                        Some(Vec::new())
                    } else {
                        None // Type checker should prevent this
                    }
                } else {
                    None
                }
            }
            "Ok" => {
                // Match Result::Ok
                if let Value::Result(Ok(inner)) = value {
                    if args.len() != 1 {
                        return None; // Type checker should prevent this
                    }
                    self.try_match_pattern(&args[0], inner)
                } else {
                    None
                }
            }
            "Err" => {
                // Match Result::Err
                if let Value::Result(Err(inner)) = value {
                    if args.len() != 1 {
                        return None; // Type checker should prevent this
                    }
                    self.try_match_pattern(&args[0], inner)
                } else {
                    None
                }
            }
            _ => None, // Unknown constructor
        }
    }

    /// Try to match array pattern
    fn try_match_array(
        &self,
        pattern_elements: &[Pattern],
        value: &Value,
    ) -> Option<Vec<(String, Value)>> {
        if let Value::Array(arr) = value {
            // Array patterns must have exact length match
            if arr.len() != pattern_elements.len() {
                return None;
            }

            let mut all_bindings = Vec::new();

            // Match each element
            for (pattern, element) in pattern_elements.iter().zip(arr.iter()) {
                if let Some(bindings) = self.try_match_pattern(pattern, element) {
                    all_bindings.extend(bindings);
                } else {
                    return None; // One element didn't match
                }
            }

            Some(all_bindings)
        } else {
            None // Not an array
        }
    }

    /// Check if two values are equal (for pattern matching)
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    // ========================================================================
    // Array Intrinsics (Callback-based operations)
    // ========================================================================

    /// map(array, callback) - Transform each element
    fn intrinsic_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "map() expects 2 arguments (array, callback)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::with_capacity(arr.len());
        for elem in arr {
            // Call callback with element
            let callback_result = self.call_value(callback, vec![elem], span)?;
            result.push(callback_result);
        }

        Ok(Value::array(result))
    }

    /// filter(array, predicate) - Keep elements matching predicate
    fn intrinsic_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "filter() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();
        for elem in arr {
            let pred_result = self.call_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => result.push(elem),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "filter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::array(result))
    }

    /// reduce(array, reducer, initial) - Accumulate to single value
    fn intrinsic_reduce(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "reduce() expects 3 arguments (array, reducer, initial)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let reducer = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut accumulator = args[2].clone();
        for elem in arr {
            accumulator = self.call_value(reducer, vec![accumulator, elem], span)?;
        }

        Ok(accumulator)
    }

    /// forEach(array, callback) - Execute callback for each element
    fn intrinsic_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "forEach() expects 2 arguments (array, callback)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for elem in arr {
            self.call_value(callback, vec![elem], span)?;
        }

        Ok(Value::Null)
    }

    /// find(array, predicate) - Find first matching element
    fn intrinsic_find(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "find() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for elem in arr {
            let pred_result = self.call_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(Value::Option(Some(Box::new(elem)))),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "find() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Option(None))
    }

    /// findIndex(array, predicate) - Find index of first matching element
    fn intrinsic_find_index(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "findIndex() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for (i, elem) in arr.iter().enumerate() {
            let pred_result = self.call_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => {
                    return Ok(Value::Option(Some(Box::new(Value::Number(i as f64)))))
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "findIndex() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Option(None))
    }

    /// flatMap(array, callback) - Map and flatten one level
    fn intrinsic_flat_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "flatMap() expects 2 arguments (array, callback)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();
        for elem in arr {
            let callback_result = self.call_value(callback, vec![elem], span)?;
            match callback_result {
                Value::Array(nested) => {
                    result.extend(nested.iter().cloned());
                }
                other => result.push(other),
            }
        }

        Ok(Value::array(result))
    }

    /// some(array, predicate) - Check if any element matches
    fn intrinsic_some(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "some() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for elem in arr {
            let pred_result = self.call_value(predicate, vec![elem], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(Value::Bool(true)),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "some() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Bool(false))
    }

    /// every(array, predicate) - Check if all elements match
    fn intrinsic_every(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "every() expects 2 arguments (array, predicate)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for elem in arr {
            let pred_result = self.call_value(predicate, vec![elem], span)?;
            match pred_result {
                Value::Bool(false) => return Ok(Value::Bool(false)),
                Value::Bool(true) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "every() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Bool(true))
    }

    /// sort(array, comparator) - Sort with custom comparator
    ///
    /// Uses insertion sort for stability and simplicity with callbacks
    fn intrinsic_sort(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "sort() expects 2 arguments (array, comparator)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let comparator = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() second argument must be function".to_string(),
                    span,
                })
            }
        };

        // Simple insertion sort (stable) with callback comparisons
        let mut sorted = arr;
        for i in 1..sorted.len() {
            let mut j = i;
            while j > 0 {
                let cmp_result = self.call_value(
                    comparator,
                    vec![sorted[j].clone(), sorted[j - 1].clone()],
                    span,
                )?;
                match cmp_result {
                    Value::Number(n) if n < 0.0 => {
                        sorted.swap(j, j - 1);
                        j -= 1;
                    }
                    Value::Number(_) => break,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "sort() comparator must return number".to_string(),
                            span,
                        })
                    }
                }
            }
        }

        Ok(Value::array(sorted))
    }

    /// sortBy(array, keyExtractor) - Sort by extracted key
    fn intrinsic_sort_by(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "sortBy() expects 2 arguments (array, keyExtractor)".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let key_extractor = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() second argument must be function".to_string(),
                    span,
                })
            }
        };

        // Extract keys first
        let mut keyed: Vec<(Value, Value)> = Vec::new();
        for elem in arr {
            let key = self.call_value(key_extractor, vec![elem.clone()], span)?;
            keyed.push((key, elem));
        }

        // Sort by keys (stable)
        keyed.sort_by(|(key_a, _), (key_b, _)| match (key_a, key_b) {
            (Value::Number(a), Value::Number(b)) => {
                if a < b {
                    std::cmp::Ordering::Less
                } else if a > b {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            }
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        });

        // Extract sorted elements
        let sorted: Vec<Value> = keyed.into_iter().map(|(_, elem)| elem).collect();
        Ok(Value::array(sorted))
    }

    // ========================================================================
    // Result Intrinsics (Callback-based operations)
    // ========================================================================

    /// result_map(result, transform_fn) - Transform Ok value
    fn intrinsic_result_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_map() expects 2 arguments (result, transform_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let transform_fn = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_map() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => {
                let transformed = self.call_value(transform_fn, vec![(**val).clone()], span)?;
                Ok(Value::Result(Ok(Box::new(transformed))))
            }
            Value::Result(Err(err)) => Ok(Value::Result(Err(err.clone()))),
            _ => Err(RuntimeError::TypeError {
                msg: "result_map() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    /// result_map_err(result, transform_fn) - Transform Err value
    fn intrinsic_result_map_err(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_map_err() expects 2 arguments (result, transform_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let transform_fn = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_map_err() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => Ok(Value::Result(Ok(val.clone()))),
            Value::Result(Err(err)) => {
                let transformed = self.call_value(transform_fn, vec![(**err).clone()], span)?;
                Ok(Value::Result(Err(Box::new(transformed))))
            }
            _ => Err(RuntimeError::TypeError {
                msg: "result_map_err() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    /// result_and_then(result, next_fn) - Chain Results (monadic bind)
    fn intrinsic_result_and_then(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_and_then() expects 2 arguments (result, next_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let next_fn = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_and_then() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => {
                // Call next_fn which should return a Result
                self.call_value(next_fn, vec![(**val).clone()], span)
            }
            Value::Result(Err(err)) => Ok(Value::Result(Err(err.clone()))),
            _ => Err(RuntimeError::TypeError {
                msg: "result_and_then() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    /// result_or_else(result, recovery_fn) - Recover from Err
    fn intrinsic_result_or_else(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_or_else() expects 2 arguments (result, recovery_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let recovery_fn = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_or_else() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => Ok(Value::Result(Ok(val.clone()))),
            Value::Result(Err(err)) => {
                // Call recovery_fn which should return a Result
                self.call_value(recovery_fn, vec![(**err).clone()], span)
            }
            _ => Err(RuntimeError::TypeError {
                msg: "result_or_else() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    /// hashMapForEach(map, callback) - Iterate over map entries with side effects
    fn intrinsic_hashmap_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapForEach() expects 2 arguments (map, callback)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapForEach() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapForEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for (key, value) in map {
            // Call callback with (value, key) arguments
            self.call_value(callback, vec![value, key.to_value()], span)?;
        }

        Ok(Value::Null)
    }

    /// hashMapMap(map, callback) - Transform values, return new map
    fn intrinsic_hashmap_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapMap() expects 2 arguments (map, callback)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapMap() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = crate::stdlib::collections::hashmap::AtlasHashMap::new();
        for (key, value) in map {
            // Call callback with (value, key) arguments
            let new_value = self.call_value(callback, vec![value, key.clone().to_value()], span)?;
            result.insert(key, new_value);
        }

        Ok(Value::HashMap(crate::value::ValueHashMap::from_atlas(
            result,
        )))
    }

    /// hashMapFilter(map, predicate) - Filter entries, return new map
    fn intrinsic_hashmap_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapFilter() expects 2 arguments (map, predicate)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::HashMap(m) => m.entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapFilter() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapFilter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = crate::stdlib::collections::hashmap::AtlasHashMap::new();
        for (key, value) in map {
            // Call predicate with (value, key) arguments
            let pred_result =
                self.call_value(predicate, vec![value.clone(), key.clone().to_value()], span)?;
            match pred_result {
                Value::Bool(true) => {
                    result.insert(key, value);
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "hashMapFilter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::HashMap(crate::value::ValueHashMap::from_atlas(
            result,
        )))
    }

    /// hashSetForEach(set, callback) - Iterate over set elements with side effects
    fn intrinsic_hashset_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetForEach() expects 2 arguments (set, callback)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetForEach() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetForEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for element in set {
            // Call callback with element argument
            self.call_value(callback, vec![element.to_value()], span)?;
        }

        Ok(Value::Null)
    }

    /// hashSetMap(set, callback) - Transform elements to array
    fn intrinsic_hashset_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetMap() expects 2 arguments (set, callback)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetMap() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();
        for element in set {
            // Call callback with element argument
            let mapped_value = self.call_value(callback, vec![element.to_value()], span)?;
            result.push(mapped_value);
        }

        Ok(Value::array(result))
    }

    /// hashSetFilter(set, predicate) - Filter elements, return new set
    fn intrinsic_hashset_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetFilter() expects 2 arguments (set, predicate)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::HashSet(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetFilter() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetFilter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_set = crate::value::ValueHashSet::new();
        for element in set {
            // Call predicate with element argument
            let pred_result = self.call_value(predicate, vec![element.clone().to_value()], span)?;
            match pred_result {
                Value::Bool(true) => {
                    result_set.inner_mut().insert(element);
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "hashSetFilter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::HashSet(result_set))
    }

    /// Regex intrinsic: Replace first match using callback
    fn intrinsic_regex_replace_with(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "regexReplaceWith() expects 3 arguments (regex, text, callback)".to_string(),
                span,
            });
        }

        let regex = match &args[0] {
            Value::Regex(r) => r.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() first argument must be Regex".to_string(),
                    span,
                })
            }
        };

        let text = match &args[1] {
            Value::String(s) => s.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() second argument must be string".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[2] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[2],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() third argument must be function".to_string(),
                    span,
                })
            }
        };

        // Find first match
        if let Some(mat) = regex.find(text) {
            let match_start = mat.start();
            let match_end = mat.end();
            let match_text = mat.as_str();

            // Build match data HashMap
            let mut match_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "text".to_string(),
                )),
                Value::string(match_text),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "start".to_string(),
                )),
                Value::Number(match_start as f64),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "end".to_string(),
                )),
                Value::Number(match_end as f64),
            );

            // Extract capture groups
            if let Some(caps) = regex.captures(text) {
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(group) = caps.get(i) {
                        groups.push(Value::string(group.as_str()));
                    } else {
                        groups.push(Value::Null);
                    }
                }
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(groups),
                );
            } else {
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(vec![]),
                );
            }

            let match_value = Value::HashMap(crate::value::ValueHashMap::from_atlas(match_map));

            // Call callback with match data
            let replacement_value = self.call_value(callback, vec![match_value], span)?;

            // Expect string return value and clone to avoid lifetime issues
            let replacement_str = match &replacement_value {
                Value::String(s) => s.as_ref().to_string(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "regexReplaceWith() callback must return string".to_string(),
                        span,
                    })
                }
            };

            // Build result string
            let mut result = String::with_capacity(text.len());
            result.push_str(&text[..match_start]);
            result.push_str(&replacement_str);
            result.push_str(&text[match_end..]);

            Ok(Value::string(result))
        } else {
            // No match, return original text
            Ok(Value::string(text))
        }
    }

    /// Regex intrinsic: Replace all matches using callback
    fn intrinsic_regex_replace_all_with(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "regexReplaceAllWith() expects 3 arguments (regex, text, callback)"
                    .to_string(),
                span,
            });
        }

        let regex = match &args[0] {
            Value::Regex(r) => r.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() first argument must be Regex".to_string(),
                    span,
                })
            }
        };

        let text = match &args[1] {
            Value::String(s) => s.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() second argument must be string".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[2] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[2],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() third argument must be function".to_string(),
                    span,
                })
            }
        };

        // Find all matches and collect them
        let matches: Vec<_> = regex.find_iter(text).collect();

        if matches.is_empty() {
            return Ok(Value::string(text));
        }

        // Build result string by processing all matches
        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;

        for mat in matches {
            let match_start = mat.start();
            let match_end = mat.end();
            let match_text = mat.as_str();

            // Build match data HashMap
            let mut match_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "text".to_string(),
                )),
                Value::string(match_text),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "start".to_string(),
                )),
                Value::Number(match_start as f64),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "end".to_string(),
                )),
                Value::Number(match_end as f64),
            );

            // Extract capture groups
            if let Some(caps) = regex.captures(mat.as_str()) {
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(group) = caps.get(i) {
                        groups.push(Value::string(group.as_str()));
                    } else {
                        groups.push(Value::Null);
                    }
                }
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(groups),
                );
            } else {
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(vec![]),
                );
            }

            let match_value = Value::HashMap(crate::value::ValueHashMap::from_atlas(match_map));

            // Call callback with match data
            let replacement_value = self.call_value(callback, vec![match_value], span)?;

            // Expect string return value and clone to avoid lifetime issues
            let replacement_str = match &replacement_value {
                Value::String(s) => s.as_ref().to_string(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "regexReplaceAllWith() callback must return string".to_string(),
                        span,
                    })
                }
            };

            // Add text before this match
            result.push_str(&text[last_end..match_start]);
            // Add replacement
            result.push_str(&replacement_str);

            last_end = match_end;
        }

        // Add remaining text after last match
        result.push_str(&text[last_end..]);

        Ok(Value::string(result))
    }

    // ========================================================================
    // Test Intrinsics (Callable assertions)
    // ========================================================================

    fn intrinsic_assert_throws(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        crate::stdlib::test::assert_throws_with(args, span, true, |callable| {
            self.call_value(callable, vec![], span)
        })
    }

    fn intrinsic_assert_no_throw(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        crate::stdlib::test::assert_no_throw_with(args, span, true, |callable| {
            self.call_value(callable, vec![], span)
        })
    }

    /// Helper: Call a function value with arguments
    fn call_value(
        &mut self,
        func: &Value,
        args: Vec<Value>,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Builtin(name) => {
                let security =
                    self.current_security
                        .as_ref()
                        .ok_or_else(|| RuntimeError::InternalError {
                            msg: "Security context not set".to_string(),
                            span,
                        })?;
                crate::stdlib::call_builtin(name, &args, span, security, &self.output_writer)
            }
            Value::Function(func_ref) => {
                // User-defined function
                if let Some(user_func) = self.function_bodies.get(&func_ref.name).cloned() {
                    return self.call_user_function(&user_func, args, span);
                }

                Err(RuntimeError::UnknownFunction {
                    name: func_ref.name.clone(),
                    span,
                })
            }
            Value::NativeFunction(native_fn) => native_fn(&args),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected function value".to_string(),
                span,
            }),
        }
    }

    /// Apply CoW write-back for collection mutation builtins.
    ///
    /// When a builtin mutates a collection by returning a new value, we write the
    /// new value back to the first argument variable (if it's an identifier).
    ///
    /// - "returns new collection" builtins: write `result` back to arg[0]
    /// - "returns [extracted, new collection]" builtins: write `result[1]` back to arg[0]
    fn apply_cow_writeback(
        &mut self,
        name: &str,
        result: Value,
        call_args: &[Expr],
        _span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        // Builtins that return the modified collection directly
        const RETURNS_COLLECTION: &[&str] = &[
            // HashMap
            "hashMapPut",
            "hash_map_put",
            "hashMapClear",
            "hash_map_clear",
            // HashSet
            "hashSetAdd",
            "hash_set_add",
            "hashSetRemove",
            "hash_set_remove",
            "hashSetClear",
            "hash_set_clear",
            // Queue
            "queueEnqueue",
            "queue_enqueue",
            "queueClear",
            "queue_clear",
            // Stack
            "stackPush",
            "stack_push",
            "stackClear",
            "stack_clear",
            // Array (free-function variants)
            "unshift",
            "reverse",
            "flatten",
        ];

        // Builtins that return [extracted_value, new_collection]
        const RETURNS_PAIR: &[&str] = &[
            // HashMap / HashSet / Queue / Stack
            "hashMapRemove",
            "hash_map_remove",
            "queueDequeue",
            "queue_dequeue",
            "stackPop",
            "stack_pop",
            // Array (free-function variants)
            "pop",
            "shift",
        ];

        // Identify first arg as an identifier (only then can we write back)
        let first_ident = call_args.first().and_then(|e| {
            if let Expr::Identifier(id) = e {
                Some(id.name.clone())
            } else {
                None
            }
        });

        if let Some(var_name) = first_ident {
            if RETURNS_COLLECTION.contains(&name) {
                // Bypass mutability check: this is container content mutation,
                // not a variable rebinding.
                self.force_set_collection(&var_name, result.clone());
                return Ok(result);
            }

            if RETURNS_PAIR.contains(&name) {
                // result is [extracted_value, new_collection].
                // Write new_collection back to variable, return only extracted_value.
                // Atlas-level: `let item = queueDequeue(q)` → item is Option, q is updated.
                if let Value::Array(ref arr) = result {
                    let s = arr.as_slice();
                    if s.len() == 2 {
                        let extracted = s[0].clone();
                        let new_col = s[1].clone();
                        self.force_set_collection(&var_name, new_col);
                        return Ok(extracted);
                    }
                }
                return Ok(result);
            }
        }

        Ok(result)
    }

    /// Evaluate an anonymous function expression.
    ///
    /// Evaluate an anonymous function expression.
    ///
    /// Registers the function body in `self.function_bodies` under a synthetic
    /// unique name and returns a `Value::Function` referencing that name.
    ///
    /// Outer-scope locals are snapshotted at creation time into `captured` so
    /// that `var` mutations after closure creation are not visible inside the
    /// closure body — matching VM snapshot semantics (Block 4 parity rule).
    fn eval_anon_fn(
        &mut self,
        params: &[Param],
        body: &Expr,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        use std::collections::HashMap;

        let name = format!("__anon_{}", self.next_func_id);
        self.next_func_id += 1;

        // Build a Block for call_user_function to execute.
        // Arrow form: expression becomes the tail_expr (implicit return).
        // Block form: use the block directly.
        let body_block = match body {
            Expr::Block(block) => block.clone(),
            _ => Block {
                statements: vec![],
                tail_expr: Some(Box::new(body.clone())),
                span,
            },
        };

        // Snapshot non-global local-scope variables at closure creation time.
        // This aligns the interpreter with VM capture-by-value semantics:
        // outer `var` mutations after closure creation are not visible inside.
        // Global bindings must remain live so mutations inside closures persist.
        let param_names: std::collections::HashSet<&str> =
            params.iter().map(|p| p.name.name.as_str()).collect();
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

        let user_func = UserFunction {
            name: name.clone(),
            params: params.to_vec(),
            body: body_block,
            captured,
        };
        self.function_bodies.insert(name.clone(), user_func);

        Ok(Value::Function(crate::value::FunctionRef {
            name,
            arity: params.len(),
            bytecode_offset: 0,
            local_count: 0,
            param_ownership: params.iter().map(|p| p.ownership.clone()).collect(),
            param_names: params.iter().map(|p| p.name.name.clone()).collect(),
            return_ownership: None,
            is_async: false,
        }))
    }
}
