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
pub const AST_VERSION: u32 = 5;

/// An attribute annotation on a declaration.
///
/// Syntax: `@allow(unused)` or `@allow(dead_code)`
/// Currently only `@allow(lint_name)` is supported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    /// The attribute name — currently always "allow"
    pub name: String,
    /// The lint/argument inside the parens — e.g., "unused"
    pub arg: String,
    pub span: Span,
}

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
    /// Compile-time constant: `const PI = 3.14;`
    Const(ConstDecl),
    /// Trait declaration: `trait Foo { fn method(...) -> T; }`
    Trait(TraitDecl),
    /// Impl block: `impl TraitName for TypeName { ... }`
    Impl(ImplBlock),
    /// Struct declaration: `struct User { name: string, age: number }`
    Struct(StructDecl),
    /// Enum declaration: `enum Color { Red, Green, Blue, Rgb(number, number, number) }`
    Enum(EnumDecl),
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
    /// Export const: `export const PI = 3.14`
    Const(ConstDecl),
    /// Export struct: `export struct Person { name: string }`
    Struct(StructDecl),
    /// Export enum: `export enum Status { Active, Inactive }`
    Enum(EnumDecl),
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

/// Compile-time constant declaration
///
/// Syntax: `const NAME: Type = expr;` or `const NAME = expr;`
/// The initializer must be compile-time evaluable (literals, const math, other consts).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstDecl {
    pub name: Identifier,
    /// Optional type annotation
    pub type_ref: Option<TypeRef>,
    /// Initializer expression (must be compile-time evaluable)
    pub init: Expr,
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
    /// Visibility modifier (`pub`, `private`, `internal`). Default is `Private`.
    pub visibility: Visibility,
    /// Attributes on this declaration (e.g., `@allow(unused)`)
    pub attributes: Vec<Attribute>,
    /// Whether this function is declared with the `async` keyword
    pub is_async: bool,
    /// Type parameters (e.g., <T, E> in fn foo<T, E>(...))
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    /// `None` means the return type is omitted and will be inferred
    pub return_type: Option<TypeRef>,
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
/// May include a default body; implementations can override it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethodSig {
    pub name: Identifier,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub body: Option<Block>,
    pub span: Span,
}

/// A trait declaration.
///
/// Syntax: `trait Foo { fn method(self: Foo, arg: T) -> R; }`
///
/// Trait bodies contain method signatures and optional default implementations.
/// Implementations live in `ImplBlock`, but may inherit defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDecl {
    pub name: Identifier,
    /// Visibility modifier (`pub`, `private`, `internal`). Default is `Private`.
    pub visibility: Visibility,
    /// Attributes on this declaration (e.g., `@allow(unused)`)
    pub attributes: Vec<Attribute>,
    /// Type parameters for generic traits (e.g., `trait Functor<T>`)
    pub type_params: Vec<TypeParam>,
    /// Supertrait bounds (e.g., `trait B: A + C`)
    pub super_traits: Vec<String>,
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
    /// True if declared with `static fn` — no self parameter, called as Type.method()
    pub is_static: bool,
}

/// An impl block.
///
/// Two forms:
/// - Inherent: `impl TypeName { fn method(borrow self) -> T { } }`
/// - Trait:    `impl TraitName for TypeName { fn method(borrow self) -> T { } }`
///
/// `trait_name` is `None` for inherent impls, `Some(name)` for trait impls.
/// `type_name` is the type being implemented (e.g., "Buffer").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplBlock {
    /// `None` = inherent impl; `Some(name)` = trait impl.
    pub trait_name: Option<Identifier>,
    /// Type arguments applied to the trait (e.g., `impl Functor<number> for MyType`).
    /// Empty for inherent impls.
    pub trait_type_args: Vec<TypeRef>,
    pub type_name: Identifier,
    pub methods: Vec<ImplMethod>,
    pub span: Span,
}

impl ImplBlock {
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns `true` when this is an inherent impl (`impl TypeName {}`),
    /// i.e. not attached to any trait.
    pub fn is_inherent(&self) -> bool {
        self.trait_name.is_none()
    }

