//! AST instantiation tests part 2 (lines 1851-2069)

use super::*;

fn test_all_expression_types() {
    let expressions = vec![
        // Literal expressions
        Expr::Literal(Literal::Number(42.0), Span::new(0, 2)),
        Expr::Literal(Literal::String("hello".to_string()), Span::new(0, 7)),
        Expr::Literal(Literal::Bool(true), Span::new(0, 4)),
        Expr::Literal(Literal::Null, Span::new(0, 4)),
        // Identifier
        Expr::Identifier(Identifier {
            name: "x".to_string(),
            span: Span::new(0, 1),
        }),
        // Unary expressions
        Expr::Unary(UnaryExpr {
            op: UnaryOp::Negate,
            expr: Box::new(Expr::Literal(Literal::Number(5.0), Span::new(1, 2))),
            span: Span::new(0, 2),
        }),
        Expr::Unary(UnaryExpr {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Literal(Literal::Bool(true), Span::new(1, 5))),
            span: Span::new(0, 5),
        }),
        // Binary expression
        Expr::Binary(BinaryExpr {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::Number(1.0), Span::new(0, 1))),
            right: Box::new(Expr::Literal(Literal::Number(2.0), Span::new(4, 5))),
            span: Span::new(0, 5),
        }),
        // Call expression
        Expr::Call(CallExpr {
            callee: Box::new(Expr::Identifier(Identifier {
                name: "func".to_string(),
                span: Span::new(0, 4),
            })),
            args: vec![],
            span: Span::new(0, 6),
        }),
        // Index expression
        Expr::Index(IndexExpr {
            target: Box::new(Expr::Identifier(Identifier {
                name: "arr".to_string(),
                span: Span::new(0, 3),
            })),
            index: Box::new(Expr::Literal(Literal::Number(0.0), Span::new(4, 5))),
            span: Span::new(0, 6),
        }),
        // Array literal
        Expr::ArrayLiteral(ArrayLiteral {
            elements: vec![
                Expr::Literal(Literal::Number(1.0), Span::new(1, 2)),
                Expr::Literal(Literal::Number(2.0), Span::new(4, 5)),
                Expr::Literal(Literal::Number(3.0), Span::new(7, 8)),
            ],
            span: Span::new(0, 9),
        }),
        // Grouped expression
        Expr::Group(GroupExpr {
            expr: Box::new(Expr::Literal(Literal::Number(42.0), Span::new(1, 3))),
            span: Span::new(0, 4),
        }),
    ];

    assert_eq!(expressions.len(), 12);

    // Verify all expressions have valid spans
    for expr in &expressions {
        let span = expr.span();
        assert!(!span.is_empty() || span == Span::new(0, 4)); // Allow null literal span
    }
}

#[test]
fn test_all_binary_operators() {
    let operators = vec![
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Mod,
        BinaryOp::Eq,
        BinaryOp::Ne,
        BinaryOp::Lt,
        BinaryOp::Le,
        BinaryOp::Gt,
        BinaryOp::Ge,
        BinaryOp::And,
        BinaryOp::Or,
    ];

    assert_eq!(operators.len(), 13);

    // Verify all operators can be used in expressions
    for op in operators {
        let expr = BinaryExpr {
            op,
            left: Box::new(Expr::Literal(Literal::Number(1.0), Span::new(0, 1))),
            right: Box::new(Expr::Literal(Literal::Number(2.0), Span::new(4, 5))),
            span: Span::new(0, 5),
        };

        assert_eq!(expr.op, op);
    }
}

#[test]
fn test_nested_expressions() {
    // Test deeply nested expression: (1 + 2) * (3 - 4)
    let expr = Expr::Binary(BinaryExpr {
        op: BinaryOp::Mul,
        left: Box::new(Expr::Group(GroupExpr {
            expr: Box::new(Expr::Binary(BinaryExpr {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::Number(1.0), Span::new(1, 2))),
                right: Box::new(Expr::Literal(Literal::Number(2.0), Span::new(5, 6))),
                span: Span::new(1, 6),
            })),
            span: Span::new(0, 7),
        })),
        right: Box::new(Expr::Group(GroupExpr {
            expr: Box::new(Expr::Binary(BinaryExpr {
                op: BinaryOp::Sub,
                left: Box::new(Expr::Literal(Literal::Number(3.0), Span::new(11, 12))),
                right: Box::new(Expr::Literal(Literal::Number(4.0), Span::new(15, 16))),
                span: Span::new(11, 16),
            })),
            span: Span::new(10, 17),
        })),
        span: Span::new(0, 17),
    });

    assert_eq!(expr.span(), Span::new(0, 17));

    if let Expr::Binary(binary) = expr {
        assert_eq!(binary.op, BinaryOp::Mul);
        assert!(matches!(*binary.left, Expr::Group(_)));
        assert!(matches!(*binary.right, Expr::Group(_)));
    }
}

#[test]
fn test_array_type_ref() {
    // Test array type: number[][]
    let arr_type = TypeRef::Array(
        Box::new(TypeRef::Array(
            Box::new(TypeRef::Named("number".to_string(), Span::new(0, 6))),
            Span::new(0, 8),
        )),
        Span::new(0, 10),
    );

    assert_eq!(arr_type.span(), Span::new(0, 10));

    // Verify nested structure
    if let TypeRef::Array(inner, _) = arr_type {
        if let TypeRef::Array(inner_inner, _) = *inner {
            if let TypeRef::Named(name, _) = *inner_inner {
                assert_eq!(name, "number");
            } else {
                panic!("Expected named type");
            }
        } else {
            panic!("Expected array type");
        }
    } else {
        panic!("Expected array type");
    }
}

#[test]
fn test_assignment_target_variants() {
    // Test name assignment target
    let name_target = AssignTarget::Name(Identifier {
        name: "x".to_string(),
        span: Span::new(0, 1),
    });

    assert!(matches!(name_target, AssignTarget::Name(_)));

    // Test index assignment target
    let index_target = AssignTarget::Index {
        target: Box::new(Expr::Identifier(Identifier {
            name: "arr".to_string(),
            span: Span::new(0, 3),
        })),
        index: Box::new(Expr::Literal(Literal::Number(0.0), Span::new(4, 5))),
        span: Span::new(0, 6),
    };

    assert!(matches!(index_target, AssignTarget::Index { .. }));
}

#[test]
fn test_ast_serialization() {
    // Test that AST nodes can be serialized to JSON
    let program = Program {
        items: vec![Item::Statement(Stmt::VarDecl(VarDecl {
            mutable: false,
            name: Identifier {
                name: "x".to_string(),
                span: Span::new(4, 5),
            },
            type_ref: Some(TypeRef::Named("number".to_string(), Span::new(7, 13))),
            init: Expr::Literal(Literal::Number(42.0), Span::new(16, 18)),
            span: Span::new(0, 19),
        }))],
    };

    // Serialize to JSON
    let json = serde_json::to_string(&program).expect("Failed to serialize AST");

    // Deserialize back
    let deserialized: Program = serde_json::from_str(&json).expect("Failed to deserialize AST");

    assert_eq!(program, deserialized);
}

// ============================================================================
