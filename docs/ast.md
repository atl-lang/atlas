# Atlas AST (v0.1)

## Overview
This document defines the AST nodes used by the parser. Each node includes a span (start, end) for diagnostics.

## Common
- `Span { start: usize, end: usize, line: u32, column: u32 }`
- `Identifier { name: String, span: Span }`

## Program
- `Program { items: Vec<Item> }`

## Items
- `Item::Function(FunctionDecl)`
- `Item::Statement(Stmt)`

## Declarations
- `FunctionDecl { name: Identifier, params: Vec<Param>, return_type: TypeRef, body: Block, span: Span }`
- `Param { name: Identifier, type_ref: TypeRef, span: Span }`

## Statements
- `Stmt::VarDecl(VarDecl)`
- `Stmt::Assign(Assign)`
- `Stmt::If(IfStmt)`
- `Stmt::While(WhileStmt)`
- `Stmt::For(ForStmt)`
- `Stmt::Return(ReturnStmt)`
- `Stmt::Break(Span)`
- `Stmt::Continue(Span)`
- `Stmt::Expr(ExprStmt)`

### Statement Nodes
- `VarDecl { mutable: bool, name: Identifier, type_ref: Option<TypeRef>, init: Expr, span: Span }`
- `Assign { target: AssignTarget, value: Expr, span: Span }`
- `IfStmt { cond: Expr, then_block: Block, else_block: Option<Block>, span: Span }`
- `WhileStmt { cond: Expr, body: Block, span: Span }`
- `ForStmt { init: Box<Stmt>, cond: Expr, step: Box<Stmt>, body: Block, span: Span }`
- `ReturnStmt { value: Option<Expr>, span: Span }`
- `ExprStmt { expr: Expr, span: Span }`

## Expressions
- `Expr::Literal(Literal)`
- `Expr::Identifier(Identifier)`
- `Expr::Unary(UnaryExpr)`
- `Expr::Binary(BinaryExpr)`
- `Expr::Call(CallExpr)`
- `Expr::Index(IndexExpr)`
- `Expr::ArrayLiteral(ArrayLiteral)`
- `Expr::Group(GroupExpr)`

### Expression Nodes
- `UnaryExpr { op: UnaryOp, expr: Box<Expr>, span: Span }`
- `BinaryExpr { op: BinaryOp, left: Box<Expr>, right: Box<Expr>, span: Span }`
- `CallExpr { callee: Box<Expr>, args: Vec<Expr>, span: Span }`
- `IndexExpr { target: Box<Expr>, index: Box<Expr>, span: Span }`
- `ArrayLiteral { elements: Vec<Expr>, span: Span }`
- `GroupExpr { expr: Box<Expr>, span: Span }`

## Literals
- `Literal::Number(f64)`
- `Literal::String(String)`
- `Literal::Bool(bool)`
- `Literal::Null`

## Types
- `TypeRef::Named(String, Span)`
- `TypeRef::Array(Box<TypeRef>, Span)`

## Operators
- `UnaryOp::Negate` ("-")
- `UnaryOp::Not` ("!")
- `BinaryOp::Add` ("+")
- `BinaryOp::Sub` ("-")
- `BinaryOp::Mul` ("*")
- `BinaryOp::Div` ("/")
- `BinaryOp::Mod` ("%")
- `BinaryOp::Eq` ("==")
- `BinaryOp::Ne` ("!=")
- `BinaryOp::Lt` ("<")
- `BinaryOp::Le` ("<=")
- `BinaryOp::Gt` (">")
- `BinaryOp::Ge` (">=")
- `BinaryOp::And` ("&&")
- `BinaryOp::Or` ("||")

## Assignment Targets
- `AssignTarget::Name(Identifier)`
- `AssignTarget::Index { target: Box<Expr>, index: Box<Expr>, span: Span }`
