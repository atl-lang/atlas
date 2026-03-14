//! Name binding and scope resolution
//!
//! The binder performs two-pass analysis:
//! 1. Collect all top-level function declarations (hoisting)
//! 2. Bind all items and resolve identifiers

use crate::ast::*;
use crate::diagnostic::error_codes;
use crate::diagnostic::Diagnostic;
use crate::module_loader::ModuleRegistry;
use crate::span::Span;
use crate::symbol::{Symbol, SymbolKind, SymbolTable};
use crate::types::{StructuralMemberType, Type, TypeParamDef};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Binder for name resolution and scope management
pub struct Binder {
    /// Symbol table
    symbol_table: SymbolTable,
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Type parameter scopes (stack of scopes, each scope maps param name -> TypeParam)
    type_param_scopes: Vec<HashMap<String, TypeParam>>,
    /// Stack of aliases being resolved (for circular detection)
    type_alias_stack: Vec<String>,
    /// Struct declarations collected in pre-pass (name → StructDecl), so that
    /// `resolve_type_ref` can resolve named struct types before Phase 2 processes them.
    struct_decls: HashMap<String, StructDecl>,
    /// Enum names collected in pre-pass, so that `resolve_type_ref` can resolve
    /// named enum types in function signatures before Phase 2 processes them.
    enum_names: HashSet<String>,
    /// Enum variant names collected in pre-pass (variant_name → enum_name), so that
    /// bare enum variants can be used without EnumName:: qualification (H-295).
    enum_variants: HashMap<String, String>,
}

