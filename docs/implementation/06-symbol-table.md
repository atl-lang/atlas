# Symbol Table & Binding

## Scope Stack Architecture

```rust
// symbol.rs
pub struct SymbolTable {
    scopes: Vec<Scope>,
    functions: HashMap<String, Symbol>,  // Top-level functions (hoisted)
}

type Scope = HashMap<String, Symbol>;

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            scopes: vec![HashMap::new()],  // Global scope
            functions: HashMap::new(),
        };

        // Add prelude builtins
        table.define_builtin("print");
        table.define_builtin("len");
        table.define_builtin("str");

        table
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, symbol: Symbol) -> Result<(), String> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name) {
            return Err(format!("Symbol '{}' already defined in this scope", name));
        }
        scope.insert(name, symbol);
        Ok(())
    }

    pub fn define_function(&mut self, name: String, symbol: Symbol) -> Result<(), String> {
        if self.functions.contains_key(&name) {
            return Err(format!("Function '{}' already defined", name));
        }
        self.functions.insert(name, symbol);
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        // Check local scopes first (innermost to outermost)
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }

        // Check top-level functions
        self.functions.get(name)
    }

    fn define_builtin(&mut self, name: &str) {
        // Simplified - actual types defined in typechecker
        self.functions.insert(name.to_string(), Symbol {
            name: name.to_string(),
            ty: Type::Unknown,  // Will be set properly later
            mutable: false,
            kind: SymbolKind::Builtin,
            span: DUMMY_SPAN,
        });
    }
}
```

## Binder (Two-Pass Algorithm)

```rust
pub struct Binder {
    symbol_table: SymbolTable,
    diagnostics: Vec<Diagnostic>,
}

impl Binder {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn bind(&mut self, program: &Program) -> (SymbolTable, Vec<Diagnostic>) {
        // Phase 1: Collect all top-level function declarations (hoisting)
        for item in &program.items {
            if let Item::Function(func) = item {
                let param_types = func.params.iter()
                    .map(|p| self.resolve_type_ref(&p.type_ref))
                    .collect();
                let return_type = self.resolve_type_ref(&func.return_type);

                let symbol = Symbol {
                    name: func.name.name.clone(),
                    ty: Type::Function {
                        params: param_types,
                        return_type: Box::new(return_type),
                    },
                    mutable: false,
                    kind: SymbolKind::Function,
                    span: func.span,
                };

                if let Err(e) = self.symbol_table.define_function(func.name.name.clone(), symbol) {
                    self.diagnostics.push(
                        Diagnostic::error("AT0003", &e, func.name.span)
                    );
                }
            }
        }

        // Phase 2: Bind all items
        for item in &program.items {
            self.bind_item(item);
        }

        (
            std::mem::take(&mut self.symbol_table),
            std::mem::take(&mut self.diagnostics)
        )
    }

    fn bind_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.bind_function(func),
            Item::Statement(stmt) => self.bind_statement(stmt),
        }
    }

    fn bind_function(&mut self, func: &FunctionDecl) {
        self.symbol_table.enter_scope();

        // Bind parameters
        for param in &func.params {
            let ty = self.resolve_type_ref(&param.type_ref);
            let symbol = Symbol {
                name: param.name.name.clone(),
                ty,
                mutable: false,
                kind: SymbolKind::Parameter,
                span: param.span,
            };

            if let Err(e) = self.symbol_table.define(param.name.name.clone(), symbol) {
                self.diagnostics.push(
                    Diagnostic::error("AT0003", &e, param.name.span)
                );
            }
        }

        // Bind body
        self.bind_block(&func.body);

        self.symbol_table.exit_scope();
    }

    fn bind_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(var) => {
                let ty = if let Some(type_ref) = &var.type_ref {
                    self.resolve_type_ref(type_ref)
                } else {
                    Type::Unknown  // Will be inferred by typechecker
                };

                let symbol = Symbol {
                    name: var.name.name.clone(),
                    ty,
                    mutable: var.mutable,
                    kind: SymbolKind::Variable,
                    span: var.span,
                };

                if let Err(e) = self.symbol_table.define(var.name.name.clone(), symbol) {
                    self.diagnostics.push(
                        Diagnostic::error("AT0003", &e, var.name.span)
                    );
                }

                self.bind_expr(&var.init);
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
            // ... other statements
            _ => {}
        }
    }

    fn bind_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(id) => {
                if self.symbol_table.resolve(&id.name).is_none() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "AT0002",
                            &format!("Unknown symbol '{}'", id.name),
                            id.span
                        )
                    );
                }
            }
            Expr::Binary(binary) => {
                self.bind_expr(&binary.left);
                self.bind_expr(&binary.right);
            }
            Expr::Call(call) => {
                self.bind_expr(&call.callee);
                for arg in &call.args {
                    self.bind_expr(arg);
                }
            }
            // ... other expressions
            _ => {}
        }
    }

    fn resolve_type_ref(&self, type_ref: &TypeRef) -> Type {
        match type_ref {
            TypeRef::Named(name, _) => match name.as_str() {
                "number" => Type::Number,
                "string" => Type::String,
                "bool" => Type::Bool,
                "void" => Type::Void,
                "null" => Type::Null,
                _ => Type::Unknown,
            },
            TypeRef::Array(elem, _) => Type::Array(Box::new(self.resolve_type_ref(elem))),
        }
    }
}
```

## Key Design Decisions

- **Two-pass binding:** Hoist functions first, then bind everything
- **Scope stack:** `Vec<HashMap>` for lexical scoping
- **Separate function table:** Top-level functions live separately
- **Error recovery:** Continue binding after errors, collect all diagnostics
