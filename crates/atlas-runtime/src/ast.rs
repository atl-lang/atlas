//! Abstract Syntax Tree (AST) definitions
//!
//! Complete AST implementation matching the Atlas specification.

use crate::method_dispatch::TypeTag;
use crate::span::Span;
use serde::{Deserialize, Serialize};
use std::cell::Cell;

/// AST schema version
///
/// This version number is included in JSON dumps to ensure compatibility.
/// Increment when making breaking changes to the AST structure.
pub const AST_VERSION: u32 = 2;

/// Top-level program containing all items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Versioned AST wrapper for JSON serialization
///
/// This struct wraps a Program with version metadata for stable JSON output.
/// Used when dumping AST to JSON for tooling and AI agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedProgram {
    /// AST schema version
    pub ast_version: u32,
    /// The actual program AST
    #[serde(flatten)]
    pub program: Program,
}

impl VersionedProgram {
    /// Create a new versioned program wrapper
    pub fn new(program: Program) -> Self {
        Self {
            ast_version: AST_VERSION,
            program,
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl From<Program> for VersionedProgram {
    fn from(program: Program) -> Self {
        Self::new(program)
    }
}

/// Top-level item (function, statement, import, export, or extern)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Function(FunctionDecl),
    Statement(Stmt),
    Import(ImportDecl),
    Export(ExportDecl),
    Extern(ExternDecl),
    TypeAlias(TypeAliasDecl),
    /// Trait declaration: `trait Foo { fn method(...) -> T; }`
    Trait(TraitDecl),
    /// Impl block: `impl TraitName for TypeName { ... }`
    Impl(ImplBlock),
}

/// Import declaration
///
/// Syntax: `import { x, y } from "./path"` or `import * as ns from "./path"`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportDecl {
    /// What to import (named imports or namespace)
    pub specifiers: Vec<ImportSpecifier>,
    /// Module path (e.g., "./math", "/src/utils")
    pub source: String,
    pub span: Span,
}

/// Import specifier (what to import)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportSpecifier {
    /// Named import: `{ x }`
    Named { name: Identifier, span: Span },
    /// Namespace import: `* as ns`
    Namespace { alias: Identifier, span: Span },
}

/// Export declaration
///
/// Syntax: `export fn foo()` or `export let x = 5`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportDecl {
    /// What is being exported
    pub item: ExportItem,
    pub span: Span,
}

/// Exportable items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportItem {
    /// Export function: `export fn foo() {}`
    Function(FunctionDecl),
    /// Export variable: `export let x = 5`
    Variable(VarDecl),
    /// Export type alias: `export type Foo = bar`
    TypeAlias(TypeAliasDecl),
}

/// Extern function declaration (FFI)
///
/// Syntax: `extern fn name(param: c_type, ...) -> c_type from "library"`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternDecl {
    pub name: String,
    pub library: String,
    pub symbol: Option<String>, // Optional symbol name (if different from name)
    pub params: Vec<(String, ExternTypeAnnotation)>,
    pub return_type: ExternTypeAnnotation,
    pub span: Span,
}

/// Type alias declaration
///
/// Syntax: `type Name = type_expr;`
/// Supports optional type parameters: `type Result<T, E> = ...;`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAliasDecl {
    pub name: Identifier,
    /// Type parameters (e.g., <T, E> in type Foo<T, E> = ...)
    pub type_params: Vec<TypeParam>,
    /// Aliased type expression
    pub type_ref: TypeRef,
    /// Optional doc comment text (without leading ///)
    pub doc_comment: Option<String>,
    pub span: Span,
}

/// Extern type annotation for FFI signatures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExternTypeAnnotation {
    CInt,
    CLong,
    CDouble,
    CCharPtr,
    CVoid,
    CBool,
}

/// Function declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub name: Identifier,
    /// Type parameters (e.g., <T, E> in fn foo<T, E>(...))
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    /// Ownership annotation on the return type, or `None` if unannotated
    pub return_ownership: Option<OwnershipAnnotation>,
    /// Optional type predicate for type guards (e.g., `-> bool is x: string`)
    pub predicate: Option<TypePredicate>,
    pub body: Block,
    pub span: Span,
}

/// Type predicate for type guard functions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypePredicate {
    pub param: Identifier,
    pub target: TypeRef,
    pub span: Span,
}

// ============================================================================
// Trait system (v0.3+)
// ============================================================================

/// A trait bound on a type parameter: `T: TraitName`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitBound {
    /// The trait name (e.g., "Copy", "Display", "MyTrait")
    pub trait_name: String,
    pub span: Span,
}

/// A method signature in a trait declaration.
/// Has no body — the body lives in the `ImplBlock`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethodSig {
    pub name: Identifier,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub span: Span,
}