    /// Produces the mangled name for a method inside this impl block.
    /// - Inherent:  `__impl__TypeName__MethodName`
    /// - Trait:     `__impl__TypeName__TraitName__MethodName`
    pub fn mangle_method_name(&self, method_name: &str) -> String {
        match &self.trait_name {
            None => format!("__impl__{}__{}", self.type_name.name, method_name),
            Some(t) => format!(
                "__impl__{}__{}__{}",
                self.type_name.name, t.name, method_name
            ),
        }
    }
}

/// A field in a struct declaration.
///
/// Syntax: `name: type`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: Identifier,
    pub type_ref: TypeRef,
    pub span: Span,
}

/// A struct declaration.
///
/// Syntax: `struct User { name: string, age: number }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructDecl {
    /// Visibility modifier (`pub`, `private`, `internal`). Default is `Private`.
    pub visibility: Visibility,
    /// Attributes on this declaration (e.g., `@allow(unused)`)
    pub attributes: Vec<Attribute>,
    pub name: Identifier,
    /// Type parameters for generic structs (e.g., `struct Pair<T, U>`)
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<StructField>,
    pub span: Span,
}

impl StructDecl {
    pub fn span(&self) -> Span {
        self.span
    }
}

/// A variant in an enum declaration.
///
/// Variants can be:
/// - Unit: `Red` (no associated data)
/// - Tuple: `Rgb(number, number, number)` (positional data)
/// - Struct: `Named { x: number, y: number }` (named fields)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnumVariant {
    /// Unit variant: `Red`
    Unit { name: Identifier, span: Span },
    /// Tuple variant: `Rgb(number, number, number)`
    Tuple {
        name: Identifier,
        fields: Vec<TypeRef>,
        span: Span,
    },
    /// Struct variant: `Named { x: number, y: number }`
    Struct {
        name: Identifier,
        fields: Vec<StructField>,
        span: Span,
    },
}

impl EnumVariant {
    pub fn name(&self) -> &Identifier {
        match self {
            EnumVariant::Unit { name, .. } => name,
            EnumVariant::Tuple { name, .. } => name,
            EnumVariant::Struct { name, .. } => name,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            EnumVariant::Unit { span, .. } => *span,
            EnumVariant::Tuple { span, .. } => *span,
            EnumVariant::Struct { span, .. } => *span,
        }
    }
}

/// An enum declaration.
///
/// Syntax: `enum Color { Red, Green, Blue, Rgb(number, number, number) }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDecl {
    pub name: Identifier,
    /// Visibility modifier (`pub`, `private`, `internal`). Default is `Private`.
    pub visibility: Visibility,
    /// Type parameters for generic enums (e.g., `enum Option<T>`)
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

impl EnumDecl {
    pub fn span(&self) -> Span {
        self.span
    }
}

/// Type parameter declaration (e.g., T in fn foo<T>(...))
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
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
    /// `share param: T` — shared reference; both caller and callee hold valid refs, neither mutates
    Share,
}

/// Visibility modifier for declarations (B37: Systems-Level Completion)
/// Determines which scopes can access the declared item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Visibility {
    /// `pub` — accessible from any module
    Public,
    /// `private` (default) — accessible only within the same file
    #[default]
    Private,
    /// `internal` — accessible within the same module (all files in the module)
    Internal,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: Identifier,
    /// Type annotation. Always present. Use `TypeRef::SelfType` for bare `self`
    /// params whose concrete type is resolved later from the impl context.
    pub type_ref: TypeRef,
    /// Ownership annotation (`own`, `borrow`, `share`), or `None` if unannotated.
    /// For bare params the parser defaults this to `Some(Borrow)` per D-040,
    /// but `ownership_explicit` distinguishes written-in-source from defaulted.
    pub ownership: Option<OwnershipAnnotation>,
    /// `true` when the ownership keyword was present in source (`borrow x`, `own x`, `share x`).
    /// `false` for bare params where borrow is the implicit default (D-040).
    /// Use this to guard AT3054 — bare-param returns are always valid.
    pub ownership_explicit: bool,
    /// `mut` keyword present — parameter can be reassigned inside the function body.
    pub mutable: bool,
    /// Default value for optional parameters (B39-P05).
    /// If present, this parameter can be omitted at call sites.
    pub default_value: Option<Box<Expr>>,
    /// Rest parameter (`...args: T[]`) — must be last param. Collects remaining call args into an array.
    pub is_rest: bool,
    pub span: Span,
}

