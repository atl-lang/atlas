// H-110: match on user-defined enum in function body returns '?' type
// H-111: matching same user enum variable twice at top level — second result is '?'
use super::super::*;

/// H-110: enum param in fn body — calling describe(s) should not trigger AT2013
/// (AT2013 fires when the argument type is '?' / Unknown, meaning Status resolved to Unknown)
#[test]
fn test_h110_enum_match_in_fn_body_no_at2013() {
    let diagnostics = typecheck_source(
        r#"
        enum Status { Active, Inactive }
        fn describe(borrow s: Status): string {
            match s {
                Status::Active => "active",
                Status::Inactive => "inactive"
            }
        }
        let s: Status = Status::Active;
        let result: string = describe(s);
        "#,
    );
    let at2013: Vec<_> = diagnostics.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        at2013.is_empty(),
        "H-110: AT2013 (Type '?' is not Copy) should not fire when passing a user enum arg — enum type resolves to Unknown. Got: {:?}",
        at2013.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// H-110: enum param — AT2013 should not fire for enum-typed arguments
#[test]
fn test_h110_enum_param_not_flagged_as_move() {
    let diagnostics = typecheck_source(
        r#"
        enum Color { Red, Green, Blue }
        fn name(borrow c: Color): string {
            match c {
                Color::Red => "red",
                Color::Green => "green",
                Color::Blue => "blue"
            }
        }
        let c: Color = Color::Red;
        let n: string = name(c);
        "#,
    );
    let at2013: Vec<_> = diagnostics.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        at2013.is_empty(),
        "H-110: AT2013 should not fire for user-defined enum arguments, got: {:?}",
        at2013.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

/// H-111: second match on same enum variable at top-level should not return '?'
#[test]
fn test_h111_double_match_same_enum_var() {
    let diagnostics = typecheck_source(
        r#"
        enum Status { Active, Inactive }
        let s: Status = Status::Active;
        let label1: string = match s { Status::Active => "active", Status::Inactive => "inactive" };
        let label2: string = match s { Status::Active => "yes", Status::Inactive => "no" };
        "#,
    );
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "H-111: second match on same enum var should not produce errors, got: {:?}",
        errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// H-120: enum tuple-variant bindings must resolve to declared field types, not Unknown
#[test]
fn test_h120_enum_tuple_variant_bindings_typed() {
    let diagnostics = typecheck_source(
        r#"
enum Shape { Circle(number), Rect(number, number) }
let s = Shape::Circle(5.0);
let area: number = match s {
    Shape::Circle(r) => r * r * 3.14,
    Shape::Rect(w, h) => w * h,
};
        "#,
    );
    assert_no_errors(&diagnostics);
}

// H-223: bare variant patterns — no EnumName:: prefix required in match arms
#[test]
fn test_h223_bare_unit_variants_in_match() {
    let diagnostics = typecheck_source(
        r#"
enum Direction { North, South, East, West }
fn go(d: Direction): string {
    match d {
        North => "north",
        South => "south",
        East  => "east",
        West  => "west",
    }
}
        "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_h223_bare_tuple_variants_in_match() {
    let diagnostics = typecheck_source(
        r#"
enum CommandResult { Done(string), Fail(string), Pending }
fn handle(r: CommandResult): string {
    match r {
        Done(msg)  => msg,
        Fail(err)  => err,
        Pending    => "pending",
    }
}
        "#,
    );
    assert_no_errors(&diagnostics);
}

#[test]
fn test_h223_explicit_prefix_still_valid() {
    // EnumName:: explicit form must still work after H-223
    let diagnostics = typecheck_source(
        r#"
enum Status { Active, Inactive }
fn check(s: Status): string {
    match s {
        Status::Active   => "active",
        Status::Inactive => "inactive",
    }
}
        "#,
    );
    assert_no_errors(&diagnostics);
}

// H-229: enum variant tuple args must be type-checked against declared field types
#[test]
fn test_h229_enum_variant_arg_type_mismatch_rejected() {
    let diagnostics = typecheck_source(
        r#"
enum Status { Active(string), Inactive }
let s = Status::Active(42);
        "#,
    );
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "H-229: Status::Active(42) where Active(string) should produce a type error, got none"
    );
}

#[test]
fn test_h229_enum_variant_arg_count_mismatch_rejected() {
    let diagnostics = typecheck_source(
        r#"
enum Shape { Circle(number), Rect(number, number) }
let s = Shape::Circle(1.0, 2.0);
        "#,
    );
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == atlas_runtime::diagnostic::DiagnosticLevel::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "H-229: Shape::Circle(1.0, 2.0) where Circle has 1 field should produce an error"
    );
}

#[test]
fn test_h229_enum_variant_correct_args_accepted() {
    let diagnostics = typecheck_source(
        r#"
enum Status { Active(string), Inactive }
let s = Status::Active("running");
        "#,
    );
    assert_no_errors(&diagnostics);
}
