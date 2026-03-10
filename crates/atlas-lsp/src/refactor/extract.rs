//! Extract variable and extract function refactoring

use super::{
    create_workspace_edit, extract_all_names, generate_unique_name, validate_new_name,
    RefactorError, RefactorResult,
};
use atlas_runtime::ast::*;
use atlas_runtime::symbol::{SymbolKind, SymbolTable};
use atlas_runtime::typechecker::inference::{infer_expr_type, infer_return_type, InferredReturn};
use atlas_runtime::types::{Type, ANY_TYPE_PARAM};
use atlas_runtime::{Lexer, Parser};
use tower_lsp::lsp_types::*;

/// Extract the selected expression to a new variable
///
/// Analyzes the selected range, extracts the expression, generates a unique name,
/// inserts a `let` binding before the usage, and replaces the expression with a reference.
pub fn extract_variable(
    uri: &Url,
    range: Range,
    text: &str,
    program: &Program,
    _symbols: Option<&SymbolTable>,
    suggested_name: Option<&str>,
) -> RefactorResult {
    // Find the expression at the given range
    let expr_text = extract_text_at_range(text, range)?;

    // Generate a unique name
    let existing_names = extract_all_names(program);
    let base_name = suggested_name.unwrap_or("extracted");
    let var_name = generate_unique_name(base_name, &existing_names);

    // Validate the name
    validate_new_name(&var_name)?;

    // Create the variable declaration
    let var_decl = format!("let {} = {};", var_name, expr_text);

    // Find insertion point (beginning of the current statement/line)
    let insert_position = Position {
        line: range.start.line,
        character: 0,
    };

    // Create text edits
    let mut edits = vec![
        // Insert variable declaration
        TextEdit {
            range: Range {
                start: insert_position,
                end: insert_position,
            },
            new_text: format!("{}\n", var_decl),
        },
        // Replace expression with variable reference
        TextEdit {
            range,
            new_text: var_name.clone(),
        },
    ];

    // Sort edits by position (last to first) to avoid invalidating positions
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));

    Ok(create_workspace_edit(uri, edits))
}

/// Extract the selected statements to a new function
///
/// Analyzes the selected range, determines captured variables (which become parameters),
/// infers the return type, generates a function signature, inserts the function definition,
/// and replaces the selection with a function call.
pub fn extract_function(
    uri: &Url,
    range: Range,
    text: &str,
    program: &Program,
    symbols: Option<&SymbolTable>,
    suggested_name: Option<&str>,
) -> RefactorResult {
    // Extract the selected statements
    let statements_text = extract_text_at_range(text, range)?;

    // Generate a unique function name
    let existing_names = extract_all_names(program);
    let base_name = suggested_name.unwrap_or("extracted_function");
    let func_name = generate_unique_name(base_name, &existing_names);

    // Validate the name
    validate_new_name(&func_name)?;

    let extracted_block = parse_block_from_snippet(&statements_text)?;

    let mut captured_vars = analyze_captured_variables(&extracted_block);
    if let Some(symbols) = symbols {
        captured_vars.retain(|name| match symbols.lookup(name) {
            Some(symbol) => !matches!(symbol.kind, SymbolKind::Function | SymbolKind::Builtin),
            None => true,
        });
    }

    let params_text = if captured_vars.is_empty() {
        String::new()
    } else {
        captured_vars.join(", ")
    };

    let return_type_text = infer_return_type_text(&extracted_block);

    let signature = if let Some(return_type) = return_type_text {
        format!("fn {}({}) -> {}", func_name, params_text, return_type)
    } else {
        format!("fn {}({})", func_name, params_text)
    };

    let func_decl = format!(
        "{} {{\n{}\n}}\n\n",
        signature,
        indent_text(&statements_text, 1)
    );

    // Find insertion point for the function (after the current function or at file start)
    let insert_position = Position {
        line: 0,
        character: 0,
    };

    // Create text edits
    let edits = vec![
        // Insert function declaration
        TextEdit {
            range: Range {
                start: insert_position,
                end: insert_position,
            },
            new_text: func_decl,
        },
        // Replace selection with function call
        TextEdit {
            range,
            new_text: if captured_vars.is_empty() {
                format!("{}();", func_name)
            } else {
                format!("{}({});", func_name, captured_vars.join(", "))
            },
        },
    ];

    Ok(create_workspace_edit(uri, edits))
}

