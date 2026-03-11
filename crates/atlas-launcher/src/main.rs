//! Atlas native binary launcher
//!
//! This binary is embedded into Atlas executables produced by `atlas build`.
//! At runtime it reads bytecode appended to itself (after a magic sentinel),
//! then executes it using the Atlas VM.
//!
//! See B19 for the full design: self-appending launcher pattern.

fn main() {
    eprintln!("atlas-launcher: not yet implemented");
    std::process::exit(1);
}
