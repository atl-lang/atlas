# AST Structures

Complete Rust definitions matching `docs/ast.md`.

```rust
// ast.rs

pub struct Program {
    pub items: Vec<Item>,
}

pub enum Item {
    Function(FunctionDecl),
    Statement(Stmt),
}

pub struct FunctionDecl {
    pub name: Identifier,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub body: Block,
    pub span: Span,
}

pub struct Param {
    pub name: Identifier,
    pub type_ref: TypeRef,
    pub span: Span,
}

pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

pub enum Stmt {
    VarDecl(VarDecl),
    Assign(Assign),
    CompoundAssign(CompoundAssign),
    Increment(IncrementStmt),
    Decrement(DecrementStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Return(ReturnStmt),
    Break(Span),
    Continue(Span),
    Expr(ExprStmt),
}

pub struct VarDecl {
    pub mutable: bool,
    pub name: Identifier,
    pub type_ref: Option<TypeRef>,
    pub init: Expr,
    pub span: Span,
}

pub struct Assign {
    pub target: AssignTarget,
    pub value: Expr,
    pub span: Span,
}

pub struct CompoundAssign {
    pub target: AssignTarget,
    pub op: CompoundOp,
    pub value: Expr,
    pub span: Span,
}

pub struct IncrementStmt {
    pub target: AssignTarget,
    pub span: Span,
}

pub struct DecrementStmt {
    pub target: AssignTarget,
    pub span: Span,
}

pub enum CompoundOp {
    AddAssign,  // +=
    SubAssign,  // -=
    MulAssign,  // *=
    DivAssign,  // /=
    ModAssign,  // %=
}

pub enum AssignTarget {
    Name(Identifier),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
}

pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

pub struct WhileStmt {
    pub cond: Expr,
    pub body: Block,
    pub span: Span,
}

pub struct ForStmt {
    pub init: Box<Stmt>,
    pub cond: Expr,
    pub step: Box<Stmt>,
    pub body: Block,
    pub span: Span,
}

pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

pub enum Expr {
    Literal(Literal, Span),
    Identifier(Identifier),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Index(IndexExpr),
    ArrayLiteral(ArrayLiteral),
    Group(GroupExpr),
}

pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub span: Span,
}

pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

pub struct IndexExpr {
    pub target: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

pub struct ArrayLiteral {
    pub elements: Vec<Expr>,
    pub span: Span,
}

pub struct GroupExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

pub struct Identifier {
    pub name: String,
    pub span: Span,
}

pub enum TypeRef {
    Named(String, Span),
    Array(Box<TypeRef>, Span),
}

pub enum UnaryOp {
    Negate,  // -
    Not,     // !
}

pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}
```

## Helper Methods

```rust
impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Identifier(id) => id.span,
            Expr::Unary(u) => u.span,
            Expr::Binary(b) => b.span,
            Expr::Call(c) => c.span,
            Expr::Index(i) => i.span,
            Expr::ArrayLiteral(a) => a.span,
            Expr::Group(g) => g.span,
        }
    }
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl(v) => v.span,
            Stmt::Assign(a) => a.span,
            Stmt::CompoundAssign(c) => c.span,
            Stmt::Increment(i) => i.span,
            Stmt::Decrement(d) => d.span,
            Stmt::If(i) => i.span,
            Stmt::While(w) => w.span,
            Stmt::For(f) => f.span,
            Stmt::Return(r) => r.span,
            Stmt::Break(s) | Stmt::Continue(s) => *s,
            Stmt::Expr(e) => e.span,
        }
    }
}
```
