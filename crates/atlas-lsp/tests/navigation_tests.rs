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

var counter: number = 0;

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
var result: number = add(1, 2);
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
var x: number = add(1, 2);
var y: number = add(3, 4);
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
    // Currently returns None (TODO: implement)
    assert!(result.is_none());
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
