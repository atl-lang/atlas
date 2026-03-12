//! Expression compilation

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

use crate::ast::*;
use crate::bytecode::Opcode;
use crate::compiler::{Compiler, Local, UpvalueCapture, UpvalueContext};
use crate::diagnostic::error_codes::INTERNAL_ERROR;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::value::Value;

impl Compiler {
    /// Compile an expression
    pub(super) fn compile_expr(&mut self, expr: &Expr) -> Result<(), Vec<Diagnostic>> {
        match expr {
            Expr::Literal(lit, span) => self.compile_literal(lit, *span),
            Expr::TemplateString { parts, span } => self.compile_template_string(parts, *span),
            Expr::Identifier(ident) => self.compile_identifier(ident),
            Expr::Binary(bin) => self.compile_binary(bin),
            Expr::Unary(un) => self.compile_unary(un),
            Expr::Group(group) => self.compile_expr(&group.expr),
            Expr::ArrayLiteral(arr) => self.compile_array_literal(arr),
            Expr::Index(index) => self.compile_index(index),
            Expr::Call(call) => self.compile_call(call),
            Expr::Match(match_expr) => self.compile_match(match_expr),
            Expr::Member(member) => self.compile_member(member),
            Expr::Try(try_expr) => self.compile_try(try_expr),
            Expr::AnonFn {
                params,
                return_type: _,
                body,
                span,
            } => self.compile_anon_fn(params, body, *span),
            Expr::Block(block) => self.compile_block_expr(block),
            Expr::ObjectLiteral(obj) => self.compile_object_literal(obj),
            Expr::StructExpr(struct_expr) => self.compile_struct_expr(struct_expr),
            Expr::Range {
                start,
                end,
                inclusive,
                span,
            } => self.compile_range(start, end, *inclusive, *span),
            Expr::EnumVariant(ev) => self.compile_enum_variant(ev),
            Expr::TupleLiteral { elements, span } => {
                for elem in elements {
                    self.compile_expr(elem)?;
                }
                self.bytecode.emit(Opcode::Tuple, *span);
                self.bytecode.emit_u16(elements.len() as u16);
                Ok(())
            }
            Expr::Await { expr, span } => {
                self.compile_expr(expr)?;
                self.bytecode.emit(Opcode::Await, *span);
                Ok(())
            }
        }
    }

    fn compile_template_string(
        &mut self,
        parts: &[TemplatePart],
        span: Span,
    ) -> Result<(), Vec<Diagnostic>> {
        if parts.is_empty() {
            let idx = self.bytecode.add_constant(Value::string(""));
            self.bytecode.emit(Opcode::Constant, span);
            self.bytecode.emit_u16(idx);
            return Ok(());
        }

        let mut first = true;
        for part in parts {
            match part {
                TemplatePart::Literal(text) => {
                    let idx = self.bytecode.add_constant(Value::string(text));
                    self.bytecode.emit(Opcode::Constant, span);
                    self.bytecode.emit_u16(idx);
                }
                TemplatePart::Expression(expr) => {
                    self.compile_expr(expr)?;
                    self.bytecode.emit(Opcode::ToString, span);
                }
            }

            if first {
                first = false;
            } else {
                self.bytecode.emit(Opcode::Add, span);
            }
        }

        Ok(())
    }

    fn compile_range(
        &mut self,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    ) -> Result<(), Vec<Diagnostic>> {
        if let Some(start) = start {
            self.compile_expr(start)?;
        } else {
            self.bytecode.emit(Opcode::Null, span);
        }

        if let Some(end) = end {
            self.compile_expr(end)?;
        } else {
            self.bytecode.emit(Opcode::Null, span);
        }

        self.bytecode.emit(Opcode::Range, span);
        self.bytecode.emit_u8(if inclusive { 1 } else { 0 });

        Ok(())
    }

    /// Compile a struct instantiation expression
    fn compile_struct_expr(&mut self, struct_expr: &StructExpr) -> Result<(), Vec<Diagnostic>> {
        // Push field values onto stack (interleaved with keys)
        for field in &struct_expr.fields {
            // Push field name as string constant
            let key_idx = self
                .bytecode
                .add_constant(Value::string(field.name.name.clone()));
            self.bytecode.emit(Opcode::Constant, field.name.span);
            self.bytecode.emit_u16(key_idx);

            // Compile and push field value
            self.compile_expr(&field.value)?;
        }

        // Emit Struct opcode with name + count
        let name_idx = self
            .bytecode
            .add_constant(Value::string(struct_expr.name.name.clone()));
        self.bytecode.emit(Opcode::Struct, struct_expr.span);
        self.bytecode.emit_u16(name_idx);
        self.bytecode.emit_u16(struct_expr.fields.len() as u16);

        Ok(())
    }

    /// Compile an enum variant expression
    fn compile_enum_variant(
        &mut self,
        ev: &crate::ast::EnumVariantExpr,
    ) -> Result<(), Vec<Diagnostic>> {
        // Push enum name and variant name as string constants
        let enum_name_idx = self
            .bytecode
            .add_constant(Value::string(ev.enum_name.name.clone()));
        let variant_name_idx = self
            .bytecode
            .add_constant(Value::string(ev.variant_name.name.clone()));

        // Push the names
        self.bytecode.emit(Opcode::Constant, ev.enum_name.span);
        self.bytecode.emit_u16(enum_name_idx);
        self.bytecode.emit(Opcode::Constant, ev.variant_name.span);
        self.bytecode.emit_u16(variant_name_idx);

        // Compile and push any arguments
        let arg_count = if let Some(args) = &ev.args {
            for arg in args {
                self.compile_expr(arg)?;
            }
            args.len() as u8
        } else {
            0
        };

        // Emit the enum variant construction opcode
        self.bytecode.emit(Opcode::EnumVariant, ev.span);
        self.bytecode.emit_u8(arg_count);

        Ok(())
    }

    /// Compile a function call expression
    fn compile_call(&mut self, call: &CallExpr) -> Result<(), Vec<Diagnostic>> {
        // Bare user-defined enum variant constructor: `Unknown(raw)` without `EnumName::`.
        // Short-circuit BEFORE callee lookup so variant names never hit GetGlobal.
        // Skip stdlib constructors (Ok, Err, Some, None) — they have dedicated Value types.
        if let Expr::Identifier(id) = call.callee.as_ref() {
            let is_stdlib_ctor = matches!(id.name.as_str(), "Ok" | "Err" | "Some" | "None");
            if !is_stdlib_ctor {
                if let Some((enum_name, arity)) = self.enum_variants.get(&id.name).cloned() {
                    if arity > 0 {
                        // Push enum_name, variant_name, then args, then EnumVariant opcode.
                        let enum_name_idx = self
                            .bytecode
                            .add_constant(crate::value::Value::string(enum_name));
                        let variant_name_idx = self
                            .bytecode
                            .add_constant(crate::value::Value::string(id.name.clone()));
                        self.bytecode.emit(Opcode::Constant, call.span);
                        self.bytecode.emit_u16(enum_name_idx);
                        self.bytecode.emit(Opcode::Constant, call.span);
                        self.bytecode.emit_u16(variant_name_idx);
                        for arg in &call.args {
                            self.compile_expr(arg)?;
                        }
                        self.bytecode.emit(Opcode::EnumVariant, call.span);
                        self.bytecode.emit_u8(call.args.len() as u8);
                        return Ok(());
                    }
                }
            }
        }

        // Extract function name from callee, or compile a complex callee expression
        let func_name_owned: Option<String> = match call.callee.as_ref() {
            Expr::Identifier(ident) => Some(ident.name.clone()),
            _ => None,
        };
        if func_name_owned.is_none() {
            // Complex callee (e.g. index expression, member call on result):
            // push the callee value, then args, then Call.
            self.compile_expr(call.callee.as_ref())?;
            for arg in &call.args {
                self.compile_expr(arg)?;
            }
            self.bytecode.emit(Opcode::Call, call.span);
            self.bytecode.emit_u8(call.args.len() as u8);
            return Ok(());
        }
        let func_name = match func_name_owned.as_deref() {
            Some(name) => name,
            None => return Ok(()),
        };

        // Load the function from local or global scope
        // Don't hardcode builtins - let GetGlobal handle them so natives can override
        {
            // Try local first (for nested functions)
            if let Some(local_idx) = self.resolve_local(func_name) {
                let local = &self.locals[local_idx];

                // Check if this local is from current function's scope or parent scope
                if local.depth < self.scope_depth {
                    if let Some(name_to_use) = local.scoped_name.as_ref() {
                        // Nested function in parent scope — accessible via global scoped name
                        let name_idx = self
                            .bytecode
                            .add_constant(crate::value::Value::string(name_to_use));
                        self.bytecode.emit(Opcode::GetGlobal, call.span);
                        self.bytecode.emit_u16(name_idx);
                    } else if !self.upvalue_stack.is_empty() {
                        // Regular closure/variable from outer scope — load via upvalue
                        let upvalue_idx = self.register_upvalue(func_name, local_idx);
                        self.bytecode.emit(Opcode::GetUpvalue, call.span);
                        self.bytecode.emit_u16(upvalue_idx as u16);
                    } else {
                        // Fallback: GetGlobal
                        let name_idx = self
                            .bytecode
                            .add_constant(crate::value::Value::string(func_name));
                        self.bytecode.emit(Opcode::GetGlobal, call.span);
                        self.bytecode.emit_u16(name_idx);
                    }
                } else {
                    // Current function's scope - use GetLocal with function-relative index
                    let function_relative_idx = local_idx - self.current_function_base;
                    self.bytecode.emit(Opcode::GetLocal, call.span);
                    self.bytecode.emit_u16(function_relative_idx as u16);
                }
            } else {
                // Load from global
                let name_idx = self
                    .bytecode
                    .add_constant(crate::value::Value::string(func_name));
                self.bytecode.emit(Opcode::GetGlobal, call.span);
                self.bytecode.emit_u16(name_idx);
            }
        }

        // Compile all arguments (they'll be pushed on top of the function)
        for arg in &call.args {
            self.compile_expr(arg)?;
        }

        // Emit call instruction with argument count.
        // Use AsyncCall for known async functions so the VM wraps the result in a Future.
        if self.async_fn_names.contains(func_name) {
            self.bytecode.emit(Opcode::AsyncCall, call.span);
        } else {
            self.bytecode.emit(Opcode::Call, call.span);
        }
        self.bytecode.emit_u8(call.args.len() as u8);

        // CoW write-back: collection mutation builtins return the new collection.
        // If the first argument is an identifier, write the result back to that variable.
        self.emit_cow_writeback_if_needed(func_name, call);

        Ok(())
    }

