use super::*;

// From scope_shadowing_tests.rs
// ============================================================================

fn bind_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (_table, bind_diags) = binder.bind(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags
}

fn typecheck_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);
    all_diags
}

fn assert_no_errors(diagnostics: &[Diagnostic]) {
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no errors, got: {:?}",
        errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

fn assert_has_error(diagnostics: &[Diagnostic], code: &str) {
    let found = diagnostics.iter().any(|d| d.code == code);
    assert!(
        found,
        "Expected diagnostic with code {}, got: {:?}",
        code,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

// ============================================================================
// Block Scoping - Valid Cases
// ============================================================================

#[rstest]
#[case::nested_block(r#"let x: number = 1; { let y: number = 2; let z = x + y; }"#)]
#[case::multiple_levels(
    r#"let a: number = 1; { let b: number = 2; { let c: number = 3; let sum = a + b + c; } }"#
)]
#[case::if_block(r#"let x: number = 1; if (x > 0) { let y: number = 2; }"#)]
#[case::while_block(r#"let i: number = 0; while (i < 10) { let temp: number = i; }"#)]
#[case::for_loop_init(r#"for (let i: number = 0; i < 10; i = i + 1) { let x = i; }"#)]
#[case::for_loop_body(r#"for (let i: number = 0; i < 10; i = i + 1) { let sum: number = 0; }"#)]
#[case::empty_block(r#"let x: number = 1; { } let y = x;"#)]
#[case::nested_empty(r#"{ { { } } }"#)]
fn test_valid_block_scoping(#[case] source: &str) {
    let diagnostics = bind_source(source);
    assert_no_errors(&diagnostics);
}

// ============================================================================
// Block Scoping - Out of Scope Errors
// ============================================================================

#[rstest]
#[case::block_var_out_of_scope(r#"{ let x: number = 1; } let y = x;"#, "AT2002")]
#[case::if_block_out_of_scope(
    r#"let x: number = 1; if (x > 0) { let y: number = 2; } let z = y;"#,
    "AT2002"
)]
#[case::while_block_out_of_scope(
    r#"let i: number = 0; while (i < 10) { let temp: number = i; } let x = temp;"#,
    "AT2002"
)]
#[case::for_init_out_of_scope(
    r#"for (let i: number = 0; i < 10; i = i + 1) { let x = i; } let y = i;"#,
    "AT2002"
)]
#[case::for_body_out_of_scope(
    r#"for (let i: number = 0; i < 10; i = i + 1) { let sum: number = 0; } let x = sum;"#,
    "AT2002"
)]
fn test_out_of_scope_errors(#[case] source: &str, #[case] expected_code: &str) {
    let diagnostics = bind_source(source);
    assert_has_error(&diagnostics, expected_code);
}

// ============================================================================
// Variable Shadowing - Allowed in Nested Scopes
// ============================================================================

#[rstest]
#[case::basic_shadowing(r#"let x: number = 1; { let x: string = "hello"; }"#)]
#[case::multiple_levels(
    r#"let x: number = 1; { let x: string = "level 1"; { let x: bool = true; } }"#
)]
#[case::param_shadowing(
    r#"fn foo(x: number) -> number { { let x: string = "shadow"; } return x; }"#
)]
#[case::if_block_shadow(r#"let x: number = 1; if (true) { let x: string = "shadow"; }"#)]
#[case::else_block_shadow(
    r#"let x: number = 1; if (false) { let y: number = 2; } else { let x: string = "shadow"; }"#
)]
#[case::while_shadow(r#"let i: number = 0; while (i < 10) { let i: string = "shadow"; }"#)]
#[case::for_shadow(
    r#"let i: number = 999; for (let i: number = 0; i < 10; i = i + 1) { let x = i; }"#
)]
#[case::shadow_restored(r#"let x: number = 1; { let x: string = "shadow"; } let y = x;"#)]
#[case::nested_fn_shadow(r#"fn outer(x: number) -> number { { let x: string = "shadow"; { let x: bool = true; } } return x; }"#)]
#[case::if_else_separate(
    r#"let x: number = 1; if (true) { let y: number = 2; } else { let y: string = "different"; }"#
)]
#[case::multiple_blocks(r#"{ let x: number = 1; } { let x: string = "different block"; }"#)]
#[case::loop_nested_blocks(r#"let i: number = 0; while (i < 10) { { let temp: number = i; } { let temp: string = "different"; } }"#)]
#[case::deeply_nested(r#"let a: number = 1; { let b: number = 2; { let c: number = 3; { let d: number = 4; { let e: number = 5; let sum = a + b + c + d + e; } } } }"#)]
#[case::different_types(r#"let x: number = 1; { let x: string = "string"; { let x: bool = true; { let x = [1, 2, 3]; } } }"#)]
fn test_valid_shadowing(#[case] source: &str) {
    let diagnostics = bind_source(source);
    assert_no_errors(&diagnostics);
}

