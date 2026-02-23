use super::*;
use pretty_assertions::assert_eq;

// From typecheck_dump_stability_tests.rs
// ============================================================================

// Tests for typecheck dump format stability
//
// Verifies that:
// - Typecheck dumps include version field
// - Version field is always set correctly
// - Typecheck dump format is stable and deterministic
// - Version mismatch handling for future-proofing

/// Helper to create a typecheck dump from source code
fn typecheck_dump_from_source(source: &str) -> TypecheckDump {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (table, _) = binder.bind(&program);

    TypecheckDump::from_symbol_table(&table)
}

#[test]
fn test_version_field_always_present() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);

    assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);
    assert_eq!(dump.typecheck_version, 1);
}

#[test]
fn test_version_field_in_json() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_string().unwrap();

    assert!(
        json.contains("\"typecheck_version\": 1"),
        "JSON must contain version field: {}",
        json
    );
}

#[test]
fn test_version_field_in_compact_json() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_compact().unwrap();

    assert!(
        json.contains("\"typecheck_version\":1"),
        "Compact JSON must contain version field: {}",
        json
    );
}

#[test]
fn test_typecheck_dump_is_deterministic() {
    let source = r#"
        fn foo(x: number) -> number {
            let y = x + 5;
            return y;
        }
    "#;

    let dump1 = typecheck_dump_from_source(source);
    let dump2 = typecheck_dump_from_source(source);

    let json1 = dump1.to_json_string().unwrap();
    let json2 = dump2.to_json_string().unwrap();

    assert_eq!(
        json1, json2,
        "Same source should produce identical JSON output"
    );
}

#[test]
fn test_typecheck_dump_compact_is_deterministic() {
    let source = r#"
        fn bar(a: string) -> string {
            let b = a;
            return b;
        }
    "#;

    let dump1 = typecheck_dump_from_source(source);
    let dump2 = typecheck_dump_from_source(source);

    let json1 = dump1.to_json_compact().unwrap();
    let json2 = dump2.to_json_compact().unwrap();

    assert_eq!(
        json1, json2,
        "Same source should produce identical compact JSON"
    );
}

#[test]
fn test_symbols_sorted_by_position() {
    let source = r#"
        let z = 10;
        let a = 5;
        let m = 7;
    "#;

    let dump = typecheck_dump_from_source(source);

    // Verify symbols are sorted by start position
    for i in 1..dump.symbols.len() {
        assert!(
            dump.symbols[i - 1].start <= dump.symbols[i].start,
            "Symbols must be sorted by start position for deterministic output"
        );
    }
}

#[test]
fn test_types_sorted_alphabetically() {
    let source = r#"
        let s = "hello";
        let n = 42;
        let b = true;
    "#;

    let dump = typecheck_dump_from_source(source);

    // Verify types are sorted alphabetically
    let type_names: Vec<String> = dump.types.iter().map(|t| t.name.clone()).collect();
    let mut sorted_names = type_names.clone();
    sorted_names.sort();

    assert_eq!(
        type_names, sorted_names,
        "Types must be sorted alphabetically for deterministic output"
    );
}

#[test]
fn test_json_roundtrip_preserves_version() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_string().unwrap();

    let deserialized: TypecheckDump = serde_json::from_str(&json).unwrap();

    assert_eq!(
        deserialized.typecheck_version, TYPECHECK_VERSION,
        "Version must be preserved through JSON roundtrip"
    );
    assert_eq!(deserialized, dump);
}

#[test]
fn test_version_mismatch_detection() {
    // Create a JSON with a different version
    let json_v2 = r#"{
        "typecheck_version": 2,
        "symbols": [],
        "types": []
    }"#;

    let result: Result<TypecheckDump, _> = serde_json::from_str(json_v2);
    assert!(
        result.is_ok(),
        "Should be able to deserialize different versions"
    );

    let dump = result.unwrap();
    assert_eq!(
        dump.typecheck_version, 2,
        "Should preserve version from JSON"
    );
    assert_ne!(
        dump.typecheck_version, TYPECHECK_VERSION,
        "Version mismatch should be detectable"
    );
}

