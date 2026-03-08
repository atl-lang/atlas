use super::super::*;
#[test]
fn test_impl_builtin_trait_copy_no_error() {
    let diags = typecheck_source("impl Copy for number { }");
    let trait_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        trait_errors.is_empty(),
        "impl built-in Copy should not produce AT3032, borrow got: {diags:?}"
    );
}

#[test]
fn test_trait_with_generic_method_no_diagnostics() {
    let diags = typecheck_source(
        "
        trait Printer {
            fn print<T: Display>(borrow value: T) -> void;
        }
    ",
    );
    assert!(
        diags.is_empty(),
        "Trait with generic method should produce no errors: {diags:?}"
    );
}

#[test]
fn test_multiple_traits_no_conflict() {
    let diags = typecheck_source(
        "
        trait Foo { fn foo() -> void; }
        trait Bar { fn bar() -> void; }
        trait Baz { fn baz() -> void; }
    ",
    );
    assert!(
        diags.is_empty(),
        "Multiple distinct traits should produce no errors: {diags:?}"
    );
}

#[test]
fn test_impl_multiple_traits_for_same_type() {
    let diags = typecheck_source(
        "
        trait Foo { fn foo() -> void; }
        trait Bar { fn bar() -> void; }
        impl Foo for number { }
        impl Bar for number { }
    ",
    );
    let trait_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        trait_errors.is_empty(),
        "impl multiple traits should not error, borrow got: {diags:?}"
    );
}

// ── Phase 07: Impl Conformance Checking ────────────────────────────────────

