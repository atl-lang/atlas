//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to submodule files: tests/api/{core,conversions,native,reflection,...}.rs
//! This file only declares submodules and shared helpers.

use atlas_runtime::api::{
    ConversionError, EvalError, ExecutionMode, FromAtlas, Runtime, RuntimeConfig, ToAtlas,
};
use atlas_runtime::reflect::{get_value_type_info, TypeInfo, TypeKind, ValueInfo};
use atlas_runtime::span::Span;
use atlas_runtime::types::Type;
use atlas_runtime::value::RuntimeError;
use atlas_runtime::{Atlas, DiagnosticLevel, JsonValue, RuntimeResult, Value};
use rstest::rstest;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

// ============================================================================
// Submodules
// ============================================================================

#[path = "api/core_part1.rs"]
mod core_part1;

#[path = "api/core_part2.rs"]
mod core_part2;

#[path = "api/conversions_part1.rs"]
mod conversions_part1;

#[path = "api/conversions_part2.rs"]
mod conversions_part2;

#[path = "api/native_part1.rs"]
mod native_part1;

#[path = "api/native_part2.rs"]
mod native_part2;

#[path = "api/sandboxing.rs"]
mod sandboxing;

#[path = "api/reflection_part1.rs"]
mod reflection_part1;

#[path = "api/reflection_part2.rs"]
mod reflection_part2;

#[path = "api/json.rs"]
mod json;

#[path = "api/runtime_api.rs"]
mod runtime_api;
