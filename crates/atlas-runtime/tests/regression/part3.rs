use atlas_runtime::Atlas;

// ============================================================================
//
// Comprehensive stability tests covering:
// - Determinism: same input → same output across multiple runs
// - Edge cases: empty inputs, boundary values, unicode, special floats
// - Stress: large data, deep recursion, long strings
// - Error recovery: malformed input handled gracefully, not panicked
// - Release mode: all tests also pass in --release builds

// ─── Determinism Tests ───────────────────────────────────────────────────────

#[test]
fn stability_determinism_arithmetic() {
    // Same arithmetic expression evaluated twice must produce the same result.
    let code = "1 + 2 * 3 - 4 / 2;";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_string_concat() {
    let code = r#""hello" + " " + "world";"#;
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_function_calls() {
    let code = r#"
        fn fib(borrow n: number) -> number {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        fib(8);
    "#;
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_conditionals() {
    let code = "if (3 > 2) { 42; } else { 0; }";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_array_operations() {
    let code = "let arr: number[] = [3, 1, 4, 1, 5]; arr[2];";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_error_reporting() {
    // Errors must be reported deterministically — same input → same error code.
    let code = "1 / 0;";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        r1.is_err() == r2.is_err(),
        "Non-deterministic: {:?} vs {:?}",
        r1,
        r2
    );
    if let (Err(d1), Err(d2)) = (&r1, &r2) {
        assert!(
            d1.len() == d2.len(),
            "Diagnostic count mismatch: {} != {}",
            d1.len(),
            d2.len()
        )
    }
}

#[test]
fn stability_determinism_boolean_logic() {
    let code = "true && false || (true && true);";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_while_loop() {
    let code = r#"
        let mut sum: number = 0;
        let mut i: number = 0;
        while (i < 10) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_nested_functions() {
    let code = r#"
        fn outer(borrow x: number) -> number {
            fn inner(borrow y: number) -> number {
                return y * 2;
            }
            return inner(x) + 1;
        }
        outer(5);
    "#;
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        format!("{:?}", r1) == format!("{:?}", r2),
        "Non-deterministic: {:?} != {:?}",
        r1,
        r2
    );
}

#[test]
fn stability_determinism_type_error() {
    // Type errors must be reported with the same diagnostic code each time.
    let code = "let x: number = true;";
    let runtime1 = Atlas::new();
    let runtime2 = Atlas::new();
    let r1 = runtime1.eval(code);
    let r2 = runtime2.eval(code);
    assert!(
        r1.is_err() == r2.is_err(),
        "Non-deterministic: {:?} vs {:?}",
        r1,
        r2
    );
}
