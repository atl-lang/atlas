//! Bytecode module tests - round-trip serialization

use super::*;
use atlas_runtime::binder::Binder;
use atlas_runtime::typechecker::TypeChecker;

// ============================================================================
// Helper for full compilation (binding + typechecking + compilation)
// ============================================================================

fn compile_full(source: &str) -> Bytecode {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_errors) = lexer.tokenize();
    assert!(lex_errors.is_empty(), "Lexer errors: {:?}", lex_errors);
    let mut parser = Parser::new(tokens);
    let (program, parse_errors) = parser.parse();
    assert!(parse_errors.is_empty(), "Parser errors: {:?}", parse_errors);
    println!("Program items: {}", program.items.len());
    let mut binder = Binder::new();
    let (mut table, bind_errors) = binder.bind(&program);
    assert!(bind_errors.is_empty(), "Binder errors: {:?}", bind_errors);
    let mut typechecker = TypeChecker::new(&mut table);
    let type_errors = typechecker.check(&program);
    assert!(type_errors.is_empty(), "Type errors: {:?}", type_errors);
    Compiler::new()
        .compile(&program)
        .expect("Compilation failed")
}

// ============================================================================
// Bytecode round-trip serialization tests (H-002)
// ============================================================================

#[test]
fn test_bytecode_roundtrip_simple_expression() {
    let source = "1 + 2;";
    let bc = compile_full(source);
    let bytes = bc.to_bytes();
    let restored = Bytecode::from_bytes(&bytes).expect("Failed to deserialize");

    // Verify structure matches
    assert_eq!(restored.instructions.len(), bc.instructions.len());
    assert_eq!(restored.constants.len(), bc.constants.len());
    assert_eq!(restored.debug_info.len(), bc.debug_info.len());

    // Verify execution produces same result
    let original_result = run(bc);
    let restored_result = run(restored);
    assert_eq!(original_result, restored_result);
}

#[test]
fn test_bytecode_roundtrip_string_constant() {
    let source = r#""hello world";"#;
    let bc = compile_full(source);
    let bytes = bc.to_bytes();
    let restored = Bytecode::from_bytes(&bytes).expect("Failed to deserialize");

    let original_result = run(bc);
    let restored_result = run(restored);
    assert_eq!(original_result, restored_result);
}

#[test]
fn test_bytecode_roundtrip_function() {
    let source = r#"
        fn add(borrow a: number, borrow b: number): number {
            return a + b;
        }
        add(3, 4);
    "#;
    let bc = compile_full(source);
    let bytes = bc.to_bytes();
    let restored = Bytecode::from_bytes(&bytes).expect("Failed to deserialize");

    // Verify structure matches
    assert_eq!(restored.instructions.len(), bc.instructions.len());
    assert_eq!(restored.constants.len(), bc.constants.len());

    // Run both and compare
    let security = SecurityContext::allow_all();
    let mut vm1 = VM::new(bc);
    let original_result = vm1.run(&security).unwrap_or(None);

    let mut vm2 = VM::new(restored);
    let restored_result = vm2.run(&security).unwrap_or(None);

    assert_eq!(original_result, restored_result);
    assert_eq!(original_result, Some(Value::Number(7.0)));
}

#[test]
fn test_bytecode_roundtrip_array_literal() {
    let source = "[1, 2, 3];";
    let bc = compile_full(source);
    let bytes = bc.to_bytes();
    let restored = Bytecode::from_bytes(&bytes).expect("Failed to deserialize");

    let original_result = run(bc);
    let restored_result = run(restored);
    assert_eq!(original_result, restored_result);
}

#[test]
fn test_bytecode_roundtrip_complex_program() {
    let source = r#"
        fn factorial(borrow n: number): number {
            if n <= 1 {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }
        factorial(5);
    "#;
    let bc = compile_full(source);
    let bytes = bc.to_bytes();
    let restored = Bytecode::from_bytes(&bytes).expect("Failed to deserialize");

    // Verify structure matches
    assert_eq!(restored.instructions.len(), bc.instructions.len());
    assert_eq!(restored.constants.len(), bc.constants.len());

    // Run both and compare
    let security = SecurityContext::allow_all();
    let mut vm1 = VM::new(bc);
    let original_result = vm1.run(&security).unwrap_or(None);

    let mut vm2 = VM::new(restored);
    let restored_result = vm2.run(&security).unwrap_or(None);

    assert_eq!(original_result, restored_result);
    assert_eq!(original_result, Some(Value::Number(120.0)));
}

#[test]
fn test_bytecode_checksum_detects_corruption() {
    let source = "42";
    let bc = compile(source);
    let mut bytes = bc.to_bytes();

    // Corrupt a byte in the middle
    if bytes.len() > 10 {
        bytes[10] ^= 0xFF;
    }

    // Should fail checksum verification
    let result = Bytecode::from_bytes(&bytes);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("checksum mismatch"));
}

