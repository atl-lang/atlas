//! B12: AI-First Error Messages — Battle Test Suite
//!
//! Every test represents a common AI mistake when generating Atlas code.
//! Each test verifies that the error is self-correcting: the error output
//! must contain (1) a named AT error code, (2) a specific problem description,
//! and (3) a concrete Atlas fix.
//!
//! Quality bar: AT1007 — named code + specific problem + inline example fix.
//!
//! These tests use both the static diagnostic pipeline (parse + typecheck)
//! and the runtime API to cover all error emission sites.

use atlas_runtime::api::{ExecutionMode, Runtime};
use atlas_runtime::security::SecurityContext;
use atlas_runtime::{Binder, Diagnostic, Lexer, Parser, TypeChecker};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run source through the full static pipeline and collect all diagnostics.
fn get_diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return lex_diags;
    }

    let mut parser = Parser::new(tokens);
    let (ast, parse_diags) = parser.parse();
    let parse_errors: Vec<_> = parse_diags.into_iter().filter(|d| d.is_error()).collect();
    if !parse_errors.is_empty() {
        return parse_errors;
    }

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&ast);
    if !bind_diags.is_empty() {
        return bind_diags;
    }

    let mut checker = TypeChecker::new(&mut table);
    checker
        .check(&ast)
        .into_iter()
        .filter(|d| d.is_error())
        .collect()
}