    /// Emit CoW write-back bytecode after a collection mutation builtin call.
    ///
    /// - RETURNS_COLLECTION: `SetLocal/SetGlobal(var)` (peek, keeps value on stack)
    /// - RETURNS_PAIR `[extracted, new_col]`: dup → index(1) → set_var → pop → index(0)
    ///   Result on stack becomes just `extracted` (item), new_col is written to var.
    fn emit_cow_writeback_if_needed(&mut self, func_name: &str, call: &CallExpr) {
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

        let first_ident = call.args.first().and_then(|e| {
            if let Expr::Identifier(id) = e {
                Some(id.name.as_str())
            } else {
                None
            }
        });
        let var_name = match first_ident {
            Some(n) => n,
            None => return,
        };

        if RETURNS_COLLECTION.contains(&func_name) {
            // Stack: new_collection
            // Emit SetLocal/SetGlobal (peek — keeps value on stack for caller)
            self.emit_force_writeback(var_name, call.span);
        } else if RETURNS_PAIR.contains(&func_name) {
            // Stack: [extracted, new_collection]
            // Dup → [..., pair, pair]
            // Constant(1), GetIndex → [..., pair, new_collection]
            // SetLocal/SetGlobal(var) — peek, keeps new_collection on stack
            // Pop → [..., pair]
            // Constant(0), GetIndex → [..., extracted]
            self.bytecode.emit(Opcode::Dup, call.span);
            let idx1 = self.bytecode.add_constant(crate::value::Value::Number(1.0));
            self.bytecode.emit(Opcode::Constant, call.span);
            self.bytecode.emit_u16(idx1);
            self.bytecode.emit(Opcode::GetIndex, call.span);
            self.emit_force_writeback(var_name, call.span);
            self.bytecode.emit(Opcode::Pop, call.span);
            let idx0 = self.bytecode.add_constant(crate::value::Value::Number(0.0));
            self.bytecode.emit(Opcode::Constant, call.span);
            self.bytecode.emit_u16(idx0);
            self.bytecode.emit(Opcode::GetIndex, call.span);
        }
    }

    /// Emit SetLocal or SetGlobal for `var_name`, bypassing mutability checks.
    ///
    /// This mirrors `force_set_collection` in the interpreter: container content
    /// mutation is not a variable rebinding, so mutability doesn't apply.
    fn emit_force_writeback(&mut self, var_name: &str, span: Span) {
        if let Some(local_idx) = self.resolve_local(var_name) {
            let local = &self.locals[local_idx];
            if local.depth < self.scope_depth && !self.upvalue_stack.is_empty() {
                let upvalue_idx = self.register_upvalue(var_name, local_idx);
                self.bytecode.emit(Opcode::SetUpvalue, span);
                self.bytecode.emit_u16(upvalue_idx as u16);
            } else {
                let function_relative_idx = if local.depth < self.scope_depth {
                    local_idx
                } else {
                    local_idx - self.current_function_base
                };
                self.bytecode.emit(Opcode::SetLocal, span);
                self.bytecode.emit_u16(function_relative_idx as u16);
            }
        } else {
            // Global variable (or doesn't exist — silently skip)
            let name_idx = self
                .bytecode
                .add_constant(crate::value::Value::string(var_name));
            self.bytecode.emit(Opcode::SetGlobal, span);
            self.bytecode.emit_u16(name_idx);
        }
    }

