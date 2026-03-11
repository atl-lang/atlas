//! Bytecode execution via the Atlas VM.
//!
//! Runs multiple module bytecodes in dependency order on a single VM so that
//! globals defined by dependency modules are visible to the entry point.

use atlas_runtime::{Bytecode, SecurityContext, VM};

/// Deserialize and execute all module bytecodes in order.
///
/// `module_bytecodes` must be in dependency order: dependencies first, entry point last.
/// Returns exit code: 0 on success, 1 on any error.
pub fn run_bytecodes(module_bytecodes: &[Vec<u8>]) -> i32 {
    if module_bytecodes.is_empty() {
        eprintln!("atlas: embedded program contains no modules");
        return 1;
    }

    let security = SecurityContext::allow_all();

    // Deserialize the first module and create the VM
    let first_bc = match Bytecode::from_bytes(&module_bytecodes[0]) {
        Ok(bc) => bc,
        Err(e) => {
            eprintln!("atlas: failed to load embedded program (module 0): {e}");
            return 1;
        }
    };
    let mut vm = VM::new(first_bc);
    if let Err(e) = vm.run(&security) {
        eprintln!("atlas: runtime error in module 0: {e}");
        return 1;
    }

    // Load and run each subsequent module on the SAME VM (globals are preserved)
    for (i, bc_bytes) in module_bytecodes[1..].iter().enumerate() {
        let bc = match Bytecode::from_bytes(bc_bytes) {
            Ok(bc) => bc,
            Err(e) => {
                eprintln!(
                    "atlas: failed to load embedded program (module {}): {e}",
                    i + 1
                );
                return 1;
            }
        };
        vm.load_module(bc);
        if let Err(e) = vm.run(&security) {
            eprintln!("atlas: runtime error in module {}: {e}", i + 1);
            return 1;
        }
    }

    0
}