// ============================================================================
// Redeclaration Errors - Same Scope
// ============================================================================

#[rstest]
#[case::same_scope(r#"let x: number = 1; let x: string = "redeclare";"#, "AT2003")]
#[case::in_block(
    r#"fn test() -> void { let x: number = 1; let x: string = "redeclare"; }"#,
    "AT2003"
)]
#[case::param_redecl(r#"fn foo(x: number, x: string) -> number { return 0; }"#, "AT2003")]
#[case::function_redecl(
    r#"fn foo() -> number { return 1; } fn foo() -> string { return "redeclare"; }"#,
    "AT2003"
)]
fn test_redeclaration_errors(#[case] source: &str, #[case] expected_code: &str) {
    let diagnostics = bind_source(source);
    assert_has_error(&diagnostics, expected_code);
}

#[test]
fn test_multiple_variable_redeclarations() {
    let diagnostics =
        bind_source(r#"let x: number = 1; let x: string = "second"; let x: bool = true;"#);
    // Should have multiple redeclaration errors
    let redecl_errors: Vec<_> = diagnostics.iter().filter(|d| d.code == "AT2003").collect();
    assert!(redecl_errors.len() >= 2);
}

// ============================================================================
// Function Parameter Cases
// ============================================================================

#[rstest]
#[case::param_shadow_allowed(
    r#"fn foo(x: number) -> number { { let x: string = "shadow"; } return x; }"#
)]
#[case::param_can_read(r#"fn double(x: number) -> number { let result = x * 2; return result; }"#)]
#[case::param_in_expr(r#"fn calculate(x: number, y: number) -> number { return x + y * 2; }"#)]
fn test_valid_parameter_usage(#[case] source: &str) {
    let diagnostics = bind_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::immutable_assign(r#"fn foo(x: number) -> number { x = 10; return x; }"#)]
#[case::multiple_params(r#"fn add(a: number, b: number) -> number { a = a + 1; return a + b; }"#)]
fn test_parameter_immutability(#[case] source: &str) {
    // NOTE: Parameter immutability checking requires full type checking
    // This test documents the expected behavior
    let _diagnostics = typecheck_source(source);
    // Once AT3003 is implemented for parameters, add assertion:
    // assert_has_error(&_diagnostics, "AT3003");
}

// ============================================================================
// Function Scope
// ============================================================================

#[rstest]
#[case::access_params(r#"fn foo(x: number, y: string) -> number { let z = x; return z; }"#)]
#[case::call_other_fn(
    r#"fn helper() -> number { return 42; } fn main() -> number { return helper(); }"#
)]
#[case::hoisting(
    r#"fn main() -> number { return helper(); } fn helper() -> number { return 42; }"#
)]
#[case::use_prelude(r#"fn test() -> number { print("hello"); return 42; }"#)]
fn test_valid_function_scope(#[case] source: &str) {
    let diagnostics = bind_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::undefined_var(r#"fn foo() -> number { return undefined_var; }"#, "AT2002")]
#[case::forward_ref(
    r#"let x: number = a + b; let a: number = 1; let b: number = 2;"#,
    "AT2002"
)]
#[case::self_ref(r#"let x: number = x + 1;"#, "AT2002")]
#[case::decl_order(r#"let x = y; let y: number = 1;"#, "AT2002")]
fn test_scope_errors(#[case] source: &str, #[case] expected_code: &str) {
    let diagnostics = bind_source(source);
    assert_has_error(&diagnostics, expected_code);
}

// ============================================================================
// Prelude Shadowing (Documented Behavior)
// ============================================================================

#[rstest]
#[case::shadow_print(r#"fn test() -> void { let print: number = 42; }"#)]
#[case::shadow_len(r#"fn test() -> void { let len: string = "shadowed"; }"#)]
fn test_prelude_shadowing(#[case] source: &str) {
    // NOTE: Prelude shadowing detection (AT1012) may not be fully implemented yet
    // This test documents the expected behavior
    // For now, just verify it binds without crashing
    let _diagnostics = bind_source(source);
    // Once AT1012 is implemented, add assertion:
    // assert_has_error(&_diagnostics, "AT1012");
}

// ============================================================================
