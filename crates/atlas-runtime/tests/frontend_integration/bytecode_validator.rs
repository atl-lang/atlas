//! Bytecode validator tests (lines 2070-2418)

use super::*;

// Bytecode Validator Tests (from bytecode_validator_tests.rs)
// ============================================================================

// ============================================================================
// Helpers
// ============================================================================

fn span() -> Span {
    Span::dummy()
}

fn num_const(bc: &mut Bytecode, n: f64) -> u16 {
    bc.add_constant(Value::Number(n))
}

fn str_const(bc: &mut Bytecode, s: &str) -> u16 {
    bc.add_constant(Value::string(s))
}

fn push_num(bc: &mut Bytecode, n: f64) {
    let idx = num_const(bc, n);
    bc.emit(Opcode::Constant, span());
    bc.emit_u16(idx);
}

// ============================================================================
// 1. Valid compiler-like programs pass validation
// ============================================================================

#[test]
fn test_validate_simple_expression() {
    // "1 + 2" → Constant(1), Constant(2), Add, Halt
    let mut bc = Bytecode::new();
    push_num(&mut bc, 1.0);
    push_num(&mut bc, 2.0);
    bc.emit(Opcode::Add, span());
    bc.emit(Opcode::Halt, span());
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_boolean_expression() {
    // "true && false" → True, Dup, JumpIfFalse(1), Pop, False, Halt
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, span()); // 0
    bc.emit(Opcode::Dup, span()); // 1
    bc.emit(Opcode::JumpIfFalse, span()); // 2: operand at 3-4
    bc.emit_i16(2); // jump forward 2 bytes; target = 5 + 2 = 7 (Halt)
    bc.emit(Opcode::Pop, span()); // 5
    bc.emit(Opcode::False, span()); // 6
    bc.emit(Opcode::Halt, span()); // 7
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_negation() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, span());
    bc.emit(Opcode::Not, span());
    bc.emit(Opcode::Halt, span());
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_null() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Null, span());
    bc.emit(Opcode::Halt, span());
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_set_and_get_global() {
    let mut bc = Bytecode::new();
    let name = str_const(&mut bc, "myVar");
    push_num(&mut bc, 42.0);
    bc.emit(Opcode::SetGlobal, span());
    bc.emit_u16(name);
    bc.emit(Opcode::GetGlobal, span());
    bc.emit_u16(name);
    bc.emit(Opcode::Pop, span());
    bc.emit(Opcode::Halt, span());
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_local_variable() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, span());
    bc.emit(Opcode::SetLocal, span());
    bc.emit_u16(0);
    bc.emit(Opcode::GetLocal, span());
    bc.emit_u16(0);
    bc.emit(Opcode::Halt, span());
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_array_creation() {
    let mut bc = Bytecode::new();
    push_num(&mut bc, 1.0);
    push_num(&mut bc, 2.0);
    push_num(&mut bc, 3.0);
    bc.emit(Opcode::Array, span());
    bc.emit_u16(3);
    bc.emit(Opcode::Halt, span());
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_get_index() {
    let mut bc = Bytecode::new();
    push_num(&mut bc, 10.0);
    bc.emit(Opcode::Array, span());
    bc.emit_u16(1);
    push_num(&mut bc, 0.0);
    bc.emit(Opcode::GetIndex, span());
    bc.emit(Opcode::Halt, span());
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_all_arithmetic_ops() {
    for op in [
        Opcode::Add,
        Opcode::Sub,
        Opcode::Mul,
        Opcode::Div,
        Opcode::Mod,
    ] {
        let mut bc = Bytecode::new();
        push_num(&mut bc, 10.0);
        push_num(&mut bc, 2.0);
        bc.emit(op, span());
        bc.emit(Opcode::Halt, span());
        assert!(validate(&bc).is_ok(), "arithmetic op {:?} failed", op);
    }
}

#[test]
fn test_validate_simple_if_pattern() {
    // if (true) { 1 } else { 2 }
    // True, JumpIfFalse -> else, Const(1), Jump -> end, Const(2), Halt
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, span()); // 0
    bc.emit(Opcode::JumpIfFalse, span()); // 1, op at 2-3, next=4
                                          // Jump forward to else branch (Const(2) at offset 10)
                                          // target = 4 + 6 = 10
    bc.emit_i16(6);
    let i1 = num_const(&mut bc, 1.0);
    bc.emit(Opcode::Constant, span()); // 4
    bc.emit_u16(i1); // 5-6
    bc.emit(Opcode::Jump, span()); // 7, op at 8-9, next=10
    bc.emit_i16(3); // target = 10 + 3 = 13 (Halt)
    let i2 = num_const(&mut bc, 2.0);
    bc.emit(Opcode::Constant, span()); // 10
    bc.emit_u16(i2); // 11-12
    bc.emit(Opcode::Halt, span()); // 13
    assert!(validate(&bc).is_ok());
}

#[test]
fn test_validate_return_in_function() {
    let mut bc = Bytecode::new();
    push_num(&mut bc, 99.0);
    bc.emit(Opcode::Return, span());
    assert!(validate(&bc).is_ok());
}

// ============================================================================
// 2. Error cases — invalid bytecode is detected
// ============================================================================

#[test]
fn test_invalid_opcode_0x00() {
    let mut bc = Bytecode::new();
    bc.instructions.push(0x00); // not a valid opcode
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, ValidationErrorKind::UnknownOpcode(0x00))),
        "expected UnknownOpcode(0x00), got: {:?}",
        errors
    );
}

