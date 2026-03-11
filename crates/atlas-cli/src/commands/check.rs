//! check command — DEPRECATED
//!
//! `atlas check` only analyzed a single file in isolation and missed cross-module
//! errors, making it misleading. Use `atlas run` instead.
//!
//! The command is kept registered in the CLI so users get a clear deprecation
//! message rather than "unknown command". The dispatch in main.rs prints the
//! message and exits 1 — this module is no longer called.
