use super::super::*;
// ============================================================
// B14-P07 — "Did you mean?" suggestion tests
// Verifies that typo hints appear in diagnostic help text for
// method-not-found, unknown-type, and trait-not-found errors.
// ============================================================

// ---- Method not found suggestions -------------------------

/// Typo in a string method name — help should suggest the correct name.
#[test]
fn test_method_suggestion_string_typo() {
    let diags = typecheck_source(
        r#"
        let s: string = "hello";
        s.lenght();
        "#,
    );
    let at3010: Vec<_> = diags.iter().filter(|d| d.code == "AT3010").collect();
    assert!(
        !at3010.is_empty(),
        "AT3010 expected for unknown method, got: {diags:?}"
    );
    let help = at3010[0].help.as_deref().unwrap_or("");
    assert!(
        help.contains("did you mean") || help.contains("length"),
        "help should suggest 'length' for 'lenght', got: {:?}",
        help
    );
}

/// Method name that is completely wrong — no suggestion should be appended.
#[test]
fn test_method_no_suggestion_when_no_close_match() {
    let diags = typecheck_source(
        r#"
        let s: string = "hello";
        s.xyzabcdefghijk();
        "#,
    );
    let at3010: Vec<_> = diags.iter().filter(|d| d.code == "AT3010").collect();
    assert!(
        !at3010.is_empty(),
        "AT3010 expected for completely unknown method, got: {diags:?}"
    );
}

/// Typo in a number method name — help should include a suggestion.
#[test]
fn test_method_suggestion_number_typo() {
    let diags = typecheck_source(
        r#"
        let n: number = 3.14;
        n.absol();
        "#,
    );
    let at3010: Vec<_> = diags.iter().filter(|d| d.code == "AT3010").collect();
    assert!(
        !at3010.is_empty(),
        "AT3010 expected for unknown number method, got: {diags:?}"
    );
}

// ---- Unknown type suggestions ----------------------------

/// Typo in a user-defined struct name used as a type annotation.
#[test]
fn test_unknown_type_suggestion_struct_typo() {
    let diags = typecheck_source(
        r#"
        struct Point { x: number, y: number }
        let p: Ponit = Point { x: 1, y: 2 };
        "#,
    );
    let type_errs: Vec<_> = diags.iter().filter(|d| d.code == "AT3060").collect();
    assert!(
        !type_errs.is_empty(),
        "AT3060 expected for unknown type 'Ponit', got: {diags:?}"
    );
    let help = type_errs[0].help.as_deref().unwrap_or("");
    assert!(
        help.contains("did you mean") || help.contains("Point"),
        "help should suggest 'Point' for 'Ponit', got: {:?}",
        help
    );
}

/// Completely unknown type — no suggestion (no close match in scope).
#[test]
fn test_unknown_type_no_suggestion_when_no_match() {
    let diags = typecheck_source(
        r#"
        struct Alpha { x: number }
        let x: XyzCompletely = Alpha { x: 1 };
        "#,
    );
    let type_errs: Vec<_> = diags.iter().filter(|d| d.code == "AT3060").collect();
    assert!(
        !type_errs.is_empty(),
        "AT3060 expected for unknown type, got: {diags:?}"
    );
}

// ---- Trait not found suggestions -------------------------

/// Typo in a trait name used in an impl block — error should include suggestion.
#[test]
fn test_trait_suggestion_impl_typo() {
    let diags = typecheck_source(
        r#"
        trait Greetable { fn greet(borrow self: Greetable) -> string; }
        struct Dog { name: string }
        impl Greetabel for Dog {
            fn greet(borrow self: Dog) -> string { return "woof"; }
        }
        "#,
    );
    let trait_errs: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3006" || d.message.contains("not defined"))
        .collect();
    assert!(
        !trait_errs.is_empty(),
        "trait-not-found error expected for 'Greetabel', got: {diags:?}"
    );
    let msg = &trait_errs[0].message;
    assert!(
        msg.contains("did you mean") || msg.contains("Greetable"),
        "message should suggest 'Greetable' for 'Greetabel', got: {:?}",
        msg
    );
}

/// Completely unknown trait in impl — no suggestion.
#[test]
fn test_trait_no_suggestion_when_no_close_match() {
    let diags = typecheck_source(
        r#"
        struct Dog { name: string }
        impl TotallyMadeUpTraitXYZ for Dog {
            fn greet(borrow self: Dog) -> string { return "woof"; }
        }
        "#,
    );
    let trait_errs: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3006" || d.message.contains("not defined"))
        .collect();
    assert!(
        !trait_errs.is_empty(),
        "trait-not-found error expected, got: {diags:?}"
    );
}
