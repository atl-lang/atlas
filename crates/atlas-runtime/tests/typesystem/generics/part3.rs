use super::super::*;

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
        diags[0].help.as_ref().is_some_and(|h| h.contains("!=")),
        "Expected comparison suggestion, got: {:?}",
        diags[0].help
    );
}

#[test]
fn test_while_condition_string_suggests_comparison() {
    let diags = errors(r#"while ("hello") { }"#);
    assert!(!diags.is_empty());
    assert!(
        diags[0].help.as_ref().is_some_and(|h| h.contains("len")),
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
        diags[0].help.as_ref().is_some_and(|h| h.contains("var")),
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
            .is_some_and(|h| h.contains("(number, number) -> number")),
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
        diags[0].help.as_ref().is_some_and(|h| h.contains("remove")),
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
        diags[0].help.as_ref().is_some_and(|h| h.contains("num(")),
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
        diags[0].help.as_ref().is_some_and(|h| h.contains("range")),
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

// ============================================================================

// NOTE: test block removed — required access to private function `is_ok`

// ============================================================================
// Generic call-site type argument inference (Block 5 Phase 5)
// ============================================================================

#[test]
fn test_generic_identity_infers_number() {
    // identity(42) infers T=number without explicit type arg
    let diags = errors(
        r#"
fn identity<T>(x: T) -> T { return x; }
let _n: number = identity(42);
"#,
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for identity(42), got: {:?}",
        type_errors
    );
}

#[test]
fn test_generic_identity_infers_string() {
    let diags = errors(
        r#"
fn identity<T>(x: T) -> T { return x; }
let _s: string = identity("hello");
"#,
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for identity(string), got: {:?}",
        type_errors
    );
}

#[test]
fn test_generic_first_infers_element_type() {
    let diags = errors(
        r#"
fn first<T>(arr: T[]) -> T { return arr[0]; }
let _n: number = first([1, 2, 3]);
"#,
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for first([1,2,3]), got: {:?}",
        type_errors
    );
}

#[test]
fn test_generic_explicit_type_arg_still_works() {
    // Explicit identity::<number>(42) must still work
    let diags = errors(
        r#"
fn identity<T>(x: T) -> T { return x; }
let _n: number = identity::<number>(42);
"#,
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for explicit type arg, got: {:?}",
        type_errors
    );
}

#[test]
fn test_generic_multi_param_inference() {
    // fn pair<T, U>(x: T, y: U) -> T — both T and U inferrable from args
    let diags = errors(
        r#"
fn pair<T, U>(x: T, y: U) -> T { return x; }
let _n: number = pair(1, "a");
"#,
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Expected no AT3001 for multi-param inference, got: {:?}",
        type_errors
    );
}

#[test]
fn test_generic_at3051_return_only_type_param() {
    // fn make<T>() -> T cannot infer T from args → AT3051
    let diags = errors(
        r#"
fn make<T>() -> T { return 42; }
make();
"#,
    );
    assert!(
        diags.iter().any(|d| d.code == "AT3051"),
        "Expected AT3051 for uninferrable type param, got: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}
