use super::super::*;
#[test]
fn test_stdlib_method_not_shadowed_by_trait() {
    // Array push() is stdlib — a trait method named push doesn't conflict
    let diags = typecheck_source(
        "
        trait Pushable { fn push(self: Pushable, x: number) -> void; }
        impl Pushable for number { fn push(self: number, x: number) -> void { } }
        let arr: number[] = [1, 2, 3];
        arr = arr.push(4);
    ",
    );
    // arr.push(4) hits stdlib — no AT3010 expected
    let method_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3010").collect();
    assert!(
        method_errors.is_empty(),
        "Stdlib array.push should not be shadowed: {diags:?}"
    );
}

#[test]
fn test_trait_method_bool_return_resolves() {
    let diags = typecheck_source(
        "
        trait Check { fn is_valid(self: Check) -> bool; }
        impl Check for number {
            fn is_valid(self: number) -> bool { return self > 0; }
        }
        let x: number = 5;
        let ok: bool = x.is_valid();
    ",
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "bool-returning trait method should resolve correctly: {diags:?}"
    );
}

// ── Phase 09: Copy/Move + Ownership Integration ─────────────────────────────

#[test]
fn test_number_passed_without_annotation_no_error() {
    // number is Copy — no ownership annotation needed
    let diags = typecheck_source(
        "
        fn double(x: number) -> number { return x * 2; }
        let n: number = 5;
        let result: number = double(n);
    ",
    );
    // Should produce no ownership-related diagnostics
    let ownership_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_diags.is_empty(),
        "number is Copy, no AT2013 expected: {diags:?}"
    );
}

#[test]
fn test_string_passed_without_annotation_no_error() {
    let diags = typecheck_source(
        "
        fn greet(name: string) -> string { return name; }
        let s: string = \"hello\";
        let g: string = greet(s);
    ",
    );
    let ownership_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_diags.is_empty(),
        "string is Copy, no AT2013 expected: {diags:?}"
    );
}

#[test]
fn test_bool_passed_without_annotation_no_error() {
    let diags = typecheck_source(
        "
        fn negate(b: bool) -> bool { return !b; }
        let flag: bool = true;
        let result: bool = negate(flag);
    ",
    );
    let ownership_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_diags.is_empty(),
        "bool is Copy, no AT2013 expected: {diags:?}"
    );
}

#[test]
fn test_array_passed_without_annotation_no_error() {
    let diags = typecheck_source(
        "
        fn first(arr: number[]) -> number { return arr[0]; }
        let a: number[] = [1, 2, 3];
        let n: number = first(a);
    ",
    );
    let ownership_diags: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_diags.is_empty(),
        "array is Copy (CoW), no AT2013 expected: {diags:?}"
    );
}

#[test]
fn test_redefine_builtin_copy_trait_is_error() {
    // Attempting to declare `trait Copy` should produce AT3030
    let diags = typecheck_source("trait Copy { fn do_copy() -> void; }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Copy should produce AT3030: {diags:?}"
    );
}

#[test]
fn test_explicit_own_on_copy_type_allowed() {
    // own annotation on Copy type is redundant but not an error
    let diags = typecheck_source(
        "
        fn consume(own x: number) -> number { return x; }
        let n: number = 42;
        let result: number = consume(n);
    ",
    );
    // No errors — own on Copy is always valid
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Explicit own on Copy type should not produce errors: {diags:?}"
    );
}

#[test]
fn test_impl_copy_for_type_registers_in_trait_registry() {
    // impl Copy for number (built-in Copy, already in registry) should not AT3030
    let diags = typecheck_source("impl Copy for number { }");
    let builtin_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        builtin_errors.is_empty(),
        "impl Copy for number should not produce AT3030: {diags:?}"
    );
}

// ── Phase 10: Trait Bounds Enforcement ─────────────────────────────────────

#[test]
fn test_copy_bound_satisfied_by_number() {
    let diags = typecheck_source(
        "
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let n: number = safe_copy(42);
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "number satisfies Copy bound, no AT3037 expected: {diags:?}"
    );
}

#[test]
fn test_copy_bound_satisfied_by_string() {
    let diags = typecheck_source(
        "
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let s: string = safe_copy(\"hello\");
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "string satisfies Copy bound, no AT3037 expected: {diags:?}"
    );
}

#[test]
fn test_copy_bound_satisfied_by_bool() {
    let diags = typecheck_source(
        "
        fn safe_copy<T: Copy>(x: T) -> T { return x; }
        let b: bool = safe_copy(true);
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "bool satisfies Copy bound, no AT3037 expected: {diags:?}"
    );
}

#[test]
fn test_unbounded_type_param_no_error() {
    // Unbounded type params must still work
    let diags = typecheck_source(
        "
        fn identity<T>(x: T) -> T { return x; }
        let n: number = identity(42);
        let s: string = identity(\"hello\");
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "Unbounded type params should not produce AT3037: {diags:?}"
    );
}

#[test]
fn test_user_trait_bound_satisfied() {
    let diags = typecheck_source(
        "
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number {
            fn print_self(self: number) -> void { }
        }
        fn log_it<T: Printable>(x: T) -> void { }
        log_it(42);
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "number implements Printable, bound satisfied: {diags:?}"
    );
}

#[test]
fn test_user_trait_bound_not_satisfied_is_error() {
    let diags = typecheck_source(
        "
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number {
            fn print_self(self: number) -> void { }
        }
        fn log_it<T: Printable>(x: T) -> void { }
        log_it(\"hello\");
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        !bound_errors.is_empty(),
        "string doesn't implement Printable — AT3037 expected: {diags:?}"
    );
}

#[test]
fn test_multiple_bounds_all_satisfied() {
    let diags = typecheck_source(
        "
        trait Printable { fn print_self(self: Printable) -> void; }
        impl Printable for number { fn print_self(self: number) -> void { } }
        fn process<T: Copy + Printable>(x: T) -> void { }
        process(42);
    ",
    );
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        bound_errors.is_empty(),
        "number is Copy AND Printable, both bounds satisfied: {diags:?}"
    );
}

#[test]
fn test_multiple_bounds_one_missing_is_error() {
    let diags = typecheck_source(
        "
        trait Printable { fn print_self(self: Printable) -> void; }
        fn process<T: Copy + Printable>(x: T) -> void { }
        process(42);
    ",
    );
    // number is Copy but no impl Printable here
    let bound_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3037").collect();
    assert!(
        !bound_errors.is_empty(),
        "Missing Printable impl — AT3037 expected: {diags:?}"
    );
}
