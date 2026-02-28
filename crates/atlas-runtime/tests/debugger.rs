//! THIN ROUTER — DO NOT ADD TESTS HERE.
//! Add tests to the submodule files: tests/debugger/{breakpoints,session,inspector,protocol_part1,protocol_part2,interpreter}.rs
//! This file only declares submodules and shared helpers.

use atlas_runtime::bytecode::{Bytecode, DebugSpan};
use atlas_runtime::compiler::Compiler;
use atlas_runtime::debugger::breakpoints::{
    BreakpointCondition, BreakpointEntry, BreakpointManager, ShouldFire,
};
use atlas_runtime::debugger::inspection::{
    format_value_with_depth, EvalResult, Inspector, ScopedVariable, VariableScope,
};
use atlas_runtime::debugger::protocol::{
    deserialize_event, deserialize_request, deserialize_response, serialize_event,
    serialize_request, serialize_response, Breakpoint, DebugEvent, DebugRequest, DebugResponse,
    DebugStackFrame, PauseReason, SourceLocation, Variable,
};
use atlas_runtime::debugger::source_map::{
    byte_offset_to_line_column, compute_line_offsets, SourceMap,
};
use atlas_runtime::debugger::state::{DebuggerState, StepMode};
use atlas_runtime::debugger::stepping::{StepRequest, StepTracker};
use atlas_runtime::debugger::DebuggerSession;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::value::Value;
use atlas_runtime::vm::VM;

// Shared helper functions
fn compile(source: &str) -> Bytecode {
    let tokens = Lexer::new(source).tokenize().0;
    let (ast, _) = Parser::new(tokens).parse();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("compile failed")
}

fn security() -> SecurityContext {
    SecurityContext::allow_all()
}

fn loc(line: u32) -> SourceLocation {
    SourceLocation::new("test.atlas", line, 1)
}

fn new_session(source: &str) -> DebuggerSession {
    let bc = compile(source);
    DebuggerSession::new(bc, source, "test.atlas")
}

// Domain submodules (files live in tests/debugger/)
#[path = "debugger/breakpoints.rs"]
mod breakpoints;
#[path = "debugger/inspector.rs"]
mod inspector;
#[path = "debugger/interpreter.rs"]
mod interpreter;
#[path = "debugger/protocol_part1.rs"]
mod protocol_part1;
#[path = "debugger/protocol_part2.rs"]
mod protocol_part2;
#[path = "debugger/session.rs"]
mod session;
