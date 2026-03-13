//! VMContext — per-thread execution state for the Atlas VM.
//!
//! # Design (D-057)
//!
//! The VM is split into two layers:
//!
//! - **VMContext** (this file): everything that is private to one thread of
//!   execution.  Stack, call frames, instruction pointer, defer stacks, and
//!   scratch buffers live here.  VMContext is `Clone` so that worker threads
//!   can cheaply fork a fresh context from the main one at startup.
//!
//! - **VM** (`mod.rs`): the thin orchestrator that owns one `VMContext` plus
//!   shared resources (globals, bytecode, profiler, debugger, security, I/O,
//!   JIT).  In the concurrency model (B44-P02+), workers hold their own
//!   `VMContext` and share the remaining VM resources via `Arc`.
//!
//! ## Clone semantics
//!
//! Cloning a `VMContext` produces a logically independent execution context
//! with its own stack and frame list.  For a fresh worker the clone is
//! immediately followed by a reset (empty stack, single main frame) before the
//! worker starts executing tasks — the clone is just to inherit any
//! interpreter-level state that must be replicated (e.g. the string scratch
//! buffer capacity).

use std::collections::{HashMap, HashSet};

use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::vm::frame::CallFrame;

/// Per-thread VM execution state.
///
/// Holds everything that must be isolated per worker thread: value stack,
/// call frames, instruction pointer, defer stacks, and scratch buffers.
/// All shared resources (globals, bytecode, profiler, JIT) remain in `VM`.
#[derive(Clone, Debug)]
pub struct VMContext {
    /// Value stack — grows upward during execution.
    pub(super) stack: Vec<Value>,

    /// Call frame stack — one entry per active function invocation.
    pub(super) frames: Vec<CallFrame>,

    /// Instruction pointer — index into the bytecode stream.
    pub(super) ip: usize,

    /// Per-frame defer stacks.  Each entry is `(body_start_ip, body_length)`.
    /// Deferred blocks run LIFO when the frame they were pushed in exits.
    pub(super) defer_stacks: Vec<Vec<(usize, usize)>>,

    /// Reusable scratch buffer for string operations (reduces allocations).
    pub(super) string_buffer: String,

    /// Nominal struct type names for `HashMap`-backed struct values,
    /// keyed by the map's memory identity (pointer cast to `usize`).
    pub(super) struct_type_names: HashMap<usize, String>,

    /// Set to `true` by the execute loop when the debugger requests a pause.
    /// Cleared by `run_debuggable` after it reads the flag.
    pub(super) debug_pause_pending: bool,

    /// Runtime warnings collected during execution (ownership mismatches, …).
    /// Callers retrieve and drain these via `VM::take_runtime_warnings()`.
    pub(super) runtime_warnings: Vec<Diagnostic>,

    /// Parallel to `stack`: tracks where each value originated (debug builds only).
    /// `None` for computed/literal values; `Some(Local(slot))` for GetLocal;
    /// `Some(Global(name))` for GetGlobal.  Used to mark caller's binding as
    /// consumed when a value is passed to an `own` parameter.
    #[cfg(debug_assertions)]
    pub(super) value_origins: Vec<Option<super::StackValueOrigin>>,

    /// Per-frame consumed-slot tracking (debug builds only).
    /// `consumed_slots[frame_idx][slot]` is `true` when that local has been
    /// moved into an `own` parameter and may no longer be read.
    #[cfg(debug_assertions)]
    pub(super) consumed_slots: Vec<Vec<bool>>,

    /// Consumed global names (debug builds only).  A global is inserted here
    /// when it is passed to an `own` parameter; subsequent `GetGlobal` errors.
    #[cfg(debug_assertions)]
    pub(super) consumed_globals: HashSet<String>,
}

impl VMContext {
    /// Create a fresh `VMContext` for the main execution frame.
    ///
    /// `main_local_count` is `bytecode.top_level_local_count` — needed to
    /// pre-size the debug consumed-slots vector.
    pub(super) fn new(
        main_frame: CallFrame,
        #[cfg(debug_assertions)] main_local_count: usize,
    ) -> Self {
        Self {
            stack: Vec::with_capacity(1024),
            frames: vec![main_frame],
            ip: 0,
            defer_stacks: vec![Vec::new()],
            string_buffer: String::new(),
            struct_type_names: HashMap::new(),
            debug_pause_pending: false,
            runtime_warnings: Vec::new(),
            #[cfg(debug_assertions)]
            value_origins: Vec::with_capacity(1024),
            #[cfg(debug_assertions)]
            consumed_slots: vec![vec![false; main_local_count]],
            #[cfg(debug_assertions)]
            consumed_globals: HashSet::new(),
        }
    }

    /// Reset this context to a clean state suitable for a new top-level
    /// execution (empty stack, single main frame, cleared warnings).
    // Used in B44-P02 worker thread setup; suppress dead_code during scaffolding.
    #[allow(dead_code)]
    ///
    /// The new main frame's `local_count` is provided by the caller.
    pub(super) fn reset(
        &mut self,
        main_frame: CallFrame,
        #[cfg(debug_assertions)] main_local_count: usize,
    ) {
        self.stack.clear();
        self.frames.clear();
        self.frames.push(main_frame);
        self.ip = 0;
        self.defer_stacks.clear();
        self.defer_stacks.push(Vec::new());
        self.string_buffer.clear();
        self.struct_type_names.clear();
        self.debug_pause_pending = false;
        self.runtime_warnings.clear();
        #[cfg(debug_assertions)]
        {
            self.value_origins.clear();
            self.consumed_slots.clear();
            self.consumed_slots.push(vec![false; main_local_count]);
            self.consumed_globals.clear();
        }
    }
}