impl Binder {
    /// Create a new binder
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            diagnostics: Vec::new(),
            type_param_scopes: Vec::new(),
            type_alias_stack: Vec::new(),
            struct_decls: HashMap::new(),
            enum_names: HashSet::new(),
            enum_variants: HashMap::new(),
        }
    }

    /// Create a binder with an existing symbol table (for REPL state persistence)
    pub fn with_symbol_table(symbol_table: SymbolTable) -> Self {
        Self {
            symbol_table,
            diagnostics: Vec::new(),
            type_param_scopes: Vec::new(),
            type_alias_stack: Vec::new(),
            struct_decls: HashMap::new(),
            enum_names: HashSet::new(),
            enum_variants: HashMap::new(),
        }
    }

    /// Bind a program (two-pass: hoist functions, then bind everything)
    pub fn bind(&mut self, program: &Program) -> (SymbolTable, Vec<Diagnostic>) {
        // Phase 0a: Collect struct declarations (so named struct types resolve in signatures)
        self.collect_struct_decl_prepass(program);
        // Phase 0b: Collect type aliases (so they can be used in signatures)
        self.collect_type_aliases(program);
        // Phase 0c: Collect const declarations (so they can be used in expressions)
        self.collect_consts(program);

        // Phase 1: Collect all top-level function declarations (hoisting)
        for item in &program.items {
            if let Item::Function(func) = item {
                self.hoist_function(func);
            } else if let Item::Extern(extern_decl) = item {
                self.hoist_extern(extern_decl);
            } else if let Item::Export(export_decl) = item {
                // Also hoist exported functions
                if let ExportItem::Function(func) = &export_decl.item {
                    self.hoist_function(func);
                }
            }
        }

        // Phase 2: Bind all items
        for item in &program.items {
            self.bind_item(item);
        }

        // Phase 3: Mark exported symbols
        for item in &program.items {
            if let Item::Export(export_decl) = item {
                match &export_decl.item {
                    ExportItem::Struct(decl) => {
                        self.symbol_table.add_struct_export(decl.clone());
                    }
                    ExportItem::Enum(decl) => {
                        self.symbol_table.add_enum_export(decl.clone());
                    }
                    ExportItem::ReExport { .. } => {
                        // Re-exports are satisfied by the source module; no symbol marking needed.
                    }
                    _ => {
                        let name = match &export_decl.item {
                            ExportItem::Function(func) => &func.name.name,
                            ExportItem::Variable(var) => &var.name.name,
                            ExportItem::TypeAlias(alias) => &alias.name.name,
                            ExportItem::Const(decl) => &decl.name.name,
                            ExportItem::Struct(_)
                            | ExportItem::Enum(_)
                            | ExportItem::ReExport { .. } => unreachable!(),
                        };

                        let mut exported = self.symbol_table.mark_exported(name);
                        if !exported {
                            exported = self.symbol_table.mark_type_alias_exported(name);
                        }
                        if !exported {
                            exported = self.symbol_table.mark_const_exported(name);
                        }

                        if !exported {
                            self.diagnostics.push(
                                error_codes::EXPORT_NOT_FOUND
                                    .emit(export_decl.span)
                                    .arg(
                                        "detail",
                                        format!("Cannot export '{}': symbol not found", name),
                                    )
                                    .build()
                                    .with_label("export declaration")
                                    .with_help(format!("define '{}' before exporting it", name)),
                            );
                        }
                    }
                }
            }
        }

        (
            std::mem::take(&mut self.symbol_table),
            std::mem::take(&mut self.diagnostics),
        )
    }

    /// Bind a program with cross-module support (BLOCKER 04-C)
    ///
    /// Takes a module registry to resolve imports from other modules.
    /// Returns symbol table and diagnostics.
    ///
    /// # Arguments
    /// * `program` - The AST to bind
    /// * `module_path` - Absolute path to this module
    /// * `registry` - Registry of already-bound modules for import resolution
    pub fn bind_with_modules(
        &mut self,
        program: &Program,
        module_path: &Path,
        registry: &ModuleRegistry,
    ) -> (SymbolTable, Vec<Diagnostic>) {
        // Phase 0a: Collect struct declarations (so named struct types resolve in signatures)
        self.collect_struct_decl_prepass(program);
        // Phase 0b: Collect type aliases (so they can be used in signatures)
        self.collect_type_aliases(program);
        // Phase 0c: Collect const declarations (so they can be used in expressions)
        self.collect_consts(program);

        // Bind imports early to support aliases in signatures.
        // Also bind re-exports — they are implicit imports from the source module.
        for item in &program.items {
            match item {
                Item::Import(import_decl) => {
                    self.bind_import(import_decl, module_path, registry);
                }
                Item::Export(export_decl) => {
                    if let ExportItem::ReExport {
                        names,
                        source,
                        span,
                    } = &export_decl.item
                    {
                        // Synthesize an ImportDecl so bind_import resolves the symbols.
                        let synthetic = ImportDecl {
                            specifiers: names
                                .iter()
                                .map(|s| ImportSpecifier::Named {
                                    name: s.name.clone(),
                                    span: s.span,
                                })
                                .collect(),
                            source: source.clone(),
                            span: *span,
                        };
                        self.bind_import(&synthetic, module_path, registry);
                    }
                }
                _ => {}
            }
        }

        // Phase 1: Collect all top-level function declarations (hoisting)
        for item in &program.items {
            if let Item::Function(func) = item {
                self.hoist_function(func);
            } else if let Item::Extern(extern_decl) = item {
                self.hoist_extern(extern_decl);
            } else if let Item::Export(export_decl) = item {
                // Also hoist exported functions
                if let ExportItem::Function(func) = &export_decl.item {
                    self.hoist_function(func);
                }
            }
        }

        // Phase 2: Bind all items (including imports and exports)
        for item in &program.items {
            self.bind_item_with_modules(item, module_path, registry);
        }

        // Phase 3: Mark exported symbols
        for item in &program.items {
            if let Item::Export(export_decl) = item {
                match &export_decl.item {
                    ExportItem::Struct(decl) => {
                        self.symbol_table.add_struct_export(decl.clone());
                    }
                    ExportItem::Enum(decl) => {
                        self.symbol_table.add_enum_export(decl.clone());
                    }
                    ExportItem::ReExport { names, .. } => {
                        // The symbols were bound as imports above; now mark each one exported.
                        for spec in names {
                            let exported_name = spec
                                .alias
                                .as_ref()
                                .map(|a| &a.name)
                                .unwrap_or(&spec.name.name);
                            self.symbol_table.mark_exported(exported_name);
                        }
                    }
                    _ => {
                        let name = match &export_decl.item {
                            ExportItem::Function(func) => &func.name.name,
                            ExportItem::Variable(var) => &var.name.name,
                            ExportItem::TypeAlias(alias) => &alias.name.name,
                            ExportItem::Const(decl) => &decl.name.name,
                            ExportItem::Struct(_)
                            | ExportItem::Enum(_)
                            | ExportItem::ReExport { .. } => unreachable!(),
                        };

                        let mut exported = self.symbol_table.mark_exported(name);
                        if !exported {
                            exported = self.symbol_table.mark_type_alias_exported(name);
                        }
                        if !exported {
                            exported = self.symbol_table.mark_const_exported(name);
                        }

                        if !exported {
                            self.diagnostics.push(
                                error_codes::EXPORT_NOT_FOUND
                                    .emit(export_decl.span)
                                    .arg(
                                        "detail",
                                        format!("Cannot export '{}': symbol not found", name),
                                    )
                                    .build()
                                    .with_label("export declaration")
                                    .with_help(format!("define '{}' before exporting it", name)),
                            );
                        }
                    }
                }
            }
        }

        // Phase 4: H-406 — Export inherent impl method signatures for exported structs.
        // Collected here (not typechecker) so they are available to importers before typechecking.
        let exported_struct_names: std::collections::HashSet<String> = self
            .symbol_table
            .get_struct_exports()
            .keys()
            .cloned()
            .collect();
        for item in &program.items {
            let impl_block = match item {
                Item::Impl(b) => b,
                _ => continue,
            };
            // Only inherent impls (no trait_name) for exported structs
            if impl_block.trait_name.is_some() {
                continue;
            }
            let struct_name = &impl_block.type_name.name;
            if !exported_struct_names.contains(struct_name) {
                continue;
            }
            for method in &impl_block.methods {
                self.symbol_table.add_impl_method_export(
                    struct_name.clone(),
                    method.name.name.clone(),
                    method.clone(),
                );
            }
        }

        (
            std::mem::take(&mut self.symbol_table),
            std::mem::take(&mut self.diagnostics),
        )
    }

    /// Hoist a top-level function declaration
    fn hoist_function(&mut self, func: &FunctionDecl) {
        // Check for global shadowing of prelude builtins
        if self.symbol_table.is_prelude_builtin(&func.name.name) {
            let diag = error_codes::SHADOWING_PRELUDE.emit(func.name.span).arg("detail", format!(
                    "Cannot shadow prelude builtin '{}' in global scope",
                    func.name.name
                )).build()
            .with_label("shadows prelude builtin")
            .with_help("Prelude builtins cannot be redefined at the top level. Use a different name or shadow in a nested scope.".to_string());

            self.diagnostics.push(diag);
            return;
        }

        // Enter type parameter scope to resolve generic types
        self.enter_type_param_scope();
        for type_param in &func.type_params {
            self.register_type_parameter(type_param);
        }

        let param_types: Vec<Type> = func
            .params
            .iter()
            .map(|p| self.resolve_type_ref(&p.type_ref))
            .collect();

        let return_type = func
            .return_type
            .as_ref()
            .map_or(Type::Unknown, |t| self.resolve_type_ref(t));

        // Exit type parameter scope
        self.exit_type_param_scope();

        let type_params = func
            .type_params
            .iter()
            .map(|param| TypeParamDef {
                name: param.name.clone(),
                trait_bounds: param
                    .trait_bounds
                    .iter()
                    .map(|tb| tb.trait_name.clone())
                    .collect(),
            })
            .collect();

        let symbol = Symbol {
            name: func.name.name.clone(),
            ty: Type::Function {
                type_params,
                params: param_types,
                return_type: Box::new(return_type),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: func.name.span,
            exported: false,
            visibility: func.visibility,
        };

        if let Err(err) = self.symbol_table.define_function(symbol) {
            let (msg, existing) = *err;
            let mut diag = error_codes::DUPLICATE_DECLARATION
                .emit(func.name.span)
                .arg("detail", &msg)
                .build()
                .with_label("redeclaration")
                .with_help(format!(
                    "rename or remove one of the '{}' declarations",
                    func.name.name
                ));

            // Add related location if we have the existing symbol
            if let Some(existing_symbol) = existing {
                diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                    file: "<input>".to_string(),
                    line: 1,
                    column: existing_symbol.span.start + 1,
                    length: existing_symbol
                        .span
                        .end
                        .saturating_sub(existing_symbol.span.start),
                    message: format!("'{}' first defined here", existing_symbol.name),

                    snippet: String::new(),
                    label: String::new(),
                    is_occurrence: false,
                });
            }

            self.diagnostics.push(diag);
        }
    }

    /// Hoist an extern function declaration (FFI)
    ///
    /// Extern functions are registered in the symbol table with their C ABI signature
    /// converted to Atlas types.
    fn hoist_extern(&mut self, extern_decl: &ExternDecl) {
        // Convert extern type annotations to Atlas types
        let param_types: Vec<Type> = extern_decl
            .params
            .iter()
            .map(|(_, ty)| self.extern_type_to_atlas(ty))
            .collect();

        let return_type = self.extern_type_to_atlas(&extern_decl.return_type);

        let symbol = Symbol {
            name: extern_decl.name.clone(),
            ty: Type::Function {
                type_params: vec![], // Extern functions have no generics
                params: param_types,
                return_type: Box::new(return_type),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: extern_decl.span,
            exported: false,
            visibility: Visibility::Private, // Extern functions default to private
        };

        if let Err(err) = self.symbol_table.define_function(symbol) {
            let (msg, _) = *err;
            let diag = error_codes::DUPLICATE_DECLARATION
                .emit(extern_decl.span)
                .arg("detail", &msg)
                .build()
                .with_label("redeclaration of extern function")
                .with_help(format!(
                    "rename or remove one of the '{}' declarations",
                    extern_decl.name
                ));
            self.diagnostics.push(diag);
        }
    }

    /// Convert an extern type annotation to an Atlas type
    fn extern_type_to_atlas(&self, ty: &ExternTypeAnnotation) -> Type {
        match ty {
            ExternTypeAnnotation::CInt => Type::Number,
            ExternTypeAnnotation::CLong => Type::Number,
            ExternTypeAnnotation::CDouble => Type::Number,
            ExternTypeAnnotation::CCharPtr => Type::String,
            ExternTypeAnnotation::CVoid => Type::Null,
            ExternTypeAnnotation::CBool => Type::Bool,
        }
    }

    /// Hoist a scoped (nested) function declaration
    ///
    /// Unlike top-level functions, scoped functions:
    /// - Are defined in the current scope (not global)
    /// - Can shadow outer functions and builtins
    /// - Follow lexical scoping rules
    fn hoist_scoped_function(&mut self, func: &FunctionDecl) {
        // Note: Nested functions CAN shadow builtins (unlike top-level functions)
        // This is allowed because they follow lexical scoping rules

        // Enter type parameter scope to resolve generic types
        self.enter_type_param_scope();
        for type_param in &func.type_params {
            self.register_type_parameter(type_param);
        }

        let param_types: Vec<Type> = func
            .params
            .iter()
            .map(|p| self.resolve_type_ref(&p.type_ref))
            .collect();

        let return_type = func
            .return_type
            .as_ref()
            .map_or(Type::Unknown, |t| self.resolve_type_ref(t));

        // Exit type parameter scope
        self.exit_type_param_scope();

        let type_params = func
            .type_params
            .iter()
            .map(|param| TypeParamDef {
                name: param.name.clone(),
                trait_bounds: param
                    .trait_bounds
                    .iter()
                    .map(|tb| tb.trait_name.clone())
                    .collect(),
            })
            .collect();

        let symbol = Symbol {
            name: func.name.name.clone(),
            ty: Type::Function {
                type_params,
                params: param_types,
                return_type: Box::new(return_type),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: func.name.span,
            exported: false,
            visibility: func.visibility,
        };

        // Use define_scoped_function to add to current scope (not global functions)
        if let Err(err) = self.symbol_table.define_scoped_function(symbol) {
            let (msg, existing) = *err;
            let mut diag = error_codes::DUPLICATE_DECLARATION
                .emit(func.name.span)
                .arg("detail", &msg)
                .build()
                .with_label("redeclaration")
                .with_help(format!(
                    "rename or remove one of the '{}' declarations",
                    func.name.name
                ));

            // Add related location if we have the existing symbol
            if let Some(existing_symbol) = existing {
                diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                    file: "<input>".to_string(),
                    line: 1,
                    column: existing_symbol.span.start + 1,
                    length: existing_symbol
                        .span
                        .end
                        .saturating_sub(existing_symbol.span.start),
                    message: format!("'{}' first defined here", existing_symbol.name),

                    snippet: String::new(),
                    label: String::new(),
                    is_occurrence: false,
                });
            }

            self.diagnostics.push(diag);
        }
    }

    /// Bind a top-level item
    fn bind_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.bind_function(func),
            Item::Statement(stmt) => self.bind_statement(stmt),
            Item::Import(_) => {
                // Import binding handled in BLOCKER 04-C (cross-module binding)
                // For now, just skip - imports are syntactically valid but not yet functional
            }
            Item::Export(export_decl) => {
                // Export wraps an item - bind the inner item
                match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => self.bind_function(func),
                    crate::ast::ExportItem::Variable(var) => {
                        // Bind variable by treating it as a statement
                        self.bind_statement(&crate::ast::Stmt::VarDecl(var.clone()));
                    }
                    crate::ast::ExportItem::TypeAlias(_) => {
                        // Type aliases are handled during collection
                    }
                    crate::ast::ExportItem::Const(_) => {
                        // Consts are handled during collection
                    }
                    crate::ast::ExportItem::Struct(_) | crate::ast::ExportItem::Enum(_) => {
                        // Struct/enum type declarations are type-system only
                    }
                    crate::ast::ExportItem::ReExport { .. } => {
                        // Re-exports are resolved by the module loader; no binding needed here
                    }
                }
            }
            Item::Extern(_) => {
                // Extern binding handled in phase-10b (FFI infrastructure)
                // For now, just skip - full implementation pending
            }
            Item::TypeAlias(_) => {
                // Type aliases are handled during collection
            }
            Item::Const(_) => {
                // Consts are handled during collection
            }
            Item::Trait(_) | Item::Impl(_) => {
                // Trait/impl binding handled in Block 3 (trait system)
            }
            Item::Struct(_) | Item::Enum(_) => {
                // Struct/enum type declarations are handled by the type system
                // The binder focuses on value bindings, not type definitions
            }
        }
    }

    /// Bind a top-level item with module registry support (BLOCKER 04-C)
    fn bind_item_with_modules(
        &mut self,
        item: &Item,
        _module_path: &Path,
        _registry: &ModuleRegistry,
    ) {
        match item {
            Item::Function(func) => self.bind_function(func),
            Item::Statement(stmt) => self.bind_statement(stmt),
            Item::Import(_) => {
                // Imports already bound in the pre-pass for module-aware binding
            }
            Item::Export(export_decl) => {
                // Export wraps an item - bind the inner item
                match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => self.bind_function(func),
                    crate::ast::ExportItem::Variable(var) => {
                        // Bind variable by treating it as a statement
                        self.bind_statement(&crate::ast::Stmt::VarDecl(var.clone()));
                    }
                    crate::ast::ExportItem::TypeAlias(_) => {
                        // Type aliases are handled during collection
                    }
                    crate::ast::ExportItem::Const(_) => {
                        // Consts are handled during collection
                    }
                    crate::ast::ExportItem::Struct(_) | crate::ast::ExportItem::Enum(_) => {
                        // Struct/enum type declarations are type-system only
                    }
                    crate::ast::ExportItem::ReExport { .. } => {
                        // Re-exports are resolved by the module loader; no binding needed here
                    }
                }
            }
            Item::Extern(_) => {
                // Extern binding handled in phase-10b (FFI infrastructure)
                // For now, just skip - full implementation pending
            }
            Item::TypeAlias(_) => {
                // Type aliases are handled during collection
            }
            Item::Const(_) => {
                // Consts are handled during collection
            }
            Item::Trait(_) | Item::Impl(_) => {
                // Trait/impl binding handled in Block 3 (trait system)
            }
            Item::Struct(_) | Item::Enum(_) => {
                // Struct/enum type declarations are handled by the type system
            }
        }
    }

    /// Bind an import declaration (BLOCKER 04-C)
    ///
    /// Creates local bindings for imported symbols by looking them up in the source module's
    /// symbol table (from the registry).
    fn bind_import(
        &mut self,
        import_decl: &ImportDecl,
        module_path: &Path,
        registry: &ModuleRegistry,
    ) {
        // Resolve source module path (this will be done by ModuleResolver in practice)
        // For now, we'll need to resolve the source path relative to the importing module
        // This is a simplified version - full path resolution happens in ModuleResolver

        // Convert import source to absolute path
        // Note: In practice, this should use ModuleResolver.resolve(), but for binding
        // we assume the source path is already resolved by the loader
        let source_path = Self::resolve_import_path(&import_decl.source, module_path);

        // Look up source module's symbol table — try both .atlas and .atl extensions
        // since the registry key is the real path (which may use either extension).
        let alt_path = if source_path.extension().and_then(|e| e.to_str()) == Some("atlas") {
            source_path.with_extension("atl")
        } else {
            source_path.with_extension("atlas")
        };
        let source_symbols = match registry
            .get(&source_path)
            .or_else(|| registry.get(&alt_path))
        {
            Some(symbol_table) => symbol_table,
            None => {
                // Source module not in registry - error
                self.diagnostics.push(
                    error_codes::IMPORT_RESOLUTION_FAILED
                        .emit(import_decl.span)
                        .arg("path", &import_decl.source)
                        .arg(
                            "detail",
                            format!("Cannot find module '{}'", import_decl.source),
                        )
                        .build()
                        .with_label("import statement")
                        .with_help(
                            "ensure the module exists and has been loaded before importing from it",
                        ),
                );
                return;
            }
        };

        // Get exported symbols from source module
        let exports = source_symbols.get_exports();
        let type_alias_exports = source_symbols.get_type_alias_exports();
        let struct_exports = source_symbols.get_struct_exports().clone();
        let enum_exports = source_symbols.get_enum_exports().clone();
        // H-406: impl method exports for cross-module inherent method visibility
        let impl_method_exports = source_symbols.get_impl_method_exports().clone();
        // H-313: Get all symbols for visibility checking (emit better error for private access)
        let all_symbols = source_symbols.get_all_top_level_symbols();

        // Process each import specifier
        for specifier in &import_decl.specifiers {
            match specifier {
                ImportSpecifier::Named { name, span } => {
                    // Named import: `import { foo } from "./module"`
                    // Look up the symbol in source module's exports
                    match exports.get(&name.name) {
                        Some(exported_symbol) => {
                            // Create a local binding for the imported symbol
                            let imported_symbol = Symbol {
                                name: name.name.clone(),
                                ty: exported_symbol.ty.clone(),
                                mutable: false, // Imported symbols are immutable
                                kind: exported_symbol.kind.clone(),
                                span: *span,
                                exported: false, // Imports are not automatically re-exported
                                visibility: exported_symbol.visibility,
                            };

                            if let Err(err) = self.symbol_table.define(imported_symbol) {
                                let (msg, _) = *err;
                                self.diagnostics.push(
                                    error_codes::DUPLICATE_DECLARATION.emit(*span).arg("detail", &msg).build()
                                        .with_label("imported symbol")
                                        .with_help("rename the import or remove the conflicting local declaration"),
                                );
                            }
                        }
                        None => {
                            // Check for exported type alias
                            if let Some(type_alias) = type_alias_exports.get(&name.name) {
                                if let Err(err) =
                                    self.symbol_table.define_type_alias(type_alias.clone())
                                {
                                    let (msg, _) = *err;
                                    self.diagnostics.push(
                                        error_codes::DUPLICATE_DECLARATION.emit(*span).arg("detail", &msg).build()
                                            .with_label("imported type alias")
                                            .with_help(
                                                "rename the import or remove the conflicting alias declaration",
                                            ),
                                    );
                                }
                            } else if let Some(struct_decl) = struct_exports.get(&name.name) {
                                // Exported struct type — inject into this module's symbol table
                                self.symbol_table.add_struct_export(struct_decl.clone());
                                // H-406: also import impl methods for this struct
                                let struct_name = &name.name;
                                for ((sn, mn), method) in &impl_method_exports {
                                    if sn == struct_name {
                                        self.symbol_table.add_impl_method_export(
                                            sn.clone(),
                                            mn.clone(),
                                            method.clone(),
                                        );
                                    }
                                }
                            } else if let Some(enum_decl) = enum_exports.get(&name.name) {
                                // Exported enum type — inject into this module's symbol table
                                self.symbol_table.add_enum_export(enum_decl.clone());
                                // H-295: Collect variant names for bare variant constructor support
                                for variant in &enum_decl.variants {
                                    self.enum_variants.insert(
                                        variant.name().name.clone(),
                                        enum_decl.name.name.clone(),
                                    );
                                }
                            } else {
                                // H-313: Check if symbol exists but is private (not exported)
                                if let Some(private_symbol) = all_symbols.get(&name.name) {
                                    // Symbol exists but is private — emit AT3059
                                    let kind = match private_symbol.kind {
                                        SymbolKind::Function => "function",
                                        SymbolKind::Variable => "variable",
                                        SymbolKind::Parameter => "parameter",
                                        SymbolKind::Builtin => "builtin",
                                        SymbolKind::Const => "constant",
                                    };
                                    self.diagnostics.push(
                                        error_codes::PRIVATE_ACCESS_VIOLATION
                                            .emit(*span)
                                            .arg("kind", kind)
                                            .arg("name", &name.name)
                                            .build()
                                            .with_label("cannot import private symbol")
                                            .with_help(format!(
                                                "add `pub` or `export` to '{}' in {} to make it importable",
                                                name.name, import_decl.source
                                            )),
                                    );
                                } else {
                                    // Symbol doesn't exist at all
                                    self.diagnostics.push(
                                        error_codes::MODULE_NOT_EXPORTED
                                            .emit(*span)
                                            .arg("name", &name.name)
                                            .arg("module", &import_decl.source)
                                            .build()
                                            .with_label("imported name")
                                            .with_help(
                                                "check the module's exports or import a different symbol",
                                            ),
                                    );
                                }
                            }
                        }
                    }
                }
                ImportSpecifier::Namespace { alias, span } => {
                    // Namespace import: `import * as ns from "./module"`
                    let mut members: Vec<StructuralMemberType> = exports
                        .iter()
                        .map(|(name, symbol)| StructuralMemberType {
                            name: name.clone(),
                            ty: symbol.ty.clone(),
                        })
                        .collect();
                    members.sort_by(|a, b| a.name.cmp(&b.name));

                    let imported_symbol = Symbol {
                        name: alias.name.clone(),
                        ty: Type::Structural { members },
                        mutable: false,
                        kind: SymbolKind::Variable,
                        span: *span,
                        exported: false,
                        visibility: Visibility::Private,
                    };

                    if let Err(err) = self.symbol_table.define(imported_symbol) {
                        let (msg, _) = *err;
                        self.diagnostics.push(
                            error_codes::DUPLICATE_DECLARATION
                                .emit(*span)
                                .arg("detail", &msg)
                                .build()
                                .with_label("namespace import")
                                .with_help(
                                    "rename the import or remove the conflicting local declaration",
                                ),
                        );
                    }
                }
            }
        }
    }

    /// Resolve an import path relative to the importing module.
    /// Public for use by the compiler (H-296: register imported enum variants).
    pub fn resolve_import_path(source: &str, module_path: &Path) -> PathBuf {
        let base_path = if source.starts_with("./") || source.starts_with("../") {
            let base = module_path.parent().unwrap_or(Path::new("."));
            // Strip the leading "./" from relative imports before joining to avoid
            // producing paths like "/src/./types" with a redundant component.
            let rel = source.strip_prefix("./").unwrap_or(source);
            base.join(rel)
        } else if source.starts_with('/') {
            let stripped = PathBuf::from(source.trim_start_matches('/'));
            let root = module_path.ancestors().last().unwrap_or(Path::new("/"));
            root.join(stripped)
        } else {
            PathBuf::from(source)
        };

        // If source already has an extension, use it directly
        if Path::new(source)
            .extension()
            .and_then(|e| e.to_str())
            .is_some()
        {
            // Canonicalize to resolve `..` components, matching the registry
            return base_path.canonicalize().unwrap_or(base_path);
        }

        // No extension provided — try both .atlas and .atl, canonicalizing whichever exists
        let atlas_path = base_path.with_extension("atlas");
        if let Ok(canonical) = atlas_path.canonicalize() {
            return canonical;
        }

        let atl_path = base_path.with_extension("atl");
        if let Ok(canonical) = atl_path.canonicalize() {
            return canonical;
        }

        // Neither exists — return .atlas path (will fail lookup with clear error)
        atlas_path
    }

    /// Pre-pass: collect all struct declarations from the program so that
    /// `resolve_type_ref` can resolve named struct types in function signatures
    /// before Phase 2 processes them. Handles both `struct Foo {}` and `export struct Foo {}`.
    fn collect_struct_decl_prepass(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Struct(decl) => {
                    self.struct_decls
                        .insert(decl.name.name.clone(), decl.clone());
                }
                Item::Enum(decl) => {
                    self.enum_names.insert(decl.name.name.clone());
                    // H-295: Collect variant names for bare variant constructor support
                    for variant in &decl.variants {
                        self.enum_variants
                            .insert(variant.name().name.clone(), decl.name.name.clone());
                    }
                }
                Item::Export(export_decl) => match &export_decl.item {
                    ExportItem::Struct(decl) => {
                        self.struct_decls
                            .insert(decl.name.name.clone(), decl.clone());
                    }
                    ExportItem::Enum(decl) => {
                        self.enum_names.insert(decl.name.name.clone());
                        // H-295: Collect variant names for bare variant constructor support
                        for variant in &decl.variants {
                            self.enum_variants
                                .insert(variant.name().name.clone(), decl.name.name.clone());
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        // Imported enum/struct types (from bind_import pre-pass) are added to
        // struct_exports / enum_exports on the symbol table — resolve_type_ref checks those too.
    }

    fn collect_type_aliases(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::TypeAlias(alias) => {
                    self.define_type_alias(alias);
                }
                Item::Export(export_decl) => {
                    if let ExportItem::TypeAlias(alias) = &export_decl.item {
                        self.define_type_alias(alias);
                    }
                }
                _ => {}
            }
        }
    }

    fn define_type_alias(&mut self, alias: &TypeAliasDecl) {
        // Validate type parameter uniqueness
        let mut seen = HashSet::new();
        for param in &alias.type_params {
            if !seen.insert(param.name.clone()) {
                self.diagnostics.push(
                    error_codes::DUPLICATE_DECLARATION
                        .emit(param.span)
                        .arg(
                            "detail",
                            format!("Duplicate type parameter '{}'", param.name),
                        )
                        .build()
                        .with_label("duplicate type parameter")
                        .with_help("remove or rename the duplicate type parameter"),
                );
                return;
            }
        }

        if let Err(err) = self.symbol_table.define_type_alias(alias.clone()) {
            let (msg, existing) = *err;
            let mut diag = error_codes::DUPLICATE_DECLARATION
                .emit(alias.name.span)
                .arg("detail", &msg)
                .build()
                .with_label("duplicate type alias")
                .with_help(format!(
                    "rename or remove one of the '{}' aliases",
                    alias.name.name
                ));

            if let Some(existing_alias) = existing {
                diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                    file: "<input>".to_string(),
                    line: 1,
                    column: existing_alias.name.span.start + 1,
                    length: existing_alias
                        .name
                        .span
                        .end
                        .saturating_sub(existing_alias.name.span.start),
                    message: format!("'{}' first defined here", existing_alias.name.name),

                    snippet: String::new(),
                    label: String::new(),
                    is_occurrence: false,
                });
            }

            self.diagnostics.push(diag);
        }
    }

    fn collect_consts(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Const(decl) => {
                    self.define_const(decl);
                }
                Item::Export(export_decl) => {
                    if let ExportItem::Const(decl) = &export_decl.item {
                        self.define_const(decl);
                    }
                }
                _ => {}
            }
        }
    }

    fn define_const(&mut self, decl: &ConstDecl) {
        if let Err(err) = self.symbol_table.define_const(decl.clone()) {
            let (msg, existing) = *err;
            let mut diag = error_codes::DUPLICATE_DECLARATION
                .emit(decl.name.span)
                .arg("detail", &msg)
                .build()
                .with_label("duplicate constant")
                .with_help(format!(
                    "rename or remove one of the '{}' constants",
                    decl.name.name
                ));

            if let Some(existing_const) = existing {
                diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                    file: "<input>".to_string(),
                    line: 1,
                    column: existing_const.name.span.start + 1,
                    length: existing_const
                        .name
                        .span
                        .end
                        .saturating_sub(existing_const.name.span.start),
                    message: format!("'{}' first defined here", existing_const.name.name),
                    snippet: String::new(),
                    label: String::new(),
                    is_occurrence: false,
                });
            }

            self.diagnostics.push(diag);
        }
    }

    /// Bind a function declaration
    fn bind_function(&mut self, func: &FunctionDecl) {
        // Enter function scope
        self.symbol_table.enter_scope();

        // Note: Type parameter scope was already handled in hoist_function
        // when we resolved the function signature types

        // Validate default parameters (B39-P05):
        // 1. Required params must come before default params
        // 2. Ownership params (own/share) cannot have defaults
        let mut seen_default = false;
        for param in &func.params {
            if let Some(ref default) = param.default_value {
                seen_default = true;
                // Check ownership restriction: own/share cannot have defaults
                if let Some(ref ownership) = param.ownership {
                    let has_restricted_ownership = matches!(
                        ownership,
                        OwnershipAnnotation::Own | OwnershipAnnotation::Share
                    );
                    // Only error if explicitly annotated (not implicit borrow)
                    if has_restricted_ownership && param.ownership_explicit {
                        let ownership_str = match ownership {
                            OwnershipAnnotation::Own => "own",
                            OwnershipAnnotation::Share => "share",
                            OwnershipAnnotation::Borrow => "borrow",
                        };
                        self.diagnostics.push(
                            error_codes::DEFAULT_ON_OWNERSHIP_PARAM
                                .emit(default.span())
                                .arg("name", &param.name.name)
                                .arg("ownership", ownership_str)
                                .build()
                                .with_label("default value not allowed here"),
                        );
                    }
                }
            } else if seen_default {
                // Required param after a default param
                self.diagnostics.push(
                    error_codes::REQUIRED_PARAM_AFTER_DEFAULT
                        .emit(param.name.span)
                        .arg("name", &param.name.name)
                        .build()
                        .with_label("required parameter")
                        .with_help("move this parameter before parameters with default values"),
                );
            }
        }

        // Bind parameters
        for param in &func.params {
            let ty = self.resolve_type_ref(&param.type_ref);
            let symbol = Symbol {
                name: param.name.name.clone(),
                ty,
                mutable: false,
                kind: SymbolKind::Parameter,
                span: param.name.span,
                exported: false,
                visibility: Visibility::Private,
            };

            if let Err(err) = self.symbol_table.define(symbol) {
                let (msg, existing) = *err;
                let mut diag = error_codes::DUPLICATE_DECLARATION
                    .emit(param.name.span)
                    .arg("detail", &msg)
                    .build()
                    .with_label("parameter redeclaration")
                    .with_help(format!(
                        "rename this parameter to avoid conflict with '{}'",
                        param.name.name
                    ));

                // Add related location if we have the existing symbol
                if let Some(existing_symbol) = existing {
                    diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                        file: "<input>".to_string(),
                        line: 1,
                        column: existing_symbol.span.start + 1,
                        length: existing_symbol
                            .span
                            .end
                            .saturating_sub(existing_symbol.span.start),
                        message: format!("'{}' first defined here", existing_symbol.name),

                        snippet: String::new(),
                        label: String::new(),
                        is_occurrence: false,
                    });
                }

                self.diagnostics.push(diag);
            }
        }

        // Bind function body
        self.bind_block(&func.body);

        // Exit function scope
        self.symbol_table.exit_scope();
    }

    /// Bind a block
    ///
    /// Uses two-pass binding to support forward references to nested functions:
    /// 1. First pass: Hoist all nested function declarations
    /// 2. Second pass: Bind all statements (including function bodies)
    fn bind_block(&mut self, block: &Block) {
        // Blocks create their own scope
        self.symbol_table.enter_scope();

        // Phase 1: Hoist all nested function declarations
        // This allows functions to reference other functions declared later in the same block
        for stmt in &block.statements {
            if let Stmt::FunctionDecl(func) = stmt {
                self.hoist_scoped_function(func);
            }
        }

        // Phase 2: Bind all statements (including function bodies)
        for stmt in &block.statements {
            self.bind_statement(stmt);
        }

        // Phase 3: Bind tail expression (implicit return) if present
        if let Some(tail) = &block.tail_expr {
            self.bind_expr(tail);
        }

        self.symbol_table.exit_scope();
    }

    /// Bind a statement
    fn bind_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(var) => {
                // Check for global shadowing of prelude builtins
                if self.symbol_table.is_global_scope()
                    && self.symbol_table.is_prelude_builtin(&var.name.name)
                {
                    let diag = error_codes::SHADOWING_PRELUDE.emit(var.name.span).arg("detail", format!(
                            "Cannot shadow prelude builtin '{}' in global scope",
                            var.name.name
                        )).build()
                    .with_label("shadows prelude builtin")
                    .with_help("Prelude builtins cannot be redefined at the top level. Use a different name or shadow in a nested scope.".to_string());

                    self.diagnostics.push(diag);
                    return;
                }

                // First bind the initializer (can't reference the variable being declared)
                self.bind_expr(&var.init);

                // Then define the variable
                let ty = if let Some(type_ref) = &var.type_ref {
                    self.resolve_type_ref(type_ref)
                } else {
                    Type::Unknown // Will be inferred by typechecker
                };

                let symbol = Symbol {
                    name: var.name.name.clone(),
                    ty,
                    mutable: var.mutable,
                    kind: SymbolKind::Variable,
                    span: var.name.span,
                    exported: false,
                    visibility: Visibility::Private,
                };

                if let Err(err) = self.symbol_table.define(symbol) {
                    let (msg, existing) = *err;
                    let mut diag = error_codes::DUPLICATE_DECLARATION
                        .emit(var.name.span)
                        .arg("detail", &msg)
                        .build()
                        .with_label("variable redeclaration")
                        .with_help(format!(
                            "rename this variable or remove the previous declaration of '{}'",
                            var.name.name
                        ));

                    // Add related location if we have the existing symbol
                    if let Some(existing_symbol) = existing {
                        diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                            file: "<input>".to_string(),
                            line: 1,
                            column: existing_symbol.span.start + 1,
                            length: existing_symbol
                                .span
                                .end
                                .saturating_sub(existing_symbol.span.start),
                            message: format!("'{}' first defined here", existing_symbol.name),

                            snippet: String::new(),
                            label: String::new(),
                            is_occurrence: false,
                        });
                    }

                    self.diagnostics.push(diag);
                }
            }
            Stmt::LetDestructure(d) => {
                // Bind the initializer first
                self.bind_expr(&d.init);
                // Define each destructured name in scope
                for name in &d.names {
                    let symbol = Symbol {
                        name: name.name.clone(),
                        ty: Type::Unknown,
                        mutable: d.mutable,
                        span: name.span,
                        kind: SymbolKind::Variable,
                        exported: false,
                        visibility: Visibility::Private,
                    };
                    let _ = self.symbol_table.define(symbol);
                }
            }
            Stmt::Assign(assign) => {
                // Bind assignment target and value
                self.bind_assign_target(&assign.target);
                self.bind_expr(&assign.value);
            }
            Stmt::CompoundAssign(compound) => {
                // Bind compound assignment target and value
                self.bind_assign_target(&compound.target);
                self.bind_expr(&compound.value);
            }
            Stmt::If(if_stmt) => {
                self.bind_expr(&if_stmt.cond);
                self.bind_block(&if_stmt.then_block);
                if let Some(else_block) = &if_stmt.else_block {
                    self.bind_block(else_block);
                }
            }
            Stmt::While(while_stmt) => {
                self.bind_expr(&while_stmt.cond);
                self.bind_block(&while_stmt.body);
            }
            Stmt::ForIn(for_in_stmt) => {
                // Bind iterable expression in current scope
                self.bind_expr(&for_in_stmt.iterable);

                // Create new scope for loop body (includes loop variable)
                self.symbol_table.enter_scope();

                // Add loop variable to scope (type will be inferred by typechecker)
                let symbol = Symbol {
                    name: for_in_stmt.variable.name.clone(),
                    ty: Type::Unknown, // Will be inferred from array element type
                    mutable: false,    // Loop variables are immutable
                    kind: SymbolKind::Variable,
                    span: for_in_stmt.variable.span,
                    exported: false,
                    visibility: Visibility::Private,
                };

                if let Err(err) = self.symbol_table.define(symbol) {
                    let (msg, existing) = *err;
                    let mut diag = error_codes::DUPLICATE_DECLARATION
                        .emit(for_in_stmt.variable.span)
                        .arg("detail", &msg)
                        .build()
                        .with_label("variable redeclaration");

                    if let Some(existing_symbol) = existing {
                        diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                            file: "<input>".to_string(),
                            line: 1,
                            column: existing_symbol.span.start + 1,
                            length: existing_symbol
                                .span
                                .end
                                .saturating_sub(existing_symbol.span.start),
                            message: format!("'{}' first defined here", existing_symbol.name),

                            snippet: String::new(),
                            label: String::new(),
                            is_occurrence: false,
                        });
                    }

                    self.diagnostics.push(diag);
                }

                // Bind body statements
                self.bind_block(&for_in_stmt.body);

                // Exit loop scope
                self.symbol_table.exit_scope();
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.value {
                    self.bind_expr(expr);
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) => {
                // No binding needed
            }
            Stmt::Expr(expr_stmt) => {
                self.bind_expr(&expr_stmt.expr);
            }
            Stmt::FunctionDecl(func) => {
                // Function declaration in statement position
                // Note: Hoisting is already done by bind_block's first pass for nested functions
                // We only need to bind the function body here
                self.bind_function(func);
            }
            Stmt::Defer(defer) => {
                // Bind the deferred block - it executes in the same scope
                self.bind_block(&defer.body);
            }
        }
    }

    /// Bind an assignment target
    fn bind_assign_target(&mut self, target: &AssignTarget) {
        match target {
            AssignTarget::Name(id) => {
                // Check if the identifier exists
                if self.symbol_table.lookup(&id.name).is_none() {
                    let suggestion = crate::typechecker::suggestions::suggest_similar_name(
                        &id.name,
                        self.symbol_table.all_names_for_suggestion().into_iter(),
                    );
                    let mut diag = crate::diagnostic::error_codes::UNDEFINED_SYMBOL.emit(id.span).arg("name", &id.name).arg("detail", format!("unknown symbol '{}'", id.name)).build()
                    .with_label("undefined variable")
                    .with_help(format!(
                        "declare `{}` with `let` before assigning to it: `let {} = value;`",
                        id.name, id.name
                    ))
                    .with_note("variables must be declared before assignment — Atlas has no implicit declaration");
                    if let Some(ref sugg) = suggestion {
                        diag = diag.with_note(format!("did you mean `{}`?", sugg));
                    }
                    self.diagnostics.push(diag);
                }
            }
            AssignTarget::Index { target, index, .. } => {
                self.bind_expr(target);
                self.bind_expr(index);
            }
            AssignTarget::Member { target, .. } => {
                self.bind_expr(target);
            }
        }
    }

    /// Bind an expression
    fn bind_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(_, _) => {
                // Literals don't need binding
            }
            Expr::TemplateString { parts, .. } => {
                for part in parts {
                    if let TemplatePart::Expression(expr) = part {
                        self.bind_expr(expr);
                    }
                }
            }
            Expr::Identifier(id) => {
                // H-305: Check symbol table FIRST to allow user variables to shadow deprecated
                // bare global names. A user declaring `let log = ...` should be valid.
                let is_defined = self.symbol_table.lookup(&id.name).is_some()
                    || crate::stdlib::is_array_intrinsic(&id.name)
                    || crate::method_dispatch::is_static_namespace(&id.name)
                    || crate::method_dispatch::is_allowed_bare_global(&id.name)
                    || self.enum_variants.contains_key(&id.name)
                    // B39-P02: struct type names are valid for static method calls (Type.method())
                    || self.struct_decls.contains_key(&id.name);

                if is_defined {
                    // Identifier is valid — nothing to do
                    return;
                }

                // Not defined — check if it's a deprecated bare global for better error message
                if let Some(hint) = crate::method_dispatch::namespace_hint_for_bare_global(&id.name)
                {
                    // This is a deprecated bare global — error with migration hint
                    let diag = crate::diagnostic::error_codes::UNDEFINED_SYMBOL
                        .emit(id.span)
                        .arg("name", &id.name)
                        .arg(
                            "detail",
                            format!("bare global `{}` has been removed", id.name),
                        )
                        .build()
                        .with_label("deprecated bare global")
                        .with_help(format!("use `{}` instead", hint))
                        .with_note("bare globals were removed in favor of namespace syntax");
                    self.diagnostics.push(diag);
                    return;
                }

                // Generic undefined identifier error
                let suggestion = crate::typechecker::suggestions::suggest_similar_name(
                    &id.name,
                    self.symbol_table.all_names_for_suggestion().into_iter(),
                );
                let case_hint = crate::method_dispatch::wrong_case_namespace_hint(&id.name);
                let mut diag = crate::diagnostic::error_codes::UNDEFINED_SYMBOL
                    .emit(id.span)
                    .arg("name", &id.name)
                    .arg("detail", format!("unknown identifier `{}`", id.name))
                    .build()
                    .with_label("undefined identifier");
                if let Some(ref correct) = case_hint {
                    diag = diag.with_help(format!(
                        "use `{}` instead — Atlas namespace names are lowercase (e.g. `{}.method()`)",
                        correct, correct
                    ));
                } else {
                    diag = diag
                        .with_help(format!(
                            "declare `{}` with `let` before using it: `let {} = value;`",
                            id.name, id.name
                        ))
                        .with_help(format!(
                            "if `{}` is defined in another file, import it: `import {{ {} }} from \"./module\"`",
                            id.name, id.name
                        ));
                }
                diag =
                    diag.with_note("identifiers must be declared in the current scope before use");
                if let Some(ref sugg) = suggestion {
                    diag = diag.with_note(format!("did you mean `{}`?", sugg));
                }
                self.diagnostics.push(diag);
            }
            Expr::Binary(binary) => {
                self.bind_expr(&binary.left);
                self.bind_expr(&binary.right);
            }
            Expr::Unary(unary) => {
                self.bind_expr(&unary.expr);
            }
            Expr::Call(call) => {
                self.bind_expr(&call.callee);
                for arg in &call.args {
                    self.bind_expr(arg);
                }
            }
            Expr::Index(index) => {
                self.bind_expr(&index.target);
                let IndexValue::Single(expr) = &index.index;
                self.bind_expr(expr);
            }
            Expr::ArrayLiteral(arr) => {
                for elem in &arr.elements {
                    self.bind_expr(elem);
                }
            }
            Expr::ObjectLiteral(obj) => {
                // Bind all value expressions in the object literal
                for entry in &obj.entries {
                    self.bind_expr(&entry.value);
                }
            }
            Expr::StructExpr(struct_expr) => {
                // Bind all field value expressions in the struct instantiation
                for field in &struct_expr.fields {
                    self.bind_expr(&field.value);
                }
            }
            Expr::Group(group) => {
                self.bind_expr(&group.expr);
            }
            Expr::Range { start, end, .. } => {
                if let Some(start) = start {
                    self.bind_expr(start);
                }
                if let Some(end) = end {
                    self.bind_expr(end);
                }
            }
            Expr::Match(match_expr) => {
                // Bind scrutinee
                self.bind_expr(&match_expr.scrutinee);
                // Bind each arm
                for arm in &match_expr.arms {
                    // Collect pattern variables
                    let pattern_vars = self.collect_pattern_variables(&arm.pattern);

                    // Enter scope and add pattern variables
                    self.symbol_table.enter_scope();
                    for (var_name, var_span) in &pattern_vars {
                        let symbol = Symbol {
                            name: var_name.clone(),
                            ty: Type::Unknown, // Type will be determined during type checking
                            mutable: false,
                            kind: SymbolKind::Variable,
                            span: *var_span,
                            exported: false,
                            visibility: Visibility::Private,
                        };
                        let _ = self.symbol_table.define(symbol);
                    }

                    // Bind guard expression (if present) with pattern variables in scope
                    if let Some(guard) = &arm.guard {
                        self.bind_expr(guard);
                    }

                    // Bind arm body with pattern variables in scope
                    self.bind_expr(&arm.body);

                    // Exit scope
                    self.symbol_table.exit_scope();
                }
            }
            Expr::Member(member) => {
                // Bind target expression
                self.bind_expr(&member.target);
                // Bind arguments if present
                if let Some(args) = &member.args {
                    for arg in args {
                        self.bind_expr(arg);
                    }
                }
            }
            Expr::Try(try_expr) => {
                // Bind the expression being tried
                self.bind_expr(&try_expr.expr);
            }
            Expr::AnonFn { params, body, .. } => {
                // Enter a new scope for the anonymous function body
                self.symbol_table.enter_scope();
                // Define params as symbols (they are definitions, not references)
                for param in params {
                    let ty = self.resolve_type_ref(&param.type_ref);
                    let symbol = Symbol {
                        name: param.name.name.clone(),
                        ty,
                        mutable: false,
                        kind: SymbolKind::Parameter,
                        span: param.name.span,
                        exported: false,
                        visibility: Visibility::Private,
                    };
                    let _ = self.symbol_table.define(symbol);
                }
                // Bind the body with params in scope
                self.bind_expr(body);
                self.symbol_table.exit_scope();
            }
            Expr::Block(block) => {
                self.symbol_table.enter_scope();
                for stmt in &block.statements {
                    self.bind_statement(stmt);
                }
                // Bind tail expression if present
                if let Some(tail) = &block.tail_expr {
                    self.bind_expr(tail);
                }
                self.symbol_table.exit_scope();
            }
            Expr::EnumVariant(ev) => {
                // Bind any arguments in the enum variant
                if let Some(args) = &ev.args {
                    for arg in args {
                        self.bind_expr(arg);
                    }
                }
            }
            Expr::TupleLiteral { elements, .. } => {
                for elem in elements {
                    self.bind_expr(elem);
                }
            }
            Expr::Await { expr, .. } => {
                self.bind_expr(expr);
            }
            Expr::New { args, .. } => {
                for arg in args {
                    self.bind_expr(arg);
                }
            }
        }
    }

    /// Collect all variable bindings from a pattern
    fn collect_pattern_variables(
        &self,
        pattern: &crate::ast::Pattern,
    ) -> Vec<(String, crate::span::Span)> {
        use crate::ast::Pattern;
        let mut vars = Vec::new();

        match pattern {
            Pattern::Literal(_, _) | Pattern::Wildcard(_) => {
                // No variables
            }
            Pattern::Variable(id) => {
                vars.push((id.name.clone(), id.span));
            }
            Pattern::Constructor { args, .. } => {
                // Collect from nested patterns
                for arg in args {
                    vars.extend(self.collect_pattern_variables(arg));
                }
            }
            Pattern::Array { elements, .. } => {
                // Collect from all element patterns
                for elem in elements {
                    vars.extend(self.collect_pattern_variables(elem));
                }
            }
            Pattern::Tuple { elements, .. } => {
                // Collect from all tuple element patterns
                for elem in elements {
                    vars.extend(self.collect_pattern_variables(elem));
                }
            }
            Pattern::Or(alternatives, _) => {
                // Collect variables from first sub-pattern (all must bind same names)
                if let Some(first) = alternatives.first() {
                    vars.extend(self.collect_pattern_variables(first));
                }
            }
            Pattern::EnumVariant { args, .. } => {
                // Collect from nested patterns in enum variant args
                for arg in args {
                    vars.extend(self.collect_pattern_variables(arg));
                }
            }
            Pattern::BareVariant { args, .. } => {
                // Collect from nested patterns in bare variant args
                for arg in args {
                    vars.extend(self.collect_pattern_variables(arg));
                }
            }
            Pattern::Struct { fields, .. } => {
                // Collect variables from struct field patterns
                for field in fields {
                    match &field.pattern {
                        Some(sub) => vars.extend(self.collect_pattern_variables(sub)),
                        // Shorthand binding: field name becomes a variable
                        None => vars.push((field.name.name.clone(), field.name.span)),
                    }
                }
            }
        }

        vars
    }

    /// Get the expected arity for a built-in generic type
    fn get_generic_type_arity(&self, name: &str) -> Option<usize> {
        match name {
            "Option" => Some(1),
            "Result" => Some(2),
            "Array" => Some(1), // Array<T> is sugar for []T
            "Map" => Some(2),
            "Set" => Some(1),
            _ => None, // Unknown generic type
        }
    }

    /// Resolve a type reference to a Type
    fn resolve_type_ref(&mut self, type_ref: &TypeRef) -> Type {
        match type_ref {
            TypeRef::Named(name, span) => match name.as_str() {
                "number" => Type::Number,
                "string" => Type::String,
                "bool" => Type::Bool,
                "void" => Type::Void,
                "null" => Type::Null,
                "any" => Type::any_placeholder(),
                "json" => Type::JsonValue,
                "array" => Type::Array(Box::new(Type::any_placeholder())),
                "Comparable" | "Numeric" => Type::Number,
                "Iterable" => Type::Array(Box::new(Type::any_placeholder())),
                "Equatable" => {
                    Type::union(vec![Type::Number, Type::String, Type::Bool, Type::Null])
                }
                "Serializable" => Type::union(vec![
                    Type::Number,
                    Type::String,
                    Type::Bool,
                    Type::Null,
                    Type::JsonValue,
                ]),
                _ => {
                    // Check if it's a type parameter
                    if let Some(_type_param) = self.lookup_type_parameter(name) {
                        return Type::TypeParameter { name: name.clone() };
                    }

                    // Check if it's a type alias
                    if let Some(alias) = self.symbol_table.get_type_alias(name).cloned() {
                        if alias.type_params.is_empty() {
                            return self.resolve_type_alias(&alias, Vec::new(), *span);
                        }
                        // Defer generic alias arity checks to the type checker (allows inference)
                        return Type::Unknown;
                    }

                    // Check if it's a struct type — look in the pre-pass struct map first,
                    // then fall back to struct_exports (populated by imported structs).
                    let struct_decl = self
                        .struct_decls
                        .get(name)
                        .cloned()
                        .or_else(|| self.symbol_table.get_struct_exports().get(name).cloned());
                    if let Some(decl) = struct_decl {
                        let members = decl
                            .fields
                            .iter()
                            .map(|f| StructuralMemberType {
                                name: f.name.name.clone(),
                                ty: self.resolve_type_ref(&f.type_ref),
                            })
                            .collect();
                        return Type::Structural { members };
                    }

                    // Check if it's an enum type — pre-pass set or imported enum exports.
                    let is_enum = self.enum_names.contains(name)
                        || self.symbol_table.get_enum_exports().contains_key(name);
                    if is_enum {
                        // Enums use the same Generic representation the typechecker uses.
                        return Type::Generic {
                            name: name.clone(),
                            type_args: vec![],
                        };
                    }

                    // Unknown type - will be caught by typechecker
                    Type::Unknown
                }
            },
            TypeRef::Array(elem, _) => Type::Array(Box::new(self.resolve_type_ref(elem))),
            TypeRef::Function {
                params,
                return_type,
                ..
            } => {
                let param_types = params.iter().map(|p| self.resolve_type_ref(p)).collect();
                let ret_type = Box::new(self.resolve_type_ref(return_type));
                Type::Function {
                    type_params: vec![], // Function types don't have type params (only decls do)
                    params: param_types,
                    return_type: ret_type,
                }
            }
            TypeRef::Structural { members, .. } => Type::Structural {
                members: members
                    .iter()
                    .map(|member| StructuralMemberType {
                        name: member.name.clone(),
                        ty: self.resolve_type_ref(&member.type_ref),
                    })
                    .collect(),
            },
            TypeRef::Generic {
                name,
                type_args,
                span,
            } => {
                if let Some(alias) = self.symbol_table.get_type_alias(name).cloned() {
                    if type_args.len() != alias.type_params.len() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "Type alias '{}' expects {} type argument(s), found {}",
                                    name,
                                    alias.type_params.len(),
                                    type_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments")
                            .with_help(format!(
                                "provide exactly {} type argument(s) for '{}'",
                                alias.type_params.len(),
                                name
                            )),
                        );
                        return Type::Unknown;
                    }

                    let resolved_args = type_args
                        .iter()
                        .map(|arg| self.resolve_type_ref(arg))
                        .collect::<Vec<_>>();
                    return self.resolve_type_alias(&alias, resolved_args, *span);
                }

                // BLOCKER 02-B: Validate generic type arity
                let expected_arity = self.get_generic_type_arity(name);

                if let Some(arity) = expected_arity {
                    if type_args.len() != arity {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "Generic type '{}' expects {} type argument(s), found {}",
                                    name,
                                    arity,
                                    type_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments")
                            .with_help(format!(
                                "provide exactly {} type argument(s) for '{}'",
                                arity, name
                            )),
                        );
                        return Type::Unknown;
                    }
                } else {
                    // Unknown generic type
                    self.diagnostics.push(
                        Diagnostic::error(format!("Unknown generic type '{}'", name), *span)
                            .with_label("unknown type")
                            .with_help("valid generic types are: Option, Result, Array"),
                    );
                    return Type::Unknown;
                }

                // Resolve type arguments
                let resolved_args = type_args
                    .iter()
                    .map(|arg| self.resolve_type_ref(arg))
                    .collect();

                Type::Generic {
                    name: name.clone(),
                    type_args: resolved_args,
                }
            }
            TypeRef::Union { members, .. } => {
                let resolved = members.iter().map(|m| self.resolve_type_ref(m)).collect();
                Type::union(resolved)
            }
            TypeRef::Intersection { members, .. } => {
                let resolved = members.iter().map(|m| self.resolve_type_ref(m)).collect();
                Type::intersection(resolved)
            }
            TypeRef::Tuple { elements, .. } => {
                let resolved = elements.iter().map(|e| self.resolve_type_ref(e)).collect();
                Type::Tuple(resolved)
            }
            TypeRef::Future { inner, .. } => {
                let inner_ty = self.resolve_type_ref(inner);
                Type::Generic {
                    name: "Future".to_string(),
                    type_args: vec![inner_ty],
                }
            }
            TypeRef::SelfType(_) => Type::Unknown,
        }
    }

    fn resolve_type_alias(
        &mut self,
        alias: &TypeAliasDecl,
        type_args: Vec<Type>,
        span: Span,
    ) -> Type {
        let alias_name = alias.name.name.clone();
        if self.type_alias_stack.contains(&alias_name) {
            self.diagnostics.push(
                error_codes::TYPE_ERROR
                    .emit(span)
                    .arg(
                        "detail",
                        format!("Circular type alias detected for '{}'", alias_name),
                    )
                    .build()
                    .with_label("circular type alias")
                    .with_help("remove the circular reference in type aliases"),
            );
            return Type::Unknown;
        }

        let substitutions = alias
            .type_params
            .iter()
            .map(|param| param.name.clone())
            .zip(type_args.iter().cloned())
            .collect::<HashMap<_, _>>();

        self.type_alias_stack.push(alias_name.clone());
        let resolved_target =
            self.resolve_type_ref_with_alias_params(&alias.type_ref, &substitutions);
        self.type_alias_stack.pop();

        Type::Alias {
            name: alias_name,
            type_args,
            target: Box::new(resolved_target),
        }
    }

    fn resolve_type_ref_with_alias_params(
        &mut self,
        type_ref: &TypeRef,
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        match type_ref {
            TypeRef::Named(name, _) => {
                if let Some(sub) = substitutions.get(name) {
                    return sub.clone();
                }
                self.resolve_type_ref(type_ref)
            }
            TypeRef::Array(elem, _) => Type::Array(Box::new(
                self.resolve_type_ref_with_alias_params(elem, substitutions),
            )),
            TypeRef::Function {
                params,
                return_type,
                ..
            } => {
                let param_types = params
                    .iter()
                    .map(|p| self.resolve_type_ref_with_alias_params(p, substitutions))
                    .collect();
                let ret_type =
                    Box::new(self.resolve_type_ref_with_alias_params(return_type, substitutions));
                Type::Function {
                    type_params: vec![],
                    params: param_types,
                    return_type: ret_type,
                }
            }
            TypeRef::Structural { members, .. } => Type::Structural {
                members: members
                    .iter()
                    .map(|member| StructuralMemberType {
                        name: member.name.clone(),
                        ty: self
                            .resolve_type_ref_with_alias_params(&member.type_ref, substitutions),
                    })
                    .collect(),
            },
            TypeRef::Union { members, .. } => {
                let resolved = members
                    .iter()
                    .map(|m| self.resolve_type_ref_with_alias_params(m, substitutions))
                    .collect();
                Type::union(resolved)
            }
            TypeRef::Intersection { members, .. } => {
                let resolved = members
                    .iter()
                    .map(|m| self.resolve_type_ref_with_alias_params(m, substitutions))
                    .collect();
                Type::intersection(resolved)
            }
            TypeRef::Generic {
                name,
                type_args,
                span,
            } => {
                if let Some(alias) = self.symbol_table.get_type_alias(name).cloned() {
                    if type_args.len() != alias.type_params.len() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "Type alias '{}' expects {} type argument(s), found {}",
                                    name,
                                    alias.type_params.len(),
                                    type_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments")
                            .with_help(format!(
                                "provide exactly {} type argument(s) for '{}'",
                                alias.type_params.len(),
                                name
                            )),
                        );
                        return Type::Unknown;
                    }

                    let resolved_args = type_args
                        .iter()
                        .map(|arg| self.resolve_type_ref_with_alias_params(arg, substitutions))
                        .collect::<Vec<_>>();
                    return self.resolve_type_alias(&alias, resolved_args, *span);
                }

                // Fall back to normal resolution (includes built-in generic validation)
                self.resolve_type_ref(type_ref)
            }
            TypeRef::Tuple { elements, .. } => {
                let resolved = elements
                    .iter()
                    .map(|e| self.resolve_type_ref_with_alias_params(e, substitutions))
                    .collect();
                Type::Tuple(resolved)
            }
            TypeRef::Future { inner, .. } => {
                let inner_ty = self.resolve_type_ref_with_alias_params(inner, substitutions);
                Type::Generic {
                    name: "Future".to_string(),
                    type_args: vec![inner_ty],
                }
            }
            TypeRef::SelfType(_) => Type::Unknown,
        }
    }

    // === Type Parameter Scope Management ===

    /// Enter a new type parameter scope
    fn enter_type_param_scope(&mut self) {
        self.type_param_scopes.push(HashMap::new());
    }

    /// Exit the current type parameter scope
    fn exit_type_param_scope(&mut self) {
        self.type_param_scopes.pop();
    }

    /// Register a type parameter in the current scope
    fn register_type_parameter(&mut self, type_param: &TypeParam) {
        if let Some(current_scope) = self.type_param_scopes.last_mut() {
            // Check for duplicate type parameter in this scope
            if current_scope.contains_key(&type_param.name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        format!("Duplicate type parameter '{}'", type_param.name),
                        type_param.span,
                    )
                    .with_label("duplicate type parameter")
                    .with_help(format!(
                        "remove the duplicate '{}' or rename it to a unique name",
                        type_param.name
                    )),
                );
                return;
            }

            current_scope.insert(type_param.name.clone(), type_param.clone());
        }
    }

    /// Look up a type parameter in the scope stack
    fn lookup_type_parameter(&self, name: &str) -> Option<&TypeParam> {
        // Search from innermost to outermost scope
        for scope in self.type_param_scopes.iter().rev() {
            if let Some(type_param) = scope.get(name) {
                return Some(type_param);
            }
        }
        None
    }
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}
