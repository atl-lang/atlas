//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to submodule files: tests/stdlib/real_world/{csv,json_api,log_analysis,pipelines,text,config}.rs
//!
//! Real-World Standard Library Integration Tests
//!
//! This test suite demonstrates practical, real-world usage patterns of the Atlas
//! standard library. Tests read like actual programs users would write:
//! - CSV processing
//! - JSON API handling
//! - Log file analysis
//! - Data transformation pipelines
//! - Text processing
//! - Configuration file processing
//!
//! ALL tests verify interpreter/VM parity (100% identical output).

// Subdomain modules (150 tests total)
mod csv_part1; // 15 tests - CSV processing (basic operations)
mod csv_part2; // 15 tests - CSV processing (advanced operations)
mod json_api; // 30 tests - JSON API response handling
mod log_analysis_part1; // 15 tests - Log file analysis (basic patterns)
mod log_analysis_part2; // 15 tests - Log file analysis (advanced aggregation)
mod pipelines; // 30 tests - Data transformation pipelines
mod text; // 20 tests - Text processing
mod config; // 10 tests - Configuration file processing