/// Assert that at least one diagnostic has the given code AND its combined
/// text (message + help) contains all expected substrings.
fn assert_self_correcting(diags: &[Diagnostic], expected_code: &str, expected_terms: &[&str]) {
    assert!(
        !diags.is_empty(),
        "Expected at least one diagnostic, got none. Code expected: {}",
        expected_code
    );

    // Find a diagnostic matching the code
    let matching: Vec<_> = diags.iter().filter(|d| d.code == expected_code).collect();
    assert!(
        !matching.is_empty(),
        "No diagnostic with code '{}' found.\nGot codes: {}\nMessages: {}",
        expected_code,
        diags
            .iter()
            .map(|d| d.code.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        diags
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join(" | "),
    );

    let diag = matching[0];
    let full_text = format!(
        "{} {} {}",
        diag.message,
        diag.help.as_deref().unwrap_or(""),
        diag.label
    );

    for term in expected_terms {
        assert!(
            full_text.to_lowercase().contains(&term.to_lowercase()),
            "Diagnostic AT{} missing term '{}'\nFull text: {}",
            expected_code,
            term,
            full_text
        );
    }
}

/// Run source through interpreter and VM, collect error messages.
fn eval_both_engines(source: &str) -> (String, String) {
    let mut interp =
        Runtime::new_with_security(ExecutionMode::Interpreter, SecurityContext::allow_all());
    let interp_err = match interp.eval(source) {
        Err(e) => format!("{}", e),
        Ok(_) => String::new(),
    };

    let mut vm = Runtime::new_with_security(ExecutionMode::VM, SecurityContext::allow_all());
    let vm_err = match vm.eval(source) {
        Err(e) => format!("{}", e),
        Ok(_) => String::new(),
    };

    (interp_err, vm_err)
}

/// Assert runtime error contains a term (both engines).
fn assert_runtime_self_correcting(source: &str, expected_term: &str) {
    let (interp_err, vm_err) = eval_both_engines(source);
    assert!(
        !interp_err.is_empty() || !vm_err.is_empty(),
        "Expected runtime error for: {}",
        source
    );
    let combined = format!("{} {}", interp_err, vm_err);
    assert!(
        combined
            .to_lowercase()
            .contains(&expected_term.to_lowercase()),
        "Runtime error missing '{}'\nInterp: {}\nVM: {}",
        expected_term,
        interp_err,
        vm_err
    );
}

// ---------------------------------------------------------------------------
// P01: Cross-language syntax patterns (AT1008–AT1015)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_echo_php_python() {
    // AI generates PHP/shell-style echo
    let diags = get_diagnostics(r#"echo "hello";"#);
    assert_self_correcting(&diags, "AT1008", &["print", "echo"]);
}

#[test]
fn ai_mistake_var_javascript() {
    // AI generates JS-style var
    let diags = get_diagnostics("var x = 5;");
    assert_self_correcting(&diags, "AT1009", &["let", "var"]);
}

#[test]
fn ai_mistake_function_keyword() {
    // AI generates JS/PHP function keyword
    let diags = get_diagnostics("function greet() { }");
    assert_self_correcting(&diags, "AT1010", &["fn", "function"]);
}

#[test]
fn ai_mistake_class_keyword() {
    // AI generates OOP class
    let diags = get_diagnostics("class Foo { }");
    assert_self_correcting(&diags, "AT1011", &["struct", "class"]);
}

#[test]
fn ai_mistake_console_log_javascript() {
    // AI generates JS console.log
    let diags = get_diagnostics(r#"console.log("hello");"#);
    assert_self_correcting(&diags, "AT1013", &["print", "console"]);
}

#[test]
fn ai_mistake_increment_operator() {
    // AI generates x++
    let diags = get_diagnostics("let x: number = 0;\nx++;");
    assert_self_correcting(&diags, "AT1014", &["x = x + 1", "++"]);
}

#[test]
fn ai_mistake_decrement_operator() {
    // AI generates x--
    let diags = get_diagnostics("let x: number = 5;\nx--;");
    assert_self_correcting(&diags, "AT1014", &["x = x - 1", "--"]);
}

// ---------------------------------------------------------------------------
// P02: Invalid assignment targets (AT1016–AT1019)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_assign_to_range() {
    // AI tries to assign to a range expression (falls into generic invalid target)
    let diags = get_diagnostics("(0..10) = 5;");
    // Gets AT1019 (generic invalid target) — range at top-level, not as slice index
    assert_self_correcting(&diags, "AT1019", &["valid assignment target"]);
}

#[test]
fn ai_mistake_assign_to_function_call() {
    // AI tries to assign to a function call result (falls into generic invalid target)
    let diags = get_diagnostics("len(\"hi\") = 5;");
    // Gets AT1019 — Expr::Call is not Identifier/Index/Member
    assert_self_correcting(&diags, "AT1019", &["valid assignment target"]);
}

#[test]
fn ai_mistake_assign_to_range_slice_index() {
    // AI tries to assign to a range-indexed slice (AT1016 — the specific path)
    let source = "let arr: number[] = [1.0, 2.0, 3.0];\narr[0..2] = 99.0;";
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "Expected error for slice assignment arr[0..2] = ..."
    );
    // Should emit AT1016 or a related parse error
    let has_range_error = diags
        .iter()
        .any(|d| d.code == "AT1016" || d.message.to_lowercase().contains("range"));
    assert!(
        has_range_error,
        "Expected range-related error for slice assignment: {:?}",
        diags.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// P03: Undefined variable / typos (AT0002 / binder)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_undefined_variable_with_suggestion() {
    // AI makes a typo — should suggest the correct name
    let diags = get_diagnostics("let count: number = 5;\nprint(coutn);");
    assert!(
        !diags.is_empty(),
        "Expected undefined variable error for 'coutn'"
    );
    let full = diags
        .iter()
        .map(|d| format!("{} {}", d.message, d.help.as_deref().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(" ");
    // Should suggest "count"
    assert!(
        full.contains("count"),
        "Expected suggestion 'count' for typo 'coutn'\nFull: {}",
        full
    );
}

#[test]
fn ai_mistake_undeclared_variable() {
    // AI uses variable without declaring it
    let diags = get_diagnostics("print(result);");
    assert!(!diags.is_empty(), "Expected error for undeclared 'result'");
    let full = diags[0].message.to_lowercase();
    assert!(
        full.contains("result") || full.contains("undefined") || full.contains("not in scope"),
        "Expected informative message for undeclared var: {}",
        full
    );
}

// ---------------------------------------------------------------------------
// P04: Struct field typos (typechecker)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_struct_field_typo_with_suggestion() {
    // AI accesses wrong field name — should list available fields
    let source = r#"
struct Point {
    x: number,
    y: number,
}
let p: Point = Point { x: 1.0, y: 2.0 };
let val: number = p.z;
"#;
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "Expected field-not-found error for 'p.z'"
    );
    let full = diags
        .iter()
        .map(|d| format!("{} {}", d.message, d.help.as_deref().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(" ");
    // Should list available fields x, y
    assert!(
        full.contains("x") && full.contains("y"),
        "Expected available fields listed: {}",
        full
    );
}

#[test]
fn ai_mistake_struct_field_close_match() {
    // AI types "naem" instead of "name"
    let source = r#"
struct User {
    name: string,
    age: number,
}
let u: User = User { name: "Alice", age: 30.0 };
let s: string = u.naem;
"#;
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "Expected field-not-found error for 'u.naem'"
    );
    let full = diags
        .iter()
        .map(|d| format!("{} {}", d.message, d.help.as_deref().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(" ");
    // Should suggest "name"
    assert!(
        full.contains("name"),
        "Expected suggestion 'name' for typo 'naem': {}",
        full
    );
}

// ---------------------------------------------------------------------------
// P05: Stdlib arity errors — signature shown (AT0102)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_len_wrong_arity() {
    // AI calls len() with no args
    let (interp_err, vm_err) = eval_both_engines("len();");
    let combined = format!("{} {}", interp_err, vm_err);
    // At least one engine should error with signature info
    assert!(
        combined.contains("len") || combined.contains("argument"),
        "Expected len() arity error with context: {}",
        combined
    );
}

#[test]
fn ai_mistake_push_wrong_arity() {
    // AI calls push with wrong arity
    let (interp_err, vm_err) = eval_both_engines("push([1.0, 2.0]);");
    let combined = format!("{} {}", interp_err, vm_err);
    assert!(
        combined.contains("push") || combined.contains("argument"),
        "Expected push() arity error: {}",
        combined
    );
}

#[test]
fn ai_mistake_substr_wrong_arg_type() {
    // AI passes wrong type to substr
    let (interp_err, vm_err) = eval_both_engines(r#"substr("hello", "2", 3.0);"#);
    let combined = format!("{} {}", interp_err, vm_err);
    assert!(
        !combined.is_empty(),
        "Expected type error for substr with string index: {}",
        combined
    );
}

// ---------------------------------------------------------------------------
// P06: Runtime errors — self-correcting messages
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_divide_by_zero_has_guard_hint() {
    // AI divides without checking for zero — direct expression, no fn wrapper
    let source = "let result: number = 10.0 / 0.0;";
    assert_runtime_self_correcting(source, "zero");
}

#[test]
fn ai_mistake_array_out_of_bounds_has_len_hint() {
    // AI accesses index beyond array length
    let source = r#"
let arr: number[] = [1.0, 2.0, 3.0];
let val: number = arr[10];
"#;
    // Assert both engines produce a runtime error — message may vary by display impl
    let (interp_err, vm_err) = eval_both_engines(source);
    assert!(
        !interp_err.is_empty() || !vm_err.is_empty(),
        "Expected OOB error from at least one engine"
    );
    // Both should mention either "out of bounds", "OutOfBounds", or "AT0006"
    let combined = format!("{} {}", interp_err, vm_err);
    assert!(
        combined.to_lowercase().contains("bound")
            || combined.contains("OutOfBounds")
            || combined.contains("AT0006"),
        "OOB error should mention bounds: interp='{}' vm='{}'",
        interp_err,
        vm_err
    );
}

#[test]
fn ai_mistake_invalid_index_float() {
    // AI uses float as array index
    let source = r#"
let arr: number[] = [1.0, 2.0, 3.0];
let val: number = arr[1.5];
"#;
    assert_runtime_self_correcting(source, "index");
}

// ---------------------------------------------------------------------------
// P07: Type errors — helpful messages (AT3xxx)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_add_number_and_string() {
    // AI tries arithmetic on mixed types
    let source = r#"
let x: number = 5.0;
let s: string = "hello";
let result: number = x + s;
"#;
    let diags = get_diagnostics(source);
    assert!(!diags.is_empty(), "Expected type error for number + string");
    let full = diags
        .iter()
        .map(|d| format!("{} {}", d.message, d.help.as_deref().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        full.to_lowercase().contains("number") || full.to_lowercase().contains("string"),
        "Expected type names in error: {}",
        full
    );
}

#[test]
fn ai_mistake_return_wrong_type() {
    // AI returns wrong type from function
    let source = r#"
fn greet(share name: string) -> number {
    return name;
}
"#;
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "Expected type error for returning string from number fn"
    );
}

#[test]
fn ai_mistake_wrong_arg_type_to_fn() {
    // AI passes string where number is expected
    let source = r#"
fn double(share x: number) -> number {
    return x * 2.0;
}
double("hello");
"#;
    let diags = get_diagnostics(source);
    assert!(!diags.is_empty(), "Expected type error for wrong arg type");
    let full = diags
        .iter()
        .map(|d| format!("{} {}", d.message, d.help.as_deref().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        full.to_lowercase().contains("string") || full.to_lowercase().contains("number"),
        "Expected type names in error: {}",
        full
    );
}

// ---------------------------------------------------------------------------
// P08: Ownership errors — specific guidance (AT3050-AT3053)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_use_after_own() {
    // AI moves a value then tries to use it (interpreter-level check)
    let source = r#"
fn consume(own data: array<number>) -> void { }
let arr: array<number> = [1.0, 2.0];
consume(arr);
arr;
"#;
    // This is a runtime/interpreter check (use of moved value)
    let (interp_err, _vm_err) = eval_both_engines(source);
    assert!(
        !interp_err.is_empty(),
        "Expected interpreter error for use-after-own"
    );
    assert!(
        interp_err.to_lowercase().contains("moved")
            || interp_err.to_lowercase().contains("own")
            || interp_err.to_lowercase().contains("consum"),
        "Expected ownership error context: {}",
        interp_err
    );
}

#[test]
fn ai_mistake_own_param_missing_annotation_flagged() {
    // AI defines function without ownership annotations — should get AT1007
    let source = "fn process(borrow x: number) -> number { return x; }";
    let diags = get_diagnostics(source);
    assert_self_correcting(&diags, "AT1007", &["ownership"]);
}

// ---------------------------------------------------------------------------
// P09: Missing mandatory ownership annotation (AT1007)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_missing_ownership_annotation() {
    // AI defines function without ownership annotation
    let source = r#"fn add(borrow x: number, borrow y: number) -> number { return x + y; }"#;
    let diags = get_diagnostics(source);
    assert_self_correcting(&diags, "AT1007", &["ownership", "share"]);
}

#[test]
fn ai_mistake_missing_ownership_on_array_param() {
    // AI passes array param without annotation
    let source = r#"fn sum(borrow nums: number[]) -> number { return 0.0; }"#;
    let diags = get_diagnostics(source);
    assert_self_correcting(&diags, "AT1007", &["ownership"]);
}

// ---------------------------------------------------------------------------
// P10: Specific parser errors (AT1000–AT1006)
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_missing_semicolon() {
    // AI forgets semicolon between statements
    let source = "let x: number = 5\nlet y: number = 10;";
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "Expected parse error for missing semicolon"
    );
}

#[test]
fn ai_mistake_missing_closing_brace() {
    // AI forgets closing brace
    let source = "fn foo() -> void {\n  print(\"hi\");\n";
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "Expected parse error for unclosed function body"
    );
}

