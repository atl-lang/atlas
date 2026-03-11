//! Native binary packaging — wraps Atlas bytecode in a self-contained OS executable.
//!
//! Multi-module archive format (trailer at end of file, D-048):
//!
//!   [launcher_binary]
//!   [module_count: u32 LE]
//!   for each module in dependency order:
//!     [bytecode_len: u32 LE]
//!     [bytecode_bytes]
//!   [ATLAS_BC_MAGIC: 16 bytes]  ← sentinel
//!   [payload_len: u64 LE]       ← byte count from module_count to before ATLAS_BC_MAGIC
//!
//! Modules are stored in build/dependency order: dependencies first, entry point last.
//! The launcher executes them in order on a single VM so globals accumulate correctly.
//!
//! CRITICAL: ATLAS_BC_MAGIC must be byte-for-byte identical to the constant in
//! crates/atlas-launcher/src/self_path.rs. Any divergence means the launcher
//! will never find its payload. (D-048)

use crate::error::{BuildError, BuildResult};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Unique 16-byte sentinel separating the launcher binary from appended bytecode.
///
/// Must match `atlas_launcher::self_path::ATLAS_BC_MAGIC` exactly.
pub const ATLAS_BC_MAGIC: [u8; 16] = [
    0xA7, 0x7C, 0x41, 0x53, // ·|AS
    0x42, 0x43, 0x21, 0xFE, // BC!·
    0xDE, 0xAD, 0xBE, 0xEF, // dead beef
    0x19, 0x20, 0x21, 0x22, // version marker
];

/// Locate the `atlas-launcher` binary on the current machine.
///
/// Search order:
///   1. Same directory as the currently running `atlas` binary
///   2. Directories in the PATH environment variable
///
/// Returns None if the launcher is not found anywhere.
pub fn find_launcher_binary() -> Option<PathBuf> {
    let launcher_name = if cfg!(windows) {
        "atlas-launcher.exe"
    } else {
        "atlas-launcher"
    };

    // 1. Same directory as current exe (most reliable after `cargo install`)
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let candidate = dir.join(launcher_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    // 2. Search PATH
    if let Ok(path_var) = std::env::var("PATH") {
        let separator = if cfg!(windows) { ';' } else { ':' };
        for dir in path_var.split(separator) {
            let candidate = Path::new(dir).join(launcher_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Package a native binary: copy the launcher, append the multi-module archive, and
/// write the ATLAS_BC_MAGIC trailer.
///
/// `module_bytecodes` must be in dependency order: dependencies first, entry point last.
/// Sets executable permissions on Unix (mode 0o755).
pub fn emit_native_binary(
    launcher_path: &Path,
    module_bytecodes: &[Vec<u8>],
    output_path: &Path,
) -> BuildResult<()> {
    // Copy the launcher binary to the output path
    std::fs::copy(launcher_path, output_path).map_err(|e| BuildError::io(output_path, e))?;

    // Build the payload in memory so we can compute its length for the trailer
    let mut payload: Vec<u8> = Vec::new();

    // module_count (u32 LE)
    let module_count = module_bytecodes.len() as u32;
    payload.extend_from_slice(&module_count.to_le_bytes());

    // for each module: [bytecode_len u32 LE][bytecode_bytes]
    for bc in module_bytecodes {
        let bc_len = bc.len() as u32;
        payload.extend_from_slice(&bc_len.to_le_bytes());
        payload.extend_from_slice(bc);
    }

    let payload_len = payload.len() as u64;

    // Append payload + ATLAS_BC_MAGIC + payload_len (trailer)
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(output_path)
        .map_err(|e| BuildError::io(output_path, e))?;

    file.write_all(&payload)
        .map_err(|e| BuildError::io(output_path, e))?;

    file.write_all(&ATLAS_BC_MAGIC)
        .map_err(|e| BuildError::io(output_path, e))?;

    file.write_all(&payload_len.to_le_bytes())
        .map_err(|e| BuildError::io(output_path, e))?;

    drop(file);

    // Set executable bit on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(output_path)
            .map_err(|e| BuildError::io(output_path, e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(output_path, perms).map_err(|e| BuildError::io(output_path, e))?;
    }

    Ok(())
}
