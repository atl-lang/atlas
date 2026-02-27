use super::*;

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_override_builtin_name(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    // Register a native with the same name as a builtin
    runtime.register_function("len", 1, |args| {
        // Custom len that always returns 42
        let _ = args;
        Ok(Value::Number(42.0))
    });

    // Native should take precedence over builtin
    let result = runtime.eval("len([1, 2, 3])").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_zero_arity(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("getFortyTwo", 0, |_args| Ok(Value::Number(42.0)));

    let result = runtime.eval("getFortyTwo()").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_three_args(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("sum3", 3, |args| {
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
        let c = match &args[2] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        Ok(Value::Number(a + b + c))
    });

    let result = runtime.eval("sum3(10, 20, 30)").unwrap();
    assert_eq!(result, Value::Number(60.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_variadic_with_zero_args(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_variadic("count", |args| Ok(Value::Number(args.len() as f64)));

    let result = runtime.eval("count()").unwrap();
    assert_eq!(result, Value::Number(0.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_variadic_with_many_args(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_variadic("count", |args| Ok(Value::Number(args.len() as f64)));

    let result = runtime
        .eval("count(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)")
        .unwrap();
    assert_eq!(result, Value::Number(10.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_in_expression(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("double", 1, |args| {
        let n = match &args[0] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        Ok(Value::Number(n * 2.0))
    });

    let result = runtime.eval("double(5) + double(10)").unwrap();
    assert_eq!(result, Value::Number(30.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_nested_calls(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("inc", 1, |args| {
        let n = match &args[0] {
            Value::Number(n) => *n,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };
        Ok(Value::Number(n + 1.0))
    });

    let result = runtime.eval("inc(inc(inc(0)))").unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_option_return(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("makeSome", 1, |args| {
        Ok(Value::Option(Some(Box::new(args[0].clone()))))
    });

    let result = runtime.eval("makeSome(42)").unwrap();
    match result {
        Value::Option(Some(val)) => assert_eq!(*val, Value::Number(42.0)),
        _ => panic!("Expected Some value"),
    }
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_result_return(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("makeOk", 1, |args| {
        Ok(Value::Result(Ok(Box::new(args[0].clone()))))
    });

    let result = runtime.eval("makeOk(42)").unwrap();
    match result {
        Value::Result(Ok(val)) => assert_eq!(*val, Value::Number(42.0)),
        _ => panic!("Expected Ok value"),
    }
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_persists_across_evaluations(#[case] mode: ExecutionMode) {
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

    // Call in separate evaluations
    let result1 = runtime.eval("add(1, 2)").unwrap();
    assert_eq!(result1, Value::Number(3.0));

    let result2 = runtime.eval("add(10, 20)").unwrap();
    assert_eq!(result2, Value::Number(30.0));
}

#[rstest]
#[case::interpreter(ExecutionMode::Interpreter)]
#[case::vm(ExecutionMode::VM)]
fn test_native_with_complex_logic(#[case] mode: ExecutionMode) {
    let mut runtime = Runtime::new(mode);

    runtime.register_function("fibonacci", 1, |args| {
        let n = match &args[0] {
            Value::Number(n) => *n as i32,
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "Expected number".to_string(),
                    span: Span::dummy(),
                })
            }
        };

        fn fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }

        Ok(Value::Number(fib(n) as f64))
    });

    let result = runtime.eval("fibonacci(10)").unwrap();
    assert_eq!(result, Value::Number(55.0));
}