/// A trait declaration.
///
/// Syntax: `trait Foo { fn method(self: Foo, arg: T) -> R; }`
///
/// Trait bodies contain only method signatures (no implementations).
/// Implementations live in `ImplBlock`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDecl {
    pub name: Identifier,
    /// Type parameters for generic traits (e.g., `trait Functor<T>`)
    pub type_params: Vec<TypeParam>,
    pub methods: Vec<TraitMethodSig>,
    pub span: Span,
}

impl TraitDecl {
    pub fn span(&self) -> Span {
        self.span
    }
}

/// A method implementation inside an `impl` block.
/// Identical in structure to `FunctionDecl` but scoped to an impl.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplMethod {
    pub name: Identifier,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub body: Block,
    pub span: Span,
}

/// An impl block.
///
/// Syntax: `impl TraitName for TypeName { fn method(...) { ... } }`
///
/// `trait_name` is the trait being implemented (e.g., "Display").
/// `type_name` is the type implementing the trait (e.g., "Buffer").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplBlock {
    pub trait_name: Identifier,
    /// Type arguments applied to the trait (e.g., `impl Functor<number> for MyType`)
    pub trait_type_args: Vec<TypeRef>,
    pub type_name: Identifier,
    pub methods: Vec<ImplMethod>,
    pub span: Span,
}

impl ImplBlock {
    pub fn span(&self) -> Span {
        self.span
    }
}

/// Type parameter declaration (e.g., T in fn foo<T>(...))
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
    /// Optional constraint bound (e.g., `T extends number`)
    pub bound: Option<TypeRef>,
    /// Trait bounds on this type parameter (e.g., `T: Copy + Display`)
    pub trait_bounds: Vec<TraitBound>,
    pub span: Span,
}

/// Ownership annotation on a function parameter or return type
///
/// Determines the memory transfer semantics at a call site.
/// `None` on a `Param` means unannotated — the typechecker applies the
/// default rule (value types copy implicitly; resource types require annotation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OwnershipAnnotation {
    /// `own param: T` — move semantics; caller's binding is invalidated after the call
    Own,
    /// `borrow param: T` — immutable borrow; caller retains ownership
    Borrow,
    /// `shared param: T` — shared mutable reference (Arc<Mutex<T>>)
    Shared,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: Identifier,
    /// Type annotation. `None` for untyped arrow-fn params — typechecker infers.
    pub type_ref: Option<TypeRef>,
    /// Ownership annotation (`own`, `borrow`, `shared`), or `None` if unannotated
    pub ownership: Option<OwnershipAnnotation>,
    pub span: Span,
}

/// Block of statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

/// Statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Variable declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarDecl {
    pub mutable: bool,
    pub name: Identifier,
    pub type_ref: Option<TypeRef>,
    pub init: Expr,
    pub span: Span,
}

/// Assignment statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assign {
    pub target: AssignTarget,
    pub value: Expr,
    pub span: Span,
}

/// Assignment target (name or indexed expression)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignTarget {
    Name(Identifier),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
}

/// Compound assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompoundOp {
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
}

/// Compound assignment statement (+=, -=, *=, /=, %=)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompoundAssign {
    pub target: AssignTarget,
    pub op: CompoundOp,
    pub value: Expr,
    pub span: Span,
}

/// Increment statement (++)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncrementStmt {
    pub target: AssignTarget,
    pub span: Span,
}

/// Decrement statement (--)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementStmt {
    pub target: AssignTarget,
    pub span: Span,
}

/// If statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

/// While loop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Block,
    pub span: Span,
}

/// For loop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForStmt {
    pub init: Box<Stmt>,
    pub cond: Expr,
    pub step: Box<Stmt>,
    pub body: Block,
    pub span: Span,
}

/// For-in loop statement
///
/// Syntax: `for item in array { body }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForInStmt {
    /// Loop variable name
    pub variable: Identifier,
    /// Expression to iterate over
    pub iterable: Box<Expr>,
    /// Loop body
    pub body: Block,
    pub span: Span,
}

/// Return statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

/// Expression statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

/// Expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Anonymous function expression.
    /// Syntax: `fn(x: number, y: number) -> number { x + y }`
    /// Arrow:  `(x) => x + 1`  (desugared to this by the parser)
    AnonFn {
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Box<Expr>,
        span: Span,
    },
    /// Block expression: `{ stmt* }`
    /// Used as the body of anonymous functions and other block-level expressions.
    Block(Block),
}

/// Unary expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Binary expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub span: Span,
}

/// Function call expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Array index expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexExpr {
    pub target: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

/// Member access expression (method call or property access)
///
/// Syntax: `expr.member` or `expr.method(args)`
/// This is sugar for function calls: `Type::method(expr, args)`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberExpr {
    /// The target expression (left side of dot)
    pub target: Box<Expr>,
    /// The member name (right side of dot)
    pub member: Identifier,
    /// Arguments if this is a method call, None if property access
    pub args: Option<Vec<Expr>>,
    /// Type tag for method dispatch (set by typechecker, used by interpreter/compiler)
    #[serde(skip)]
    pub type_tag: Cell<Option<TypeTag>>,
    /// Trait dispatch info: (type_name, trait_name) when this is a user trait method call.
    /// Set by the typechecker, used by the compiler and interpreter for static dispatch.
    #[serde(skip)]
    pub trait_dispatch: std::cell::RefCell<Option<(String, String)>>,
    pub span: Span,
}

