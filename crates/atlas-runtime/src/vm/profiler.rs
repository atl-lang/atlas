//! VM profiler — delegates to crate::profiler
//!
//! Re-exports the comprehensive `Profiler` from `crate::profiler` so that
//! `vm::Profiler` continues to work for all existing callers.

pub use crate::profiler::Profiler;
