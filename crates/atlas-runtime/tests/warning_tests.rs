use atlas_runtime::{Binder, DiagnosticLevel, Lexer, Parser, TypeChecker};

/// Helper to get all diagnostics from source code
fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&table);
    let type_diags = checker.check(&program);

    // Combine all diagnostics
    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);

    all_diags
}

#[test]
fn test_unused_variable_warning() {
    let source = r#"fn main() -> number {
    let x: number = 42;
    return 5;
}"#;

    let diags = get_all_diagnostics(source);

    // Should have one warning for unused variable
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2001 warning, got {}: {:?}", warnings.len(), warnings.iter().map(|d| &d.message).collect::<Vec<_>>());
    assert!(warnings[0].message.contains("Unused variable 'x'"));
}

#[test]
fn test_used_variable_no_warning() {
    let source = r#"fn main() -> number {
    let x: number = 42;
    return x;
}"#;

    let diags = get_all_diagnostics(source);

    // Should have no unused variable warnings
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 0, "Expected no AT2001 warnings, got {}: {:?}", warnings.len(), warnings.iter().map(|d| &d.message).collect::<Vec<_>>());
}

#[test]
fn test_underscore_prefix_suppresses_warning() {
    let source = r#"fn main() -> number {
    let _unused: number = 42;
    return 5;
}"#;

    let diags = get_all_diagnostics(source);

    // Should have no warnings - underscore prefix suppresses
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 0);
}

#[test]
fn test_unused_parameter_warning() {
    let source = r#"fn add(a: number, b: number) -> number {
    return a;
}"#;

    let diags = get_all_diagnostics(source);

    // Should warn about unused parameter 'b'
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2001 warning for unused param, got {}: {:?}", warnings.len(), warnings.iter().map(|d| &d.message).collect::<Vec<_>>());
    assert!(warnings[0].message.contains("Unused parameter 'b'"));
}

#[test]
fn test_unreachable_code_after_return() {
    let source = r#"fn main() -> number {
    return 42;
    let x: number = 10;
}"#;

    let diags = get_all_diagnostics(source);

    // Should warn about unreachable code
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2002").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2002 warning, got {}: {:?}", warnings.len(), warnings.iter().map(|d| &d.message).collect::<Vec<_>>());
    assert!(warnings[0].message.contains("Unreachable code"));
}

#[test]
fn test_no_unreachable_warning_without_return() {
    let source = r#"fn main() -> number {
    let x: number = 42;
    let y: number = 10;
    return x;
}"#;

    let diags = get_all_diagnostics(source);

    // Should not have unreachable code warning
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2002").collect();
    assert_eq!(warnings.len(), 0);
}

#[test]
fn test_warnings_with_errors() {
    let source = r#"fn main() -> number {
    let x: number = "bad";
    return 5;
}"#;

    let diags = get_all_diagnostics(source);

    // Should have both error (type mismatch) and warning (unused variable)
    let errors: Vec<_> = diags.iter().filter(|d| d.level == DiagnosticLevel::Error).collect();
    let warnings: Vec<_> = diags.iter().filter(|d| d.level == DiagnosticLevel::Warning).collect();

    assert!(errors.len() > 0, "Expected type error"); // Type error
    assert!(warnings.len() > 0, "Expected unused warning"); // Unused variable warning
}

#[test]
fn test_multiple_unused_variables() {
    let source = r#"fn main() -> number {
    let x: number = 1;
    let y: number = 2;
    let z: number = 3;
    return 0;
}"#;

    let diags = get_all_diagnostics(source);

    // Should warn about all three unused variables
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 3, "Expected 3 AT2001 warnings, got {}: {:?}", warnings.len(), warnings.iter().map(|d| &d.message).collect::<Vec<_>>());
}
