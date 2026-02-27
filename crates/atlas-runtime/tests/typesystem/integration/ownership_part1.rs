use super::super::*;
// Typechecker Ownership Annotation Tests (Phase 06 — Block 2)
// ============================================================================

fn typecheck_with_checker(
    source: &str,
) -> (
    Vec<atlas_runtime::diagnostic::Diagnostic>,
    atlas_runtime::typechecker::TypeChecker<'static>,
) {
    // This helper is only usable when we own the table — use typecheck_source for diagnostics-only.
    // For registry inspection we parse + bind inline.
    use atlas_runtime::binder::Binder;
    use atlas_runtime::lexer::Lexer;
    use atlas_runtime::parser::Parser;
    use atlas_runtime::typechecker::TypeChecker;

    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut table, _) = binder.bind(&program);
    // SAFETY: We box the table to pin it in memory for the 'static TypeChecker.
    // This is test-only scaffolding; the checker is dropped before the box.
    let table_ptr: *mut _ = &mut table;
    let checker_table: &'static mut _ = unsafe { &mut *table_ptr };
    let mut checker = TypeChecker::new(checker_table);
    let diags = checker.check(&program);
    (diags, checker)
}

#[test]
fn test_typechecker_stores_own_annotation() {
    use atlas_runtime::ast::OwnershipAnnotation;
    let src = "fn process(own data: number[]) -> void { }";
    let (diags, checker) = typecheck_with_checker(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let entry = checker
        .fn_ownership_registry
        .get("process")
        .expect("process not in ownership registry");
    assert_eq!(entry.0.len(), 1);
    assert_eq!(entry.0[0], Some(OwnershipAnnotation::Own));
    assert_eq!(entry.1, None); // no return annotation
}

#[test]
fn test_typechecker_warns_own_on_primitive() {
    let src = "fn bad(own _x: number) -> void { }";
    let diags = typecheck_source(src);
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning && d.code == "AT2010")
        .collect();
    assert!(
        !warnings.is_empty(),
        "expected AT2010 warning for `own` on primitive, got: {diags:?}"
    );
}

#[test]
fn test_typechecker_accepts_own_on_array() {
    let src = "fn process(own _data: number[]) -> void { }";
    let diags = typecheck_source(src);
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning && d.code == "AT2010")
        .collect();
    assert!(
        warnings.is_empty(),
        "unexpected AT2010 warning for `own` on array: {diags:?}"
    );
}

#[test]
fn test_typechecker_accepts_borrow_annotation() {
    let src = "fn read(borrow _data: number[]) -> number { return 0; }";
    let diags = typecheck_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_typechecker_stores_return_ownership() {
    use atlas_runtime::ast::OwnershipAnnotation;
    let src = "fn allocate(_size: number) -> own number { return 0; }";
    let (diags, checker) = typecheck_with_checker(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let entry = checker
        .fn_ownership_registry
        .get("allocate")
        .expect("allocate not in ownership registry");
    assert_eq!(entry.1, Some(OwnershipAnnotation::Own));
}

// ============================================================================
// Call-Site Ownership Checking Tests (Phase 07 — Block 2)
// ============================================================================

#[test]
fn test_typechecker_borrow_to_own_warning() {
    // Passing a `borrow`-annotated caller param to an `own` param should warn AT2012
    let src = r#"
fn consumer(own _data: number[]) -> void { }
fn caller(borrow data: number[]) -> void { consumer(data); }
"#;
    let diags = typecheck_source(src);
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning && d.code == "AT2012")
        .collect();
    assert!(
        !warnings.is_empty(),
        "expected AT2012 warning for borrow-to-own, got: {diags:?}"
    );
}

#[test]
fn test_typechecker_own_param_accepts_owned_value() {
    // Passing a plain (non-borrow) variable to an `own` param is OK
    let src = r#"
fn consume(own _data: number[]) -> void { }
fn caller() -> void {
    let arr: number[] = [1, 2, 3];
    consume(arr);
}
"#;
    let diags = typecheck_source(src);
    let at2012: Vec<_> = diags.iter().filter(|d| d.code == "AT2012").collect();
    assert!(
        at2012.is_empty(),
        "unexpected AT2012 for owned-value-to-own, got: {diags:?}"
    );
}

#[test]
fn test_typechecker_borrow_param_accepts_any_value() {
    // Any value can be passed to a `borrow` param — no diagnostic
    let src = r#"
fn reader(borrow _data: number[]) -> void { }
fn caller() -> void {
    let arr: number[] = [1, 2, 3];
    reader(arr);
}
"#;
    let diags = typecheck_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_typechecker_borrow_param_accepts_borrow_arg() {
    // Passing a `borrow` param to a `borrow` param is fine
    let src = r#"
fn reader(borrow _data: number[]) -> void { }
fn caller(borrow data: number[]) -> void { reader(data); }
"#;
    let diags = typecheck_source(src);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn test_typechecker_non_shared_to_shared_error() {
    // Passing a plain (non-shared) value to a `shared` param should emit AT3028
    let src = r#"
fn register(shared _handler: number[]) -> void { }
fn caller() -> void {
    let arr: number[] = [1, 2, 3];
    register(arr);
}
"#;
    let diags = typecheck_source(src);
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3028").collect();
    assert!(
        !errors.is_empty(),
        "expected AT3028 error for non-shared-to-shared, got: {diags:?}"
    );
}

// ── Phase 06: Trait Registry + Built-in Traits ─────────────────────────────

#[test]
fn test_trait_decl_no_diagnostics() {
    let diags = typecheck_source("trait Marker { }");
    assert!(
        diags.is_empty(),
        "Empty trait should produce no errors: {diags:?}"
    );
}

#[test]
fn test_trait_with_multiple_methods_no_diagnostics() {
    let diags = typecheck_source(
        "
        trait Comparable {
            fn compare(self: Comparable, other: Comparable) -> number;
            fn equals(self: Comparable, other: Comparable) -> bool;
        }
    ",
    );
    assert!(
        diags.is_empty(),
        "Multi-method trait should produce no errors: {diags:?}"
    );
}

#[test]
fn test_duplicate_trait_declaration_is_error() {
    let diags = typecheck_source(
        "
        trait Foo { fn bar() -> void; }
        trait Foo { fn baz() -> void; }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3031").collect();
    assert!(
        !errors.is_empty(),
        "Duplicate trait should produce AT3031, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_copy_is_error() {
    let diags = typecheck_source("trait Copy { fn do_copy() -> void; }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Copy should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_move_is_error() {
    let diags = typecheck_source("trait Move { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Move should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_drop_is_error() {
    let diags = typecheck_source("trait Drop { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Drop should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_display_is_error() {
    let diags = typecheck_source("trait Display { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Display should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_redefining_builtin_trait_debug_is_error() {
    let diags = typecheck_source("trait Debug { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3030").collect();
    assert!(
        !errors.is_empty(),
        "Redefining Debug should produce AT3030, got: {diags:?}"
    );
}

#[test]
fn test_impl_unknown_trait_is_error() {
    let diags = typecheck_source("impl UnknownTrait for number { }");
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        !errors.is_empty(),
        "impl unknown trait should produce AT3032, got: {diags:?}"
    );
}

#[test]
fn test_impl_known_user_trait_no_error() {
    let diags = typecheck_source(
        "
        trait Marker { }
        impl Marker for number { }
    ",
    );
    let trait_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        trait_errors.is_empty(),
        "impl known trait should not produce AT3032, got: {diags:?}"
    );
}
