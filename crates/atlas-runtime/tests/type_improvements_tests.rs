//! Tests for improved type error messages and suggestions (Phase typing-01)
//!
//! Validates that type error messages show clear expected vs actual comparisons
//! and provide actionable fix suggestions.

mod common;

use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};

/// Helper: typecheck source and return diagnostics (binder + typechecker)
fn typecheck(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return lex_diags;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return parse_diags;
    }

    let mut binder = Binder::new();
    let (mut table, mut bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let mut type_diags = checker.check(&program);

    bind_diags.append(&mut type_diags);
    bind_diags
}

/// Helper: get only error-level diagnostics
fn errors(source: &str) -> Vec<Diagnostic> {
    typecheck(source)
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect()
}

/// Helper: get only warning-level diagnostics
fn warnings(source: &str) -> Vec<Diagnostic> {
    typecheck(source)
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect()
}

// ============================================================================
// 1. Type mismatch: clear expected vs actual
// ============================================================================

#[test]
fn test_var_type_mismatch_number_string() {
    let diags = errors(r#"let x: number = "hello";"#);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("expected"));
    assert!(diags[0].message.contains("found"));
    assert_eq!(diags[0].code, "AT3001");
}

#[test]
fn test_var_type_mismatch_string_number() {
    let diags = errors("let x: string = 42;");
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("expected string"));
    assert!(diags[0].message.contains("found number"));
}