impl PartialEq for MemberExpr {
    fn eq(&self, other: &Self) -> bool {
        // type_tag and trait_dispatch are ephemeral annotations — exclude from equality
        self.target == other.target
            && self.member == other.member
            && self.args == other.args
            && self.span == other.span
    }
}

/// Array literal expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayLiteral {
    pub elements: Vec<Expr>,
    pub span: Span,
}

/// Grouped expression (parenthesized)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Try expression (error propagation operator ?)
///
/// Unwraps Ok value or returns Err early from current function
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TryExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Match expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchExpr {
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Match arm (pattern => expression)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    /// Optional guard clause: `pattern if <guard> => body`
    pub guard: Option<Box<Expr>>,
    pub body: Expr,
    pub span: Span,
}

/// Pattern for match expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Literal pattern: 42, "hello", true, false, null
    Literal(Literal, Span),
    /// Wildcard pattern: _
    Wildcard(Span),
    /// Variable binding pattern: x, value, etc.
    Variable(Identifier),
    /// Constructor pattern: Ok(x), Err(e), Some(value), None
    Constructor {
        name: Identifier,
        args: Vec<Pattern>,
        span: Span,
    },
    /// Array pattern: [], [x], [x, y]
    Array { elements: Vec<Pattern>, span: Span },
    /// OR pattern: pat1 | pat2 | pat3
    Or(Vec<Pattern>, Span),
}

/// Literal value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

/// Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

/// Type reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeRef {
    Named(String, Span),
    Array(Box<TypeRef>, Span),
    Function {
        params: Vec<TypeRef>,
        return_type: Box<TypeRef>,
        span: Span,
    },
    /// Structural type: { field: type, method: (params) -> return }
    Structural {
        members: Vec<StructuralMember>,
        span: Span,
    },
    /// Generic type application: Type<T1, T2, ...>
    Generic {
        name: String,
        type_args: Vec<TypeRef>,
        span: Span,
    },
    /// Union type: A | B
    Union {
        members: Vec<TypeRef>,
        span: Span,
    },
    /// Intersection type: A & B
    Intersection {
        members: Vec<TypeRef>,
        span: Span,
    },
}

/// Structural type member
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralMember {
    pub name: String,
    pub type_ref: TypeRef,
    pub span: Span,
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate, // -
    Not,    // !
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}

// Helper methods for getting spans from AST nodes

impl Expr {
    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Identifier(id) => id.span,
            Expr::Unary(u) => u.span,
            Expr::Binary(b) => b.span,
            Expr::Call(c) => c.span,
            Expr::Index(i) => i.span,
            Expr::Member(m) => m.span,
            Expr::ArrayLiteral(a) => a.span,
            Expr::Group(g) => g.span,
            Expr::Match(m) => m.span,
            Expr::Try(t) => t.span,
            Expr::AnonFn { span, .. } => *span,
            Expr::Block(block) => block.span,
        }
    }
}

impl Stmt {
    /// Get the span of this statement
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl(v) => v.span,
            Stmt::FunctionDecl(f) => f.span,
            Stmt::Assign(a) => a.span,
            Stmt::CompoundAssign(c) => c.span,
            Stmt::Increment(i) => i.span,
            Stmt::Decrement(d) => d.span,
            Stmt::If(i) => i.span,
            Stmt::While(w) => w.span,
            Stmt::For(f) => f.span,
            Stmt::ForIn(f) => f.span,
            Stmt::Return(r) => r.span,
            Stmt::Break(s) | Stmt::Continue(s) => *s,
            Stmt::Expr(e) => e.span,
        }
    }
}

impl TypeRef {
    /// Get the span of this type reference
    pub fn span(&self) -> Span {
        match self {
            TypeRef::Named(_, span) => *span,
            TypeRef::Array(_, span) => *span,
            TypeRef::Function { span, .. } => *span,
            TypeRef::Structural { span, .. } => *span,
            TypeRef::Generic { span, .. } => *span,
            TypeRef::Union { span, .. } => *span,
            TypeRef::Intersection { span, .. } => *span,
        }
    }
}

impl Pattern {
    /// Get the span of this pattern
    pub fn span(&self) -> Span {
        match self {
            Pattern::Literal(_, span) => *span,
            Pattern::Wildcard(span) => *span,
            Pattern::Variable(id) => id.span,
            Pattern::Constructor { span, .. } => *span,
            Pattern::Array { span, .. } => *span,
            Pattern::Or(_, span) => *span,
        }
    }
}