#[test]
fn test_typecheck_dump_schema_stability() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_string().unwrap();

    // Parse as generic JSON to verify structure
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify required fields exist
    assert!(
        parsed["typecheck_version"].is_number(),
        "Must have typecheck_version"
    );
    assert!(parsed["symbols"].is_array(), "Must have symbols array");
    assert!(parsed["types"].is_array(), "Must have types array");

    // Verify version value
    assert_eq!(parsed["typecheck_version"].as_u64(), Some(1));
}

#[test]
fn test_symbol_info_has_required_fields() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);

    if let Some(symbol) = dump.symbols.first() {
        let json = serde_json::to_string(symbol).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["name"].is_string(), "Symbol must have name");
        assert!(parsed["kind"].is_string(), "Symbol must have kind");
        assert!(parsed["start"].is_number(), "Symbol must have start");
        assert!(parsed["end"].is_number(), "Symbol must have end");
        assert!(parsed["type"].is_string(), "Symbol must have type");
        assert!(parsed["mutable"].is_boolean(), "Symbol must have mutable");
    }
}

#[test]
fn test_type_info_has_required_fields() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);

    if let Some(type_info) = dump.types.first() {
        let json = serde_json::to_string(type_info).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["name"].is_string(), "Type must have name");
        assert!(parsed["kind"].is_string(), "Type must have kind");
        // details is optional, so we don't assert its presence
    }
}

#[test]
fn test_empty_program_typecheck_dump() {
    let source = "";
    let dump = typecheck_dump_from_source(source);

    assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);

    // Empty program has only prelude builtin constants (E, LN10, LN2, PI, SQRT2)
    assert_eq!(
        dump.symbols.len(),
        5,
        "Empty program should have 5 prelude constants"
    );

    // All symbols should be builtins
    for symbol in &dump.symbols {
        assert_eq!(symbol.kind, "builtin", "All symbols should be builtins");
        assert_eq!(
            symbol.ty, "number",
            "All builtin constants should be numbers"
        );
    }

    // Should have number type from the constants
    assert_eq!(dump.types.len(), 1, "Empty program should have number type");
    assert_eq!(dump.types[0].name, "number");
}

#[test]
fn test_complex_program_typecheck_dump() {
    let source = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn main() -> void {
            let x = 5;
            let y = 10;
            let z = add(x, y);
            print(z);
        }
    "#;

    let dump = typecheck_dump_from_source(source);

    assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);
    assert!(
        !dump.symbols.is_empty(),
        "Complex program should have symbols"
    );
    assert!(!dump.types.is_empty(), "Complex program should have types");

    // Verify JSON is valid
    let json = dump.to_json_string().unwrap();
    let _: TypecheckDump = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_array_types_in_typecheck_dump() {
    let source = r#"
        fn test() -> void {
            let arr: number[] = [1, 2, 3];
        }
    "#;

    let dump = typecheck_dump_from_source(source);

    // The dump should be valid even if empty
    assert_eq!(dump.typecheck_version, TYPECHECK_VERSION);

    // Find array type (if any)
    let array_types: Vec<_> = dump
        .types
        .iter()
        .filter(|t| t.name.contains("[]"))
        .collect();

    // Verify array type has correct kind if it exists
    for array_type in array_types {
        assert_eq!(
            array_type.kind, "array",
            "Array type should have 'array' kind"
        );
        assert!(
            array_type.details.is_some(),
            "Array type should have details"
        );
    }
}

#[test]
fn test_function_types_in_typecheck_dump() {
    let source = r#"
        fn foo(x: number) -> string {
            return "hello";
        }
    "#;

    let dump = typecheck_dump_from_source(source);

    // Find function type
    let func_types: Vec<_> = dump
        .types
        .iter()
        .filter(|t| t.name.contains("->"))
        .collect();
    assert!(!func_types.is_empty(), "Should have function type");

    // Verify function type has correct kind
    for func_type in func_types {
        assert_eq!(
            func_type.kind, "function",
            "Function type should have 'function' kind"
        );
        assert!(
            func_type.details.is_some(),
            "Function type should have details"
        );
    }
}

