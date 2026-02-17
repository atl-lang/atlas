//! Compression utilities
//!
//! Provides gzip compression/decompression and tar archive management.

pub mod gzip;
pub mod tar;

// Re-export main functions
pub use gzip::{compress_bytes, compress_string, decompress_bytes, decompress_string, is_gzip};
