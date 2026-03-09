//! Expression type checking

use crate::ast::*;
use crate::diagnostic::error_codes;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::typechecker::suggestions;
use crate::typechecker::TypeChecker;
use crate::types::{StructuralMemberType, Type, TypeParamDef, ANY_TYPE_PARAM};
use std::collections::{HashMap, HashSet};

/// Resolve the return type for a static namespace method call (Json.parse, Math.sqrt, etc.)
fn resolve_namespace_return_type(ns: &str, method: &str) -> Type {
    match (ns, method) {
        // Json namespace
        ("Json", "parse") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::JsonValue, Type::String],
        },
        ("Json", "stringify") => Type::String,
        ("Json", "isValid") => Type::Bool,
        ("Json", "prettify") => Type::String,
        // Math namespace
        (
            "Math",
            "abs" | "floor" | "ceil" | "round" | "min" | "max" | "pow" | "sign" | "random",
        ) => Type::Number,
        ("Math", "sqrt" | "clamp" | "log" | "sin" | "cos" | "tan") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Number, Type::String],
        },
        // Env namespace
        ("Env", "get") => Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::String],
        },
        ("Env", "set" | "unset") => Type::Null,
        // File namespace
        ("File", "read") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::String, Type::String],
        },
        ("File", "write" | "append" | "createDir" | "removeDir" | "remove") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Null, Type::String],
        },
        ("File", "exists") => Type::Bool,
        // Process namespace
        ("Process", "cwd") => Type::String,
        ("Process", "pid") => Type::Number,
        ("Process", "env") => Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::String],
        },
        // Path namespace
        (
            "Path",
            "join" | "dirname" | "basename" | "extension" | "normalize" | "absolute" | "parent"
            | "canonical" | "homedir" | "cwd" | "tempdir" | "separator",
        ) => Type::String,
        ("Path", "exists" | "isAbsolute" | "isRelative") => Type::Bool,
        // DateTime namespace
        (
            "DateTime",
            "now" | "fromTimestamp" | "fromComponents" | "parseIso" | "parse" | "parseRfc3339"
            | "parseRfc2822" | "utc",
        ) => Type::Unknown,
        // Regex namespace
        ("Regex", "new") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Unknown, Type::String],
        },
        ("Regex", "test" | "isMatch") => Type::Bool,
        ("Regex", "find") => Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::String],
        },
        ("Regex", "findAll") => Type::Array(Box::new(Type::String)),
        ("Regex", "replace" | "replaceAll" | "escape") => Type::String,
        ("Regex", "split") => Type::Array(Box::new(Type::String)),
        // Crypto namespace
        ("Crypto", "sha256" | "sha512") => Type::String,
        // Http namespace
        ("Http", "get" | "post" | "put" | "delete" | "patch" | "request") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Unknown, Type::String],
        },
        // Net namespace
        ("Net", "tcpConnect" | "tcpListen") => Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Unknown, Type::String],
        },
        // Io namespace
        ("Io", "readLine" | "readLinePrompt") => Type::String,
        // Default: unknown for unrecognized combinations
        _ => Type::Unknown,
    }
}

