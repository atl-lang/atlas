---
paths:
  - "crates/atlas-runtime/src/typechecker/**"
  - "crates/atlas-runtime/src/types.rs"
---

# Atlas TypeChecker Quick-Ref

**Verified against:** `typechecker/mod.rs`, `types.rs`
**Update trigger:** Any phase adding Type variants or TypeChecker fields — update at GATE 7.

---

## TypeChecker Struct Fields (typechecker/mod.rs)

```rust
pub struct TypeChecker<'a> {
    symbol_table: &'a mut SymbolTable,
    pub(super) diagnostics: Vec<Diagnostic>,
    last_expr_type: Option<Type>,
    current_function_return_type: Option<Type>,
    current_function_info: Option<(String, Span)>,
    in_loop: bool,
    pub(super) declared_symbols: HashMap<String, (Span, SymbolKind)>,
    pub(super) used_symbols: HashSet<String>,
    pub(super) method_table: methods::MethodTable,
    pub(super) type_guards: type_guards::TypeGuardRegistry,
    type_aliases: HashMap<String, TypeAliasDecl>,
    alias_cache: HashMap<AliasKey, Type>,
    alias_resolution_stack: Vec<String>,
    pub fn_ownership_registry: HashMap<String, FnOwnershipEntry>,
    pub(super) current_fn_param_ownerships: HashMap<String, Option<OwnershipAnnotation>>,
    pub trait_registry: TraitRegistry,
    pub impl_registry: ImplRegistry,
}
// FnOwnershipEntry = (Vec<Option<OwnershipAnnotation>>, Option<OwnershipAnnotation>)
```

## Type Enum (types.rs)

```rust
pub enum Type {
    Never, Number, String, Bool, Null, Void,
    Array(Box<Type>),
    Function { type_params: Vec<TypeParamDef>, params: Vec<Type>, return_type: Box<Type> },
    JsonValue,
    Generic { name: String, type_args: Vec<Type> },
    Alias { name: String, type_args: Vec<Type>, target: Box<Type> },
    TypeParameter { name: String },   // struct variant — NOT TypeParameter(name)
    Unknown,
    Extern(ExternType),
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    Structural { members: Vec<StructuralMemberType> },
}
```

## Key Methods

```rust
// Type compatibility
a.is_assignable_to(&b) -> bool       // NOT types_compatible(a, b)
t.display_name() -> String
t.normalized() -> Type               // expand aliases
Type::union(members: Vec<Type>)
Type::intersection(members: Vec<Type>)

// TypeChecker
tc.check_expr(&expr) -> Type
tc.check_stmt(&stmt)
tc.check_function(&fn_decl)
tc.enter_scope() / tc.exit_scope()
tc.diagnostics.push(Diagnostic { ... })

// impl key lookup
type_to_impl_key(ty: &Type) -> Option<String>
// Number->"number", String->"string", Bool->"bool", Generic{name,[]}->"name"

// Return type inference (typechecker/inference.rs) — Block 5
infer_return_type(body: &Block) -> InferredReturn
// InferredReturn::Void | Uniform(Type) | Inconsistent { types, has_void_path }
// Use directly on func.body — do NOT rely on symbol table for inferred return types
```

## Anti-Patterns

```rust
// WRONG
types_compatible(a, b)          // → a.is_assignable_to(&b)
Type::TypeParameter("T".into()) // → Type::TypeParameter { name: "T".into() }
```
