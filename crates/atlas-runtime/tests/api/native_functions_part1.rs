use super::*;

    // Register a simple add function
    runtime.register_function("add", 2, |args| {
        let a = match &args[0] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        let b = match &args[1] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        Ok(Value::Number(a + b))
    });

    // Call the native function
    let result = runtime.eval("add(10, 20)").unwrap();
    assert_eq!(result, Value::Number(30.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_register_variadic_native(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    // Register a variadic sum function
    runtime.register_variadic("sum", |args| {
        let mut total = 0.0;
        for arg in args {
            match arg {
                Value::Number(n) => total += n,
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "All arguments must be numbers".to_string(),
                        span: Span::dummy(),
                    })
                }
            }
        }
        Ok(Value::Number(total))
    });

    // Call with different argument counts
    let result = runtime.eval("sum()").unwrap();
    assert_eq!(result, Value::Number(0.0));

    let result = runtime.eval("sum(42)").unwrap();
    assert_eq!(result, Value::Number(42.0));

    let result = runtime.eval("sum(1, 2, 3, 4, 5)").unwrap();
    assert_eq!(result, Value::Number(15.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_arity_validation_too_few(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("add", 2, |args| {
        let a = match &args[0] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        let b = match &args[1] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        Ok(Value::Number(a + b))
    });

    // Call with too few arguments
    let result = runtime.eval("add(10)");
    assert!(result.is_err());
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_arity_validation_too_many(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("add", 2, |args| {
        let a = match &args[0] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        let b = match &args[1] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        Ok(Value::Number(a + b))
    });

    // Call with too many arguments
    let result = runtime.eval("add(10, 20, 30)");
    assert!(result.is_err());
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_returning_error(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("alwaysFails", 0, |_args| {
        Err(RuntimeError::TypeError {
            msg: "This function always fails".to_string(),
            span: Span::dummy(),
        })
    });

    let result = runtime.eval("alwaysFails()");
    assert!(result.is_err());
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_string_args(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("greet", 1, |args| match &args[0] {
        Value::String(s) => Ok(Value::string(format!("Hello, {}!", s))),
        _ => Err(RuntimeError::TypeError {
            msg: "Expected string".to_string(),
            span: Span::dummy(),
        }),
    });

    let result = runtime.eval(r#"greet("World")"#).unwrap();
    match result {
        Value::String(s) => assert_eq!(s.as_ref(), "Hello, World!"),
        _ => panic!("Expected string result"),
    }
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_bool_args(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("negate", 1, |args| match &args[0] {
        Value::Bool(b) => Ok(Value::Bool(!b)),
        _ => Err(RuntimeError::TypeError {
            msg: "Expected bool".to_string(),
            span: Span::dummy(),
        }),
    });

    let result = runtime.eval("negate(true)").unwrap();
    assert_eq!(result, Value::Bool(false));

    let result = runtime.eval("negate(false)").unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_array_args(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("arrayLength", 1, |args| match &args[0] {
        Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
        _ => Err(RuntimeError::TypeError {
            msg: "Expected array".to_string(),
            span: Span::dummy(),
        }),
    });

    let result = runtime.eval("arrayLength([1, 2, 3, 4, 5])").unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_returning_null(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("returnNull", 0, |_args| Ok(Value::Null));

    let result = runtime.eval("returnNull()").unwrap();
    assert_eq!(result, Value::Null);
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_returning_array(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("makeRange", 1, |args| {
        let n = match &args[0] {
            Value::Number(n) => *n as usize,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        let arr: Vec<Value> = (0..n).map(|i| Value::Number(i as f64)).collect();
        Ok(Value::array(arr))
    });

    let result = runtime.eval("makeRange(5)").unwrap();
    match result {
        Value::Array(arr) => {
            let borrowed = arr.as_slice();
            assert_eq!(borrowed.len(), 5);
            assert_eq!(borrowed[0], Value::Number(0.0));
            assert_eq!(borrowed[4], Value::Number(4.0));
        }
        _ => panic!("Expected array result"),
    }
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_called_from_atlas_function(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("multiply", 2, |args| {
        let a = match &args[0] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        let b = match &args[1] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        Ok(Value::Number(a * b))
    });

    runtime
        .eval("fn square(x: number) -> number { return multiply(x, x); }")
        .unwrap();
    let result = runtime.eval("square(5)").unwrap();
    assert_eq!(result, Value::Number(25.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_closure_capture(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    let multiplier = 10.0;
    runtime.register_function("scale", 1, move |args| {
        let n = match &args[0] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
