use super::super::*;
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
