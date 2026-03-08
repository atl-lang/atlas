use super::*;

// ============================================================================
// B10-P10: AT9000 deprecation warnings on old global names
// Old names keep working — zero breaking changes.
// Warning text: "Deprecated: use X.method() instead of old()"
// ============================================================================

/// Run typechecker on source and return all diagnostic codes + messages.
fn collect_diagnostics(source: &str) -> Vec<String> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut symbol_table, _) = binder.bind(&program);
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diagnostics = typechecker.check(&program);
    diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

fn has_at9000(source: &str) -> bool {
    collect_diagnostics(source)
        .iter()
        .any(|d| d.starts_with("AT9000"))
}

fn no_at9000(source: &str) -> bool {
    !has_at9000(source)
}

// --- Old global names emit AT9000 ---

#[test]
fn test_deprecated_parse_json() {
    assert!(has_at9000(r#"parseJSON("{\"a\": 1}");"#));
}

#[test]
fn test_deprecated_to_json() {
    assert!(has_at9000(r#"toJSON(42);"#));
}

#[test]
fn test_deprecated_is_valid_json() {
    assert!(has_at9000(r#"isValidJSON("{}");"#));
}

#[test]
fn test_deprecated_array_push() {
    assert!(has_at9000(r#"let a = [1, 2]; arrayPush(a, 3);"#));
}

#[test]
fn test_deprecated_array_pop() {
    assert!(has_at9000(r#"let a = [1, 2]; arrayPop(a);"#));
}

#[test]
fn test_deprecated_hashmap_get() {
    assert!(has_at9000(r#"let m = {}; hashMapGet(m, "k");"#));
}

#[test]
fn test_deprecated_hashmap_put() {
    assert!(has_at9000(r#"let m = {}; hashMapPut(m, "k", 1);"#));
}

#[test]
fn test_deprecated_read_file() {
    assert!(has_at9000(r#"readFile("/tmp/x.txt");"#));
}

#[test]
fn test_deprecated_write_file() {
    assert!(has_at9000(r#"writeFile("/tmp/x.txt", "hello");"#));
}

#[test]
fn test_deprecated_file_exists() {
    assert!(has_at9000(r#"fileExists("/tmp/x.txt");"#));
}

#[test]
fn test_deprecated_path_join() {
    assert!(has_at9000(r#"pathJoin("/tmp", "foo");"#));
}

#[test]
fn test_deprecated_get_env() {
    assert!(has_at9000(r#"getEnv("PATH");"#));
}

#[test]
fn test_deprecated_get_cwd() {
    assert!(has_at9000(r#"getCwd();"#));
}

#[test]
fn test_deprecated_date_time_now() {
    assert!(has_at9000(r#"dateTimeNow();"#));
}

#[test]
fn test_deprecated_regex_new() {
    assert!(has_at9000(r#"regexNew("[0-9]+");"#));
}

#[test]
fn test_deprecated_sha256() {
    assert!(has_at9000(r#"sha256("hello");"#));
}

#[test]
fn test_deprecated_sqrt() {
    assert!(has_at9000(r#"sqrt(9);"#));
}

#[test]
fn test_deprecated_abs() {
    assert!(has_at9000(r#"abs(-5);"#));
}

// --- New method syntax does NOT emit AT9000 ---

#[test]
fn test_no_warning_method_syntax() {
    assert!(no_at9000(r#"let a = [1, 2]; a.push(3);"#));
}

#[test]
fn test_no_warning_json_namespace() {
    assert!(no_at9000(r#"Json.parse("{\"a\": 1}");"#));
}

#[test]
fn test_no_warning_math_namespace() {
    assert!(no_at9000(r#"Math.sqrt(9);"#));
}

#[test]
fn test_no_warning_file_namespace() {
    assert!(no_at9000(r#"File.exists("/tmp/foo");"#));
}

// --- Old names still work (no runtime error) ---

#[test]
fn test_old_parse_json_still_works() {
    // AT9000 is a warning, not an error — old names still execute
    let src = r#"isOk(parseJSON("{\"x\": 1}"));"#;
    assert_eval_bool(src, true);
}

#[test]
fn test_old_array_push_still_works() {
    let src = r#"let a = [1, 2]; let a2 = arrayPush(a, 3); len(a2) == 3;"#;
    assert_eval_bool(src, true);
}

// ============================================================================
// End B10-P10 tests
// ============================================================================
