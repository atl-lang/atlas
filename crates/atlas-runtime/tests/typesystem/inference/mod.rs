//! Type inference test suite router
//!
//! Subdirectory split from monolithic inference.rs (1959 lines / 52KB → 8 files under 10KB each)
//!
//! ## Structure
//! - `helpers.rs` — shared test helpers and parity functions
//! - `advanced_part1.rs` — bidirectional, higher-rank, let-polymorphism, flow-sensitive tests
//! - `advanced_part2.rs` — unification, constraint solving, cross-module, heuristics tests
//! - `integration.rs` — integration tests (complex programs, real-world scenarios)
//! - `return_types_part1.rs` — return type inference tests sections 1-9
//! - `return_types_part2.rs` — return type inference tests sections 10-11 + Block 5 Phase 3-4
//! - `error_messages.rs` — inference error message tests (Block 5 Phase 6)
//! - `parity_suite.rs` — comprehensive parity test suite (Block 5 Phase 7)

mod helpers;

mod advanced_part1;
mod advanced_part2;
mod error_messages;
mod integration;
mod parity_suite;
mod return_types_part1;
mod return_types_part2;
