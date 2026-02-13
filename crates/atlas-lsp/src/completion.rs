//! Code completion helpers

use atlas_runtime::ast::*;
use atlas_runtime::symbol::SymbolTable;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

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
                .map(|p| format!("{}: {:?}", p.name.name, p.type_ref))
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

/// Generate all completion items
pub fn generate_completions(
    program: Option<&Program>,
    symbols: Option<&SymbolTable>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Always include keywords and builtins
    items.extend(keyword_completions());
    items.extend(type_completions());
    items.extend(builtin_completions());

    // Add symbols from current document if available
    if let (Some(prog), Some(syms)) = (program, symbols) {
        items.extend(symbol_completions(prog, syms));
    }

    items
}