/// Block of statements
///
/// Supports Rust-style implicit returns: if the last item in a block is an expression
/// without a trailing semicolon, it becomes the block's value (stored in `tail_expr`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Stmt>,
    /// Optional trailing expression (implicit return value)
    pub tail_expr: Option<Box<Expr>>,
    pub span: Span,
}

/// Statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    VarDecl(VarDecl),
    LetDestructure(LetDestructure),
    FunctionDecl(FunctionDecl),
    Assign(Assign),
    CompoundAssign(CompoundAssign),
    If(IfStmt),
    While(WhileStmt),
    ForIn(ForInStmt),
    Return(ReturnStmt),
    Break(Span),
    Continue(Span),
    Expr(ExprStmt),
    /// Deferred execution: runs on scope exit in LIFO order
    Defer(DeferStmt),
}

/// Variable declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarDecl {
    pub mutable: bool,
    /// True if declared with deprecated `var` keyword instead of `let`/`let mut`.
    /// Used to emit AT2014 deprecation warning.
    pub uses_deprecated_var: bool,
    pub name: Identifier,
    pub type_ref: Option<TypeRef>,
    pub init: Expr,
    pub span: Span,
    /// Drop annotation: if Some(type_name), compiler emits drop call at scope exit.
    /// Set by typechecker when variable's type implements Drop trait.
    #[serde(skip)]
    pub needs_drop: std::cell::RefCell<Option<String>>,
}

/// Tuple destructuring declaration: `let (a, b) = expr;`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetDestructure {
    pub mutable: bool,
    pub names: Vec<Identifier>,
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

/// Assignment target (name, indexed expression, or member)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignTarget {
    Name(Identifier),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Member {
        target: Box<Expr>,
        member: Identifier,
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

/// Defer statement — executes block on scope exit (LIFO order)
///
/// Syntax: `defer { cleanup(); }` or `defer cleanup();`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeferStmt {
    pub body: Block,
    pub span: Span,
}

/// Expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(Literal, Span),
    TemplateString {
        parts: Vec<TemplatePart>,
        span: Span,
    },
    Identifier(Identifier),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Index(IndexExpr),
    Member(MemberExpr),
    ArrayLiteral(ArrayLiteral),
    /// Object literal expression: `record { key: value, key2: value2 }`
    /// or anonymous struct literal `{ key: value, key }`.
    ObjectLiteral(ObjectLiteral),
    /// Struct instantiation: `User { name: "Alice", age: 30 }`
    StructExpr(StructExpr),
    /// Range expression: `1..3`, `1..=3`, `..3`, `1..`
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    },
    Group(GroupExpr),
    /// Tuple literal: `(1, 2)`, `(1,)` single-element, `()` unit — B15
    TupleLiteral {
        elements: Vec<Expr>,
        span: Span,
    },
    Match(MatchExpr),
    Try(TryExpr),
    /// Anonymous function expression.
    ///
    /// Syntax: `fn(x: number, y: number) -> number { x + y }`
    ///
    /// Type annotations are optional: `fn(x) { x + 1 }` is also valid.
    AnonFn {
        params: Vec<Param>,
        return_type: Option<TypeRef>,
        body: Box<Expr>,
        span: Span,
    },
    /// Block expression: `{ stmt* }`
    /// Used as the body of anonymous functions and other block-level expressions.
    Block(Block),
    /// Enum variant expression: `EnumName::VariantName` or `EnumName::VariantName(args)`
    EnumVariant(EnumVariantExpr),
    /// `await expr` — suspends until the Future resolves (B8)
    Await {
        expr: Box<Expr>,
        span: Span,
    },
    /// `new TypeName<TypeArgs>(args)` — collection/struct constructor (H-374)
    /// Examples: `new Map<string, number>()`, `new Set<string>()`
    New {
        /// The type being constructed (e.g. "Map", "Set", "Queue", "Stack")
        type_name: Identifier,
        /// Optional generic type arguments: `<K, V>` in `new Map<K, V>()`
        type_args: Vec<TypeRef>,
        /// Constructor arguments (usually empty for collections)
        args: Vec<Expr>,
        span: Span,
    },
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
    /// Type arguments for generic calls: `Json.parse<User>(str)`
    pub type_args: Vec<TypeRef>,
    pub span: Span,
}