#[test]
fn test_var_type_mismatch_bool_string() {
    let diags = errors(r#"let x: bool = "true";"#);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("expected bool"));
}

#[test]
fn test_assignment_type_mismatch() {
    let diags = errors(
        r#"
        var x: number = 1;
        x = "hello";
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
    assert!(diags[0].message.contains("found string"));
}

// ============================================================================
// 2. Suggestions for number-string mismatch
// ============================================================================

#[test]
fn test_suggest_num_conversion() {
    let diags = errors(r#"let x: number = "42";"#);
    assert!(!diags.is_empty());
    // Should suggest num() conversion
    assert!(
        diags[0].help.as_ref().map_or(false, |h| h.contains("num(")),
        "Expected num() suggestion, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_suggest_str_conversion() {
    let diags = errors("let x: string = 42;");
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().map_or(false, |h| h.contains("str(")),
        "Expected str() suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 3. Suggestions for missing return / return mismatch
// ============================================================================

#[test]
fn test_return_type_mismatch_suggests_fix() {
    let diags = errors(
        r#"
        fn foo() -> number {
            return "hello";
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
    assert!(diags[0].message.contains("found string"));
}

#[test]
fn test_missing_return_suggests_adding_one() {
    let diags = errors(
        r#"
        fn foo() -> number {
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0]
        .message
        .contains("Not all code paths return a value"));
}

#[test]
fn test_return_void_from_number_function() {
    let diags = errors(
        r#"
        fn foo() -> number {
            return;
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("expected number"));
    assert!(
        diags[0]
            .help
            .as_ref()
            .map_or(false, |h| h.contains("missing return")),
        "Expected missing return suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 4. Suggestions for wrong operator
// ============================================================================

#[test]
fn test_add_string_number_suggests_str() {
    let diags = errors(r#"let _x = "hello" + 42;"#);
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3002");
    assert!(
        diags[0].help.as_ref().map_or(false, |h| h.contains("str(")),
        "Expected str() suggestion for string + number, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_add_number_string_suggests_str() {
    let diags = errors(r#"let _x = 42 + "hello";"#);
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().map_or(false, |h| h.contains("str(")),
        "Expected str() suggestion for number + string, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_subtract_strings_error() {
    let diags = errors(r#"let _x = "a" - "b";"#);
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3002");
}

// ============================================================================
// 5. Suggestions for undefined variables (unknown symbol)
// ============================================================================

#[test]
fn test_undefined_variable_error() {
    let diags = errors("let _x = foo;");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT2002");
    assert!(diags[0].message.contains("Unknown symbol 'foo'"));
}

// ============================================================================
// 6. Complex type display - function signatures
// ============================================================================

#[test]
fn test_function_type_display_in_error() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x: string = add;
    "#,
    );
    assert!(!diags.is_empty());
    // Function should display as "(number, number) -> number" not just "function"
    assert!(
        diags[0].message.contains("(number, number) -> number"),
        "Expected function signature in error, got: {}",
        diags[0].message
    );
}

#[test]
fn test_function_type_display_void_return() {
    let diags = errors(
        r#"
        fn greet(_name: string) -> void { }
        let _x: number = greet;
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("(string) -> void"),
        "Expected function signature, got: {}",
        diags[0].message
    );
}

#[test]
fn test_function_type_display_no_params() {
    let diags = errors(
        r#"
        fn foo() -> number { return 1; }
        let _x: string = foo;
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("() -> number"),
        "Expected () -> number in error, got: {}",
        diags[0].message
    );
}

// ============================================================================
// 7. Array and generic type display
// ============================================================================

#[test]
fn test_array_type_display_in_error() {
    let diags = errors(
        r#"
        let arr = [1, 2, 3];
        let _x: string = arr;
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("number[]"),
        "Expected number[] in error, got: {}",
        diags[0].message
    );
}

// ============================================================================
// 8. Error location accuracy (span info)
// ============================================================================

#[test]
fn test_error_has_span_info() {
    let diags = errors("let x: number = true;");
    assert!(!diags.is_empty());
    // Should have valid location info (length > 0)
    assert!(diags[0].length > 0, "Error should have valid span info");
}

// ============================================================================
// 9. Condition type errors with suggestions
// ============================================================================

#[test]
fn test_if_condition_number_suggests_comparison() {
    let diags = errors("if (42) { }");
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().map_or(false, |h| h.contains("!=")),
        "Expected comparison suggestion, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_while_condition_string_suggests_comparison() {
    let diags = errors(r#"while ("hello") { }"#);
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().map_or(false, |h| h.contains("len")),
        "Expected len suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 10. Immutable variable suggestions
// ============================================================================

#[test]
fn test_immutable_variable_suggests_var() {
    let diags = errors(
        r#"
        let x = 5;
        x = 10;
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3003");
    assert!(
        diags[0].help.as_ref().map_or(false, |h| h.contains("var")),
        "Expected var suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 11. Function call errors
// ============================================================================

#[test]
fn test_wrong_arity_shows_signature() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x = add(1);
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3005");
    // Help should include the function signature
    assert!(
        diags[0]
            .help
            .as_ref()
            .map_or(false, |h| h.contains("(number, number) -> number")),
        "Expected function signature in help, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_too_many_args_says_remove() {
    let diags = errors(
        r#"
        fn single(a: number) -> number { return a; }
        let _x = single(1, 2, 3);
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0]
            .help
            .as_ref()
            .map_or(false, |h| h.contains("remove")),
        "Expected 'remove' suggestion, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_wrong_arg_type_suggests_conversion() {
    let diags = errors(
        r#"
        fn double(x: number) -> number { return x * 2; }
        let _x = double("hello");
    "#,
    );
    assert!(!diags.is_empty());
    // Should suggest num() conversion
    assert!(
        diags[0].help.as_ref().map_or(false, |h| h.contains("num(")),
        "Expected num() suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 12. Not callable errors
// ============================================================================

#[test]
fn test_call_string_not_callable() {
    let diags = errors(
        r#"
        let _x = "hello"(42);
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3006");
}

#[test]
fn test_call_number_not_callable() {
    let diags = errors("let _x = 42(1);");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3006");
}

// ============================================================================
// 13. For-in errors with suggestions
// ============================================================================

#[test]
fn test_for_in_number_suggests_range() {
    let diags = errors(
        r#"
        fn test() -> void {
            for x in 42 {
                print(x);
            }
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(
        diags[0]
            .help
            .as_ref()
            .map_or(false, |h| h.contains("range")),
        "Expected range suggestion, got: {:?}",
        diags[0].help
    );
}

// ============================================================================
// 14. Compound assignment errors
// ============================================================================

#[test]
fn test_compound_assign_wrong_type() {
    let diags = errors(
        r#"
        var x: string = "hello";
        x += 1;
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3001");
}

// ============================================================================
// 15. Unreachable code warnings
// ============================================================================

#[test]
fn test_unreachable_code_warning() {
    let diags = warnings(
        r#"
        fn foo() -> number {
            return 42;
            let _x = 1;
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT2002");
    assert!(diags[0].message.contains("Unreachable"));
}

// ============================================================================
// 16. Unused variable warnings
// ============================================================================

#[test]
fn test_unused_variable_warning() {
    let diags = warnings(
        r#"
        fn foo() -> void {
            let x = 42;
        }
    "#,
    );
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("Unused variable 'x'"));
}

#[test]
fn test_underscore_prefix_suppresses_unused() {
    let diags = warnings(
        r#"
        fn foo() -> void {
            let _x = 42;
        }
    "#,
    );
    assert!(
        diags.is_empty(),
        "Underscore-prefixed should not warn: {:?}",
        diags
    );
}

// ============================================================================
// 17. Break/continue outside loop
// ============================================================================

#[test]
fn test_break_outside_loop_error() {
    let diags = errors("break;");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3010");
}

#[test]
fn test_continue_outside_loop_error() {
    let diags = errors("continue;");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3010");
}

// ============================================================================
// 18. Return outside function
// ============================================================================

#[test]
fn test_return_outside_function_error() {
    let diags = errors("return 5;");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AT3011");
}

// ============================================================================
// 19. Valid code still passes
// ============================================================================

#[test]
fn test_valid_arithmetic() {
    let diags = errors("let _x = 1 + 2;");
    assert!(diags.is_empty());
}

#[test]
fn test_valid_string_concat() {
    let diags = errors(r#"let _x = "hello" + " world";"#);
    assert!(diags.is_empty());
}

#[test]
fn test_valid_function_call() {
    let diags = errors(
        r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let _x = add(1, 2);
    "#,
    );
    assert!(
        diags.is_empty(),
        "Valid code should have no errors: {:?}",
        diags
    );
}

#[test]
fn test_valid_if_bool() {
    let diags = errors("if (true) { }");
    assert!(diags.is_empty());
}

#[test]
fn test_valid_var_mutation() {
    let diags = errors(
        r#"
        var x = 5;
        x = 10;
    "#,
    );
    assert!(
        diags.is_empty(),
        "Mutable assignment should work: {:?}",
        diags
    );
}

#[test]
fn test_valid_for_in_array() {
    let diags = errors(
        r#"
        fn test() -> void {
            let arr = [1, 2, 3];
            for x in arr {
                print(x);
            }
        }
    "#,
    );
    assert!(
        diags.is_empty(),
        "For-in over array should work: {:?}",
        diags
    );
}