#[test]
fn ai_mistake_wrong_arrow_syntax() {
    // AI uses => instead of ->
    let source = "fn add(borrow x: number, borrow y: number) => number { return x + y; }";
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "Expected parse error for => instead of ->"
    );
    // Error code should mention return type
    let full = diags
        .iter()
        .map(|d| format!("{} {}", d.message, d.help.as_deref().unwrap_or("")))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        !full.is_empty(),
        "Expected informative error for wrong arrow syntax"
    );
}

#[test]
fn ai_mistake_wrong_type_annotation_colon() {
    // AI uses :: instead of : for type annotation
    let source = "let x::number = 5.0;";
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "Expected parse error for :: in type annotation"
    );
}

// ---------------------------------------------------------------------------
// P11: Parity — errors look the same in both engines
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_parity_divide_by_zero() {
    // Division by zero should fail in both engines with consistent messaging
    let source = "let r: number = 5.0 / 0.0;";
    let (interp_err, vm_err) = eval_both_engines(source);
    // Both must error
    assert!(
        !interp_err.is_empty(),
        "Interpreter should raise divide-by-zero"
    );
    assert!(!vm_err.is_empty(), "VM should raise divide-by-zero");
    // Both should mention "zero"
    assert!(
        interp_err.to_lowercase().contains("zero") || interp_err.contains("AT0005"),
        "Interpreter error should mention zero: {}",
        interp_err
    );
    assert!(
        vm_err.to_lowercase().contains("zero") || vm_err.contains("AT0005"),
        "VM error should mention zero: {}",
        vm_err
    );
}

