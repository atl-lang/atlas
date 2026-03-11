//! Bytecode execution via the Atlas VM.

use atlas_runtime::{Bytecode, SecurityContext, VM};

/// Deserialize and execute Atlas bytecode.
///
/// Returns the exit code: 0 on success, 1 on any runtime or deserialization error.
pub fn run_bytecode(bytecode_bytes: &[u8]) -> i32 {
    let bytecode = match Bytecode::from_bytes(bytecode_bytes) {
        Ok(bc) => bc,
        Err(e) => {
            eprintln!("atlas: failed to load embedded program: {e}");
            return 1;
        }
    };

    let security = SecurityContext::allow_all();
    let mut vm = VM::new(bytecode);

    match vm.run(&security) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("atlas: runtime error: {e}");
            1
        }
    }
}
