//! Public embedding API for Atlas runtime
//!
//! This module provides a comprehensive API for embedding Atlas in Rust applications.
//! It includes:
//! - Runtime execution via Compiler+VM (D-052)
//! - Value conversion between Rust and Atlas types
//! - Native function registration
//! - Function calling and global variable management
//! - Comprehensive error handling
//!
//! # Examples
//!
//! ```rust,no_run
//! use atlas_runtime::api::Runtime;
//! use atlas_runtime::value::Value;
//!
//! // Create a runtime
//! let mut runtime = Runtime::new();
//!
//! // Evaluate code
//! let result = runtime.eval("1 + 2").expect("eval failed");
//!
//! // Call Atlas functions from Rust
//! runtime.eval("fn add(x: number, y: number): number { return x + y; }").expect("eval failed");
//! let result = runtime.call("add", vec![Value::Number(1.0), Value::Number(2.0)]).expect("call failed");
//! ```

pub mod config;
pub mod conversion;
pub mod native;
pub mod runtime;

// Re-export main types for convenience
pub use config::{ExecutionLimits, RuntimeConfig};
pub use conversion::{ConversionError, FromAtlas, ToAtlas};
pub use native::{BuildError, NativeFunctionBuilder};
pub use runtime::{EvalError, Runtime};
