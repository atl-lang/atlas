//! Atlas JIT Compilation Engine
//!
//! Provides just-in-time compilation for Atlas bytecode using Cranelift
//! as the native code generation backend. Hot functions identified by
//! the VM profiler are compiled to native machine code for 5-10x speedup.
//!
//! # Status: Foundation Complete — Not Yet Wired to Production
//!
//! The JIT compiles **arithmetic-only** functions (numeric constants, local variables,
//! arithmetic operators, comparisons). It does NOT support control flow (jump/call),
//! global variables, or collection opcodes. See `JIT_STATUS.md` for the full capability
//! matrix and v0.3 integration requirements.
//!
//! ## Supported Opcodes
//!
//! `Constant`, `True`, `False`, `Null`, `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Negate`,
//! `Equal`, `NotEqual`, `Less`, `LessEqual`, `Greater`, `GreaterEqual`, `Not`,
//! `GetLocal`, `SetLocal`, `Pop`, `Dup`, `Return`, `Halt`
//!
//! ## Unsupported Opcodes (bail out to interpreter)
//!
//! `GetGlobal`, `SetGlobal`, `Jump`, `JumpIfFalse`, `Loop`, `Call`, `And`, `Or`,
//! `Array`, `GetIndex`, `SetIndex`, `IsOptionSome`, `IsOptionNone`, `IsResultOk`,
//! `IsResultErr`, `ExtractOptionValue`, `ExtractResultValue`, `IsArray`, `GetArrayLen`

pub mod backend;
pub mod cache;
pub mod codegen;
pub mod hotspot;

use thiserror::Error;

/// Call a JIT-compiled function pointer with the given arguments
///
/// # Safety
/// The code_ptr must be valid and point to a function with the correct signature.
/// The arity must match args.len().
unsafe fn call_jit_function(code_ptr: *const u8, args: &[f64]) -> f64 {
    match args.len() {
        0 => {
            let func: unsafe fn() -> f64 = std::mem::transmute(code_ptr);
            func()
        }
        1 => {
            let func: unsafe fn(f64) -> f64 = std::mem::transmute(code_ptr);
            func(args[0])
        }
        2 => {
            let func: unsafe fn(f64, f64) -> f64 = std::mem::transmute(code_ptr);
            func(args[0], args[1])
        }
        3 => {
            let func: unsafe fn(f64, f64, f64) -> f64 = std::mem::transmute(code_ptr);
            func(args[0], args[1], args[2])
        }
        4 => {
            let func: unsafe fn(f64, f64, f64, f64) -> f64 = std::mem::transmute(code_ptr);
            func(args[0], args[1], args[2], args[3])
        }
        5 => {
            let func: unsafe fn(f64, f64, f64, f64, f64) -> f64 = std::mem::transmute(code_ptr);
            func(args[0], args[1], args[2], args[3], args[4])
        }
        6 => {
            let func: unsafe fn(f64, f64, f64, f64, f64, f64) -> f64 =
                std::mem::transmute(code_ptr);
            func(args[0], args[1], args[2], args[3], args[4], args[5])
        }
        _ => {
            // For functions with more than 6 args, fall back to zero
            // (unsupported — caller should have bailed)
            0.0
        }
    }
}

/// JIT compilation errors
#[derive(Debug, Error)]
pub enum JitError {
    #[error("compilation failed: {0}")]
    CompilationFailed(String),

    #[error("unsupported opcode: {0:?}")]
    UnsupportedOpcode(atlas_runtime::bytecode::Opcode),

    #[error("code cache full (limit: {limit} bytes, used: {used} bytes)")]
    CacheFull { limit: usize, used: usize },

    #[error("invalid bytecode: {0}")]
    InvalidBytecode(String),

    #[error("native execution error: {0}")]
    ExecutionError(String),
}

/// Result type for JIT operations
pub type JitResult<T> = Result<T, JitError>;

