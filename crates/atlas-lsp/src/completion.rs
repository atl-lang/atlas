//! Code completion helpers

use atlas_runtime::ast::*;
use atlas_runtime::symbol::SymbolTable;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position};

/// Completion items for ownership annotation keywords, shown in parameter position only.
pub fn ownership_annotation_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "own".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Ownership annotation".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Move semantics: caller's binding is invalidated after call.".to_string(),
            )),
            insert_text: Some("own ${1:name}: ${2:Type}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "borrow".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Ownership annotation".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Immutable reference: caller retains ownership, no mutation.".to_string(),
            )),
            insert_text: Some("borrow ${1:name}: ${2:Type}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "shared".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Ownership annotation".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Shared reference: Arc<T> semantics, requires shared<T> value.".to_string(),
            )),
            insert_text: Some("shared ${1:name}: ${2:Type}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Detect whether the cursor is inside a function parameter list.
///
/// Heuristic: walk backwards from the cursor byte offset. If we encounter `(`
/// before `fn`, we might be in a call. But if `fn` precedes `(`, we are in a
/// parameter definition context. Returns `true` when we are in param-definition
/// position (i.e., after `fn name(`).
pub fn is_in_param_position(text: &str, position: Position) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = position.line as usize;
    if line_idx >= lines.len() {
        return false;
    }

    // Build the prefix up to the cursor on the current line
    let line = lines[line_idx];
    let col = (position.character as usize).min(line.len());
    let prefix_on_line = &line[..col];

    // Collect all text up to the cursor (previous lines + prefix)
    let mut prefix = String::new();
    for l in lines.iter().take(line_idx) {
        prefix.push_str(l);
        prefix.push('\n');
    }
    prefix.push_str(prefix_on_line);

    // Walk backwards from the end of the prefix to find context
    let mut paren_depth: i32 = 0;
    let chars: Vec<char> = prefix.chars().collect();
    let mut i = chars.len();

    while i > 0 {
        i -= 1;
        match chars[i] {
            ')' => paren_depth += 1,
            '(' => {
                if paren_depth == 0 {
                    // We found the opening paren — look for `fn` before it
                    // Skip whitespace and the function name
                    let before_paren = &prefix[..chars[..i].iter().collect::<String>().len()];
                    let trimmed = before_paren.trim_end();
                    // The part before the paren should end with an identifier (function name)
                    // and before that should be `fn` keyword
                    let _word_end = trimmed.len();
                    let word_start = trimmed
                        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let before_name = trimmed[..word_start].trim_end();
                    return before_name.ends_with("fn");
                }
                paren_depth -= 1;
            }
            _ => {}
        }
    }

    false
}

/// Format an ownership annotation as a parameter prefix string
fn format_ownership(ownership: &Option<OwnershipAnnotation>) -> &'static str {
    match ownership {
        Some(OwnershipAnnotation::Own) => "own ",
        Some(OwnershipAnnotation::Borrow) => "borrow ",
        Some(OwnershipAnnotation::Shared) => "shared ",
        None => "",
    }
}

/// Generate completion items for keywords
pub fn keyword_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "let".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Variable declaration".to_string()),
            insert_text: Some("let ${1:name}: ${2:type} = ${3:value};".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "var".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Mutable variable declaration".to_string()),
            insert_text: Some("var ${1:name}: ${2:type} = ${3:value};".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "fn".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Function declaration".to_string()),
            insert_text: Some("fn ${1:name}(${2:params}) -> ${3:type} {\n\t${4}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "if".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("If statement".to_string()),
            insert_text: Some("if (${1:condition}) {\n\t${2}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "while".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("While loop".to_string()),
            insert_text: Some("while (${1:condition}) {\n\t${2}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "for".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("For loop".to_string()),
            insert_text: Some(
                "for (${1:init}; ${2:condition}; ${3:update}) {\n\t${4}\n}".to_string(),
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "return".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Return statement".to_string()),
            insert_text: Some("return ${1:value};".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "break".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Break statement".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "continue".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Continue statement".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "true".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Boolean true".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "false".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Boolean false".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "null".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Null value".to_string()),
            ..Default::default()
        },
    ]
}

/// Generate completion items for type keywords
pub fn type_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "number".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Number type".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "string".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("String type".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "bool".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Boolean type".to_string()),
            ..Default::default()
        },
    ]
}

/// Generate completion items for built-in functions
pub fn builtin_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "print".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn(value: any) -> null".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Print a value to stdout".to_string(),
            )),
            insert_text: Some("print(${1:value})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "len".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn(array: T[]) -> number".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Get the length of an array".to_string(),
            )),
            insert_text: Some("len(${1:array})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "push".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn(array: T[], value: T) -> null".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Add an element to the end of an array".to_string(),
            )),
            insert_text: Some("push(${1:array}, ${2:value})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "pop".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn(array: T[]) -> T | null".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Remove and return the last element of an array".to_string(),
            )),
            insert_text: Some("pop(${1:array})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Generate completion items from symbols in scope
pub fn symbol_completions(program: &Program, _symbols: &SymbolTable) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Add functions
    for item in &program.items {
        if let Item::Function(func) = item {
            let params: Vec<String> = func
                .params
                .iter()
                .map(|p| {
                    format!(
                        "{}{}: {:?}",
                        format_ownership(&p.ownership),
                        p.name.name,
                        p.type_ref
                    )
                })
                .collect();

            items.push(CompletionItem {
                label: func.name.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!(
                    "fn({}) -> {:?}",
                    params.join(", "),
                    func.return_type
                )),
                ..Default::default()
            });
        }
    }

    // Add top-level variables
    for item in &program.items {
        if let Item::Statement(Stmt::VarDecl(var)) = item {
            items.push(CompletionItem {
                label: var.name.name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some(format!("{:?}", var.type_ref)),
                ..Default::default()
            });
        }
    }

    items
}

/// Generate all completion items for the given cursor position.
///
/// `text` and `position` are used for context detection: ownership annotation
/// completions (`own`, `borrow`, `shared`) are only suggested when the cursor
/// is inside a function parameter list. Other completions are always included.
pub fn generate_completions(
    text: Option<&str>,
    position: Option<Position>,
    program: Option<&Program>,
    symbols: Option<&SymbolTable>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Always include keywords and builtins
    items.extend(keyword_completions());
    items.extend(type_completions());
    items.extend(builtin_completions());

    // Ownership annotations only in parameter position
    if let (Some(src), Some(pos)) = (text, position) {
        if is_in_param_position(src, pos) {
            items.extend(ownership_annotation_completions());
        }
    }

    // Add symbols from current document if available
    if let (Some(prog), Some(syms)) = (program, symbols) {
        items.extend(symbol_completions(prog, syms));
    }

    items
}