    /// Compile a member expression (method call)
    ///
    /// Desugars method calls to stdlib function calls at compile time.
    /// The function name is determined from the method name using a standard mapping:
    ///   value.as_string() → jsonAsString(value)
    fn compile_member(&mut self, member: &MemberExpr) -> Result<(), Vec<Diagnostic>> {
        if member.args.is_none() {
            self.compile_expr(&member.target)?;
            // Tuple element access: .0, .1, ... → TupleGet with numeric index
            if let Ok(idx) = member.member.name.parse::<u16>() {
                self.bytecode.emit(Opcode::TupleGet, member.span);
                self.bytecode.emit_u16(idx);
                return Ok(());
            }
            let key_idx = self
                .bytecode
                .add_constant(Value::string(&member.member.name));
            self.bytecode.emit(Opcode::Constant, member.span);
            self.bytecode.emit_u16(key_idx);
            self.bytecode.emit(Opcode::GetField, member.span);
            return Ok(());
        }

        // Check for static method dispatch (Type.staticMethod() calls).
        // The typechecker annotates `static_dispatch` when a static method is resolved.
        if let Some(type_name) = member.static_dispatch.borrow().clone() {
            // Static methods: __static__TypeName__MethodName (no self parameter)
            let mangled_name = format!("__static__{}__{}", type_name, member.member.name);

            // Push the mangled function by name from globals
            let name_idx = self
                .bytecode
                .add_constant(crate::value::Value::string(&mangled_name));
            self.bytecode.emit(Opcode::GetGlobal, member.span);
            self.bytecode.emit_u16(name_idx);

            // Compile method arguments (no self for static methods)
            if let Some(args) = &member.args {
                for arg in args {
                    self.compile_expr(arg)?;
                }
            }

            let arg_count = member.args.as_ref().map(|a| a.len()).unwrap_or(0);
            self.bytecode.emit(Opcode::Call, member.span);
            self.bytecode.emit_u8(arg_count as u8);

            return Ok(());
        }

        // Check for trait dispatch (user-defined impl methods) first.
        // The typechecker annotates `trait_dispatch` when a trait method is resolved.
        if let Some((type_name, trait_name)) = member.trait_dispatch.borrow().clone() {
            if type_name.is_empty() {
                // Dynamic trait dispatch: resolve impl at runtime
                let trait_idx = self
                    .bytecode
                    .add_constant(crate::value::Value::string(&trait_name));
                let method_idx = self
                    .bytecode
                    .add_constant(crate::value::Value::string(&member.member.name));

                // Compile target (becomes `self` — first argument)
                self.compile_expr(&member.target)?;

                // Compile method arguments
                if let Some(args) = &member.args {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                }

                let arg_count = 1 + member.args.as_ref().map(|a| a.len()).unwrap_or(0);
                self.bytecode.emit(Opcode::TraitDispatch, member.span);
                self.bytecode.emit_u16(trait_idx);
                self.bytecode.emit_u16(method_idx);
                self.bytecode.emit_u8(arg_count as u8);
            } else {
                // Inherent:  __impl__TypeName__MethodName      (trait_name is empty)
                // Trait:     __impl__TypeName__TraitName__MethodName
                let mangled_name = if trait_name.is_empty() {
                    format!("__impl__{}__{}", type_name, member.member.name)
                } else {
                    format!(
                        "__impl__{}__{}__{}",
                        type_name, trait_name, member.member.name
                    )
                };

                // Push the mangled function by name from globals
                let name_idx = self
                    .bytecode
                    .add_constant(crate::value::Value::string(&mangled_name));
                self.bytecode.emit(Opcode::GetGlobal, member.span);
                self.bytecode.emit_u16(name_idx);

                // Compile target (becomes `self` — first argument)
                self.compile_expr(&member.target)?;

                // Compile method arguments
                if let Some(args) = &member.args {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                }

                let arg_count = 1 + member.args.as_ref().map(|a| a.len()).unwrap_or(0);
                self.bytecode.emit(Opcode::Call, member.span);
                self.bytecode.emit_u8(arg_count as u8);
            }

            return Ok(());
        }

        // Resolve method via shared dispatch table (type tag set by typechecker).
        // If type_tag is None, the typechecker could not resolve the target type — surface as
        // a diagnostic rather than panicking (fuzzer safety, and graceful error for callers).
        if let Some(type_tag) = member.type_tag.get() {
            // For HashMap targets, if resolve_method fails, fall back to treating the member
            // as a callable field. This handles namespace imports (`import * as ns`) which are
            // stored as Value::HashMap but whose fields are user-defined functions.
            if matches!(type_tag, crate::method_dispatch::TypeTag::HashMap)
                && member.args.is_some()
                && crate::method_dispatch::resolve_method(type_tag, &member.member.name).is_none()
            {
                // Emit: load the HashMap, get the field (function value), then call it
                self.compile_expr(&member.target)?;
                let field_const = self
                    .bytecode
                    .add_constant(crate::value::Value::string(&member.member.name));
                self.bytecode.emit(Opcode::Constant, member.span);
                self.bytecode.emit_u16(field_const);
                self.bytecode.emit(Opcode::GetIndex, member.span);

                // Compile arguments
                if let Some(args) = &member.args {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                }
                let arg_count = member.args.as_ref().map(|a| a.len()).unwrap_or(0);
                self.bytecode.emit(Opcode::Call, member.span);
                self.bytecode.emit_u8(arg_count as u8);
                return Ok(());
            }

            let func_name = crate::method_dispatch::resolve_method(type_tag, &member.member.name)
                .ok_or_else(|| {
                vec![crate::diagnostic::Diagnostic::error(
                    format!("No method '{}' on type {:?}", member.member.name, type_tag),
                    member.span,
                )]
            })?;

            // Load the stdlib function as a Builtin constant
            let func_value = crate::value::Value::Builtin(std::sync::Arc::from(func_name.as_str()));
            let const_idx = self.bytecode.add_constant(func_value);

            // Load the function constant
            self.bytecode.emit(Opcode::Constant, member.span);
            self.bytecode.emit_u16(const_idx);

            // For static namespaces (Json/Math/Env), the target is a sentinel — not a receiver.
            // Skip compiling it as an argument.
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
                    | crate::method_dispatch::TypeTag::ConsoleNs
                    | crate::method_dispatch::TypeTag::GzipNs
                    | crate::method_dispatch::TypeTag::TarNs
                    | crate::method_dispatch::TypeTag::ZipNs
                    | crate::method_dispatch::TypeTag::TaskNs
                    | crate::method_dispatch::TypeTag::SyncNs
                    | crate::method_dispatch::TypeTag::FutureNs
                    | crate::method_dispatch::TypeTag::TestNs
            );

            if !is_ns {
                // Compile target (becomes first argument for instance methods)
                self.compile_expr(&member.target)?;
            }

            // Compile method arguments
            if let Some(args) = &member.args {
                for arg in args {
                    self.compile_expr(arg)?;
                }
            }

            // Emit call instruction
            let arg_count = if is_ns {
                // No receiver for namespace calls — only explicit args
                member.args.as_ref().map(|a| a.len()).unwrap_or(0)
            } else {
                1 + member.args.as_ref().map(|a| a.len()).unwrap_or(0)
            };
            self.bytecode.emit(Opcode::Call, member.span);
            self.bytecode.emit_u8(arg_count as u8);

            // CoW write-back: for mutating collection methods, update the receiver variable.
            // Only possible when the target is a simple identifier.
            if let crate::ast::Expr::Identifier(id) = member.target.as_ref() {
                let var_name = id.name.as_str();
                if crate::method_dispatch::is_array_mutating_collection(&func_name) {
                    // Stack: new_array — peek-set to receiver, value stays on stack
                    self.emit_force_writeback(var_name, member.span);
                } else if crate::method_dispatch::is_array_mutating_pair(&func_name) {
                    // Stack: [extracted, new_array]
                    // Dup → get index 1 (new_array) → set receiver → pop → get index 0 (extracted)
                    self.bytecode.emit(Opcode::Dup, member.span);
                    let idx1 = self.bytecode.add_constant(crate::value::Value::Number(1.0));
                    self.bytecode.emit(Opcode::Constant, member.span);
                    self.bytecode.emit_u16(idx1);
                    self.bytecode.emit(Opcode::GetIndex, member.span);
                    self.emit_force_writeback(var_name, member.span);
                    self.bytecode.emit(Opcode::Pop, member.span);
                    let idx0 = self.bytecode.add_constant(crate::value::Value::Number(0.0));
                    self.bytecode.emit(Opcode::Constant, member.span);
                    self.bytecode.emit_u16(idx0);
                    self.bytecode.emit(Opcode::GetIndex, member.span);
                } else if crate::method_dispatch::is_collection_mutating_simple(&func_name) {
                    // Stack: new_collection — write back to receiver, value stays on stack
                    self.emit_force_writeback(var_name, member.span);
                } else if crate::method_dispatch::is_collection_mutating_pair(&func_name) {
                    // Stack: [extracted, new_collection] — same pattern as array pair
                    self.bytecode.emit(Opcode::Dup, member.span);
                    let idx1 = self.bytecode.add_constant(crate::value::Value::Number(1.0));
                    self.bytecode.emit(Opcode::Constant, member.span);
                    self.bytecode.emit_u16(idx1);
                    self.bytecode.emit(Opcode::GetIndex, member.span);
                    self.emit_force_writeback(var_name, member.span);
                    self.bytecode.emit(Opcode::Pop, member.span);
                    let idx0 = self.bytecode.add_constant(crate::value::Value::Number(0.0));
                    self.bytecode.emit(Opcode::Constant, member.span);
                    self.bytecode.emit_u16(idx0);
                    self.bytecode.emit(Opcode::GetIndex, member.span);
                }
            }

            Ok(())
        } else {
            // Structural function call: resolve member then call with args.
            self.compile_expr(&member.target)?;
            let key_idx = self
                .bytecode
                .add_constant(Value::string(&member.member.name));
            self.bytecode.emit(Opcode::Constant, member.span);
            self.bytecode.emit_u16(key_idx);
            self.bytecode.emit(Opcode::GetField, member.span);

            if let Some(args) = &member.args {
                for arg in args {
                    self.compile_expr(arg)?;
                }
            }

            let arg_count = member.args.as_ref().map(|a| a.len()).unwrap_or(0);
            self.bytecode.emit(Opcode::Call, member.span);
            self.bytecode.emit_u8(arg_count as u8);
            Ok(())
        }
    }

    /// Compile a literal
    fn compile_literal(&mut self, lit: &Literal, span: Span) -> Result<(), Vec<Diagnostic>> {
        match lit {
            Literal::Number(n) => {
                let idx = self.bytecode.add_constant(Value::Number(*n));
                self.bytecode.emit(Opcode::Constant, span);
                self.bytecode.emit_u16(idx);
            }
            Literal::String(s) => {
                let idx = self.bytecode.add_constant(Value::string(s));
                self.bytecode.emit(Opcode::Constant, span);
                self.bytecode.emit_u16(idx);
            }
            Literal::Bool(b) => {
                let opcode = if *b { Opcode::True } else { Opcode::False };
                self.bytecode.emit(opcode, span);
            }
            Literal::Null => {
                self.bytecode.emit(Opcode::Null, span);
            }
        }
        Ok(())
    }

    /// Compile an identifier (variable access)
    fn compile_identifier(&mut self, ident: &Identifier) -> Result<(), Vec<Diagnostic>> {
        // Check for compile-time constant first — inline the value directly
        if let Some(const_val) = self.const_values.get(&ident.name).cloned() {
            let idx = self.bytecode.add_constant(const_val);
            self.bytecode.emit(Opcode::Constant, ident.span);
            self.bytecode.emit_u16(idx);
            return Ok(());
        }

        // User-defined unit enum variants (e.g. `Quit` from `enum CommandResult { Quit, ... }`).
        // Emit EnumVariant opcode directly — avoids GetGlobal lookup failure.
        // Skip stdlib constructors (Ok, Err, Some, None).
        let is_stdlib_ctor = matches!(ident.name.as_str(), "Ok" | "Err" | "Some" | "None");
        if !is_stdlib_ctor {
            if let Some((enum_name, arity)) = self.enum_variants.get(&ident.name).cloned() {
                if arity == 0 {
                    let enum_name_idx = self
                        .bytecode
                        .add_constant(crate::value::Value::string(enum_name));
                    let variant_name_idx = self
                        .bytecode
                        .add_constant(crate::value::Value::string(ident.name.clone()));
                    self.bytecode.emit(Opcode::Constant, ident.span);
                    self.bytecode.emit_u16(enum_name_idx);
                    self.bytecode.emit(Opcode::Constant, ident.span);
                    self.bytecode.emit_u16(variant_name_idx);
                    self.bytecode.emit(Opcode::EnumVariant, ident.span);
                    self.bytecode.emit_u8(0);
                    return Ok(());
                }
            }
        }

        // Try to resolve as local first
        if let Some(local_idx) = self.resolve_local(&ident.name) {
            let local = &self.locals[local_idx];

            if local_idx >= self.current_function_base {
                // Local in current function (including outer block scopes).
                let function_relative_idx = local_idx - self.current_function_base;
                self.bytecode.emit(Opcode::GetLocal, ident.span);
                self.bytecode.emit_u16(function_relative_idx as u16);
            } else if let Some(name_to_use) = local.scoped_name.as_ref() {
                // Nested function in parent scope — accessible via its global scoped name
                let name_idx = self.bytecode.add_constant(Value::string(name_to_use));
                self.bytecode.emit(Opcode::GetGlobal, ident.span);
                self.bytecode.emit_u16(name_idx);
            } else if !self.upvalue_stack.is_empty() {
                // Regular variable from outer function scope — capture as upvalue
                let upvalue_idx = self.register_upvalue(&ident.name, local_idx);
                self.bytecode.emit(Opcode::GetUpvalue, ident.span);
                self.bytecode.emit_u16(upvalue_idx as u16);
            } else {
                // Outer scope but not in a nested function — use GetGlobal fallback
                let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
                self.bytecode.emit(Opcode::GetGlobal, ident.span);
                self.bytecode.emit_u16(name_idx);
            }
        } else {
            // Global variable
            let name_idx = self.bytecode.add_constant(Value::string(&ident.name));
            self.bytecode.emit(Opcode::GetGlobal, ident.span);
            self.bytecode.emit_u16(name_idx);
        }
        Ok(())
    }

    /// Compile a binary expression
    fn compile_binary(&mut self, bin: &BinaryExpr) -> Result<(), Vec<Diagnostic>> {
        // Handle short-circuit evaluation for && and ||
        match bin.op {
            BinaryOp::And => {
                // For &&: if left is false, result is false (don't eval right)
                // Compile left
                self.compile_expr(&bin.left)?;
                // Duplicate for the check
                self.bytecode.emit(Opcode::Dup, bin.span);
                // Jump to end if false (keeping false on stack)
                self.bytecode.emit(Opcode::JumpIfFalse, bin.span);
                let end_jump = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF); // Placeholder

                // Left was true, pop it and eval right
                self.bytecode.emit(Opcode::Pop, bin.span);
                self.compile_expr(&bin.right)?;

                // Patch jump
                self.bytecode.patch_jump(end_jump);
                Ok(())
            }
            BinaryOp::Or => {
                // For ||: if left is true, result is true (don't eval right)
                // Compile left
                self.compile_expr(&bin.left)?;
                // Duplicate for the check
                self.bytecode.emit(Opcode::Dup, bin.span);
                // If true, jump to end (keeping true on stack)
                // We need "jump if true" but we only have "jump if false"
                // So: if NOT false, jump to end
                // Actually, we need to negate the logic:
                // Dup, Not, JumpIfFalse (jumps if original was true)
                self.bytecode.emit(Opcode::Not, bin.span);
                self.bytecode.emit(Opcode::JumpIfFalse, bin.span);
                let end_jump = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF); // Placeholder

                // Left was false, pop it and eval right
                self.bytecode.emit(Opcode::Pop, bin.span);
                self.compile_expr(&bin.right)?;

                // Patch jump
                self.bytecode.patch_jump(end_jump);
                Ok(())
            }
            _ => {
                // For all other operators, evaluate both sides
                self.compile_expr(&bin.left)?;
                self.compile_expr(&bin.right)?;

                // Emit the appropriate opcode
                let opcode = match bin.op {
                    BinaryOp::Add => Opcode::Add,
                    BinaryOp::Sub => Opcode::Sub,
                    BinaryOp::Mul => Opcode::Mul,
                    BinaryOp::Div => Opcode::Div,
                    BinaryOp::Mod => Opcode::Mod,
                    BinaryOp::Eq => Opcode::Equal,
                    BinaryOp::Ne => Opcode::NotEqual,
                    BinaryOp::Lt => Opcode::Less,
                    BinaryOp::Le => Opcode::LessEqual,
                    BinaryOp::Gt => Opcode::Greater,
                    BinaryOp::Ge => Opcode::GreaterEqual,
                    BinaryOp::And | BinaryOp::Or => unreachable!(), // Handled above
                };
                self.bytecode.emit(opcode, bin.span);
                Ok(())
            }
        }
    }

    /// Compile a unary expression
    fn compile_unary(&mut self, un: &UnaryExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile the operand
        self.compile_expr(&un.expr)?;

        // Emit the appropriate opcode
        let opcode = match un.op {
            UnaryOp::Negate => Opcode::Negate,
            UnaryOp::Not => Opcode::Not,
        };
        self.bytecode.emit(opcode, un.span);
        Ok(())
    }

    /// Compile an array literal
    fn compile_array_literal(&mut self, arr: &ArrayLiteral) -> Result<(), Vec<Diagnostic>> {
        // Compile all elements (leaves them on stack)
        for elem in &arr.elements {
            self.compile_expr(elem)?;
        }

        // Emit Array instruction with element count
        self.bytecode.emit(Opcode::Array, arr.span);
        self.bytecode.emit_u16(arr.elements.len() as u16);

        Ok(())
    }

    /// Compile an object literal: `record { key: value, key2: value2 }`
    fn compile_object_literal(&mut self, obj: &ObjectLiteral) -> Result<(), Vec<Diagnostic>> {
        // Push key-value pairs onto stack (interleaved: key1, val1, key2, val2, ...)
        for entry in &obj.entries {
            // Push key as string constant
            let key_idx = self
                .bytecode
                .add_constant(Value::string(entry.key.name.clone()));
            self.bytecode.emit(Opcode::Constant, entry.key.span);
            self.bytecode.emit_u16(key_idx);

            // Compile value expression
            self.compile_expr(&entry.value)?;
        }

        // Emit HashMap instruction with entry count
        self.bytecode.emit(Opcode::HashMap, obj.span);
        self.bytecode.emit_u16(obj.entries.len() as u16);

        Ok(())
    }

    /// Compile an index expression
    fn compile_index(&mut self, index: &IndexExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile the target (array)
        self.compile_expr(&index.target)?;

        match &index.index {
            IndexValue::Single(expr) => {
                self.compile_expr(expr)?;
                self.bytecode.emit(Opcode::GetIndex, index.span);
            }
        }

        Ok(())
    }

    /// Compile a match expression
    ///
    /// Strategy: Use Dup for non-last arms so the scrutinee is available for
    /// subsequent arms. Pattern check consumes the copy (dup or original for last
    /// arm). After body compilation, a temp global is used to save/restore the
    /// result while popping extra stack values (scrutinee, pattern variables).
    ///
    /// This avoids using hidden locals (which break when match is used inside
    /// other expressions due to temporaries corrupting local indices).
    fn compile_match(&mut self, match_expr: &MatchExpr) -> Result<(), Vec<Diagnostic>> {
        // Compile scrutinee (leaves value on stack as a temporary)
        self.compile_expr(&match_expr.scrutinee)?;

        // Temp global name for saving the match result during cleanup
        let temp_name = "$match_result";
        let temp_name_idx = self.bytecode.add_constant(Value::string(temp_name));

        let mut arm_end_jumps = Vec::new();

        for (arm_idx, arm) in match_expr.arms.iter().enumerate() {
            let is_last_arm = arm_idx == match_expr.arms.len() - 1;
            let locals_before = self.locals.len();

            if !is_last_arm {
                // Dup scrutinee so next arm can use the original
                self.bytecode.emit(Opcode::Dup, arm.span);
            }

            // Pattern check consumes the top value (dup or scrutinee for last arm).
            // On success: pushes True, may add pattern variable locals.
            // On failure: stack clean (copy consumed), jumps to fail target.
            let match_failed_jump =
                self.compile_pattern_check(&arm.pattern, arm.span, locals_before)?;

            // Pop True (pattern success flag)
            self.bytecode.emit(Opcode::Pop, arm.span);

            // Compile guard if present — guard failure jumps to next arm
            let guard_failed_jump = if let Some(guard_expr) = &arm.guard {
                self.compile_expr(guard_expr)?;
                // JumpIfFalse: if guard is false, skip this arm
                self.bytecode.emit(Opcode::JumpIfFalse, arm.span);
                let guard_jump = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);
                Some(guard_jump)
            } else {
                None
            };

            // Compile arm body (result on top of stack)
            self.compile_expr(&arm.body)?;

            // Cleanup: remove extras (pattern vars + scrutinee for non-last) from
            // below the body result. Save result to temp global, pop extras, restore.
            let pattern_var_count = self.locals.len() - locals_before;
            let extras = pattern_var_count + if !is_last_arm { 1 } else { 0 };

            if extras > 0 {
                // Save result to temp global (SetGlobal peeks, value stays)
                self.bytecode.emit(Opcode::SetGlobal, arm.span);
                self.bytecode.emit_u16(temp_name_idx);
                // Pop result copy + all extras
                for _ in 0..=extras {
                    self.bytecode.emit(Opcode::Pop, arm.span);
                }
                // Restore result from temp global
                self.bytecode.emit(Opcode::GetGlobal, arm.span);
                self.bytecode.emit_u16(temp_name_idx);
            }

            // Jump to end (skip other arms)
            if !is_last_arm {
                self.bytecode.emit(Opcode::Jump, arm.span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);
                arm_end_jumps.push(jump_offset);
            }

            // Guard cleanup (if guard was present): guard failure jumps here, pops stack items,
            // then falls through to next arm. Pattern failure jumps AFTER cleanup (no pops needed).
            if let Some(guard_jump) = guard_failed_jump {
                // Patch guard jump to point here (start of cleanup)
                self.bytecode.patch_jump(guard_jump);
                // Pop all extras: pattern vars + dup copy (for non-last arms).
                // This mirrors the `extras` formula used in the success path, ensuring the stack
                // is restored to the base scrutinee level for the next arm.
                let pattern_var_count_guard = self.locals.len() - locals_before;
                let guard_cleanup_count =
                    pattern_var_count_guard + if !is_last_arm { 1 } else { 0 };
                for _ in 0..guard_cleanup_count {
                    self.bytecode.emit(Opcode::Pop, arm.span);
                }
                // Fall through to next arm (match_failed_jump patches below)
            }
            // Patch the failed pattern jump (next arm starts here, after any guard cleanup)
            if let Some(failed_jump) = match_failed_jump {
                self.bytecode.patch_jump(failed_jump);
            }

            // Clean up locals tracking
            self.locals.truncate(locals_before);
        }

        // Patch all end jumps to point here
        for jump_offset in arm_end_jumps {
            self.bytecode.patch_jump(jump_offset);
        }

        // Stack: [body_result] — match expression produces exactly one value
        Ok(())
    }

    /// Compile pattern matching check
    ///
    /// Contract:
    /// - INPUT: scrutinee copy on top of stack (from GetLocal in compile_match)
    /// - SUCCESS: scrutinee copy consumed, True pushed on stack, pattern vars added as locals
    /// - FAILURE: scrutinee copy consumed, stack clean, jumps to returned offset
    ///
    /// Returns: Optional jump offset to patch if match fails (None for wildcard/variable)
    fn compile_pattern_check(
        &mut self,
        pattern: &Pattern,
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::ast::{Literal, Pattern};

        match pattern {
            // Wildcard: always matches, no bindings
            Pattern::Wildcard(_) => {
                // Pop scrutinee copy (consumed)
                self.bytecode.emit(Opcode::Pop, span);
                // Push true (match succeeded)
                self.bytecode.emit(Opcode::True, span);
                Ok(None) // No jump needed, always matches
            }

            // Variable: always matches, bind to local
            Pattern::Variable(id) => {
                // Register local for the pattern variable
                self.push_local(Local {
                    name: id.name.clone(),
                    depth: self.scope_depth,
                    mutable: false,
                    scoped_name: None,
                    drop_type: None, // match bindings: copy semantics
                });
                let local_idx = (self.locals.len() - 1 - self.current_function_base) as u16;

                // Copy value from stack top to the local's slot position.
                // This is necessary because temporaries (from enclosing
                // expressions, scrutinee copies, etc.) may sit between the
                // previous locals and the stack top, so the value isn't
                // naturally at the right position.
                self.bytecode.emit(Opcode::SetLocal, span);
                self.bytecode.emit_u16(local_idx);

                // Push true (match succeeded)
                self.bytecode.emit(Opcode::True, span);
                Ok(None) // No jump needed, always matches
            }

            // Literal: check equality
            Pattern::Literal(lit, lit_span) => {
                // Scrutinee copy is on stack. Push literal value for comparison.
                match lit {
                    Literal::Number(n) => {
                        let const_idx = self.bytecode.add_constant(Value::Number(*n));
                        self.bytecode.emit(Opcode::Constant, *lit_span);
                        self.bytecode.emit_u16(const_idx);
                    }
                    Literal::String(s) => {
                        let const_idx = self.bytecode.add_constant(Value::string(s.clone()));
                        self.bytecode.emit(Opcode::Constant, *lit_span);
                        self.bytecode.emit_u16(const_idx);
                    }
                    Literal::Bool(true) => {
                        self.bytecode.emit(Opcode::True, *lit_span);
                    }
                    Literal::Bool(false) => {
                        self.bytecode.emit(Opcode::False, *lit_span);
                    }
                    Literal::Null => {
                        self.bytecode.emit(Opcode::Null, *lit_span);
                    }
                }

                // Compare: pops both (scrutinee copy + literal), pushes bool
                self.bytecode.emit(Opcode::Equal, span);

                // If false, jump to fail target
                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);

                // On success path: Equal consumed the copy, JumpIfFalse consumed the bool.
                // Push True to satisfy the contract.
                self.bytecode.emit(Opcode::True, span);

                Ok(Some(jump_offset))
            }

            // Constructor: Some(x), None, Ok(x), Err(e)
            Pattern::Constructor { name, args, span } => {
                self.compile_constructor_pattern(name, args, *span, locals_before)
            }

            // Array: [x, y, z]
            Pattern::Array { elements, span } => {
                self.compile_array_pattern(elements, *span, locals_before)
            }

            // Tuple: (x, y, z) — not yet compiled; emit diagnostic stub
            Pattern::Tuple { elements, span } => {
                self.compile_tuple_pattern(elements, *span, locals_before)
            }

            // OR pattern: sub1 | sub2 | sub3
            Pattern::Or(alternatives, or_span) => {
                self.compile_or_pattern(alternatives, *or_span, locals_before)
            }

            // Enum variant pattern: State::Running, Color::Rgb(r, g, b)
            Pattern::EnumVariant {
                enum_name,
                variant_name,
                args,
                span,
            } => self.compile_enum_variant_pattern(
                enum_name,
                variant_name,
                args,
                *span,
                locals_before,
            ),

            // Bare variant pattern: Running, Pending(msg) — no EnumName:: prefix.
            // Compile as EnumVariant with an empty-string sentinel for enum_name;
            // the VM skips the enum_name check when it is the empty string.
            Pattern::BareVariant { name, args, span } => {
                let sentinel = crate::ast::Identifier {
                    name: String::new(),
                    span: name.span,
                };
                self.compile_enum_variant_pattern(&sentinel, name, args, *span, locals_before)
            }
        }
    }

    /// Compile constructor pattern (Some, None, Ok, Err)
    ///
    /// Contract (same as compile_pattern_check):
    /// - INPUT: scrutinee copy on top of stack
    /// - SUCCESS: copy consumed, True on stack, pattern vars as locals
    /// - FAILURE: copy consumed, stack clean, jumps to returned offset
    fn compile_constructor_pattern(
        &mut self,
        name: &crate::ast::Identifier,
        args: &[Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::bytecode::Opcode;

        match name.name.as_str() {
            "None" => {
                if !args.is_empty() {
                    return Err(vec![INTERNAL_ERROR.emit(span)
                        .arg("detail", "None pattern should not have arguments")
                        .with_help("use 'None' without arguments to match empty Option values")
                        .build()]);
                }

                // IsOptionNone: pops copy, pushes bool
                self.bytecode.emit(Opcode::IsOptionNone, span);
                // JumpIfFalse: pops bool. On failure: stack clean, jumps.
                self.bytecode.emit(Opcode::JumpIfFalse, span);
                let jump_offset = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);
                // On success: copy consumed, bool consumed. Push True.
                self.bytecode.emit(Opcode::True, span);

                Ok(Some(jump_offset))
            }

            "Some" => {
                if args.len() != 1 {
                    return Err(vec![INTERNAL_ERROR.emit(span)
                        .arg("detail", "Some pattern requires exactly one argument")
                        .with_help("use 'Some(value)' to match and extract the inner value from Option")
                        .build()]);
                }

                self.compile_wrapping_constructor_pattern(
                    Opcode::IsOptionSome,
                    Opcode::ExtractOptionValue,
                    &args[0],
                    span,
                    locals_before,
                )
            }

            "Ok" => {
                if args.len() != 1 {
                    return Err(vec![INTERNAL_ERROR.emit(span)
                        .arg("detail", "Ok pattern requires exactly one argument")
                        .with_help("use 'Ok(value)' to match and extract the success value from Result")
                        .build()]);
                }

                self.compile_wrapping_constructor_pattern(
                    Opcode::IsResultOk,
                    Opcode::ExtractResultValue,
                    &args[0],
                    span,
                    locals_before,
                )
            }

            "Err" => {
                if args.len() != 1 {
                    return Err(vec![INTERNAL_ERROR.emit(span)
                        .arg("detail", "Err pattern requires exactly one argument")
                        .with_help("use 'Err(error)' to match and extract the error value from Result")
                        .build()]);
                }

                self.compile_wrapping_constructor_pattern(
                    Opcode::IsResultErr,
                    Opcode::ExtractResultValue,
                    &args[0],
                    span,
                    locals_before,
                )
            }

            _ => Err(vec![INTERNAL_ERROR.emit(span)
                .arg("detail", format!("unknown constructor pattern: {}", name.name))
                .with_help("valid constructor patterns are: Some, None (for Option) and Ok, Err (for Result)")
                .build()]),
        }
    }

    /// Compile a wrapping constructor pattern (Some(x), Ok(x), Err(x))
    ///
    /// These patterns check a variant, extract the inner value, then recursively
    /// match the inner pattern.
    ///
    /// Stack protocol (same as compile_pattern_check):
    /// - INPUT: [copy] on stack
    /// - SUCCESS: copy consumed, [inner_locals...] [True] on stack
    /// - FAILURE: copy consumed, stack clean, jumps to returned offset
    ///
    /// Emitted code structure:
    /// ```text
    ///   Dup                              // [copy, dup]
    ///   check_opcode                     // [copy, bool]
    ///   JumpIfFalse → outer_fail         // [copy]
    ///   extract_opcode                   // [inner]
    ///   <inner pattern check>            // [inner_locals..., True] or jump
    ///   Jump → success_exit              // skip failure code
    ///   [inner_fail: Jump → fail_exit]   // (only if inner can fail)
    ///   outer_fail: Pop                  // [] clean
    ///   fail_exit: Jump → ???            // compile_match patches this
    ///   success_exit:                    // [inner_locals..., True]
    /// ```
    fn compile_wrapping_constructor_pattern(
        &mut self,
        check_opcode: Opcode,
        extract_opcode: Opcode,
        inner_pattern: &Pattern,
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        // Stack: [copy]
        self.bytecode.emit(Opcode::Dup, span);
        // Stack: [copy, dup]

        self.bytecode.emit(check_opcode, span);
        // Stack: [copy, bool]

        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let outer_fail = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);
        // Stack (success path): [copy]

        self.bytecode.emit(extract_opcode, span);
        // Stack: [inner]

        // Recursively match inner pattern
        let inner_failed = self.compile_pattern_check(inner_pattern, span, locals_before)?;
        // Success: [inner_locals..., True]

        // Jump over failure code
        self.bytecode.emit(Opcode::Jump, span);
        let success_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // --- Failure paths ---

        // Inner pattern failure (if inner can fail)
        if let Some(inner_jump) = inner_failed {
            self.bytecode.patch_jump(inner_jump);
            // Inner pattern guarantees stack is clean (inner value consumed).
            // Jump over the outer_fail Pop to the shared fail exit.
            self.bytecode.emit(Opcode::Jump, span);
            let inner_to_fail = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF);

            // Outer variant check failure: copy still on stack
            self.bytecode.patch_jump(outer_fail);
            self.bytecode.emit(Opcode::Pop, span); // Pop copy → clean

            // Shared fail exit (inner and outer paths converge)
            self.bytecode.patch_jump(inner_to_fail);
        } else {
            // No inner failure possible. Only outer fail path.
            self.bytecode.patch_jump(outer_fail);
            self.bytecode.emit(Opcode::Pop, span); // Pop copy → clean
        }

        // Emit fail jump for compile_match to patch
        self.bytecode.emit(Opcode::Jump, span);
        let fail_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Success exit: inner pattern's True is already on stack
        self.bytecode.patch_jump(success_exit);

        Ok(Some(fail_exit))
    }

    /// Compile array pattern [x, y, z]
    ///
    /// Stack protocol (same as compile_pattern_check):
    /// - INPUT: [copy] (array value) on stack
    /// - SUCCESS: copy consumed, [element_locals...] [True] on stack
    /// - FAILURE: copy consumed, stack clean, jumps to returned offset
    ///
    /// The array is stored in a temp global ("$match_array") so it can be
    /// accessed for each element without interfering with stack/local positions.
    fn compile_array_pattern(
        &mut self,
        elements: &[Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::bytecode::Opcode;

        let array_global_name = "$match_array";
        let array_name_idx = self.bytecode.add_constant(Value::string(array_global_name));

        // Stack: [copy] (the array)
        // Store array to temp global for repeated access
        self.bytecode.emit(Opcode::SetGlobal, span);
        self.bytecode.emit_u16(array_name_idx);
        // Pop the array from stack (SetGlobal peeks)
        self.bytecode.emit(Opcode::Pop, span);
        // Stack: [] (array is in temp global)

        // Check if value is an array
        self.bytecode.emit(Opcode::GetGlobal, span);
        self.bytecode.emit_u16(array_name_idx);
        self.bytecode.emit(Opcode::IsArray, span);

        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let not_array_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Check array length
        self.bytecode.emit(Opcode::GetGlobal, span);
        self.bytecode.emit_u16(array_name_idx);
        self.bytecode.emit(Opcode::GetArrayLen, span);

        let expected_len = elements.len() as f64;
        let const_idx = self.bytecode.add_constant(Value::Number(expected_len));
        self.bytecode.emit(Opcode::Constant, span);
        self.bytecode.emit_u16(const_idx);

        self.bytecode.emit(Opcode::Equal, span);

        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let wrong_length_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Array type and length match. Match each element.
        // Track element failure info: (jump_offset, element_locals_above_baseline)
        let mut elem_fail_info: Vec<(usize, usize)> = Vec::new();

        for (idx, elem_pattern) in elements.iter().enumerate() {
            // Get array from temp global
            self.bytecode.emit(Opcode::GetGlobal, span);
            self.bytecode.emit_u16(array_name_idx);

            // Push index
            let idx_const = self.bytecode.add_constant(Value::Number(idx as f64));
            self.bytecode.emit(Opcode::Constant, span);
            self.bytecode.emit_u16(idx_const);

            // Get element (pops array_copy and index, pushes element)
            self.bytecode.emit(Opcode::GetIndex, span);

            // Match element pattern
            let elem_failed = self.compile_pattern_check(elem_pattern, span, locals_before)?;

            // Pop True from successful element match
            self.bytecode.emit(Opcode::Pop, span);

            if let Some(jump) = elem_failed {
                // Record how many element locals exist at this failure point
                let elem_locals = self.locals.len() - locals_before;
                elem_fail_info.push((jump, elem_locals));
            }
        }

        // All elements matched! Push True (contract).
        self.bytecode.emit(Opcode::True, span);

        // Jump over failure code
        self.bytecode.emit(Opcode::Jump, span);
        let success_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // --- Failure paths ---

        // Element failure handlers: pop element locals, jump to fail_exit
        let mut handler_jumps = Vec::new();
        for (jump, elem_locals) in &elem_fail_info {
            self.bytecode.patch_jump(*jump);
            // Pop element locals from earlier successful elements
            for _ in 0..*elem_locals {
                self.bytecode.emit(Opcode::Pop, span);
            }
            self.bytecode.emit(Opcode::Jump, span);
            handler_jumps.push(self.bytecode.current_offset());
            self.bytecode.emit_u16(0xFFFF);
        }

        // Type/length check failures: stack is already clean
        self.bytecode.patch_jump(not_array_jump);
        self.bytecode.patch_jump(wrong_length_jump);

        // All failure paths converge here
        for j in &handler_jumps {
            self.bytecode.patch_jump(*j);
        }

        // Fail exit: compile_match patches this
        self.bytecode.emit(Opcode::Jump, span);
        let fail_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Success exit
        self.bytecode.patch_jump(success_exit);
        // Stack: [element_locals..., True]

        Ok(Some(fail_exit))
    }

    /// Compile tuple pattern: (p1, p2, ...)
    ///
    /// Contract same as compile_array_pattern:
    /// - INPUT: scrutinee on stack top
    /// - SUCCESS: scrutinee consumed, True on stack, pattern vars added as locals
    /// - FAILURE: scrutinee consumed, stack clean, returns jump offset for caller to patch
    fn compile_tuple_pattern(
        &mut self,
        elements: &[Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::bytecode::Opcode;

        // Store the tuple in a temp global so we can get each element without consuming it.
        let tuple_global_name = "$match_tuple";
        let name_idx = self.bytecode.add_constant(Value::string(tuple_global_name));

        // INPUT: tuple on TOS
        self.bytecode.emit(Opcode::SetGlobal, span);
        self.bytecode.emit_u16(name_idx);
        self.bytecode.emit(Opcode::Pop, span);

        // Check that value is a tuple with the right arity.
        // We load the temp global, push expected length constant, compare.
        // There's no IsArray equivalent for tuples — use TupleGet out-of-bounds as check.
        // Simpler: load the global and check via a length comparison using VM introspection.
        // Atlas has no built-in "is_tuple" opcode, so we rely on TupleGet returning an error
        // on non-tuple values. But we can't catch VM errors in the pattern match.
        //
        // Strategy: emit a Constant(expected_len), GetGlobal(temp), call a pattern-check
        // sequence that loads each element by TupleGet — if TupleGet fails (non-tuple or
        // wrong arity) the VM raises a runtime error.
        //
        // For now: TupleGet is safe — if value is wrong type VM raises RuntimeError which
        // propagates. This matches how interpreter returns None for non-tuple.
        // A real "is_tuple" opcode would be B15-P07 follow-on.
        // For now: just load the elements and match. Arity errors will be runtime errors.

        let mut elem_fail_info: Vec<(usize, usize)> = Vec::new();

        for (idx, elem_pattern) in elements.iter().enumerate() {
            // Load tuple from temp global, extract element idx
            self.bytecode.emit(Opcode::GetGlobal, span);
            self.bytecode.emit_u16(name_idx);
            self.bytecode.emit(Opcode::TupleGet, span);
            self.bytecode.emit_u16(idx as u16);

            // Match element pattern
            let elem_failed = self.compile_pattern_check(elem_pattern, span, locals_before)?;

            // Pop True from successful element match
            self.bytecode.emit(Opcode::Pop, span);

            if let Some(jump) = elem_failed {
                let elem_locals = self.locals.len() - locals_before;
                elem_fail_info.push((jump, elem_locals));
            }
        }

        // All elements matched — push True
        self.bytecode.emit(Opcode::True, span);

        // Jump over failure code
        self.bytecode.emit(Opcode::Jump, span);
        let success_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Element failure handlers
        let mut handler_jumps = Vec::new();
        for (jump, elem_locals) in &elem_fail_info {
            self.bytecode.patch_jump(*jump);
            for _ in 0..*elem_locals {
                self.bytecode.emit(Opcode::Pop, span);
            }
            self.bytecode.emit(Opcode::Jump, span);
            handler_jumps.push(self.bytecode.current_offset());
            self.bytecode.emit_u16(0xFFFF);
        }

        for j in &handler_jumps {
            self.bytecode.patch_jump(*j);
        }

        // Fail exit
        self.bytecode.emit(Opcode::Jump, span);
        let fail_exit = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        // Patch success exit
        self.bytecode.patch_jump(success_exit);

        Ok(Some(fail_exit))
    }

    /// Compile enum variant pattern: State::Running, Color::Rgb(r, g, b)
    ///
    /// Contract (same as compile_pattern_check):
    /// - INPUT: scrutinee on stack top
    /// - SUCCESS: scrutinee consumed, True on stack, pattern vars added as locals
    /// - FAILURE: scrutinee consumed, stack clean, jumps to returned offset
    fn compile_enum_variant_pattern(
        &mut self,
        enum_name: &crate::ast::Identifier,
        variant_name: &crate::ast::Identifier,
        args: &[Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        use crate::bytecode::Opcode;

        // Stack: [copy] (the enum value)

        // Dup for check (CheckEnumVariant will consume one copy)
        self.bytecode.emit(Opcode::Dup, span);
        // Stack: [copy, dup]

        // Push enum_name and variant_name
        let enum_name_idx = self
            .bytecode
            .add_constant(Value::string(enum_name.name.clone()));
        self.bytecode.emit(Opcode::Constant, span);
        self.bytecode.emit_u16(enum_name_idx);
        // Stack: [copy, dup, enum_name]

        let variant_name_idx = self
            .bytecode
            .add_constant(Value::string(variant_name.name.clone()));
        self.bytecode.emit(Opcode::Constant, span);
        self.bytecode.emit_u16(variant_name_idx);
        // Stack: [copy, dup, enum_name, variant_name]

        // CheckEnumVariant: pops (dup, enum_name, variant_name), pushes bool
        self.bytecode.emit(Opcode::CheckEnumVariant, span);
        // Stack: [copy, bool]

        // Jump if variant doesn't match
        self.bytecode.emit(Opcode::JumpIfFalse, span);
        let variant_fail = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);
        // Stack (on success path): [copy]

        if args.is_empty() {
            // Unit enum variant (no data to match)
            // Pop the copy since we don't need it
            self.bytecode.emit(Opcode::Pop, span);
            // Push True for success
            self.bytecode.emit(Opcode::True, span);

            // Jump over failure code
            self.bytecode.emit(Opcode::Jump, span);
            let success_exit = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF);

            // Failure path: pop the copy and jump to fail
            self.bytecode.patch_jump(variant_fail);
            self.bytecode.emit(Opcode::Pop, span);

            // Emit fail exit jump
            self.bytecode.emit(Opcode::Jump, span);
            let fail_exit = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF);

            // Success exit
            self.bytecode.patch_jump(success_exit);

            Ok(Some(fail_exit))
        } else {
            // Tuple enum variant - need to extract data and match args
            // ExtractEnumData: pops EnumValue, pushes Array of data
            self.bytecode.emit(Opcode::ExtractEnumData, span);
            // Stack: [data_array]

            // Now we treat this like an array pattern match
            // Store data array to temp global
            let data_global_name = "$match_enum_data";
            let data_name_idx = self.bytecode.add_constant(Value::string(data_global_name));

            self.bytecode.emit(Opcode::SetGlobal, span);
            self.bytecode.emit_u16(data_name_idx);
            self.bytecode.emit(Opcode::Pop, span);
            // Stack: [] (data array is in temp global)

            // Check data length matches arg count
            self.bytecode.emit(Opcode::GetGlobal, span);
            self.bytecode.emit_u16(data_name_idx);
            self.bytecode.emit(Opcode::GetArrayLen, span);

            let expected_len = args.len() as f64;
            let len_const_idx = self.bytecode.add_constant(Value::Number(expected_len));
            self.bytecode.emit(Opcode::Constant, span);
            self.bytecode.emit_u16(len_const_idx);
            self.bytecode.emit(Opcode::Equal, span);

            self.bytecode.emit(Opcode::JumpIfFalse, span);
            let len_fail = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF);

            // Match each argument pattern against data elements
            let mut inner_fails = Vec::new();

            for (i, arg_pattern) in args.iter().enumerate() {
                // Get element from data array
                self.bytecode.emit(Opcode::GetGlobal, span);
                self.bytecode.emit_u16(data_name_idx);

                let idx_const = self.bytecode.add_constant(Value::Number(i as f64));
                self.bytecode.emit(Opcode::Constant, span);
                self.bytecode.emit_u16(idx_const);
                self.bytecode.emit(Opcode::GetIndex, span);
                // Stack: [element_i]

                // Recursively match the argument pattern
                if let Some(fail_jump) =
                    self.compile_pattern_check(arg_pattern, span, locals_before)?
                {
                    inner_fails.push(fail_jump);
                }
                // Stack after success: [True] (or with locals added)

                // Pop the True from sub-pattern (we'll emit our own at the end)
                self.bytecode.emit(Opcode::Pop, span);
            }

            // All patterns matched - emit success True
            self.bytecode.emit(Opcode::True, span);

            // Jump over failure paths
            self.bytecode.emit(Opcode::Jump, span);
            let success_exit = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF);

            // --- Failure paths ---
            //
            // H-297: Different failure points have different stack states:
            // - variant_fail: stack is [copy] (CheckEnumVariant failed, need to pop)
            // - len_fail: stack is [] (data extracted, length comparison done)
            // - inner_fails: stack is [] (sub-pattern matching done)
            //
            // We need separate handlers for variant_fail vs the others.

            // len_fail and inner_fails can share the same handler (stack is [])
            self.bytecode.patch_jump(len_fail);
            for fail_jump in inner_fails {
                self.bytecode.patch_jump(fail_jump);
            }
            // Jump to common fail exit
            self.bytecode.emit(Opcode::Jump, span);
            let len_inner_fail_exit = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF);

            // variant_fail handler: stack is [copy], need to pop it first
            self.bytecode.patch_jump(variant_fail);
            self.bytecode.emit(Opcode::Pop, span);
            // Fall through to common fail exit (will be patched below)

            // Common fail exit point
            self.bytecode.patch_jump(len_inner_fail_exit);
            self.bytecode.emit(Opcode::Jump, span);
            let fail_exit = self.bytecode.current_offset();
            self.bytecode.emit_u16(0xFFFF);

            // Success exit
            self.bytecode.patch_jump(success_exit);

            Ok(Some(fail_exit))
        }
    }

    /// Compile OR pattern: pat1 | pat2 | pat3
    ///
    /// Contract (same as compile_pattern_check):
    /// - INPUT: scrutinee on stack top
    /// - SUCCESS: scrutinee consumed, True on stack, pattern vars added as locals
    /// - FAILURE: scrutinee consumed, stack clean, jumps to returned offset
    fn compile_or_pattern(
        &mut self,
        alternatives: &[crate::ast::Pattern],
        span: Span,
        locals_before: usize,
    ) -> Result<Option<usize>, Vec<Diagnostic>> {
        // For each alternative except last: Dup scrutinee, try sub-pattern
        // On success: jump to success_exit (pop extra dup)
        // On failure: try next alternative
        // Last alternative: no Dup, result determines overall success/failure

        let mut success_jumps: Vec<usize> = Vec::new();

        for (i, alt) in alternatives.iter().enumerate() {
            let is_last = i == alternatives.len() - 1;

            if !is_last {
                // Dup scrutinee for this alternative (pattern check consumes it)
                self.bytecode.emit(Opcode::Dup, span);
            }

            // Try this sub-pattern
            let sub_failed_jump = self.compile_pattern_check(alt, span, locals_before)?;

            if !is_last {
                // Sub-pattern succeeded: pop True, emit jump to success block
                self.bytecode.emit(Opcode::Pop, span);
                // Push True for overall match success
                self.bytecode.emit(Opcode::True, span);
                self.bytecode.emit(Opcode::Jump, span);
                success_jumps.push(self.bytecode.current_offset());
                self.bytecode.emit_u16(0xFFFF);

                // Sub-pattern failed: patch its fail jump here to try next alt
                if let Some(failed_jump) = sub_failed_jump {
                    self.bytecode.patch_jump(failed_jump);
                }
            } else {
                // Last alternative: its result IS the OR result
                // If failed, its fail jump is our overall fail jump
                // Success exit patches below
                self.bytecode.emit(Opcode::Jump, span);
                let last_success_exit = self.bytecode.current_offset();
                self.bytecode.emit_u16(0xFFFF);

                // Fail exit for last alt (and overall fail)
                let overall_fail_jump = if let Some(failed_jump) = sub_failed_jump {
                    // Patch last alt failure to emit overall fail jump
                    self.bytecode.patch_jump(failed_jump);
                    self.bytecode.emit(Opcode::Jump, span);
                    let fail_exit = self.bytecode.current_offset();
                    self.bytecode.emit_u16(0xFFFF);
                    // Patch success exits from earlier alternatives here
                    for sj in &success_jumps {
                        self.bytecode.patch_jump(*sj);
                    }
                    self.bytecode.patch_jump(last_success_exit);
                    Some(fail_exit)
                } else {
                    // Last alt always matches (wildcard/variable) — overall always succeeds
                    for sj in &success_jumps {
                        self.bytecode.patch_jump(*sj);
                    }
                    self.bytecode.patch_jump(last_success_exit);
                    None
                };

                return Ok(overall_fail_jump);
            }
        }

        // Should never reach here (alternatives is non-empty, last handled above)
        Ok(None)
    }

    /// Compile try expression (error propagation operator ?)
    ///
    /// Desugars to match-based early return:
    /// ```atlas
    /// value?
    /// // becomes:
    /// match value {
    ///     Ok(v) => v,
    ///     Err(e) => return Err(e)
    /// }
    /// ```
    fn compile_try(&mut self, try_expr: &TryExpr) -> Result<(), Vec<Diagnostic>> {
        use crate::ast::TryTargetKind;

        // 1. Compile the expression being tried
        self.compile_expr(&try_expr.expr)?;

        // Determine target kind (set by typechecker, default to Result for backwards compat)
        let target_kind = try_expr
            .target_kind
            .borrow()
            .unwrap_or(TryTargetKind::Result);

        // 2. Duplicate the value for pattern matching
        self.bytecode.emit(Opcode::Dup, try_expr.span);

        // 3. Check if it's the success variant (Ok for Result, Some for Option)
        let (check_opcode, extract_opcode) = match target_kind {
            TryTargetKind::Result => (Opcode::IsResultOk, Opcode::ExtractResultValue),
            TryTargetKind::Option => (Opcode::IsOptionSome, Opcode::ExtractOptionValue),
        };
        self.bytecode.emit(check_opcode, try_expr.span);

        // 4. Jump to error/none handling if false
        self.bytecode.emit(Opcode::JumpIfFalse, try_expr.span);
        let err_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // 5. Success path: extract the inner value
        self.bytecode.emit(extract_opcode, try_expr.span);

        // Skip error handling
        self.bytecode.emit(Opcode::Jump, try_expr.span);
        let ok_skip = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // 6. Error/None path: value is still on stack from Dup, return it
        self.bytecode.patch_jump(err_jump);
        self.bytecode.emit(Opcode::Return, try_expr.span);

        // 7. Patch success skip jump
        self.bytecode.patch_jump(ok_skip);

        Ok(())
    }

    /// Compile an anonymous function expression (`fn(params) { body }` or `(params) => expr`).
    ///
    /// Mirrors `compile_nested_function` in stmt.rs, but:
    /// - Uses a synthetic name (`__anon_N`)
    /// - Handles `Expr::Block` body (block form) and any other `Expr` body (arrow form)
    /// - Does NOT store the result in a local or global — value stays on the stack
    fn compile_anon_fn(
        &mut self,
        params: &[Param],
        body: &Expr,
        span: Span,
    ) -> Result<(), Vec<Diagnostic>> {
        let anon_name = format!("__anon_{}", self.next_func_id);
        self.next_func_id += 1;

        // Jump over the function body so it isn't executed at definition time.
        self.bytecode.emit(Opcode::Jump, span);
        let skip_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // placeholder

        let function_offset = self.bytecode.current_offset();

        // --- Compile function body with upvalue tracking ---
        let old_scope = self.scope_depth;
        let local_base = self.locals.len();
        self.scope_depth += 1;

        let prev_watermark = std::mem::replace(&mut self.locals_watermark, local_base);

        for param in params {
            self.push_local(Local {
                name: param.name.name.clone(),
                depth: self.scope_depth,
                mutable: true,
                scoped_name: None,
                drop_type: None, // params: caller owns lifetime
            });
        }

        let prev_local_base = std::mem::replace(&mut self.current_function_base, local_base);

        self.upvalue_stack.push(UpvalueContext {
            parent_base: prev_local_base,
            captures: Vec::new(),
        });

        // Body dispatch: block form vs arrow form
        match body {
            Expr::Block(block) => {
                self.compile_block(block)?;
                // If block has tail expression, it's the implicit return value
                if let Some(tail) = &block.tail_expr {
                    self.compile_expr(tail)?;
                } else {
                    // No tail expression = implicit null return
                    self.bytecode.emit(Opcode::Null, span);
                }
                self.bytecode.emit(Opcode::Return, span);
            }
            _ => {
                // Arrow form: expression is the return value
                self.compile_expr(body)?;
                self.bytecode.emit(Opcode::Return, span);
            }
        }

        let upvalue_ctx = self.upvalue_stack.pop().ok_or_else(|| {
            vec![INTERNAL_ERROR
                .emit(span)
                .arg("detail", "missing upvalue context")
                .build()]
        })?;
        let upvalues = upvalue_ctx.captures;

        self.current_function_base = prev_local_base;
        let total_local_count = self.locals_watermark - local_base;
        self.locals_watermark = prev_watermark;

        self.scope_depth = old_scope;
        self.locals.truncate(local_base);

        // Patch the skip jump to land after the function body.
        self.bytecode.patch_jump(skip_jump);

        // --- Definition site ---
        let n_upvalues = upvalues.len();

        let func_ref = crate::value::FunctionRef {
            name: anon_name,
            arity: params.len(),
            bytecode_offset: function_offset,
            local_count: total_local_count,
            param_ownership: params.iter().map(|p| p.ownership.clone()).collect(),
            param_names: params.iter().map(|p| p.name.name.clone()).collect(),
            return_ownership: None,
            is_async: false,
        };
        let const_idx = self
            .bytecode
            .add_constant(crate::value::Value::Function(func_ref));

        if n_upvalues == 0 {
            // No upvalues: push the function value directly
            self.bytecode.emit(Opcode::Constant, span);
            self.bytecode.emit_u16(const_idx);
        } else {
            // Push each captured value, then MakeClosure
            for (_, capture) in &upvalues {
                match capture {
                    UpvalueCapture::Local(abs_local_idx) => {
                        let outer_rel_idx = *abs_local_idx - prev_local_base;
                        self.bytecode.emit(Opcode::GetLocal, span);
                        self.bytecode.emit_u16(outer_rel_idx as u16);
                    }
                    UpvalueCapture::Upvalue(parent_upvalue_idx) => {
                        self.bytecode.emit(Opcode::GetUpvalue, span);
                        self.bytecode.emit_u16(*parent_upvalue_idx as u16);
                    }
                }
            }
            self.bytecode.emit(Opcode::MakeClosure, span);
            self.bytecode.emit_u16(const_idx);
            self.bytecode.emit_u16(n_upvalues as u16);
        }

        Ok(())
    }

    /// Compile a block expression - creates a new scope and returns tail expression value
    pub(super) fn compile_block_expr(&mut self, block: &Block) -> Result<(), Vec<Diagnostic>> {
        let old_scope = self.scope_depth;
        let local_base = self.locals.len();
        self.scope_depth += 1;

        if let Some(tail) = &block.tail_expr {
            for stmt in &block.statements {
                self.compile_stmt(stmt)?;
            }
            self.compile_expr(tail)?;
        } else if let Some((last, rest)) = block.statements.split_last() {
            for stmt in rest {
                self.compile_stmt(stmt)?;
            }
            self.compile_stmt_as_value(last, block.span)?;
        } else {
            self.bytecode.emit(Opcode::Null, block.span);
        }

        // B37-P02: Emit drop calls for locals going out of scope (LIFO order)
        self.emit_drops_for_scope(local_base, self.locals.len(), block.span);

        // Pop locals created in this scope AFTER evaluating tail expression,
        // while preserving the resulting value on the stack.
        let locals_to_pop = self.locals.len() - local_base;
        if locals_to_pop > 0 {
            let result_slot = local_base - self.current_function_base;
            self.bytecode.emit(Opcode::SetLocal, block.span);
            self.bytecode.emit_u16(result_slot as u16);
            self.bytecode.emit(Opcode::Pop, block.span);
            for _ in 1..locals_to_pop {
                self.bytecode.emit(Opcode::Pop, block.span);
            }
        }

        self.locals.truncate(local_base);
        self.scope_depth = old_scope;

        Ok(())
    }
}
