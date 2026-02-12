use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::ast::*;

fn parse_source(source: &str) -> (Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// Helper to extract expression from first statement
fn get_first_expr(program: &Program) -> &Expr {
    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => &expr_stmt.expr,
        _ => panic!("Expected expression statement"),
    }
}

// ========== Multiplication/Division over Addition/Subtraction ==========

#[test]
fn test_precedence_mul_over_add() {
    let (program, diagnostics) = parse_source("1 + 2 * 3;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: 1 + (2 * 3)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Add);
            match &*bin.right {
                Expr::Binary(right_bin) => assert_eq!(right_bin.op, BinaryOp::Mul),
                _ => panic!("Expected multiplication on right"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_precedence_div_over_sub() {
    let (program, diagnostics) = parse_source("10 - 6 / 2;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: 10 - (6 / 2)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Sub);
            match &*bin.right {
                Expr::Binary(right_bin) => assert_eq!(right_bin.op, BinaryOp::Div),
                _ => panic!("Expected division on right"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_precedence_mul_over_add_complex() {
    let (program, diagnostics) = parse_source("1 + 2 * 3 + 4;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 + (2 * 3)) + 4
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Add);
            // Left side should be: 1 + (2 * 3)
            match &*bin.left {
                Expr::Binary(left_bin) => {
                    assert_eq!(left_bin.op, BinaryOp::Add);
                    match &*left_bin.right {
                        Expr::Binary(inner) => assert_eq!(inner.op, BinaryOp::Mul),
                        _ => panic!("Expected multiplication"),
                    }
                }
                _ => panic!("Expected binary on left"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Comparison over Logical AND/OR ==========

#[test]
fn test_precedence_comparison_over_and() {
    let (program, diagnostics) = parse_source("1 < 2 && 3 > 4;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 < 2) && (3 > 4)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::And);
            match (&*bin.left, &*bin.right) {
                (Expr::Binary(left_bin), Expr::Binary(right_bin)) => {
                    assert_eq!(left_bin.op, BinaryOp::Lt);
                    assert_eq!(right_bin.op, BinaryOp::Gt);
                }
                _ => panic!("Expected comparisons on both sides"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_precedence_comparison_over_or() {
    let (program, diagnostics) = parse_source("1 == 2 || 3 != 4;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 == 2) || (3 != 4)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Or);
            match (&*bin.left, &*bin.right) {
                (Expr::Binary(left_bin), Expr::Binary(right_bin)) => {
                    assert_eq!(left_bin.op, BinaryOp::Eq);
                    assert_eq!(right_bin.op, BinaryOp::Ne);
                }
                _ => panic!("Expected equality checks on both sides"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Logical AND over Logical OR ==========

#[test]
fn test_precedence_and_over_or() {
    let (program, diagnostics) = parse_source("true || false && false;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: true || (false && false)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Or);
            match &*bin.right {
                Expr::Binary(right_bin) => assert_eq!(right_bin.op, BinaryOp::And),
                _ => panic!("Expected AND on right"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Equality over Logical AND ==========

#[test]
fn test_precedence_equality_over_and() {
    let (program, diagnostics) = parse_source("1 == 2 && 3 == 4;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 == 2) && (3 == 4)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::And);
            match (&*bin.left, &*bin.right) {
                (Expr::Binary(left_bin), Expr::Binary(right_bin)) => {
                    assert_eq!(left_bin.op, BinaryOp::Eq);
                    assert_eq!(right_bin.op, BinaryOp::Eq);
                }
                _ => panic!("Expected equality checks"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Comparison over Equality ==========

#[test]
fn test_precedence_comparison_over_equality() {
    let (program, diagnostics) = parse_source("1 < 2 == 3 > 4;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 < 2) == (3 > 4)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Eq);
            match (&*bin.left, &*bin.right) {
                (Expr::Binary(left_bin), Expr::Binary(right_bin)) => {
                    assert_eq!(left_bin.op, BinaryOp::Lt);
                    assert_eq!(right_bin.op, BinaryOp::Gt);
                }
                _ => panic!("Expected comparisons"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Addition/Subtraction over Comparison ==========

#[test]
fn test_precedence_add_over_comparison() {
    let (program, diagnostics) = parse_source("1 + 2 < 3 + 4;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 + 2) < (3 + 4)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Lt);
            match (&*bin.left, &*bin.right) {
                (Expr::Binary(left_bin), Expr::Binary(right_bin)) => {
                    assert_eq!(left_bin.op, BinaryOp::Add);
                    assert_eq!(right_bin.op, BinaryOp::Add);
                }
                _ => panic!("Expected additions"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Unary operators have highest precedence ==========

#[test]
fn test_precedence_unary_over_binary() {
    let (program, diagnostics) = parse_source("1 + -2;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: 1 + (-2)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Add);
            match &*bin.right {
                Expr::Unary(unary) => assert_eq!(unary.op, UnaryOp::Negate),
                _ => panic!("Expected unary on right"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_precedence_unary_not_over_and() {
    let (program, diagnostics) = parse_source("!true && false;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (!true) && false
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::And);
            match &*bin.left {
                Expr::Unary(unary) => assert_eq!(unary.op, UnaryOp::Not),
                _ => panic!("Expected unary on left"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Call/Index have highest precedence ==========

#[test]
fn test_precedence_call_over_unary() {
    let (program, diagnostics) = parse_source("-foo();");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: -(foo())
    let expr = get_first_expr(&program);
    match expr {
        Expr::Unary(unary) => {
            assert_eq!(unary.op, UnaryOp::Negate);
            match &*unary.expr {
                Expr::Call(_) => {},
                _ => panic!("Expected call in unary"),
            }
        }
        _ => panic!("Expected unary expression"),
    }
}

#[test]
fn test_precedence_index_over_unary() {
    let (program, diagnostics) = parse_source("-arr[0];");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: -(arr[0])
    let expr = get_first_expr(&program);
    match expr {
        Expr::Unary(unary) => {
            assert_eq!(unary.op, UnaryOp::Negate);
            match &*unary.expr {
                Expr::Index(_) => {},
                _ => panic!("Expected index in unary"),
            }
        }
        _ => panic!("Expected unary expression"),
    }
}

// ========== Associativity Tests (All Binary Ops are Left-Associative) ==========

#[test]
fn test_associativity_addition_left() {
    let (program, diagnostics) = parse_source("1 + 2 + 3;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 + 2) + 3 (left-associative)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Add);
            match &*bin.left {
                Expr::Binary(left_bin) => assert_eq!(left_bin.op, BinaryOp::Add),
                _ => panic!("Expected addition on left"),
            }
            match &*bin.right {
                Expr::Literal(Literal::Number(n), _) => assert_eq!(*n, 3.0),
                _ => panic!("Expected number on right"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_associativity_subtraction_left() {
    let (program, diagnostics) = parse_source("10 - 5 - 2;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (10 - 5) - 2 (left-associative)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Sub);
            match &*bin.left {
                Expr::Binary(left_bin) => assert_eq!(left_bin.op, BinaryOp::Sub),
                _ => panic!("Expected subtraction on left"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_associativity_multiplication_left() {
    let (program, diagnostics) = parse_source("2 * 3 * 4;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (2 * 3) * 4 (left-associative)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Mul);
            match &*bin.left {
                Expr::Binary(left_bin) => assert_eq!(left_bin.op, BinaryOp::Mul),
                _ => panic!("Expected multiplication on left"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_associativity_and_left() {
    let (program, diagnostics) = parse_source("true && false && true;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (true && false) && true (left-associative)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::And);
            match &*bin.left {
                Expr::Binary(left_bin) => assert_eq!(left_bin.op, BinaryOp::And),
                _ => panic!("Expected AND on left"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_associativity_or_left() {
    let (program, diagnostics) = parse_source("true || false || true;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (true || false) || true (left-associative)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Or);
            match &*bin.left {
                Expr::Binary(left_bin) => assert_eq!(left_bin.op, BinaryOp::Or),
                _ => panic!("Expected OR on left"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Complex Multi-Level Precedence ==========

#[test]
fn test_precedence_complex_expression() {
    let (program, diagnostics) = parse_source("1 + 2 * 3 == 7 && 10 - 5 > 3 || false;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: ((1 + (2 * 3)) == 7 && (10 - 5) > 3) || false
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            // Top level should be OR
            assert_eq!(bin.op, BinaryOp::Or);
        }
        _ => panic!("Expected binary expression"),
    }
}

// ========== Parentheses Override Precedence ==========

#[test]
fn test_precedence_parentheses_override() {
    let (program, diagnostics) = parse_source("(1 + 2) * 3;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 + 2) * 3, not 1 + (2 * 3)
    let expr = get_first_expr(&program);
    match expr {
        Expr::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Mul);
            match &*bin.left {
                Expr::Group(group) => {
                    match &*group.expr {
                        Expr::Binary(inner) => assert_eq!(inner.op, BinaryOp::Add),
                        _ => panic!("Expected addition in group"),
                    }
                }
                _ => panic!("Expected group on left"),
            }
        }
        _ => panic!("Expected binary expression"),
    }
}