/// Array index expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexExpr {
    pub target: Box<Expr>,
    pub index: IndexValue,
    pub span: Span,
}

/// Index expression payload (single index or slice)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndexValue {
    Single(Box<Expr>),
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
    /// Type arguments for generic method calls: `obj.method<T>(x)`
    pub type_args: Vec<TypeRef>,
    /// Type tag for method dispatch (set by typechecker, used by interpreter/compiler)
    #[serde(skip)]
    pub type_tag: Cell<Option<TypeTag>>,
    /// Trait dispatch info: (type_name, trait_name) when this is a user trait method call.
    /// Set by the typechecker, used by the compiler and interpreter for static dispatch.
    #[serde(skip)]
    pub trait_dispatch: std::cell::RefCell<Option<(String, String)>>,
    /// Static method dispatch: type_name when this is a `Type.staticMethod()` call.
    /// Set by the typechecker, used by the compiler. Static methods have no self parameter.
    #[serde(skip)]
    pub static_dispatch: std::cell::RefCell<Option<String>>,
    pub span: Span,
}

impl PartialEq for MemberExpr {
    fn eq(&self, other: &Self) -> bool {
        // type_tag, trait_dispatch, static_dispatch are ephemeral annotations — exclude from equality
        self.target == other.target
            && self.member == other.member
            && self.args == other.args
            && self.type_args == other.type_args
            && self.span == other.span
    }
}

/// Array literal expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayLiteral {
    pub elements: Vec<Expr>,
    pub span: Span,
}

/// Object literal expression: `record { key: value, key2: value2 }`
///
/// Desugars to HashMap<string, T> at runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectLiteral {
    /// Key-value pairs (keys are identifiers, values are expressions)
    pub entries: Vec<ObjectEntry>,
    pub span: Span,
}

/// A single key-value pair in an object literal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectEntry {
    pub key: Identifier,
    pub value: Expr,
    pub span: Span,
}

/// Struct instantiation expression.
///
/// Syntax: `User { name: "Alice", age: 30 }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructExpr {
    /// The struct type name (e.g., `User`)
    pub name: Identifier,
    /// Field initializers
    pub fields: Vec<StructFieldInit>,
    pub span: Span,
}

/// A field initializer in a struct expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructFieldInit {
    pub name: Identifier,
    pub value: Expr,
    pub span: Span,
}

/// Grouped expression (parenthesized)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Enum variant expression: `EnumName::VariantName` or `EnumName::VariantName(args)`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariantExpr {
    /// The enum type name (e.g., `State` in `State::Running`)
    pub enum_name: Identifier,
    /// The variant name (e.g., `Running` in `State::Running`)
    pub variant_name: Identifier,
    /// Arguments for tuple variants (e.g., `[x, y]` in `Color::Rgb(x, y, z)`)
    pub args: Option<Vec<Expr>>,
    pub span: Span,
}

/// Whether the `?` operator targets a Result or Option type.
/// Set by the typechecker, read by the compiler for opcode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TryTargetKind {
    Result,
    Option,
}

/// Try expression (error propagation operator ?)
///
/// Unwraps Ok/Some value or returns Err/None early from current function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryExpr {
    pub expr: Box<Expr>,
    /// Set by the typechecker to indicate whether `?` is applied to Result or Option.
    /// The compiler reads this to emit the correct opcodes.
    #[serde(skip)]
    pub target_kind: std::cell::RefCell<Option<TryTargetKind>>,
    pub span: Span,
}