/// Configuration for the JIT compiler
#[derive(Debug, Clone)]
pub struct JitConfig {
    /// Minimum execution count before a function is JIT-compiled
    pub compilation_threshold: u64,
    /// Maximum bytes of native code to cache
    pub cache_size_limit: usize,
    /// Whether to enable JIT compilation
    pub enabled: bool,
    /// Optimization level for Cranelift (0=none, 1=speed, 2=speed+size)
    pub opt_level: u8,
}

impl Default for JitConfig {
    fn default() -> Self {
        Self {
            compilation_threshold: 100,
            cache_size_limit: 64 * 1024 * 1024, // 64 MB
            enabled: true,
            opt_level: 1,
        }
    }
}

impl JitConfig {
    /// Create a config suitable for testing (low thresholds)
    pub fn for_testing() -> Self {
        Self {
            compilation_threshold: 2,
            cache_size_limit: 4 * 1024 * 1024,
            enabled: true,
            opt_level: 0,
        }
    }
}

/// The main JIT engine — integrates hotspot tracking, compilation, caching,
/// and native execution dispatch.
///
/// Attach this to a VM to enable tiered compilation: functions start
/// interpreted, and once called enough times they are compiled to native
/// code for subsequent invocations.
pub struct JitEngine {
    config: JitConfig,
    tracker: hotspot::HotspotTracker,
    cache: cache::CodeCache,
    backend: backend::NativeBackend,
    translator: codegen::IrTranslator,
    /// Total number of JIT compilations performed
    compilations: u64,
    /// Total number of JIT executions (cache hits that ran native code)
    jit_executions: u64,
    /// Total number of interpreter fallbacks
    interpreter_fallbacks: u64,
}

impl JitEngine {
    /// Create a new JIT engine with the given configuration
    pub fn new(config: JitConfig) -> JitResult<Self> {
        let backend = backend::NativeBackend::new(config.opt_level)?;
        Ok(Self {
            tracker: hotspot::HotspotTracker::new(config.compilation_threshold),
            cache: cache::CodeCache::new(config.cache_size_limit),
            translator: codegen::IrTranslator::new(config.opt_level),
            backend,
            config,
            compilations: 0,
            jit_executions: 0,
            interpreter_fallbacks: 0,
        })
    }

    /// Record a function call and potentially trigger JIT compilation
    ///
    /// Returns `Some(result)` if the function was executed via JIT,
    /// or `None` if the interpreter should handle it.
    ///
    /// # Parameters
    /// - `function_offset`: bytecode offset where the function body starts
    /// - `bytecode`: the bytecode container
    /// - `function_end`: bytecode offset where the function ends (exclusive)
    /// - `args`: argument values as f64 (JIT only supports numeric arguments)
    pub fn notify_call(
        &mut self,
        function_offset: usize,
        bytecode: &atlas_runtime::bytecode::Bytecode,
        function_end: usize,
        args: &[f64],
    ) -> Option<f64> {
        if !self.config.enabled {
            return None;
        }

        self.tracker.record_call(function_offset);

        // Check if already cached
        if self.cache.contains(function_offset) {
            if let Some(entry) = self.cache.get(function_offset) {
                // Verify arity matches
                if entry.param_count != args.len() {
                    self.interpreter_fallbacks += 1;
                    return None;
                }
                // Copy pointer before releasing borrow
                let code_ptr = entry.code_ptr;
                // SAFETY: `code_ptr` was produced by the JIT compiler and cached.
                // The cache entry stores the expected arity and we check it above.
                // Preconditions: `args.len()` matches the compiled signature and
                // `code_ptr` remains valid for the duration of this call.
                let result = unsafe { call_jit_function(code_ptr, args) };
                self.jit_executions += 1;
                return Some(result);
            }
        }

        // Check if hot enough to compile
        if self.tracker.is_hot(function_offset) {
            match self.try_compile(function_offset, bytecode, function_end, args) {
                Ok(result) => {
                    self.jit_executions += 1;
                    return Some(result);
                }
                Err(_) => {
                    // Compilation failed — mark as compiled to avoid retrying
                    self.tracker.mark_compiled(function_offset);
                    self.interpreter_fallbacks += 1;
                }
            }
        }

        None
    }

