use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::ast::*;

fn parse_source(source: &str) -> (Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ========== Literals ==========

#[test]
fn test_parse_number_literal() {
    let (program, diagnostics) = parse_source("42;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Literal(Literal::Number(n), _) => assert_eq!(*n, 42.0),
                _ => panic!("Expected number literal"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_float_literal() {
    let (program, diagnostics) = parse_source("3.14;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Literal(Literal::Number(n), _) => assert_eq!(*n, 3.14),
                _ => panic!("Expected number literal"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_string_literal() {
    let (program, diagnostics) = parse_source(r#""hello";"#);
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Literal(Literal::String(s), _) => assert_eq!(s, "hello"),
                _ => panic!("Expected string literal"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_bool_true() {
    let (program, diagnostics) = parse_source("true;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Literal(Literal::Bool(b), _) => assert_eq!(*b, true),
                _ => panic!("Expected bool literal"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_bool_false() {
    let (program, diagnostics) = parse_source("false;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Literal(Literal::Bool(b), _) => assert_eq!(*b, false),
                _ => panic!("Expected bool literal"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_null_literal() {
    let (program, diagnostics) = parse_source("null;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Literal(Literal::Null, _) => {},
                _ => panic!("Expected null literal"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Variables ==========

#[test]
fn test_parse_variable_reference() {
    let (program, diagnostics) = parse_source("x;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Identifier(id) => assert_eq!(id.name, "x"),
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Binary Operators ==========

#[test]
fn test_parse_addition() {
    let (program, diagnostics) = parse_source("1 + 2;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => assert_eq!(bin_expr.op, BinaryOp::Add),
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_subtraction() {
    let (program, diagnostics) = parse_source("5 - 3;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => assert_eq!(bin_expr.op, BinaryOp::Sub),
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_multiplication() {
    let (program, diagnostics) = parse_source("3 * 4;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => assert_eq!(bin_expr.op, BinaryOp::Mul),
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_division() {
    let (program, diagnostics) = parse_source("10 / 2;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => assert_eq!(bin_expr.op, BinaryOp::Div),
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_comparison_operators() {
    let tests = vec![
        ("1 < 2;", BinaryOp::Lt),
        ("1 <= 2;", BinaryOp::Le),
        ("1 > 2;", BinaryOp::Gt),
        ("1 >= 2;", BinaryOp::Ge),
        ("1 == 2;", BinaryOp::Eq),
        ("1 != 2;", BinaryOp::Ne),
    ];

    for (source, expected_op) in tests {
        let (program, diagnostics) = parse_source(source);
        assert_eq!(diagnostics.len(), 0, "Failed for: {}", source);

        match &program.items[0] {
            Item::Statement(Stmt::Expr(expr_stmt)) => {
                match &expr_stmt.expr {
                    Expr::Binary(bin_expr) => assert_eq!(bin_expr.op, expected_op, "Failed for: {}", source),
                    _ => panic!("Expected binary expression for: {}", source),
                }
            }
            _ => panic!("Expected expression statement for: {}", source),
        }
    }
}

#[test]
fn test_parse_logical_operators() {
    let (program, diagnostics) = parse_source("true && false;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => assert_eq!(bin_expr.op, BinaryOp::And),
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }

    let (program, diagnostics) = parse_source("true || false;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => assert_eq!(bin_expr.op, BinaryOp::Or),
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Unary Operators ==========

#[test]
fn test_parse_negation() {
    let (program, diagnostics) = parse_source("-5;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Unary(unary_expr) => assert_eq!(unary_expr.op, UnaryOp::Negate),
                _ => panic!("Expected unary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_logical_not() {
    let (program, diagnostics) = parse_source("!true;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Unary(unary_expr) => assert_eq!(unary_expr.op, UnaryOp::Not),
                _ => panic!("Expected unary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Grouping ==========

#[test]
fn test_parse_grouping() {
    let (program, diagnostics) = parse_source("(1 + 2) * 3;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => {
                    assert_eq!(bin_expr.op, BinaryOp::Mul);
                    match &*bin_expr.left {
                        Expr::Group(group_expr) => {
                            match &*group_expr.expr {
                                Expr::Binary(inner_bin) => assert_eq!(inner_bin.op, BinaryOp::Add),
                                _ => panic!("Expected addition in grouping"),
                            }
                        }
                        _ => panic!("Expected grouping"),
                    }
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Array Literals ==========

#[test]
fn test_parse_empty_array() {
    let (program, diagnostics) = parse_source("[];");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::ArrayLiteral(arr_lit) => assert_eq!(arr_lit.elements.len(), 0),
                _ => panic!("Expected array literal"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_array_with_elements() {
    let (program, diagnostics) = parse_source("[1, 2, 3];");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::ArrayLiteral(arr_lit) => assert_eq!(arr_lit.elements.len(), 3),
                _ => panic!("Expected array literal"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Array Indexing ==========

#[test]
fn test_parse_array_index() {
    let (program, diagnostics) = parse_source("arr[0];");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Index(index_expr) => {
                    match &*index_expr.target {
                        Expr::Identifier(id) => assert_eq!(id.name, "arr"),
                        _ => panic!("Expected identifier in index target"),
                    }
                }
                _ => panic!("Expected index expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Function Calls ==========

#[test]
fn test_parse_function_call_no_args() {
    let (program, diagnostics) = parse_source("foo();");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Call(call_expr) => {
                    match &*call_expr.callee {
                        Expr::Identifier(id) => assert_eq!(id.name, "foo"),
                        _ => panic!("Expected identifier in callee"),
                    }
                    assert_eq!(call_expr.args.len(), 0);
                }
                _ => panic!("Expected call expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_function_call_with_args() {
    let (program, diagnostics) = parse_source("foo(1, 2, 3);");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Call(call_expr) => {
                    match &*call_expr.callee {
                        Expr::Identifier(id) => assert_eq!(id.name, "foo"),
                        _ => panic!("Expected identifier in callee"),
                    }
                    assert_eq!(call_expr.args.len(), 3);
                }
                _ => panic!("Expected call expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Variable Declarations ==========

#[test]
fn test_parse_let_declaration() {
    let (program, diagnostics) = parse_source("let x = 42;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::VarDecl(var_decl)) => {
            assert_eq!(var_decl.mutable, false);
            assert_eq!(var_decl.name.name, "x");
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_var_declaration() {
    let (program, diagnostics) = parse_source("var x = 42;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::VarDecl(var_decl)) => {
            assert_eq!(var_decl.mutable, true);
            assert_eq!(var_decl.name.name, "x");
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_var_declaration_with_type() {
    let (program, diagnostics) = parse_source("let x: number = 42;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::VarDecl(var_decl)) => {
            assert_eq!(var_decl.name.name, "x");
            match &var_decl.type_ref {
                Some(TypeRef::Named(name, _)) => assert_eq!(name, "number"),
                _ => panic!("Expected named type reference"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

// ========== Assignment ==========

#[test]
fn test_parse_simple_assignment() {
    let (program, diagnostics) = parse_source("x = 42;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Name(id) => assert_eq!(id.name, "x"),
                _ => panic!("Expected name target"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_array_element_assignment() {
    let (program, diagnostics) = parse_source("arr[0] = 42;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Index { target, .. } => {
                    match &**target {
                        Expr::Identifier(id) => assert_eq!(id.name, "arr"),
                        _ => panic!("Expected identifier in index target"),
                    }
                }
                _ => panic!("Expected index target"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

// ========== If Statements ==========

#[test]
fn test_parse_if_statement() {
    let (program, diagnostics) = parse_source("if (true) { x; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::If(if_stmt)) => {
            assert_eq!(if_stmt.else_block.is_none(), true);
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_if_else_statement() {
    let (program, diagnostics) = parse_source("if (true) { x; } else { y; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::If(if_stmt)) => {
            assert_eq!(if_stmt.else_block.is_some(), true);
        }
        _ => panic!("Expected if statement"),
    }
}

// ========== While Loops ==========

#[test]
fn test_parse_while_loop() {
    let (program, diagnostics) = parse_source("while (true) { x; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::While(while_stmt)) => {
            match &while_stmt.cond {
                Expr::Literal(Literal::Bool(b), _) => assert_eq!(*b, true),
                _ => panic!("Expected bool literal in condition"),
            }
        }
        _ => panic!("Expected while statement"),
    }
}

// ========== For Loops ==========

#[test]
fn test_parse_for_loop() {
    let (program, diagnostics) = parse_source("for (let i = 0; i < 10; i = i + 1) { x; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::For(for_stmt)) => {
            // Just verify it parsed correctly
            match &*for_stmt.init {
                Stmt::VarDecl(_) => {},
                _ => panic!("Expected var decl in for init"),
            }
        }
        _ => panic!("Expected for statement"),
    }
}

// ========== Return Statements ==========

#[test]
fn test_parse_return_statement() {
    let (program, diagnostics) = parse_source("return 42;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Return(ret_stmt)) => {
            assert_eq!(ret_stmt.value.is_some(), true);
        }
        _ => panic!("Expected return statement"),
    }
}

#[test]
fn test_parse_return_no_value() {
    let (program, diagnostics) = parse_source("return;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Return(ret_stmt)) => {
            assert_eq!(ret_stmt.value.is_none(), true);
        }
        _ => panic!("Expected return statement"),
    }
}

// ========== Break/Continue ==========

#[test]
fn test_parse_break_statement() {
    let (program, diagnostics) = parse_source("break;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Break(_)) => {},
        _ => panic!("Expected break statement"),
    }
}

#[test]
fn test_parse_continue_statement() {
    let (program, diagnostics) = parse_source("continue;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Continue(_)) => {},
        _ => panic!("Expected continue statement"),
    }
}

// ========== Block Statements ==========

#[test]
fn test_parse_block_in_if() {
    let (program, diagnostics) = parse_source("if (true) { let x = 1; let y = 2; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::If(if_stmt)) => {
            assert_eq!(if_stmt.then_block.statements.len(), 2);
        }
        _ => panic!("Expected if statement"),
    }
}

// ========== Function Declarations ==========

#[test]
fn test_parse_function_no_params() {
    let (program, diagnostics) = parse_source("fn foo() { return 42; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Function(func) => {
            assert_eq!(func.name.name, "foo");
            assert_eq!(func.params.len(), 0);
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_parse_function_with_params() {
    let (program, diagnostics) = parse_source("fn add(x: number, y: number) -> number { return x + y; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Function(func) => {
            assert_eq!(func.name.name, "add");
            assert_eq!(func.params.len(), 2);
            assert_eq!(func.params[0].name.name, "x");
            assert_eq!(func.params[1].name.name, "y");
        }
        _ => panic!("Expected function declaration"),
    }
}

// ========== Operator Precedence ==========

#[test]
fn test_operator_precedence_multiplication_over_addition() {
    let (program, diagnostics) = parse_source("1 + 2 * 3;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: 1 + (2 * 3), not (1 + 2) * 3
    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => {
                    assert_eq!(bin_expr.op, BinaryOp::Add);
                    // Right side should be 2 * 3
                    match &*bin_expr.right {
                        Expr::Binary(inner_bin) => assert_eq!(inner_bin.op, BinaryOp::Mul),
                        _ => panic!("Expected multiplication on right"),
                    }
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_operator_precedence_comparison_over_logical() {
    let (program, diagnostics) = parse_source("1 < 2 && 3 > 4;");
    assert_eq!(diagnostics.len(), 0);

    // Should parse as: (1 < 2) && (3 > 4)
    match &program.items[0] {
        Item::Statement(Stmt::Expr(expr_stmt)) => {
            match &expr_stmt.expr {
                Expr::Binary(bin_expr) => {
                    assert_eq!(bin_expr.op, BinaryOp::And);
                    // Both sides should be comparisons
                    match &*bin_expr.left {
                        Expr::Binary(left_bin) => assert_eq!(left_bin.op, BinaryOp::Lt),
                        _ => panic!("Expected comparison on left"),
                    }
                    match &*bin_expr.right {
                        Expr::Binary(right_bin) => assert_eq!(right_bin.op, BinaryOp::Gt),
                        _ => panic!("Expected comparison on right"),
                    }
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

// ========== Complex Programs ==========

#[test]
fn test_parse_multiple_statements() {
    let (program, diagnostics) = parse_source("let x = 1; let y = 2; let z = x + y;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_parse_nested_blocks() {
    let (program, diagnostics) = parse_source("if (true) { if (false) { let x = 1; } }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::If(if_stmt)) => {
            assert_eq!(if_stmt.then_block.statements.len(), 1);
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_function_with_complex_body() {
    let source = r#"
fn factorial(n: number) -> number {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}
    "#;

    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Function(func) => {
            assert_eq!(func.name.name, "factorial");
        }
        _ => panic!("Expected function declaration"),
    }
}

// ========== Error Recovery ==========

#[test]
fn test_parse_error_recovery() {
    let (program, diagnostics) = parse_source("let x = ; let y = 2;");
    assert!(!diagnostics.is_empty(), "Expected syntax error");
    // Parser should recover and parse the second statement
    assert!(program.items.len() >= 1, "Expected at least one item after recovery");
}

#[test]
fn test_parse_missing_semicolon_error() {
    let (_program, diagnostics) = parse_source("let x = 1 let y = 2;");
    assert!(!diagnostics.is_empty(), "Expected syntax error for missing semicolon");
}
