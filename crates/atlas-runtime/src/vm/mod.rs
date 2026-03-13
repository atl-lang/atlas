//! Stack-based virtual machine
//!
//! Executes bytecode instructions with a value stack and call frames.
//! - Arithmetic operations check for NaN/Infinity
//! - Variables are stored in locals (stack) or globals (HashMap)
//! - Control flow uses jumps and loops

#![cfg_attr(not(test), deny(clippy::unwrap_used))]

mod debugger;
pub mod dispatch;
mod frame;
mod profiler;

pub use debugger::{DebugAction, DebugHook, Debugger};
pub use frame::CallFrame;
pub use profiler::Profiler;

use crate::bytecode::{Bytecode, Opcode};
use crate::diagnostic::StackTraceFrame;
use crate::ffi::{ExternFunction, LibraryLoader};
use crate::span::Span;
use crate::value::{RuntimeError, Value, ValueArray, ValueHashMap, ValueHashSet};
use std::collections::HashMap;
use std::sync::Arc;

/// Tracks the origin of a value on the operand stack (debug builds only).
///
/// When an `own` parameter is called, the origin tells us which local slot or
/// global name to mark as consumed so that subsequent reads produce an error.
#[cfg(debug_assertions)]
#[derive(Clone, Debug)]
enum StackValueOrigin {
    /// Value loaded from a frame-relative local slot index.
    Local(usize),
    /// Value loaded from a named global variable.
    Global(String),
}

/// Result returned by [`VM::run_debuggable`].
#[derive(Debug)]
pub enum VmRunResult {
    /// Execution completed normally.
    Complete(Option<Value>),
    /// Execution was paused (breakpoint hit or step condition met).
    ///
    /// The VM state is fully preserved.  Resume by calling `run_debuggable` again
    /// (or `run` to run without further debug checks).
    Paused {
        /// Instruction pointer at the pause point (the instruction that would
        /// have been executed next has NOT been executed yet).
        ip: usize,
    },
}

/// Virtual machine state
pub struct VM {
    /// Value stack
    stack: Vec<Value>,
    /// Call frames (for function calls)
    frames: Vec<CallFrame>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Bytecode to execute
    bytecode: Bytecode,
    /// Instruction pointer
    ip: usize,
    /// Optional profiler for performance analysis
    profiler: Option<Profiler>,
    /// Optional debugger for step-through execution
    debugger: Option<Debugger>,
    /// Set to `true` by the execute_loop when the debugger requests a pause.
    /// Cleared by `run_debuggable` after it reads the flag.
    debug_pause_pending: bool,
    /// Security context for current execution (set during run())
    current_security: Option<std::sync::Arc<crate::security::SecurityContext>>,
    /// Execution limits for timeout enforcement
    execution_limits: Option<std::sync::Arc<crate::api::config::ExecutionLimits>>,
    /// Output writer for print() (defaults to stdout)
    output_writer: crate::stdlib::OutputWriter,
    /// FFI library loader (phase-10b)
    library_loader: LibraryLoader,
    /// Loaded extern functions (phase-10b)
    extern_functions: HashMap<String, ExternFunction>,
    /// Reusable string buffer for temporary string operations (reduces allocations)
    string_buffer: String,
    /// Nominal struct names for HashMap-backed struct values (keyed by map identity).
    struct_type_names: HashMap<usize, String>,
    /// Parallel to `stack`: tracks where each value originated (debug builds only).
    /// `None` for computed/literal values; `Some(StackValueOrigin::Local(slot))` for values
    /// loaded via `GetLocal`; `Some(StackValueOrigin::Global(name))` for `GetGlobal`.
    /// Used to mark the caller's binding as consumed when a value is passed to an `own`
    /// parameter.  Zero overhead in release builds.
    #[cfg(debug_assertions)]
    value_origins: Vec<Option<StackValueOrigin>>,
    /// Per-frame consumed-slot tracking (debug builds only).  `consumed_slots[frame_idx][slot]`
    /// is `true` when that local slot has been moved into an `own` parameter and may no longer
    /// be read.
    #[cfg(debug_assertions)]
    consumed_slots: Vec<Vec<bool>>,
    /// Consumed global names (debug builds only).  A global is inserted here when it is
    /// passed to an `own` parameter.  Subsequent `GetGlobal` for the same name errors.
    #[cfg(debug_assertions)]
    consumed_globals: std::collections::HashSet<String>,
    /// Optional JIT compiler for hot function execution
    jit: Option<Box<dyn crate::JitCompiler>>,
    /// Runtime warnings collected during execution (ownership mismatches, etc.).
    /// Callers retrieve and emit these after execution via `take_runtime_warnings()`.
    pub runtime_warnings: Vec<crate::diagnostic::Diagnostic>,
    /// Per-frame defer stacks. Each entry is (body_start_ip, body_length).
    /// Deferred blocks execute in LIFO order when the frame exits.
    defer_stacks: Vec<Vec<(usize, usize)>>,
}

impl VM {
    /// Create a new VM with bytecode
    pub fn new(bytecode: Bytecode) -> Self {
        // Create an initial "main" frame for top-level code
        #[cfg(debug_assertions)]
        let main_local_count = bytecode.top_level_local_count;

        let main_frame = CallFrame {
            function_name: "<main>".to_string(),
            return_ip: 0,
            stack_base: 0,
            local_count: bytecode.top_level_local_count,
            upvalues: std::sync::Arc::new(Vec::new()),
        };

        Self {
            stack: Vec::with_capacity(1024),
            frames: vec![main_frame],
            globals: HashMap::new(),
            bytecode,
            ip: 0,
            profiler: None,
            debugger: None,
            debug_pause_pending: false,
            current_security: None,
            execution_limits: None,
            output_writer: crate::stdlib::stdout_writer(),
            library_loader: LibraryLoader::new(),
            extern_functions: HashMap::new(),
            string_buffer: String::with_capacity(256),
            struct_type_names: HashMap::new(),
            #[cfg(debug_assertions)]
            value_origins: Vec::with_capacity(1024),
            #[cfg(debug_assertions)]
            consumed_slots: vec![vec![false; main_local_count]],
            #[cfg(debug_assertions)]
            consumed_globals: std::collections::HashSet::new(),
            jit: None,
            runtime_warnings: Vec::new(),
            defer_stacks: vec![Vec::new()], // One empty stack for main frame
        }
    }

    /// Load a new module bytecode into the VM, resetting execution state while
    /// preserving the global variable table. Use this to run dependency modules
    /// in sequence on a single VM so their exported globals are visible to later modules.
    ///
    /// The new module's bytecode is MERGED into the existing bytecode so that all
    /// function `bytecode_offset` values remain valid across module boundaries.
    /// This is required for cross-module function calls: a function defined in module A
    /// has a `bytecode_offset` relative to module A's instruction stream; after merging,
    /// that offset is adjusted to be correct in the combined instruction stream.
    ///
    /// Call `run()` after this to execute the loaded module.
    pub fn load_module(&mut self, new_bc: Bytecode) {
        let instr_base = self.bytecode.instructions.len();
        let const_base = self.bytecode.constants.len();
        let local_count = new_bc.top_level_local_count;

        // 1. Merge constants: adjust FunctionRef bytecode_offsets by instr_base so they
        //    remain valid in the combined instruction stream.
        for constant in new_bc.constants {
            let adjusted = match constant {
                crate::value::Value::Function(mut f) => {
                    f.bytecode_offset += instr_base;
                    crate::value::Value::Function(f)
                }
                other => other,
            };
            self.bytecode.constants.push(adjusted);
        }

        // 2. Adjust the new module's instruction stream: all constant pool index operands
        //    must be shifted by `const_base` to account for the pre-existing constants.
        let adjusted_instrs = Self::adjust_constant_refs(&new_bc.instructions, const_base as u16);

        // 3. Merge debug info: shift instruction offsets by instr_base.
        for mut dbg in new_bc.debug_info {
            dbg.instruction_offset += instr_base;
            self.bytecode.debug_info.push(dbg);
        }

        // 4. Append adjusted instructions — the new module's code now lives at instr_base..end.
        self.bytecode.instructions.extend(adjusted_instrs);

        // 5. Reset execution state to start at the new module's entry point.
        self.ip = instr_base;
        self.stack.clear();
        self.frames = vec![CallFrame {
            function_name: "<main>".to_string(),
            return_ip: 0,
            stack_base: 0,
            local_count,
            upvalues: std::sync::Arc::new(Vec::new()),
        }];

        #[cfg(debug_assertions)]
        {
            self.value_origins.clear();
            self.consumed_slots = vec![vec![false; local_count]];
            // Preserve consumed_globals across modules — they accumulate.
        }
    }

    /// Rewrite all constant pool index operands in an instruction stream by adding `const_base`.
    ///
    /// Jump offsets (relative i16) and non-constant-index operands are passed through unchanged.
    /// This is used by `load_module` to fix up a new module's instruction stream before
    /// appending it to the existing combined bytecode.
    fn adjust_constant_refs(instructions: &[u8], const_base: u16) -> Vec<u8> {
        if const_base == 0 {
            return instructions.to_vec();
        }

        fn read_u16(bytes: &[u8]) -> u16 {
            ((bytes[0] as u16) << 8) | bytes[1] as u16
        }
        fn push_u16(result: &mut Vec<u8>, v: u16) {
            result.push((v >> 8) as u8);
            result.push((v & 0xFF) as u8);
        }

        let mut result = Vec::with_capacity(instructions.len());
        let mut i = 0;

        while i < instructions.len() {
            let opcode = instructions[i];
            result.push(opcode);
            i += 1;

            match opcode {
                // ONE u16 constant-pool index → adjust
                0x01 | // Constant
                0x12 | // GetGlobal
                0x13   // SetGlobal
                => {
                    let idx = read_u16(&instructions[i..]);
                    push_u16(&mut result, idx + const_base);
                    i += 2;
                }

                // MakeClosure: [u16 func_const_idx][u16 n_upvalues]
                // Only func_const_idx is a pool reference; n_upvalues is a count.
                0x14 => {
                    let func_idx = read_u16(&instructions[i..]);
                    let n_up = read_u16(&instructions[i + 2..]);
                    push_u16(&mut result, func_idx + const_base);
                    push_u16(&mut result, n_up);
                    i += 4;
                }

                // TraitDispatch: [u16 trait_idx][u16 method_idx][u8 arg_count]
                // Both u16s are pool references.
                0x62 => {
                    let trait_idx = read_u16(&instructions[i..]);
                    let method_idx = read_u16(&instructions[i + 2..]);
                    let argc = instructions[i + 4];
                    push_u16(&mut result, trait_idx + const_base);
                    push_u16(&mut result, method_idx + const_base);
                    result.push(argc);
                    i += 5;
                }

                // Struct: [u16 name_idx][u16 field_count]
                // Only name_idx is a pool reference; field_count is a count.
                0x7B => {
                    let name_idx = read_u16(&instructions[i..]);
                    let field_count = read_u16(&instructions[i + 2..]);
                    push_u16(&mut result, name_idx + const_base);
                    push_u16(&mut result, field_count);
                    i += 4;
                }

                // AsyncCall, SpawnTask: [u16 fn_const_idx][u8 arg_count]
                // fn_const_idx is a pool reference.
                0xA0 | 0xA3 => {
                    let fn_idx = read_u16(&instructions[i..]);
                    let argc = instructions[i + 2];
                    push_u16(&mut result, fn_idx + const_base);
                    result.push(argc);
                    i += 3;
                }

                // ONE u16 operand that is NOT a pool index → pass through unchanged
                0x10 | // GetLocal (local index)
                0x11 | // SetLocal (local index)
                0x15 | // GetUpvalue (upvalue index)
                0x16 | // SetUpvalue (upvalue index)
                0x70 | // Array (element count)
                0x73 | // HashMap (pair count)
                0x7C | // Tuple (element count)
                0x7D   // TupleGet (tuple index)
                => {
                    let v = read_u16(&instructions[i..]);
                    push_u16(&mut result, v);
                    i += 2;
                }

                // i16 relative jump offsets → pass through unchanged (they're relative to ip)
                // 0x50=Jump, 0x51=JumpIfFalse, 0x52=Loop
                0x50..=0x52 => {
                    result.push(instructions[i]);
                    result.push(instructions[i + 1]);
                    i += 2;
                }

                // ONE u8 operand → pass through unchanged
                0x60 | // Call (arg_count)
                0x98   // EnumVariant (arg_count)
                => {
                    result.push(instructions[i]);
                    i += 1;
                }

                // No operands (or unknown opcode) → nothing extra to copy
                _ => {}
            }
        }

        result
    }

    /// Create a new VM with profiling enabled
    pub fn with_profiling(bytecode: Bytecode) -> Self {
        let mut vm = Self::new(bytecode);
        vm.profiler = Some(Profiler::enabled());
        vm
    }

    /// Create a new VM with debugging enabled
    pub fn with_debugging(bytecode: Bytecode) -> Self {
        let mut vm = Self::new(bytecode);
        vm.debugger = Some(Debugger::enabled());
        vm
    }

    /// Set the output writer (used by Runtime to redirect print() output)
    pub fn set_output_writer(&mut self, writer: crate::stdlib::OutputWriter) {
        self.output_writer = writer;
    }

    /// Drain and return all runtime warnings collected during execution.
    /// Called by the runtime layer after execute() to surface warnings through
    /// the proper diagnostic system instead of inline eprintln!().
    pub fn take_runtime_warnings(&mut self) -> Vec<crate::diagnostic::Diagnostic> {
        std::mem::take(&mut self.runtime_warnings)
    }