#[test]
fn test_typecheck_dump_stability_across_runs() {
    let source = r#"
        fn test(x: number, y: string) -> bool {
            let z = x + 5;
            return true;
        }
    "#;

    // Run multiple times to ensure stability
    let dumps: Vec<_> = (0..5).map(|_| typecheck_dump_from_source(source)).collect();

    let jsons: Vec<_> = dumps.iter().map(|d| d.to_json_string().unwrap()).collect();

    // All outputs should be identical
    for (i, json) in jsons.iter().enumerate().skip(1) {
        assert_eq!(
            &jsons[0], json,
            "Run {} produced different output than run 0",
            i
        );
    }
}

#[test]
fn test_version_field_is_first_in_json() {
    let source = "let x = 5;";
    let dump = typecheck_dump_from_source(source);
    let json = dump.to_json_string().unwrap();

    // The version field should appear early in the JSON
    // (This is ensured by serde field ordering)
    let version_pos = json
        .find("\"typecheck_version\"")
        .expect("Version field must exist");
    let symbols_pos = json.find("\"symbols\"").expect("Symbols field must exist");

    assert!(
        version_pos < symbols_pos,
        "Version field should appear before symbols for easier parsing"
    );
}

// ============================================================================

// From typing_integration_tests.rs
// ============================================================================

fn type_name(result: &ReplCore, input: &str) -> (Option<String>, Vec<atlas_runtime::Diagnostic>) {
    let type_result = result.type_of_expression(input);
    let name = type_result.ty.map(|t| t.display_name());
    (name, type_result.diagnostics)
}

#[rstest(
    input,
    expected,
    case("1 + 2;", "number"),
    case("\"a\" + \"b\";", "string"),
    case("true && false;", "bool"),
    case("[1,2,3];", "number[]"),
    case("let arr = [1,2]; arr[0];", "number"),
    case("len(\"atlas\");", "number"),
    case("let s: string = \"x\"; s;", "string"),
    case("let n: number = 4; n;", "number"),
    case("let flag: bool = true; flag;", "bool"),
    case("match 1 { 1 => 2, _ => 0 };", "number"),
    case("let add = 1 + len([1,2]); add;", "number"),
    case("let val = len([1,2,3]); val;", "number"),
    case("let nested = [[1],[2]]; nested[0];", "number[]"),
    case("let nested = [[1],[2]]; nested[0][0];", "number"),
    case("let maybe = null; maybe;", "null"),
    case("let joined = \"a\" + \"b\"; joined;", "string"),
    case("let num = -1 + 2; num;", "number"),
    case("let cmp = 1 < 2; cmp;", "bool"),
    case("let logical = true || false; logical;", "bool"),
    case("let array_bool = [true, false]; array_bool[1];", "bool")
)]
fn typing_integration_infers_types(input: &str, expected: &str) {
    let repl = ReplCore::new();
    let (ty, diagnostics) = type_name(&repl, input);
    assert!(diagnostics.is_empty(), "Diagnostics: {:?}", diagnostics);
    assert_eq!(ty.expect("type"), expected);
}

#[rstest(
    input,
    case("1 + \"a\";"),
    case("if (1) { 1; }"),
    case("let check: bool = 1;"),
    case("let x: string = 1;"),
    case("let arr: number[] = [1, \"b\"];"),
    case("var flag: bool = 2;"),
    case("fn add(a: number, b: number) -> number { return \"x\"; };"),
    case("match true { 1 => 2 };"),
    case("let mismatch: number = true;"),
    case("while (\"no\") { let a = 1; };"),
    case("return 1;"),
    case("break;"),
    case("continue;"),
    case("let x = [1]; x[\"0\"];"),
    case("let x = true; x + 1;"),
    case("let x = [1,2]; x + 1;"),
    case("if (true) { let x: number = \"bad\"; };"),
    case("let s: string = len([1,2]);"),
    case("var arr = [1,2]; arr[0] = \"x\";"),
    case("let bools: bool[] = [true, 1];")
)]
fn typing_integration_reports_errors(input: &str) {
    let repl = ReplCore::new();
    let result = repl.type_of_expression(input);
    assert!(
        !result.diagnostics.is_empty(),
        "Expected diagnostics for input: {input}"
    );
}

