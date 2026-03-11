//! Self-path detection and appended bytecode parsing.
//!
//! When `atlas build` produces a native binary it writes:
//!   [launcher_binary][ATLAS_BC_MAGIC 16 bytes][bytecode_len u64 LE][bytecode]
//!
//! At startup the launcher reads itself from disk, scans the end of the file
//! for ATLAS_BC_MAGIC, and extracts the embedded bytecode.

use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

/// Unique 16-byte sentinel that separates the launcher binary from appended bytecode.
///
/// CRITICAL: This constant must be byte-for-byte identical in:
///   - crates/atlas-launcher/src/self_path.rs  (here)
///   - crates/atlas-build/src/binary_emit.rs   (P04)
///
/// The bytes spell "ATLAS_BC" + 0x21 0xFE followed by random uniqueness bytes.
pub const ATLAS_BC_MAGIC: [u8; 16] = [
    0xA7, 0x7C, 0x41, 0x53, // ·|AS
    0x42, 0x43, 0x21, 0xFE, // BC!·
    0xDE, 0xAD, 0xBE, 0xEF, // dead beef
    0x19, 0x20, 0x21, 0x22, // version marker
];

/// Number of bytes at the end of the file to scan for the magic sentinel.
/// Large enough for any reasonable OS page + padding, small enough to be fast.
const SCAN_WINDOW: usize = 4096;

/// Returns the path to the currently running executable.
pub fn current_exe_path() -> io::Result<PathBuf> {
    std::env::current_exe()
}

/// Scan the end of the file at `exe_path` for ATLAS_BC_MAGIC.
/// If found, extract and return the appended bytecode bytes.
/// Returns None if the sentinel is not present (binary has no embedded program).
pub fn find_appended_bytecode(exe_path: &std::path::Path) -> io::Result<Option<Vec<u8>>> {
    let mut file = std::fs::File::open(exe_path)?;
    let file_len = file.seek(SeekFrom::End(0))?;

    // Need at least magic(16) + len(8) + 1 byte of bytecode
    if file_len < (ATLAS_BC_MAGIC.len() + 8 + 1) as u64 {
        return Ok(None);
    }

    // Read the scan window from the end of the file
    let scan_start = file_len.saturating_sub(SCAN_WINDOW as u64);
    file.seek(SeekFrom::Start(scan_start))?;

    let mut window = Vec::with_capacity((file_len - scan_start) as usize);
    file.read_to_end(&mut window)?;

    // Search backwards through the window for the magic sentinel
    let magic_len = ATLAS_BC_MAGIC.len();
    if window.len() < magic_len + 8 {
        return Ok(None);
    }

    // Scan from end towards start — the sentinel is near the end
    let search_end = window.len().saturating_sub(magic_len + 8);
    for i in (0..=search_end).rev() {
        if window[i..i + magic_len] == ATLAS_BC_MAGIC {
            // Found sentinel at window offset i
            // Next 8 bytes = bytecode length (u64 LE)
            let len_bytes: [u8; 8] = window[i + magic_len..i + magic_len + 8]
                .try_into()
                .expect("slice is exactly 8 bytes");
            let bytecode_len = u64::from_le_bytes(len_bytes) as usize;

            let payload_start = i + magic_len + 8;
            if payload_start + bytecode_len > window.len() {
                // Bytecode extends before the scan window — re-read from absolute offset
                let abs_offset = scan_start + payload_start as u64;
                file.seek(SeekFrom::Start(abs_offset))?;
                let mut bytecode = vec![0u8; bytecode_len];
                file.read_exact(&mut bytecode)?;
                return Ok(Some(bytecode));
            }

            let bytecode = window[payload_start..payload_start + bytecode_len].to_vec();
            return Ok(Some(bytecode));
        }
    }

    Ok(None)
}
