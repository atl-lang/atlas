use atlas_runtime::repl::ReplCore;

#[test]
fn repl_captures_stdout_per_eval() {
    let mut repl = ReplCore::new();

    let first = repl.eval_line("console.log(\"hello\");");
    assert!(first.diagnostics.is_empty());
    assert_eq!(first.stdout, "hello\n");

    let second = repl.eval_line("console.log(\"world\");");
    assert!(second.diagnostics.is_empty());
    assert_eq!(second.stdout, "world\n");
}

#[test]
fn repl_captures_stdout_for_multiple_prints() {
    let mut repl = ReplCore::new();
    let result = repl.eval_line("console.log(\"a\"); console.log(\"b\");");
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.stdout, "a\nb\n");
}