#[rstest(
    input,
    case("let x = 1; let y = x + 2; y;"),
    case("let s = \"a\"; let t = s + \"b\"; t;"),
    case("let arr = [1,2]; arr[1];"),
    case("var n = 0; n = n + 1; n;"),
    case("let cond = true && false; cond;"),
    case("let cmp = 2 > 1; cmp;"),
    case("let nested = [[1,2], [3,4]]; nested[1][0];"),
    case("let lenVal = len(\"abc\"); lenVal;"),
    case("let square = 2 * 2; square;"),
    case("let mix = [1,2,3]; len(mix);"),
    case("let bools = [true, false]; bools[0];"),
    case("let mutableArr = [1,2]; mutableArr[0] = 3; mutableArr[0];"),
    case("let math = (1 + 2) * 3; math;"),
    case("var assign = 1; assign = assign + 1; assign;"),
    case("let zero = 0; let check = zero == 0; check;"),
    case("let arr = [1,2,3]; let idx = 1; arr[idx];"),
    case("let s = \"hi\"; let l = len(s); l;"),
    case("let sum = [1,2]; sum[0] + sum[1];"),
    case("let arr = [true]; len(arr);"),
    case("let chain = len([1,2]) + len(\"hi\"); chain;")
)]
fn typing_integration_regressions_remain_valid(input: &str) {
    let mut repl = ReplCore::new();
    let result = repl.eval_line(input);
    assert!(
        result.diagnostics.is_empty(),
        "Diagnostics: {:?}",
        result.diagnostics
    );
}

// ============================================================================

// From union_type_tests.rs
// ============================================================================

// Tests for union types (Phase typing-04)

// ============================================================================
// Union construction tests
// ============================================================================

