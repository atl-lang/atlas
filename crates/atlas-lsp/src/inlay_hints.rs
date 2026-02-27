//! Inlay hint provider
//!
//! Provides inlay hints for:
//! - Type hints for variables with inferred types
//! - Parameter name hints for function calls

use atlas_runtime::ast::*;
use atlas_runtime::symbol::{SymbolKind as AtlasSymbolKind, SymbolTable};
use atlas_runtime::typechecker::inference::{infer_return_type, InferredReturn};
use atlas_runtime::types::Type;
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Position, Range};

use crate::symbols::offset_to_position;

/// Configuration for inlay hints
#[derive(Debug, Clone)]
pub struct InlayHintConfig {
    /// Show type hints for variables without explicit type annotations
    pub show_type_hints: bool,
    /// Show parameter name hints for function calls
    pub show_parameter_hints: bool,
    /// Show inferred return type hint on functions with omitted return type annotation
    pub show_inferred_return: bool,
    /// Maximum length for type hints before truncating
    pub max_type_length: usize,
    /// Skip type hints for obvious types (literals)
    pub skip_obvious_types: bool,
}

impl Default for InlayHintConfig {
    fn default() -> Self {
        Self {
            show_type_hints: true,
            show_parameter_hints: true,
            show_inferred_return: true,
            max_type_length: 25,
            skip_obvious_types: true,
        }
    }
}

/// Generate inlay hints for a document range
pub fn generate_inlay_hints(
    text: &str,
    range: Range,
    ast: Option<&Program>,
    symbols: Option<&SymbolTable>,
    config: &InlayHintConfig,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();

    if let Some(program) = ast {
        // Convert range to byte offsets for filtering
        let start_offset = position_to_offset_simple(text, range.start);
        let end_offset = position_to_offset_simple(text, range.end);

        for item in &program.items {
            extract_item_hints(
                text,
                item,
                symbols,
                config,
                start_offset,
                end_offset,
                &mut hints,
            );
        }
    }

    hints
}

