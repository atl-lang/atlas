// Stdlib function tests split into multiple parts
// Originally functions.rs (~40KB)
// Split to stay under 12KB file size limit per atlas-testing.md

pub(crate) use super::*;

fn span() -> Span {
    Span::dummy()
}

fn bool_val(b: bool) -> Value {
    Value::Bool(b)
}

fn str_val(s: &str) -> Value {
    Value::string(s)
}

fn num_val(n: f64) -> Value {
    Value::Number(n)
}

fn arr_val(items: Vec<Value>) -> Value {
    Value::array(items)
}

fn ok_val(v: Value) -> Value {
    Value::Result(Ok(Box::new(v)))
}

fn some_val(v: Value) -> Value {
    Value::Option(Some(Box::new(v)))
}

fn throwing_fn() -> Value {
    Value::NativeFunction(Arc::new(|_| {
        Err(RuntimeError::TypeError {
            msg: "intentional".to_string(),
            span: Span::dummy(),
        })
    }))
}

fn ok_fn() -> Value {
    Value::NativeFunction(Arc::new(|_| Ok(Value::Null)))
}

/// Evaluate Atlas source and assert it succeeds (returns Null or any value).
fn eval_ok(source: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Expected success, got errors: {:?}", diags),
    }
}

/// Evaluate Atlas source and assert it fails with an error containing `fragment`.
fn eval_err_contains(source: &str, fragment: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Err(diags) => {
            let combined = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                combined.contains(fragment),
                "Error message {:?} did not contain {:?}",
                combined,
                fragment
            );
        }
        Ok(val) => panic!("Expected error, got success: {:?}", val),
    }
}

fn check_file(filename: &str) -> Vec<Diagnostic> {
    let path = Path::new("../../tests/prelude").join(filename);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path.display()));

    // Use unique file path to avoid source cache collisions between parallel tests
    let mut lexer = Lexer::new(&source).with_file(path.to_string_lossy());
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        return lex_diagnostics;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        return parse_diagnostics;
    }

    let mut binder = Binder::new();
    let (_symbol_table, bind_diagnostics) = binder.bind(&program);

    bind_diagnostics
}

mod functions_loops;
mod part1;
mod part2;
mod part3;
mod part4;
mod part5;
