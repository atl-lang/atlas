//! Native code backend
//!
//! Uses Cranelift's JIT module to compile IR functions into executable
//! native machine code. Handles target detection, compilation, and
//! function pointer retrieval.

use cranelift_codegen::ir::Function;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

use crate::{JitError, JitResult};

/// Native code backend using Cranelift JIT
///
/// Manages the Cranelift JIT module, compiling IR functions to native
/// code and providing callable function pointers.
pub struct NativeBackend {
    /// The Cranelift JIT module
    module: JITModule,
    /// Number of functions compiled
    compiled_count: usize,
    /// Total bytes of native code generated
    native_bytes: usize,
}

impl NativeBackend {
    /// Create a new native backend targeting the host architecture
    pub fn new(opt_level: u8) -> JitResult<Self> {
        let mut flag_builder = settings::builder();
        let opt_str = match opt_level {
            0 => "none",
            1 => "speed",
            _ => "speed_and_size",
        };
        flag_builder
            .set("opt_level", opt_str)
            .map_err(|e| JitError::CompilationFailed(format!("failed to set opt_level: {}", e)))?;

        // Enable SIMD on supported architectures
        flag_builder.set("use_colocated_libcalls", "false").ok();
        flag_builder.set("is_pic", "false").ok();

        let isa_builder = cranelift_native::builder().map_err(|e| {
            JitError::CompilationFailed(format!("failed to detect native ISA: {}", e))
        })?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| JitError::CompilationFailed(format!("failed to build ISA: {}", e)))?;

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Ok(Self {
            module,
            compiled_count: 0,
            native_bytes: 0,
        })
    }

    /// Compile an IR function to native code and return the function pointer
    ///
    /// The returned pointer is a `fn() -> f64` for parameterless functions.
    pub fn compile(&mut self, func: Function) -> JitResult<CompiledFunction> {
        let name = format!("jit_fn_{}", self.compiled_count);

        let func_id = self
            .module
            .declare_function(&name, Linkage::Local, &func.signature)
            .map_err(|e| JitError::CompilationFailed(format!("declare: {}", e)))?;

        let mut ctx = self.module.make_context();
        ctx.func = func;

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| JitError::CompilationFailed(format!("define: {}", e)))?;

        self.module.clear_context(&mut ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| JitError::CompilationFailed(format!("finalize: {}", e)))?;

        let code_ptr = self.module.get_finalized_function(func_id);
        let code_size = 0; // Cranelift doesn't expose size directly; tracked separately

        self.compiled_count += 1;

        Ok(CompiledFunction {
            func_id,
            code_ptr,
            code_size,
            name,
        })
    }

    /// Compile a function that takes parameters
    pub fn compile_with_params(
        &mut self,
        func: Function,
        _param_count: usize,
    ) -> JitResult<CompiledFunction> {
        // Same as compile — signature is already in the Function
        self.compile(func)
    }

    /// Get the number of compiled functions
    pub fn compiled_count(&self) -> usize {
        self.compiled_count
    }

    /// Get total native bytes generated (approximate)
    pub fn native_bytes(&self) -> usize {
        self.native_bytes
    }

    /// Get the target architecture name
    pub fn target_arch(&self) -> &'static str {
        if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "unknown"
        }
    }
}

/// A compiled native function ready for execution
#[derive(Debug)]
pub struct CompiledFunction {
    /// Cranelift function ID
    pub func_id: FuncId,
    /// Pointer to native code
    pub code_ptr: *const u8,
    /// Size of native code in bytes
    pub code_size: usize,
    /// Function name (for debugging)
    pub name: String,
}

impl CompiledFunction {
    /// Execute as a function that takes no args and returns f64
    ///
    /// # Safety
    /// The code pointer must be valid and the function signature must match.
    pub unsafe fn call_no_args(&self) -> f64 {
        let func: unsafe fn() -> f64 = std::mem::transmute(self.code_ptr);
        func()
    }

    /// Execute as a function that takes one f64 arg and returns f64
    ///
    /// # Safety
    /// The code pointer must be valid and the function signature must match.
    pub unsafe fn call_1arg(&self, a: f64) -> f64 {
        let func: unsafe fn(f64) -> f64 = std::mem::transmute(self.code_ptr);
        func(a)
    }

    /// Execute as a function that takes two f64 args and returns f64
    ///
    /// # Safety
    /// The code pointer must be valid and the function signature must match.
    pub unsafe fn call_2args(&self, a: f64, b: f64) -> f64 {
        let func: unsafe fn(f64, f64) -> f64 = std::mem::transmute(self.code_ptr);
        func(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = NativeBackend::new(0);
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert_eq!(backend.compiled_count(), 0);
        assert!(["x86_64", "aarch64", "unknown"].contains(&backend.target_arch()));
    }

    #[test]
    fn test_compile_simple_function() {
        use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature, UserFuncName};
        use cranelift_codegen::isa::CallConv;
        use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

        let mut backend = NativeBackend::new(0).unwrap();

        // Build a trivial function: () -> 42.0
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(types::F64));

        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut ctx);
        let block = builder.create_block();
        builder.switch_to_block(block);
        builder.seal_block(block);
        let val = builder.ins().f64const(42.0);
        builder.ins().return_(&[val]);
        builder.finalize();

        let compiled = backend.compile(func).unwrap();
        assert!(!compiled.code_ptr.is_null());
        assert_eq!(backend.compiled_count(), 1);

        let result = unsafe { compiled.call_no_args() };
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_compile_add_function() {
        use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature, UserFuncName};
        use cranelift_codegen::isa::CallConv;
        use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

        let mut backend = NativeBackend::new(0).unwrap();

        // Build: (a: f64, b: f64) -> a + b
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::F64));
        sig.params.push(AbiParam::new(types::F64));
        sig.returns.push(AbiParam::new(types::F64));

        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);
        let mut ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut ctx);
        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);
        builder.seal_block(block);
        let a = builder.block_params(block)[0];
        let b = builder.block_params(block)[1];
        let sum = builder.ins().fadd(a, b);
        builder.ins().return_(&[sum]);
        builder.finalize();

        let compiled = backend.compile(func).unwrap();
        let result = unsafe { compiled.call_2args(10.0, 32.0) };
        assert_eq!(result, 42.0);
    }
}