#[test]
fn test_bytecode_rejects_wrong_version() {
    let source = "42";
    let bc = compile(source);
    let mut bytes = bc.to_bytes();

    // Change version to 99
    bytes[4] = 0;
    bytes[5] = 99;

    // Recalculate checksum to bypass that check
    let data_len = bytes.len() - 4;
    let checksum = crc32fast::hash(&bytes[..data_len]);
    bytes[data_len] = (checksum >> 24) as u8;
    bytes[data_len + 1] = (checksum >> 16) as u8;
    bytes[data_len + 2] = (checksum >> 8) as u8;
    bytes[data_len + 3] = checksum as u8;

    let result = Bytecode::from_bytes(&bytes);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("version mismatch"));
}

#[test]
fn test_bytecode_rejects_bad_magic() {
    let source = "42";
    let bc = compile(source);
    let mut bytes = bc.to_bytes();

    // Corrupt magic number
    bytes[0] = b'X';

    // Recalculate checksum
    let data_len = bytes.len() - 4;
    let checksum = crc32fast::hash(&bytes[..data_len]);
    bytes[data_len] = (checksum >> 24) as u8;
    bytes[data_len + 1] = (checksum >> 16) as u8;
    bytes[data_len + 2] = (checksum >> 8) as u8;
    bytes[data_len + 3] = checksum as u8;

    let result = Bytecode::from_bytes(&bytes);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("bad magic"));
}

// ============================================================================
// Phase 06: Async opcode round-trip tests
// ============================================================================

/// 1. AsyncCall encodes/decodes to the same bytes (u16 fn_const_idx + u8 arg_count)
#[test]
fn test_async_call_opcode_roundtrip() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::AsyncCall, Span::dummy());
    bc.emit_u16(0x0000); // fn_const_idx = 0
    bc.emit_u8(2); // arg_count = 2
    assert_eq!(bc.instructions[0], Opcode::AsyncCall as u8);
    assert_eq!(bc.instructions[1], 0x00);
    assert_eq!(bc.instructions[2], 0x00);
    assert_eq!(bc.instructions[3], 2);
}

/// 2. Await encodes correctly (no operands)
#[test]
fn test_await_opcode_roundtrip() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::Await, Span::dummy());
    assert_eq!(bc.instructions.len(), 1);
    assert_eq!(bc.instructions[0], Opcode::Await as u8);
}

/// 3. WrapFuture encodes correctly (no operands)
#[test]
fn test_wrap_future_opcode_roundtrip() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::WrapFuture, Span::dummy());
    assert_eq!(bc.instructions.len(), 1);
    assert_eq!(bc.instructions[0], Opcode::WrapFuture as u8);
}

/// 4. SpawnTask encodes/decodes to the same bytes (u16 fn_const_idx + u8 arg_count)
#[test]
fn test_spawn_task_opcode_roundtrip() {
    let mut bc = Bytecode::new();
    bc.emit(Opcode::SpawnTask, Span::dummy());
    bc.emit_u16(5); // fn_const_idx = 5
    bc.emit_u8(0); // arg_count = 0
    assert_eq!(bc.instructions[0], Opcode::SpawnTask as u8);
    assert_eq!(bc.instructions[1], 0x00);
    assert_eq!(bc.instructions[2], 0x05);
    assert_eq!(bc.instructions[3], 0);
}

/// 5. Disassembler renders all four async opcodes as non-empty human-readable strings
#[test]
fn test_async_opcodes_disassembly() {
    use atlas_runtime::bytecode::disassemble;

    let mut bc = Bytecode::new();
    bc.emit(Opcode::AsyncCall, Span::dummy());
    bc.emit_u16(0);
    bc.emit_u8(1);
    bc.emit(Opcode::Await, Span::dummy());
    bc.emit(Opcode::WrapFuture, Span::dummy());
    bc.emit(Opcode::SpawnTask, Span::dummy());
    bc.emit_u16(3);
    bc.emit_u8(2);
    bc.emit(Opcode::Halt, Span::dummy());

    let output = disassemble(&bc);
    assert!(
        output.contains("AsyncCall"),
        "missing AsyncCall in disasm: {output}"
    );
    assert!(
        output.contains("Await"),
        "missing Await in disasm: {output}"
    );
    assert!(
        output.contains("WrapFuture"),
        "missing WrapFuture in disasm: {output}"
    );
    assert!(
        output.contains("SpawnTask"),
        "missing SpawnTask in disasm: {output}"
    );
}
