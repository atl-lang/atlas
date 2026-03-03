//! AST instantiation tests part 1 (lines 1619-1850)

use super::*;

// AST Instantiation Tests (from ast_instantiation.rs)
// ============================================================================

#[test]
fn test_complete_program_construction() {
    // Build a complete program with various node types
    let program = Program {
        items: vec![
            // Function declaration
            Item::Function(FunctionDecl {
                name: Identifier {
                    name: "add".to_string(),
                    span: Span::new(5, 8),
                },
                type_params: vec![],
                params: vec![
                    Param {
                        name: Identifier {
                            name: "a".to_string(),
                            span: Span::new(9, 10),
                        },
                        type_ref: Some(TypeRef::Named("number".to_string(), Span::new(12, 18))),
                        ownership: None,
                        span: Span::new(9, 18),
                    },
                    Param {
                        name: Identifier {
                            name: "b".to_string(),
                            span: Span::new(20, 21),
                        },
                        type_ref: Some(TypeRef::Named("number".to_string(), Span::new(23, 29))),
                        ownership: None,
                        span: Span::new(20, 29),
                    },
                ],
                return_type: Some(TypeRef::Named("number".to_string(), Span::new(34, 40))),
                return_ownership: None,
                predicate: None,
                body: Block {
                    statements: vec![Stmt::Return(ReturnStmt {
                        value: Some(Expr::Binary(BinaryExpr {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::Identifier(Identifier {
                                name: "a".to_string(),
                                span: Span::new(50, 51),
                            })),
                            right: Box::new(Expr::Identifier(Identifier {
                                name: "b".to_string(),
                                span: Span::new(54, 55),
                            })),
                            span: Span::new(50, 55),
                        })),
                        span: Span::new(43, 56),
                    })],
                    tail_expr: None,
                    span: Span::new(41, 58),
                },
                span: Span::new(0, 58),
            }),
            // Variable declaration statement
            Item::Statement(Stmt::VarDecl(VarDecl {
                mutable: false,
                uses_deprecated_var: false,
                name: Identifier {
                    name: "result".to_string(),
                    span: Span::new(64, 70),
                },
                type_ref: Some(TypeRef::Named("number".to_string(), Span::new(72, 78))),
                init: Expr::Call(CallExpr {
                    callee: Box::new(Expr::Identifier(Identifier {
                        name: "add".to_string(),
                        span: Span::new(81, 84),
                    })),
                    args: vec![
                        Expr::Literal(Literal::Number(5.0), Span::new(85, 86)),
                        Expr::Literal(Literal::Number(3.0), Span::new(88, 89)),
                    ],
                    span: Span::new(81, 90),
                }),
                span: Span::new(60, 91),
            })),
        ],
    };

    // Verify structure
    assert_eq!(program.items.len(), 2);

    // Verify function
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.name.name, "add");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.body.statements.len(), 1);
    } else {
        panic!("Expected function declaration");
    }

    // Verify variable declaration
    if let Item::Statement(Stmt::VarDecl(var_decl)) = &program.items[1] {
        assert_eq!(var_decl.name.name, "result");
        assert!(!var_decl.mutable);
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
fn test_all_statement_types() {
    let statements = vec![
        // Variable declaration
        Stmt::VarDecl(VarDecl {
            mutable: true,
            uses_deprecated_var: false,
            name: Identifier {
                name: "x".to_string(),
                span: Span::new(0, 1),
            },
            type_ref: None,
            init: Expr::Literal(Literal::Number(42.0), Span::new(4, 6)),
            span: Span::new(0, 7),
        }),
        // Assignment
        Stmt::Assign(Assign {
            target: AssignTarget::Name(Identifier {
                name: "x".to_string(),
                span: Span::new(0, 1),
            }),
            value: Expr::Literal(Literal::Number(100.0), Span::new(4, 7)),
            span: Span::new(0, 8),
        }),
        // If statement
        Stmt::If(IfStmt {
            cond: Expr::Literal(Literal::Bool(true), Span::new(4, 8)),
            then_block: Block {
                statements: vec![],
                tail_expr: None,
                span: Span::new(9, 11),
            },
            else_block: Some(Block {
                statements: vec![],
                tail_expr: None,
                span: Span::new(17, 19),
            }),
            span: Span::new(0, 19),
        }),
        // While loop
        Stmt::While(WhileStmt {
            cond: Expr::Literal(Literal::Bool(true), Span::new(6, 10)),
            body: Block {
                statements: vec![],
                tail_expr: None,
                span: Span::new(11, 13),
            },
            span: Span::new(0, 13),
        }),
        // For-in loop
        Stmt::ForIn(ForInStmt {
            variable: Identifier {
                name: "i".to_string(),
                span: Span::new(4, 5),
            },
            iterable: Box::new(Expr::Identifier(Identifier {
                name: "items".to_string(),
                span: Span::new(9, 14),
            })),
            body: Block {
                statements: vec![],
                tail_expr: None,
                span: Span::new(16, 18),
            },
            span: Span::new(0, 18),
        }),
        // Return statement
        Stmt::Return(ReturnStmt {
            value: Some(Expr::Literal(Literal::Number(42.0), Span::new(7, 9))),
            span: Span::new(0, 10),
        }),
        // Break statement
        Stmt::Break(Span::new(0, 5)),
        // Continue statement
        Stmt::Continue(Span::new(0, 8)),
        // Expression statement
        Stmt::Expr(ExprStmt {
            expr: Expr::Call(CallExpr {
                callee: Box::new(Expr::Identifier(Identifier {
                    name: "print".to_string(),
                    span: Span::new(0, 5),
                })),
                args: vec![Expr::Literal(
                    Literal::String("hello".to_string()),
                    Span::new(6, 13),
                )],
                span: Span::new(0, 14),
            }),
            span: Span::new(0, 15),
        }),
    ];

    assert_eq!(statements.len(), 9);

    // Verify each statement can be pattern matched
    assert!(matches!(statements[0], Stmt::VarDecl(_)));
    assert!(matches!(statements[1], Stmt::Assign(_)));
    assert!(matches!(statements[2], Stmt::If(_)));
    assert!(matches!(statements[3], Stmt::While(_)));
    assert!(matches!(statements[4], Stmt::ForIn(_)));
    assert!(matches!(statements[5], Stmt::Return(_)));
    assert!(matches!(statements[6], Stmt::Break(_)));
    assert!(matches!(statements[7], Stmt::Continue(_)));
    assert!(matches!(statements[8], Stmt::Expr(_)));
}
