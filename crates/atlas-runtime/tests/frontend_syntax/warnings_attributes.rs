//! @allow(unused) attribute tests — H-121

use super::*;

/// Collect warning diagnostic codes from source.
fn collect_warnings(source: &str) -> Vec<String> {
    use atlas_runtime::binder::Binder;
    use atlas_runtime::typechecker::TypeChecker;
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut table, _) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let diags = checker.check(&program);
    diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .map(|d| d.code.clone())
        .collect()
}

fn has_at2001(source: &str) -> bool {
    collect_warnings(source).iter().any(|c| c == "AT2001")
}

// --- Baseline: unused warnings fire without @allow ---

#[test]
fn test_unused_var_emits_at2001_baseline() {
    assert!(has_at2001(
        r#"
        fn foo(): number {
            let unused_x = 42;
            0;
        }
        foo();
        "#
    ));
}

// --- @allow(unused) suppresses AT2001 on variables ---

#[test]
fn test_allow_unused_on_fn_suppresses_param_warning() {
    // @allow(unused) on a function suppresses unused-param warnings
    assert!(!has_at2001(
        r#"
        @allow(unused)
        fn process(borrow x: number, borrow y: number): number {
            x;
        }
        process(1, 2);
        "#
    ));
}

#[test]
fn test_allow_unused_parses_without_error() {
    let mut lexer = Lexer::new(
        r#"
        @allow(unused)
        fn scaffold(borrow x: number): number {
            42;
        }
    "#,
    );
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
}

#[test]
fn test_allow_unused_on_struct_parses() {
    let mut lexer = Lexer::new(
        r#"
        @allow(unused)
        struct Scaffold {
            x: number,
        }
    "#,
    );
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (_, diags) = parser.parse();
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
}

#[test]
fn test_allow_unknown_lint_is_warning() {
    // @allow(unknown_lint) should warn that the lint is unrecognized
    let warnings = collect_warnings(
        r#"
        @allow(bogus_lint)
        fn foo(): number { 42; }
        foo();
    "#,
    );
    assert!(
        warnings.iter().any(|w| w == "AT2009"),
        "Expected AT2009 (unknown lint) warning, got: {:?}",
        warnings
    );
}
