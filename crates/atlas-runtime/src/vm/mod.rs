//! Stack-based virtual machine
//!
//! Executes bytecode instructions with a value stack and call frames.
//! - Arithmetic operations check for NaN/Infinity
//! - Variables are stored in locals (stack) or globals (HashMap)
//! - Control flow uses jumps and loops

mod debugger;
pub mod dispatch;
mod frame;
mod profiler;

pub use debugger::{DebugAction, DebugHook, Debugger};
pub use frame::CallFrame;
pub use profiler::Profiler;

use crate::bytecode::{Bytecode, Opcode};
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
    /// Output writer for print() (defaults to stdout)
    output_writer: crate::stdlib::OutputWriter,
    /// FFI library loader (phase-10b)
    library_loader: LibraryLoader,
    /// Loaded extern functions (phase-10b)
    extern_functions: HashMap<String, ExternFunction>,
    /// Reusable string buffer for temporary string operations (reduces allocations)
    string_buffer: String,
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
            output_writer: crate::stdlib::stdout_writer(),
            library_loader: LibraryLoader::new(),
            extern_functions: HashMap::new(),
            string_buffer: String::with_capacity(256),
            #[cfg(debug_assertions)]
            value_origins: Vec::with_capacity(1024),
            #[cfg(debug_assertions)]
            consumed_slots: vec![vec![false; main_local_count]],
            #[cfg(debug_assertions)]
            consumed_globals: std::collections::HashSet::new(),
        }
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
                let extern_fn = unsafe { ExternFunction::new(*fn_ptr, param_types, return_type) };

                // Store the extern function
                self.extern_functions
                    .insert(extern_decl.name.clone(), extern_fn);

                // Register as a callable global
                let func_value = Value::Function(FunctionRef {
                    name: extern_decl.name.clone(),
                    arity: extern_decl.params.len(),
                    bytecode_offset: 0, // Not used for extern functions
                    local_count: 0,     // Not used for extern functions
                    param_ownership: vec![],
                    param_names: vec![],
                    return_ownership: None,
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
        let dbg = self.debugger.as_mut().unwrap();
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
        self.debugger
            .as_mut()
            .unwrap()
            .set_step_condition(step_condition);

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
                        *self.value_origins.last_mut().unwrap() =
                            Some(StackValueOrigin::Local(index));
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
                    } else {
                        // Check math constants
                        match name.as_str() {
                            "PI" => Value::Number(crate::stdlib::math::PI),
                            "E" => Value::Number(crate::stdlib::math::E),
                            "SQRT2" => Value::Number(crate::stdlib::math::SQRT2),
                            "LN2" => Value::Number(crate::stdlib::math::LN2),
                            "LN10" => Value::Number(crate::stdlib::math::LN10),
                            _ => {
                                return Err(RuntimeError::UndefinedVariable {
                                    name: name.clone(),
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }
                        }
                    };
                    self.push(value);
                    // Record global origin for own-consume tracking (debug builds only).
                    // Only track user-defined globals (not builtins, constructors, math constants).
                    #[cfg(debug_assertions)]
                    if self.globals.contains_key(&name) {
                        *self.value_origins.last_mut().unwrap() =
                            Some(StackValueOrigin::Global(name));
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
                    let frame = self.frames.last_mut().expect("no call frame");
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
                            // Reuse string buffer to reduce allocations
                            self.string_buffer.clear();
                            self.string_buffer.push_str(x);
                            self.string_buffer.push_str(y);
                            self.push(Value::String(Arc::new(self.string_buffer.clone())));
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
                Opcode::And | Opcode::Or => {
                    // TODO: Short-circuit evaluation
                    return Err(RuntimeError::UnknownOpcode {
                        span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                    });
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
                Opcode::Call => {
                    let arg_count = self.read_u8()? as usize;

                    // Get the function value from stack (it's below the arguments)
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

                                let security = self
                                    .current_security
                                    .as_ref()
                                    .expect("Security context not set");
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
                            if let Some(extern_fn) = self.extern_functions.get(&func.name).cloned()
                            {
                                let mut args = Vec::with_capacity(arg_count);
                                for _ in 0..arg_count {
                                    args.push(self.pop());
                                }
                                args.reverse();
                                self.pop(); // Pop function value

                                let result = unsafe { extern_fn.call(&args) }.map_err(|e| {
                                    RuntimeError::TypeError {
                                        msg: format!("FFI call error: {}", e),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    }
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
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }

                                // Create a new call frame
                                let frame = CallFrame {
                                    function_name: func.name.clone(),
                                    return_ip: self.ip,
                                    stack_base: self.stack.len() - arg_count, // Points to first argument
                                    local_count: func.local_count, // Use total locals, not just arity
                                    upvalues: std::sync::Arc::new(Vec::new()),
                                };

                                // Verify argument count matches
                                if arg_count != func.arity {
                                    return Err(RuntimeError::TypeError {
                                        msg: format!(
                                            "Function {} expects {} arguments, got {}",
                                            func.name, func.arity, arg_count
                                        ),
                                        span: self
                                            .current_span()
                                            .unwrap_or_else(crate::span::Span::dummy),
                                    });
                                }

                                // Debug mode: for each `own` parameter, mark the caller's
                                // local slot or global name as consumed so subsequent reads
                                // produce a runtime error.
                                #[cfg(debug_assertions)]
                                {
                                    let caller_frame_idx = self.frames.len() - 1;
                                    let args_base = self.stack.len() - arg_count;
                                    for (i, ownership) in func.param_ownership.iter().enumerate() {
                                        if *ownership == Some(crate::ast::OwnershipAnnotation::Own)
                                        {
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

                                // Debug mode: enforce `shared` parameter ownership contracts.
                                #[cfg(debug_assertions)]
                                {
                                    let args_base = self.stack.len() - arg_count;
                                    for (i, ownership) in func.param_ownership.iter().enumerate() {
                                        let arg = &self.stack[args_base + i];
                                        match ownership {
                                            Some(crate::ast::OwnershipAnnotation::Shared) => {
                                                if !matches!(arg, Value::SharedValue(_)) {
                                                    return Err(RuntimeError::TypeError {
                                                        msg: format!(
                                                            "ownership violation: parameter '{}' expects shared<T> but received {}",
                                                            func.param_names.get(i).map(|s| s.as_str()).unwrap_or("?"),
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
                                                        Some(
                                                            crate::ast::OwnershipAnnotation::Own,
                                                        ) => "own",
                                                        Some(
                                                            crate::ast::OwnershipAnnotation::Borrow,
                                                        ) => "borrow",
                                                        _ => unreachable!(),
                                                    };
                                                    eprintln!(
                                                        "warning: passing shared<T> value to '{}' parameter '{}' — consider using the 'shared' annotation",
                                                        ann_str,
                                                        func.param_names.get(i).map(|s| s.as_str()).unwrap_or("?")
                                                    );
                                                }
                                            }
                                            None => {}
                                        }
                                    }
                                }

                                // Push the frame (and its consumed-slot tracking vector)
                                self.frames.push(frame);
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
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
                            }

                            if arg_count != func.arity {
                                return Err(RuntimeError::TypeError {
                                    msg: format!(
                                        "Function {} expects {} arguments, got {}",
                                        func.name, func.arity, arg_count
                                    ),
                                    span: self
                                        .current_span()
                                        .unwrap_or_else(crate::span::Span::dummy),
                                });
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

                            // Debug mode: enforce `shared` parameter ownership contracts.
                            #[cfg(debug_assertions)]
                            {
                                let args_base = self.stack.len() - arg_count;
                                for (i, ownership) in func.param_ownership.iter().enumerate() {
                                    let arg = &self.stack[args_base + i];
                                    match ownership {
                                        Some(crate::ast::OwnershipAnnotation::Shared) => {
                                            if !matches!(arg, Value::SharedValue(_)) {
                                                return Err(RuntimeError::TypeError {
                                                    msg: format!(
                                                        "ownership violation: parameter '{}' expects shared<T> but received {}",
                                                        func.param_names.get(i).map(|s| s.as_str()).unwrap_or("?"),
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
                                                    Some(crate::ast::OwnershipAnnotation::Own) => {
                                                        "own"
                                                    }
                                                    Some(
                                                        crate::ast::OwnershipAnnotation::Borrow,
                                                    ) => "borrow",
                                                    _ => unreachable!(),
                                                };
                                                eprintln!(
                                                    "warning: passing shared<T> value to '{}' parameter '{}' — consider using the 'shared' annotation",
                                                    ann_str,
                                                    func.param_names.get(i).map(|s| s.as_str()).unwrap_or("?")
                                                );
                                            }
                                        }
                                        None => {}
                                    }
                                }
                            }

                            let frame = CallFrame {
                                function_name: func.name.clone(),
                                return_ip: self.ip,
                                stack_base: self.stack.len() - arg_count,
                                local_count: func.local_count,
                                upvalues,
                            };

                            self.frames.push(frame);
                            #[cfg(debug_assertions)]
                            self.consumed_slots.push(vec![false; func.local_count]);
                            if let Some(ref mut profiler) = self.profiler {
                                if profiler.is_enabled() {
                                    profiler.record_function_call(&func.name);
                                }
                            }
                            self.ip = func.bytecode_offset;
                        }
                        Value::NativeFunction(native_fn) => {
                            // Call the native Rust closure
                            let mut args = Vec::with_capacity(arg_count);
                            for _ in 0..arg_count {
                                args.push(self.pop());
                            }
                            args.reverse(); // Arguments were pushed in reverse order

                            // Pop the function value from stack
                            self.pop();

                            let result = native_fn(&args)?;
                            self.push(result);
                        }
                        // None() is a valid zero-arg constructor call
                        Value::Option(None) if arg_count == 0 => {
                            self.pop(); // Pop the Option(None) function value
                            self.push(Value::Option(None));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot call non-function value".to_string(),
                                span: self.current_span().unwrap_or_else(crate::span::Span::dummy),
                            });
                        }
                    }
                }
                Opcode::Return => {
                    // Pop the return value from stack (if any)
                    let return_value = if self.stack.is_empty() {
                        Value::Null
                    } else {
                        self.pop()
                    };

                    // Pop the call frame
                    let frame = self.frames.pop();
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
                    let mut elements = Vec::with_capacity(size);
                    for _ in 0..size {
                        elements.push(self.pop());
                    }
                    elements.reverse(); // Stack is LIFO, so reverse to get correct order
                    self.push(Value::Array(ValueArray::from_vec(elements)));
                }
                Opcode::GetIndex => {
                    let index_val = self.pop();
                    let target = self.pop();
                    match target {
                        Value::Array(arr) => {
                            // Array indexing requires number
                            if let Value::Number(index) = index_val {
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
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "Cannot index non-array/json".to_string(),
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
                        _ => {
                            return Err(RuntimeError::TypeError {
                                msg: "GetArrayLen requires Array".to_string(),
                                span: Span::dummy(),
                            })
                        }
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
        // SAFETY: VM invariants guarantee stack is non-empty when pop is called
        unsafe { self.stack.pop().unwrap_unchecked() }
    }

    #[inline(always)]
    fn peek(&self, distance: usize) -> &Value {
        // SAFETY: The compiler guarantees stack depth matches operand requirements.
        // Each opcode that calls peek() is emitted only when sufficient values exist.
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
        // SAFETY: VM execution always pushes a frame before running, and frames
        // are only popped when returning. The frames vec is never empty during execution.
        unsafe { self.frames.last().unwrap_unchecked() }
    }

    /// Generate a stack trace from the current call frames
    /// Returns a vector of function names from innermost to outermost
    #[allow(dead_code)]
    fn stack_trace(&self) -> Vec<String> {
        self.frames
            .iter()
            .rev()
            .map(|frame| frame.function_name.clone())
            .collect()
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
            "forEach" => self.vm_intrinsic_for_each(args, span),
            "find" => self.vm_intrinsic_find(args, span),
            "findIndex" => self.vm_intrinsic_find_index(args, span),
            "flatMap" => self.vm_intrinsic_flat_map(args, span),
            "some" => self.vm_intrinsic_some(args, span),
            "every" => self.vm_intrinsic_every(args, span),
            "sort" => self.vm_intrinsic_sort(args, span),
            "sortBy" => self.vm_intrinsic_sort_by(args, span),
            // Result intrinsics (callback-based)
            "result_map" => self.vm_intrinsic_result_map(args, span),
            "result_map_err" => self.vm_intrinsic_result_map_err(args, span),
            "result_and_then" => self.vm_intrinsic_result_and_then(args, span),
            "result_or_else" => self.vm_intrinsic_result_or_else(args, span),
            // HashMap intrinsics (callback-based)
            "hashMapForEach" => self.vm_intrinsic_hashmap_for_each(args, span),
            "hashMapMap" => self.vm_intrinsic_hashmap_map(args, span),
            "hashMapFilter" => self.vm_intrinsic_hashmap_filter(args, span),
            // HashSet intrinsics (callback-based)
            "hashSetForEach" => self.vm_intrinsic_hashset_for_each(args, span),
            "hashSetMap" => self.vm_intrinsic_hashset_map(args, span),
            "hashSetFilter" => self.vm_intrinsic_hashset_filter(args, span),
            // Regex intrinsics (callback-based)
            "regexReplaceWith" => self.vm_intrinsic_regex_replace_with(args, span),
            "regexReplaceAllWith" => self.vm_intrinsic_regex_replace_all_with(args, span),
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
                Value::Bool(true) => return Ok(elem),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "find() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Null)
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
                Value::Bool(true) => return Ok(Value::Number(i as f64)),
                Value::Bool(false) => {}
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "findIndex() predicate must return bool".to_string(),
                        span,
                    })
                }
            }
        }

        Ok(Value::Number(-1.0))
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
            Value::HashMap(m) => m.inner().entries(),
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
            Value::HashMap(m) => m.inner().entries(),
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

        Ok(Value::HashMap(ValueHashMap::from_atlas(result_map)))
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
            Value::HashMap(m) => m.inner().entries(),
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

        Ok(Value::HashMap(ValueHashMap::from_atlas(result_map)))
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
            Value::HashSet(s) => s.inner().to_vec(),
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
            Value::HashSet(s) => s.inner().to_vec(),
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
            Value::HashSet(s) => s.inner().to_vec(),
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

        Ok(Value::HashSet(ValueHashSet::from_atlas(result_set)))
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

            let match_value = Value::HashMap(ValueHashMap::from_atlas(match_map));

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

            let match_value = Value::HashMap(ValueHashMap::from_atlas(match_map));

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

    /// Helper: Call a function value with arguments (VM version)
    fn vm_call_function_value(
        &mut self,
        func: &Value,
        args: Vec<Value>,
        span: crate::span::Span,
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Builtin(name) => {
                let security = self
                    .current_security
                    .as_ref()
                    .expect("Security context not set");
                crate::stdlib::call_builtin(name, &args, span, security, &self.output_writer)
            }
            Value::Function(func_ref) => {
                // User-defined function - execute via VM
                let saved_ip = self.ip;
                let saved_frame_depth = self.frames.len();
                let stack_base = self.stack.len();
                let arg_count = args.len();

                // Verify arity before pushing
                if func_ref.arity != arg_count {
                    return Err(RuntimeError::TypeError {
                        msg: format!(
                            "Function {} expects {} arguments, got {}",
                            func_ref.name, func_ref.arity, arg_count
                        ),
                        span,
                    });
                }

                // Push arguments onto stack (they become the function's locals)
                for arg in args {
                    self.push(arg);
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

                if func.arity != arg_count {
                    return Err(RuntimeError::TypeError {
                        msg: format!(
                            "Function {} expects {} arguments, got {}",
                            func.name, func.arity, arg_count
                        ),
                        span,
                    });
                }

                for arg in args {
                    self.push(arg);
                }

                let frame = CallFrame {
                    function_name: func.name.clone(),
                    return_ip: saved_ip,
                    stack_base,
                    local_count: func.local_count,
                    upvalues,
                };
                self.frames.push(frame);
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

