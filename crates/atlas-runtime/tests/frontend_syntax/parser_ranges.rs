use super::*;

#[test]
fn test_parse_range_expressions() {
    let source = "1..3; 1..=3; ..3; 1..;";
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no parser errors");

    let mut ranges = Vec::new();
    for item in program.items {
        if let Item::Statement(Stmt::Expr(expr_stmt)) = item {
            if let Expr::Range {
                start,
                end,
                inclusive,
                ..
            } = expr_stmt.expr
            {
                ranges.push((start, end, inclusive));
            }
        }
    }

    assert_eq!(ranges.len(), 4, "Expected four range expressions");

    // 1..3
    match (&ranges[0].0, &ranges[0].1, ranges[0].2) {
        (Some(start), Some(end), false) => {
            assert!(matches!(
                start.as_ref(),
                Expr::Literal(Literal::Number(1.0), _)
            ));
            assert!(matches!(
                end.as_ref(),
                Expr::Literal(Literal::Number(3.0), _)
            ));
        }
        other => panic!("Unexpected range 0: {:?}", other),
    }

    // 1..=3
    match (&ranges[1].0, &ranges[1].1, ranges[1].2) {
        (Some(start), Some(end), true) => {
            assert!(matches!(
                start.as_ref(),
                Expr::Literal(Literal::Number(1.0), _)
            ));
            assert!(matches!(
                end.as_ref(),
                Expr::Literal(Literal::Number(3.0), _)
            ));
        }
        other => panic!("Unexpected range 1: {:?}", other),
    }

    // ..3
    match (&ranges[2].0, &ranges[2].1, ranges[2].2) {
        (None, Some(end), false) => {
            assert!(matches!(
                end.as_ref(),
                Expr::Literal(Literal::Number(3.0), _)
            ));
        }
        other => panic!("Unexpected range 2: {:?}", other),
    }

    // 1..
    match (&ranges[3].0, &ranges[3].1, ranges[3].2) {
        (Some(start), None, false) => {
            assert!(matches!(
                start.as_ref(),
                Expr::Literal(Literal::Number(1.0), _)
            ));
        }
        other => panic!("Unexpected range 3: {:?}", other),
    }
}
