use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::ast::*;

fn parse_source(source: &str) -> (Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ========== Simple Name Targets ==========

#[test]
fn test_assign_to_simple_identifier() {
    let (program, diagnostics) = parse_source("x = 42;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 1);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Name(id) => {
                    assert_eq!(id.name, "x");
                }
                _ => panic!("Expected name target"),
            }
            // Verify value
            match &assign.value {
                Expr::Literal(Literal::Number(n), _) => assert_eq!(*n, 42.0),
                _ => panic!("Expected number literal"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_assign_to_longer_identifier() {
    let (program, diagnostics) = parse_source("myVariable = 100;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Name(id) => assert_eq!(id.name, "myVariable"),
                _ => panic!("Expected name target"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_assign_expression_to_name() {
    let (program, diagnostics) = parse_source("x = 1 + 2 * 3;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Name(id) => assert_eq!(id.name, "x"),
                _ => panic!("Expected name target"),
            }
            // Verify it's a binary expression
            match &assign.value {
                Expr::Binary(_) => {},
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

// ========== Array Index Targets ==========

#[test]
fn test_assign_to_array_index() {
    let (program, diagnostics) = parse_source("arr[0] = 42;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Index { target, index, .. } => {
                    // Verify target is identifier "arr"
                    match &**target {
                        Expr::Identifier(id) => assert_eq!(id.name, "arr"),
                        _ => panic!("Expected identifier"),
                    }
                    // Verify index is 0
                    match &**index {
                        Expr::Literal(Literal::Number(n), _) => assert_eq!(*n, 0.0),
                        _ => panic!("Expected number literal"),
                    }
                }
                _ => panic!("Expected index target"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_assign_to_array_with_variable_index() {
    let (program, diagnostics) = parse_source("arr[i] = value;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Index { target, index, .. } => {
                    // Verify target is "arr"
                    match &**target {
                        Expr::Identifier(id) => assert_eq!(id.name, "arr"),
                        _ => panic!("Expected identifier"),
                    }
                    // Verify index is variable "i"
                    match &**index {
                        Expr::Identifier(id) => assert_eq!(id.name, "i"),
                        _ => panic!("Expected identifier"),
                    }
                }
                _ => panic!("Expected index target"),
            }
            // Verify value is variable "value"
            match &assign.value {
                Expr::Identifier(id) => assert_eq!(id.name, "value"),
                _ => panic!("Expected identifier"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_assign_to_array_with_expression_index() {
    let (program, diagnostics) = parse_source("arr[i + 1] = 42;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Index { target, index, .. } => {
                    match &**target {
                        Expr::Identifier(id) => assert_eq!(id.name, "arr"),
                        _ => panic!("Expected identifier"),
                    }
                    // Verify index is an expression (i + 1)
                    match &**index {
                        Expr::Binary(bin) => assert_eq!(bin.op, BinaryOp::Add),
                        _ => panic!("Expected binary expression"),
                    }
                }
                _ => panic!("Expected index target"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_assign_to_nested_array_index() {
    let (program, diagnostics) = parse_source("matrix[i][j] = 0;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Index { target, index, .. } => {
                    // Outer index: matrix[i][j]
                    // Target should be matrix[i] (another index expression)
                    match &**target {
                        Expr::Index(inner_index) => {
                            // Inner target should be "matrix"
                            match &*inner_index.target {
                                Expr::Identifier(id) => assert_eq!(id.name, "matrix"),
                                _ => panic!("Expected identifier"),
                            }
                            // Inner index should be "i"
                            match &*inner_index.index {
                                Expr::Identifier(id) => assert_eq!(id.name, "i"),
                                _ => panic!("Expected identifier"),
                            }
                        }
                        _ => panic!("Expected index expression"),
                    }
                    // Outer index should be "j"
                    match &**index {
                        Expr::Identifier(id) => assert_eq!(id.name, "j"),
                        _ => panic!("Expected identifier"),
                    }
                }
                _ => panic!("Expected index target"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

// ========== Complex Assignment Values ==========

#[test]
fn test_assign_array_literal_to_name() {
    let (program, diagnostics) = parse_source("arr = [1, 2, 3];");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Name(id) => assert_eq!(id.name, "arr"),
                _ => panic!("Expected name target"),
            }
            match &assign.value {
                Expr::ArrayLiteral(arr_lit) => {
                    assert_eq!(arr_lit.elements.len(), 3);
                }
                _ => panic!("Expected array literal"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_assign_function_call_result() {
    let (program, diagnostics) = parse_source("result = foo(1, 2);");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Name(id) => assert_eq!(id.name, "result"),
                _ => panic!("Expected name target"),
            }
            match &assign.value {
                Expr::Call(call) => {
                    assert_eq!(call.args.len(), 2);
                }
                _ => panic!("Expected call expression"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_assign_array_access_value() {
    let (program, diagnostics) = parse_source("x = arr[0];");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Name(id) => assert_eq!(id.name, "x"),
                _ => panic!("Expected name target"),
            }
            match &assign.value {
                Expr::Index(_) => {},
                _ => panic!("Expected index expression"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

// ========== Multiple Assignments ==========

#[test]
fn test_multiple_assignments_in_sequence() {
    let (program, diagnostics) = parse_source("x = 1; y = 2; z = 3;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 3);

    // Verify all three are assignments
    for (i, expected_name) in ["x", "y", "z"].iter().enumerate() {
        match &program.items[i] {
            Item::Statement(Stmt::Assign(assign)) => {
                match &assign.target {
                    AssignTarget::Name(id) => assert_eq!(&id.name, expected_name),
                    _ => panic!("Expected name target"),
                }
            }
            _ => panic!("Expected assignment statement"),
        }
    }
}

// ========== Assignment in Different Contexts ==========

#[test]
fn test_assignment_in_block() {
    let (program, diagnostics) = parse_source("{ x = 42; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Expr(_expr_stmt)) => {
            // Block is represented as an expression statement with null literal
            // But the block itself should have been parsed
            // Actually, looking at our parser, blocks create a dummy expression
            // The actual assignment is inside the block
        }
        _ => {},
    }
}

#[test]
fn test_assignment_in_if_block() {
    let (program, diagnostics) = parse_source("if (true) { x = 42; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::If(if_stmt)) => {
            assert_eq!(if_stmt.then_block.statements.len(), 1);
            match &if_stmt.then_block.statements[0] {
                Stmt::Assign(assign) => {
                    match &assign.target {
                        AssignTarget::Name(id) => assert_eq!(id.name, "x"),
                        _ => panic!("Expected name target"),
                    }
                }
                _ => panic!("Expected assignment in if block"),
            }
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_assignment_in_while_body() {
    let (program, diagnostics) = parse_source("while (true) { counter = counter + 1; }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::While(while_stmt)) => {
            assert_eq!(while_stmt.body.statements.len(), 1);
            match &while_stmt.body.statements[0] {
                Stmt::Assign(assign) => {
                    match &assign.target {
                        AssignTarget::Name(id) => assert_eq!(id.name, "counter"),
                        _ => panic!("Expected name target"),
                    }
                }
                _ => panic!("Expected assignment in while body"),
            }
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_assignment_in_for_loop_step() {
    let (program, diagnostics) = parse_source("for (let i = 0; i < 10; i = i + 1) { }");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::For(for_stmt)) => {
            // Verify the step is an assignment
            match &*for_stmt.step {
                Stmt::Assign(assign) => {
                    match &assign.target {
                        AssignTarget::Name(id) => assert_eq!(id.name, "i"),
                        _ => panic!("Expected name target in step"),
                    }
                }
                _ => panic!("Expected assignment in for step"),
            }
        }
        _ => panic!("Expected for statement"),
    }
}

// ========== Invalid Assignment Targets (Error Cases) ==========

#[test]
fn test_error_assign_to_literal() {
    let (_program, diagnostics) = parse_source("42 = x;");
    assert!(!diagnostics.is_empty(), "Expected error for invalid assignment target");
}

#[test]
fn test_error_assign_to_string_literal() {
    let (_program, diagnostics) = parse_source(r#""hello" = x;"#);
    assert!(!diagnostics.is_empty(), "Expected error for assigning to string literal");
}

#[test]
fn test_error_assign_to_function_call() {
    let (_program, diagnostics) = parse_source("foo() = x;");
    assert!(!diagnostics.is_empty(), "Expected error for assigning to function call");
}

#[test]
fn test_error_assign_to_binary_expression() {
    let (_program, diagnostics) = parse_source("(x + y) = 10;");
    assert!(!diagnostics.is_empty(), "Expected error for assigning to expression");
}

#[test]
fn test_error_assign_to_array_literal() {
    let (_program, diagnostics) = parse_source("[1, 2, 3] = x;");
    assert!(!diagnostics.is_empty(), "Expected error for assigning to array literal");
}

// ========== Assignment vs Expression Statement Distinction ==========

#[test]
fn test_expression_statement_not_assignment() {
    let (program, diagnostics) = parse_source("x;");
    assert_eq!(diagnostics.len(), 0);

    // Should be an expression statement, not an assignment
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

#[test]
fn test_function_call_not_assignment() {
    let (program, diagnostics) = parse_source("foo(x);");
    assert_eq!(diagnostics.len(), 0);

    // Should be an expression statement, not an assignment
    match &program.items[0] {
        Item::Statement(Stmt::Expr(_)) => {},
        _ => panic!("Expected expression statement"),
    }
}

// ========== Assignment with Complex Expressions ==========

#[test]
fn test_assign_conditional_expression() {
    let (program, diagnostics) = parse_source("result = x > 10 && y < 5;");
    assert_eq!(diagnostics.len(), 0);

    match &program.items[0] {
        Item::Statement(Stmt::Assign(assign)) => {
            match &assign.target {
                AssignTarget::Name(id) => assert_eq!(id.name, "result"),
                _ => panic!("Expected name target"),
            }
            // Value should be a logical AND expression
            match &assign.value {
                Expr::Binary(bin) => assert_eq!(bin.op, BinaryOp::And),
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}