#[test]
fn test_impl_complete_conformance_no_errors() {
    let diags = typecheck_source(
        "
        trait Greet { fn greet(borrow self: Greet) -> string; }
        impl Greet for number {
            fn greet(borrow self: number) -> string { return \"hello\"; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Complete impl should have no conformance errors: {diags:?}"
    );
}

#[test]
fn test_impl_missing_required_method_is_error() {
    let diags = typecheck_source(
        "
        trait Shape {
            fn area(borrow self: Shape) -> number;
            fn perimeter(borrow self: Shape) -> number;
        }
        impl Shape for number {
            fn area(borrow self: number) -> number { return 1.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3033").collect();
    assert!(
        !errors.is_empty(),
        "Missing method should produce AT3033: {diags:?}"
    );
}

#[test]
fn test_impl_wrong_return_type_is_error() {
    let diags = typecheck_source(
        "
        trait Stringify { fn to_str(borrow self: Stringify) -> string; }
        impl Stringify for number {
            fn to_str(borrow self: number) -> number { return 0.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3034").collect();
    assert!(
        !errors.is_empty(),
        "Wrong return type should produce AT3034: {diags:?}"
    );
}

#[test]
fn test_impl_wrong_param_type_is_error() {
    let diags = typecheck_source(
        "
        trait Adder { fn add(borrow self: Adder, borrow x: number) -> number; }
        impl Adder for number {
            fn add(borrow self: number, borrow x: string) -> number { return 0.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3034").collect();
    assert!(
        !errors.is_empty(),
        "Wrong param type should produce AT3034: {diags:?}"
    );
}

#[test]
fn test_duplicate_impl_is_error() {
    let diags = typecheck_source(
        "
        trait Marker { }
        impl Marker for number { }
        impl Marker for number { }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3029").collect();
    assert!(
        !errors.is_empty(),
        "Duplicate impl should produce AT3029: {diags:?}"
    );
}

#[test]
fn test_empty_trait_impl_for_multiple_types_is_valid() {
    let diags = typecheck_source(
        "
        trait Marker { }
        impl Marker for number { }
        impl Marker for string { }
        impl Marker for bool { }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3029" || d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Multiple impls of marker trait should be valid: {diags:?}"
    );
}

#[test]
fn test_impl_method_body_type_error_caught() {
    let diags = typecheck_source(
        "
        trait Negate { fn negate(borrow self: Negate) -> bool; }
        impl Negate for number {
            fn negate(borrow self: number) -> bool { return 42; }
        }
    ",
    );
    // Body return type mismatch: returning number where bool expected
    assert!(
        !diags.is_empty(),
        "Type error in impl method body should produce diagnostics"
    );
}

#[test]
fn test_impl_extra_methods_beyond_trait_allowed() {
    let diags = typecheck_source(
        "
        trait Greet { fn greet(borrow self: Greet) -> string; }
        impl Greet for number {
            fn greet(borrow self: number) -> string { return \"hi\"; }
            fn extra(borrow self: number) -> number { return 0.0; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Extra methods beyond trait should be allowed: {diags:?}"
    );
}

#[test]
fn test_impl_inherits_trait_default_method() {
    let diags = typecheck_source(
        "
        trait Greetable {
            fn greet(borrow self: Greetable) -> string { return \"Hello!\"; }
        }
        struct Robot { name: string }
        impl Greetable for Robot { }
        let r = Robot { name: \"Atlas\" };
        let s: string = r.greet();
        s
    ",
    );
    let missing_method: Vec<_> = diags.iter().filter(|d| d.code == "AT3034").collect();
    let not_impl: Vec<_> = diags.iter().filter(|d| d.code == "AT3035").collect();
    assert!(
        missing_method.is_empty(),
        "default method should satisfy impl requirements: {diags:?}"
    );
    assert!(
        not_impl.is_empty(),
        "default method should resolve trait dispatch: {diags:?}"
    );
}

#[test]
fn test_impl_multi_method_trait_all_provided() {
    let diags = typecheck_source(
        "
        trait Comparable {
            fn less_than(borrow self: Comparable, borrow other: Comparable) -> bool;
            fn equals(borrow self: Comparable, borrow other: Comparable) -> bool;
        }
        impl Comparable for number {
            fn less_than(borrow self: number, borrow other: number) -> bool { return false; }
            fn equals(borrow self: number, borrow other: number) -> bool { return false; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "All methods provided should have no conformance errors: {diags:?}"
    );
}

// ── Phase 08: User Trait Method Call Typechecking ──────────────────────────

#[test]
fn test_trait_method_call_resolves_return_type() {
    // x.display() returns string — assigning to string: no error
    let diags = typecheck_source(
        "
        trait Display { fn display(borrow self: Display) -> string; }
        impl Display for number {
            fn display(borrow self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.display();
    ",
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Trait method call should resolve return type cleanly: {diags:?}"
    );
}

#[test]
fn test_trait_method_call_wrong_assignment_is_error() {
    // x.display() returns string — assigning to number: type error
    let diags = typecheck_source(
        "
        trait Display { fn display(borrow self: Display) -> string; }
        impl Display for number {
            fn display(borrow self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let n: number = x.display();
    ",
    );
    assert!(
        !diags.is_empty(),
        "Assigning string return to number should produce a diagnostic: {diags:?}"
    );
}

#[test]
fn test_trait_method_call_number_return_resolves() {
    let diags = typecheck_source(
        "
        trait Doubler { fn double(borrow self: Doubler) -> number; }
        impl Doubler for number {
            fn double(borrow self: number) -> number { return self * 2; }
        }
        let x: number = 5;
        let y: number = x.double();
    ",
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "number-returning trait method should resolve correctly: {diags:?}"
    );
}

#[test]
fn test_trait_method_not_found_on_unimplemented_type() {
    // string doesn't implement Display in this program — AT3035 fires (trait known but not impl)
    let diags = typecheck_source(
        "
        trait Display { fn display(borrow self: Display) -> string; }
        impl Display for number {
            fn display(borrow self: number) -> string { return str(self); }
        }
        let s: string = \"hello\";
        let result: string = s.display();
    ",
    );
    // string has no Display impl here — AT3035 fires (trait exists but type doesn't implement it)
    let method_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3035").collect();
    assert!(
        !method_errors.is_empty(),
        "Method call on unimplemented type should produce AT3035: {diags:?}"
    );
}
