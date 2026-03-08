use super::super::*;

// D-039: Only `T: Trait` bounds are valid. `extends` keyword is removed.
// Structural bounds (T extends { ... }), union bounds (T extends A | B),
// and primitive type bounds (T extends number) are all removed.
// Use named traits (Iterable, Serializable, Equatable, Comparable, Numeric) instead.
// ============================================================================

fn assert_constraint_no_errors(source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        !has_error(&diagnostics),
        "Expected no errors, got: {:?}",
        diagnostics
    );
}

fn assert_constraint_has_error(source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        has_error(&diagnostics),
        "Expected errors, got: {:?}",
        diagnostics
    );
}

// -----------------------------------------------------------------------------
// Constraint syntax — valid (T: Trait only)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T: Iterable>(x: T) -> number { return 0; }")]
#[case("fn f<T: Serializable>(x: T) -> string { return str(x); }")]
#[case("fn f<T: Equatable>(x: T) -> bool { return true; }")]
#[case("fn f<T: Comparable>(x: T) -> bool { return true; }")]
#[case("fn f<T: Iterable + Serializable>(x: T) -> number { return 0; }")]
fn test_constraint_syntax_valid(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

// -----------------------------------------------------------------------------
// extends keyword is a parse error (D-039)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; }")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; }")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> T { return x; }")]
#[case("type Box<T extends number> = T;")]
fn test_extends_syntax_is_parse_error(#[case] source: &str) {
    assert_constraint_has_error(source);
}

// -----------------------------------------------------------------------------
// Constraint checking — success (nominal traits)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T: Equatable>(x: T) -> bool { return true; } let y = f(false);")]
#[case("fn f<T: Serializable>(x: T) -> string { return str(x); } let y = f(1);")]
#[case("fn f<T: Iterable>(x: T) -> number { return 0; } let y = f([1, 2]);")]
fn test_constraint_checking_success(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

// -----------------------------------------------------------------------------
// Constraint checking — failure (trait not satisfied)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T: Iterable>(x: T) -> number { return 0; } let y = f(1);")]
#[case("fn f<T: Serializable>(x: T) -> string { return str(x); } let y = f([1, 2]);")]
#[case("fn f<T: UnknownTrait>(x: T) -> T { return x; }")]
fn test_constraint_checking_failure(#[case] source: &str) {
    assert_constraint_has_error(source);
}

// -----------------------------------------------------------------------------
// Multiple trait bounds (T: Trait1 + Trait2)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T: Serializable + Equatable>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T: Iterable + Serializable>(x: T) -> number { return 0; } let y = f([1]);")]
fn test_multiple_constraints_success(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

// -----------------------------------------------------------------------------
// Practical constraint patterns
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T: Comparable>(x: T) -> bool { return true; } let y = f(1);")]
#[case("fn f<T: Iterable>(x: T) -> number { return 0; } let y = f([1]);")]
#[case("fn f<T: Equatable>(x: T) -> bool { return true; } let y = f(\"a\");")]
#[case("fn f<T: Serializable>(x: T) -> string { return str(x); } let y = f(true);")]
fn test_practical_constraint_patterns_success(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

#[rstest]
#[case("fn f<T: Iterable>(x: T) -> number { return 0; } let y = f(1);")]
#[case("fn f<T: Serializable>(x: T) -> string { return str(x); } let y = f([1]);")]
fn test_practical_constraint_patterns_failure(#[case] source: &str) {
    assert_constraint_has_error(source);
}

// ============================================================================