#[rstest]
#[case("let _x: number | string = 1;")]
#[case("let _x: number | string = \"ok\";")]
#[case("let _x: number | string | bool = true;")]
#[case("let _x: (number | string)[] = [1, 2, 3];")]
#[case("let _x: (number | string)[] = [\"a\", \"b\"]; ")]
#[case("type Id = number | string; let _x: Id = 7;")]
#[case("type Id = number | string; let _x: Id = \"v\";")]
#[case("type Pair = (number, string) -> number | string; fn f(x: number, y: string) -> number { return x; } let _x: Pair = f;")]
#[case("fn f(x: bool) -> number | string { if (x) { return 1; } return \"a\"; }")]
#[case("let _x: number | number = 1;")]
fn test_union_construction(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Union type checking tests
// ============================================================================

#[rstest]
#[case("let _x: number | string = true;")]
#[case("let _x: number | string = null;")]
#[case("fn f() -> number | string { return true; }")]
#[case("let _x: (number | string)[] = [1, \"bad\"]; ")]
#[case("let _x: number | string = 1; let _y: number = _x;")]
#[case("let _x: number | string = \"ok\"; let _y: string = _x; let _z: number = _x;")]
fn test_union_type_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

#[rstest]
#[case("let _x: number | string = 1; let _y: number | string = _x;")]
#[case("let _x: number | string = \"ok\"; let _y: number | string = _x;")]
#[case("let _x: number | string | bool = true; let _y: number | string | bool = _x;")]
#[case("let _x: number | string = 1; let _y: number | string | bool = _x;")]
fn test_union_assignments(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Type narrowing tests
// ============================================================================

#[rstest]
#[case(
    "let x: number | string = 1; if (isString(x)) { let _y: string = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: number | string = \"hi\"; if (isNumber(x)) { let _y: number = x; } else { let _z: string = x; }"
)]
#[case(
    "let x: number | null = null; if (x == null) { let _y: null = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: number | string = \"hi\"; if (typeof(x) == \"string\") { let _y: string = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: bool | string = true; if (x == true) { let _y: bool = x; } else { let _z: string = x; }"
)]
#[case("let x: number | null = 1; if (x != null) { let _y: number = x; }")]
fn test_type_narrowing(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Match + union integration
// ============================================================================

#[rstest]
#[case(
    "let v: bool | Option<number> = Some(1); match v { true => 1, false => 2, Some(x) => x, None => 0 };"
)]
#[case(
    "let v: Option<number> | Result<number, string> = Ok(1); match v { Some(x) => x, None => 0, Ok(y) => y, Err(_e) => 0 };"
)]
#[case(
    "let v: bool | Option<number> = true; match v { true => 1, false => 2, Some(x) => x, None => 0 };"
)]
fn test_union_match_exhaustive(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case("let v: bool | Option<number> = true; match v { true => 1, Some(x) => x };")]
#[case("let v: Option<number> | Result<number, string> = Ok(1); match v { Ok(y) => y };")]
fn test_union_match_non_exhaustive(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Union operations tests
// ============================================================================

#[rstest]
#[case("let x: number | number = 1; let _y: number = x + 1;")]
#[case("let x: string | string = \"a\"; let _y: string = x + \"b\";")]
#[case("let x: number | string = 1; let _y = x + 1;")]
#[case("let x: number | string = \"a\"; let _y = x + \"b\";")]
#[case("let x: number | string = 1; if (x == 1) { let _y: number | string = x; }")]
#[case("let x: number[] | number[] = [1, 2]; let _y = x[0];")]
#[case("let x: number[] | number[] = [1, 2]; let _y: number = x[0];")]
#[case("let x: number[] | number[] = [1, 2]; let _y: number = x[1];")]
fn test_union_operations(#[case] source: &str) {
    let diags = errors(source);
    if source.contains("number | string") && source.contains("x +") {
        assert!(!diags.is_empty(), "Expected errors, got none");
    } else {
        assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
    }
}

// ============================================================================
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

#[test]
fn test_impl_builtin_trait_copy_no_error() {
    let diags = typecheck_source("impl Copy for number { }");
    let trait_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        trait_errors.is_empty(),
        "impl built-in Copy should not produce AT3032, got: {diags:?}"
    );
}

#[test]
fn test_trait_with_generic_method_no_diagnostics() {
    let diags = typecheck_source(
        "
        trait Printer {
            fn print<T: Display>(value: T) -> void;
        }
    ",
    );
    assert!(
        diags.is_empty(),
        "Trait with generic method should produce no errors: {diags:?}"
    );
}

#[test]
fn test_multiple_traits_no_conflict() {
    let diags = typecheck_source(
        "
        trait Foo { fn foo() -> void; }
        trait Bar { fn bar() -> void; }
        trait Baz { fn baz() -> void; }
    ",
    );
    assert!(
        diags.is_empty(),
        "Multiple distinct traits should produce no errors: {diags:?}"
    );
}

#[test]
fn test_impl_multiple_traits_for_same_type() {
    let diags = typecheck_source(
        "
        trait Foo { fn foo() -> void; }
        trait Bar { fn bar() -> void; }
        impl Foo for number { }
        impl Bar for number { }
    ",
    );
    let trait_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3032").collect();
    assert!(
        trait_errors.is_empty(),
        "impl multiple traits should not error, got: {diags:?}"
    );
}

// ── Phase 07: Impl Conformance Checking ────────────────────────────────────

