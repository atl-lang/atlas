//! Navigation feature tests

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

#[tokio::test]
async fn test_document_symbols() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    // Open document with functions and variables
    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}

let mut counter: number = 0;

fn increment() -> number {
    counter = counter + 1;
    return counter;
}
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source.to_string(),
        },
    };
    server.did_open(open_params).await;

    // Get document symbols
    let symbol_params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.document_symbol(symbol_params).await.unwrap();
    assert!(result.is_some());

    // Should have symbols for functions and variables
    if let Some(DocumentSymbolResponse::Nested(symbols)) = result {
        assert!(!symbols.is_empty());
        // We should have at least the functions and variable
        assert!(symbols.len() >= 2);
    }
}

#[tokio::test]
async fn test_hover_on_function() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source.to_string(),
        },
    };
    server.did_open(open_params).await;

    // Hover over function name (line 1, column 4 - "add")
    let hover_params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 4,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let result = server.hover(hover_params).await.unwrap();
    // Hover should return function signature information
    assert!(result.is_some());
}

#[tokio::test]
async fn test_goto_definition_placeholder() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
let mut result: number = add(1, 2);
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source.to_string(),
        },
    };
    server.did_open(open_params).await;

    // Try to go to definition of "add" on line 5
    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 4,
                character: 22,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.goto_definition(params).await.unwrap();
    // Currently returns None (TODO: implement with position info)
    assert!(result.is_none());
}

#[tokio::test]
async fn test_references_placeholder() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
let mut x: number = add(1, 2);
let mut y: number = add(3, 4);
"#;

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source.to_string(),
        },
    };
    server.did_open(open_params).await;

    // Find references to "add"
    let params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 4,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: ReferenceContext {
            include_declaration: true,
        },
    };

    let result = server.references(params).await.unwrap();
    // Should find references: declaration + 2 calls = 3 total
    assert!(result.is_some());
    let locations = result.unwrap();
    assert!(locations.len() >= 2); // At least the 2 call sites, maybe declaration too
}

#[tokio::test]
async fn test_document_symbols_empty_file() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let server = service.inner();

    let uri = Url::parse("file:///empty.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

    let symbol_params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let result = server.document_symbol(symbol_params).await.unwrap();
    // Empty file should have no symbols
    if let Some(DocumentSymbolResponse::Nested(symbols)) = result {
        assert!(symbols.is_empty());
    }
}

#[tokio::test]
async fn test_symbol_index_find_definition_at() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let _server = service.inner();

    let uri = Url::parse("file:///index_test.atl").unwrap();
    let source = r#"fn add(a: number) -> number {
    return a;
}

let mut result: number = add(1);
"#;

    let mut lexer = atlas_runtime::Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::Parser::new(tokens);
    let (program, _diagnostics) = parser.parse();

    let mut index = atlas_lsp::index::SymbolIndex::new();
    index.index_document(&uri, source, Some(&program));

    let definition = index.find_definition_at(
        &uri,
        Position {
            line: 0,
            character: 4,
        },
    );

    assert!(definition.is_some());
    let definition = definition.unwrap();
    assert_eq!(definition.name, "add");
    assert_eq!(definition.kind, atlas_lsp::index::SymbolKind::Function);
    assert_eq!(definition.location.uri, uri);
}

#[tokio::test]
async fn test_symbol_index_import_export_indexing() {
    let (service, _socket) = LspService::new(AtlasLspServer::new);
    let _server = service.inner();

    let export_uri = Url::parse("file:///export.atl").unwrap();
    let export_source = r#"export fn foo() -> number {
    return 1;
}
"#;

    let mut export_lexer = atlas_runtime::Lexer::new(export_source);
    let (export_tokens, _) = export_lexer.tokenize();
    let mut export_parser = atlas_runtime::Parser::new(export_tokens);
    let (export_program, _export_diagnostics) = export_parser.parse();

    let import_uri = Url::parse("file:///import.atl").unwrap();
    let import_source = r#"import { foo } from "./export";
let mut value: number = foo();
"#;

    let mut import_lexer = atlas_runtime::Lexer::new(import_source);
    let (import_tokens, _) = import_lexer.tokenize();
    let mut import_parser = atlas_runtime::Parser::new(import_tokens);
    let (import_program, _import_diagnostics) = import_parser.parse();

    let mut index = atlas_lsp::index::SymbolIndex::new();
    index.index_document(&export_uri, export_source, Some(&export_program));
    index.index_document(&import_uri, import_source, Some(&import_program));

    let definitions = index.find_definitions("foo");
    let mut uris: Vec<Url> = definitions
        .iter()
        .map(|def| def.location.uri.clone())
        .collect();
    uris.sort();
    uris.dedup();

    assert!(uris.contains(&export_uri));
    assert!(uris.contains(&import_uri));
}