/// Extract inlay hints from an AST item
fn extract_item_hints(
    text: &str,
    item: &Item,
    symbols: Option<&SymbolTable>,
    config: &InlayHintConfig,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    match item {
        Item::Function(func) => {
            // Inferred return type hint on unannotated functions
            if config.show_inferred_return && func.return_type.is_none() {
                emit_inferred_return_hint(text, func, symbols, start_offset, end_offset, hints);
            }
            // Hints for function body
            extract_block_hints(
                text,
                &func.body,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Item::Statement(stmt) => {
            extract_statement_hints(text, stmt, symbols, config, start_offset, end_offset, hints);
        }
        _ => {}
    }
}

/// Extract inlay hints from a block
fn extract_block_hints(
    text: &str,
    block: &Block,
    symbols: Option<&SymbolTable>,
    config: &InlayHintConfig,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    for stmt in &block.statements {
        extract_statement_hints(text, stmt, symbols, config, start_offset, end_offset, hints);
    }
}

/// Extract inlay hints from a statement
fn extract_statement_hints(
    text: &str,
    stmt: &Stmt,
    symbols: Option<&SymbolTable>,
    config: &InlayHintConfig,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    match stmt {
        Stmt::VarDecl(var) => {
            // Type hints for variables without explicit types
            if config.show_type_hints && var.type_ref.is_none() {
                // Check if in range
                if var.span.start >= start_offset && var.span.start <= end_offset {
                    // Skip obvious types if configured
                    if !config.skip_obvious_types || !is_obvious_type(&var.init) {
                        // Try to get type from symbol table
                        if let Some(type_str) = get_variable_type(symbols, &var.name.name) {
                            let truncated = truncate_type(&type_str, config.max_type_length);
                            let position = offset_to_position(text, var.name.span.end);

                            hints.push(InlayHint {
                                position,
                                label: InlayHintLabel::String(format!(": {}", truncated)),
                                kind: Some(InlayHintKind::TYPE),
                                text_edits: None,
                                tooltip: if truncated != type_str {
                                    Some(tower_lsp::lsp_types::InlayHintTooltip::String(type_str))
                                } else {
                                    None
                                },
                                padding_left: Some(false),
                                padding_right: Some(true),
                                data: None,
                            });
                        }
                    }
                }
            }

            // Check initializer for function calls
            extract_expression_hints(
                text,
                &var.init,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Stmt::FunctionDecl(func) => {
            // Inferred return type hint on unannotated nested functions
            if config.show_inferred_return && func.return_type.is_none() {
                emit_inferred_return_hint(text, func, symbols, start_offset, end_offset, hints);
            }
            extract_block_hints(
                text,
                &func.body,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Stmt::If(if_stmt) => {
            extract_expression_hints(
                text,
                &if_stmt.cond,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
            extract_block_hints(
                text,
                &if_stmt.then_block,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
            if let Some(else_block) = &if_stmt.else_block {
                extract_block_hints(
                    text,
                    else_block,
                    symbols,
                    config,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        Stmt::While(while_stmt) => {
            extract_expression_hints(
                text,
                &while_stmt.cond,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
            extract_block_hints(
                text,
                &while_stmt.body,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Stmt::For(for_stmt) => {
            extract_block_hints(
                text,
                &for_stmt.body,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Stmt::ForIn(for_in) => {
            extract_expression_hints(
                text,
                &for_in.iterable,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
            extract_block_hints(
                text,
                &for_in.body,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Stmt::Return(ret) => {
            if let Some(expr) = &ret.value {
                extract_expression_hints(
                    text,
                    expr,
                    symbols,
                    config,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        Stmt::Expr(expr_stmt) => {
            extract_expression_hints(
                text,
                &expr_stmt.expr,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        _ => {}
    }
}

/// Extract inlay hints from an expression
fn extract_expression_hints(
    text: &str,
    expr: &Expr,
    symbols: Option<&SymbolTable>,
    config: &InlayHintConfig,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    // Check if in range
    let expr_span = expr.span();
    if expr_span.start < start_offset || expr_span.start > end_offset {
        return;
    }

    match expr {
        Expr::Call(call) => {
            // Parameter name hints
            if config.show_parameter_hints && !call.args.is_empty() {
                if let Some(param_names) = get_function_params(symbols, &call.callee) {
                    for (i, arg) in call.args.iter().enumerate() {
                        if let Some(param_name) = param_names.get(i) {
                            // Skip if argument is already named or obvious
                            if !is_obvious_argument(arg, param_name) {
                                let position = offset_to_position(text, arg.span().start);

                                hints.push(InlayHint {
                                    position,
                                    label: InlayHintLabel::String(format!("{}:", param_name)),
                                    kind: Some(InlayHintKind::PARAMETER),
                                    text_edits: None,
                                    tooltip: None,
                                    padding_left: Some(false),
                                    padding_right: Some(true),
                                    data: None,
                                });
                            }
                        }
                    }
                }
            }

            // Recursively check arguments
            for arg in &call.args {
                extract_expression_hints(
                    text,
                    arg,
                    symbols,
                    config,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        Expr::Member(member) => {
            // Method call parameter hints
            if config.show_parameter_hints {
                if let Some(args) = &member.args {
                    for arg in args {
                        extract_expression_hints(
                            text,
                            arg,
                            symbols,
                            config,
                            start_offset,
                            end_offset,
                            hints,
                        );
                    }
                }
            }
            extract_expression_hints(
                text,
                &member.target,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Expr::Binary(bin) => {
            extract_expression_hints(
                text,
                &bin.left,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
            extract_expression_hints(
                text,
                &bin.right,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Expr::Unary(unary) => {
            extract_expression_hints(
                text,
                &unary.expr,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Expr::ArrayLiteral(arr) => {
            for elem in &arr.elements {
                extract_expression_hints(
                    text,
                    elem,
                    symbols,
                    config,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        Expr::Index(index) => {
            extract_expression_hints(
                text,
                &index.target,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
            extract_expression_hints(
                text,
                &index.index,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Expr::Group(group) => {
            extract_expression_hints(
                text,
                &group.expr,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
        }
        Expr::Match(match_expr) => {
            extract_expression_hints(
                text,
                &match_expr.scrutinee,
                symbols,
                config,
                start_offset,
                end_offset,
                hints,
            );
            for arm in &match_expr.arms {
                extract_expression_hints(
                    text,
                    &arm.body,
                    symbols,
                    config,
                    start_offset,
                    end_offset,
                    hints,
                );
            }
        }
        _ => {}
    }
}

/// Emit a `→ T` inlay hint at the start of a function body for unannotated return types.
///
/// Uses `infer_return_type` directly on the function body so the hint reflects the actual
/// inferred type regardless of whether the symbol table was updated by the typechecker.
fn emit_inferred_return_hint(
    text: &str,
    func: &FunctionDecl,
    _symbols: Option<&SymbolTable>,
    start_offset: usize,
    end_offset: usize,
    hints: &mut Vec<InlayHint>,
) {
    // Only emit if the function body is within the requested range
    if func.body.span.start < start_offset || func.body.span.start > end_offset {
        return;
    }

    let ret_type_str = match infer_return_type(&func.body) {
        InferredReturn::Uniform(ty) => {
            let s = format_type(&ty);
            if s == "?" || s == "void" || s == "unknown" {
                return;
            }
            s
        }
        _ => return,
    };

    let truncated = truncate_type(&ret_type_str, 25);
    // Place the hint at the opening brace of the function body
    let position = offset_to_position(text, func.body.span.start);

    hints.push(InlayHint {
        position,
        label: InlayHintLabel::String(format!("→ {} ", truncated)),
        kind: Some(InlayHintKind::TYPE),
        text_edits: None,
        tooltip: if truncated != ret_type_str {
            Some(tower_lsp::lsp_types::InlayHintTooltip::String(ret_type_str))
        } else {
            None
        },
        padding_left: Some(true),
        padding_right: Some(false),
        data: None,
    });
}

/// Check if a type is obvious from the initializer
fn is_obvious_type(expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(_, _) | Expr::ArrayLiteral(_))
}

/// Check if an argument is obvious (same name as parameter or literal)
fn is_obvious_argument(arg: &Expr, param_name: &str) -> bool {
    match arg {
        // If argument is an identifier with same name as parameter
        Expr::Identifier(ident) => ident.name.eq_ignore_ascii_case(param_name),
        // Literals are generally obvious
        Expr::Literal(_, _) => true,
        _ => false,
    }
}

/// Get variable type from symbol table
fn get_variable_type(symbols: Option<&SymbolTable>, name: &str) -> Option<String> {
    let symbols = symbols?;
    let symbol = symbols.lookup(name)?;

    if symbol.kind == AtlasSymbolKind::Variable || symbol.kind == AtlasSymbolKind::Parameter {
        Some(format_type(&symbol.ty))
    } else {
        None
    }
}

/// Get function parameter names
fn get_function_params(symbols: Option<&SymbolTable>, callee: &Expr) -> Option<Vec<String>> {
    // Extract function name from callee
    let func_name = match callee {
        Expr::Identifier(id) => &id.name,
        _ => return None,
    };

    // Check symbol table for function
    let symbols = symbols?;
    let symbol = symbols.lookup(func_name)?;

    if symbol.kind == AtlasSymbolKind::Function {
        // Try to extract parameter names from the function type
        if let Type::Function { params, .. } = &symbol.ty {
            // The Type doesn't store parameter names, so we'd need to look at the AST
            // For now, return generic names based on parameter count
            return Some((0..params.len()).map(|i| format!("arg{}", i)).collect());
        }
    }

    None
}

/// Format a type for display
fn format_type(ty: &Type) -> String {
    match ty {
        Type::Number => "number".to_string(),
        Type::String => "string".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Null => "null".to_string(),
        Type::Void => "void".to_string(),
        Type::Never => "never".to_string(),
        Type::Array(inner) => format!("{}[]", format_type(inner)),
        Type::Function {
            params,
            return_type,
            ..
        } => {
            let param_strs: Vec<String> = params.iter().map(format_type).collect();
            format!(
                "({}) -> {}",
                param_strs.join(", "),
                format_type(return_type)
            )
        }
        Type::Generic { name, type_args } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let arg_strs: Vec<String> = type_args.iter().map(format_type).collect();
                format!("{}<{}>", name, arg_strs.join(", "))
            }
        }
        Type::Alias { name, .. } => name.clone(),
        Type::TypeParameter { name } => name.clone(),
        Type::JsonValue => "JsonValue".to_string(),
        Type::Union(types) => {
            let formatted: Vec<String> = types.iter().map(format_type).collect();
            formatted.join(" | ")
        }
        Type::Intersection(types) => {
            let formatted: Vec<String> = types.iter().map(format_type).collect();
            formatted.join(" & ")
        }
        Type::Structural { members } => {
            let field_strs: Vec<String> = members
                .iter()
                .map(|m| format!("{}: {}", m.name, format_type(&m.ty)))
                .collect();
            format!("{{ {} }}", field_strs.join(", "))
        }
        Type::Unknown | Type::Extern(_) => "?".to_string(),
    }
}

/// Truncate a type string if too long
fn truncate_type(type_str: &str, max_len: usize) -> String {
    if type_str.len() <= max_len {
        type_str.to_string()
    } else {
        format!("{}...", &type_str[..max_len.saturating_sub(3)])
    }
}

/// Simple position to offset conversion
fn position_to_offset_simple(text: &str, position: Position) -> usize {
    let mut offset = 0;
    let mut line = 0u32;

    for ch in text.chars() {
        if line == position.line {
            break;
        }
        if ch == '\n' {
            line += 1;
        }
        offset += ch.len_utf8();
    }

    // Add character offset
    for (col, ch) in text[offset..].chars().enumerate() {
        if col as u32 >= position.character || ch == '\n' {
            break;
        }
        offset += ch.len_utf8();
    }

    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_obvious_type_literals() {
        use atlas_runtime::span::Span;

        let lit = Expr::Literal(Literal::Number(42.0), Span::dummy());
        assert!(is_obvious_type(&lit));
    }

    #[test]
    fn test_is_obvious_type_complex() {
        use atlas_runtime::span::Span;

        let call = Expr::Call(CallExpr {
            callee: Box::new(Expr::Identifier(Identifier {
                name: "foo".to_string(),
                span: Span::dummy(),
            })),
            args: vec![],
            span: Span::dummy(),
        });
        assert!(!is_obvious_type(&call));
    }

    #[test]
    fn test_truncate_type_short() {
        assert_eq!(truncate_type("number", 25), "number");
    }

    #[test]
    fn test_truncate_type_long() {
        let long_type = "HashMap<string, Array<number>>";
        let truncated = truncate_type(long_type, 20);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 20);
    }

    #[test]
    fn test_format_type_simple() {
        assert_eq!(format_type(&Type::Number), "number");
        assert_eq!(format_type(&Type::String), "string");
        assert_eq!(format_type(&Type::Bool), "bool");
    }

    #[test]
    fn test_format_type_array() {
        let arr_type = Type::Array(Box::new(Type::Number));
        assert_eq!(format_type(&arr_type), "number[]");
    }

    #[test]
    fn test_format_type_function() {
        let fn_type = Type::Function {
            type_params: vec![],
            params: vec![Type::String, Type::Number],
            return_type: Box::new(Type::Bool),
        };
        assert_eq!(format_type(&fn_type), "(string, number) -> bool");
    }

    #[test]
    fn test_inlay_hint_config_default() {
        let config = InlayHintConfig::default();
        assert!(config.show_type_hints);
        assert!(config.show_parameter_hints);
        assert_eq!(config.max_type_length, 25);
        assert!(config.skip_obvious_types);
    }
}
