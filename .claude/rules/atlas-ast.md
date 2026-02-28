---
globs: ["crates/atlas-runtime/src/ast.rs", "crates/atlas-runtime/src/parser/**", "crates/atlas-runtime/src/typechecker/**", "crates/atlas-runtime/src/binder.rs"]
---

# Atlas AST Quick-Ref

**Verified against:** `crates/atlas-runtime/src/ast.rs`
**Update trigger:** Any phase that adds/renames AST nodes — run atlas-doc-auditor at GATE 7.

---

## Stmt Enum (ast.rs:305)

```rust
pub enum Stmt {
    VarDecl(VarDecl),
    FunctionDecl(FunctionDecl),
    Assign(Assign),
    CompoundAssign(CompoundAssign),
    Increment(IncrementStmt),
    Decrement(DecrementStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    ForIn(ForInStmt),
    Return(ReturnStmt),
    Break(Span),
    Continue(Span),
    Expr(ExprStmt),
}
```

## Expr Enum (ast.rs:441)

```rust
pub enum Expr {
    Literal(Literal, Span),
    Identifier(Identifier),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Index(IndexExpr),
    Member(MemberExpr),
    ArrayLiteral(ArrayLiteral),
    Group(GroupExpr),
    Match(MatchExpr),
    Try(TryExpr),
    /// fn(params) -> return_type { body } OR (params) => expr
    AnonFn {
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Box<Expr>,   // Expr::Block for fn-syntax, any Expr for arrow
        span: Span,
    },
    Block(Block),
}
```

## Key Structs

```rust
// FunctionDecl
pub name: Identifier
pub type_params: Vec<TypeParam>
pub params: Vec<Param>
pub return_type: TypeRef          // NOT Option — defaults to null type
pub return_ownership: Option<OwnershipAnnotation>
pub predicate: Option<TypePredicate>
pub body: Block
pub span: Span

// VarDecl
pub mutable: bool                 // let = false, var = true
pub name: Identifier
pub type_ref: Option<TypeRef>
pub init: Expr
pub span: Span

// Param
pub name: Identifier
pub type_ref: Option<TypeRef>     // Option — arrow fn params may be untyped
pub ownership: Option<OwnershipAnnotation>
pub span: Span

// Block
pub statements: Vec<Stmt>
pub span: Span
// !! Block has NO tail_expr field !!

// Identifier
pub name: String
pub span: Span

// IfStmt
pub cond: Expr
pub then_block: Block
pub else_block: Option<Block>
pub span: Span
```

## TypeRef Enum (ast.rs:600)

```rust
pub enum TypeRef {
    Named(String, Span),
    Array(Box<TypeRef>, Span),
    Function { params: Vec<TypeRef>, return_type: Box<TypeRef>, span: Span },
    Structural { members: Vec<StructuralMember>, span: Span },
    Generic { name: String, type_args: Vec<TypeRef>, span: Span },
    Union { members: Vec<TypeRef>, span: Span },
    Intersection { members: Vec<TypeRef>, span: Span },
}
```

## OwnershipAnnotation (ast.rs:277)

```rust
pub enum OwnershipAnnotation { Own, Borrow, Shared }
```

## Anti-Patterns (verified wrong)

```rust
// WRONG
Stmt::Let(...)           // → Stmt::VarDecl(VarDecl)
Stmt::Var(...)           // → Stmt::VarDecl(VarDecl)
block.tail_expr          // → Block has no tail_expr
Expr::If(...)            // → Stmt::If(IfStmt), not an Expr variant
param.type_ref.unwrap()  // → Param.type_ref is Option<TypeRef>

// RIGHT
Stmt::VarDecl(VarDecl { mutable: false, .. })
block.statements.last()
```
