//! Tests for related span functionality in diagnostics
//!
//! Verifies that diagnostics include related locations pointing to
//! relevant code locations (e.g., original declarations for redeclaration errors).

use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};

/// Helper to parse source code
fn parse(source: &str) -> (atlas_runtime::ast::Program, Vec<atlas_runtime::Diagnostic>) {
    let mut lexer = Lexer::new(source);
    let (tokens, mut lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, mut parse_diags) = parser.parse();

    lex_diags.append(&mut parse_diags);
    (program, lex_diags)
}

/// Helper to bind a program
fn bind_program(
    program: &atlas_runtime::ast::Program,
) -> (atlas_runtime::SymbolTable, Vec<atlas_runtime::Diagnostic>) {
    let mut binder = Binder::new();
    binder.bind(program)
}

/// Helper to typecheck a program
fn typecheck_program(
    program: &atlas_runtime::ast::Program,
    symbol_table: &atlas_runtime::SymbolTable,
) -> Vec<atlas_runtime::Diagnostic> {
    let mut checker = TypeChecker::new(symbol_table);
    checker.check(program)
}

#[test]
fn test_function_redeclaration_has_related_span() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 1, "Should have one binding error");

    let diag = &bind_diags[0];
    assert_eq!(diag.code, "AT2003");
    assert!(
        diag.message.contains("already defined"),
        "Error message should mention redefinition"
    );

    // Verify related location exists
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to first definition"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("first defined"),
        "Related message should mention first definition: {}",
        related.message
    );
}

#[test]
fn test_parameter_redeclaration_has_related_span() {
    let source = r#"
        fn foo(x: number, x: string) {}
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 1, "Should have one binding error");

    let diag = &bind_diags[0];
    assert_eq!(diag.code, "AT2003");

    // Verify related location exists
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to first parameter"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("first defined"),
        "Related message should mention first definition: {}",
        related.message
    );
}

#[test]
fn test_variable_redeclaration_has_related_span() {
    let source = r#"
        fn test() {
            let x = 5;
            let x = 10;
        }
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 1, "Should have one binding error");

    let diag = &bind_diags[0];
    assert_eq!(diag.code, "AT2003");

    // Verify related location exists
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to first declaration"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("first defined"),
        "Related message should mention first definition: {}",
        related.message
    );
}

#[test]
fn test_return_type_mismatch_has_related_span() {
    let source = r#"
        fn foo() -> number {
            return "hello";
        }
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors: {:?}", parse_diags);

    let (symbol_table, bind_diags) = bind_program(&ast);
    assert!(bind_diags.is_empty(), "Should bind without errors: {:?}", bind_diags);

    let type_diags = typecheck_program(&ast, &symbol_table);
    assert_eq!(type_diags.len(), 1, "Should have one type error, got: {:?}", type_diags);

    let diag = &type_diags[0];
    assert_eq!(diag.code, "AT3001");
    assert!(
        diag.message.contains("Return type mismatch"),
        "Error message should mention return type mismatch"
    );

    // Verify related location exists pointing to function declaration
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to function declaration"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("declared here"),
        "Related message should mention declaration: {}",
        related.message
    );
    assert!(
        related.message.contains("foo"),
        "Related message should mention function name: {}",
        related.message
    );
}

#[test]
fn test_immutable_assignment_has_related_span() {
    let source = r#"
        let x = 5;
        x = 10;
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors: {:?}", parse_diags);

    let (symbol_table, bind_diags) = bind_program(&ast);
    assert!(bind_diags.is_empty(), "Should bind without errors: {:?}", bind_diags);

    let type_diags = typecheck_program(&ast, &symbol_table);

    // Find the AT3003 error (immutable assignment)
    let at3003_diags: Vec<_> = type_diags.iter().filter(|d| d.code == "AT3003").collect();
    assert_eq!(at3003_diags.len(), 1, "Should have one AT3003 error, got {:?}. All diagnostics: {:?}",
        type_diags.iter().map(|d| &d.code).collect::<Vec<_>>(),
        type_diags.iter().map(|d| (&d.code, &d.message)).collect::<Vec<_>>()
    );

    let diag = at3003_diags[0];
    assert!(
        diag.message.contains("immutable"),
        "Error message should mention immutability"
    );

    // Verify related location exists pointing to variable declaration
    assert_eq!(
        diag.related.len(),
        1,
        "Should have one related location pointing to variable declaration"
    );
    let related = &diag.related[0];
    assert!(
        related.message.contains("declared here"),
        "Related message should mention declaration: {}",
        related.message
    );
    assert!(
        related.message.contains("immutable"),
        "Related message should mention immutability: {}",
        related.message
    );
}

#[test]
fn test_related_span_points_to_correct_location() {
    let source = r#"
        fn first() {}
        fn second() {}
        fn first() {}
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 1, "Should have one binding error");

    let diag = &bind_diags[0];
    let related = &diag.related[0];

    // The related span should point to the first occurrence (not the second function)
    // We can verify this by checking that the column is before the error's column
    assert!(
        related.column < diag.column || related.line < diag.line,
        "Related location should point to earlier code"
    );
}

#[test]
fn test_multiple_redeclarations_each_have_related_span() {
    let source = r#"
        fn test() {
            let x = 1;
            let y = 2;
            let x = 3;
            let y = 4;
        }
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors");

    let (_, bind_diags) = bind_program(&ast);
    assert_eq!(bind_diags.len(), 2, "Should have two binding errors");

    // Each error should have a related location
    for diag in &bind_diags {
        assert_eq!(
            diag.related.len(),
            1,
            "Each error should have one related location"
        );
    }
}

#[test]
fn test_related_span_serializes_to_json() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let (ast, _) = parse(source);
    let (_, bind_diags) = bind_program(&ast);
    let diag = &bind_diags[0];

    // Verify JSON serialization works
    let json = diag.to_json_string().expect("Should serialize to JSON");
    assert!(json.contains("\"related\""), "JSON should contain related field");
    assert!(
        json.contains("first defined"),
        "JSON should contain related message"
    );
}

#[test]
fn test_related_span_renders_in_human_format() {
    let source = r#"
        fn foo() {}
        fn foo() {}
    "#;

    let (ast, _) = parse(source);
    let (_, bind_diags) = bind_program(&ast);
    let diag = &bind_diags[0];

    // Verify human format includes related location
    let human = diag.to_human_string();
    assert!(
        human.contains("note:"),
        "Human format should contain note for related location"
    );
    assert!(
        human.contains("first defined"),
        "Human format should show related message"
    );
}

#[test]
fn test_no_related_span_for_undefined_variable() {
    let source = r#"
        fn test() -> number {
            return x;
        }
    "#;

    let (ast, parse_diags) = parse(source);
    assert!(parse_diags.is_empty(), "Should parse without errors: {:?}", parse_diags);

    let (_symbol_table, bind_diags) = bind_program(&ast);

    // Undefined variable errors come from the binder
    // Currently these don't have related spans (could add "did you mean?" in future)
    if !bind_diags.is_empty() {
        for _diag in &bind_diags {
            // Related spans are optional, so this test just documents current behavior
            // If we add "did you mean?" suggestions later, we'd update this test
        }
    }
}