fn parse_block_from_snippet(snippet: &str) -> Result<Block, RefactorError> {
    let wrapped = format!("fn __extract_temp() {{\n{}\n}}", snippet);
    let mut lexer = Lexer::new(&wrapped);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    for item in program.items {
        if let Item::Function(func) = item {
            return Ok(func.body);
        }
    }

    Err(RefactorError::AnalysisFailed(
        "Unable to parse selected statements".to_string(),
    ))
}

fn analyze_captured_variables(block: &Block) -> Vec<String> {
    use std::collections::{HashSet, VecDeque};

    let mut free_vars = HashSet::new();
    let mut scopes: VecDeque<HashSet<String>> = VecDeque::new();
    scopes.push_back(HashSet::new());

    collect_free_vars_block(block, &mut scopes, &mut free_vars);

    let mut vars: Vec<String> = free_vars.into_iter().collect();
    vars.sort();
    vars
}

fn collect_free_vars_block(
    block: &Block,
    scopes: &mut std::collections::VecDeque<std::collections::HashSet<String>>,
    free_vars: &mut std::collections::HashSet<String>,
) {
    for stmt in &block.statements {
        collect_free_vars_stmt(stmt, scopes, free_vars);
    }

    if let Some(tail_expr) = &block.tail_expr {
        collect_free_vars_expr(tail_expr, scopes, free_vars);
    }
}

