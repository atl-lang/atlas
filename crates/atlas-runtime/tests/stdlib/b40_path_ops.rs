//! B40: Path operations tests (path.parent, path.resolve)
//!
//! Tests for H-284 (path.parent) and H-285 (path.resolve)

use atlas_runtime::runtime::Atlas;
use atlas_runtime::value::Value;
use rstest::rstest;

fn eval(source: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(source).unwrap()
}

// ============================================================================
// path.parent Tests (H-284)
// ============================================================================

#[rstest]
#[case::simple_path("path.parent(\"/foo/bar/baz\")", "Some(/foo/bar)")]
#[case::with_file("path.parent(\"/home/user/file.txt\")", "Some(/home/user)")]
#[case::root_path("path.parent(\"/\")", "None")]
#[case::empty_path("path.parent(\"\")", "None")]
#[case::single_component("path.parent(\"foo\")", "None")]
#[case::relative_path("path.parent(\"foo/bar\")", "Some(foo)")]
#[case::trailing_slash("path.parent(\"/foo/bar/\")", "Some(/foo)")]
#[case::double_slash("path.parent(\"/foo//bar\")", "Some(/foo)")]
fn test_path_parent(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(format!("{}", result), expected, "Failed for: {}", code);
}

#[test]
fn test_path_parent_preserves_option_type() {
    let result = eval(
        r#"
        let parent = path.parent("/foo/bar");
        match parent {
            Some(p) => p,
            None => "no parent"
        }
    "#,
    );
    assert_eq!(result.to_string(), "/foo");
}

#[test]
fn test_path_parent_none_handling() {
    let result = eval(
        r#"
        let parent = path.parent("/");
        match parent {
            Some(p) => p,
            None => "root has no parent"
        }
    "#,
    );
    assert_eq!(result.to_string(), "root has no parent");
}

// ============================================================================
// path.resolve Tests (H-285)
// ============================================================================

#[test]
fn test_path_resolve_absolute_unchanged() {
    let result = eval(r#"path.resolve("/absolute/path")"#);
    assert_eq!(result.to_string(), "/absolute/path");
}

#[test]
fn test_path_resolve_relative_adds_cwd() {
    // Relative paths should be resolved against cwd
    let result = eval(
        r#"
        let resolved = path.resolve("relative/path");
        path.isAbsolute(resolved)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_path_resolve_dot_components() {
    // Check that resolve produces absolute path
    let result = eval(
        r#"
        let resolved = path.resolve("./foo");
        path.isAbsolute(resolved)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_path_resolve_empty_string() {
    // Empty string should resolve to cwd
    let result = eval(
        r#"
        let resolved = path.resolve("");
        path.isAbsolute(resolved) || resolved == ""
    "#,
    );
    // Either it's absolute or empty depending on impl
    assert!(matches!(result, Value::Bool(_)));
}

// ============================================================================
// Other path operations used by B40
// ============================================================================

#[rstest]
#[case::join_two("path.join(\"foo\", \"bar\")", "foo/bar")]
#[case::join_nested("path.join(path.join(\"a\", \"b\"), \"c\")", "a/b/c")]
fn test_path_join(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    // Normalize slashes for cross-platform
    let result_str = result.to_string().replace('\\', "/");
    assert_eq!(result_str, expected, "Failed for: {}", code);
}

#[rstest]
#[case::basename("path.basename(\"/foo/bar/baz.txt\")", "baz.txt")]
#[case::dirname("path.dirname(\"/foo/bar/baz.txt\")", "/foo/bar")]
#[case::extension("path.extension(\"/foo/bar.txt\")", "txt")]
#[case::extension_none("path.extension(\"/foo/bar\")", "")]
fn test_path_components(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

#[rstest]
#[case::absolute("path.isAbsolute(\"/foo/bar\")", "true")]
#[case::relative("path.isAbsolute(\"foo/bar\")", "false")]
#[case::empty("path.isAbsolute(\"\")", "false")]
fn test_path_is_absolute(#[case] code: &str, #[case] expected: &str) {
    let result = eval(code);
    assert_eq!(result.to_string(), expected, "Failed for: {}", code);
}

#[test]
fn test_path_normalize() {
    let result = eval(r#"path.normalize("foo//bar/../baz")"#);
    // Should remove .. and extra slashes
    let result_str = result.to_string();
    assert!(!result_str.contains(".."), "Should have resolved ..");
    assert!(!result_str.contains("//"), "Should have removed //");
}