#[test]
fn test_jump_to_negative_address() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Jump, span()); // at 0, op at 1-2, next=3
    bc.emit_i16(-100); // target = 3 + (-100) = -97 → out of bounds
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ValidationErrorKind::JumpOutOfBounds { .. })));
}

#[test]
fn test_loop_backward_out_of_bounds() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Loop, span()); // at 0
    bc.emit_i16(i16::MIN); // extremely far backward
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ValidationErrorKind::JumpOutOfBounds { .. })));
}

#[test]
fn test_constant_index_zero_empty_pool() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Constant, span());
    bc.emit_u16(0); // pool is empty
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        ValidationErrorKind::ConstantIndexOutOfBounds {
            index: 0,
            pool_size: 0
        }
    )));
}

#[test]
fn test_constant_index_exceeds_pool() {
    let mut bc = Bytecode::new();
    bc.add_constant(Value::Number(1.0)); // pool size = 1
    bc.emit(Opcode::Constant, span());
    bc.emit_u16(5); // index 5 > pool size 1
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        ValidationErrorKind::ConstantIndexOutOfBounds { index: 5, .. }
    )));
}

#[test]
fn test_stack_underflow_on_empty_comparison() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Equal, span()); // pops 2, stack empty
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ValidationErrorKind::StackUnderflow { .. })));
}

#[test]
fn test_stack_underflow_on_negate_after_pop() {
    // Push then pop → depth 0, then JumpIfFalse (pops condition from depth 0)
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, span()); // depth 1
    bc.emit(Opcode::Pop, span()); // depth 0
    bc.emit(Opcode::JumpIfFalse, span()); // pops from empty → underflow
    bc.emit_i16(0);
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ValidationErrorKind::StackUnderflow { .. })));
}

#[test]
fn test_missing_terminator_only_push() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, span());
    bc.emit(Opcode::False, span());
    // No Halt or Return
    let errors = validate(&bc).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ValidationErrorKind::MissingTerminator)));
}

#[test]
fn test_truncated_array_operand() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Array, span());
    bc.instructions.push(0x00); // only 1 byte; needs 2
    let errors = validate(&bc).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ValidationErrorKind::TruncatedInstruction { .. })));
}

#[test]
fn test_truncated_get_local_operand() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::GetLocal, span());
    bc.instructions.push(0x00); // only 1 byte; needs 2
    let errors = validate(&bc).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, ValidationErrorKind::TruncatedInstruction { .. })));
}

// ============================================================================
// 3. Error struct properties
// ============================================================================

#[test]
fn test_error_has_correct_offset() {
    // Unknown opcode at offset 3 (after True, True, True)
    let mut bc = Bytecode::new();
    bc.emit(Opcode::True, span()); // offset 0
    bc.emit(Opcode::True, span()); // offset 1
    bc.emit(Opcode::True, span()); // offset 2
    bc.instructions.push(0xFF - 1); // unknown at offset 3 (0xFE is not Halt=0xFF)
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    let bad = errors
        .iter()
        .find(|e| matches!(e.kind, ValidationErrorKind::UnknownOpcode(_)));
    assert!(bad.is_some(), "expected UnknownOpcode error");
    assert_eq!(bad.unwrap().offset, 3);
}

#[test]
fn test_errors_are_ordered_by_discovery() {
    // First error at offset 0 (unknown), second at later offset
    let mut bc = Bytecode::new();
    bc.instructions.push(0xDD); // offset 0: unknown
    bc.instructions.push(0xEE); // offset 1: unknown
    bc.emit(Opcode::Halt, span());
    let errors = validate(&bc).unwrap_err();
    assert!(errors.len() >= 2);
    // First reported error should be at offset 0
    assert_eq!(errors[0].offset, 0);
}

#[test]
fn test_valid_bytecode_has_no_errors() {
    let mut bc = Bytecode::new();
    push_num(&mut bc, std::f64::consts::PI);
    push_num(&mut bc, 2.0);
    bc.emit(Opcode::Mul, span());
    bc.emit(Opcode::Pop, span());
    bc.emit(Opcode::Halt, span());
    let result = validate(&bc);
    assert!(result.is_ok(), "valid bytecode should have no errors");
}

// ============================================================================