impl<'a> TypeChecker<'a> {
    /// Check an expression and return its type
    pub(super) fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Number(_) => Type::Number,
                Literal::String(_) => Type::String,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Null,
            },
            Expr::TemplateString { parts, .. } => {
                for part in parts {
                    if let TemplatePart::Expression(expr) = part {
                        self.check_expr(expr);
                    }
                }
                Type::String
            }
            Expr::Identifier(id) => {
                // Track that this symbol was used
                self.used_symbols.insert(id.name.clone());

                // AT3053: use-after-own — variable was moved into an `own` call
                if self.moved_vars.contains(&id.name) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            error_codes::USE_AFTER_OWN,
                            format!(
                                "use of moved value `{}`: value was transferred via `own` and is no longer valid",
                                id.name
                            ),
                            id.span,
                        )
                        .with_label("value already moved")
                        .with_help(format!(
                            "after passing `{}` to an `own` parameter, the binding is invalidated.\n\
                             To keep using the value, change the parameter annotation:\n\
                             • `borrow {}` — read-only access, caller retains ownership\n\
                             • `share {}`  — both hold valid refs simultaneously",
                            id.name, id.name, id.name
                        )),
                    );
                }

                if id.name == "None" {
                    return Type::Generic {
                        name: "Option".to_string(),
                        type_args: vec![Type::any_placeholder()],
                    };
                }

                if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                    symbol.ty.clone()
                } else {
                    // Symbol not found - may be a builtin or undefined variable
                    // Binder should have caught undefined variables, so this is likely a builtin
                    Type::Unknown
                }
            }
            Expr::Binary(binary) => self.check_binary(binary),
            Expr::Unary(unary) => self.check_unary(unary),
            Expr::Call(call) => self.check_call(call),
            Expr::Index(index) => self.check_index(index),
            Expr::ArrayLiteral(arr) => self.check_array_literal(arr),
            Expr::Group(group) => self.check_expr(&group.expr),
            Expr::Match(match_expr) => self.check_match(match_expr),
            Expr::Member(member) => self.check_member(member),
            Expr::Try(try_expr) => self.check_try(try_expr),
            Expr::AnonFn {
                params,
                return_type,
                body,
                span,
            } => self.check_anon_fn(params, return_type.as_ref(), body, *span),
            Expr::Block(block) => {
                // H-115: if/else as expression — parser wraps `if cond { a } else { b }`
                // as Block { statements: [Stmt::If(...)], tail_expr: None }. Infer type from
                // both branches when both have tail expressions.
                if block.tail_expr.is_none()
                    && block.statements.len() == 1
                    && block.tail_expr.is_none()
                {
                    if let Stmt::If(if_stmt) = &block.statements[0] {
                        if let Some(else_block) = &if_stmt.else_block {
                            if let (Some(then_tail), Some(else_tail)) =
                                (&if_stmt.then_block.tail_expr, &else_block.tail_expr)
                            {
                                self.enter_scope();
                                let then_type = self.check_expr(then_tail);
                                self.exit_scope();
                                self.enter_scope();
                                let else_type = self.check_expr(else_tail);
                                self.exit_scope();
                                if then_type == else_type {
                                    return then_type;
                                }
                                return Type::union(vec![then_type, else_type]);
                            }
                        }
                    }
                }
                self.enter_scope();
                for stmt in &block.statements {
                    self.check_statement(stmt);
                }
                // If the block ends with a return/break/continue, its type is Never
                // (the bottom type — compatible with any other type in unification).
                // An empty block or a block of pure statements (no tail expr) has type Void.
                let block_type = match block.statements.last() {
                    Some(Stmt::Return(_)) => Type::Never,
                    _ => Type::Void,
                };
                self.exit_scope();
                block_type
            }
            Expr::ObjectLiteral(obj) => {
                let mut members = Vec::with_capacity(obj.entries.len());
                for entry in &obj.entries {
                    let value_type = self.check_expr(&entry.value);
                    members.push(StructuralMemberType {
                        name: entry.key.name.clone(),
                        ty: value_type,
                    });
                }
                Type::Structural { members }
            }
            Expr::StructExpr(struct_expr) => {
                let struct_name = struct_expr.name.name.as_str();
                let struct_type = self.resolve_struct_type(struct_name, struct_expr.name.span);

                if let Some(struct_type) = struct_type {
                    let members = match struct_type.normalized() {
                        Type::Structural { members } => members,
                        _ => Vec::new(),
                    };

                    let mut member_types: HashMap<String, Type> = HashMap::new();
                    for member in members {
                        member_types.insert(member.name.clone(), member.ty.clone());
                    }

                    let mut seen_fields: HashSet<String> = HashSet::new();
                    for field in &struct_expr.fields {
                        // AT3054: borrow param cannot escape into a struct field.
                        // No exemption for primitives — explicit `borrow` annotation means
                        // the value must not outlive the function scope.
                        if let Expr::Identifier(id) = &field.value {
                            let ownership = self
                                .current_fn_param_ownerships
                                .get(&id.name)
                                .cloned()
                                .flatten();
                            if ownership == Some(OwnershipAnnotation::Borrow) {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        error_codes::BORROW_ESCAPE,
                                        format!(
                                            "cannot use `borrow` parameter `{}` as a struct field: \
                                             borrows cannot outlive their scope",
                                            id.name
                                        ),
                                        field.span,
                                    )
                                    .with_label("borrow escapes into struct")
                                    .with_help(
                                        "copy the value or use a computation result instead of \
                                         storing a `borrow` parameter directly in a struct",
                                    ),
                                );
                            }
                        }
                        let value_type = self.check_expr(&field.value);
                        if let Some(expected_type) = member_types.get(&field.name.name) {
                            if !seen_fields.insert(field.name.name.clone()) {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3001",
                                        format!(
                                            "Duplicate field '{}' in struct '{}'",
                                            field.name.name, struct_expr.name.name
                                        ),
                                        field.span,
                                    )
                                    .with_label("duplicate field initializer")
                                    .with_help("each struct field can only be initialized once"),
                                );
                            }
                            // H-162: empty array literal [] in struct field — skip mismatch
                            // if the declared field type is an array. The [] is typed as ?[]
                            // by check_array_literal but is valid when the field type is T[].
                            let is_empty_array = matches!(
                                &field.value,
                                Expr::ArrayLiteral(a) if a.elements.is_empty()
                            );
                            let expected_is_array =
                                matches!(expected_type.normalized(), Type::Array(_));
                            if is_empty_array && expected_is_array {
                                // Valid — empty literal assigned to typed array field
                            } else if !self.is_assignable_with_traits(&value_type, expected_type) {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3001",
                                        format!(
                                            "Type mismatch: expected {}, found {}",
                                            expected_type.display_name(),
                                            value_type.display_name()
                                        ),
                                        field.span,
                                    )
                                    .with_label(format!(
                                        "expected {}, found {}",
                                        expected_type.display_name(),
                                        value_type.display_name()
                                    ))
                                    .with_help(format!(
                                        "field '{}' must be of type {}",
                                        field.name.name,
                                        expected_type.display_name()
                                    )),
                                );
                            }
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3010",
                                    format!(
                                        "Struct '{}' has no field named '{}'",
                                        struct_expr.name.name, field.name.name
                                    ),
                                    field.span,
                                )
                                .with_label("unknown field")
                                .with_help("check the struct declaration for valid fields"),
                            );
                        }
                    }

                    for (field_name, _) in member_types {
                        if !seen_fields.contains(&field_name) {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    format!(
                                        "Missing field '{}' in struct '{}'",
                                        field_name, struct_expr.name.name
                                    ),
                                    struct_expr.span,
                                )
                                .with_label("missing field")
                                .with_help(format!("provide a value for field '{}'", field_name)),
                            );
                        }
                    }

                    struct_type
                } else {
                    let mut members = Vec::with_capacity(struct_expr.fields.len());
                    let mut seen = HashSet::new();
                    for field in &struct_expr.fields {
                        if !seen.insert(field.name.name.clone()) {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    format!(
                                        "Duplicate field '{}' in struct '{}'",
                                        field.name.name, struct_expr.name.name
                                    ),
                                    field.span,
                                )
                                .with_label("duplicate field initializer")
                                .with_help("each struct field can only be initialized once"),
                            );
                        }
                        let value_type = self.check_expr(&field.value);
                        members.push(StructuralMemberType {
                            name: field.name.name.clone(),
                            ty: value_type,
                        });
                    }
                    Type::Structural { members }
                }
            }
            Expr::Range {
                start,
                end,
                inclusive: _,
                span,
            } => self.check_range(start, end, *span),
            Expr::EnumVariant(ev) => {
                // Type check any arguments in the enum variant
                if let Some(args) = &ev.args {
                    for arg in args {
                        self.check_expr(arg);
                    }
                }
                // If the enum name is registered, return its named type so that variables
                // holding enum values don't resolve to Unknown (H-110, H-111).
                if self.enum_names.contains(&ev.enum_name.name) {
                    Type::Generic {
                        name: ev.enum_name.name.clone(),
                        type_args: vec![],
                    }
                } else {
                    Type::Unknown
                }
            }
            Expr::Await { expr, span } => {
                // AT4001: await outside async context
                if !self.in_async_context {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            error_codes::AWAIT_OUTSIDE_ASYNC,
                            "`await` used outside of an async function or top-level scope"
                                .to_string(),
                            *span,
                        )
                        .with_label("not inside an async fn or top-level")
                        .with_help(
                            "move this into an `async fn`, or restructure so the `await` appears at the top level of the script",
                        ),
                    );
                    self.check_expr(expr);
                    return Type::Unknown;
                }
                let operand_ty = self.check_expr(expr);
                let operand_norm = operand_ty.normalized();
                // AT4002: await applied to non-Future value
                match operand_norm {
                    Type::Generic {
                        ref name,
                        ref type_args,
                    } if name == "Future" => type_args.first().cloned().unwrap_or(Type::Unknown),
                    Type::Unknown => Type::Unknown, // upstream error already reported
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                error_codes::AWAIT_NON_FUTURE,
                                format!(
                                    "`await` applied to a non-Future value of type `{}`",
                                    operand_norm.display_name()
                                ),
                                *span,
                            )
                            .with_label(format!(
                                "type `{}` is not `Future<_>`",
                                operand_norm.display_name()
                            ))
                            .with_help("only values of type `Future<T>` can be awaited"),
                        );
                        Type::Unknown
                    }
                }
            }
        }
    }

    fn check_range(
        &mut self,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
        _span: Span,
    ) -> Type {
        let check_bound = |this: &mut Self, bound: &Option<Box<Expr>>| {
            if let Some(expr) = bound {
                let bound_type = this.check_expr(expr);
                let bound_norm = bound_type.normalized();
                if bound_norm != Type::Number {
                    this.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Range bound must be number, found {}",
                                bound_type.display_name()
                            ),
                            expr.span(),
                        )
                        .with_label("type mismatch")
                        .with_help("range bounds must be numbers"),
                    );
                }
            }
        };

        check_bound(self, start);
        check_bound(self, end);

        Type::Range
    }

    fn check_call_against_signature(
        &mut self,
        call: &CallExpr,
        callee_type: &Type,
        type_params: &[TypeParamDef],
        params: &[Type],
        return_type: &Type,
    ) -> Type {
        self.check_call_against_signature_inner(
            call,
            callee_type,
            type_params,
            params,
            return_type,
            &[],
        )
    }

    fn check_call_against_signature_with_types(
        &mut self,
        call: &CallExpr,
        callee_type: &Type,
        type_params: &[TypeParamDef],
        params: &[Type],
        return_type: &Type,
        pre_evaluated: &[Type],
    ) -> Type {
        self.check_call_against_signature_inner(
            call,
            callee_type,
            type_params,
            params,
            return_type,
            pre_evaluated,
        )
    }

    fn check_call_against_signature_inner(
        &mut self,
        call: &CallExpr,
        callee_type: &Type,
        type_params: &[TypeParamDef],
        params: &[Type],
        return_type: &Type,
        pre_evaluated: &[Type],
    ) -> Type {
        // Check argument count
        if call.args.len() != params.len() {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3005",
                    format!(
                        "Function expects {} argument{}, found {}",
                        params.len(),
                        if params.len() == 1 { "" } else { "s" },
                        call.args.len()
                    ),
                    call.span,
                )
                .with_label("argument count mismatch")
                .with_help(suggestions::suggest_arity_fix(
                    params.len(),
                    call.args.len(),
                    callee_type,
                )),
            );
        }

        // If function has type parameters, use type inference
        if !type_params.is_empty() {
            return self.check_call_with_inference(type_params, params, return_type, call);
        }

        // Non-generic function - check argument types normally.
        // Use pre-evaluated types when available to avoid double-evaluation.
        self.check_arg_types(call, params, pre_evaluated);

        return_type.clone()
    }

    /// Check argument types against expected param types.
    /// `pre_evaluated` may be empty (args evaluated fresh) or contain pre-evaluated types
    /// (reuse to avoid double-evaluation and duplicate diagnostics).
    fn check_arg_types(&mut self, call: &CallExpr, params: &[Type], pre_evaluated: &[Type]) {
        for (i, arg) in call.args.iter().enumerate() {
            let arg_type = if let Some(t) = pre_evaluated.get(i) {
                t.clone()
            } else {
                self.check_expr(arg)
            };
            if let Some(expected_type) = params.get(i) {
                if self.is_hashmap_new_call(arg) && self.is_typed_hashmap(expected_type) {
                    continue;
                }
                if expected_type.normalized() == Type::Unknown {
                    continue;
                }
                // Skip type check when argument type is Unknown (e.g., returned from a
                // static namespace call like Json.parse() or Env.get() whose return type
                // isn't tracked by the typechecker yet).
                if arg_type.normalized() == Type::Unknown {
                    continue;
                }
                if !self.is_assignable_with_traits(&arg_type, expected_type) {
                    let help = suggestions::suggest_type_mismatch(expected_type, &arg_type)
                        .unwrap_or_else(|| {
                            format!(
                                "argument {} must be of type {}",
                                i + 1,
                                expected_type.display_name()
                            )
                        });
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Argument {} type mismatch: expected {}, found {}",
                                i + 1,
                                expected_type.display_name(),
                                arg_type.display_name()
                            ),
                            arg.span(),
                        )
                        .with_label(format!(
                            "expected {}, found {}",
                            expected_type.display_name(),
                            arg_type.display_name()
                        ))
                        .with_help(help),
                    );
                }
            }
        }
    }

    /// Check ownership constraints at a call site.
    ///
    /// Accepts pre-evaluated argument types to avoid double-evaluating expressions.
    /// Validates:
    /// - `own` param: warn if argument is a `borrow`-annotated param of the caller
    /// - `shared` param: error if argument type is not `share<T>`
    /// - `borrow` param: always accepted, no diagnostic
    fn check_call_ownership(&mut self, call: &CallExpr, callee_name: &str, arg_types: &[Type]) {
        let ownerships = match self.fn_ownership_registry.get(callee_name) {
            Some(entry) => entry.0.clone(),
            None => return,
        };
        for (i, arg) in call.args.iter().enumerate() {
            let param_ownership = match ownerships.get(i) {
                Some(o) => o.clone(),
                None => continue,
            };
            match param_ownership {
                Some(OwnershipAnnotation::Own) => {
                    // Warn if argument is a `borrow`-annotated parameter of the enclosing function
                    if let Expr::Identifier(id) = arg {
                        let caller_ownership = self
                            .current_fn_param_ownerships
                            .get(&id.name)
                            .cloned()
                            .flatten();
                        if caller_ownership == Some(OwnershipAnnotation::Borrow) {
                            self.diagnostics.push(
                                Diagnostic::warning_with_code(
                                    error_codes::BORROW_TO_OWN,
                                    format!(
                                        "passing borrowed parameter `{}` to `own` parameter — \
                                         ownership cannot transfer",
                                        id.name
                                    ),
                                    arg.span(),
                                )
                                .with_help("pass an owned value instead of a `borrow` parameter"),
                            );
                        } else if caller_ownership == Some(OwnershipAnnotation::Share) {
                            // AT3055: share param passed to own — cannot transfer ownership of
                            // something that is shared (caller still holds a valid ref)
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    error_codes::SHARE_VIOLATION,
                                    format!(
                                        "cannot pass `share` parameter `{}` to `own` parameter: \
                                         ownership cannot transfer from a shared reference",
                                        id.name
                                    ),
                                    arg.span(),
                                )
                                .with_help(
                                    "share params are held by both caller and callee — \
                                     ownership cannot be transferred to a third party",
                                ),
                            );
                        } else {
                            // Mark variable as moved — any subsequent use triggers AT3053
                            self.moved_vars.insert(id.name.clone());
                        }
                    }
                }
                Some(OwnershipAnnotation::Share) => {
                    // A share-annotated param of the enclosing function is implicitly a shared
                    // reference — allow passing it to another share param without AT3028.
                    let arg_is_share_param = if let Expr::Identifier(id) = arg {
                        self.current_fn_param_ownerships
                            .get(&id.name)
                            .cloned()
                            .flatten()
                            == Some(OwnershipAnnotation::Share)
                    } else {
                        false
                    };
                    // Error if argument type is not `share<T>` and not a share param
                    if !arg_is_share_param {
                        if let Some(arg_type) = arg_types.get(i) {
                            let is_shared = matches!(
                                arg_type,
                                Type::Generic { name, .. } if name == "share"
                            );
                            if !is_shared {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        error_codes::NON_SHARED_TO_SHARED,
                                        format!(
                                            "expected `share<T>` value for `share` parameter, \
                                             found `{}`",
                                            arg_type.display_name()
                                        ),
                                        arg.span(),
                                    )
                                    .with_help(
                                        "wrap the value in a shared reference before passing it",
                                    ),
                                );
                            }
                        }
                    }
                }
                Some(OwnershipAnnotation::Borrow) => {
                    // borrow params accept any value — no diagnostic
                }
                None => {
                    // Unannotated param: warn if the argument type is non-Copy (Move type)
                    if let Some(arg_type) = arg_types.get(i) {
                        if matches!(arg_type.normalized(), Type::TypeParameter { .. }) {
                            continue;
                        }
                        if self.is_move_type(arg_type) {
                            self.diagnostics.push(
                                Diagnostic::warning_with_code(
                                    error_codes::MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION,
                                    format!(
                                        "Type '{}' is not Copy — consider annotating with \
                                         'own' or 'borrow' to clarify ownership intent",
                                        arg_type.display_name()
                                    ),
                                    arg.span(),
                                )
                                .with_help(
                                    "non-Copy types should use explicit 'own' or 'borrow' \
                                     ownership annotations",
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    /// Emit AT3052 if either operand of a binary expression is an identifier, providing
    /// context that the inferred type of that variable is incompatible at this use site.
    fn maybe_emit_at3052_for_binary(
        &mut self,
        binary: &BinaryExpr,
        left_type: &Type,
        right_type: &Type,
    ) {
        // Emit for the left side if it is an identifier
        if matches!(*binary.left, Expr::Identifier(_)) {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3052",
                    format!(
                        "inferred type '{}' is incompatible with '{}' at this use site",
                        left_type.display_name(),
                        right_type.display_name()
                    ),
                    binary.left.span(),
                )
                .with_label(format!("has inferred type '{}'", left_type.display_name()))
                .with_help("add an explicit type annotation to clarify the intended type"),
            );
        } else if matches!(*binary.right, Expr::Identifier(_)) {
            // Emit for the right side
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3052",
                    format!(
                        "inferred type '{}' is incompatible with '{}' at this use site",
                        right_type.display_name(),
                        left_type.display_name()
                    ),
                    binary.right.span(),
                )
                .with_label(format!("has inferred type '{}'", right_type.display_name()))
                .with_help("add an explicit type annotation to clarify the intended type"),
            );
        }
    }

    /// Check a binary expression
    fn check_binary(&mut self, binary: &BinaryExpr) -> Type {
        let left_type = self.check_expr(&binary.left);
        let right_type = self.check_expr(&binary.right);
        let left_norm = left_type.normalized();
        let right_norm = right_type.normalized();

        let left_is_any =
            matches!(left_norm, Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM);
        let right_is_any =
            matches!(right_norm, Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM);
        if left_is_any || right_is_any {
            return match binary.op {
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::And
                | BinaryOp::Or => Type::Bool,
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    Type::any_placeholder()
                }
            };
        }

        // Skip type checking if either side is Unknown (error recovery)
        if left_norm == Type::Unknown || right_norm == Type::Unknown {
            return Type::Unknown;
        }

        match binary.op {
            BinaryOp::Add => {
                if let Some(array_type) = self.array_concat_result(&left_norm, &right_norm) {
                    return array_type;
                }
                if self.all_union_pairs_valid(&left_norm, &right_norm, |a, b| {
                    (*a == Type::Number && *b == Type::Number)
                        || (*a == Type::String && *b == Type::String)
                }) {
                    if left_norm == Type::String || right_norm == Type::String {
                        Type::String
                    } else {
                        Type::Number
                    }
                } else {
                    let help = suggestions::suggest_binary_operator_fix("+", &left_type, &right_type)
                        .unwrap_or_else(|| "ensure both operands are numbers (for addition), strings (for concatenation), or arrays with compatible element types".to_string());
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "'+' requires matching types, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label(format!(
                            "found {} and {}",
                            left_type.display_name(),
                            right_type.display_name()
                        ))
                        .with_help(help),
                    );
                    self.maybe_emit_at3052_for_binary(binary, &left_type, &right_type);
                    Type::Unknown
                }
            }
            BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if self.all_union_pairs_valid(&left_norm, &right_norm, |a, b| {
                    *a == Type::Number && *b == Type::Number
                }) {
                    Type::Number
                } else {
                    let op_str = match binary.op {
                        BinaryOp::Sub => "-",
                        BinaryOp::Mul => "*",
                        BinaryOp::Div => "/",
                        BinaryOp::Mod => "%",
                        _ => unreachable!(),
                    };
                    let help =
                        suggestions::suggest_binary_operator_fix(op_str, &left_type, &right_type)
                            .unwrap_or_else(|| {
                                format!(
                                    "'{op_str}' requires both operands to be numbers; found {} and {}. Use num() to convert strings.",
                                    left_type.display_name(),
                                    right_type.display_name()
                                )
                            });
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "'{op_str}' requires number operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch")
                        .with_help(help),
                    );
                    Type::Unknown
                }
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                // Equality requires same types
                if !self.types_overlap(&left_norm, &right_norm) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Equality comparison requires same types, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("both operands must have the same type for equality comparison"),
                    );
                }
                Type::Bool
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                let left_cmp = self.comparable_operand_type(&left_norm);
                let right_cmp = self.comparable_operand_type(&right_norm);
                if self.all_union_pairs_valid(&left_cmp, &right_cmp, |a, b| {
                    *a == Type::Number && *b == Type::Number
                }) {
                    Type::Bool
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Comparison requires number operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("comparison operators (<, <=, >, >=) only work with numbers"),
                    );
                    Type::Bool // Still return bool for error recovery
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if !self.all_union_pairs_valid(&left_norm, &right_norm, |a, b| {
                    *a == Type::Bool && *b == Type::Bool
                }) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Logical operators require bool operands, found {} and {}",
                                left_type.display_name(),
                                right_type.display_name()
                            ),
                            binary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("logical operators (and, or) only work with bool values"),
                    );
                }
                Type::Bool
            }
        }
    }

    /// Check a unary expression
    fn check_unary(&mut self, unary: &UnaryExpr) -> Type {
        let expr_type = self.check_expr(&unary.expr);
        let expr_norm = expr_type.normalized();

        if matches!(expr_norm, Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM) {
            return Type::any_placeholder();
        }

        match unary.op {
            UnaryOp::Negate => {
                if expr_norm != Type::Number {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Unary '-' requires number operand, found {}",
                                expr_type.display_name()
                            ),
                            unary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("negation (-) only works with numbers"),
                    );
                    Type::Unknown
                } else {
                    Type::Number
                }
            }
            UnaryOp::Not => {
                if expr_norm != Type::Bool {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3002",
                            format!(
                                "Unary '!' requires bool operand, found {}",
                                expr_type.display_name()
                            ),
                            unary.span,
                        )
                        .with_label("type mismatch")
                        .with_help("logical not (!) only works with bool values"),
                    );
                    Type::Unknown
                } else {
                    Type::Bool
                }
            }
        }
    }

    /// Check a function call
    fn check_call(&mut self, call: &CallExpr) -> Type {
        let callee_type = self.check_expr(&call.callee);
        let callee_norm = callee_type.normalized();

        // Extract callee name for ownership registry lookup (direct calls only)
        let callee_name = if let Expr::Identifier(id) = call.callee.as_ref() {
            Some(id.name.clone())
        } else {
            None
        };

        if let Some(ref name) = callee_name {
            // AT9000: Deprecation warning for old global stdlib names
            if let Some(replacement) = crate::method_dispatch::deprecated_global_replacement(name) {
                self.diagnostics.push(
                    Diagnostic::warning_with_code(
                        "AT9000",
                        format!("Deprecated: use {} instead of {}()", replacement, name),
                        call.span,
                    )
                    .with_label("deprecated global")
                    .with_help("Use method syntax or static namespace instead."),
                );
            }
            match name.as_str() {
                "Some" => {
                    if call.args.len() != 1 {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3023",
                                format!("Some expects 1 argument, found {}", call.args.len()),
                                call.span,
                            )
                            .with_label("wrong arity")
                            .with_help("Some requires exactly 1 argument: Some(value)"),
                        );
                        return Type::Unknown;
                    }
                    let arg_type = self.check_expr(&call.args[0]);
                    return Type::Generic {
                        name: "Option".to_string(),
                        type_args: vec![arg_type],
                    };
                }
                "None" => {
                    if !call.args.is_empty() {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3023",
                                format!("None expects 0 arguments, found {}", call.args.len()),
                                call.span,
                            )
                            .with_label("wrong arity")
                            .with_help("None requires no arguments: None"),
                        );
                        return Type::Unknown;
                    }
                    return Type::Generic {
                        name: "Option".to_string(),
                        type_args: vec![Type::any_placeholder()],
                    };
                }
                "Ok" => {
                    if call.args.len() != 1 {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3023",
                                format!("Ok expects 1 argument, found {}", call.args.len()),
                                call.span,
                            )
                            .with_label("wrong arity")
                            .with_help("Ok requires exactly 1 argument: Ok(value)"),
                        );
                        return Type::Unknown;
                    }
                    let arg_type = self.check_expr(&call.args[0]);
                    return Type::Generic {
                        name: "Result".to_string(),
                        type_args: vec![arg_type, Type::any_placeholder()],
                    };
                }
                "Err" => {
                    if call.args.len() != 1 {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3023",
                                format!("Err expects 1 argument, found {}", call.args.len()),
                                call.span,
                            )
                            .with_label("wrong arity")
                            .with_help("Err requires exactly 1 argument: Err(value)"),
                        );
                        return Type::Unknown;
                    }
                    let arg_type = self.check_expr(&call.args[0]);
                    return Type::Generic {
                        name: "Result".to_string(),
                        type_args: vec![Type::any_placeholder(), arg_type],
                    };
                }
                "hashMapNew" | "hash_map_new" => {
                    if !call.args.is_empty() {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3005",
                                format!(
                                    "hashMapNew expects 0 arguments, found {}",
                                    call.args.len()
                                ),
                                call.span,
                            )
                            .with_label("argument count mismatch")
                            .with_help("hashMapNew() takes no arguments"),
                        );
                        return Type::Unknown;
                    }
                    return Type::Generic {
                        name: "HashMap".to_string(),
                        type_args: vec![Type::any_placeholder(), Type::any_placeholder()],
                    };
                }
                "hashMapPut" | "hash_map_put" => {
                    if call.args.len() != 3 {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3005",
                                format!(
                                    "hashMapPut expects 3 arguments, found {}",
                                    call.args.len()
                                ),
                                call.span,
                            )
                            .with_label("argument count mismatch")
                            .with_help("hashMapPut(map, key, value) requires exactly 3 arguments"),
                        );
                        return Type::Unknown;
                    }

                    let map_type = self.check_expr(&call.args[0]);
                    let key_type = self.check_expr(&call.args[1]);
                    let value_type = self.check_expr(&call.args[2]);

                    let Some((expected_key, expected_value)) = self.hashmap_type_args(&map_type)
                    else {
                        let map_norm = map_type.normalized();
                        let map_is_any_or_unknown = matches!(map_norm, Type::Unknown)
                            || matches!(
                                map_norm,
                                Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM
                            );
                        let map_is_structural = matches!(map_norm, Type::Structural { .. });
                        if map_is_any_or_unknown || map_is_structural {
                            return map_type;
                        }
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "hashMapPut expects HashMap for argument 1, found {}",
                                    map_type.display_name()
                                ),
                                call.args[0].span(),
                            )
                            .with_label("type mismatch")
                            .with_help("argument 1 must be a HashMap"),
                        );
                        return Type::Unknown;
                    };

                    if !self.is_untyped_hashmap(&map_type) {
                        if !self.is_assignable_with_traits(&key_type, &expected_key) {
                            let help = suggestions::suggest_type_mismatch(&expected_key, &key_type)
                                .unwrap_or_else(|| {
                                    format!(
                                        "expected {}, found {}",
                                        expected_key.display_name(),
                                        key_type.display_name()
                                    )
                                });
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    format!(
                                        "hashMapPut key type mismatch: expected {}, found {}",
                                        expected_key.display_name(),
                                        key_type.display_name()
                                    ),
                                    call.args[1].span(),
                                )
                                .with_label("type mismatch")
                                .with_help(help),
                            );
                        }
                        if !self.is_assignable_with_traits(&value_type, &expected_value) {
                            let help =
                                suggestions::suggest_type_mismatch(&expected_value, &value_type)
                                    .unwrap_or_else(|| {
                                        format!(
                                            "expected {}, found {}",
                                            expected_value.display_name(),
                                            value_type.display_name()
                                        )
                                    });
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    format!(
                                        "hashMapPut value type mismatch: expected {}, found {}",
                                        expected_value.display_name(),
                                        value_type.display_name()
                                    ),
                                    call.args[2].span(),
                                )
                                .with_label("type mismatch")
                                .with_help(help),
                            );
                        }
                    }

                    return map_type;
                }
                "hashMapGet" | "hash_map_get" => {
                    if call.args.len() != 2 {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3005",
                                format!(
                                    "hashMapGet expects 2 arguments, found {}",
                                    call.args.len()
                                ),
                                call.span,
                            )
                            .with_label("argument count mismatch")
                            .with_help("hashMapGet(map, key) requires exactly 2 arguments"),
                        );
                        return Type::Unknown;
                    }

                    let map_type = self.check_expr(&call.args[0]);
                    let key_type = self.check_expr(&call.args[1]);

                    let Some((expected_key, expected_value)) = self.hashmap_type_args(&map_type)
                    else {
                        let map_norm = map_type.normalized();
                        let map_is_any_or_unknown = matches!(map_norm, Type::Unknown)
                            || matches!(
                                map_norm,
                                Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM
                            );
                        let map_is_structural = matches!(map_norm, Type::Structural { .. });
                        if map_is_any_or_unknown || map_is_structural {
                            return Type::Generic {
                                name: "Option".to_string(),
                                type_args: vec![Type::any_placeholder()],
                            };
                        }
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "hashMapGet expects HashMap for argument 1, found {}",
                                    map_type.display_name()
                                ),
                                call.args[0].span(),
                            )
                            .with_label("type mismatch")
                            .with_help("argument 1 must be a HashMap"),
                        );
                        return Type::Unknown;
                    };

                    if !self.is_untyped_hashmap(&map_type)
                        && !self.is_assignable_with_traits(&key_type, &expected_key)
                    {
                        let help = suggestions::suggest_type_mismatch(&expected_key, &key_type)
                            .unwrap_or_else(|| {
                                format!(
                                    "expected {}, found {}",
                                    expected_key.display_name(),
                                    key_type.display_name()
                                )
                            });
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "hashMapGet key type mismatch: expected {}, found {}",
                                    expected_key.display_name(),
                                    key_type.display_name()
                                ),
                                call.args[1].span(),
                            )
                            .with_label("type mismatch")
                            .with_help(help),
                        );
                    }

                    if self.is_untyped_hashmap(&map_type) {
                        return Type::Generic {
                            name: "Option".to_string(),
                            type_args: vec![Type::any_placeholder()],
                        };
                    }

                    return Type::Generic {
                        name: "Option".to_string(),
                        type_args: vec![expected_value],
                    };
                }
                // H-112: hashMapHas / hashSetHas return bool
                "hashMapHas" | "hash_map_has" | "hashSetHas" | "hash_set_has" => {
                    return Type::Bool;
                }
                // H-164: unwrap() returns the inner type T from Option<T> or Result<T, E>
                "unwrap" => {
                    if call.args.len() == 1 {
                        let arg_type = self.check_expr(&call.args[0]);
                        let inner = match arg_type.normalized() {
                            Type::Generic { name, type_args }
                                if (name == "Option" || name == "Result")
                                    && !type_args.is_empty() =>
                            {
                                type_args[0].clone()
                            }
                            _ => arg_type,
                        };
                        return inner;
                    }
                }
                _ => {}
            }
        }

        // Pre-evaluate arg types for ownership checking (avoids double-evaluation in check_expr
        // for the `shared` param path). check_call_against_signature re-evaluates independently.
        let arg_types_for_ownership: Vec<Type> = if callee_name.is_some() {
            call.args.iter().map(|a| self.check_expr(a)).collect()
        } else {
            Vec::new()
        };

        match &callee_norm {
            Type::Function {
                type_params,
                params,
                return_type,
            } => {
                if let Some(ref name) = callee_name {
                    self.check_call_ownership(call, name, &arg_types_for_ownership);
                }
                self.check_call_against_signature_with_types(
                    call,
                    &callee_type,
                    type_params,
                    params,
                    return_type,
                    &arg_types_for_ownership,
                )
            }
            Type::Union(members) => {
                if members.is_empty() {
                    return Type::Unknown;
                }

                let mut signature: Option<Type> = None;
                for member in members {
                    match member {
                        Type::Function { .. } => {
                            if signature.is_none() {
                                signature = Some(member.clone());
                            } else if signature.as_ref() != Some(member) {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3005",
                                        "Cannot call union of incompatible function signatures",
                                        call.span,
                                    )
                                    .with_label("ambiguous call")
                                    .with_help(
                                        "ensure all union members share the same function signature",
                                    ),
                                );
                                return Type::Unknown;
                            }
                        }
                        _ => {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3006",
                                    format!(
                                        "Cannot call non-function type {}",
                                        member.display_name()
                                    ),
                                    call.span,
                                )
                                .with_label("not callable")
                                .with_help(suggestions::suggest_not_callable(&callee_type)),
                            );
                            return Type::Unknown;
                        }
                    }
                }

                if let Some(Type::Function {
                    type_params,
                    params,
                    return_type,
                }) = signature
                {
                    return self.check_call_against_signature(
                        call,
                        &callee_type,
                        &type_params,
                        &params,
                        &return_type,
                    );
                }

                Type::Unknown
            }
            Type::Unknown => {
                // Error recovery: still check arguments for side effects (usage tracking)
                // This ensures parameters referenced in arguments are marked as used
                for arg in &call.args {
                    self.check_expr(arg);
                }
                Type::any_placeholder()
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3006",
                        format!(
                            "Cannot call non-function type {}",
                            callee_type.display_name()
                        ),
                        call.span,
                    )
                    .with_label("not callable")
                    .with_help(suggestions::suggest_not_callable(&callee_type)),
                );
                Type::Unknown
            }
        }
    }

    /// Check a generic function call with type inference
    fn check_call_with_inference(
        &mut self,
        type_params: &[TypeParamDef],
        params: &[Type],
        return_type: &Type,
        call: &CallExpr,
    ) -> Type {
        use crate::typechecker::generics::TypeInferer;

        let mut inferer = TypeInferer::new();

        // Check each argument and try to infer type parameters
        for (i, arg) in call.args.iter().enumerate() {
            let arg_type = self.check_expr(arg);

            if let Some(param_type) = params.get(i) {
                // Try to unify parameter type with argument type
                if let Err(e) = inferer.unify(param_type, &arg_type) {
                    // Inference failed - report error
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Type inference failed: cannot match argument {} of type {} with parameter of type {}",
                                i + 1,
                                arg_type.display_name(),
                                param_type.display_name()
                            ),
                            arg.span(),
                        )
                        .with_label("type mismatch")
                        .with_help(format!("Inference error: {:?}", e)),
                    );
                    return Type::Unknown;
                }
            }
        }

        // Check if all type parameters were inferred
        if !inferer.all_inferred(type_params) {
            // Some type parameters couldn't be inferred
            let uninferred: Vec<String> = type_params
                .iter()
                .filter(|param| inferer.get_substitution(&param.name).is_none())
                .map(|param| param.name.clone())
                .collect();

            for param_name in &uninferred {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3051",
                        format!(
                            "cannot infer type argument `{}` — add explicit type annotation `<{}>`",
                            param_name, param_name
                        ),
                        call.span,
                    )
                    .with_label("type argument cannot be inferred from call arguments")
                    .with_help(
                        "This type parameter only appears in the return type or is unconstrained. \
                         Provide an explicit type argument: `func::<Type>(args)`",
                    ),
                );
            }
            return Type::Unknown;
        }

        // Apply substitutions to return type
        let inferred_return = inferer.apply_substitutions(return_type);

        // Validate constraints
        if !self.check_constraints(type_params, &inferer, call.span) {
            return Type::Unknown;
        }

        inferred_return
    }

    fn array_concat_result(&self, left: &Type, right: &Type) -> Option<Type> {
        let left_elem = self.array_elem_type_if_all_arrays(left)?;
        let right_elem = self.array_elem_type_if_all_arrays(right)?;

        if self.is_assignable_with_traits(&left_elem, &right_elem) {
            Some(Type::Array(Box::new(right_elem)))
        } else if self.is_assignable_with_traits(&right_elem, &left_elem) {
            Some(Type::Array(Box::new(left_elem)))
        } else {
            None
        }
    }

    fn array_elem_type_if_all_arrays(&self, ty: &Type) -> Option<Type> {
        match ty.normalized() {
            Type::Array(elem) => Some(*elem),
            Type::Union(members) => {
                let mut element_types = Vec::with_capacity(members.len());
                for member in members {
                    match member.normalized() {
                        Type::Array(elem) => element_types.push(*elem),
                        _ => return None,
                    }
                }
                if element_types.is_empty() {
                    None
                } else {
                    Some(Type::union(element_types))
                }
            }
            _ => None,
        }
    }

    fn all_union_pairs_valid<F>(&self, left: &Type, right: &Type, mut predicate: F) -> bool
    where
        F: FnMut(&Type, &Type) -> bool,
    {
        let left_members = self.union_members(left);
        let right_members = self.union_members(right);

        for l in &left_members {
            for r in &right_members {
                if matches!(l.normalized(), Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM)
                    || matches!(r.normalized(), Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM)
                {
                    continue;
                }
                if l.normalized() == Type::Unknown || r.normalized() == Type::Unknown {
                    return false;
                }
                if !predicate(l, r) {
                    return false;
                }
            }
        }
        true
    }

    fn types_overlap(&self, left: &Type, right: &Type) -> bool {
        let left_members = self.union_members(left);
        let right_members = self.union_members(right);

        for l in &left_members {
            for r in &right_members {
                if matches!(l.normalized(), Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM)
                    || matches!(r.normalized(), Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM)
                {
                    return true;
                }
                if l.normalized() == Type::Unknown || r.normalized() == Type::Unknown {
                    return false;
                }
                if self.is_assignable_with_traits(l, r) || self.is_assignable_with_traits(r, l) {
                    return true;
                }
            }
        }
        false
    }

    fn union_members(&self, ty: &Type) -> Vec<Type> {
        match ty.normalized() {
            Type::Union(members) => members,
            other => vec![other],
        }
    }

    fn check_structural_method_call(
        &mut self,
        method_name: &str,
        members: &[StructuralMemberType],
        member: &MemberExpr,
    ) -> Option<Type> {
        let required = members.iter().find(|m| m.name == method_name)?;
        let Type::Function {
            params,
            return_type,
            ..
        } = &required.ty
        else {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3010",
                    format!(
                        "Type '{}' has no method named '{}'",
                        required.ty.display_name(),
                        method_name
                    ),
                    member.member.span,
                )
                .with_label("method not found")
                .with_help(format!(
                    "type '{}' does not support method '{}'",
                    required.ty.display_name(),
                    method_name
                ))
                .with_note(format!(
                    "trait constraint `{}` requires this method — check that the correct trait is implemented for `{}`",
                    method_name,
                    required.ty.display_name()
                )),
            );
            return Some(Type::Unknown);
        };

        let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);
        let expected_args = params.len();
        if provided_args != expected_args {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3005",
                    format!(
                        "Method '{}' expects {} arguments, found {}",
                        method_name, expected_args, provided_args
                    ),
                    member.span,
                )
                .with_label("argument count mismatch")
                .with_help(format!(
                    "method '{}' requires exactly {} argument{}",
                    method_name,
                    expected_args,
                    if expected_args == 1 { "" } else { "s" }
                )),
            );
        }

        if let Some(args) = &member.args {
            for (i, arg) in args.iter().enumerate() {
                let arg_type = self.check_expr(arg);
                if let Some(expected_type) = params.get(i) {
                    // Unknown expected type means the method accepts any argument
                    // (e.g. callback-based array methods: arr.map, arr.filter, etc.)
                    if expected_type.normalized() == crate::typechecker::Type::Unknown {
                        continue;
                    }
                    if !self.is_assignable_with_traits(&arg_type, expected_type) {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Argument {} has wrong type: expected {}, found {}",
                                    i + 1,
                                    expected_type.display_name(),
                                    arg_type.display_name()
                                ),
                                arg.span(),
                            )
                            .with_label("type mismatch")
                            .with_help(format!(
                                "argument {} must be of type {}",
                                i + 1,
                                expected_type.display_name()
                            )),
                        );
                    }
                }
            }
        }

        Some(*return_type.clone())
    }

    fn check_structural_property_access(
        &mut self,
        member_name: &str,
        members: &[StructuralMemberType],
        member: &MemberExpr,
    ) -> Option<Type> {
        let required = members.iter().find(|m| m.name == member_name);
        let Some(required) = required else {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3010",
                    format!("Type has no member named '{}'", member_name),
                    member.member.span,
                )
                .with_label("member not found")
                .with_help(format!(
                    "check that '{}' exists on this record or namespace",
                    member_name
                )),
            );
            return Some(Type::Unknown);
        };

        Some(required.ty.clone())
    }

    fn comparable_operand_type(&mut self, ty: &Type) -> Type {
        ty.normalized()
    }

    /// Check a member expression (method call)
    fn check_member(&mut self, member: &MemberExpr) -> Type {
        // Fast-path: static namespace identifiers (Json, Math, Env).
        // These are not registered in the symbol table — detect by identifier name.
        if let crate::ast::Expr::Identifier(id) = member.target.as_ref() {
            if let Some(ns_tag) = crate::method_dispatch::namespace_type_tag(&id.name) {
                member.type_tag.set(Some(ns_tag));
                // Resolve return type for namespace method calls
                let return_type = resolve_namespace_return_type(&id.name, &member.member.name);
                // D-010: Type::Unknown is always an error state, never a silent wildcard.
                // If a namespace method has no type entry, emit a diagnostic immediately.
                if return_type == Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3061",
                            format!(
                                "{}.{}() has no return type registered in the typechecker",
                                id.name, member.member.name
                            ),
                            member.span,
                        )
                        .with_label("untyped namespace method")
                        .with_help(format!(
                            "add a return type entry for {}.{}() in resolve_namespace_return_type()",
                            id.name, member.member.name
                        )),
                    );
                }
                return return_type;
            }
        }

        // Type-check the target expression
        let target_type = self.check_expr(&member.target);

        // Annotate MemberExpr with TypeTag for method dispatch parity
        let type_tag = match target_type.normalized() {
            Type::JsonValue => Some(crate::method_dispatch::TypeTag::JsonValue),
            Type::Array(_) => Some(crate::method_dispatch::TypeTag::Array),
            Type::String => Some(crate::method_dispatch::TypeTag::String),
            Type::Generic { ref name, .. } if name == "HashMap" => {
                Some(crate::method_dispatch::TypeTag::HashMap)
            }
            Type::Generic { ref name, .. } if name == "HashSet" => {
                Some(crate::method_dispatch::TypeTag::HashSet)
            }
            Type::Generic { ref name, .. } if name == "Queue" => {
                Some(crate::method_dispatch::TypeTag::Queue)
            }
            Type::Generic { ref name, .. } if name == "Stack" => {
                Some(crate::method_dispatch::TypeTag::Stack)
            }
            Type::Generic { ref name, .. } if name == "Option" => {
                Some(crate::method_dispatch::TypeTag::Option)
            }
            Type::Generic { ref name, .. } if name == "Result" => {
                Some(crate::method_dispatch::TypeTag::Result)
            }
            _ => None,
        };
        member.type_tag.set(type_tag);

        // Skip error recovery cases
        if target_type.normalized() == Type::Unknown {
            return Type::Unknown;
        }

        // Look up the method in the method table and clone the signature to avoid borrow issues
        let method_name = &member.member.name;
        let target_norm = target_type.normalized();

        if member.args.is_none() {
            if let Type::Structural { members } = &target_norm {
                if let Some(return_type) =
                    self.check_structural_property_access(method_name, members, member)
                {
                    return return_type;
                }
            }
        }

        if let Type::Union(members) = target_norm {
            let mut return_types = Vec::new();
            let mut signatures = Vec::new();

            for member_ty in &members {
                if let Some(sig) = self.method_table.lookup(member_ty, method_name) {
                    signatures.push(sig.clone());
                    return_types.push(sig.return_type);
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3010",
                            format!(
                                "Type '{}' has no method named '{}'",
                                member_ty.display_name(),
                                method_name
                            ),
                            member.member.span,
                        )
                        .with_label("method not found")
                        .with_help(format!(
                            "method '{}' must exist on all union members",
                            method_name
                        )),
                    );
                    return Type::Unknown;
                }
            }

            if let Some(first_sig) = signatures.first() {
                let expected_args = first_sig.arg_types.len();
                let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);

                if provided_args != expected_args {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3005",
                            format!(
                                "Method '{}' expects {} arguments, found {}",
                                method_name, expected_args, provided_args
                            ),
                            member.span,
                        )
                        .with_label("argument count mismatch")
                        .with_help(format!(
                            "method '{}' requires exactly {} argument{}",
                            method_name,
                            expected_args,
                            if expected_args == 1 { "" } else { "s" }
                        )),
                    );
                }

                if let Some(args) = &member.args {
                    for (i, arg) in args.iter().enumerate() {
                        let arg_type = self.check_expr(arg);
                        for sig in &signatures {
                            if let Some(expected_type) = sig.arg_types.get(i) {
                                // Unknown expected type means the method accepts any argument
                                if expected_type.normalized() == crate::typechecker::Type::Unknown {
                                    continue;
                                }
                                if !self.is_assignable_with_traits(&arg_type, expected_type) {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3001",
                                            format!(
                                                "Argument {} has wrong type: expected {}, found {}",
                                                i + 1,
                                                expected_type.display_name(),
                                                arg_type.display_name()
                                            ),
                                            arg.span(),
                                        )
                                        .with_label("type mismatch")
                                        .with_help(
                                            format!(
                                                "argument {} must be of type {}",
                                                i + 1,
                                                expected_type.display_name()
                                            ),
                                        ),
                                    );
                                    return Type::Unknown;
                                }
                            }
                        }
                    }
                }

                return Type::union(return_types);
            }

            return Type::Unknown;
        }

        if member.args.is_some() {
            if let Type::Structural { ref members } = target_norm {
                if let Some(return_type) =
                    self.check_structural_method_call(method_name, members, member)
                {
                    return return_type;
                }
            }
        }

        if let Type::TraitObject { name: trait_name } = &target_norm {
            if let Some(methods) = self.trait_registry.get_methods(trait_name) {
                if let Some(method_sig) = methods.iter().find(|m| m.name == *method_name).cloned() {
                    let param_types = method_sig.param_types.clone();
                    let return_type = method_sig.return_type.clone();
                    let expected_args = param_types.len();
                    let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);
                    if provided_args != expected_args {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3005",
                                format!(
                                    "Method '{}' expects {} arguments, found {}",
                                    method_name, expected_args, provided_args
                                ),
                                member.span,
                            )
                            .with_label("argument count mismatch")
                            .with_help(format!(
                                "method '{}' requires exactly {} argument{}",
                                method_name,
                                expected_args,
                                if expected_args == 1 { "" } else { "s" }
                            )),
                        );
                    }

                    if let Some(args) = &member.args {
                        for (i, arg) in args.iter().enumerate() {
                            let arg_type = self.check_expr(arg);
                            if let Some(expected_type) = param_types.get(i) {
                                // Unknown expected type means the method accepts any argument
                                if expected_type.normalized() == crate::typechecker::Type::Unknown {
                                    continue;
                                }
                                if !self.is_assignable_with_traits(&arg_type, expected_type) {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3001",
                                            format!(
                                                "Argument {} has wrong type: expected {}, found {}",
                                                i + 1,
                                                expected_type.display_name(),
                                                arg_type.display_name()
                                            ),
                                            arg.span(),
                                        )
                                        .with_label("type mismatch")
                                        .with_help(
                                            format!(
                                                "argument {} must be of type {}",
                                                i + 1,
                                                expected_type.display_name()
                                            ),
                                        ),
                                    );
                                    return Type::Unknown;
                                }
                            }
                        }
                    }

                    *member.trait_dispatch.borrow_mut() = Some((String::new(), trait_name.clone()));
                    return return_type;
                } else if member.args.is_some() {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3010",
                            format!(
                                "Trait '{}' has no method named '{}'",
                                trait_name, method_name
                            ),
                            member.member.span,
                        )
                        .with_label("method not found")
                        .with_help(format!(
                            "trait `{trait_name}` does not define a method `{method_name}` — check the trait definition for the correct method name"
                        ))
                        .with_note("if you intended to call an inherent method, remove the trait annotation from the `impl` block"),
                    );
                    return Type::Unknown;
                }
            }
        }

        let method_sig = self.method_table.lookup(&target_type, method_name);

        if let Some(method_sig) = method_sig {
            // Check argument count
            let provided_args = member.args.as_ref().map(|args| args.len()).unwrap_or(0);
            let expected_args = method_sig.arg_types.len();

            if provided_args != expected_args {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3005",
                        format!(
                            "Method '{}' expects {} arguments, found {}",
                            method_name, expected_args, provided_args
                        ),
                        member.span,
                    )
                    .with_label("argument count mismatch")
                    .with_help(format!(
                        "method '{}' requires exactly {} argument{}",
                        method_name,
                        expected_args,
                        if expected_args == 1 { "" } else { "s" }
                    )),
                );
            }

            // Check argument types if present
            if let Some(args) = &member.args {
                for (i, arg) in args.iter().enumerate() {
                    let arg_type = self.check_expr(arg);
                    if let Some(expected_type) = method_sig.arg_types.get(i) {
                        // Unknown expected type means the method accepts any argument
                        // (e.g. callback-based array methods: arr.map, arr.filter, etc.)
                        if expected_type.normalized() == crate::typechecker::Type::Unknown {
                            continue;
                        }
                        if !self.is_assignable_with_traits(&arg_type, expected_type) {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    format!(
                                        "Argument {} has wrong type: expected {}, found {}",
                                        i + 1,
                                        expected_type.display_name(),
                                        arg_type.display_name()
                                    ),
                                    arg.span(),
                                )
                                .with_label("type mismatch")
                                .with_help(format!(
                                    "argument {} must be of type {}",
                                    i + 1,
                                    expected_type.display_name()
                                )),
                            );
                        }
                    }
                }
            }

            // For callback-based array methods whose return type depends on the callback's
            // return type (map, flatMap), infer the element type from the callback arg.
            let return_type = if matches!(method_name.as_str(), "map" | "flatMap") {
                if let Some(args) = &member.args {
                    if let Some(callback_arg) = args.first() {
                        let cb_type = self.check_expr(callback_arg);
                        if let Type::Function {
                            return_type: cb_ret,
                            ..
                        } = cb_type.normalized()
                        {
                            if cb_ret.normalized() != Type::Unknown {
                                if method_name == "flatMap" {
                                    match cb_ret.normalized() {
                                        Type::Array(inner) => Type::Array(inner),
                                        other => Type::Array(Box::new(other)),
                                    }
                                } else {
                                    Type::Array(cb_ret)
                                }
                            } else {
                                method_sig.return_type
                            }
                        } else {
                            method_sig.return_type
                        }
                    } else {
                        method_sig.return_type
                    }
                } else {
                    method_sig.return_type
                }
            } else {
                method_sig.return_type
            };
            return_type
        } else if let Some((return_type, type_name)) =
            self.resolve_inherent_method_call(&target_type, method_name)
        {
            // Slot 2a: inherent method dispatch (D-037: inherent takes priority over trait)
            // trait_dispatch uses empty string for trait_name to signal inherent dispatch.
            *member.trait_dispatch.borrow_mut() = Some((type_name, String::new()));

            if let Some(args) = &member.args {
                for arg in args.iter() {
                    let _ = self.check_expr(arg);
                }
            }
            return_type
        } else if let Some((return_type, type_name, trait_name)) =
            self.resolve_trait_method_call_with_info(&target_type, method_name)
        {
            // Slot 2b: trait method dispatch — found a matching impl
            // Annotate MemberExpr with dispatch info for compiler/interpreter
            *member.trait_dispatch.borrow_mut() = Some((type_name, trait_name));

            // Check args if present (non-self params only)
            if let Some(args) = &member.args {
                for arg in args.iter() {
                    let _ = self.check_expr(arg);
                }
            }
            return_type
        } else {
            // Check if a declared trait has this method — if so, emit AT3035 (not implemented)
            // rather than generic AT3010 (not found).
            let trait_name_with_method = self
                .trait_registry
                .find_trait_with_method(method_name)
                .map(|s| s.to_owned());

            if let Some(trait_name) = trait_name_with_method {
                let type_display = self.nominal_display_name(&target_type);
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        error_codes::TYPE_DOES_NOT_IMPLEMENT_TRAIT,
                        format!(
                            "Type '{}' does not implement trait '{}' required for method '{}'",
                            type_display, trait_name, method_name
                        ),
                        member.member.span,
                    )
                    .with_label(format!("trait '{}' not implemented", trait_name))
                    .with_help(format!(
                        "implement '{}' for '{}' with: impl {} for {} {{ ... }}",
                        trait_name, type_display, trait_name, type_display
                    )),
                );
            } else {
                // Method not found for this type (not a trait method either)
                let suggestion = self.method_suggestion_for(&target_type, method_name);
                let help = match suggestion {
                    Some(s) => format!(
                        "type '{}' does not support method '{}' — {}",
                        target_type.display_name(),
                        method_name,
                        s
                    ),
                    None => format!(
                        "type '{}' does not support method '{}'",
                        target_type.display_name(),
                        method_name
                    ),
                };
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3010",
                        format!(
                            "Type '{}' has no method named '{}'",
                            target_type.display_name(),
                            method_name
                        ),
                        member.member.span,
                    )
                    .with_label("method not found")
                    .with_help(help),
                );
            }
            Type::Unknown
        }
    }

    /// Check an index expression
    fn check_index(&mut self, index: &IndexExpr) -> Type {
        let target_type = self.check_expr(&index.target);
        let target_norm = target_type.normalized();

        if matches!(target_norm, Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM) {
            let IndexValue::Single(expr) = &index.index;
            self.check_expr(expr);
            return Type::any_placeholder();
        }

        let IndexValue::Single(index_expr) = &index.index;
        let index_type = self.check_expr(index_expr);
        let index_norm = index_type.normalized();

        let index_is_range = index_norm == Type::Range;

        match target_norm {
            Type::Array(elem_type) => {
                if index_is_range {
                    Type::Array(elem_type)
                } else {
                    if index_norm != Type::Number {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Array index must be number, found {}",
                                    index_type.display_name()
                                ),
                                index_expr.span(),
                            )
                            .with_label("type mismatch")
                            .with_help("array indices must be numbers"),
                        );
                    }
                    *elem_type
                }
            }
            Type::JsonValue => {
                if index_is_range {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            "Range indices are only valid for arrays".to_string(),
                            index_expr.span(),
                        )
                        .with_label("not indexable")
                        .with_help("only arrays can be sliced with ranges"),
                    );
                    Type::Unknown
                } else {
                    if index_norm != Type::String && index_norm != Type::Number {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "JSON index must be string or number, found {}",
                                    index_type.display_name()
                                ),
                                index_expr.span(),
                            )
                            .with_label("type mismatch")
                            .with_help("use a string key or numeric index to access JSON values"),
                        );
                    }
                    Type::JsonValue
                }
            }
            Type::String => {
                if index_is_range {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            "Range indices are only valid for arrays".to_string(),
                            index_expr.span(),
                        )
                        .with_label("not indexable")
                        .with_help("only arrays can be sliced with ranges"),
                    );
                    Type::Unknown
                } else {
                    if index_norm != Type::Number {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "String index must be number, found {}",
                                    index_type.display_name()
                                ),
                                index_expr.span(),
                            )
                            .with_label("type mismatch")
                            .with_help("string indices must be numbers"),
                        );
                    }
                    Type::String
                }
            }
            Type::Union(members) => {
                let mut result_types = Vec::new();
                for member in members {
                    match member {
                        Type::Array(elem_type) => {
                            if index_is_range {
                                result_types.push(Type::Array(elem_type));
                            } else {
                                if index_norm != Type::Number {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3001",
                                            format!(
                                                "Array index must be number, found {}",
                                                index_type.display_name()
                                            ),
                                            index_expr.span(),
                                        )
                                        .with_label("type mismatch")
                                        .with_help("array indices must be numbers"),
                                    );
                                }
                                result_types.push(*elem_type);
                            }
                        }
                        Type::JsonValue => {
                            if index_is_range {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3001",
                                        "Range indices are only valid for arrays".to_string(),
                                        index_expr.span(),
                                    )
                                    .with_label("not indexable")
                                    .with_help("only arrays can be sliced with ranges"),
                                );
                                return Type::Unknown;
                            }
                            if index_norm != Type::String && index_norm != Type::Number {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3001",
                                        format!(
                                            "JSON index must be string or number, found {}",
                                            index_type.display_name()
                                        ),
                                        index_expr.span(),
                                    )
                                    .with_label("type mismatch")
                                    .with_help(
                                        "use a string key or numeric index to access JSON values",
                                    ),
                                );
                            }
                            result_types.push(Type::JsonValue);
                        }
                        Type::String => {
                            if index_is_range {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3001",
                                        "Range indices are only valid for arrays".to_string(),
                                        index_expr.span(),
                                    )
                                    .with_label("not indexable")
                                    .with_help("only arrays can be sliced with ranges"),
                                );
                                return Type::Unknown;
                            }
                            if index_norm != Type::Number {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3001",
                                        format!(
                                            "String index must be number, found {}",
                                            index_type.display_name()
                                        ),
                                        index_expr.span(),
                                    )
                                    .with_label("type mismatch")
                                    .with_help("string indices must be numbers"),
                                );
                            }
                            result_types.push(Type::String);
                        }
                        _ => {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3001",
                                    format!("Cannot index into type {}", member.display_name()),
                                    index.target.span(),
                                )
                                .with_label("not indexable")
                                .with_help("only arrays, strings, and json values can be indexed"),
                            );
                            return Type::Unknown;
                        }
                    }
                }
                Type::union(result_types)
            }
            Type::Unknown => Type::Unknown,
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!("Cannot index into type {}", target_type.display_name()),
                        index.target.span(),
                    )
                    .with_label("not indexable")
                    .with_help("only arrays, strings, and json values can be indexed"),
                );
                Type::Unknown
            }
        }
    }

    /// Check an array literal
    fn check_array_literal(&mut self, arr: &ArrayLiteral) -> Type {
        if arr.elements.is_empty() {
            // Empty array - element type is unknown until constrained
            return Type::Array(Box::new(Type::Unknown));
        }

        // Check first element to determine array type
        let first_type = self.check_expr(&arr.elements[0]);

        // Check that all elements have the same type
        for (i, elem) in arr.elements.iter().enumerate().skip(1) {
            let elem_type = self.check_expr(elem);
            if !self.is_assignable_with_traits(&elem_type, &first_type) {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!(
                            "Array element {} has wrong type: expected {}, found {}",
                            i,
                            first_type.display_name(),
                            elem_type.display_name()
                        ),
                        elem.span(),
                    )
                    .with_label("type mismatch")
                    .with_help(format!(
                        "all array elements must be type {} (inferred from first element)",
                        first_type.display_name()
                    )),
                );
            }
        }

        Type::Array(Box::new(first_type))
    }

    /// Check a match expression
    fn check_match(&mut self, match_expr: &crate::ast::MatchExpr) -> Type {
        // 1. Check scrutinee type
        let scrutinee_type = self.check_expr(&match_expr.scrutinee);

        if scrutinee_type.normalized() == Type::Unknown {
            // Error in scrutinee, skip match checking
            return Type::Unknown;
        }

        // 2. Check each arm and collect result types
        let mut arm_types = Vec::new();

        for (arm_idx, arm) in match_expr.arms.iter().enumerate() {
            // Check pattern against scrutinee type
            let pattern_bindings = self.check_pattern(&arm.pattern, &scrutinee_type);

            // Enter a new scope for pattern bindings
            self.symbol_table.enter_scope();

            // Add pattern bindings to symbol table for this arm's scope
            for (var_name, var_type, var_span) in &pattern_bindings {
                let symbol = crate::symbol::Symbol {
                    name: var_name.clone(),
                    ty: var_type.clone(),
                    mutable: false, // Pattern bindings are immutable
                    kind: crate::symbol::SymbolKind::Variable,
                    span: *var_span,
                    exported: false,
                };
                // Ignore if binding fails (duplicate names in pattern - will be caught separately)
                let _ = self.symbol_table.define(symbol);
            }

            // Check guard if present — must be bool (AT3029)
            if let Some(guard) = &arm.guard {
                let guard_type = self.check_expr(guard);
                if guard_type.normalized() != Type::Bool {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3029",
                            format!(
                                "Guard expression must be bool, found {}",
                                guard_type.display_name()
                            ),
                            guard.span(),
                        )
                        .with_label("must be bool")
                        .with_help("guard expressions must evaluate to a boolean value"),
                    );
                }
            }

            // Check arm body with bindings in scope
            let arm_type = self.check_expr(&arm.body);
            arm_types.push((arm_type.clone(), arm.body.span(), arm_idx));

            // Exit scope (removes pattern bindings)
            self.symbol_table.exit_scope();
        }

        // 3. Ensure all arms return compatible types
        if arm_types.is_empty() {
            // Empty match (parser should prevent this, but handle gracefully)
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3020",
                    "Match expression must have at least one arm",
                    match_expr.span,
                )
                .with_label("empty match")
                .with_help("add at least one match arm with a pattern and expression"),
            );
            return Type::Unknown;
        }

        let mut unified = arm_types[0].0.clone();
        for (arm_type, arm_span, arm_idx) in &arm_types[1..] {
            if let Some(lub) = crate::typechecker::inference::least_upper_bound(&unified, arm_type)
            {
                unified = lub;
            } else {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3021",
                        format!(
                            "Match arm {} returns incompatible type: expected {}, found {}",
                            arm_idx + 1,
                            unified.display_name(),
                            arm_type.display_name()
                        ),
                        *arm_span,
                    )
                    .with_label("type mismatch")
                    .with_help(format!(
                        "all match arms must return compatible types (current: {})",
                        unified.display_name()
                    )),
                );
            }
        }

        // 4. Check exhaustiveness
        self.check_exhaustiveness(&match_expr.arms, &scrutinee_type, match_expr.span);

        // 5. Return the unified type
        unified
    }

    /// Check if a pattern covers a given constructor name (including inside OR patterns)
    fn pattern_covers_constructor(pattern: &crate::ast::Pattern, ctor: &str) -> bool {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Constructor { name, .. } => name.name == ctor,
            Pattern::Or(alternatives, _) => alternatives
                .iter()
                .any(|alt| Self::pattern_covers_constructor(alt, ctor)),
            _ => false,
        }
    }

    /// Check if a pattern covers a given bool literal (including inside OR patterns)
    fn pattern_covers_bool(pattern: &crate::ast::Pattern, val: bool) -> bool {
        use crate::ast::{Literal, Pattern};
        match pattern {
            Pattern::Literal(Literal::Bool(b), _) => *b == val,
            Pattern::Or(alternatives, _) => alternatives
                .iter()
                .any(|alt| Self::pattern_covers_bool(alt, val)),
            _ => false,
        }
    }

    /// Check if a pattern is a catch-all (wildcard or unguarded variable, including inside OR)
    fn pattern_is_catch_all(pattern: &crate::ast::Pattern) -> bool {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Wildcard(_) | Pattern::Variable(_) => true,
            Pattern::Or(alternatives, _) => alternatives.iter().any(Self::pattern_is_catch_all),
            _ => false,
        }
    }

    /// Check exhaustiveness of match arms
    fn check_exhaustiveness(
        &mut self,
        arms: &[crate::ast::MatchArm],
        scrutinee_type: &Type,
        match_span: Span,
    ) {
        // Check if there's a catch-all pattern (wildcard or variable binding, unguarded)
        let has_catch_all = arms
            .iter()
            .any(|arm| arm.guard.is_none() && Self::pattern_is_catch_all(&arm.pattern));

        if has_catch_all {
            // Wildcard or variable catches everything - exhaustive
            return;
        }

        // Check exhaustiveness based on scrutinee type
        let scrutinee_norm = scrutinee_type.normalized();
        if let Type::Union(members) = scrutinee_norm {
            for member in members {
                self.check_exhaustiveness(arms, &member, match_span);
            }
            return;
        }

        match scrutinee_norm {
            Type::Generic { name, .. } if name == "Option" => {
                // Option<T> requires Some and None to be covered
                let has_some = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_constructor(&arm.pattern, "Some")
                });

                let has_none = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_constructor(&arm.pattern, "None")
                });

                if !has_some || !has_none {
                    let missing = if !has_some && !has_none {
                        "Some(_), None".to_string()
                    } else if !has_some {
                        "Some(_)".to_string()
                    } else {
                        "None".to_string()
                    };

                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3027",
                            format!("Non-exhaustive match on Option: missing {}", missing),
                            match_span,
                        )
                        .with_label("non-exhaustive")
                        .with_help(format!("Add arm: {} => ...", missing)),
                    );
                }
            }

            Type::Generic { name, .. } if name == "Result" => {
                // Result<T,E> requires Ok and Err to be covered
                let has_ok = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_constructor(&arm.pattern, "Ok")
                });

                let has_err = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_constructor(&arm.pattern, "Err")
                });

                if !has_ok || !has_err {
                    let missing = if !has_ok && !has_err {
                        "Ok(_), Err(_)".to_string()
                    } else if !has_ok {
                        "Ok(_)".to_string()
                    } else {
                        "Err(_)".to_string()
                    };

                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3027",
                            format!("Non-exhaustive match on Result: missing {}", missing),
                            match_span,
                        )
                        .with_label("non-exhaustive")
                        .with_help(format!("Add arm: {} => ...", missing)),
                    );
                }
            }

            Type::Bool => {
                // Bool requires true and false to be covered (or wildcard)
                let has_true = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_bool(&arm.pattern, true)
                });
                let has_false = arms.iter().any(|arm| {
                    arm.guard.is_none() && Self::pattern_covers_bool(&arm.pattern, false)
                });

                if !has_true || !has_false {
                    let missing = if !has_true && !has_false {
                        "true, false".to_string()
                    } else if !has_true {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    };

                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3027",
                            format!("Non-exhaustive match on bool: missing {}", missing),
                            match_span,
                        )
                        .with_label("non-exhaustive")
                        .with_help(format!("Add arm: {} => ... or use wildcard _", missing)),
                    );
                }
            }

            Type::Number | Type::String | Type::Array(_) | Type::Null => {
                // These types have infinite values - require wildcard
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3027",
                        format!(
                            "Non-exhaustive match on {}: patterns must cover all possible values",
                            scrutinee_type.display_name()
                        ),
                        match_span,
                    )
                    .with_label("non-exhaustive")
                    .with_help("Add wildcard pattern: _ => ..."),
                );
            }

            _ => {
                // For other types, warn but don't error (conservative approach)
            }
        }
    }

    /// Check a pattern and return variable bindings (name, type, span)
    fn check_pattern(
        &mut self,
        pattern: &Pattern,
        expected_type: &Type,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();
        let expected_norm = expected_type.normalized();

        if let Type::Union(members) = expected_norm {
            match pattern {
                Pattern::Wildcard(_) => return bindings,
                Pattern::Variable(id) => {
                    bindings.push((id.name.clone(), Type::Union(members), id.span));
                    return bindings;
                }
                Pattern::Literal(lit, span) => {
                    let lit_type = match lit {
                        Literal::Number(_) => Type::Number,
                        Literal::String(_) => Type::String,
                        Literal::Bool(_) => Type::Bool,
                        Literal::Null => Type::Null,
                    };
                    if !members
                        .iter()
                        .any(|member| self.is_assignable_with_traits(&lit_type, member))
                    {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3022",
                                format!(
                                    "Pattern type mismatch: expected {}, found {}",
                                    Type::Union(members).display_name(),
                                    lit_type.display_name()
                                ),
                                *span,
                            )
                            .with_label("type mismatch")
                            .with_help("use a matching literal or wildcard pattern"),
                        );
                    }
                    return bindings;
                }
                Pattern::Constructor { name, args, span } => {
                    let ctor_name = name.name.as_str();
                    let target_member = members.iter().find(|member| match member.normalized() {
                        Type::Generic { name, .. }
                            if ctor_name == "Some" || ctor_name == "None" =>
                        {
                            name == "Option"
                        }
                        Type::Generic { name, .. } if ctor_name == "Ok" || ctor_name == "Err" => {
                            name == "Result"
                        }
                        _ => false,
                    });

                    if let Some(member) = target_member {
                        return self.check_constructor_pattern(name, args, member, *span);
                    }

                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3022",
                            format!(
                                "Pattern type mismatch: expected {}, found constructor {}",
                                Type::Union(members).display_name(),
                                name.name
                            ),
                            *span,
                        )
                        .with_label("type mismatch")
                        .with_help("use a matching constructor or wildcard pattern"),
                    );
                    return bindings;
                }
                Pattern::Array { elements, span } => {
                    for member in &members {
                        if matches!(member.normalized(), Type::Array(_)) {
                            return self.check_array_pattern(elements, member, *span);
                        }
                    }
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3022",
                            format!(
                                "Pattern type mismatch: expected {}, found array pattern",
                                Type::Union(members).display_name()
                            ),
                            *span,
                        )
                        .with_label("type mismatch")
                        .with_help("use a matching array pattern or wildcard"),
                    );
                    return bindings;
                }
                Pattern::Or(alternatives, _) => {
                    // Check each sub-pattern independently; bindings from first sub-pattern used
                    for alt in alternatives {
                        let alt_bindings = self.check_pattern(alt, &Type::Union(members.clone()));
                        if bindings.is_empty() {
                            bindings = alt_bindings;
                        }
                    }
                    return bindings;
                }
                Pattern::EnumVariant {
                    enum_name,
                    variant_name,
                    args,
                    ..
                } => {
                    // H-120: look up variant field types so bindings get proper types
                    let field_types =
                        self.enum_variant_field_types(&enum_name.name, &variant_name.name);
                    for (i, arg) in args.iter().enumerate() {
                        let field_ty = field_types
                            .get(i)
                            .map(|tr| self.resolve_type_ref(tr))
                            .unwrap_or(Type::Unknown);
                        bindings.extend(self.check_pattern(arg, &field_ty));
                    }
                    return bindings;
                }
            }
        }

        match pattern {
            Pattern::Literal(lit, span) => {
                // Check literal type matches expected type
                let lit_type = match lit {
                    Literal::Number(_) => Type::Number,
                    Literal::String(_) => Type::String,
                    Literal::Bool(_) => Type::Bool,
                    Literal::Null => Type::Null,
                };

                if !self.is_assignable_with_traits(&lit_type, &expected_norm) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3022",
                            format!(
                                "Pattern type mismatch: expected {}, found {}",
                                expected_norm.display_name(),
                                lit_type.display_name()
                            ),
                            *span,
                        )
                        .with_label("type mismatch")
                        .with_help(format!(
                            "use a {} literal or wildcard pattern",
                            expected_norm.display_name()
                        )),
                    );
                }
            }

            Pattern::Wildcard(_) => {
                // Wildcard matches anything, no bindings
            }

            Pattern::Variable(id) => {
                // Variable binding - binds the entire scrutinee value
                bindings.push((id.name.clone(), expected_norm.clone(), id.span));
            }

            Pattern::Constructor { name, args, span } => {
                // Check constructor pattern (Ok, Err, Some, None)
                bindings.extend(self.check_constructor_pattern(name, args, &expected_norm, *span));
            }

            Pattern::Array { elements, span } => {
                // Check array pattern
                bindings.extend(self.check_array_pattern(elements, &expected_norm, *span));
            }

            Pattern::Or(alternatives, _) => {
                // Check each sub-pattern independently; bindings from first sub-pattern used
                for alt in alternatives {
                    let alt_bindings = self.check_pattern(alt, expected_type);
                    if bindings.is_empty() {
                        bindings = alt_bindings;
                    }
                }
            }

            Pattern::EnumVariant {
                enum_name,
                variant_name,
                args,
                ..
            } => {
                // H-120: look up variant field types so bindings get proper types
                let field_types =
                    self.enum_variant_field_types(&enum_name.name, &variant_name.name);
                for (i, arg) in args.iter().enumerate() {
                    let field_ty = field_types
                        .get(i)
                        .map(|tr| self.resolve_type_ref(tr))
                        .unwrap_or(Type::Unknown);
                    bindings.extend(self.check_pattern(arg, &field_ty));
                }
            }
        }

        bindings
    }

    /// Check constructor pattern (Ok, Err, Some, None)
    fn check_constructor_pattern(
        &mut self,
        name: &Identifier,
        args: &[Pattern],
        expected_type: &Type,
        span: Span,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();
        let expected_norm = expected_type.normalized();

        match expected_norm {
            Type::Generic {
                name: type_name,
                type_args,
            } => {
                match type_name.as_str() {
                    "Option" if type_args.len() == 1 => {
                        // Option<T> has constructors: Some(T), None
                        match name.name.as_str() {
                            "Some" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3023",
                                            format!(
                                                "Some expects 1 argument, found {}",
                                                args.len()
                                            ),
                                            span,
                                        )
                                        .with_label("wrong arity")
                                        .with_help("Some requires exactly 1 argument: Some(value)"),
                                    );
                                } else {
                                    // Check inner pattern against T
                                    bindings.extend(self.check_pattern(&args[0], &type_args[0]));
                                }
                            }
                            "None" => {
                                if !args.is_empty() {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3023",
                                            format!(
                                                "None expects 0 arguments, found {}",
                                                args.len()
                                            ),
                                            span,
                                        )
                                        .with_label("wrong arity")
                                        .with_help("None requires no arguments: None"),
                                    );
                                }
                            }
                            _ => {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3024",
                                        format!("Unknown Option constructor: {}", name.name),
                                        name.span,
                                    )
                                    .with_label("unknown constructor")
                                    .with_help(
                                        "Option only has constructors: Some(value) and None",
                                    ),
                                );
                            }
                        }
                    }
                    "Result" if type_args.len() == 2 => {
                        // Result<T, E> has constructors: Ok(T), Err(E)
                        match name.name.as_str() {
                            "Ok" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3023",
                                            format!("Ok expects 1 argument, found {}", args.len()),
                                            span,
                                        )
                                        .with_label("wrong arity")
                                        .with_help("Ok requires exactly 1 argument: Ok(value)"),
                                    );
                                } else {
                                    // Check inner pattern against T
                                    bindings.extend(self.check_pattern(&args[0], &type_args[0]));
                                }
                            }
                            "Err" => {
                                if args.len() != 1 {
                                    self.diagnostics.push(
                                        Diagnostic::error_with_code(
                                            "AT3023",
                                            format!("Err expects 1 argument, found {}", args.len()),
                                            span,
                                        )
                                        .with_label("wrong arity")
                                        .with_help("Err requires exactly 1 argument: Err(error)"),
                                    );
                                } else {
                                    // Check inner pattern against E
                                    bindings.extend(self.check_pattern(&args[0], &type_args[1]));
                                }
                            }
                            _ => {
                                self.diagnostics.push(
                                    Diagnostic::error_with_code(
                                        "AT3024",
                                        format!("Unknown Result constructor: {}", name.name),
                                        name.span,
                                    )
                                    .with_label("unknown constructor")
                                    .with_help(
                                        "Result only has constructors: Ok(value) and Err(error)",
                                    ),
                                );
                            }
                        }
                    }
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3025",
                                format!(
                                    "Constructor patterns not supported for type {}",
                                    expected_type.display_name()
                                ),
                                span,
                            )
                            .with_label("unsupported type")
                            .with_help(
                                "constructor patterns only work with Option and Result types",
                            ),
                        );
                    }
                }
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3025",
                        format!(
                            "Constructor patterns not supported for type {}",
                            expected_type.display_name()
                        ),
                        span,
                    )
                    .with_label("unsupported type"),
                );
            }
        }

        bindings
    }

    /// Check array pattern
    fn check_array_pattern(
        &mut self,
        elements: &[Pattern],
        expected_type: &Type,
        span: Span,
    ) -> Vec<(String, Type, Span)> {
        let mut bindings = Vec::new();
        let expected_norm = expected_type.normalized();

        match expected_norm {
            Type::Array(elem_type) => {
                // Check each pattern element against the array element type
                for pattern in elements {
                    bindings.extend(self.check_pattern(pattern, &elem_type));
                }
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3026",
                        format!(
                            "Array pattern used on non-array type: {}",
                            expected_type.display_name()
                        ),
                        span,
                    )
                    .with_label("type mismatch")
                    .with_help("array patterns can only match array types"),
                );
            }
        }

        bindings
    }

    /// Check try expression (error propagation operator ?)
    fn check_try(&mut self, try_expr: &TryExpr) -> Type {
        use crate::ast::TryTargetKind;

        // Type check the expression being tried
        let expr_type = self.check_expr(&try_expr.expr);
        let expr_norm = expr_type.normalized();

        // Skip if expression type is unknown (error already reported)
        if expr_norm == Type::Unknown {
            return Type::Unknown;
        }

        // Expression must be Result<T, E> or Option<T>
        enum TrySource {
            Result { ok_type: Type, err_type: Type },
            Option { inner_type: Type },
        }

        let source = match &expr_norm {
            Type::Generic { name, type_args } if name == "Result" && type_args.len() == 2 => {
                TrySource::Result {
                    ok_type: type_args[0].clone(),
                    err_type: type_args[1].clone(),
                }
            }
            Type::Generic { name, type_args } if name == "Option" && type_args.len() == 1 => {
                TrySource::Option {
                    inner_type: type_args[0].clone(),
                }
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3027",
                        format!(
                            "? operator requires Result<T, E> or Option<T> type, found {}",
                            expr_type.display_name()
                        ),
                        try_expr.span,
                    )
                    .with_label("not a Result or Option type")
                    .with_help(
                        "the ? operator can only be applied to Result<T, E> or Option<T> values",
                    ),
                );
                return Type::Unknown;
            }
        };

        // At top-level (script/REPL context), ? is allowed without a function.
        // Propagation semantics: Ok(v) → v, Err(e) → early eval termination.
        let is_top_level = self.current_function_return_type.is_none();

        match source {
            TrySource::Result { ok_type, err_type } => {
                // Annotate for compiler
                *try_expr.target_kind.borrow_mut() = Some(TryTargetKind::Result);

                if is_top_level {
                    return ok_type;
                }

                let function_return_type = self.current_function_return_type.clone().unwrap();
                let function_return_norm = function_return_type.normalized();

                // Function must return Result<T', E'>
                match &function_return_norm {
                    Type::Generic { name, type_args }
                        if name == "Result" && type_args.len() == 2 =>
                    {
                        let function_err_type = &type_args[1];

                        let err_norm = err_type.normalized();
                        let function_err_norm = function_err_type.normalized();
                        let err_is_any = matches!(
                            err_norm,
                            Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM
                        );
                        let function_err_is_any = matches!(
                            function_err_norm,
                            Type::TypeParameter { ref name } if name == ANY_TYPE_PARAM
                        );

                        // Error types must be compatible (any-placeholder is always compatible)
                        if !err_is_any && !function_err_is_any && err_norm != function_err_norm {
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT3029",
                                    format!(
                                        "? operator error type mismatch: expression has error type {}, but function returns {}",
                                        err_type.display_name(),
                                        function_err_type.display_name()
                                    ),
                                    try_expr.span,
                                )
                                .with_label("error type mismatch")
                                .with_help(format!(
                                    "convert the error type to {} or change the function's error type",
                                    function_err_type.display_name()
                                )),
                            );
                        }

                        ok_type
                    }
                    _ => {
                        // Allow `?` as an unwrap in non-Result functions (runtime error on Err).
                        ok_type
                    }
                }
            }
            TrySource::Option { inner_type } => {
                // Annotate for compiler
                *try_expr.target_kind.borrow_mut() = Some(TryTargetKind::Option);

                if is_top_level {
                    return inner_type;
                }

                let function_return_type = self.current_function_return_type.clone().unwrap();
                let function_return_norm = function_return_type.normalized();

                // Function must return Option<T'>
                match &function_return_norm {
                    Type::Generic { name, type_args }
                        if name == "Option" && type_args.len() == 1 =>
                    {
                        inner_type
                    }
                    _ => {
                        // Allow `?` as an unwrap in non-Option functions (runtime error on None).
                        inner_type
                    }
                }
            }
        }
    }

    /// Typecheck an anonymous function expression.
    ///
    /// Resolves param types (any for untyped arrow-fn params — Block 5 infers them),
    /// checks the body in a new scope, validates capture semantics, and returns
    /// `Type::Function { params, return_type }`.
    pub(super) fn check_anon_fn(
        &mut self,
        params: &[crate::ast::Param],
        return_type_ref: Option<&crate::ast::TypeRef>,
        body: &Expr,
        span: Span,
    ) -> Type {
        // Resolve param types — None type_ref → any (inferred later in Block 5)
        let param_types: Vec<Type> = params
            .iter()
            .map(|p| self.resolve_type_ref(&p.type_ref))
            .collect();

        // Resolve declared return type (if present)
        let declared_return = return_type_ref.map(|t| self.resolve_type_ref(t));

        // Save and update function context so return statements inside the closure
        // are valid and checked against the declared return type.
        let prev_return_type = self.current_function_return_type.clone();
        let prev_function_info = self.current_function_info.clone();
        self.current_function_return_type =
            Some(declared_return.clone().unwrap_or(Type::any_placeholder()));
        self.current_function_info = Some(("<closure>".to_string(), span));

        // Enter a new scope for the closure body
        self.enter_scope();

        // Define params as locals in the closure scope
        for (param, ty) in params.iter().zip(param_types.iter()) {
            let symbol = crate::symbol::Symbol {
                name: param.name.name.clone(),
                ty: ty.clone(),
                mutable: false,
                kind: crate::symbol::SymbolKind::Parameter,
                span: param.name.span,
                exported: false,
            };
            let _ = self.symbol_table.define(symbol);
        }

        // Validate capture semantics for identifiers referenced in the body
        self.check_capture_semantics(body, span);

        // Check the body — for block bodies, extract the last expr type if present
        let body_type = match body {
            Expr::Block(block) => {
                for stmt in &block.statements {
                    self.check_statement(stmt);
                }
                // Infer type from the tail expression or last statement:
                // - Tail expr (bare expression at end of block, no semicolon) → its type
                // - Bare expr statement → use its type
                // - Return statement → use the returned expr type (check_statement already validated it)
                // - Anything else → Void
                if let Some(tail) = &block.tail_expr {
                    self.check_expr(tail)
                } else {
                    block
                        .statements
                        .last()
                        .and_then(|s| match s {
                            crate::ast::Stmt::Expr(e) => Some(self.check_expr(&e.expr)),
                            crate::ast::Stmt::Return(r) => r
                                .value
                                .as_ref()
                                .map(|e| self.check_expr(e))
                                .or(Some(Type::Void)),
                            _ => None,
                        })
                        .unwrap_or(Type::Void)
                }
            }
            _ => self.check_expr(body),
        };

        self.exit_scope();

        // Restore function context
        self.current_function_return_type = prev_return_type;
        self.current_function_info = prev_function_info;

        let return_type = match declared_return {
            Some(declared) => {
                if !self.is_assignable_with_traits(&body_type, &declared) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "closure body returns {} but declared return type is {}",
                                body_type.display_name(),
                                declared.display_name()
                            ),
                            span,
                        )
                        .with_label("return type mismatch"),
                    );
                }
                declared
            }
            None => body_type,
        };

        Type::Function {
            type_params: vec![],
            params: param_types,
            return_type: Box::new(return_type),
        }
    }

    /// Walk a closure body expression and emit diagnostics for invalid captures.
    ///
    /// Rules:
    /// - Copy types: captured by copy — always valid
    /// - Non-Copy types: captured by move — valid, caller loses ownership
    /// - `borrow`-annotated variables: **error** — borrows cannot outlive their scope
    fn check_capture_semantics(&mut self, expr: &Expr, closure_span: Span) {
        match expr {
            Expr::Identifier(id) => {
                // Check if this identifier is a borrow-annotated param in the enclosing fn
                if let Some(ownership) = self.current_fn_param_ownerships.get(&id.name) {
                    if matches!(ownership, Some(crate::ast::OwnershipAnnotation::Borrow)) {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3040",
                                format!(
                                    "cannot capture `{}` by reference in a closure — borrows cannot outlive their scope",
                                    id.name
                                ),
                                closure_span,
                            )
                            .with_label("borrow captured here")
                            .with_help(format!(
                                "cannot capture `borrow` parameter `{}` in a closure — borrows cannot outlive their scope.\n\
                                 Fix: change the parameter annotation to `own` (moved in) or `share` (shared ref),\n\
                                 or pass a copy of the value into the closure explicitly.",
                                id.name
                            )),
                        );
                    }
                }
            }
            Expr::Binary(b) => {
                self.check_capture_semantics(&b.left, closure_span);
                self.check_capture_semantics(&b.right, closure_span);
            }
            Expr::Unary(u) => self.check_capture_semantics(&u.expr, closure_span),
            Expr::Call(c) => {
                self.check_capture_semantics(&c.callee, closure_span);
                for arg in &c.args {
                    self.check_capture_semantics(arg, closure_span);
                }
            }
            Expr::TemplateString { parts, .. } => {
                for part in parts {
                    if let TemplatePart::Expression(expr) = part {
                        self.check_capture_semantics(expr, closure_span);
                    }
                }
            }
            Expr::Group(g) => self.check_capture_semantics(&g.expr, closure_span),
            Expr::Block(block) => {
                for stmt in &block.statements {
                    self.check_capture_semantics_stmt(stmt, closure_span);
                }
            }
            Expr::AnonFn { body, .. } => {
                self.check_capture_semantics(body, closure_span);
            }
            _ => {}
        }
    }

    fn check_capture_semantics_stmt(&mut self, stmt: &crate::ast::Stmt, closure_span: Span) {
        match stmt {
            crate::ast::Stmt::Expr(e) => {
                self.check_capture_semantics(&e.expr, closure_span);
            }
            crate::ast::Stmt::Return(r) => {
                if let Some(val) = &r.value {
                    self.check_capture_semantics(val, closure_span);
                }
            }
            crate::ast::Stmt::VarDecl(v) => {
                self.check_capture_semantics(&v.init, closure_span);
            }
            _ => {}
        }
    }
}
