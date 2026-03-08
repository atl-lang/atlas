use super::super::*;

// ============================================================================
// B10-P11: Typechecker integration — method call return type resolution
// Tests that method calls on built-in types return proper types, not ?.
// ============================================================================

/// Run typechecker on source and return no errors (just warnings ok).
fn check_no_type_errors(source: &str) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut table, _) = binder.bind(&program);
    let mut checker = TypeChecker::new(&mut table);
    let diags = checker.check(&program);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no type errors for:\n{}\nGot: {:?}",
        source,
        errors
    );
}

// --- String method return types ---

#[test]
fn test_str_split_returns_array() {
    // split() returns []string — usable as array
    check_no_type_errors(
        r#"
        let s = "hello world";
        let parts = s.split(" ");
        let n = len(parts);
    "#,
    );
}

#[test]
fn test_str_to_upper_returns_string() {
    check_no_type_errors(
        r#"
        let s = "hello";
        let upper = s.toUpperCase();
        let n = len(upper);
    "#,
    );
}

#[test]
fn test_str_index_of_returns_option() {
    check_no_type_errors(
        r#"
        let s = "hello";
        let idx = s.indexOf("e");
        isSome(idx);
    "#,
    );
}

#[test]
fn test_str_includes_returns_bool() {
    check_no_type_errors(
        r#"
        let s = "hello world";
        let found = s.includes("world");
        if found { true; }
    "#,
    );
}

// --- Array method return types ---

#[test]
fn test_arr_push_returns_array() {
    check_no_type_errors(
        r#"
        let a = [1, 2, 3];
        let a2 = a.push(4);
        len(a2);
    "#,
    );
}

#[test]
fn test_arr_filter_returns_array() {
    check_no_type_errors(
        r#"
        let a = [1, 2, 3, 4];
        let evens = a.filter(fn(x) -> bool { x > 2; });
        len(evens);
    "#,
    );
}

#[test]
fn test_arr_len_returns_number() {
    check_no_type_errors(
        r#"
        let a = [1, 2, 3];
        let n = a.len();
        n + 1;
    "#,
    );
}

#[test]
fn test_arr_join_returns_string() {
    check_no_type_errors(
        r#"
        let a = ["a", "b", "c"];
        let s = a.join(", ");
        len(s);
    "#,
    );
}

#[test]
fn test_arr_includes_returns_bool() {
    check_no_type_errors(
        r#"
        let a = [1, 2, 3];
        let found = a.includes(2);
        if found { true; }
    "#,
    );
}

// --- Namespace method return types ---

#[test]
fn test_json_is_valid_returns_bool() {
    check_no_type_errors(
        r#"
        let ok = Json.isValid("{\"a\": 1}");
        if ok { true; }
    "#,
    );
}

#[test]
fn test_json_stringify_returns_string() {
    check_no_type_errors(
        r#"
        let s = Json.stringify(42);
        len(s);
    "#,
    );
}

#[test]
fn test_math_abs_returns_number() {
    check_no_type_errors(
        r#"
        let n = Math.abs(-5);
        n + 1;
    "#,
    );
}

#[test]
fn test_math_floor_returns_number() {
    check_no_type_errors(
        r#"
        let n = Math.floor(3.7);
        n + 1;
    "#,
    );
}

#[test]
fn test_math_random_returns_number() {
    check_no_type_errors(
        r#"
        let r = Math.random();
        r >= 0;
    "#,
    );
}

#[test]
fn test_file_exists_returns_bool() {
    check_no_type_errors(
        r#"
        let exists = File.exists("/tmp/foo");
        if exists { true; }
    "#,
    );
}

#[test]
fn test_path_join_returns_string() {
    check_no_type_errors(
        r#"
        let p = Path.join("/tmp", "foo", "bar.txt");
        len(p);
    "#,
    );
}

#[test]
fn test_path_is_absolute_returns_bool() {
    check_no_type_errors(
        r#"
        let ok = Path.isAbsolute("/tmp/foo");
        if ok { true; }
    "#,
    );
}

#[test]
fn test_regex_test_returns_bool() {
    check_no_type_errors(
        r#"
        let r = unwrap(Regex.new("[0-9]+"));
        let ok = Regex.test(r, "hello 42");
        if ok { true; }
    "#,
    );
}

// ============================================================================
// End B10-P11 tests
// ============================================================================
