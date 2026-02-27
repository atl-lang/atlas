use super::*;

#[case("reflect_is_primitive(true)", true)]
#[case("reflect_is_primitive(null)", true)]
#[case("reflect_is_primitive([1, 2])", false)]
fn test_interpreter_is_primitive(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect_same_type(42, 99)", true)]
#[case("reflect_same_type(42, \"test\")", false)]
#[case("reflect_same_type(\"a\", \"b\")", true)]
#[case("reflect_same_type(true, false)", true)]
fn test_interpreter_same_type(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect_get_length([1, 2, 3])", 3.0)]
#[case("reflect_get_length(\"hello\")", 5.0)]
#[case("reflect_get_length([])", 0.0)]
#[case("reflect_get_length(\"\")", 0.0)]
fn test_interpreter_get_length(#[case] code: &str, #[case] expected: f64) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Number(expected));
}

#[rstest]
#[case("reflect_is_empty([])", true)]
#[case("reflect_is_empty(\"\")", true)]
#[case("reflect_is_empty([1])", false)]
#[case("reflect_is_empty(\"x\")", false)]
fn test_interpreter_is_empty(#[case] code: &str, #[case] expected: bool) {
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(expected));
}

#[test]
fn test_interpreter_type_describe() {
    let result = run_interpreter("reflect_type_describe(42)");
    assert_eq!(result, Value::string("primitive number type"));

    let result = run_interpreter("reflect_type_describe([1, 2])");
    // Just verify it returns a string
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_interpreter_clone() {
    let result = run_interpreter("reflect_clone(42)");
    assert_eq!(result, Value::Number(42.0));

    let result = run_interpreter("reflect_clone(\"test\")");
    assert_eq!(result, Value::string("test"));
}

#[test]
fn test_interpreter_value_to_string() {
    let result = run_interpreter("reflect_value_to_string(42)");
    assert_eq!(result, Value::string("42"));

    let result = run_interpreter("reflect_value_to_string([1, 2, 3])");
    assert_eq!(result, Value::string("[1, 2, 3]"));
}

#[test]
fn test_interpreter_deep_equals() {
    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        reflect_deep_equals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(true));

    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 4];
        reflect_deep_equals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_interpreter_nested_deep_equals() {
    let code = r#"
        let a = [[1, 2], [3, 4]];
        let b = [[1, 2], [3, 4]];
        reflect_deep_equals(a, b)
    "#;
    let result = run_interpreter(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Stdlib Reflection Integration Tests (VM)
// ============================================================================

fn run_vm(code: &str) -> Value {
    use atlas_runtime::compiler::Compiler;
    use atlas_runtime::lexer::Lexer;
    use atlas_runtime::parser::Parser;
    use atlas_runtime::vm::VM;
    use atlas_runtime::SecurityContext;

    // Add semicolon if needed (like Atlas::eval() does)
    let code = code.trim();
    let code_with_semi = if !code.is_empty() && !code.ends_with(';') && !code.ends_with('}') {
        format!("{};", code)
    } else {
        code.to_string()
    };

    // Lex
    let mut lexer = Lexer::new(&code_with_semi);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        panic!("Lexer errors: {:?}", lex_diags);
    }

    // Parse
    let mut parser = Parser::new(tokens);
    let (ast, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        panic!("Parser errors: {:?}", parse_diags);
    }

    // Compile
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("Compilation failed");

    // Run in VM
    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all())
        .expect("VM execution failed")
        .expect("VM returned None")
}

#[rstest]
#[case("reflect_typeof(42)", "number")]
#[case("reflect_typeof(\"hello\")", "string")]
#[case("reflect_typeof(true)", "bool")]
#[case("reflect_typeof(null)", "null")]
#[case("reflect_typeof([1, 2, 3])", "array")]
fn test_vm_typeof(#[case] code: &str, #[case] expected: &str) {
    let result = run_vm(code);
    assert_eq!(result, Value::string(expected));
}

#[rstest]
#[case("reflect_is_primitive(42)", true)]
#[case("reflect_is_primitive(\"test\")", true)]
#[case("reflect_is_primitive(true)", true)]
#[case("reflect_is_primitive(null)", true)]
#[case("reflect_is_primitive([1, 2])", false)]
fn test_vm_is_primitive(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect_same_type(42, 99)", true)]
#[case("reflect_same_type(42, \"test\")", false)]
#[case("reflect_same_type(\"a\", \"b\")", true)]
#[case("reflect_same_type(true, false)", true)]
fn test_vm_same_type(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[rstest]
#[case("reflect_get_length([1, 2, 3])", 3.0)]
#[case("reflect_get_length(\"hello\")", 5.0)]
#[case("reflect_get_length([])", 0.0)]
#[case("reflect_get_length(\"\")", 0.0)]
fn test_vm_get_length(#[case] code: &str, #[case] expected: f64) {
    let result = run_vm(code);
    assert_eq!(result, Value::Number(expected));
}

#[rstest]
#[case("reflect_is_empty([])", true)]
#[case("reflect_is_empty(\"\")", true)]
#[case("reflect_is_empty([1])", false)]
#[case("reflect_is_empty(\"x\")", false)]
fn test_vm_is_empty(#[case] code: &str, #[case] expected: bool) {
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(expected));
}

#[test]
fn test_vm_type_describe() {
    let result = run_vm("reflect_type_describe(42)");
    assert_eq!(result, Value::string("primitive number type"));

    let result = run_vm("reflect_type_describe([1, 2])");
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_vm_clone() {
    let result = run_vm("reflect_clone(42)");
    assert_eq!(result, Value::Number(42.0));

    let result = run_vm("reflect_clone(\"test\")");
    assert_eq!(result, Value::string("test"));
}

#[test]
fn test_vm_value_to_string() {
    let result = run_vm("reflect_value_to_string(42)");
    assert_eq!(result, Value::string("42"));

    let result = run_vm("reflect_value_to_string([1, 2, 3])");
    assert_eq!(result, Value::string("[1, 2, 3]"));
}

#[test]
fn test_vm_deep_equals() {
    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        reflect_deep_equals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(true));

    let code = r#"
        let a = [1, 2, 3];
        let b = [1, 2, 4];
        reflect_deep_equals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_vm_nested_deep_equals() {
    let code = r#"
        let a = [[1, 2], [3, 4]];
        let b = [[1, 2], [3, 4]];
        reflect_deep_equals(a, b)
    "#;
    let result = run_vm(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Parity Verification Tests
// ============================================================================

#[rstest]
#[case("reflect_typeof(42)")]
#[case("reflect_typeof(\"test\")")]
#[case("reflect_typeof([1, 2, 3])")]
#[case("reflect_is_primitive(42)")]
#[case("reflect_is_primitive([1])")]
#[case("reflect_same_type(1, 2)")]
#[case("reflect_same_type(1, \"a\")")]
#[case("reflect_get_length([1, 2, 3])")]
#[case("reflect_get_length(\"hello\")")]
#[case("reflect_is_empty([])")]
#[case("reflect_is_empty([1])")]
#[case("reflect_clone(42)")]
#[case("reflect_value_to_string(42)")]
fn test_parity_reflection_functions(#[case] code: &str) {
    let interpreter_result = run_interpreter(code);
    let vm_result = run_vm(code);

    assert_eq!(
        interpreter_result, vm_result,
        "Parity violation for: {}",
        code
    );
}

#[test]
fn test_parity_deep_equals() {
    let cases = vec![
        "reflect_deep_equals([1, 2], [1, 2])",
        "reflect_deep_equals([1, 2], [1, 3])",
        "reflect_deep_equals([[1]], [[1]])",
        "reflect_deep_equals(42, 42)",
        "reflect_deep_equals(\"a\", \"a\")",
    ];

    for code in cases {
        let interpreter_result = run_interpreter(code);
        let vm_result = run_vm(code);

        assert_eq!(
            interpreter_result, vm_result,
            "Parity violation for: {}",
            code
        );
    }
}

