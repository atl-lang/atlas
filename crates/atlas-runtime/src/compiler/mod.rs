//! AST to bytecode compiler
//!
//! Compiles AST directly to stack-based bytecode.
//! - Expressions leave their result on the stack
//! - Statements may or may not leave values on the stack
//! - Locals are tracked by index (stack slots)
//! - Globals are tracked by name (string constants)

mod expr;
mod stmt;

use crate::ast::*;
use crate::bytecode::{Bytecode, Opcode, Optimizer};
use crate::diagnostic::Diagnostic;
use crate::optimizer::{ConstantFoldingPass, DeadCodeEliminationPass, PeepholePass};
use crate::span::Span;

/// Local variable information
#[derive(Debug, Clone)]
pub(super) struct Local {
    pub(super) name: String,
    /// Scope depth of this local (for shadowing resolution)
    pub(super) depth: usize,
    /// Whether this local is mutable (let vs var)
    pub(super) mutable: bool,
    /// Scoped name for nested functions (None for regular variables)
    /// Used to access nested functions globally from siblings
    pub(super) scoped_name: Option<String>,
}

/// Loop context for break/continue
#[derive(Debug, Clone)]
pub(super) struct LoopContext {
    pub(super) start_offset: usize,
    pub(super) break_jumps: Vec<usize>,
}

/// How an upvalue is sourced when building a closure.
///
/// - `Local(abs_idx)`: captured directly from the immediate parent function's locals.
///   At the closure definition site, emit `GetLocal(abs_idx - parent_base)`.
/// - `Upvalue(parent_idx)`: the variable lives in a grandparent (or deeper) scope and
///   was already registered in the parent function's own upvalue list at `parent_idx`.
///   At the closure definition site, emit `GetUpvalue(parent_idx)`.
#[derive(Debug, Clone)]
pub(super) enum UpvalueCapture {
    Local(usize),   // abs local index in self.locals
    Upvalue(usize), // index into the immediate parent's upvalue list
}

/// Per-nesting-level upvalue context, pushed when entering a nested function compilation.
#[derive(Debug, Clone)]
pub(super) struct UpvalueContext {
    /// `current_function_base` of the *parent* function at the time this context was pushed.
    /// Any `abs_local_idx >= parent_base` belongs to the immediate parent; anything smaller
    /// belongs to a grandparent and requires upvalue chaining.
    pub(super) parent_base: usize,
    /// Captured variables for this level, in insertion order.
    pub(super) captures: Vec<(String, UpvalueCapture)>,
}

