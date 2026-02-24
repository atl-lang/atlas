//! Low-level opcode and bytecode construction tests
//!
//! Tests direct VM behaviour via manually constructed Bytecode —
//! constants pool bounds, stack frames, call mechanics, truncated bytecode.
//! Migrated from src/vm/mod.rs inline tests.

use super::*;
use atlas_runtime::bytecode::{Bytecode, Opcode};
use atlas_runtime::value::{FunctionRef, RuntimeError};
use atlas_runtime::Span;
use pretty_assertions::assert_eq;

// ============================================================================
// Constants pool
// ============================================================================

#[test]
fn test_vm_load_number_constant() {
    assert_eq!(vm_eval("123.456;"), Some(Value::Number(123.456)));
}

#[test]
fn test_vm_load_string_constant() {
    let result = vm_eval("\"hello world\";");
    if let Some(Value::String(s)) = result {
        assert_eq!(s.as_ref(), "hello world");
    } else {
        panic!("Expected string value");
    }
}

#[test]
fn test_vm_load_multiple_constants() {
    assert_eq!(vm_eval("1; 2; 3;"), Some(Value::Number(3.0)));
}

#[test]
fn test_vm_constants_in_expression() {
    assert_eq!(vm_eval("10 + 20 + 30;"), Some(Value::Number(60.0)));
}

#[test]
fn test_vm_constant_reuse() {
    assert_eq!(
        vm_eval("let x = 5; let y = 5; x + y;"),
        Some(Value::Number(10.0))
    );
}

#[test]
fn test_vm_large_constant_index() {
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!("let x{} = {}; ", i, i));
    }
    source.push_str("x99;");
    assert_eq!(vm_eval(&source), Some(Value::Number(99.0)));
}

#[test]
fn test_vm_string_constants_in_variables() {
    let result = vm_eval("let s = \"test\"; s;");
    if let Some(Value::String(s)) = result {
        assert_eq!(s.as_ref(), "test");
    } else {
        panic!("Expected string value");
    }
}

#[test]
fn test_vm_mixed_constant_types() {
    assert_eq!(
        vm_eval("let n = 42; let s = \"hello\"; let b = true; n;"),
        Some(Value::Number(42.0))
    );
}

#[test]
fn test_vm_constant_bounds_check() {
    let mut bytecode = Bytecode::new();
    bytecode.add_constant(Value::Number(1.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(999); // index out of bounds
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err());
}

#[test]
fn test_vm_empty_constant_pool() {
    let mut bytecode = Bytecode::new();
    bytecode.emit(Opcode::True, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Bool(true)));
    // Constants pool stays empty when no Constant opcodes are emitted
}

// ============================================================================
// Stack frames
// ============================================================================

// Note: test_vm_initial_main_frame was removed — it tested private VM struct
// fields (frames, stack_base) which are implementation details not accessible
// from external test binaries.

