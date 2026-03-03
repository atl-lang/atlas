use super::{vm_eval, Value};
use pretty_assertions::assert_eq;

#[test]
fn test_vm_for_loop() {
    assert_eq!(
        vm_eval("let mut sum = 0; for i in [0, 1, 2, 3, 4] { sum = sum + i; } sum;"),
        Some(Value::Number(10.0))
    );
}
