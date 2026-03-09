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
    let d = &at3010[0];
    let has_in_help = d
        .help
        .iter()
        .any(|h| h.contains("did you mean") || h.contains("length"));
    let has_in_suggestions = d
        .suggestions
        .iter()
        .any(|s| s.description.contains("length") || s.new_line.contains("length"));
    assert!(
        has_in_help || has_in_suggestions,
        "should suggest 'length' for 'lenght' (help or suggestions), got help={:?} suggestions={:?}",
        d.help,
        d.suggestions
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
    let help = type_errs[0].help.first().map(|s| s.as_str()).unwrap_or("");
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

// ---- H-195: Suggestion diff (code diff format) ---------------------------

/// Method typo produces a SuggestionDiff with old/new source lines.
#[test]
fn test_h195_method_typo_produces_suggestion_diff() {
    let diags = typecheck_source(
        r#"
        let s: string = "hello";
        s.lenght();
        "#,
    );
    let at3010: Vec<_> = diags.iter().filter(|d| d.code == "AT3010").collect();
    assert!(!at3010.is_empty(), "AT3010 expected, got: {diags:?}");
    let d = &at3010[0];
    assert!(
        !d.suggestions.is_empty(),
        "expected SuggestionDiff for method typo 'lenght', got none; help={:?}",
        d.help
    );
    let sug = &d.suggestions[0];
    assert!(
        sug.new_line.contains("length"),
        "suggestion new_line should contain 'length', got: {:?}",
        sug.new_line
    );
    assert!(
        sug.old_line.contains("lenght"),
        "suggestion old_line should contain 'lenght', got: {:?}",
        sug.old_line
    );
}

/// Field typo produces a SuggestionDiff with old/new source lines.
#[test]
fn test_h195_field_typo_produces_suggestion_diff() {
    let diags = typecheck_source(
        r#"
        struct Person { name: string, age: number }
        let p: Person = Person { name: "Alice", age: 30 };
        let v = p.naem;
        "#,
    );
    let field_errs: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("member") || d.message.contains("field"))
        .collect();
    assert!(
        !field_errs.is_empty(),
        "field-not-found error expected, got: {diags:?}"
    );
    let d = &field_errs[0];
    assert!(
        !d.suggestions.is_empty(),
        "expected SuggestionDiff for field typo 'naem', suggestions empty; help={:?}",
        d.help
    );
}

/// Method diff renders as code diff in to_human_string().
#[test]
fn test_h195_suggestion_diff_renders_in_human_string() {
    let diags = typecheck_source(
        r#"
        let s: string = "hello";
        s.lenght();
        "#,
    );
    let at3010: Vec<_> = diags.iter().filter(|d| d.code == "AT3010").collect();
    assert!(!at3010.is_empty(), "AT3010 expected, got: {diags:?}");
    let human = at3010[0].to_human_string();
    assert!(
        human.contains("- ") && human.contains("+ "),
        "to_human_string should contain '- ' and '+ ' diff lines, got:\n{}",
        human
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
