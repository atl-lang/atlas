use super::super::*;

// H-227 / D-039 update: Generic bounds use TypeScript-style `T extends Trait` and `T extends Trait1 & Trait2`.
// Old Rust-style `T: Trait` and `T: Trait1 + Trait2` are no longer valid.
// Structural bounds (T extends { ... }), union bounds, and primitive bounds remain invalid.
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
// Constraint syntax — valid (T extends Trait)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends Iterable>(borrow x: T) -> number { return 0; }")]
#[case("fn f<T extends Serializable>(borrow x: T) -> string { return str(x); }")]
#[case("fn f<T extends Equatable>(borrow x: T) -> bool { return true; }")]
#[case("fn f<T extends Comparable>(borrow x: T) -> bool { return true; }")]
#[case("fn f<T extends Iterable & Serializable>(borrow x: T) -> number { return 0; }")]
fn test_constraint_syntax_valid(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

// -----------------------------------------------------------------------------
// Old Rust-style `: Trait` syntax is now a parse error (H-227)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T: Iterable>(borrow x: T) -> number { return 0; }")]
#[case("fn f<T: Serializable + Equatable>(borrow x: T) -> T { return x; }")]
fn test_old_colon_syntax_is_parse_error(#[case] source: &str) {
    assert_constraint_has_error(source);
}

// -----------------------------------------------------------------------------
// Constraint checking — success (nominal traits)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends Equatable>(borrow x: T) -> bool { return true; } let y = f(false);")]
#[case("fn f<T extends Serializable>(borrow x: T) -> string { return str(x); } let y = f(1);")]
#[case("fn f<T extends Iterable>(borrow x: T) -> number { return 0; } let y = f([1, 2]);")]
fn test_constraint_checking_success(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

// -----------------------------------------------------------------------------
// Constraint checking — failure (trait not satisfied)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends Iterable>(borrow x: T) -> number { return 0; } let y = f(1);")]
#[case("fn f<T extends Serializable>(borrow x: T) -> string { return str(x); } let y = f([1, 2]);")]
#[case("fn f<T extends UnknownTrait>(borrow x: T) -> T { return x; }")]
fn test_constraint_checking_failure(#[case] source: &str) {
    assert_constraint_has_error(source);
}

// -----------------------------------------------------------------------------
// Multiple trait bounds (T extends Trait1 & Trait2)
// -----------------------------------------------------------------------------

#[rstest]
#[case(
    "fn f<T extends Serializable & Equatable>(borrow x: T) -> T { return x; } let y = f(\"a\");"
)]
#[case(
    "fn f<T extends Iterable & Serializable>(borrow x: T) -> number { return 0; } let y = f([1]);"
)]
fn test_multiple_constraints_success(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

// -----------------------------------------------------------------------------
// Practical constraint patterns
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends Comparable>(borrow x: T) -> bool { return true; } let y = f(1);")]
#[case("fn f<T extends Iterable>(borrow x: T) -> number { return 0; } let y = f([1]);")]
#[case("fn f<T extends Equatable>(borrow x: T) -> bool { return true; } let y = f(\"a\");")]
#[case("fn f<T extends Serializable>(borrow x: T) -> string { return str(x); } let y = f(true);")]
fn test_practical_constraint_patterns_success(#[case] source: &str) {
    assert_constraint_no_errors(source);
}

#[rstest]
#[case("fn f<T extends Iterable>(borrow x: T) -> number { return 0; } let y = f(1);")]
#[case("fn f<T extends Serializable>(borrow x: T) -> string { return str(x); } let y = f([1]);")]
fn test_practical_constraint_patterns_failure(#[case] source: &str) {
    assert_constraint_has_error(source);
}

// ============================================================================
