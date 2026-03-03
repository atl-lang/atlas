//! JIT compiler trait for VM integration
//!
//! Defines the interface between the VM and JIT compiler implementations.
//! This trait lives in atlas-runtime to avoid circular dependencies with
//! atlas-jit (which depends on atlas-runtime for bytecode/opcode types).

use crate::bytecode::Bytecode;

/// Statistics from a JIT compiler
#[derive(Debug, Clone, Default)]
pub struct JitStats {
    /// Total JIT compilations performed
    pub compilations: u64,
    /// Total native code executions
    pub jit_executions: u64,
    /// Total interpreter fallbacks (JIT failed or unsupported)
    pub interpreter_fallbacks: u64,
    /// Number of functions in the code cache
    pub cached_functions: usize,
    /// Total bytes of cached native code
    pub cache_bytes: usize,
}

/// Trait for JIT compilers that can be plugged into the VM
///
/// The VM calls `try_execute` when a function is invoked. If the JIT can
/// handle the function (it's hot enough and supported), it compiles and
/// executes it, returning the result. Otherwise, it returns None and the
/// VM falls back to interpreted execution.
///
/// # Thread Safety
/// JIT implementations must be `Send` so they can be moved between threads
/// (e.g., from main thread to VM thread). `Sync` is not required as the VM
/// uses the JIT exclusively from a single thread.
pub trait JitCompiler: Send {
    /// Try to execute a function via JIT compilation
    ///
    /// This method is called by the VM when a user-defined function is invoked.
    /// The JIT may:
    /// - Return `Some(result)` if it compiled and executed the function
    /// - Return `None` if the function should be interpreted (not hot enough,
    ///   uses unsupported opcodes, or JIT is disabled)
    ///
    /// # Parameters
    /// - `bytecode`: The full bytecode container
    /// - `function_offset`: Bytecode offset where the function body starts
    /// - `function_end`: Bytecode offset where the function ends (exclusive)
    /// - `args`: Argument values as f64 (JIT currently only supports numeric functions)
    ///
    /// # Returns
    /// - `Some(f64)` if JIT executed the function
    /// - `None` if the interpreter should handle it
    fn try_execute(
        &mut self,
        bytecode: &Bytecode,
        function_offset: usize,
        function_end: usize,
        args: &[f64],
    ) -> Option<f64>;

    /// Check if JIT is enabled
    fn is_enabled(&self) -> bool;

    /// Get JIT statistics
    fn stats(&self) -> JitStats;

    /// Invalidate all cached native code
    ///
    /// Called when bytecode changes (e.g., in REPL mode) to ensure
    /// stale compiled code isn't executed.
    fn invalidate_cache(&mut self);

    /// Reset all JIT state (tracking, cache, stats)
    fn reset(&mut self);
}