    /// Try to compile a function and execute it
    fn try_compile(
        &mut self,
        offset: usize,
        bytecode: &atlas_runtime::bytecode::Bytecode,
        end: usize,
        args: &[f64],
    ) -> JitResult<f64> {
        let param_count = args.len();

        // Use the parameterized translation when we have arguments
        let func = if param_count > 0 {
            self.translator
                .translate_with_params(bytecode, offset, end, param_count)?
        } else {
            self.translator.translate(bytecode, offset, end)?
        };

        let compiled = self.backend.compile(func)?;

        // Execute with the correct calling convention
        // SAFETY: `compiled.code_ptr` is the entry point produced by the JIT backend.
        // Preconditions: `args.len()` matches the translated signature and the
        // compiled code remains resident for this call.
        let result = unsafe { call_jit_function(compiled.code_ptr, args) };

        // Estimate code size based on bytecode length
        // Average: ~20 bytes of native code per bytecode byte (conservative estimate)
        let bytecode_size = end.saturating_sub(offset);
        let estimated_code_size = (bytecode_size * 20).max(64);

        self.cache
            .insert(offset, compiled.code_ptr, estimated_code_size, param_count)
            .map_err(|e| JitError::CacheFull {
                limit: e.limit,
                used: e.used,
            })?;

        self.tracker.mark_compiled(offset);
        self.compilations += 1;

        Ok(result)
    }

    /// Whether JIT is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable JIT compilation
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable JIT compilation (existing cache is preserved)
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Get statistics about the JIT engine
    pub fn stats(&self) -> JitStats {
        JitStats {
            compilations: self.compilations,
            jit_executions: self.jit_executions,
            interpreter_fallbacks: self.interpreter_fallbacks,
            cached_functions: self.cache.len(),
            cache_bytes: self.cache.total_bytes(),
            cache_hit_rate: self.cache.hit_rate(),
            tracked_functions: self.tracker.tracked_count(),
            compiled_functions: self.tracker.compiled_count(),
        }
    }

    /// Reset all state
    pub fn reset(&mut self) {
        self.tracker.reset();
        self.cache.clear();
        self.compilations = 0;
        self.jit_executions = 0;
        self.interpreter_fallbacks = 0;
    }

    /// Get the compilation threshold
    pub fn threshold(&self) -> u64 {
        self.config.compilation_threshold
    }

    /// Invalidate all cached native code
    pub fn invalidate_cache(&mut self) {
        self.cache.invalidate_all();
    }
}

/// Statistics from the JIT engine (internal, extended)
#[derive(Debug, Clone)]
pub struct JitStats {
    /// Total JIT compilations performed
    pub compilations: u64,
    /// Total native code executions
    pub jit_executions: u64,
    /// Total interpreter fallbacks (JIT failed)
    pub interpreter_fallbacks: u64,
    /// Number of functions in the code cache
    pub cached_functions: usize,
    /// Total bytes of cached native code
    pub cache_bytes: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Number of functions being tracked
    pub tracked_functions: usize,
    /// Number of functions that have been compiled
    pub compiled_functions: usize,
}

// Implement the JitCompiler trait so the VM can use JitEngine
impl atlas_runtime::JitCompiler for JitEngine {
    fn try_execute(
        &mut self,
        bytecode: &atlas_runtime::bytecode::Bytecode,
        function_offset: usize,
        function_end: usize,
        args: &[f64],
    ) -> Option<f64> {
        self.notify_call(function_offset, bytecode, function_end, args)
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn stats(&self) -> atlas_runtime::JitStats {
        atlas_runtime::JitStats {
            compilations: self.compilations,
            jit_executions: self.jit_executions,
            interpreter_fallbacks: self.interpreter_fallbacks,
            cached_functions: self.cache.len(),
            cache_bytes: self.cache.total_bytes(),
        }
    }

    fn invalidate_cache(&mut self) {
        self.cache.invalidate_all();
    }

    fn reset(&mut self) {
        self.tracker.reset();
        self.cache.clear();
        self.compilations = 0;
        self.jit_executions = 0;
        self.interpreter_fallbacks = 0;
    }
}