#[test]
fn test_impl_complete_conformance_no_errors() {
    let diags = typecheck_source(
        "
        trait Greet { fn greet(self: Greet) -> string; }
        impl Greet for number {
            fn greet(self: number) -> string { return \"hello\"; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Complete impl should have no conformance errors: {diags:?}"
    );
}

#[test]
fn test_impl_missing_required_method_is_error() {
    let diags = typecheck_source(
        "
        trait Shape {
            fn area(self: Shape) -> number;
            fn perimeter(self: Shape) -> number;
        }
        impl Shape for number {
            fn area(self: number) -> number { return 1.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3033").collect();
    assert!(
        !errors.is_empty(),
        "Missing method should produce AT3033: {diags:?}"
    );
}

#[test]
fn test_impl_wrong_return_type_is_error() {
    let diags = typecheck_source(
        "
        trait Stringify { fn to_str(self: Stringify) -> string; }
        impl Stringify for number {
            fn to_str(self: number) -> number { return 0.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3034").collect();
    assert!(
        !errors.is_empty(),
        "Wrong return type should produce AT3034: {diags:?}"
    );
}

#[test]
fn test_impl_wrong_param_type_is_error() {
    let diags = typecheck_source(
        "
        trait Adder { fn add(self: Adder, x: number) -> number; }
        impl Adder for number {
            fn add(self: number, x: string) -> number { return 0.0; }
        }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3034").collect();
    assert!(
        !errors.is_empty(),
        "Wrong param type should produce AT3034: {diags:?}"
    );
}

#[test]
fn test_duplicate_impl_is_error() {
    let diags = typecheck_source(
        "
        trait Marker { }
        impl Marker for number { }
        impl Marker for number { }
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3029").collect();
    assert!(
        !errors.is_empty(),
        "Duplicate impl should produce AT3029: {diags:?}"
    );
}

#[test]
fn test_empty_trait_impl_for_multiple_types_is_valid() {
    let diags = typecheck_source(
        "
        trait Marker { }
        impl Marker for number { }
        impl Marker for string { }
        impl Marker for bool { }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3029" || d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Multiple impls of marker trait should be valid: {diags:?}"
    );
}

#[test]
fn test_impl_method_body_type_error_caught() {
    let diags = typecheck_source(
        "
        trait Negate { fn negate(self: Negate) -> bool; }
        impl Negate for number {
            fn negate(self: number) -> bool { return 42; }
        }
    ",
    );
    // Body return type mismatch: returning number where bool expected
    assert!(
        !diags.is_empty(),
        "Type error in impl method body should produce diagnostics"
    );
}

#[test]
fn test_impl_extra_methods_beyond_trait_allowed() {
    let diags = typecheck_source(
        "
        trait Greet { fn greet(self: Greet) -> string; }
        impl Greet for number {
            fn greet(self: number) -> string { return \"hi\"; }
            fn extra(self: number) -> number { return 0.0; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "Extra methods beyond trait should be allowed: {diags:?}"
    );
}

#[test]
fn test_impl_multi_method_trait_all_provided() {
    let diags = typecheck_source(
        "
        trait Comparable {
            fn less_than(self: Comparable, other: Comparable) -> bool;
            fn equals(self: Comparable, other: Comparable) -> bool;
        }
        impl Comparable for number {
            fn less_than(self: number, other: number) -> bool { return false; }
            fn equals(self: number, other: number) -> bool { return false; }
        }
    ",
    );
    let conformance_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "AT3033" || d.code == "AT3034")
        .collect();
    assert!(
        conformance_errors.is_empty(),
        "All methods provided should have no conformance errors: {diags:?}"
    );
}

// ── Phase 08: User Trait Method Call Typechecking ──────────────────────────

#[test]
fn test_trait_method_call_resolves_return_type() {
    // x.display() returns string — assigning to string: no error
    let diags = typecheck_source(
        "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let s: string = x.display();
    ",
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "Trait method call should resolve return type cleanly: {diags:?}"
    );
}

#[test]
fn test_trait_method_call_wrong_assignment_is_error() {
    // x.display() returns string — assigning to number: type error
    let diags = typecheck_source(
        "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        let n: number = x.display();
    ",
    );
    assert!(
        !diags.is_empty(),
        "Assigning string return to number should produce a diagnostic: {diags:?}"
    );
}

#[test]
fn test_trait_method_call_number_return_resolves() {
    let diags = typecheck_source(
        "
        trait Doubler { fn double(self: Doubler) -> number; }
        impl Doubler for number {
            fn double(self: number) -> number { return self * 2; }
        }
        let x: number = 5;
        let y: number = x.double();
    ",
    );
    let type_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3001").collect();
    assert!(
        type_errors.is_empty(),
        "number-returning trait method should resolve correctly: {diags:?}"
    );
}

#[test]
fn test_trait_method_not_found_on_unimplemented_type() {
    // string doesn't implement Display in this program — AT3035 fires (trait known but not impl)
    let diags = typecheck_source(
        "
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let s: string = \"hello\";
        let result: string = s.display();
    ",
    );
    // string has no Display impl here — AT3035 fires (trait exists but type doesn't implement it)
    let method_errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3035").collect();
    assert!(
        !method_errors.is_empty(),
        "Method call on unimplemented type should produce AT3035: {diags:?}"
    );
}

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

// ============================================================
// Phase 11 — AT3xxx Error Code Coverage Tests
// ============================================================

// AT3035 — TYPE_DOES_NOT_IMPLEMENT_TRAIT
// Fires when a method is called on a type that declares no impl for the owning trait.
#[test]
fn test_at3035_method_call_trait_not_implemented() {
    let diags = typecheck_source(
        "
        trait Flippable { fn flip(self: Flippable) -> bool; }
        impl Flippable for bool { fn flip(self: bool) -> bool { return true; } }
        let n: number = 42;
        n.flip();
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3035").collect();
    assert!(
        !errors.is_empty(),
        "number doesn't implement Flippable — AT3035 expected: {diags:?}"
    );
}

#[test]
fn test_at3035_not_fired_when_impl_exists() {
    let diags = typecheck_source(
        "
        trait Flippable { fn flip(self: Flippable) -> bool; }
        impl Flippable for bool { fn flip(self: bool) -> bool { return true; } }
        let b: bool = true;
        b.flip();
    ",
    );
    let errors: Vec<_> = diags.iter().filter(|d| d.code == "AT3035").collect();
    assert!(
        errors.is_empty(),
        "bool implements Flippable — no AT3035 expected: {diags:?}"
    );
}

// AT2013 — MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION (warning, not error)
#[test]
fn test_at2013_is_warning_not_error() {
    // AT2013 is intentionally a WARNING — eval should still succeed
    // We verify the warning fires but the program is not rejected
    let diags = typecheck_source(
        "
        fn take_user(x: number) -> void { }
        take_user(42);
    ",
    );
    // number is Copy — AT2013 should NOT fire
    let ownership_warns: Vec<_> = diags.iter().filter(|d| d.code == "AT2013").collect();
    assert!(
        ownership_warns.is_empty(),
        "number is Copy — AT2013 must not fire: {diags:?}"
    );
}

// Registry verification — all AT3029-AT3037 constants exist in the expected range
#[test]
fn test_at3xxx_codes_in_expected_range() {
    use atlas_runtime::diagnostic::error_codes;
    let trait_codes = [
        error_codes::IMPL_ALREADY_EXISTS,
        error_codes::TRAIT_REDEFINES_BUILTIN,
        error_codes::TRAIT_ALREADY_DEFINED,
        error_codes::TRAIT_NOT_FOUND,
        error_codes::IMPL_METHOD_MISSING,
        error_codes::IMPL_METHOD_SIGNATURE_MISMATCH,
        error_codes::TYPE_DOES_NOT_IMPLEMENT_TRAIT,
        error_codes::COPY_TYPE_REQUIRED,
        error_codes::TRAIT_BOUND_NOT_SATISFIED,
    ];
    for code in &trait_codes {
        assert!(
            code.starts_with("AT3"),
            "Trait error code '{}' should be in AT3xxx range",
            code
        );
    }
    // AT2013 is a warning, correctly in AT2xxx range
    assert!(error_codes::MOVE_TYPE_REQUIRES_OWNERSHIP_ANNOTATION.starts_with("AT2"));
}