/// Compiler state
pub struct Compiler {
    /// Output bytecode
    pub(super) bytecode: Bytecode,
    /// Local variables (stack slots)
    pub(super) locals: Vec<Local>,
    /// Current scope depth
    pub(super) scope_depth: usize,
    /// Loop context stack (for break/continue)
    pub(super) loops: Vec<LoopContext>,
    /// Bytecode optimizer (optional)
    optimizer: Option<Optimizer>,
    /// Monomorphizer for generic functions
    #[allow(dead_code)] // Will be used when generic runtime support is fully integrated
    pub(super) monomorphizer: crate::typechecker::generics::Monomorphizer,
    /// Counter for generating unique nested function names
    next_func_id: usize,
    /// Base index for current function's locals (for nested functions)
    /// Used to distinguish parent-scope locals from function-local variables
    pub(super) current_function_base: usize,
    /// Global variable mutability tracking (true = mutable, false = immutable)
    pub(super) global_mutability: std::collections::HashMap<String, bool>,
    /// High-water mark: maximum self.locals.len() seen within the current function.
    /// Updated whenever a local is pushed. Reset at function start.
    /// Used to compute accurate `local_count` even after match arm truncation.
    pub(super) locals_watermark: usize,
    /// Stack of upvalue contexts, one entry per active nested function compilation.
    /// Empty when not inside any nested function.
    pub(super) upvalue_stack: Vec<UpvalueContext>,
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loops: Vec::new(),
            optimizer: None, // Optimization disabled by default
            monomorphizer: crate::typechecker::generics::Monomorphizer::new(),
            next_func_id: 0,
            current_function_base: 0,
            global_mutability: std::collections::HashMap::new(),
            locals_watermark: 0,
            upvalue_stack: Vec::new(),
        }
    }

    /// Enable bytecode optimization with all three passes:
    /// constant folding, dead code elimination, and peephole optimizations.
    pub fn with_optimization() -> Self {
        let mut optimizer = Optimizer::new();
        optimizer.set_enabled(true);
        optimizer.add_pass(Box::new(ConstantFoldingPass));
        optimizer.add_pass(Box::new(DeadCodeEliminationPass));
        optimizer.add_pass(Box::new(PeepholePass));

        Self {
            bytecode: Bytecode::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loops: Vec::new(),
            optimizer: Some(optimizer),
            monomorphizer: crate::typechecker::generics::Monomorphizer::new(),
            next_func_id: 0,
            current_function_base: 0,
            global_mutability: std::collections::HashMap::new(),
            locals_watermark: 0,
            upvalue_stack: Vec::new(),
        }
    }

    /// Set the optimizer to use (or None to disable optimization)
    pub fn set_optimizer(&mut self, optimizer: Option<Optimizer>) {
        self.optimizer = optimizer;
    }

    /// Compile an AST to bytecode
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Vec<Diagnostic>> {
        // Compile all top-level items
        for item in &program.items {
            self.compile_item(item)?;
        }

        // Emit halt at the end
        self.bytecode.emit(Opcode::Halt, Span::dummy());

        // Take ownership of the bytecode
        let mut bytecode = std::mem::take(&mut self.bytecode);

        // Record peak local count so the VM can initialize the main frame correctly.
        bytecode.top_level_local_count = self.locals_watermark;

        // Apply optimization if enabled
        if let Some(ref optimizer) = self.optimizer {
            bytecode = optimizer.optimize(bytecode);
        }

        Ok(bytecode)
    }

    /// Compile a top-level item
    fn compile_item(&mut self, item: &Item) -> Result<(), Vec<Diagnostic>> {
        match item {
            Item::Function(func) => self.compile_function(func),
            Item::Statement(stmt) => self.compile_stmt(stmt),
            Item::Import(_) => {
                // Imports don't generate bytecode directly. In VM mode, Runtime::eval_file()
                // uses ModuleLoader to load ALL modules in dependency order, then compiles
                // each module to bytecode. Imported symbols become globals when the dependency
                // module's bytecode executes first. See DR-014 in memory/decisions.md.
                Ok(())
            }
            Item::Export(export_decl) => {
                // Export wraps an item - compile the inner item
                match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => self.compile_function(func),
                    crate::ast::ExportItem::Variable(var) => {
                        self.compile_stmt(&crate::ast::Stmt::VarDecl(var.clone()))
                    }
                    crate::ast::ExportItem::TypeAlias(_) => Ok(()),
                }
            }
            Item::Extern(_) => {
                // Extern declarations don't generate bytecode - they're loaded at runtime
                // Full implementation in phase-10b (FFI infrastructure)
                Ok(())
            }
            Item::TypeAlias(_) => Ok(()),
            Item::Trait(_) => {
                // Trait declarations are type-info only — no bytecode emitted.
                // The typechecker's TraitRegistry holds all needed runtime-check info.
                Ok(())
            }
            Item::Impl(impl_block) => self.compile_impl_block(impl_block),
        }
    }

    /// Compile a function declaration
    fn compile_function(&mut self, func: &FunctionDecl) -> Result<(), Vec<Diagnostic>> {
        // We'll update the function ref after compiling the body to get accurate local_count
        // For now, create a placeholder with bytecode_offset = 0 (will be updated)
        let placeholder_ref = crate::value::FunctionRef {
            name: func.name.name.clone(),
            arity: func.params.len(),
            bytecode_offset: 0, // Placeholder - will be updated after Jump
            local_count: 0,     // Will be updated after compiling body
            param_ownership: vec![],
            param_names: vec![],
            return_ownership: None,
        };
        let placeholder_value = crate::value::Value::Function(placeholder_ref);

        // Add placeholder to constant pool
        let const_idx = self.bytecode.add_constant(placeholder_value);
        self.bytecode.emit(Opcode::Constant, func.span);
        self.bytecode.emit_u16(const_idx);

        // Store function as a global variable (so it can be called)
        let name_idx = self
            .bytecode
            .add_constant(crate::value::Value::string(&func.name.name));
        self.bytecode.emit(Opcode::SetGlobal, func.span);
        self.bytecode.emit_u16(name_idx);
        self.bytecode.emit(Opcode::Pop, func.span);

        // Jump over the function body (so it's not executed during program init)
        self.bytecode.emit(Opcode::Jump, func.span);
        let skip_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF); // Placeholder

        // NOW record the function body offset (after all setup code)
        let function_offset = self.bytecode.current_offset();

        // Now compile the function body at function_offset
        // The function body is compiled inline in the bytecode
        // When called, the VM will jump here

        // Set up parameters as local variables
        // Parameters are expected to be on the stack when the function is called
        // They're already there from the CALL instruction
        // We just need to track them as locals
        let old_locals_len = self.locals.len();
        let old_scope = self.scope_depth;
        self.scope_depth += 1;

        // Reset watermark for this function. Save previous value for nesting.
        let prev_watermark = std::mem::replace(&mut self.locals_watermark, old_locals_len);

        // Add parameters as locals
        for param in &func.params {
            self.push_local(Local {
                name: param.name.name.clone(),
                depth: self.scope_depth,
                mutable: true, // Parameters are always mutable
                scoped_name: None,
            });
        }

        // Track function base for nested function support
        let prev_function_base = std::mem::replace(&mut self.current_function_base, old_locals_len);

        // Compile function body
        self.compile_block(&func.body)?;

        // Restore function base
        self.current_function_base = prev_function_base;

        // Calculate total local count using the watermark.
        // self.locals may have been truncated by match arm cleanup, so
        // self.locals.len() - old_locals_len would undercount. The watermark
        // records the maximum ever seen during this function's compilation.
        let total_local_count = self.locals_watermark - old_locals_len;

        // Restore watermark for the enclosing function (or top level)
        self.locals_watermark = prev_watermark;

        // If function doesn't end with explicit return, add implicit "return null"
        self.bytecode.emit(Opcode::Null, func.span);
        self.bytecode.emit(Opcode::Return, func.span);

        // Restore scope and locals
        self.scope_depth = old_scope;
        self.locals.truncate(old_locals_len);

        // Update the FunctionRef in constants with accurate local_count and ownership metadata
        let updated_ref = crate::value::FunctionRef {
            name: func.name.name.clone(),
            arity: func.params.len(),
            bytecode_offset: function_offset,
            local_count: total_local_count,
            param_ownership: func.params.iter().map(|p| p.ownership.clone()).collect(),
            param_names: func.params.iter().map(|p| p.name.name.clone()).collect(),
            return_ownership: func.return_ownership.clone(),
        };
        self.bytecode.constants[const_idx as usize] = crate::value::Value::Function(updated_ref);

        // Patch the skip jump to go past the function body
        self.bytecode.patch_jump(skip_jump);

        Ok(())
    }

    /// Compile an impl block: each method becomes a mangled top-level function.
    ///
    /// Mangling: `__impl__{TypeName}__{TraitName}__{MethodName}`
    /// e.g. `impl Display for number` → `__impl__number__Display__display`
    fn compile_impl_block(
        &mut self,
        impl_block: &crate::ast::ImplBlock,
    ) -> Result<(), Vec<Diagnostic>> {
        let type_name = &impl_block.type_name.name;
        let trait_name = &impl_block.trait_name.name;

        for method in &impl_block.methods {
            let mangled_name = format!(
                "__impl__{}__{}__{}",
                type_name, trait_name, method.name.name
            );
            self.compile_impl_method(method, &mangled_name, impl_block.span)?;
        }
        Ok(())
    }

    /// Compile one impl method as a named function with the given mangled name.
    fn compile_impl_method(
        &mut self,
        method: &crate::ast::ImplMethod,
        mangled_name: &str,
        span: crate::span::Span,
    ) -> Result<(), Vec<Diagnostic>> {
        let placeholder_ref = crate::value::FunctionRef {
            name: mangled_name.to_string(),
            arity: method.params.len(),
            bytecode_offset: 0,
            local_count: 0,
            param_ownership: vec![],
            param_names: method.params.iter().map(|p| p.name.name.clone()).collect(),
            return_ownership: None,
        };
        let placeholder_value = crate::value::Value::Function(placeholder_ref);

        let const_idx = self.bytecode.add_constant(placeholder_value);
        self.bytecode.emit(Opcode::Constant, span);
        self.bytecode.emit_u16(const_idx);

        let name_idx = self
            .bytecode
            .add_constant(crate::value::Value::string(mangled_name));
        self.bytecode.emit(Opcode::SetGlobal, span);
        self.bytecode.emit_u16(name_idx);
        self.bytecode.emit(Opcode::Pop, span);

        // Jump over the method body
        self.bytecode.emit(Opcode::Jump, span);
        let skip_jump = self.bytecode.current_offset();
        self.bytecode.emit_u16(0xFFFF);

        let function_offset = self.bytecode.current_offset();

        let old_locals_len = self.locals.len();
        let old_scope = self.scope_depth;
        self.scope_depth += 1;
        let prev_watermark = std::mem::replace(&mut self.locals_watermark, old_locals_len);

        for param in &method.params {
            self.push_local(Local {
                name: param.name.name.clone(),
                depth: self.scope_depth,
                mutable: true,
                scoped_name: None,
            });
        }

        let prev_function_base = std::mem::replace(&mut self.current_function_base, old_locals_len);

        self.compile_block(&method.body)?;

        self.current_function_base = prev_function_base;

        let total_local_count = self.locals_watermark - old_locals_len;
        self.locals_watermark = prev_watermark;

        self.bytecode.emit(Opcode::Null, span);
        self.bytecode.emit(Opcode::Return, span);

        self.scope_depth = old_scope;
        self.locals.truncate(old_locals_len);

        let updated_ref = crate::value::FunctionRef {
            name: mangled_name.to_string(),
            arity: method.params.len(),
            bytecode_offset: function_offset,
            local_count: total_local_count,
            param_ownership: method.params.iter().map(|p| p.ownership.clone()).collect(),
            param_names: method.params.iter().map(|p| p.name.name.clone()).collect(),
            return_ownership: None,
        };
        self.bytecode.constants[const_idx as usize] = crate::value::Value::Function(updated_ref);

        self.bytecode.patch_jump(skip_jump);

        Ok(())
    }

    /// Push a local variable, updating the high-water mark for accurate `local_count`.
    pub(super) fn push_local(&mut self, local: Local) {
        self.locals.push(local);
        if self.locals.len() > self.locals_watermark {
            self.locals_watermark = self.locals.len();
        }
    }

    /// Resolve a local variable by name, returning its index if found
    pub(super) fn resolve_local(&self, name: &str) -> Option<usize> {
        // Search from most recent to oldest (for shadowing)
        for (idx, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(idx);
            }
        }
        None
    }

    /// Resolve a local variable by name, returning its index and mutability if found
    pub(super) fn resolve_local_with_mutability(&self, name: &str) -> Option<(usize, bool)> {
        // Search from most recent to oldest (for shadowing)
        for (idx, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some((idx, local.mutable));
            }
        }
        None
    }

    /// Check if a global variable is mutable
    pub(super) fn is_global_mutable(&self, name: &str) -> Option<bool> {
        self.global_mutability.get(name).copied()
    }

    /// Register an upvalue capture for the current (innermost) nested function.
    ///
    /// Handles multi-level chaining: if `abs_local_idx` belongs to a grandparent scope
    /// (i.e. below the immediate parent's local base), the variable is first registered
    /// in the parent's upvalue context and then referenced as `Upvalue(parent_idx)` here.
    ///
    /// Returns the upvalue index within the current function's capture list.
    pub(super) fn register_upvalue(&mut self, name: &str, abs_local_idx: usize) -> usize {
        self.register_upvalue_at_depth(name, abs_local_idx, 0)
    }

    /// Recursive helper for `register_upvalue`.
    /// `depth` = 0 → innermost (current) function; 1 → parent; 2 → grandparent; …
    fn register_upvalue_at_depth(
        &mut self,
        name: &str,
        abs_local_idx: usize,
        depth: usize,
    ) -> usize {
        let stack_len = self.upvalue_stack.len();
        let stack_idx = stack_len - 1 - depth;

        // Return existing capture index if already registered at this depth.
        if let Some(pos) = self.upvalue_stack[stack_idx]
            .captures
            .iter()
            .position(|(n, _)| n == name)
        {
            return pos;
        }

        let parent_base = self.upvalue_stack[stack_idx].parent_base;

        let capture = if abs_local_idx >= parent_base {
            // The variable is a direct local of the immediate parent function.
            UpvalueCapture::Local(abs_local_idx)
        } else {
            // The variable lives in a grandparent scope: register it in the parent context
            // (one level up) and chain through an Upvalue reference.
            let parent_upvalue_idx = self.register_upvalue_at_depth(name, abs_local_idx, depth + 1);
            UpvalueCapture::Upvalue(parent_upvalue_idx)
        };

        let idx = self.upvalue_stack[stack_idx].captures.len();
        self.upvalue_stack[stack_idx]
            .captures
            .push((name.to_string(), capture));
        idx
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
