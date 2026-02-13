//! Code completion tests

use atlas_lsp::server::AtlasLspServer;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

#[tokio::test]
async fn test_keyword_completions() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    // Open empty document
    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

    // Request completions
    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have keywords
        assert!(items.iter().any(|item| item.label == "let"));
        assert!(items.iter().any(|item| item.label == "fn"));
        assert!(items.iter().any(|item| item.label == "if"));
        assert!(items.iter().any(|item| item.label == "while"));
        assert!(items.iter().any(|item| item.label == "return"));
    }
}

#[tokio::test]
async fn test_builtin_function_completions() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have builtin functions
        assert!(items.iter().any(|item| item.label == "print"));
        assert!(items.iter().any(|item| item.label == "len"));
        assert!(items.iter().any(|item| item.label == "push"));
        assert!(items.iter().any(|item| item.label == "pop"));
    }
}

#[tokio::test]
async fn test_type_completions() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have type keywords
        assert!(items.iter().any(|item| item.label == "number"));
        assert!(items.iter().any(|item| item.label == "string"));
        assert!(items.iter().any(|item| item.label == "bool"));
    }
}

#[tokio::test]
async fn test_function_completions_from_document() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}

fn greet(name: string) -> string {
    return name;
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

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 8,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have user-defined functions
        assert!(items.iter().any(|item| item.label == "add"));
        assert!(items.iter().any(|item| item.label == "greet"));

        // Check function details
        let add_item = items.iter().find(|item| item.label == "add");
        assert!(add_item.is_some());
        assert_eq!(add_item.unwrap().kind, Some(CompletionItemKind::FUNCTION));
    }
}

#[tokio::test]
async fn test_variable_completions_from_document() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let source = r#"
var counter: number = 0;
var name: string = "test";
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

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 3,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Should have variables
        assert!(items.iter().any(|item| item.label == "counter"));
        assert!(items.iter().any(|item| item.label == "name"));
    }
}

#[tokio::test]
async fn test_snippet_completions() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: "".to_string(),
        },
    };
    server.did_open(open_params).await;

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    assert!(result.is_some());

    if let Some(CompletionResponse::Array(items)) = result {
        // Check that snippets have insert text
        let if_item = items.iter().find(|item| item.label == "if");
        assert!(if_item.is_some());
        assert!(if_item.unwrap().insert_text.is_some());
        assert_eq!(
            if_item.unwrap().insert_text_format,
            Some(InsertTextFormat::SNIPPET)
        );

        let fn_item = items.iter().find(|item| item.label == "fn");
        assert!(fn_item.is_some());
        assert!(fn_item.unwrap().insert_text.is_some());
    }
}

#[tokio::test]
async fn test_completions_with_errors() {
    let (service, _socket) = LspService::new(|client| AtlasLspServer::new(client));
    let server = service.inner();

    let uri = Url::parse("file:///test.atl").unwrap();

    // Document with syntax errors
    let source = "let x =";

    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "atlas".to_string(),
            version: 1,
            text: source.to_string(),
        },
    };
    server.did_open(open_params).await;

    let completion_params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 7,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let result = server.completion(completion_params).await.unwrap();
    // Should still provide completions even with errors
    assert!(result.is_some());
}