    /// Set execution limits for timeout enforcement
    pub fn set_execution_limits(
        &mut self,
        limits: std::sync::Arc<crate::api::config::ExecutionLimits>,
    ) {
        self.execution_limits = Some(limits);
    }

    /// Track memory allocation and check if it exceeds the limit.
    ///
    /// Call this before creating heap-allocated values (arrays, strings, maps).
    /// Returns Ok(()) if within limits, Err if limit exceeded.
    #[inline]
    fn track_memory(&self, bytes: usize) -> Result<(), RuntimeError> {
        if let Some(ref limits) = self.execution_limits {
            limits.track_allocation(bytes)
        } else {
            Ok(())
        }
    }

    fn register_struct_type(&mut self, map: &ValueHashMap, name: &str) {
        let key = Arc::as_ptr(map.arc()) as usize;
        self.struct_type_names.insert(key, name.to_string());
    }

    fn struct_name_for_value(&self, value: &Value) -> Option<&str> {
        match value {
            Value::Map(map) => {
                let key = Arc::as_ptr(map.arc()) as usize;
                self.struct_type_names.get(&key).map(|name| name.as_str())
            }
            _ => None,
        }
    }

    /// Estimate the memory size of a Value slice (for arrays)
    ///
    /// Conservative estimate: each Value is ~64 bytes on stack + heap allocations.
    /// For nested structures, we only count the immediate level.
    #[inline]
    fn estimate_array_size(len: usize) -> usize {
        // Base overhead for Arc<Vec<_>> (~24 bytes) + Vec capacity
        const ARRAY_OVERHEAD: usize = 24;
        // Conservative estimate per Value slot
        const VALUE_SIZE: usize = 64;

        ARRAY_OVERHEAD + len * VALUE_SIZE
    }

    /// Estimate the memory size of a string
    #[inline]
    fn estimate_string_size(len: usize) -> usize {
        // Arc<String> overhead (~24 bytes) + string data
        const STRING_OVERHEAD: usize = 24;
        STRING_OVERHEAD + len
    }

    /// Set a JIT compiler for hot function execution
    ///
    /// When set, the VM will attempt to JIT-compile and execute hot functions
    /// instead of interpreting them. Functions that use unsupported opcodes
    /// or haven't been called enough times will fall back to interpretation.
    pub fn set_jit(&mut self, jit: Box<dyn crate::JitCompiler>) {
        self.jit = Some(jit);
    }

    /// Get JIT statistics if JIT is enabled
    pub fn jit_stats(&self) -> Option<crate::JitStats> {
        self.jit.as_ref().map(|j| j.stats())
    }

    /// Set a global variable
    ///
    /// Used by the Runtime to inject native functions and other complex values
    /// that can't be represented in bytecode constants.
    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    /// Get all global variables
    ///
    /// Used by the Runtime to persist VM globals back to interpreter state
    /// after execution completes.
    pub fn get_globals(&self) -> &std::collections::HashMap<String, Value> {
        &self.globals
    }

    /// Load extern declarations from AST (phase-10b)
    ///
    /// Processes extern function declarations by loading libraries and looking up symbols.
    /// Must be called before running bytecode that calls extern functions.
    pub fn load_extern_declarations(
        &mut self,
        program: &crate::ast::Program,
    ) -> Result<(), RuntimeError> {
        use crate::ast::Item;
        use crate::value::FunctionRef;

        for item in &program.items {
            if let Item::Extern(extern_decl) = item {
                // Load the dynamic library
                self.library_loader
                    .load(&extern_decl.library)
                    .map_err(|e| RuntimeError::TypeError {
                        msg: format!("Failed to load library '{}': {}", extern_decl.library, e),
                        span: extern_decl.span,
                    })?;

                // Determine the symbol name (use 'as' name if provided, otherwise function name)
                let symbol_name = extern_decl.symbol.as_ref().unwrap_or(&extern_decl.name);

                // Look up the function symbol
                // SAFETY: The library was loaded successfully and remains cached in the loader.
                // `lookup_symbol` requires the symbol to exist and have the expected type.
                // Preconditions: `extern_decl.library` is trusted and stays loaded for the
                // lifetime of the returned symbol pointer.
                let fn_ptr = unsafe {
                    self.library_loader
                        .lookup_symbol::<*const ()>(&extern_decl.library, symbol_name)
                        .map_err(|e| RuntimeError::TypeError {
                            msg: format!(
                                "Failed to find symbol '{}' in library '{}': {}",
                                symbol_name, extern_decl.library, e
                            ),
                            span: extern_decl.span,
                        })?
                };

                // Convert parameter types from AST to FFI types
                let param_types: Vec<crate::ffi::ExternType> = extern_decl
                    .params
                    .iter()
                    .map(|(_, ty)| convert_extern_type_annotation(ty))
                    .collect();

                let return_type = convert_extern_type_annotation(&extern_decl.return_type);

                // Create ExternFunction
                // SAFETY: `fn_ptr` came from a symbol lookup in the loaded library.
                // We only construct an `ExternFunction` with the exact parameter/return
                // types declared in the AST, and the library remains loaded while the
                // VM holds this function.
                let extern_fn = unsafe { ExternFunction::new(*fn_ptr, param_types, return_type) };

                // Store the extern function
                self.extern_functions
                    .insert(extern_decl.name.clone(), extern_fn);

                // Register as a callable global
                let arity = extern_decl.params.len();
                let func_value = Value::Function(FunctionRef {
                    name: extern_decl.name.clone(),
                    arity,
                    required_arity: arity, // Extern functions have no defaults
                    bytecode_offset: 0,    // Not used for extern functions
                    local_count: 0,        // Not used for extern functions
                    param_ownership: vec![],
                    param_names: vec![],
                    defaults: vec![None; arity],
                    return_ownership: None,
                    is_async: false,
                    has_rest_param: false,
                });
                self.globals.insert(extern_decl.name.clone(), func_value);
            }
        }

        Ok(())
    }
}

/// Convert ExternTypeAnnotation (AST) to ExternType (FFI runtime)
fn convert_extern_type_annotation(
    annotation: &crate::ast::ExternTypeAnnotation,
) -> crate::ffi::ExternType {
    use crate::ast::ExternTypeAnnotation;
    use crate::ffi::ExternType;

    match annotation {
        ExternTypeAnnotation::CInt => ExternType::CInt,
        ExternTypeAnnotation::CLong => ExternType::CLong,
        ExternTypeAnnotation::CDouble => ExternType::CDouble,
        ExternTypeAnnotation::CCharPtr => ExternType::CCharPtr,
        ExternTypeAnnotation::CVoid => ExternType::CVoid,
        ExternTypeAnnotation::CBool => ExternType::CBool,
    }
}

impl VM {
    /// Set the instruction pointer
    ///
    /// Used by Runtime to start execution from a specific offset when
    /// accumulating bytecode across multiple eval() calls.
    pub fn set_ip(&mut self, offset: usize) {
        self.ip = offset;
    }

    /// Enable profiling
    pub fn enable_profiling(&mut self) {
        if let Some(ref mut profiler) = self.profiler {
            profiler.enable();
        } else {
            self.profiler = Some(Profiler::enabled());
        }
    }

    /// Disable profiling
    pub fn disable_profiling(&mut self) {
        if let Some(ref mut profiler) = self.profiler {
            profiler.disable();
        }
    }

    /// Get profiler reference
    pub fn profiler(&self) -> Option<&Profiler> {
        self.profiler.as_ref()
    }

    /// Get mutable profiler reference
    pub fn profiler_mut(&mut self) -> Option<&mut Profiler> {
        self.profiler.as_mut()
    }