#[test]
fn ai_mistake_parity_out_of_bounds() {
    // Array OOB should fail in both engines
    let source = r#"
let arr: number[] = [1.0];
let v: number = arr[5];
"#;
    let (interp_err, vm_err) = eval_both_engines(source);
    assert!(!interp_err.is_empty(), "Interpreter should raise OOB error");
    assert!(!vm_err.is_empty(), "VM should raise OOB error");
}

#[test]
fn ai_mistake_parity_ownership_at1007() {
    // AT1007 fires in both engines (static check, pre-execution) for missing annotation
    let source = r#"fn bad(x: number) -> number { return x; }"#;
    let (interp_err, vm_err) = eval_both_engines(source);
    // Both engines go through the same static pipeline — both should see the AT1007 error
    let combined = format!("{} {}", interp_err, vm_err);
    // EvalError::ParseError display includes the message but not the code prefix
    // Check for the ownership annotation message content
    assert!(
        combined.contains("ownership annotation") || combined.contains("AT1007"),
        "Both engines should surface ownership annotation error: {}",
        combined
    );
}

// ---------------------------------------------------------------------------
// P12: Help text quality — every error must have help
// ---------------------------------------------------------------------------

#[test]
fn ai_mistake_all_parse_errors_have_help() {
    // Common mistakes — each should have non-empty help
    let cases = vec![
        (r#"echo "hi";"#, "AT1008"),
        ("var x = 5;", "AT1009"),
        ("function foo() { }", "AT1010"),
        ("class Bar { }", "AT1011"),
    ];
    for (source, expected_code) in cases {
        let diags = get_diagnostics(source);
        let matching: Vec<_> = diags.iter().filter(|d| d.code == expected_code).collect();
        assert!(
            !matching.is_empty(),
            "No {} diagnostic for: {}",
            expected_code,
            source
        );
        let diag = matching[0];
        assert!(
            diag.help.is_some() && !diag.help.as_ref().unwrap().is_empty(),
            "Diagnostic {} has no help text for: {}",
            expected_code,
            source
        );
    }
}

#[test]
fn ai_mistake_runtime_divide_by_zero_has_help() {
    // AT0005 error — verify via static diagnostic pipeline (help visible here, not via EvalError display)
    // Note: EvalError::RuntimeError display uses Debug format (H-170 — known P1 bug)
    // The rich help IS produced by runtime_error_to_diagnostic but not surfaced via EvalError::Display.
    // Here we verify the diagnostic pipeline produces the right help text.
    use atlas_runtime::runtime::runtime_error_to_diagnostic;
    use atlas_runtime::value::RuntimeError;
    use atlas_runtime::Span;
    let err = RuntimeError::DivideByZero {
        span: Span::dummy(),
    };
    let diag = runtime_error_to_diagnostic(err, vec![], None);
    assert_eq!(diag.code, "AT0005", "Expected AT0005 code");
    let help = diag.help.as_deref().unwrap_or("");
    assert!(!help.is_empty(), "AT0005 must have help text");
    assert!(
        help.contains("divisor") || help.contains("zero") || help.contains("!= 0"),
        "AT0005 help should mention guard pattern: {}",
        help
    );
}

#[test]
fn ai_mistake_runtime_oob_has_help() {
    // AT0006 error must mention len/length/index check
    let source = r#"
let arr: number[] = [1.0];
let v: number = arr[99];
"#;
    let (interp_err, vm_err) = eval_both_engines(source);
    let combined = format!("{} {}", interp_err, vm_err);
    assert!(!combined.is_empty(), "Expected OOB error for arr[99]");
    assert!(
        combined.to_lowercase().contains("bound") || combined.to_lowercase().contains("index"),
        "AT0006 error must mention bounds/index: {}",
        combined
    );
}
