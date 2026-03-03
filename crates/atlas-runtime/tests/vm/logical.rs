//! Tests for logical And/Or opcodes (H-014 fix verification)
//!
//! These opcodes provide non-short-circuit evaluation for pre-evaluated operands.
//! Short-circuit semantics are handled by compiler via JumpIfFalse.

use super::*;
use atlas_runtime::bytecode::{Bytecode, Opcode};
use atlas_runtime::security::SecurityContext;
use atlas_runtime::value::{RuntimeError, Value};
use atlas_runtime::vm::VM;
use atlas_runtime::Span;
use pretty_assertions::assert_eq;
use std::sync::Arc;

// ============================================================================
// Direct opcode tests (hand-constructed bytecode)
// ============================================================================

#[test]
fn test_opcode_and_true_true() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::And, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(true)));
}

#[test]
fn test_opcode_and_true_false() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::False, Span::dummy());
    bytecode.emit(Opcode::And, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(false)));
}

#[test]
fn test_opcode_and_false_true() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::False, Span::dummy());
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::And, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(false)));
}

#[test]
fn test_opcode_and_false_false() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::False, Span::dummy());
    bytecode.emit(Opcode::False, Span::dummy());
    bytecode.emit(Opcode::And, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(false)));
}

#[test]
fn test_opcode_or_true_true() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::Or, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(true)));
}

#[test]
fn test_opcode_or_true_false() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::False, Span::dummy());
    bytecode.emit(Opcode::Or, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(true)));
}

#[test]
fn test_opcode_or_false_true() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::False, Span::dummy());
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::Or, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(true)));
}

#[test]
fn test_opcode_or_false_false() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::False, Span::dummy());
    bytecode.emit(Opcode::False, Span::dummy());
    bytecode.emit(Opcode::Or, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(false)));
}

// ============================================================================
// Type error tests
// ============================================================================

#[test]
fn test_opcode_and_type_error_number() {
    let mut bytecode = Bytecode::new();
    let idx = bytecode.add_constant(Value::Number(42.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx);
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::And, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::TypeError { msg, .. } => {
            assert!(msg.contains("&&"), "Error should mention &&: {}", msg);
        }
        other => panic!("Expected TypeError, got {:?}", other),
    }
}

#[test]
fn test_opcode_or_type_error_string() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::False, Span::dummy());
    let idx = bytecode.add_constant(Value::String(Arc::new("hello".to_string())));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx);
    bytecode.emit(Opcode::Or, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::TypeError { msg, .. } => {
            assert!(msg.contains("||"), "Error should mention ||: {}", msg);
        }
        other => panic!("Expected TypeError, got {:?}", other),
    }
}

// ============================================================================
// High-level language tests (compiler uses JumpIfFalse for short-circuit)
// ============================================================================

#[test]
fn test_and_short_circuit_parity() {
    assert_parity("true && true;");
    assert_parity("true && false;");
    assert_parity("false && true;");
    assert_parity("false && false;");
}

#[test]
fn test_or_short_circuit_parity() {
    assert_parity("true || true;");
    assert_parity("true || false;");
    assert_parity("false || true;");
    assert_parity("false || false;");
}

#[test]
fn test_complex_logical_parity() {
    assert_parity("(true && false) || true;");
    assert_parity("true && (false || true);");
    assert_parity("!false && true;");
    assert_parity("!(true && false);");
}
