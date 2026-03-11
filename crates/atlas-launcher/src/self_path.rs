//! Self-path detection and multi-module bytecode extraction.
//!
//! Multi-module archive format (trailer at end — O(1) detection, D-048):
//!   [launcher_binary]
//!   [module_count: u32 LE]
//!   for each module: [bytecode_len: u32 LE][bytecode_bytes]
//!   [ATLAS_BC_MAGIC: 16 bytes]
//!   [payload_len: u64 LE]      ← byte count from module_count to before ATLAS_BC_MAGIC
//!
//! The launcher executes each module in order on a single VM so globals accumulate.

use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

/// Unique 16-byte sentinel marking the start of the trailer.
///
/// CRITICAL: This constant must be byte-for-byte identical in:
///   - crates/atlas-launcher/src/self_path.rs  (here)
///   - crates/atlas-build/src/binary_emit.rs
pub const ATLAS_BC_MAGIC: [u8; 16] = [
    0xA7, 0x7C, 0x41, 0x53, // ·|AS
    0x42, 0x43, 0x21, 0xFE, // BC!·
    0xDE, 0xAD, 0xBE, 0xEF, // dead beef
    0x19, 0x20, 0x21, 0x22, // version marker
];

/// Total size of the trailer (ATLAS_BC_MAGIC + payload_len u64).
const TRAILER_LEN: u64 = 16 + 8;

/// Returns the path to the currently running executable.
pub fn current_exe_path() -> io::Result<PathBuf> {
    std::env::current_exe()
}

/// Read the trailer from the last 24 bytes and extract all embedded module bytecodes.
///
/// Returns the bytecodes in dependency order (dependencies first, entry point last).
/// Returns None if the file has no valid Atlas trailer.
pub fn find_appended_bytecode(exe_path: &std::path::Path) -> io::Result<Option<Vec<Vec<u8>>>> {
    let mut file = std::fs::File::open(exe_path)?;
    let file_len = file.seek(SeekFrom::End(0))?;

    // Minimum: launcher(1) + module_count(4) + one module header(4) + one bytecode(1) + trailer(24)
    if file_len < TRAILER_LEN + 10 {
        return Ok(None);
    }

    // Read the last 24 bytes — the trailer
    file.seek(SeekFrom::End(-(TRAILER_LEN as i64)))?;
    let mut trailer = [0u8; 24];
    file.read_exact(&mut trailer)?;

    // First 16 bytes must be ATLAS_BC_MAGIC
    if trailer[..16] != ATLAS_BC_MAGIC {
        return Ok(None);
    }

    // Last 8 bytes = payload_len (u64 LE)
    let payload_len = u64::from_le_bytes(
        trailer[16..24]
            .try_into()
            .expect("slice is exactly 8 bytes"),
    ) as usize;

    // Read the full payload
    let payload_offset = file_len - TRAILER_LEN - payload_len as u64;
    file.seek(SeekFrom::Start(payload_offset))?;
    let mut payload = vec![0u8; payload_len];
    file.read_exact(&mut payload)?;

    // Parse payload: [module_count u32 LE] + for each: [bytecode_len u32 LE][bytecode_bytes]
    if payload.len() < 4 {
        return Ok(None);
    }
    let module_count = u32::from_le_bytes(payload[..4].try_into().expect("4 bytes")) as usize;
    let mut pos = 4;
    let mut modules = Vec::with_capacity(module_count);

    for _ in 0..module_count {
        if pos + 4 > payload.len() {
            return Ok(None);
        }
        let bc_len =
            u32::from_le_bytes(payload[pos..pos + 4].try_into().expect("4 bytes")) as usize;
        pos += 4;

        if pos + bc_len > payload.len() {
            return Ok(None);
        }
        modules.push(payload[pos..pos + bc_len].to_vec());
        pos += bc_len;
    }

    Ok(Some(modules))
}