    /// Enable debugging
    pub fn enable_debugging(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            debugger.enable();
        } else {
            self.debugger = Some(Debugger::enabled());
        }
    }

    /// Disable debugging
    pub fn disable_debugging(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            debugger.disable();
        }
    }

    /// Get debugger reference
    pub fn debugger(&self) -> Option<&Debugger> {
        self.debugger.as_ref()
    }

    /// Get mutable debugger reference
    pub fn debugger_mut(&mut self) -> Option<&mut Debugger> {
        self.debugger.as_mut()
    }

    /// Get the source span for the current instruction pointer
    ///
    /// Returns the span from debug info if available.
    /// Useful for error reporting with source location context.
    pub fn current_span(&self) -> Option<crate::span::Span> {
        if self.ip == 0 {
            return None;
        }
        self.bytecode.get_span_for_offset(self.ip - 1)
    }

    /// Get the source span for a specific instruction offset
    pub fn span_for_offset(&self, offset: usize) -> Option<crate::span::Span> {
        self.bytecode.get_span_for_offset(offset)
    }

    // ── Debugger inspection API ───────────────────────────────────────────────

    /// Get the current instruction pointer.
    pub fn current_ip(&self) -> usize {
        self.ip
    }

    /// Get the current call-frame depth (number of active frames).
    pub fn frame_depth(&self) -> usize {
        self.frames.len()
    }

    /// Get the current value-stack depth.
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Get a call frame by index (0 = innermost / most recent).
    ///
    /// Returns `None` if `index` is out of range.
    pub fn get_frame_at(&self, index: usize) -> Option<&CallFrame> {
        let len = self.frames.len();
        if index < len {
            self.frames.get(len - 1 - index)
        } else {
            None
        }
    }

    /// Get the local variable values for a call frame.
    ///
    /// `frame_index` 0 is the innermost (current) frame.
    /// Returns `(slot_index, &Value)` pairs for all initialised local slots.
    pub fn get_locals_for_frame(&self, frame_index: usize) -> Vec<(usize, &Value)> {
        let frame = match self.get_frame_at(frame_index) {
            Some(f) => f,
            None => return Vec::new(),
        };
        let base = frame.stack_base;
        let count = frame.local_count;
        (0..count)
            .filter_map(|i| self.stack.get(base + i).map(|v| (i, v)))
            .collect()
    }

    /// Get all global variables.
    pub fn get_global_variables(&self) -> &HashMap<String, Value> {
        &self.globals
    }

    /// Get debug information from the bytecode.
    pub fn debug_spans(&self) -> &[crate::bytecode::DebugSpan] {
        &self.bytecode.debug_info
    }

    // ── Debuggable execution ──────────────────────────────────────────────────

    /// Execute bytecode with live debugger-state integration.
    ///
    /// Syncs verified breakpoints and step conditions from `debug_state` into
    /// the VM's embedded debugger, then runs until the program completes or a
    /// pause condition fires.
    ///
    /// When `VmRunResult::Paused` is returned the VM state is fully preserved
    /// and execution can be resumed by calling `run_debuggable` again.
    pub fn run_debuggable(
        &mut self,
        debug_state: &mut crate::debugger::DebuggerState,
        security: &crate::security::SecurityContext,
    ) -> Result<VmRunResult, RuntimeError> {
        use crate::debugger::state::StepMode;
        use debugger::StepCondition;

        // Sync verified breakpoints from debug_state into the embedded Debugger.
        self.enable_debugging();
        let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
        let dbg = self
            .debugger
            .as_mut()
            .ok_or_else(|| RuntimeError::InternalError {
                msg: "Debugger not initialized".to_string(),
                span,
            })?;
        dbg.clear_breakpoints();
        for bp in debug_state.breakpoints() {
            if let Some(offset) = bp.instruction_offset {
                dbg.set_breakpoint(offset);
            }
        }

        // Configure the step condition.
        let step_condition = match debug_state.step_mode {
            StepMode::None => StepCondition::None,
            StepMode::Into => StepCondition::Always,
            StepMode::Over => StepCondition::OverDepth(debug_state.step_start_frame_depth),
            StepMode::Out => StepCondition::OutDepth(debug_state.step_start_frame_depth),
        };
        let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
        let dbg = self
            .debugger
            .as_mut()
            .ok_or_else(|| RuntimeError::InternalError {
                msg: "Debugger not initialized".to_string(),
                span,
            })?;
        dbg.set_step_condition(step_condition);

        // Clear any leftover pause flag from a previous run.
        self.debug_pause_pending = false;

        // Run the execute loop (profiling hooks still active).
        self.current_security = Some(std::sync::Arc::new(security.clone()));
        if let Some(ref mut profiler) = self.profiler {
            if profiler.is_enabled() {
                profiler.start_timing();
            }
        }
        let loop_result = self.execute_until_end();
        if let Some(ref mut profiler) = self.profiler {
            if profiler.is_enabled() {
                profiler.stop_timing();
            }
        }

        if self.debug_pause_pending {
            // The execute_loop broke early due to a debug pause.
            let ip = self.ip;
            let location = None; // Caller resolves via SourceMap
            let reason = if let Some(bp) = debug_state.breakpoint_at_offset(ip) {
                crate::debugger::protocol::PauseReason::Breakpoint { id: bp.id }
            } else {
                crate::debugger::protocol::PauseReason::Step
            };
            debug_state.pause(reason, location, ip);
            debug_state.clear_step_mode();
            self.debug_pause_pending = false;
            Ok(VmRunResult::Paused { ip })
        } else {
            debug_state.stop();
            Ok(VmRunResult::Complete(loop_result?))
        }
    }

    /// Execute the bytecode
    pub fn run(
        &mut self,
        security: &crate::security::SecurityContext,
    ) -> Result<Option<Value>, RuntimeError> {
        // Store security context for builtin calls
        self.current_security = Some(std::sync::Arc::new(security.clone()));
        // Start profiling timer if profiler is enabled
        if let Some(ref mut profiler) = self.profiler {
            if profiler.is_enabled() {
                profiler.start_timing();
            }
        }
        let result = self.execute_until_end();
        // Stop profiling timer
        if let Some(ref mut profiler) = self.profiler {
            if profiler.is_enabled() {
                profiler.stop_timing();
            }
        }
        result
    }

    /// Execute bytecode until reaching the end of instructions
    fn execute_until_end(&mut self) -> Result<Option<Value>, RuntimeError> {
        self.execute_loop(None)
    }

    /// Execute bytecode until a specific frame depth is reached (for function calls)
    /// If target_frame_depth is Some(n), stops when frames.len() <= n
    /// If target_frame_depth is None, runs until end of bytecode
    fn execute_loop(
        &mut self,
        target_frame_depth: Option<usize>,
    ) -> Result<Option<Value>, RuntimeError> {
        loop {
            // Check termination conditions
            if self.ip >= self.bytecode.instructions.len() {
                break;
            }

            // Check execution timeout (amortized - only checks every N instructions)
            if let Some(ref limits) = self.execution_limits {
                limits.tick_and_check()?;
            }

            // Check if we've returned from the target frame
            if let Some(depth) = target_frame_depth {
                if self.frames.len() <= depth {
                    // Frame has returned, result should be on stack
                    return Ok(Some(self.peek(0).clone()));
                }
            }

            let opcode = self.read_opcode()?;

            // Debugger hook: before instruction (zero overhead when disabled)
            if let Some(ref mut debugger) = self.debugger {
                if debugger.is_enabled() {
                    let current_ip = self.ip - 1;
                    let frame_depth = self.frames.len();
                    let action =
                        debugger.before_instruction_with_depth(current_ip, opcode, frame_depth);
                    match action {
                        DebugAction::Pause | DebugAction::Step => {
                            // Back up IP so the paused instruction is re-executed on resume
                            self.ip = current_ip;
                            self.debug_pause_pending = true;
                            break; // break the execute_loop loop
                        }
                        DebugAction::Continue => {
                            // Normal execution – continue with opcode dispatch
                        }
                    }
                }
            }

            // Record instruction for profiling (zero overhead when disabled)
            if let Some(ref mut profiler) = self.profiler {
                if profiler.is_enabled() {
                    let instruction_ip = self.ip - 1; // ip already advanced by read_opcode
                    profiler.record_instruction_at(opcode, instruction_ip);
                    profiler.update_value_stack_depth(self.stack.len());
                    profiler.update_frame_depth(self.frames.len());
                }
            }

            match opcode {
                // ===== Constants =====
                Opcode::Constant => {
                    let index = self.read_u16()? as usize;
                    if index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let value = self.bytecode.constants[index].clone();
                    self.push(value);
                }
                Opcode::Null => self.push(Value::Null),
                Opcode::True => self.push(Value::Bool(true)),
                Opcode::False => self.push(Value::Bool(false)),

                // ===== Variables =====
                Opcode::GetLocal => {
                    let index = self.read_u16()? as usize;
                    let base = self.current_frame().stack_base;
                    let absolute_index = base + index;
                    if absolute_index >= self.stack.len() {
                        return Err(RuntimeError::StackUnderflow {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    // Debug mode: reject reads of consumed (moved) local slots.
                    #[cfg(debug_assertions)]
                    {
                        let frame_idx = self.frames.len() - 1;
                        if self.consumed_slots[frame_idx]
                            .get(index)
                            .copied()
                            .unwrap_or(false)
                        {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "use of moved value: local[{}] was passed to 'own' parameter and is no longer valid",
                                    index
                                ),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                    let value = self.stack[absolute_index].clone();
                    self.push(value);
                    // Record that the top-of-stack value originated from this local slot.
                    #[cfg(debug_assertions)]
                    {
                        if let Some(origin) = self.value_origins.last_mut() {
                            *origin = Some(StackValueOrigin::Local(index));
                        } else {
                            return Err(RuntimeError::InternalError {
                                msg: "Missing value origin stack for local read".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }
                Opcode::SetLocal => {
                    let index = self.read_u16()? as usize;
                    let base = self.current_frame().stack_base;
                    let local_count = self.current_frame().local_count;
                    let absolute_index = base + index;
                    let value = self.peek(0).clone();

                    // SAFETY CHECK: Prevent unbounded stack growth
                    // This prevents memory explosion from invalid bytecode or compiler bugs
                    if index >= local_count {
                        return Err(RuntimeError::StackUnderflow {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }

                    // Extend stack if needed (for local variables not yet initialized)
                    if absolute_index >= self.stack.len() {
                        // Bounded extension: only up to the declared local_count
                        let needed = absolute_index - self.stack.len() + 1;
                        if base + local_count > self.stack.len() + needed {
                            return Err(RuntimeError::StackUnderflow {
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                        for _ in 0..needed {
                            self.push(Value::Null);
                        }
                    }
                    self.stack[absolute_index] = value;
                }
                Opcode::GetGlobal => {
                    let name_index = self.read_u16()? as usize;
                    if name_index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let name = match &self.bytecode.constants[name_index] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Expected string constant for variable name".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };
                    // Debug mode: reject reads of consumed globals.
                    #[cfg(debug_assertions)]
                    if self.consumed_globals.contains(&name) {
                        return Err(RuntimeError::TypeError {
                            msg: format!(
                                "use of moved value: '{}' was passed to 'own' parameter and is no longer valid",
                                name
                            ),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let value = if name == "None" {
                        // Constructor literal: None always evaluates to Option::None
                        Value::Option(None)
                    } else if let Some(v) = self.globals.get(&name) {
                        v.clone()
                    } else if crate::stdlib::is_builtin(&name)
                        || crate::stdlib::is_array_intrinsic(&name)
                    {
                        // Builtin or intrinsic - return builtin value
                        Value::Builtin(std::sync::Arc::from(name.as_str()))
                    } else if crate::method_dispatch::is_static_namespace(&name) {
                        // Static namespace (Json, Math, console, reflect, etc.) - return as builtin
                        Value::Builtin(std::sync::Arc::from(name.as_str()))
                    } else {
                        // B22: Math constants removed as bare identifiers. Use Math.PI, Math.E, etc.
                        return Err(RuntimeError::UndefinedVariable {
                            name: name.clone(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    };
                    self.push(value);
                    // Record global origin for own-consume tracking (debug builds only).
                    // Only track user-defined globals (not builtins, constructors, math constants).
                    #[cfg(debug_assertions)]
                    if self.globals.contains_key(&name) {
                        if let Some(origin) = self.value_origins.last_mut() {
                            *origin = Some(StackValueOrigin::Global(name));
                        } else {
                            return Err(RuntimeError::InternalError {
                                msg: "Missing value origin stack for global read".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }
                Opcode::SetGlobal => {
                    let name_index = self.read_u16()? as usize;
                    if name_index >= self.bytecode.constants.len() {
                        return Err(RuntimeError::UnknownOpcode {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let name = match &self.bytecode.constants[name_index] {
                        Value::String(s) => s.as_ref().clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Expected string constant for variable name".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                }

                Opcode::MakeClosure => {
                    let func_const_idx = self.read_u16()? as usize;
                    let n_upvalues = self.read_u16()? as usize;

                    // Get the FunctionRef from constant pool
                    let func = match self.bytecode.constants.get(func_const_idx) {
                        Some(Value::Function(f)) => f.clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "MakeClosure: constant is not a function".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };

                    // Pop upvalues from stack (in reverse order since stack is LIFO)
                    let mut upvalues = Vec::with_capacity(n_upvalues);
                    for _ in 0..n_upvalues {
                        upvalues.push(self.pop());
                    }
                    upvalues.reverse(); // Restore capture order

                    let closure = crate::value::ClosureRef {
                        func,
                        upvalues: std::sync::Arc::new(upvalues),
                    };
                    self.push(Value::Closure(closure));
                }

                Opcode::GetUpvalue => {
                    let idx = self.read_u16()? as usize;
                    let value = match self.current_frame().upvalues.get(idx) {
                        Some(v) => v.clone(),
                        None => {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "Upvalue index {} out of bounds (closure has {} upvalues)",
                                    idx,
                                    self.current_frame().upvalues.len()
                                ),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };
                    self.push(value);
                }

                Opcode::SetUpvalue => {
                    let idx = self.read_u16()? as usize;
                    let value = self.peek(0).clone();
                    let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
                    let frame =
                        self.frames
                            .last_mut()
                            .ok_or_else(|| RuntimeError::InternalError {
                                msg: "Missing call frame for SetUpvalue".to_string(),
                                span,
                            })?;
                    let upvalues = std::sync::Arc::make_mut(&mut frame.upvalues);
                    if idx < upvalues.len() {
                        upvalues[idx] = value;
                    } else {
                        return Err(RuntimeError::TypeError {
                            msg: format!(
                                "SetUpvalue: index {} out of bounds (closure has {} upvalues)",
                                idx,
                                upvalues.len()
                            ),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                }

                // ===== Arithmetic =====
                Opcode::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (Value::Number(x), Value::Number(y)) => {
                            let result = x + y;
                            if result.is_nan() || result.is_infinite() {
                                return Err(RuntimeError::InvalidNumericResult {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                            self.push(Value::Number(result));
                        }
                        (Value::String(x), Value::String(y)) => {
                            // Track memory for the concatenated string
                            let new_len = x.len() + y.len();
                            self.track_memory(Self::estimate_string_size(new_len))?;

                            // Reuse string buffer to reduce allocations
                            self.string_buffer.clear();
                            self.string_buffer.push_str(x);
                            self.string_buffer.push_str(y);
                            self.push(Value::String(Arc::new(self.string_buffer.clone())));
                        }
                        (Value::Array(x), Value::Array(y)) => {
                            let new_len = x.len() + y.len();
                            self.track_memory(Self::estimate_array_size(new_len))?;

                            let mut elements = Vec::with_capacity(new_len);
                            elements.extend_from_slice(x.as_slice());
                            elements.extend_from_slice(y.as_slice());
                            self.push(Value::Array(ValueArray::from_vec(elements)));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Invalid operands for +".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }
                Opcode::Sub => self.binary_numeric_op(|a, b| a - b)?,
                Opcode::Mul => self.binary_numeric_op(|a, b| a * b)?,
                Opcode::Div => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(RuntimeError::DivideByZero {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let result = a / b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    self.push(Value::Number(result));
                }
                Opcode::Mod => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(RuntimeError::DivideByZero {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    let result = a % b;
                    if result.is_nan() || result.is_infinite() {
                        return Err(RuntimeError::InvalidNumericResult {
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }
                    self.push(Value::Number(result));
                }
                Opcode::Negate => {
                    let value = self.pop();
                    match value {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot negate non-number".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }

                // ===== Comparison =====
                Opcode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Opcode::NotEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }
                Opcode::Less => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a < b));
                }
                Opcode::LessEqual => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a <= b));
                }
                Opcode::Greater => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a > b));
                }
                Opcode::GreaterEqual => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.push(Value::Bool(a >= b));
                }

                // ===== Logical =====
                Opcode::Not => {
                    let value = self.pop();
                    match value {
                        Value::Bool(b) => self.push(Value::Bool(!b)),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot apply ! to non-boolean".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }
                Opcode::And => {
                    // Non-short-circuit And: both operands already evaluated
                    // Short-circuit is handled by compiler via JumpIfFalse
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (Value::Bool(a_val), Value::Bool(b_val)) => {
                            self.push(Value::Bool(*a_val && *b_val));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "Cannot apply && to {} and {}",
                                    a.type_name(),
                                    b.type_name()
                                ),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }
                Opcode::Or => {
                    // Non-short-circuit Or: both operands already evaluated
                    // Short-circuit is handled by compiler via JumpIfFalse
                    let b = self.pop();
                    let a = self.pop();
                    match (&a, &b) {
                        (Value::Bool(a_val), Value::Bool(b_val)) => {
                            self.push(Value::Bool(*a_val || *b_val));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "Cannot apply || to {} and {}",
                                    a.type_name(),
                                    b.type_name()
                                ),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }

                // ===== Control Flow =====
                Opcode::Jump => {
                    let offset = self.read_i16()?;
                    self.ip = (self.ip as isize + offset as isize) as usize;
                }
                Opcode::JumpIfFalse => {
                    let offset = self.read_i16()?;
                    let condition = self.pop();
                    if !condition.is_truthy() {
                        self.ip = (self.ip as isize + offset as isize) as usize;
                    }
                }
                Opcode::Loop => {
                    let offset = self.read_i16()?;
                    self.ip = (self.ip as isize + offset as isize) as usize;
                }

                // ===== Functions =====
                Opcode::TraitDispatch => {
                    let trait_idx = self.read_u16()? as usize;
                    let method_idx = self.read_u16()? as usize;
                    let arg_count = self.read_u8()? as usize;

                    let trait_name = match self.bytecode.constants.get(trait_idx) {
                        Some(Value::String(s)) => s.as_ref().clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Expected string constant for trait name".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };
                    let method_name = match self.bytecode.constants.get(method_idx) {
                        Some(Value::String(s)) => s.as_ref().clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Expected string constant for method name".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    };

                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(self.pop());
                    }
                    args.reverse();

                    let receiver =
                        args.first()
                            .cloned()
                            .ok_or_else(|| RuntimeError::TypeError {
                                msg: "Trait dispatch requires a receiver".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })?;
                    let dispatch_type = self
                        .struct_name_for_value(&receiver)
                        .unwrap_or_else(|| receiver.type_name());
                    let mangled_name =
                        format!("__impl__{}__{}__{}", dispatch_type, trait_name, method_name);
                    let function = self.globals.get(&mangled_name).cloned().ok_or_else(|| {
                        RuntimeError::TypeError {
                            msg: format!(
                                "Trait method '{}' not found (impl not registered for this type)",
                                method_name
                            ),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }
                    })?;

                    self.push(function);
                    for arg in args {
                        self.push(arg);
                    }
                    self.execute_call(arg_count)?;
                }
                Opcode::Call => {
                    let arg_count = self.read_u8()? as usize;
                    self.execute_call(arg_count)?;
                }

                Opcode::Return => {
                    // Pop the return value from stack (if any)
                    let return_value = if self.stack.is_empty() {
                        Value::Null
                    } else {
                        self.pop()
                    };

                    // Execute deferred blocks in LIFO order before returning
                    let frame_idx = self.frames.len() - 1;
                    while let Some((body_start, body_len)) = self.defer_stacks[frame_idx].pop() {
                        // Save current IP
                        let saved_ip = self.ip;

                        // Execute defer body by setting IP and running until body_end
                        // We use a bounded loop that processes opcodes directly
                        self.ip = body_start;
                        let body_end = body_start + body_len;

                        // Execute opcodes until we reach body_end
                        while self.ip < body_end {
                            let opcode = match self.read_opcode() {
                                Ok(op) => op,
                                Err(_) => break,
                            };

                            // Execute the opcode (simplified dispatch for defer bodies)
                            // This handles common cases; complex defers may need full dispatch
                            match opcode {
                                Opcode::Constant => {
                                    let index = self.read_u16()? as usize;
                                    let value = self.bytecode.constants[index].clone();
                                    self.push(value);
                                }
                                Opcode::GetGlobal => {
                                    let name_index = self.read_u16()? as usize;
                                    if let Value::String(ref name) =
                                        self.bytecode.constants[name_index]
                                    {
                                        if let Some(value) = self.globals.get(name.as_ref()) {
                                            self.push(value.clone());
                                        } else {
                                            self.push(Value::Null);
                                        }
                                    }
                                }
                                Opcode::Call => {
                                    let arg_count = self.read_u8()? as usize;
                                    let frames_before_call = self.frames.len();
                                    self.execute_call(arg_count)?;
                                    // H-364: If execute_call pushed a new frame (user-defined
                                    // function), drive it to completion via the full dispatch
                                    // loop so that Return, arithmetic, GetLocal etc. all work.
                                    // The loop stops when frames.len() drops back to
                                    // frames_before_call, leaving the return value on the stack.
                                    if self.frames.len() > frames_before_call {
                                        self.execute_loop(Some(frames_before_call))?;
                                    }
                                }
                                Opcode::Pop => {
                                    self.pop();
                                }
                                Opcode::GetLocal => {
                                    let index = self.read_u16()? as usize;
                                    let base = self.current_frame().stack_base;
                                    let value = self.stack[base + index].clone();
                                    self.push(value);
                                }
                                _ => {
                                    // For other opcodes, skip their operands
                                    let size = dispatch::operand_size(opcode);
                                    self.ip += size;
                                }
                            }
                        }

                        // Restore IP
                        self.ip = saved_ip;
                    }

                    // Pop the call frame and defer stack
                    let frame = self.frames.pop();
                    self.defer_stacks.pop();
                    #[cfg(debug_assertions)]
                    self.consumed_slots.pop();

                    if let Some(f) = frame {
                        // Clean up the stack (remove locals, arguments, and function value)
                        self.stack.truncate(f.stack_base);
                        #[cfg(debug_assertions)]
                        self.value_origins.truncate(f.stack_base);
                        // Also remove the function value (one slot below stack_base)
                        if f.stack_base > 0 && !self.stack.is_empty() {
                            self.stack.pop();
                            #[cfg(debug_assertions)]
                            self.value_origins.pop();
                        }

                        // Restore IP to return address
                        self.ip = f.return_ip;

                        // Push return value
                        self.push(return_value);
                    } else {
                        // Returning from main - we're done
                        // Push the return value and halt
                        self.push(return_value);
                        break;
                    }
                }

                // ===== Arrays =====
                Opcode::Array => {
                    let size = self.read_u16()? as usize;

                    // Track memory allocation before creating the array
                    self.track_memory(Self::estimate_array_size(size))?;

                    let mut elements = Vec::with_capacity(size);
                    for _ in 0..size {
                        elements.push(self.pop());
                    }
                    elements.reverse(); // Stack is LIFO, so reverse to get correct order
                    self.push(Value::Array(ValueArray::from_vec(elements)));
                }
                // ===== Tuples =====
                Opcode::Tuple => {
                    let size = self.read_u16()? as usize;
                    let mut elements = Vec::with_capacity(size);
                    for _ in 0..size {
                        elements.push(self.pop());
                    }
                    elements.reverse(); // Stack is LIFO, so reverse to get correct order
                    self.push(Value::Tuple(Arc::new(elements)));
                }
                Opcode::TupleGet => {
                    let index = self.read_u16()? as usize;
                    let tuple = self.pop();
                    match tuple {
                        Value::Tuple(elems) => {
                            if index >= elems.len() {
                                return Err(RuntimeError::OutOfBounds {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                            self.push(elems[index].clone());
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "TupleGet applied to non-tuple value".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }
                Opcode::GetIndex => {
                    let index_val = self.pop();
                    let target = self.pop();
                    match target {
                        Value::Array(arr) => {
                            // Array indexing requires number
                            match index_val {
                                Value::Number(index) => {
                                    if index.fract() != 0.0 || index < 0.0 {
                                        return Err(RuntimeError::InvalidIndex {
                                            span: self
                                                .current_span()
                                                .unwrap_or_else(crate::span::Span::dummy),
                                        });
                                    }
                                    let idx = index as usize;
                                    if idx >= arr.len() {
                                        return Err(RuntimeError::OutOfBounds {
                                            span: self
                                                .current_span()
                                                .unwrap_or_else(crate::span::Span::dummy),
                                        });
                                    }
                                    self.push(arr[idx].clone());
                                }
                                Value::Range {
                                    start,
                                    end,
                                    inclusive,
                                } => {
                                    let start = start.unwrap_or(0.0);
                                    let mut end_val = end.unwrap_or(arr.len() as f64);
                                    if inclusive && end.is_some() {
                                        end_val += 1.0;
                                    }
                                    let span = self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy);
                                    let sliced = crate::stdlib::array::slice(
                                        arr.as_slice(),
                                        start,
                                        end_val,
                                        span,
                                    )?;
                                    self.push(sliced);
                                }
                                _ => {
                                    return Err(RuntimeError::InvalidIndex {
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }
                            }
                        }
                        Value::String(s) => {
                            // String indexing by character position (Unicode-aware)
                            if let Value::Number(index) = index_val {
                                if index.fract() != 0.0 || index < 0.0 {
                                    return Err(RuntimeError::InvalidIndex {
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }
                                let idx = index as usize;
                                let chars: Vec<char> = s.chars().collect();
                                if idx >= chars.len() {
                                    return Err(RuntimeError::OutOfBounds {
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }
                                self.push(Value::string(chars[idx].to_string()));
                            } else {
                                return Err(RuntimeError::InvalidIndex {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                        }
                        Value::JsonValue(json) => {
                            // JSON indexing accepts string or number
                            let result = match index_val {
                                Value::String(key) => json.index_str(key.as_ref()),
                                Value::Number(n) => json.index_num(n),
                                _ => {
                                    return Err(RuntimeError::TypeError {
                                        msg: "JSON index must be string or number".to_string(),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    })
                                }
                            };
                            self.push(Value::JsonValue(Arc::new(result)));
                        }
                        // H-116: range as for-in target — index i yields start + i
                        Value::Range { start, .. } => {
                            if let Value::Number(idx) = index_val {
                                let s = start.unwrap_or(0.0);
                                self.push(Value::Number(s + idx));
                            } else {
                                return Err(RuntimeError::InvalidIndex {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot index non-array/string/json".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                }
                Opcode::SetIndex => {
                    let value = self.pop();
                    let index = self.pop_number()?;
                    let mut array = self.pop();
                    match &mut array {
                        Value::Array(arr) => {
                            if index.fract() != 0.0 || index < 0.0 {
                                return Err(RuntimeError::InvalidIndex {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                            let idx = index as usize;
                            if idx >= arr.len() {
                                return Err(RuntimeError::OutOfBounds {
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                            // CoW: set triggers Arc::make_mut if arr is shared
                            arr.set(idx, value);
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot index non-array".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            })
                        }
                    }
                    // Push the mutated array back — compiler emits SetLocal/SetGlobal to write it
                    // back to the variable, then Pop removes it from the expression stack.
                    self.push(array);
                }
                Opcode::GetField => {
                    let key_val = self.pop();
                    let map_val = self.pop();
                    let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
                    let key =
                        crate::stdlib::collections::hash::HashKey::from_value(&key_val, span)?;

                    match map_val {
                        Value::Map(map) => match map.get(&key).cloned() {
                            Some(value) => self.push(value),
                            None => {
                                let field = match key_val {
                                    Value::String(s) => s.as_ref().to_string(),
                                    other => other.type_name().to_string(),
                                };
                                return Err(RuntimeError::TypeError {
                                    msg: format!("Missing field '{}'", field),
                                    span,
                                });
                            }
                        },
                        other => {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "Cannot access field on non-record type {}",
                                    other.type_name()
                                ),
                                span,
                            })
                        }
                    }
                }
                Opcode::SetField => {
                    let value = self.pop();
                    let key_val = self.pop();
                    let mut map_val = self.pop();
                    let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
                    let key =
                        crate::stdlib::collections::hash::HashKey::from_value(&key_val, span)?;
                    let field_name = match &key_val {
                        Value::String(s) => s.as_ref().to_string(),
                        other => other.type_name().to_string(),
                    };

                    match &mut map_val {
                        Value::Map(map) => {
                            let existing =
                                map.get(&key)
                                    .cloned()
                                    .ok_or_else(|| RuntimeError::TypeError {
                                        msg: format!("Missing field '{}'", field_name),
                                        span,
                                    })?;
                            if existing.type_name() != value.type_name() {
                                return Err(RuntimeError::TypeError {
                                    msg: format!(
                                        "Type mismatch for field '{}': expected {}, found {}",
                                        field_name,
                                        existing.type_name(),
                                        value.type_name()
                                    ),
                                    span,
                                });
                            }
                            map.insert(key, value);
                        }
                        other => {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "Cannot assign field on non-record type {}",
                                    other.type_name()
                                ),
                                span,
                            })
                        }
                    }
                    // Push the mutated map back — compiler emits SetLocal/SetGlobal to write it back
                    self.push(map_val);
                }
                Opcode::Slice => {
                    let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
                    let end = match self.pop() {
                        Value::Number(n) => n,
                        _ => return Err(RuntimeError::InvalidIndex { span }),
                    };
                    let start = match self.pop() {
                        Value::Number(n) => n,
                        _ => return Err(RuntimeError::InvalidIndex { span }),
                    };
                    let target = self.pop();
                    match target {
                        Value::Array(arr) => {
                            let sliced =
                                crate::stdlib::array::slice(arr.as_slice(), start, end, span)?;
                            self.push(sliced);
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot slice non-array".to_string(),
                                span,
                            })
                        }
                    }
                }
                Opcode::SliceFrom => {
                    let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
                    let start = match self.pop() {
                        Value::Number(n) => n,
                        _ => return Err(RuntimeError::InvalidIndex { span }),
                    };
                    let target = self.pop();
                    match target {
                        Value::Array(arr) => {
                            let end = arr.len() as f64;
                            let sliced =
                                crate::stdlib::array::slice(arr.as_slice(), start, end, span)?;
                            self.push(sliced);
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot slice non-array".to_string(),
                                span,
                            })
                        }
                    }
                }
                Opcode::SliceTo => {
                    let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
                    let end = match self.pop() {
                        Value::Number(n) => n,
                        _ => return Err(RuntimeError::InvalidIndex { span }),
                    };
                    let target = self.pop();
                    match target {
                        Value::Array(arr) => {
                            let sliced =
                                crate::stdlib::array::slice(arr.as_slice(), 0.0, end, span)?;
                            self.push(sliced);
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot slice non-array".to_string(),
                                span,
                            })
                        }
                    }
                }
                Opcode::SliceFull => {
                    let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
                    let target = self.pop();
                    match target {
                        Value::Array(arr) => {
                            let sliced = crate::stdlib::array::slice(
                                arr.as_slice(),
                                0.0,
                                arr.len() as f64,
                                span,
                            )?;
                            self.push(sliced);
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot slice non-array".to_string(),
                                span,
                            })
                        }
                    }
                }

                Opcode::Range => {
                    let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);
                    let inclusive = self.read_u8()? != 0;
                    let end_val = self.pop();
                    let start_val = self.pop();

                    let start = match start_val {
                        Value::Null => None,
                        Value::Number(n) => Some(n),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Range bound must be number".to_string(),
                                span,
                            })
                        }
                    };

                    let end = match end_val {
                        Value::Null => None,
                        Value::Number(n) => Some(n),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Range bound must be number".to_string(),
                                span,
                            })
                        }
                    };

                    if inclusive && end.is_none() {
                        return Err(RuntimeError::TypeError {
                            msg: "Inclusive range requires an end bound".to_string(),
                            span,
                        });
                    }

                    self.push(Value::Range {
                        start,
                        end,
                        inclusive,
                    });
                }

                Opcode::HashMap => {
                    use crate::stdlib::collections::hash::HashKey;
                    use crate::stdlib::collections::hashmap::AtlasHashMap;
                    use crate::value::ValueHashMap;

                    let entry_count = self.read_u16()? as usize;

                    // Stack has [key1, val1, key2, val2, ...] in order
                    // Pop them in reverse (LIFO) and insert
                    let mut entries = Vec::with_capacity(entry_count);
                    for _ in 0..entry_count {
                        let value = self.pop();
                        let key_val = self.pop();
                        entries.push((key_val, value));
                    }
                    // Reverse to get original order
                    entries.reverse();

                    let mut atlas_map = AtlasHashMap::with_capacity(entry_count);
                    for (key_val, value) in entries {
                        // Convert Value to HashKey (keys must be hashable)
                        let key = HashKey::from_value(
                            &key_val,
                            self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        )?;
                        atlas_map.insert(key, value);
                    }

                    self.push(Value::Map(ValueHashMap::from_atlas(atlas_map)));
                }
                Opcode::Struct => {
                    use crate::stdlib::collections::hash::HashKey;
                    use crate::stdlib::collections::hashmap::AtlasHashMap;
                    use crate::value::ValueHashMap;

                    let name_idx = self.read_u16()? as usize;
                    let field_count = self.read_u16()? as usize;
                    let struct_name = match self.bytecode.constants.get(name_idx) {
                        Some(Value::String(s)) => s.as_ref().clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Expected string constant for struct name".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    };

                    let mut entries = Vec::with_capacity(field_count);
                    for _ in 0..field_count {
                        let value = self.pop();
                        let key_val = self.pop();
                        entries.push((key_val, value));
                    }
                    entries.reverse();

                    let mut atlas_map = AtlasHashMap::with_capacity(field_count);
                    for (key_val, value) in entries {
                        let key = HashKey::from_value(
                            &key_val,
                            self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        )?;
                        atlas_map.insert(key, value);
                    }

                    let map = ValueHashMap::from_atlas(atlas_map);
                    self.register_struct_type(&map, &struct_name);
                    self.push(Value::Map(map));
                }

                // ===== Stack Manipulation =====
                Opcode::Pop => {
                    // Don't pop if this is the last instruction before Halt
                    // Check if next instruction is Halt
                    if self.ip < self.bytecode.instructions.len()
                        && self.bytecode.instructions[self.ip] != Opcode::Halt as u8
                    {
                        self.pop();
                    }
                }
                Opcode::Dup => {
                    let value = self.peek(0).clone();
                    self.push(value);
                }

                Opcode::Dup2 => {
                    // Duplicate top 2 stack values: [a, b] -> [a, b, a, b]
                    let b = self.peek(0).clone();
                    let a = self.peek(1).clone();
                    self.push(a);
                    self.push(b);
                }

                Opcode::Rot3 => {
                    // Rotate top 3 stack values: [a, b, c] -> [b, c, a]
                    // Pop c, b, a then push b, c, a
                    let c = self.pop();
                    let b = self.pop();
                    let a = self.pop();
                    self.push(b);
                    self.push(c);
                    self.push(a);
                }
                Opcode::ToString => {
                    let value = self.pop();
                    let span = self.current_span().unwrap_or_else(Span::dummy);
                    let string_value = crate::stdlib::types::to_string(&[value], span)?;
                    self.push(string_value);
                }

                // ===== Pattern Matching =====
                Opcode::IsOptionSome => {
                    let value = self.pop();
                    let is_some = matches!(value, Value::Option(Some(_)));
                    self.push(Value::Bool(is_some));
                }
                Opcode::IsOptionNone => {
                    let value = self.pop();
                    let is_none = matches!(value, Value::Option(None));
                    self.push(Value::Bool(is_none));
                }
                Opcode::IsResultOk => {
                    let value = self.pop();
                    let is_ok = matches!(value, Value::Result(Ok(_)));
                    self.push(Value::Bool(is_ok));
                }
                Opcode::IsResultErr => {
                    let value = self.pop();
                    let is_err = matches!(value, Value::Result(Err(_)));
                    self.push(Value::Bool(is_err));
                }
                Opcode::ExtractOptionValue => {
                    let value = self.pop();
                    match value {
                        Value::Option(Some(inner)) => self.push(*inner),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "ExtractOptionValue requires Option::Some".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    }
                }
                Opcode::ExtractResultValue => {
                    let value = self.pop();
                    match value {
                        Value::Result(Ok(inner)) => self.push(*inner),
                        Value::Result(Err(inner)) => self.push(*inner),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "ExtractResultValue requires Result".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    }
                }
                Opcode::IsArray => {
                    let value = self.pop();
                    let is_array = matches!(value, Value::Array(_));
                    self.push(Value::Bool(is_array));
                }
                Opcode::GetArrayLen => {
                    let value = self.pop();
                    match value {
                        Value::Array(arr) => {
                            let len = arr.len();
                            self.push(Value::Number(len as f64));
                        }
                        // H-116: range in for-in — compute length and push only length
                        Value::Range {
                            start,
                            end,
                            inclusive,
                        } => {
                            let s = start.unwrap_or(0.0) as i64;
                            let e = end.ok_or_else(|| RuntimeError::TypeError {
                                msg: "for-in range requires an end bound".to_string(),
                                span: Span::dummy(),
                            })? as i64;
                            let len = if inclusive {
                                (e - s + 1).max(0) as f64
                            } else {
                                (e - s).max(0) as f64
                            };
                            self.push(Value::Number(len));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "GetArrayLen requires Array".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    }
                }
                Opcode::EnumVariant => {
                    let arg_count = self.read_u8()? as usize;

                    // Pop args in reverse order
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(self.pop());
                    }
                    args.reverse();

                    // Pop variant name and enum name
                    let variant_name = self.pop();
                    let enum_name = self.pop();

                    // Extract string values
                    let enum_name_str = match enum_name {
                        Value::String(s) => (*s).clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "EnumVariant requires string enum name".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    };
                    let variant_name_str = match variant_name {
                        Value::String(s) => (*s).clone(),
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "EnumVariant requires string variant name".to_string(),
                                span: Span::dummy(),
                            })
                        }
                    };

                    // Create the enum value
                    self.push(Value::EnumValue {
                        enum_name: enum_name_str,
                        variant_name: variant_name_str,
                        data: args,
                    });
                }

                Opcode::CheckEnumVariant => {
                    // Stack: [value, enum_name, variant_name] -> [bool]
                    let variant_name = self.pop();
                    let enum_name = self.pop();
                    let value = self.pop();

                    // Extract expected names
                    let expected_enum = match &enum_name {
                        Value::String(s) => s.as_str(),
                        _ => {
                            self.push(Value::Bool(false));
                            continue;
                        }
                    };
                    let expected_variant = match &variant_name {
                        Value::String(s) => s.as_str(),
                        _ => {
                            self.push(Value::Bool(false));
                            continue;
                        }
                    };

                    // Check if value matches.
                    // H-223: empty expected_enum means bare variant pattern — skip enum_name check.
                    let matches = match &value {
                        Value::EnumValue {
                            enum_name: val_enum,
                            variant_name: val_variant,
                            ..
                        } => {
                            let enum_ok = expected_enum.is_empty() || val_enum == expected_enum;
                            enum_ok && val_variant == expected_variant
                        }
                        _ => false,
                    };

                    self.push(Value::Bool(matches));
                }

                Opcode::ExtractEnumData => {
                    // Stack: [EnumValue] -> [Array]
                    let value = self.pop();

                    match value {
                        Value::EnumValue { data, .. } => {
                            // Convert data Vec<Value> to an Array
                            self.push(Value::array(data));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "ExtractEnumData requires EnumValue, got {}",
                                    value.type_name()
                                ),
                                span: Span::dummy(),
                            });
                        }
                    }
                }

                // ===== Async (Phase 10) =====
                //
                // Encoding:
                //   AsyncCall / SpawnTask: u8 arg_count  (same layout as Call)
                //   Await / WrapFuture:    no operands
                //
                // The compiler emits WrapFuture *inside* async fn bodies so that the
                // return value is always a Value::Future.  AsyncCall therefore only needs
                // to dispatch the call normally — the callee's WrapFuture handles wrapping.
                //
                // SpawnTask mirrors AsyncCall at this stage (eager execution, same as the
                // interpreter).  True tokio::spawn concurrency requires an independent VM
                // instance per task and is deferred to Phase 11 (stdlib async I/O).
                Opcode::AsyncCall | Opcode::SpawnTask => {
                    let arg_count = self.read_u8()? as usize;
                    self.execute_call(arg_count)?;
                    // Result is already Value::Future — the callee's WrapFuture emitted it.
                }
                Opcode::Await => {
                    let val = self.pop();
                    match val {
                        Value::Future(future) => match future.get_state() {
                            crate::async_runtime::FutureState::Resolved(v) => {
                                self.push(v);
                            }
                            crate::async_runtime::FutureState::Rejected(e) => {
                                return Err(RuntimeError::TypeError {
                                    msg: format!("Awaited future was rejected: {}", e),
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                            crate::async_runtime::FutureState::Pending => {
                                return Err(RuntimeError::TypeError {
                                        msg: "Cannot await a pending future in the VM synchronous execute loop".to_string(),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                            }
                        },
                        other => {
                            return Err(RuntimeError::TypeError {
                                msg: format!(
                                    "AT4002: await operand must be Future, got {}",
                                    other.type_name()
                                ),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }
                Opcode::WrapFuture => {
                    let val = self.pop();
                    let future = crate::async_runtime::AtlasFuture::resolved(val);
                    self.push(Value::Future(Arc::new(future)));
                }

                // ===== Defer =====
                Opcode::DeferPush => {
                    // Read the body length
                    let body_len = self.read_u16()? as usize;
                    // Body starts after the Jump opcode (1 byte) + operand (2 bytes) = 3 bytes
                    // The compiler emits: DeferPush body_len Jump offset body...
                    let body_start = self.ip + 3;

                    // Record defer: (body_start, body_len) on the current frame's defer stack
                    let frame_idx = self.frames.len() - 1;
                    self.defer_stacks[frame_idx].push((body_start, body_len));

                    // Normal execution continues (Jump instruction follows to skip body)
                }
                Opcode::DeferExec => {
                    // Execute all deferred blocks for current frame in LIFO order
                    let frame_idx = self.frames.len() - 1;
                    while let Some((body_start, body_len)) = self.defer_stacks[frame_idx].pop() {
                        // Save current IP
                        let saved_ip = self.ip;

                        // Execute defer body
                        self.ip = body_start;
                        let body_end = body_start + body_len;
                        while self.ip < body_end {
                            let defer_opcode =
                                Opcode::try_from(self.bytecode.instructions[self.ip]).map_err(
                                    |_| RuntimeError::UnknownOpcode {
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    },
                                )?;
                            self.ip += 1;

                            // Execute single instruction (simplified - only handles basic ops)
                            // Return in defer body returns to defer executor, not function
                            if matches!(defer_opcode, Opcode::Return) {
                                break;
                            }
                            // For complex defer bodies, we'd need recursive execution
                            // For now, just break - the main loop will handle it
                        }

                        // Restore IP
                        self.ip = saved_ip;
                    }
                }

                // ===== Special =====
                Opcode::Halt => break,
            }
        }

        // Return top of stack if present
        Ok(if self.stack.is_empty() {
            None
        } else {
            Some(self.pop())
        })
    }

    // ===== Helper Methods =====

    #[inline(always)]
    fn push(&mut self, value: Value) {
        self.stack.push(value);
        #[cfg(debug_assertions)]
        self.value_origins.push(None);
    }

    #[inline(always)]
    fn pop(&mut self) -> Value {
        #[cfg(debug_assertions)]
        self.value_origins.pop();
        // SAFETY: VM invariants guarantee the stack is non-empty when pop() is called.
        // Preconditions: every opcode that calls pop() has already pushed its operands,
        // and debug builds mirror this via value_origins.
        unsafe { self.stack.pop().unwrap_unchecked() }
    }

    #[inline(always)]
    fn peek(&self, distance: usize) -> &Value {
        // SAFETY: The compiler guarantees stack depth matches operand requirements.
        // Preconditions: each opcode that calls peek() is emitted only when sufficient
        // values exist, and `distance` is within the current stack depth.
        unsafe { self.stack.get_unchecked(self.stack.len() - 1 - distance) }
    }

    #[inline(always)]
    fn pop_number(&mut self) -> Result<f64, RuntimeError> {
        match self.pop() {
            Value::Number(n) => Ok(n),
            _ => Err(RuntimeError::TypeError {
                msg: "Expected number".to_string(),
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            }),
        }
    }

    /// Find the end of a function body (the Return opcode)
    ///
    /// Scans bytecode from `start` looking for a Return opcode. Returns the
    /// offset just past the Return instruction. Used by JIT to determine
    /// function boundaries.
    fn find_function_end(&self, start: usize) -> usize {
        let instructions = &self.bytecode.instructions;
        let mut ip = start;

        while ip < instructions.len() {
            let byte = instructions[ip];
            if let Ok(opcode) = Opcode::try_from(byte) {
                ip += 1;
                match opcode {
                    Opcode::Return | Opcode::Halt => return ip,
                    // Skip operand bytes for opcodes that have them
                    Opcode::Constant
                    | Opcode::GetLocal
                    | Opcode::SetLocal
                    | Opcode::GetGlobal
                    | Opcode::SetGlobal
                    | Opcode::GetUpvalue
                    | Opcode::SetUpvalue
                    | Opcode::Jump
                    | Opcode::JumpIfFalse
                    | Opcode::Loop
                    | Opcode::Array
                    | Opcode::Tuple
                    | Opcode::TupleGet
                    | Opcode::HashMap
                    | Opcode::DeferPush => ip += 2,
                    Opcode::Struct => ip += 4,
                    Opcode::MakeClosure => ip += 4,
                    Opcode::Call => ip += 1,
                    Opcode::TraitDispatch => ip += 5,
                    _ => {}
                }
            } else {
                ip += 1;
            }
        }

        // If no Return found, return end of bytecode
        instructions.len()
    }

    #[inline(always)]
    fn binary_numeric_op<F>(&mut self, op: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        let result = op(a, b);
        if result.is_nan() || result.is_infinite() {
            return Err(RuntimeError::InvalidNumericResult {
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            });
        }
        self.push(Value::Number(result));
        Ok(())
    }

    #[inline(always)]
    fn read_opcode(&mut self) -> Result<Opcode, RuntimeError> {
        if self.ip >= self.bytecode.instructions.len() {
            return Err(RuntimeError::UnknownOpcode {
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            });
        }
        // SAFETY: Bounds check above guarantees self.ip < len
        let byte = self.bytecode.instructions[self.ip];
        self.ip += 1;
        // Use static dispatch table for O(1) opcode lookup
        dispatch::decode_opcode(byte).ok_or_else(|| RuntimeError::UnknownOpcode {
            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
        })
    }

    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8, RuntimeError> {
        if self.ip >= self.bytecode.instructions.len() {
            return Err(RuntimeError::UnknownOpcode {
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            });
        }
        let byte = self.bytecode.instructions[self.ip];
        self.ip += 1;
        Ok(byte)
    }

    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16, RuntimeError> {
        if self.ip + 1 >= self.bytecode.instructions.len() {
            return Err(RuntimeError::UnknownOpcode {
                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
            });
        }
        let hi = self.bytecode.instructions[self.ip] as u16;
        let lo = self.bytecode.instructions[self.ip + 1] as u16;
        self.ip += 2;
        Ok((hi << 8) | lo)
    }

    #[inline(always)]
    fn read_i16(&mut self) -> Result<i16, RuntimeError> {
        Ok(self.read_u16()? as i16)
    }

    #[inline(always)]
    fn current_frame(&self) -> &CallFrame {
        // SAFETY: VM execution always pushes a frame before running, and frames are
        // only popped when returning. The frames vec is never empty while executing.
        unsafe { self.frames.last().unwrap_unchecked() }
    }

    /// Generate a stack trace from the current call frames.
    /// Returns frames from innermost to outermost.
    pub fn stack_trace(&self, error_span: crate::span::Span) -> Vec<StackTraceFrame> {
        let include_main = self.frames.len() == 1;
        let mut frames = Vec::new();
        for frame in self.frames.iter().rev() {
            if !include_main && frame.function_name == "<main>" {
                continue;
            }
            let span = if frames.is_empty() {
                error_span
            } else {
                self.span_for_offset(frame.return_ip.saturating_sub(1))
                    .unwrap_or_else(crate::span::Span::dummy)
            };
            frames.push(crate::stack_trace::stack_frame_from_span(
                frame.function_name.clone(),
                span,
                None,
            ));
        }
        frames
    }

    fn execute_call(&mut self, arg_count: usize) -> Result<(), RuntimeError> {
        let function = self.peek(arg_count).clone();

        match function {
            Value::Builtin(ref name) => {
                // Check array intrinsics first (callback-based)
                if self.is_array_intrinsic(name) {
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(self.pop());
                    }
                    args.reverse();
                    self.pop(); // Pop function value

                    let result = self.call_array_intrinsic(name, &args)?;
                    self.push(result);
                } else {
                    // Stdlib builtin - call directly
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(self.pop());
                    }
                    args.reverse();
                    self.pop(); // Pop function value

                    let security = self.current_security.as_ref().ok_or_else(|| {
                        RuntimeError::InternalError {
                            msg: "Security context not set".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        }
                    })?;
                    let result = crate::stdlib::call_builtin(
                        name,
                        &args,
                        self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        security,
                        &self.output_writer,
                    )?;

                    self.push(result);
                }
            }
            Value::Function(func) => {
                // Check for extern functions
                if let Some(extern_fn) = self.extern_functions.get(&func.name).cloned() {
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(self.pop());
                    }
                    args.reverse();
                    self.pop(); // Pop function value

                    // SAFETY: `extern_fn` was constructed with a signature derived
                    // from the AST extern declaration, and `args` is built from the
                    // VM stack to match that arity. The library remains loaded for
                    // the duration of the call.
                    let result =
                        unsafe { extern_fn.call(&args) }.map_err(|e| RuntimeError::TypeError {
                            msg: format!("FFI call error: {}", e),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        })?;

                    self.push(result);
                } else {
                    // User-defined function
                    // Safety check: compiled functions always have bytecode_offset > 0
                    // because the compiler emits setup code (Constant, SetGlobal, Pop, Jump)
                    // before the function body. bytecode_offset == 0 indicates an
                    // interpreter-created function that has no bytecode.
                    if func.bytecode_offset == 0 {
                        return Err(RuntimeError::TypeError {
                            msg: format!(
                                "Cannot call function '{}' from VM: function was created \
                                 by interpreter and has no compiled bytecode. This typically \
                                 happens when importing functions across execution modes.",
                                func.name
                            ),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }

                    // Try JIT execution for hot numeric functions
                    if self.jit.is_some() {
                        // Gather info before borrowing jit mutably
                        let args_base = self.stack.len() - arg_count;
                        let mut numeric_args: Vec<f64> = Vec::with_capacity(arg_count);
                        let mut all_numeric = true;

                        for i in 0..arg_count {
                            match &self.stack[args_base + i] {
                                Value::Number(n) => numeric_args.push(*n),
                                _ => {
                                    all_numeric = false;
                                    break;
                                }
                            }
                        }

                        if all_numeric {
                            let function_end = self.find_function_end(func.bytecode_offset);
                            let bytecode_offset = func.bytecode_offset;

                            // Now borrow jit mutably for the call
                            if let Some(ref mut jit) = self.jit {
                                if let Some(result) = jit.try_execute(
                                    &self.bytecode,
                                    bytecode_offset,
                                    function_end,
                                    &numeric_args,
                                ) {
                                    // JIT succeeded — pop args and function, push result
                                    for _ in 0..arg_count {
                                        self.pop();
                                    }
                                    self.pop(); // Pop function value
                                    self.push(Value::Number(result));
                                    return Ok(());
                                }
                            }
                        }
                    }

                    // B41-P04: If function has a rest param, collect extra args into an array.
                    // The rest param occupies the last slot (index arity-1).
                    // All args from index (arity-1) onward are collected into one Array value.
                    let arg_count = if func.has_rest_param && arg_count >= func.required_arity {
                        let fixed_count = func.arity.saturating_sub(1); // params before rest
                        let rest_count = arg_count.saturating_sub(fixed_count);
                        // Collect rest args from top of stack (they're in order, rest_count items)
                        let stack_top = self.stack.len();
                        let rest_start = stack_top - rest_count;
                        let rest_args: Vec<Value> = self.stack.drain(rest_start..).collect();
                        let rest_array = Value::Array(crate::value::ValueArray::from(rest_args));
                        self.stack.push(rest_array);
                        // arg_count is now fixed_count + 1 (the array)
                        fixed_count + 1
                    } else {
                        arg_count
                    };

                    // Create a new call frame
                    let frame = CallFrame {
                        function_name: func.name.clone(),
                        return_ip: self.ip,
                        stack_base: self.stack.len() - arg_count, // Points to first argument
                        local_count: func.local_count, // Use total locals, not just arity
                        upvalues: std::sync::Arc::new(Vec::new()),
                    };

                    // Verify argument count matches (B39-P05: default params; B41-P04: rest params)
                    let arity_ok = if func.has_rest_param {
                        arg_count >= func.required_arity
                    } else {
                        arg_count >= func.required_arity && arg_count <= func.arity
                    };
                    if !arity_ok {
                        let expected = if func.required_arity == func.arity {
                            format!("{}", func.arity)
                        } else {
                            format!("{}-{}", func.required_arity, func.arity)
                        };
                        return Err(RuntimeError::TypeError {
                            msg: format!(
                                "Function {} expects {} arguments, got {}",
                                func.name, expected, arg_count
                            ),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        });
                    }

                    // Fill in default values for missing arguments (B39-P05)
                    for i in arg_count..func.arity {
                        if let Some(Some(default_val)) = func.defaults.get(i) {
                            self.stack.push(default_val.clone());
                        } else {
                            // Should not happen if binder validation is correct
                            self.stack.push(Value::Null);
                        }
                    }

                    // Debug mode: for each `own` parameter, mark the caller's
                    // local slot or global name as consumed so subsequent reads
                    // produce a runtime error.
                    #[cfg(debug_assertions)]
                    {
                        let caller_frame_idx = self.frames.len() - 1;
                        // Use func.arity (not arg_count) because defaults have been pushed
                        let args_base = self.stack.len() - func.arity;
                        for (i, ownership) in func.param_ownership.iter().enumerate() {
                            if *ownership == Some(crate::ast::OwnershipAnnotation::Own) {
                                if let Some(Some(origin)) =
                                    self.value_origins.get(args_base + i).cloned()
                                {
                                    match origin {
                                        StackValueOrigin::Local(slot) => {
                                            if let Some(consumed) = self
                                                .consumed_slots
                                                .get_mut(caller_frame_idx)
                                                .and_then(|v| v.get_mut(slot))
                                            {
                                                *consumed = true;
                                            }
                                        }
                                        StackValueOrigin::Global(name) => {
                                            self.consumed_globals.insert(name);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Debug mode: enforce `share` parameter ownership contracts.
                    #[cfg(debug_assertions)]
                    {
                        // Use func.arity (not arg_count) because defaults have been pushed
                        let args_base = self.stack.len() - func.arity;
                        for (i, ownership) in func.param_ownership.iter().enumerate() {
                            let arg = &self.stack[args_base + i];
                            match ownership {
                                Some(crate::ast::OwnershipAnnotation::Share) => {
                                    if !matches!(arg, Value::SharedValue(_)) {
                                        return Err(RuntimeError::TypeError {
                                            msg: format!(
                                                "ownership violation: parameter '{}' expects share<T> but received {}",
                                                func.param_names
                                                    .get(i)
                                                    .map(|s| s.as_str())
                                                    .unwrap_or("?"),
                                                arg.type_name()
                                            ),
                                            span: self
                                                .current_span()
                                                .unwrap_or_else(crate::span::Span::dummy),
                                        });
                                    }
                                }
                                Some(crate::ast::OwnershipAnnotation::Own)
                                | Some(crate::ast::OwnershipAnnotation::Borrow) => {
                                    if matches!(arg, Value::SharedValue(_)) {
                                        let ann_str = match ownership {
                                            Some(crate::ast::OwnershipAnnotation::Own) => "own",
                                            Some(crate::ast::OwnershipAnnotation::Borrow) => {
                                                "borrow"
                                            }
                                            _ => unreachable!(),
                                        };
                                        self.runtime_warnings.push(
                                            crate::diagnostic::error_codes::SHARE_PASSED_TO_NON_SHARE
                                                .emit(crate::span::Span::new(0, 0))
                                                .arg("inner", "T")
                                                .arg("annotation", ann_str)
                                                .arg("name", func.param_names.get(i).map(|s| s.as_str()).unwrap_or("?"))
                                                .build(),
                                        );
                                    }
                                }
                                None => {}
                            }
                        }
                    }

                    // Push the frame (and its consumed-slot tracking vector)
                    self.frames.push(frame);
                    self.defer_stacks.push(Vec::new());
                    #[cfg(debug_assertions)]
                    self.consumed_slots.push(vec![false; func.local_count]);
                    // Record function call in profiler
                    if let Some(ref mut profiler) = self.profiler {
                        if profiler.is_enabled() {
                            profiler.record_function_call(&func.name);
                        }
                    }

                    // Jump to function bytecode
                    self.ip = func.bytecode_offset;
                }
            }
            Value::Closure(closure) => {
                // Closure call: same as Function but passes upvalues to the frame
                let func = closure.func.clone();
                let upvalues = closure.upvalues.clone();

                if func.bytecode_offset == 0 {
                    return Err(RuntimeError::TypeError {
                        msg: format!(
                            "Cannot call closure '{}' from VM: no compiled bytecode.",
                            func.name
                        ),
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                }

                // B41-P04: Collect rest args into array for variadic closures
                let arg_count = if func.has_rest_param && arg_count >= func.required_arity {
                    let fixed_count = func.arity.saturating_sub(1);
                    let rest_count = arg_count.saturating_sub(fixed_count);
                    let stack_top = self.stack.len();
                    let rest_start = stack_top - rest_count;
                    let rest_args: Vec<Value> = self.stack.drain(rest_start..).collect();
                    let rest_array = Value::Array(crate::value::ValueArray::from(rest_args));
                    self.stack.push(rest_array);
                    fixed_count + 1
                } else {
                    arg_count
                };

                // Verify argument count matches (B39-P05: default params; B41-P04: rest params)
                let arity_ok = if func.has_rest_param {
                    arg_count >= func.required_arity
                } else {
                    arg_count >= func.required_arity && arg_count <= func.arity
                };
                if !arity_ok {
                    let expected = if func.required_arity == func.arity {
                        format!("{}", func.arity)
                    } else {
                        format!("{}-{}", func.required_arity, func.arity)
                    };
                    return Err(RuntimeError::TypeError {
                        msg: format!(
                            "Function {} expects {} arguments, got {}",
                            func.name, expected, arg_count
                        ),
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
                }

                // Fill in default values for missing arguments (B39-P05)
                for i in arg_count..func.arity {
                    if let Some(Some(default_val)) = func.defaults.get(i) {
                        self.stack.push(default_val.clone());
                    } else {
                        self.stack.push(Value::Null);
                    }
                }

                // Debug mode: mark own-parameter locals/globals consumed in caller.
                #[cfg(debug_assertions)]
                {
                    let caller_frame_idx = self.frames.len() - 1;
                    let args_base = self.stack.len() - arg_count;
                    for (i, ownership) in func.param_ownership.iter().enumerate() {
                        if *ownership == Some(crate::ast::OwnershipAnnotation::Own) {
                            if let Some(Some(origin)) =
                                self.value_origins.get(args_base + i).cloned()
                            {
                                match origin {
                                    StackValueOrigin::Local(slot) => {
                                        if let Some(consumed) = self
                                            .consumed_slots
                                            .get_mut(caller_frame_idx)
                                            .and_then(|v| v.get_mut(slot))
                                        {
                                            *consumed = true;
                                        }
                                    }
                                    StackValueOrigin::Global(name) => {
                                        self.consumed_globals.insert(name);
                                    }
                                }
                            }
                        }
                    }
                }

                // Debug mode: enforce `share` parameter ownership contracts.
                #[cfg(debug_assertions)]
                {
                    let args_base = self.stack.len() - arg_count;
                    for (i, ownership) in func.param_ownership.iter().enumerate() {
                        let arg = &self.stack[args_base + i];
                        match ownership {
                            Some(crate::ast::OwnershipAnnotation::Share) => {
                                if !matches!(arg, Value::SharedValue(_)) {
                                    return Err(RuntimeError::TypeError {
                                        msg: format!(
                                            "ownership violation: parameter '{}' expects share<T> but received {}",
                                            func.param_names
                                                .get(i)
                                                .map(|s| s.as_str())
                                                .unwrap_or("?"),
                                            arg.type_name()
                                        ),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }
                            }
                            Some(crate::ast::OwnershipAnnotation::Own)
                            | Some(crate::ast::OwnershipAnnotation::Borrow) => {
                                if matches!(arg, Value::SharedValue(_)) {
                                    let ann_str = match ownership {
                                        Some(crate::ast::OwnershipAnnotation::Own) => "own",
                                        Some(crate::ast::OwnershipAnnotation::Borrow) => "borrow",
                                        _ => unreachable!(),
                                    };
                                    self.runtime_warnings.push(
                                        crate::diagnostic::error_codes::SHARE_PASSED_TO_NON_SHARE
                                            .emit(crate::span::Span::new(0, 0))
                                            .arg("inner", "T")
                                            .arg("annotation", ann_str)
                                            .arg(
                                                "name",
                                                func.param_names
                                                    .get(i)
                                                    .map(|s| s.as_str())
                                                    .unwrap_or("?"),
                                            )
                                            .build(),
                                    );
                                }
                            }
                            None => {}
                        }
                    }
                }

                // Create a new call frame with upvalues
                let frame = CallFrame {
                    function_name: func.name.clone(),
                    return_ip: self.ip,
                    stack_base: self.stack.len() - arg_count,
                    local_count: func.local_count,
                    upvalues,
                };

                // Push the frame (and its consumed-slot tracking vector)
                self.frames.push(frame);
                self.defer_stacks.push(Vec::new());
                #[cfg(debug_assertions)]
                self.consumed_slots.push(vec![false; func.local_count]);
                if let Some(ref mut profiler) = self.profiler {
                    if profiler.is_enabled() {
                        profiler.record_function_call(&func.name);
                    }
                }

                // Jump to function bytecode
                self.ip = func.bytecode_offset;
            }
            Value::NativeFunction(native_fn) => {
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.pop());
                }
                args.reverse();
                self.pop(); // Pop function value
                let result = native_fn(&args)?;
                self.push(result);
            }
            Value::Option(None) if arg_count == 0 => {
                // None() is a valid call that returns Option::None (zero-arg constructor)
                self.pop(); // Pop function value
                self.push(Value::Option(None));
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Cannot call non-function type {}", function.type_name()),
                    span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                });
            }
        }

        Ok(())
    }

    // ========================================================================
    // Array Intrinsics (Callback-based operations)
    // ========================================================================

    fn is_array_intrinsic(&self, name: &str) -> bool {
        crate::stdlib::is_array_intrinsic(name)
    }

    fn call_array_intrinsic(&mut self, name: &str, args: &[Value]) -> Result<Value, RuntimeError> {
        let span = self.current_span().unwrap_or_else(crate::span::Span::dummy);

        match name {
            "map" => self.vm_intrinsic_map(args, span),
            "filter" => self.vm_intrinsic_filter(args, span),
            "reduce" => self.vm_intrinsic_reduce(args, span),
            "forEach" | "for_each" => self.vm_intrinsic_for_each(args, span),
            "find" => self.vm_intrinsic_find(args, span),
            "findIndex" | "find_index" => self.vm_intrinsic_find_index(args, span),
            "flatMap" | "flat_map" => self.vm_intrinsic_flat_map(args, span),
            "some" => self.vm_intrinsic_some(args, span),
            "every" => self.vm_intrinsic_every(args, span),
            "sort" => self.vm_intrinsic_sort(args, span),
            "sortBy" | "sort_by" => self.vm_intrinsic_sort_by(args, span),
            // Result intrinsics (callback-based)
            "result_map" => self.vm_intrinsic_result_map(args, span),
            "result_map_err" => self.vm_intrinsic_result_map_err(args, span),
            "result_and_then" => self.vm_intrinsic_result_and_then(args, span),
            "result_or_else" => self.vm_intrinsic_result_or_else(args, span),
            // HashMap intrinsics (callback-based)
            "hashMapForEach" | "hash_map_for_each" => {
                self.vm_intrinsic_hashmap_for_each(args, span)
            }
            "hashMapMap" | "hash_map_map" => self.vm_intrinsic_hashmap_map(args, span),
            "hashMapFilter" | "hash_map_filter" => self.vm_intrinsic_hashmap_filter(args, span),
            // HashSet intrinsics (callback-based)
            "hashSetForEach" | "hash_set_for_each" => {
                self.vm_intrinsic_hashset_for_each(args, span)
            }
            "hashSetMap" | "hash_set_map" => self.vm_intrinsic_hashset_map(args, span),
            "hashSetFilter" | "hash_set_filter" => self.vm_intrinsic_hashset_filter(args, span),
            // Regex intrinsics (callback-based)
            "regexReplaceWith" | "regex_replace_with" => {
                self.vm_intrinsic_regex_replace_with(args, span)
            }
            "regexReplaceAllWith" | "regex_replace_all_with" => {
                self.vm_intrinsic_regex_replace_all_with(args, span)
            }
            // Test intrinsics (callable assertions)
            "testNsThrows" | "assertThrows" | "assert_throws" => {
                self.vm_intrinsic_assert_throws(args, span)
            }
            "testNsNoThrow" | "assertNoThrow" | "assert_no_throw" => {
                self.vm_intrinsic_assert_no_throw(args, span)
            }
            _ => Err(RuntimeError::UnknownFunction {
                name: name.to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "map() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "map() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::with_capacity(arr.len());

        for elem in arr {
            let callback_result = self.vm_call_function_value(callback, vec![elem], span)?;
            result.push(callback_result);
        }

        Ok(Value::array(result))
    }

    fn vm_intrinsic_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "filter() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "filter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();

        for elem in arr {
            let pred_result = self.vm_call_function_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => result.push(elem),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "filter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::array(result))
    }

    fn vm_intrinsic_reduce(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "reduce() expects 3 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let reducer = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "reduce() second argument must be function".to_string(),
                    span,
                })
            }
        };
        let mut accumulator = args[2].clone();

        for elem in arr {
            accumulator = self.vm_call_function_value(reducer, vec![accumulator, elem], span)?;
        }

        Ok(accumulator)
    }

    fn vm_intrinsic_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "forEach() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "forEach() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for elem in arr {
            self.vm_call_function_value(callback, vec![elem], span)?;
        }

        Ok(Value::Null)
    }

    fn vm_intrinsic_find(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "find() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "find() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for elem in arr {
            let pred_result = self.vm_call_function_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(Value::Option(Some(Box::new(elem)))),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "find() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Option(None))
    }

    fn vm_intrinsic_find_index(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "findIndex() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "findIndex() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for (i, elem) in arr.iter().enumerate() {
            let pred_result = self.vm_call_function_value(predicate, vec![elem.clone()], span)?;
            match pred_result {
                Value::Bool(true) => {
                    return Ok(Value::Option(Some(Box::new(Value::Number(i as f64)))))
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "findIndex() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Option(None))
    }

    fn vm_intrinsic_flat_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "flatMap() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "flatMap() second argument must be function".to_string(),
                    span,
                })
            }
        };
        let mut result = Vec::new();

        for elem in arr {
            let callback_result = self.vm_call_function_value(callback, vec![elem], span)?;
            match callback_result {
                Value::Array(nested) => {
                    result.extend(nested.iter().cloned());
                }
                other => result.push(other),
            }
        }

        Ok(Value::array(result))
    }

    fn vm_intrinsic_some(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "some() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "some() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for elem in arr {
            let pred_result = self.vm_call_function_value(predicate, vec![elem], span)?;
            match pred_result {
                Value::Bool(true) => return Ok(Value::Bool(true)),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "some() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Bool(false))
    }

    fn vm_intrinsic_every(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "every() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "every() second argument must be function".to_string(),
                    span,
                })
            }
        };
        for elem in arr {
            let pred_result = self.vm_call_function_value(predicate, vec![elem], span)?;
            match pred_result {
                Value::Bool(false) => return Ok(Value::Bool(false)),
                Value::Bool(true) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "every() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Bool(true))
    }

    fn vm_intrinsic_sort(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "sort() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let comparator = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sort() second argument must be function".to_string(),
                    span,
                })
            }
        };

        // Insertion sort for stability
        let mut sorted = arr;
        for i in 1..sorted.len() {
            let mut j = i;
            while j > 0 {
                let cmp_result = self.vm_call_function_value(
                    comparator,
                    vec![sorted[j].clone(), sorted[j - 1].clone()],
                    span,
                )?;
                match cmp_result {
                    Value::Number(n) if n < 0.0 => {
                        sorted.swap(j, j - 1);
                        j -= 1;
                    }
                    Value::Number(_) => break,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "sort() comparator must return number".to_string(),
                            span,
                        })
                    }
                }
            }
        }

        Ok(Value::array(sorted))
    }

    fn vm_intrinsic_sort_by(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "sortBy() expects 2 arguments".to_string(),
                span,
            });
        }

        let arr = match &args[0] {
            Value::Array(a) => a.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() first argument must be array".to_string(),
                    span,
                })
            }
        };

        let key_extractor = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "sortBy() second argument must be function".to_string(),
                    span,
                })
            }
        };

        // Extract keys
        let mut keyed: Vec<(Value, Value)> = Vec::new();
        for elem in arr {
            let key = self.vm_call_function_value(key_extractor, vec![elem.clone()], span)?;
            keyed.push((key, elem));
        }

        // Sort by keys (insertion sort for stability)
        for i in 1..keyed.len() {
            let mut j = i;
            while j > 0 {
                let cmp = match (&keyed[j].0, &keyed[j - 1].0) {
                    (Value::Number(a), Value::Number(b)) => {
                        if a < b {
                            -1
                        } else if a > b {
                            1
                        } else {
                            0
                        }
                    }
                    (Value::String(a), Value::String(b)) => {
                        if a < b {
                            -1
                        } else if a > b {
                            1
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };

                if cmp < 0 {
                    keyed.swap(j, j - 1);
                    j -= 1;
                } else {
                    break;
                }
            }
        }

        let sorted: Vec<Value> = keyed.into_iter().map(|(_, elem)| elem).collect();
        Ok(Value::array(sorted))
    }

    // ========================================================================
    // Result Intrinsics (Callback-based operations) - VM versions
    // ========================================================================

    fn vm_intrinsic_result_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_map() expects 2 arguments (result, transform_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let transform_fn = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_map() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => {
                let transformed =
                    self.vm_call_function_value(transform_fn, vec![(**val).clone()], span)?;
                Ok(Value::Result(Ok(Box::new(transformed))))
            }
            Value::Result(Err(err)) => Ok(Value::Result(Err(err.clone()))),
            _ => Err(RuntimeError::TypeError {
                msg: "result_map() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_result_map_err(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_map_err() expects 2 arguments (result, transform_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let transform_fn = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_map_err() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => Ok(Value::Result(Ok(val.clone()))),
            Value::Result(Err(err)) => {
                let transformed =
                    self.vm_call_function_value(transform_fn, vec![(**err).clone()], span)?;
                Ok(Value::Result(Err(Box::new(transformed))))
            }
            _ => Err(RuntimeError::TypeError {
                msg: "result_map_err() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_result_and_then(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_and_then() expects 2 arguments (result, next_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let next_fn = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_and_then() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => {
                // Call next_fn which should return a Result
                self.vm_call_function_value(next_fn, vec![(**val).clone()], span)
            }
            Value::Result(Err(err)) => Ok(Value::Result(Err(err.clone()))),
            _ => Err(RuntimeError::TypeError {
                msg: "result_and_then() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_result_or_else(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "result_or_else() expects 2 arguments (result, recovery_fn)".to_string(),
                span,
            });
        }

        let result_val = &args[0];
        let recovery_fn = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "result_or_else() second argument must be function".to_string(),
                    span,
                })
            }
        };

        match result_val {
            Value::Result(Ok(val)) => Ok(Value::Result(Ok(val.clone()))),
            Value::Result(Err(err)) => {
                // Call recovery_fn which should return a Result
                self.vm_call_function_value(recovery_fn, vec![(**err).clone()], span)
            }
            _ => Err(RuntimeError::TypeError {
                msg: "result_or_else() first argument must be Result".to_string(),
                span,
            }),
        }
    }

    fn vm_intrinsic_hashmap_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapForEach() expects 2 arguments (map, callback)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::Map(m) => m.entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapForEach() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapForEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for (key, value) in map {
            self.vm_call_function_value(callback, vec![value, key.to_value()], span)?;
        }

        Ok(Value::Null)
    }

    fn vm_intrinsic_hashmap_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapMap() expects 2 arguments (map, callback)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::Map(m) => m.entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapMap() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
        for (key, value) in map {
            let new_value =
                self.vm_call_function_value(callback, vec![value, key.clone().to_value()], span)?;
            result_map.insert(key, new_value);
        }

        Ok(Value::Map(ValueHashMap::from_atlas(result_map)))
    }

    fn vm_intrinsic_hashmap_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashMapFilter() expects 2 arguments (map, predicate)".to_string(),
                span,
            });
        }

        let map = match &args[0] {
            Value::Map(m) => m.entries(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapFilter() first argument must be HashMap".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashMapFilter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
        for (key, value) in map {
            let pred_result = self.vm_call_function_value(
                predicate,
                vec![value.clone(), key.clone().to_value()],
                span,
            )?;
            match pred_result {
                Value::Bool(true) => {
                    result_map.insert(key, value);
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "hashMapFilter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Map(ValueHashMap::from_atlas(result_map)))
    }

    fn vm_intrinsic_hashset_for_each(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetForEach() expects 2 arguments (set, callback)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::Set(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetForEach() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetForEach() second argument must be function".to_string(),
                    span,
                })
            }
        };

        for element in set {
            self.vm_call_function_value(callback, vec![element.to_value()], span)?;
        }

        Ok(Value::Null)
    }

    fn vm_intrinsic_hashset_map(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetMap() expects 2 arguments (set, callback)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::Set(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetMap() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetMap() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result = Vec::new();
        for element in set {
            let mapped_value =
                self.vm_call_function_value(callback, vec![element.to_value()], span)?;
            result.push(mapped_value);
        }

        Ok(Value::array(result))
    }

    fn vm_intrinsic_hashset_filter(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError {
                msg: "hashSetFilter() expects 2 arguments (set, predicate)".to_string(),
                span,
            });
        }

        let set = match &args[0] {
            Value::Set(s) => s.inner().to_vec(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetFilter() first argument must be HashSet".to_string(),
                    span,
                })
            }
        };

        let predicate = match &args[1] {
            Value::Function(_)
            | Value::Closure(_)
            | Value::Builtin(_)
            | Value::NativeFunction(_) => &args[1],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "hashSetFilter() second argument must be function".to_string(),
                    span,
                })
            }
        };

        let mut result_set = crate::stdlib::collections::hashset::AtlasHashSet::new();
        for element in set {
            let pred_result =
                self.vm_call_function_value(predicate, vec![element.clone().to_value()], span)?;
            match pred_result {
                Value::Bool(true) => {
                    result_set.insert(element);
                }
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "hashSetFilter() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Set(ValueHashSet::from_atlas(result_set)))
    }

    /// Regex intrinsic: Replace first match using callback (VM version)
    fn vm_intrinsic_regex_replace_with(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "regexReplaceWith() expects 3 arguments (regex, text, callback)".to_string(),
                span,
            });
        }

        let regex = match &args[0] {
            Value::Regex(r) => r.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() first argument must be Regex".to_string(),
                    span,
                })
            }
        };

        let text = match &args[1] {
            Value::String(s) => s.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() second argument must be string".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[2] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[2],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceWith() third argument must be function".to_string(),
                    span,
                })
            }
        };

        // Find first match
        if let Some(mat) = regex.find(text) {
            let match_start = mat.start();
            let match_end = mat.end();
            let match_text = mat.as_str();

            // Build match data HashMap
            let mut match_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "text".to_string(),
                )),
                Value::string(match_text),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "start".to_string(),
                )),
                Value::Number(match_start as f64),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "end".to_string(),
                )),
                Value::Number(match_end as f64),
            );

            // Extract capture groups
            if let Some(caps) = regex.captures(text) {
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(group) = caps.get(i) {
                        groups.push(Value::string(group.as_str()));
                    } else {
                        groups.push(Value::Null);
                    }
                }
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(groups),
                );
            } else {
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(vec![]),
                );
            }

            let match_value = Value::Map(ValueHashMap::from_atlas(match_map));

            // Call callback with match data
            let replacement_value =
                self.vm_call_function_value(callback, vec![match_value], span)?;

            // Expect string return value and clone to avoid lifetime issues
            let replacement_str = match &replacement_value {
                Value::String(s) => s.as_ref().to_string(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "regexReplaceWith() callback must return string".to_string(),
                        span,
                    })
                }
            };

            // Build result string
            let mut result = String::with_capacity(text.len());
            result.push_str(&text[..match_start]);
            result.push_str(&replacement_str);
            result.push_str(&text[match_end..]);

            Ok(Value::string(result))
        } else {
            // No match, return original text
            Ok(Value::string(text))
        }
    }

    /// Regex intrinsic: Replace all matches using callback (VM version)
    fn vm_intrinsic_regex_replace_all_with(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        if args.len() != 3 {
            return Err(RuntimeError::TypeError {
                msg: "regexReplaceAllWith() expects 3 arguments (regex, text, callback)"
                    .to_string(),
                span,
            });
        }

        let regex = match &args[0] {
            Value::Regex(r) => r.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() first argument must be Regex".to_string(),
                    span,
                })
            }
        };

        let text = match &args[1] {
            Value::String(s) => s.as_ref(),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() second argument must be string".to_string(),
                    span,
                })
            }
        };

        let callback = match &args[2] {
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_) => &args[2],
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "regexReplaceAllWith() third argument must be function".to_string(),
                    span,
                })
            }
        };

        // Find all matches and collect them
        let matches: Vec<_> = regex.find_iter(text).collect();

        if matches.is_empty() {
            return Ok(Value::string(text));
        }

        // Build result string by processing all matches
        let mut result = String::with_capacity(text.len());
        let mut last_end = 0;

        for mat in matches {
            let match_start = mat.start();
            let match_end = mat.end();
            let match_text = mat.as_str();

            // Build match data HashMap
            let mut match_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "text".to_string(),
                )),
                Value::string(match_text),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "start".to_string(),
                )),
                Value::Number(match_start as f64),
            );
            match_map.insert(
                crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                    "end".to_string(),
                )),
                Value::Number(match_end as f64),
            );

            // Extract capture groups
            if let Some(caps) = regex.captures(mat.as_str()) {
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(group) = caps.get(i) {
                        groups.push(Value::string(group.as_str()));
                    } else {
                        groups.push(Value::Null);
                    }
                }
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(groups),
                );
            } else {
                match_map.insert(
                    crate::stdlib::collections::hash::HashKey::String(std::sync::Arc::new(
                        "groups".to_string(),
                    )),
                    Value::array(vec![]),
                );
            }

            let match_value = Value::Map(ValueHashMap::from_atlas(match_map));

            // Call callback with match data
            let replacement_value =
                self.vm_call_function_value(callback, vec![match_value], span)?;

            // Expect string return value and clone to avoid lifetime issues
            let replacement_str = match &replacement_value {
                Value::String(s) => s.as_ref().to_string(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "regexReplaceAllWith() callback must return string".to_string(),
                        span,
                    })
                }
            };

            // Add text before this match
            result.push_str(&text[last_end..match_start]);
            // Add replacement
            result.push_str(&replacement_str);

            last_end = match_end;
        }

        // Add remaining text after last match
        result.push_str(&text[last_end..]);

        Ok(Value::string(result))
    }

    // ========================================================================
    // Test Intrinsics (Callable assertions)
    // ========================================================================

    fn vm_intrinsic_assert_throws(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        crate::stdlib::test::assert_throws_with(args, span, true, |callable| {
            self.vm_call_function_value(callable, vec![], span)
        })
    }

    fn vm_intrinsic_assert_no_throw(
        &mut self,
        args: &[Value],
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        crate::stdlib::test::assert_no_throw_with(args, span, true, |callable| {
            self.vm_call_function_value(callable, vec![], span)
        })
    }

    /// Helper: Call a function value with arguments (VM version)
    fn vm_call_function_value(
        &mut self,
        func: &Value,
        args: Vec<Value>,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Builtin(name) => {
                let security =
                    self.current_security
                        .as_ref()
                        .ok_or_else(|| RuntimeError::InternalError {
                            msg: "Security context not set".to_string(),
                            span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                        })?;
                crate::stdlib::call_builtin(name, &args, span, security, &self.output_writer)
            }
            Value::Function(func_ref) => {
                // User-defined function - execute via VM
                let saved_ip = self.ip;
                let saved_frame_depth = self.frames.len();
                let stack_base = self.stack.len();
                let arg_count = args.len();

                // Verify arity before pushing (B39-P05: support default params)
                if arg_count < func_ref.required_arity || arg_count > func_ref.arity {
                    let expected = if func_ref.required_arity == func_ref.arity {
                        format!("{}", func_ref.arity)
                    } else {
                        format!("{}-{}", func_ref.required_arity, func_ref.arity)
                    };
                    return Err(RuntimeError::TypeError {
                        msg: format!(
                            "Function {} expects {} arguments, got {}",
                            func_ref.name, expected, arg_count
                        ),
                        span,
                    });
                }

                // Push arguments onto stack (they become the function's locals)
                for arg in args {
                    self.push(arg);
                }

                // Fill in default values for missing arguments (B39-P05)
                for i in arg_count..func_ref.arity {
                    if let Some(Some(default_val)) = func_ref.defaults.get(i) {
                        self.push(default_val.clone());
                    } else {
                        self.push(Value::Null);
                    }
                }

                // Create call frame
                let frame = CallFrame {
                    function_name: func_ref.name.clone(),
                    return_ip: saved_ip,
                    stack_base,
                    local_count: func_ref.local_count,
                    upvalues: std::sync::Arc::new(Vec::new()),
                };
                self.frames.push(frame);
                self.defer_stacks.push(Vec::new());
                #[cfg(debug_assertions)]
                self.consumed_slots.push(vec![false; func_ref.local_count]);

                // Jump to function bytecode
                self.ip = func_ref.bytecode_offset;

                // Execute until this frame returns (depth goes back to saved_frame_depth)
                let result = self.execute_loop(Some(saved_frame_depth))?;

                // Get the return value from stack
                let return_value = result.unwrap_or(Value::Null);
                // Clean up stack to original base
                self.stack.truncate(stack_base);
                #[cfg(debug_assertions)]
                self.value_origins.truncate(stack_base);

                // Restore IP
                self.ip = saved_ip;

                Ok(return_value)
            }
            Value::Closure(closure) => {
                // Closure call: same as Function but passes captured upvalues to the frame
                let func = closure.func.clone();
                let upvalues = closure.upvalues.clone();
                let saved_ip = self.ip;
                let saved_frame_depth = self.frames.len();
                let stack_base = self.stack.len();
                let arg_count = args.len();

                // Verify arity (B39-P05: support default params)
                if arg_count < func.required_arity || arg_count > func.arity {
                    let expected = if func.required_arity == func.arity {
                        format!("{}", func.arity)
                    } else {
                        format!("{}-{}", func.required_arity, func.arity)
                    };
                    return Err(RuntimeError::TypeError {
                        msg: format!(
                            "Function {} expects {} arguments, got {}",
                            func.name, expected, arg_count
                        ),
                        span,
                    });
                }

                for arg in args {
                    self.push(arg);
                }

                // Fill in default values for missing arguments (B39-P05)
                for i in arg_count..func.arity {
                    if let Some(Some(default_val)) = func.defaults.get(i) {
                        self.push(default_val.clone());
                    } else {
                        self.push(Value::Null);
                    }
                }

                let frame = CallFrame {
                    function_name: func.name.clone(),
                    return_ip: saved_ip,
                    stack_base,
                    local_count: func.local_count,
                    upvalues,
                };
                self.frames.push(frame);
                self.defer_stacks.push(Vec::new());
                #[cfg(debug_assertions)]
                self.consumed_slots.push(vec![false; func.local_count]);

                self.ip = func.bytecode_offset;

                let result = self.execute_loop(Some(saved_frame_depth))?;
                let return_value = result.unwrap_or(Value::Null);
                self.stack.truncate(stack_base);
                #[cfg(debug_assertions)]
                self.value_origins.truncate(stack_base);

                self.ip = saved_ip;
                Ok(return_value)
            }
            Value::NativeFunction(native_fn) => {
                // Call the native Rust closure directly
                native_fn(&args)
            }
            _ => Err(RuntimeError::TypeError {
                msg: "Expected function value".to_string(),
                span,
            }),
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(Bytecode::new())
    }
}