impl PartialEq for TryExpr {
    fn eq(&self, other: &Self) -> bool {
        // target_kind is an ephemeral annotation — exclude from equality
        self.expr == other.expr && self.span == other.span
    }
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

/// A single field binding in a struct pattern: `field_name` or `field_name: sub_pattern`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructFieldPattern {
    /// The struct field name being matched
    pub name: Identifier,
    /// Sub-pattern to match the field value against.
    /// `None` = shorthand: bind the field value to a variable with the same name.
    /// `Some(p)` = explicit: `field: pattern` form.
    pub pattern: Option<Pattern>,
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
    /// Tuple pattern: (p1, p2, ...)
    Tuple { elements: Vec<Pattern>, span: Span },
    /// OR pattern: pat1 | pat2 | pat3
    Or(Vec<Pattern>, Span),
    /// Enum variant pattern: State::Running, Color::Rgb(r, g, b)
    EnumVariant {
        enum_name: Identifier,
        variant_name: Identifier,
        args: Vec<Pattern>,
        span: Span,
    },
    /// Bare variant pattern: Running, Pending(msg) — no EnumName:: prefix.
    /// Parser emits this for uppercase identifiers not in the built-in constructor set.
    /// Typechecker resolves the enum type from the match scrutinee; at runtime only
    /// the variant_name is checked (enum_name is inferred, not required).
    BareVariant {
        name: Identifier,
        args: Vec<Pattern>,
        span: Span,
    },
    /// Struct/record pattern: `Point { x, y }` or `Point { x: px, y: py }`.
    /// `type_name` is `None` for anonymous-record patterns like `{ x, y }`.
    Struct {
        type_name: Option<Identifier>,
        fields: Vec<StructFieldPattern>,
        span: Span,
    },
}

/// Literal value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

/// Template string part
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplatePart {
    Literal(String),
    Expression(Box<Expr>),
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
    /// `Future<T>` — first-class async return type (B8)
    Future {
        inner: Box<TypeRef>,
        span: Span,
    },
    /// Tuple type: `(T1, T2, ...)` — B15
    Tuple {
        elements: Vec<TypeRef>,
        span: Span,
    },
    /// Placeholder for a bare `self` parameter whose type is inferred from the
    /// enclosing `impl` block. Never appears in user-written type positions.
    SelfType(Span),
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
            Expr::TemplateString { span, .. } => *span,
            Expr::Identifier(id) => id.span,
            Expr::Unary(u) => u.span,
            Expr::Binary(b) => b.span,
            Expr::Call(c) => c.span,
            Expr::Index(i) => i.span,
            Expr::Member(m) => m.span,
            Expr::ArrayLiteral(a) => a.span,
            Expr::ObjectLiteral(o) => o.span,
            Expr::StructExpr(s) => s.span,
            Expr::Range { span, .. } => *span,
            Expr::Group(g) => g.span,
            Expr::TupleLiteral { span, .. } => *span,
            Expr::Match(m) => m.span,
            Expr::Try(t) => t.span,
            Expr::AnonFn { span, .. } => *span,
            Expr::Block(block) => block.span,
            Expr::EnumVariant(e) => e.span,
            Expr::Await { span, .. } => *span,
            Expr::New { span, .. } => *span,
        }
    }
}

impl IndexValue {
    /// Get the span of this index value
    pub fn span(&self) -> Span {
        match self {
            IndexValue::Single(expr) => expr.span(),
        }
    }
}

impl Stmt {
    /// Get the span of this statement
    pub fn span(&self) -> Span {
        match self {
            Stmt::VarDecl(v) => v.span,
            Stmt::LetDestructure(d) => d.span,
            Stmt::FunctionDecl(f) => f.span,
            Stmt::Assign(a) => a.span,
            Stmt::CompoundAssign(c) => c.span,
            Stmt::If(i) => i.span,
            Stmt::While(w) => w.span,
            Stmt::ForIn(f) => f.span,
            Stmt::Return(r) => r.span,
            Stmt::Break(s) | Stmt::Continue(s) => *s,
            Stmt::Expr(e) => e.span,
            Stmt::Defer(d) => d.span,
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
            TypeRef::Future { span, .. } => *span,
            TypeRef::Tuple { span, .. } => *span,
            TypeRef::SelfType(span) => *span,
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
            Pattern::Tuple { span, .. } => *span,
            Pattern::Or(_, span) => *span,
            Pattern::EnumVariant { span, .. } => *span,
            Pattern::BareVariant { span, .. } => *span,
            Pattern::Struct { span, .. } => *span,
        }
    }
}
