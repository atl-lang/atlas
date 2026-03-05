### Fully Implemented
| AST Node | Parser | Interpreter | VM |
|----------|--------|-------------|-----|
| Program | ✅ | ✅ | ✅ |
| Item | ✅ | ✅ | ✅ |
| ImportDecl | ✅ | ✅ | ✅ |
| ImportSpecifier | ✅ | ✅ | ✅ |
| ExportDecl | ✅ | ✅ | ✅ |
| ExportItem | ✅ | ✅ | ✅ |
| ExternDecl | ✅ | ✅ | ✅ |
| ExternTypeAnnotation | ✅ | ✅ | ✅ |
| TypeAliasDecl | ✅ | ✅ | ✅ |
| FunctionDecl | ✅ | ✅ | ✅ |
| TraitDecl | ✅ | ✅ | ✅ |
| ImplMethod | ✅ | ✅ | ✅ |
| ImplBlock | ✅ | ✅ | ✅ |
| StructDecl | ✅ | ✅ | ✅ |
| EnumDecl | ✅ | ✅ | ✅ |
| OwnershipAnnotation | ✅ | ✅ | ✅ |
| Param | ✅ | ✅ | ✅ |
| Block | ✅ | ✅ | ✅ |
| Stmt | ✅ | ✅ | ✅ |
| VarDecl | ✅ | ✅ | ✅ |
| Assign | ✅ | ✅ | ✅ |
| AssignTarget | ✅ | ✅ | ✅ |
| CompoundOp | ✅ | ✅ | ✅ |
| CompoundAssign | ✅ | ✅ | ✅ |
| IfStmt | ✅ | ✅ | ✅ |
| WhileStmt | ✅ | ✅ | ✅ |
| ForInStmt | ✅ | ✅ | ✅ |
| ReturnStmt | ✅ | ✅ | ✅ |
| ExprStmt | ✅ | ✅ | ✅ |
| Expr | ✅ | ✅ | ✅ |
| UnaryExpr | ✅ | ✅ | ✅ |
| BinaryExpr | ✅ | ✅ | ✅ |
| CallExpr | ✅ | ✅ | ✅ |
| IndexExpr | ✅ | ✅ | ✅ |
| IndexValue | ✅ | ✅ | ✅ |
| MemberExpr | ✅ | ✅ | ✅ |
| ArrayLiteral | ✅ | ✅ | ✅ |
| ObjectLiteral | ✅ | ✅ | ✅ |
| ObjectEntry | ✅ | ✅ | ✅ |
| StructExpr | ✅ | ✅ | ✅ |
| StructFieldInit | ✅ | ✅ | ✅ |
| GroupExpr | ✅ | ✅ | ✅ |
| EnumVariantExpr | ✅ | ✅ | ✅ |
| TryExpr | ✅ | ✅ | ✅ |
| MatchExpr | ✅ | ✅ | ✅ |
| MatchArm | ✅ | ✅ | ✅ |
| Pattern | ✅ | ✅ | ✅ |
| Literal | ✅ | ✅ | ✅ |
| TemplatePart | ✅ | ✅ | ✅ |
| Identifier | ✅ | ✅ | ✅ |
| UnaryOp | ✅ | ✅ | ✅ |
| BinaryOp | ✅ | ✅ | ✅ |

### Partial Implementation (IN PROGRESS - DO NOT DELETE)
| AST Node | Parser | Interpreter | Notes |
|----------|--------|-------------|-------|
| VersionedProgram | ❌ | ❌ | JSON wrapper for tooling; not produced by parser/runtime. |
| TypePredicate | ✅ | ❌ | Type-guard metadata; handled by typechecker only. |
| TraitBound | ✅ | ❌ | Type-parameter bounds; typechecker only. |
| TraitMethodSig | ✅ | ❌ | Trait signature metadata; typechecker only. |
| StructField | ✅ | ❌ | Struct schema only; runtime is type-info only. |
| EnumVariant | ✅ | ❌ | Enum schema only; runtime is type-info only. |
| TypeParam | ✅ | ❌ | Generic parameter metadata; typechecker only. |
| TryTargetKind | ❌ | ❌ | Set by typechecker; used by compiler for `?` op selection. |
| TypeRef | ✅ | ❌ | Type AST; typechecker only. |
| StructuralMember | ✅ | ❌ | Structural type metadata; typechecker only. |

### Removed Features (artifacts may exist)
| Feature | Removal Commit | Issue |
|---------|----------------|-------|
| C-style for loop (ForStmt) | fd0c8f5 | H-034 |
| ++ / -- operators (IncrementStmt, DecrementStmt) | fd0c8f5 | H-034 |
