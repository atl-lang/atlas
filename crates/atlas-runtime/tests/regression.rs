//! Comprehensive Regression Test Suite
//!
//! This file serves as the golden test matrix for Atlas language features.
//! It provides quick regression detection by testing all core features in one place.
//!
//! Test coverage:
//! - Literals (number, string, bool, null, arrays)
//! - Operators (arithmetic, comparison, logical)
//! - Variables (let/var, mutation, scoping)
//! - Functions (declarations, calls, returns, recursion)
//! - Control flow (if/else, while, for, break, continue)
//! - Arrays (indexing, mutation, nested arrays)
//! - Type checking (type errors, type inference)
//! - Error handling (runtime errors, compile errors)
//! - Standard library functions

mod common;

// Regression test suite split into multiple parts to keep files under 12KB.
#[path = "regression/part1.rs"]
mod part1;
#[path = "regression/part10.rs"]
mod part10;
#[path = "regression/part11.rs"]
mod part11;
#[path = "regression/part12.rs"]
mod part12;
#[path = "regression/part13.rs"]
mod part13;
#[path = "regression/part2.rs"]
mod part2;
#[path = "regression/part3.rs"]
mod part3;
#[path = "regression/part4.rs"]
mod part4;
#[path = "regression/part5.rs"]
mod part5;
#[path = "regression/part6.rs"]
mod part6;
#[path = "regression/part7.rs"]
mod part7;
#[path = "regression/part8.rs"]
mod part8;
#[path = "regression/part9.rs"]
mod part9;
