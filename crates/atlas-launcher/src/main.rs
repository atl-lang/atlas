//! Atlas native binary launcher
//!
//! This binary is embedded into Atlas executables produced by `atlas build`.
//! At runtime it reads bytecode appended to itself (after a magic sentinel),
//! then executes it using the Atlas VM.
//!
//! See B19 for the full design: self-appending launcher pattern.

mod execute;

fn main() {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("atlas: failed to determine executable path: {e}");
            std::process::exit(1);
        }
    };

    let bytecode = match atlas_launcher::self_path::find_appended_bytecode(&exe_path) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => {
            eprintln!(
                "atlas: this binary has no embedded Atlas program. It may be corrupted.\n\
                 Rebuild with: atlas build"
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("atlas: failed to read embedded program: {e}");
            std::process::exit(1);
        }
    };

    let exit_code = execute::run_bytecode(&bytecode);
    std::process::exit(exit_code);
}