fn collect_free_vars_stmt(
    stmt: &Stmt,
    scopes: &mut std::collections::VecDeque<std::collections::HashSet<String>>,
    free_vars: &mut std::collections::HashSet<String>,
) {
    match stmt {
        Stmt::VarDecl(var_decl) => {
            collect_free_vars_expr(&var_decl.init, scopes, free_vars);
            if let Some(scope) = scopes.back_mut() {
                scope.insert(var_decl.name.name.clone());
            }
        }
        Stmt::LetDestructure(_) => { /* B15-P06 */ }
        Stmt::FunctionDecl(func) => {
            if let Some(scope) = scopes.back_mut() {
                scope.insert(func.name.name.clone());
            }
            scopes.push_back(
                func.params
                    .iter()
                    .map(|param| param.name.name.clone())
                    .collect(),
            );
            collect_free_vars_block(&func.body, scopes, free_vars);
            scopes.pop_back();
        }
        Stmt::If(if_stmt) => {
            collect_free_vars_expr(&if_stmt.cond, scopes, free_vars);
            scopes.push_back(std::collections::HashSet::new());
            collect_free_vars_block(&if_stmt.then_block, scopes, free_vars);
            scopes.pop_back();

            if let Some(else_block) = &if_stmt.else_block {
                scopes.push_back(std::collections::HashSet::new());
                collect_free_vars_block(else_block, scopes, free_vars);
                scopes.pop_back();
            }
        }
        Stmt::While(while_stmt) => {
            collect_free_vars_expr(&while_stmt.cond, scopes, free_vars);
            scopes.push_back(std::collections::HashSet::new());
            collect_free_vars_block(&while_stmt.body, scopes, free_vars);
            scopes.pop_back();
        }
        Stmt::ForIn(for_in_stmt) => {
            scopes.push_back(std::collections::HashSet::new());
            collect_free_vars_expr(&for_in_stmt.iterable, scopes, free_vars);
            if let Some(scope) = scopes.back_mut() {
                scope.insert(for_in_stmt.variable.name.clone());
            }
            collect_free_vars_block(&for_in_stmt.body, scopes, free_vars);
            scopes.pop_back();
        }
        Stmt::Return(ret_stmt) => {
            if let Some(expr) = &ret_stmt.value {
                collect_free_vars_expr(expr, scopes, free_vars);
            }
        }
        Stmt::Expr(expr_stmt) => {
            collect_free_vars_expr(&expr_stmt.expr, scopes, free_vars);
        }
        Stmt::Assign(assign) => {
            collect_free_vars_assign_target(&assign.target, scopes, free_vars);
            collect_free_vars_expr(&assign.value, scopes, free_vars);
        }
        Stmt::CompoundAssign(assign) => {
            collect_free_vars_assign_target(&assign.target, scopes, free_vars);
            collect_free_vars_expr(&assign.value, scopes, free_vars);
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

fn collect_free_vars_assign_target(
    target: &AssignTarget,
    scopes: &mut std::collections::VecDeque<std::collections::HashSet<String>>,
    free_vars: &mut std::collections::HashSet<String>,
) {
    match target {
        AssignTarget::Name(name) => {
            if !is_bound(name.name.as_str(), scopes) {
                free_vars.insert(name.name.clone());
            }
        }
        AssignTarget::Index { target, index, .. } => {
            collect_free_vars_expr(target, scopes, free_vars);
            collect_free_vars_expr(index, scopes, free_vars);
        }
        AssignTarget::Member { target, .. } => {
            collect_free_vars_expr(target, scopes, free_vars);
        }
    }
}

fn collect_free_vars_expr(
    expr: &Expr,
    scopes: &mut std::collections::VecDeque<std::collections::HashSet<String>>,
    free_vars: &mut std::collections::HashSet<String>,
) {
    match expr {
        Expr::Identifier(ident) => {
            if !is_bound(ident.name.as_str(), scopes) {
                free_vars.insert(ident.name.clone());
            }
        }
        Expr::Unary(unary) => collect_free_vars_expr(&unary.expr, scopes, free_vars),
        Expr::Binary(binary) => {
            collect_free_vars_expr(&binary.left, scopes, free_vars);
            collect_free_vars_expr(&binary.right, scopes, free_vars);
        }
        Expr::Call(call) => {
            collect_free_vars_expr(&call.callee, scopes, free_vars);
            for arg in &call.args {
                collect_free_vars_expr(arg, scopes, free_vars);
            }
        }
        Expr::Index(index) => {
            collect_free_vars_expr(&index.target, scopes, free_vars);
            match &index.index {
                IndexValue::Single(expr) => collect_free_vars_expr(expr, scopes, free_vars),
            }
        }
        Expr::Member(member) => collect_free_vars_expr(&member.target, scopes, free_vars),
        Expr::ArrayLiteral(array) => {
            for elem in &array.elements {
                collect_free_vars_expr(elem, scopes, free_vars);
            }
        }
        Expr::ObjectLiteral(obj) => {
            for entry in &obj.entries {
                collect_free_vars_expr(&entry.value, scopes, free_vars);
            }
        }
        Expr::StructExpr(struct_expr) => {
            for field in &struct_expr.fields {
                collect_free_vars_expr(&field.value, scopes, free_vars);
            }
        }
        Expr::TemplateString { parts, .. } => {
            for part in parts {
                if let TemplatePart::Expression(expr) = part {
                    collect_free_vars_expr(expr, scopes, free_vars);
                }
            }
        }
        Expr::Range { start, end, .. } => {
            if let Some(start) = start {
                collect_free_vars_expr(start, scopes, free_vars);
            }
            if let Some(end) = end {
                collect_free_vars_expr(end, scopes, free_vars);
            }
        }
        Expr::Group(group) => collect_free_vars_expr(&group.expr, scopes, free_vars),
        Expr::Match(match_expr) => {
            collect_free_vars_expr(&match_expr.scrutinee, scopes, free_vars);
            for arm in &match_expr.arms {
                scopes.push_back(std::collections::HashSet::new());
                collect_pattern_bindings(&arm.pattern, scopes);
                if let Some(guard) = &arm.guard {
                    collect_free_vars_expr(guard, scopes, free_vars);
                }
                collect_free_vars_expr(&arm.body, scopes, free_vars);
                scopes.pop_back();
            }
        }
        Expr::Try(try_expr) => collect_free_vars_expr(&try_expr.expr, scopes, free_vars),
        Expr::AnonFn { params, body, .. } => {
            scopes.push_back(params.iter().map(|param| param.name.name.clone()).collect());
            collect_free_vars_expr(body, scopes, free_vars);
            scopes.pop_back();
        }
        Expr::Block(block) => {
            scopes.push_back(std::collections::HashSet::new());
            collect_free_vars_block(block, scopes, free_vars);
            scopes.pop_back();
        }
        Expr::EnumVariant(enum_variant) => {
            if let Some(args) = &enum_variant.args {
                for arg in args {
                    collect_free_vars_expr(arg, scopes, free_vars);
                }
            }
        }
        Expr::TupleLiteral { elements, .. } => {
            for elem in elements {
                collect_free_vars_expr(elem, scopes, free_vars);
            }
        }
        Expr::Await { expr, .. } => {
            collect_free_vars_expr(expr, scopes, free_vars);
        }
        Expr::Literal(_, _) => {}
    }
}

fn collect_pattern_bindings(
    pattern: &atlas_runtime::ast::Pattern,
    scopes: &mut std::collections::VecDeque<std::collections::HashSet<String>>,
) {
    match pattern {
        atlas_runtime::ast::Pattern::Variable(ident) => {
            if let Some(scope) = scopes.back_mut() {
                scope.insert(ident.name.clone());
            }
        }
        atlas_runtime::ast::Pattern::Constructor { args, .. } => {
            for arg in args {
                collect_pattern_bindings(arg, scopes);
            }
        }
        atlas_runtime::ast::Pattern::Tuple { elements, .. } => {
            for elem in elements {
                collect_pattern_bindings(elem, scopes);
            }
        }
        atlas_runtime::ast::Pattern::Array { elements, .. } => {
            for elem in elements {
                collect_pattern_bindings(elem, scopes);
            }
        }
        atlas_runtime::ast::Pattern::Or(patterns, _) => {
            for pat in patterns {
                collect_pattern_bindings(pat, scopes);
            }
        }
        atlas_runtime::ast::Pattern::EnumVariant { args, .. } => {
            for arg in args {
                collect_pattern_bindings(arg, scopes);
            }
        }
        atlas_runtime::ast::Pattern::Literal(_, _) | atlas_runtime::ast::Pattern::Wildcard(_) => {}
    }
}

fn is_bound(
    name: &str,
    scopes: &std::collections::VecDeque<std::collections::HashSet<String>>,
) -> bool {
    scopes.iter().rev().any(|scope| scope.contains(name))
}

fn infer_return_type_text(block: &Block) -> Option<String> {
    let inferred = infer_return_type(block);
    match inferred {
        InferredReturn::Void => {
            if let Some(tail_expr) = &block.tail_expr {
                let ty = infer_expr_type(tail_expr);
                return format_type_for_signature(&ty);
            }
            Some("void".to_string())
        }
        InferredReturn::Uniform(ty) => format_type_for_signature(&ty),
        InferredReturn::Inconsistent { .. } => None,
    }
}

fn format_type_for_signature(ty: &Type) -> Option<String> {
    match ty {
        Type::Unknown => None,
        Type::TypeParameter { name } if name == ANY_TYPE_PARAM => None,
        Type::Never => Some("never".to_string()),
        Type::Number => Some("number".to_string()),
        Type::String => Some("string".to_string()),
        Type::Bool => Some("bool".to_string()),
        Type::Null => Some("null".to_string()),
        Type::Void => Some("void".to_string()),
        Type::Array(elem) => format_type_for_signature(elem).map(|inner| format!("{}[]", inner)),
        Type::Range => Some("range".to_string()),
        Type::Function {
            params,
            return_type,
            ..
        } => {
            let mut rendered = Vec::with_capacity(params.len());
            for param in params {
                rendered.push(format_type_for_signature(param)?);
            }
            let return_text = format_type_for_signature(return_type)?;
            Some(format!("({}) -> {}", rendered.join(", "), return_text))
        }
        Type::JsonValue => Some("json".to_string()),
        Type::Generic { name, type_args } => {
            if type_args.is_empty() {
                Some(name.clone())
            } else {
                let mut args = Vec::with_capacity(type_args.len());
                for arg in type_args {
                    args.push(format_type_for_signature(arg)?);
                }
                Some(format!("{}<{}>", name, args.join(", ")))
            }
        }
        Type::Alias {
            name, type_args, ..
        } => {
            if type_args.is_empty() {
                Some(name.clone())
            } else {
                let mut args = Vec::with_capacity(type_args.len());
                for arg in type_args {
                    args.push(format_type_for_signature(arg)?);
                }
                Some(format!("{}<{}>", name, args.join(", ")))
            }
        }
        Type::TypeParameter { name } => Some(name.clone()),
        Type::Extern(extern_type) => Some(extern_type.display_name().to_string()),
        Type::Union(members) => {
            let mut parts = Vec::with_capacity(members.len());
            for member in members {
                parts.push(format_type_for_signature(member)?);
            }
            Some(parts.join(" | "))
        }
        Type::Intersection(members) => {
            let mut parts = Vec::with_capacity(members.len());
            for member in members {
                parts.push(format_type_for_signature(member)?);
            }
            Some(parts.join(" & "))
        }
        Type::Structural { members } => {
            let mut parts = Vec::with_capacity(members.len());
            for member in members {
                let ty_text = format_type_for_signature(&member.ty)?;
                parts.push(format!("{}: {}", member.name, ty_text));
            }
            Some(format!("{{ {} }}", parts.join(", ")))
        }
        Type::Tuple(elements) => {
            let mut parts = Vec::with_capacity(elements.len());
            for elem in elements {
                parts.push(format_type_for_signature(elem)?);
            }
            Some(format!("({})", parts.join(", ")))
        }
        Type::TraitObject { name } => Some(name.clone()),
    }
}

/// Extract text from source at the given range
fn extract_text_at_range(text: &str, range: Range) -> Result<String, RefactorError> {
    let lines: Vec<&str> = text.lines().collect();

    if range.start.line as usize >= lines.len() || range.end.line as usize >= lines.len() {
        return Err(RefactorError::InvalidSelection(
            "Range exceeds file bounds".to_string(),
        ));
    }

    if range.start.line == range.end.line {
        // Single line selection
        let line = lines[range.start.line as usize];
        let start = range.start.character as usize;
        let end = range.end.character as usize;

        if start >= line.len() || end > line.len() || start > end {
            return Err(RefactorError::InvalidSelection(
                "Invalid range within line".to_string(),
            ));
        }

        Ok(line[start..end].to_string())
    } else {
        // Multi-line selection
        let mut result = String::new();

        // First line
        let first_line = lines[range.start.line as usize];
        let start = range.start.character as usize;
        if start < first_line.len() {
            result.push_str(&first_line[start..]);
        }
        result.push('\n');

        // Middle lines
        for line in lines
            .iter()
            .skip(range.start.line as usize + 1)
            .take((range.end.line as usize).saturating_sub(range.start.line as usize + 1))
        {
            result.push_str(line);
            result.push('\n');
        }

        // Last line
        let last_line = lines[range.end.line as usize];
        let end = range.end.character as usize;
        if end <= last_line.len() {
            result.push_str(&last_line[..end]);
        }

        Ok(result)
    }
}

/// Indent text by the given number of levels (4 spaces per level)
fn indent_text(text: &str, levels: usize) -> String {
    let indent = "    ".repeat(levels);
    text.lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_at_range_single_line() {
        let text = "let x = 1 + 2;";
        let range = Range {
            start: Position {
                line: 0,
                character: 8,
            },
            end: Position {
                line: 0,
                character: 13,
            },
        };
        let result = extract_text_at_range(text, range).unwrap();
        assert_eq!(result, "1 + 2");
    }

    #[test]
    fn test_extract_text_at_range_multi_line() {
        let text = "let x = {\n    1 + 2\n};";
        let range = Range {
            start: Position {
                line: 0,
                character: 8,
            },
            end: Position {
                line: 2,
                character: 1,
            },
        };
        let result = extract_text_at_range(text, range).unwrap();
        assert_eq!(result, "{\n    1 + 2\n}");
    }

    #[test]
    fn test_indent_text() {
        let text = "let x = 1;\nlet y = 2;";
        let result = indent_text(text, 1);
        assert_eq!(result, "    let x = 1;\n    let y = 2;");
    }
}
