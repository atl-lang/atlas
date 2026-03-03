use atlas_runtime::repl::ReplCore;
use atlas_runtime::Value;

pub(super) fn eval_ok(repl: &mut ReplCore, input: &str) -> Value {
    let result = repl.eval_line(input);
    if !result.diagnostics.is_empty() {
        panic!(
            "Expected success for '{}'\nGot: {:?}",
            input, result.diagnostics
        );
    }
    result.value.unwrap_or(Value::Null)
}

pub(super) fn eval_err(repl: &mut ReplCore, input: &str) {
    let result = repl.eval_line(input);
    if result.diagnostics.is_empty() {
        panic!("Expected error for '{}', but succeeded", input);
    }
}

pub(super) fn assert_value(repl: &mut ReplCore, expr: &str, expected: Value) {
    let value = eval_ok(repl, expr);
    assert_eq!(value, expected, "Expression '{}' failed", expr);
}