#[test]
fn test_vm_frame_relative_locals() {
    let mut bytecode = Bytecode::new();
    let idx_10 = bytecode.add_constant(Value::Number(10.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx_10);
    let idx_20 = bytecode.add_constant(Value::Number(20.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx_20);
    bytecode.emit(Opcode::GetLocal, Span::dummy());
    bytecode.emit_u16(0);
    bytecode.emit(Opcode::GetLocal, Span::dummy());
    bytecode.emit_u16(1);
    bytecode.emit(Opcode::Add, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Number(30.0)));
}

#[test]
fn test_vm_return_from_main() {
    let mut bytecode = Bytecode::new();
    let idx = bytecode.add_constant(Value::Number(42.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx);
    bytecode.emit(Opcode::Return, Span::dummy());
    bytecode.emit(Opcode::Null, Span::dummy());
    bytecode.emit(Opcode::Halt, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Number(42.0)));
}

// ============================================================================
// Call frames
// ============================================================================

#[test]
fn test_vm_call_frame_creation() {
    let mut bytecode = Bytecode::new();
    let function_offset = 10;
    let func_ref = FunctionRef {
        name: "test_func".to_string(),
        arity: 0,
        bytecode_offset: function_offset,
        local_count: 1,
        param_ownership: vec![],
        param_names: vec![],
        return_ownership: None,
    };
    let func_idx = bytecode.add_constant(Value::Function(func_ref));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(func_idx);
    bytecode.emit(Opcode::Call, Span::dummy());
    bytecode.emit_u8(0);
    bytecode.emit(Opcode::Halt, Span::dummy());
    while bytecode.instructions.len() < function_offset {
        bytecode.emit_u8(0);
    }
    let idx_42 = bytecode.add_constant(Value::Number(42.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx_42);
    bytecode.emit(Opcode::Return, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Number(42.0)));
}

#[test]
fn test_vm_call_with_arguments() {
    let mut bytecode = Bytecode::new();
    let function_offset = 20;
    let func_ref = FunctionRef {
        name: "add".to_string(),
        arity: 2,
        bytecode_offset: function_offset,
        local_count: 1,
        param_ownership: vec![],
        param_names: vec![],
        return_ownership: None,
    };
    let func_idx = bytecode.add_constant(Value::Function(func_ref));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(func_idx);
    let idx_5 = bytecode.add_constant(Value::Number(5.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx_5);
    let idx_3 = bytecode.add_constant(Value::Number(3.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx_3);
    bytecode.emit(Opcode::Call, Span::dummy());
    bytecode.emit_u8(2);
    bytecode.emit(Opcode::Halt, Span::dummy());
    while bytecode.instructions.len() < function_offset {
        bytecode.emit_u8(0);
    }
    bytecode.emit(Opcode::GetLocal, Span::dummy());
    bytecode.emit_u16(0);
    bytecode.emit(Opcode::GetLocal, Span::dummy());
    bytecode.emit_u16(1);
    bytecode.emit(Opcode::Add, Span::dummy());
    bytecode.emit(Opcode::Return, Span::dummy());

    let result = VM::new(bytecode)
        .run(&SecurityContext::allow_all())
        .unwrap();
    assert_eq!(result, Some(Value::Number(8.0)));
}

#[test]
fn test_vm_call_wrong_arity() {
    let mut bytecode = Bytecode::new();
    let func_ref = FunctionRef {
        name: "test".to_string(),
        arity: 2,
        bytecode_offset: 10,
        local_count: 2,
        param_ownership: vec![],
        param_names: vec![],
        return_ownership: None,
    };
    let func_idx = bytecode.add_constant(Value::Function(func_ref));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(func_idx);
    bytecode.emit(Opcode::Null, Span::dummy());
    bytecode.emit(Opcode::Call, Span::dummy());
    bytecode.emit_u8(1);

    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::TypeError { msg, .. } => assert!(msg.contains("expects 2 arguments")),
        _ => panic!("Expected TypeError"),
    }
}

#[test]
fn test_vm_call_non_function() {
    let mut bytecode = Bytecode::new();
    let idx = bytecode.add_constant(Value::Number(42.0));
    bytecode.emit(Opcode::Constant, Span::dummy());
    bytecode.emit_u16(idx);
    bytecode.emit(Opcode::Call, Span::dummy());
    bytecode.emit_u8(0);

    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::TypeError { msg, .. } => assert!(msg.contains("Cannot call non-function")),
        _ => panic!("Expected TypeError"),
    }
}

// ============================================================================
// Truncated bytecode — must produce clean errors, not UB
// ============================================================================

#[test]
fn test_truncated_bytecode_load_const_missing_operand() {
    let mut bytecode = Bytecode::new();
    bytecode.instructions = vec![Opcode::Constant as u8, 0x00]; // missing second byte
    bytecode.constants.push(Value::Number(42.0));
    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err(), "Truncated bytecode should produce error");
}

#[test]
fn test_truncated_bytecode_jump_missing_operand() {
    let mut bytecode = Bytecode::new();
    bytecode.instructions = vec![Opcode::Jump as u8, 0x00]; // missing second byte
    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err(), "Truncated jump should produce error");
}

#[test]
fn test_truncated_bytecode_empty() {
    let bytecode = Bytecode::new();
    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    // Should either Ok(None) or clean error — not UB
    let _ = result;
}

#[test]
fn test_truncated_bytecode_call_missing_arg_count() {
    let mut bytecode = Bytecode::new();
    bytecode.instructions = vec![Opcode::Call as u8]; // missing arg count byte
    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err(), "Truncated call should produce error");
}

#[test]
fn test_truncated_bytecode_get_local_missing_operand() {
    let mut bytecode = Bytecode::new();
    bytecode.instructions = vec![Opcode::GetLocal as u8, 0x00]; // missing second byte
    let result = VM::new(bytecode).run(&SecurityContext::allow_all());
    assert!(result.is_err(), "Truncated GetLocal should produce error");
}
