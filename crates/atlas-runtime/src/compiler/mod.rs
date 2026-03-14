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
    /// If Some(type_name), this local's type implements Drop.
    /// Compiler emits drop call at scope exit in LIFO order.
    pub(super) drop_type: Option<String>,
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
    /// Trait default methods with bodies (trait_name, method_name)
    pub(super) trait_default_methods:
        std::collections::HashMap<(String, String), crate::ast::TraitMethodSig>,
    /// Names of functions declared with `async fn` (for AsyncCall dispatch at call sites).
    pub(super) async_fn_names: std::collections::HashSet<String>,
    /// True while compiling the body of an `async fn` (for WrapFuture insertion).
    pub(super) in_async_fn: bool,
    /// User-defined enum variants: variant_name -> (enum_name, arity).
    /// Populated when enum declarations are processed. Used to resolve bare variant
    /// constructor calls like `Unknown(raw)` or `Quit` without `EnumName::` prefix.
    pub(super) enum_variants: std::collections::HashMap<String, (String, usize)>,
    /// Compile-time constant values: const_name -> Value.
    /// Populated from const declarations before compilation. Used to inline const
    /// values at usage sites.
    pub(super) const_values: std::collections::HashMap<String, crate::value::Value>,
    /// Struct types with a static `new` method: type_name -> true.
    /// Enables `Foo(args)` constructor sugar in compile_call.
    pub(super) constructor_types: std::collections::HashSet<String>,
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
            trait_default_methods: std::collections::HashMap::new(),
            async_fn_names: std::collections::HashSet::new(),
            in_async_fn: false,
            enum_variants: std::collections::HashMap::new(),
            const_values: std::collections::HashMap::new(),
            constructor_types: std::collections::HashSet::new(),
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
            trait_default_methods: std::collections::HashMap::new(),
            async_fn_names: std::collections::HashSet::new(),
            in_async_fn: false,
            enum_variants: std::collections::HashMap::new(),
            const_values: std::collections::HashMap::new(),
            constructor_types: std::collections::HashSet::new(),
        }
    }

    /// Set the optimizer to use (or None to disable optimization)
    pub fn set_optimizer(&mut self, optimizer: Option<Optimizer>) {
        self.optimizer = optimizer;
    }

    /// Register all variants of an enum declaration for bare constructor resolution.
    pub fn register_enum_variants(&mut self, decl: &crate::ast::EnumDecl) {
        for variant in &decl.variants {
            let variant_name = variant.name().name.clone();
            let enum_name = decl.name.name.clone();
            let arity = match variant {
                crate::ast::EnumVariant::Unit { .. } => 0,
                crate::ast::EnumVariant::Tuple { fields, .. } => fields.len(),
                crate::ast::EnumVariant::Struct { fields, .. } => fields.len(),
            };
            self.enum_variants.insert(variant_name, (enum_name, arity));
        }
    }

    /// Register enum variants from imported enums (H-296).
    ///
    /// Called before compile() to populate enum_variants with variants from
    /// enums imported from other modules. Without this, bare constructor calls
    /// like `Unknown(raw)` would fail at runtime even though the binder accepts them.
    pub fn register_imported_enums(
        &mut self,
        imports: &[crate::ast::ImportDecl],
        module_path: &std::path::Path,
        registry: &crate::module_loader::ModuleRegistry,
    ) {
        use crate::ast::ImportSpecifier;
        use crate::binder::Binder;

        for import_decl in imports {
            // Resolve source module path (same logic as binder)
            let source_path = Binder::resolve_import_path(&import_decl.source, module_path);

            // Try both .atlas and .atl extensions
            let source_sym = registry
                .get(&source_path)
                .or_else(|| registry.get(&source_path.with_extension("atlas")))
                .or_else(|| registry.get(&source_path.with_extension("atl")));

            let Some(source_symbols) = source_sym else {
                continue;
            };

            let enum_exports = source_symbols.get_enum_exports();

            for specifier in &import_decl.specifiers {
                if let ImportSpecifier::Named { name, .. } = specifier {
                    // Check if this import is an enum
                    if let Some(enum_decl) = enum_exports.get(&name.name) {
                        self.register_enum_variants(enum_decl);
                    }
                }
            }
        }
    }

    /// Register const values from the program for inlining.
    /// Called before compile() to evaluate const expressions at compile time.
    pub fn register_consts(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Const(decl) => {
                    if let Some(value) = self.eval_const_expr(&decl.init) {
                        self.const_values.insert(decl.name.name.clone(), value);
                    }
                }
                Item::Export(export_decl) => {
                    if let crate::ast::ExportItem::Const(decl) = &export_decl.item {
                        if let Some(value) = self.eval_const_expr(&decl.init) {
                            self.const_values.insert(decl.name.name.clone(), value);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Evaluate a const expression at compile time.
    /// Returns None if the expression cannot be evaluated at compile time.
    fn eval_const_expr(&self, expr: &Expr) -> Option<crate::value::Value> {
        use crate::ast::{BinaryOp, Literal, UnaryOp};
        use crate::value::Value;

        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Number(n) => Some(Value::Number(*n)),
                Literal::String(s) => Some(Value::string(s)),
                Literal::Bool(b) => Some(Value::Bool(*b)),
                Literal::Null => Some(Value::Null),
            },
            Expr::Identifier(id) => {
                // Reference to another const
                self.const_values.get(&id.name).cloned()
            }
            Expr::Unary(unary) => {
                let val = self.eval_const_expr(&unary.expr)?;
                match unary.op {
                    UnaryOp::Negate => {
                        if let Value::Number(n) = val {
                            Some(Value::Number(-n))
                        } else {
                            None
                        }
                    }
                    UnaryOp::Not => {
                        if let Value::Bool(b) = val {
                            Some(Value::Bool(!b))
                        } else {
                            None
                        }
                    }
                }
            }
            Expr::Binary(binary) => {
                let left = self.eval_const_expr(&binary.left)?;
                let right = self.eval_const_expr(&binary.right)?;

                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => match binary.op {
                        BinaryOp::Add => Some(Value::Number(l + r)),
                        BinaryOp::Sub => Some(Value::Number(l - r)),
                        BinaryOp::Mul => Some(Value::Number(l * r)),
                        BinaryOp::Div => {
                            if r != 0.0 {
                                Some(Value::Number(l / r))
                            } else {
                                None
                            }
                        }
                        BinaryOp::Mod => Some(Value::Number(l % r)),
                        _ => None,
                    },
                    _ => None,
                }
            }
            Expr::Group(group) => self.eval_const_expr(&group.expr),
            _ => None,
        }
    }

    /// Compile an AST to bytecode
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Vec<Diagnostic>> {
        // Check if program defines a zero-arg fn main()
        let has_main = program.items.iter().any(|item| {
            matches!(item,
                Item::Function(f) if f.name.name == "main" && f.params.is_empty()
            )
        });

        self.trait_default_methods.clear();
        for item in &program.items {
            if let Item::Trait(trait_decl) = item {
                for method in &trait_decl.methods {
                    if method.body.is_some() {
                        self.trait_default_methods.insert(
                            (trait_decl.name.name.clone(), method.name.name.clone()),
                            method.clone(),
                        );
                    }
                }
            }
        }

        // Collect const values for inlining
        self.register_consts(program);

        // Compile all top-level items
        for item in &program.items {
            self.compile_item(item)?;
        }

        // H-068: Auto-call fn main() if defined (zero-arg)
        if has_main {
            let name_idx = self
                .bytecode
                .add_constant(crate::value::Value::string("main"));
            self.bytecode.emit(Opcode::GetGlobal, Span::dummy());
            self.bytecode.emit_u16(name_idx);
            self.bytecode.emit(Opcode::Call, Span::dummy());
            self.bytecode.emit_u8(0);
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
                // module's bytecode executes first.
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
                    crate::ast::ExportItem::Const(_) => {
                        // Const values are inlined at usage sites — no bytecode emitted here
                        Ok(())
                    }
                    crate::ast::ExportItem::Struct(_) => {
                        // Type declarations only — no bytecode generated
                        Ok(())
                    }
                    crate::ast::ExportItem::Enum(decl) => {
                        // Register exported enum variants for bare constructor calls.
                        self.register_enum_variants(decl);
                        Ok(())
                    }
                    crate::ast::ExportItem::ReExport { .. } => {
                        // Re-exports are resolved at module-load time by the module loader.
                        // The source module's bytecode runs first (per topological order),
                        // making the re-exported symbols available as globals.
                        Ok(())
                    }
                }
            }
            Item::Extern(_) => {
                // Extern declarations don't generate bytecode - they're loaded at runtime
                // Full implementation in phase-10b (FFI infrastructure)
                Ok(())
            }
            Item::TypeAlias(_) => Ok(()),
            Item::Const(_) => {
                // Const values are inlined at usage sites — no bytecode emitted here
                Ok(())
            }
            Item::Trait(_) => {
                // Trait declarations are type-info only — no bytecode emitted.
                // The typechecker's TraitRegistry holds all needed runtime-check info.
                Ok(())
            }
            Item::Impl(impl_block) => self.compile_impl_block(impl_block),
            Item::Struct(_) => {
                // Struct declarations are type-info only — no bytecode emitted.
                Ok(())
            }
            Item::Enum(decl) => {
                // Register variants so bare constructor calls work without EnumName:: prefix.
                self.register_enum_variants(decl);
                Ok(())
            }
        }
    }

    /// Compile a function declaration
    fn compile_function(&mut self, func: &FunctionDecl) -> Result<(), Vec<Diagnostic>> {
        // We'll update the function ref after compiling the body to get accurate local_count
        // For now, create a placeholder with bytecode_offset = 0 (will be updated)
        // Register as async fn so call sites emit AsyncCall instead of Call.
        if func.is_async {
            self.async_fn_names.insert(func.name.name.clone());
        }

        // Calculate required_arity and defaults (B39-P05)
        let arity = func.params.len();
        let required_arity = func
            .params
            .iter()
            .filter(|p| !p.is_rest)
            .take_while(|p| p.default_value.is_none())
            .count();
        let defaults: Vec<Option<crate::value::Value>> = func
            .params
            .iter()
            .map(|p| {
                p.default_value
                    .as_ref()
                    .and_then(|expr| self.eval_const_expr(expr))
            })
            .collect();

        let placeholder_ref = crate::value::FunctionRef {
            name: func.name.name.clone(),
            arity,
            required_arity,
            bytecode_offset: 0, // Placeholder - will be updated after Jump
            local_count: 0,     // Will be updated after compiling body
            param_ownership: vec![],
            param_names: vec![],
            defaults: defaults.clone(),
            return_ownership: None,
            is_async: func.is_async,
            has_rest_param: func.params.last().is_some_and(|p| p.is_rest),
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
                drop_type: None, // params: caller owns lifetime
            });
        }

        // Track function base for nested function support
        let prev_function_base = std::mem::replace(&mut self.current_function_base, old_locals_len);

        // Set async context so Stmt::Return and implicit Returns emit WrapFuture first.
        let prev_in_async = std::mem::replace(&mut self.in_async_fn, func.is_async);

        if let Some(tail) = &func.body.tail_expr {
            // Compile function body statements
            self.compile_block(&func.body)?;
            // Compile tail expression (its value becomes the return value)
            self.compile_expr(tail)?;
            if func.is_async {
                self.bytecode.emit(Opcode::WrapFuture, func.span);
            }
            // B37-P02: Emit drops for function-scoped locals before return
            self.emit_drops_for_scope(old_locals_len, self.locals.len(), func.span);
            // H-301: Emit Return after tail expression — was missing, causing infinite loops
            self.bytecode.emit(Opcode::Return, func.span);
        } else if let Some((last, rest)) = func.body.statements.split_last() {
            for stmt in rest {
                self.compile_stmt(stmt)?;
            }
            if self.stmt_always_returns(last) {
                self.compile_stmt(last)?;
                // Emit implicit Null+Return fallback (should be removed by DCE).
                if func.is_async {
                    self.bytecode.emit(Opcode::WrapFuture, func.span);
                }
                // B37-P02: Emit drops for function-scoped locals before return
                self.emit_drops_for_scope(old_locals_len, self.locals.len(), func.span);
                self.bytecode.emit(Opcode::Null, func.span);
                self.bytecode.emit(Opcode::Return, func.span);
            } else {
                self.compile_stmt_as_value(last, func.span)?;
                if func.is_async {
                    self.bytecode.emit(Opcode::WrapFuture, func.span);
                }
                // B37-P02: Emit drops for function-scoped locals before return
                self.emit_drops_for_scope(old_locals_len, self.locals.len(), func.span);
                self.bytecode.emit(Opcode::Return, func.span);
            }
        } else {
            // Empty function body
            self.bytecode.emit(Opcode::Null, func.span);
            if func.is_async {
                self.bytecode.emit(Opcode::WrapFuture, func.span);
            }
            // B37-P02: No locals to drop in empty function body
            self.bytecode.emit(Opcode::Return, func.span);
        }

        // Restore async context
        self.in_async_fn = prev_in_async;

        // Restore function base
        self.current_function_base = prev_function_base;

        // Calculate total local count using the watermark.
        // self.locals may have been truncated by match arm cleanup, so
        // self.locals.len() - old_locals_len would undercount. The watermark
        // records the maximum ever seen during this function's compilation.
        let total_local_count = self.locals_watermark - old_locals_len;

        // Restore watermark for the enclosing function (or top level)
        self.locals_watermark = prev_watermark;

        // Restore scope and locals
        self.scope_depth = old_scope;
        self.locals.truncate(old_locals_len);

        // Update the FunctionRef in constants with accurate local_count and ownership metadata
        let updated_ref = crate::value::FunctionRef {
            name: func.name.name.clone(),
            arity,
            required_arity,
            bytecode_offset: function_offset,
            local_count: total_local_count,
            param_ownership: func.params.iter().map(|p| p.ownership.clone()).collect(),
            param_names: func.params.iter().map(|p| p.name.name.clone()).collect(),
            defaults: defaults.clone(),
            return_ownership: func.return_ownership.clone(),
            is_async: func.is_async,
            has_rest_param: func.params.last().is_some_and(|p| p.is_rest),
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
        let mut provided = std::collections::HashSet::new();
        for method in &impl_block.methods {
            // Static methods use __static__ prefix, instance methods use __impl__
            let mangled_name = if method.is_static {
                // Track struct types that have a static `new` for constructor sugar (H-325)
                if method.name.name == "new" {
                    self.constructor_types
                        .insert(impl_block.type_name.name.clone());
                }
                format!(
                    "__static__{}__{}",
                    impl_block.type_name.name, method.name.name
                )
            } else {
                impl_block.mangle_method_name(&method.name.name)
            };
            self.compile_impl_method(method, &mangled_name, impl_block.span)?;
            provided.insert(method.name.name.clone());
        }

        // Default trait methods only apply to trait impls.
        if let Some(trait_id) = &impl_block.trait_name {
            let trait_name = &trait_id.name;
            let default_methods: Vec<crate::ast::TraitMethodSig> = self
                .trait_default_methods
                .iter()
                .filter(|((default_trait, _), _)| default_trait == trait_name)
                .filter(|((_, method_name), _)| !provided.contains(method_name))
                .map(|(_, method_sig)| method_sig.clone())
                .collect();

            for method_sig in default_methods {
                if let Some(body) = method_sig.body.clone() {
                    let mangled_name = impl_block.mangle_method_name(&method_sig.name.name);
                    let default_method = crate::ast::ImplMethod {
                        name: method_sig.name.clone(),
                        type_params: method_sig.type_params.clone(),
                        params: method_sig.params.clone(),
                        return_type: method_sig.return_type.clone(),
                        body,
                        span: method_sig.span,
                        is_static: false, // Default trait methods are instance methods
                    };
                    self.compile_impl_method(&default_method, &mangled_name, impl_block.span)?;
                }
            }
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
        // Calculate required_arity and defaults (B39-P05)
        let method_arity = method.params.len();
        let method_required_arity = method
            .params
            .iter()
            .filter(|p| !p.is_rest)
            .take_while(|p| p.default_value.is_none())
            .count();
        let method_defaults: Vec<Option<crate::value::Value>> = method
            .params
            .iter()
            .map(|p| {
                p.default_value
                    .as_ref()
                    .and_then(|expr| self.eval_const_expr(expr))
            })
            .collect();

        let placeholder_ref = crate::value::FunctionRef {
            name: mangled_name.to_string(),
            arity: method_arity,
            required_arity: method_required_arity,
            bytecode_offset: 0,
            local_count: 0,
            param_ownership: vec![],
            param_names: method.params.iter().map(|p| p.name.name.clone()).collect(),
            defaults: method_defaults.clone(),
            return_ownership: None,
            is_async: false,
            has_rest_param: method.params.last().is_some_and(|p| p.is_rest),
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
                drop_type: None, // params: caller owns lifetime
            });
        }

        let prev_function_base = std::mem::replace(&mut self.current_function_base, old_locals_len);

        self.compile_block(&method.body)?;

        self.current_function_base = prev_function_base;

        let total_local_count = self.locals_watermark - old_locals_len;
        self.locals_watermark = prev_watermark;

        // Handle implicit return: tail expression or null
        if let Some(tail) = &method.body.tail_expr {
            self.compile_expr(tail)?;
        } else {
            self.bytecode.emit(Opcode::Null, span);
        }
        // B37-P02: Emit drops for method-scoped locals before return
        self.emit_drops_for_scope(old_locals_len, self.locals.len(), span);
        self.bytecode.emit(Opcode::Return, span);

        self.scope_depth = old_scope;
        self.locals.truncate(old_locals_len);

        let updated_ref = crate::value::FunctionRef {
            name: mangled_name.to_string(),
            arity: method_arity,
            required_arity: method_required_arity,
            bytecode_offset: function_offset,
            local_count: total_local_count,
            param_ownership: method.params.iter().map(|p| p.ownership.clone()).collect(),
            param_names: method.params.iter().map(|p| p.name.name.clone()).collect(),
            defaults: method_defaults,
            return_ownership: None,
            is_async: false,
            has_rest_param: method.params.last().is_some_and(|p| p.is_rest),
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

    /// B37-P02: Emit drop calls for locals going out of scope.
    /// Iterates from `end` down to `start` (exclusive) in LIFO order.
    /// For each local with drop_type, emits: GetLocal(idx), Call(__impl__Type__Drop__drop)
    pub(super) fn emit_drops_for_scope(
        &mut self,
        start: usize,
        end: usize,
        span: crate::span::Span,
    ) {
        // LIFO order: drop most recently declared first
        for abs_idx in (start..end).rev() {
            if let Some(local) = self.locals.get(abs_idx) {
                if let Some(type_name) = &local.drop_type {
                    // Load the local onto stack
                    let rel_idx = (abs_idx - self.current_function_base) as u16;
                    self.bytecode.emit(Opcode::GetLocal, span);
                    self.bytecode.emit_u16(rel_idx);

                    // Call mangled drop function: __impl__TypeName__Drop__drop
                    let mangled = format!("__impl__{type_name}__Drop__drop");
                    let name_idx = self
                        .bytecode
                        .add_constant(crate::value::Value::string(&mangled));
                    self.bytecode.emit(Opcode::GetGlobal, span);
                    self.bytecode.emit_u16(name_idx);
                    self.bytecode.emit(Opcode::Call, span);
                    self.bytecode.emit_u8(1); // 1 arg (self)
                    self.bytecode.emit(Opcode::Pop, span); // drop returns void
                }
            }
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
