# Atlas Testing Guide

## Quick Start

```bash
# Run all tests
cargo test -p atlas-runtime

# Run a specific test file
cargo test -p atlas-runtime --test vm_integration_tests

# Run a specific test
cargo test -p atlas-runtime test_name -- --exact

# Run with output
cargo test -p atlas-runtime test_name -- --exact --nocapture
```

## Test Organization

### VM Tests

| File | Tests | Description |
|------|-------|-------------|
| `vm_integration_tests.rs` | 86 | Cross-feature integration (optimizer + debugger + profiler) |
| `vm_complex_programs.rs` | 83 | Real-world algorithms and complex programs |
| `vm_regression_tests.rs` | 80 | Parity checks, edge cases, performance regression |
| `vm_performance_tests.rs` | 48 | Performance optimization correctness |
| `optimizer_integration_tests.rs` | ~30 | Optimizer semantic preservation |
| `optimizer_tests.rs` | ~40 | Optimizer pass unit tests |
| `profiler_tests.rs` | ~30 | Profiler pipeline tests |
| `bytecode_validator_tests.rs` | ~20 | Bytecode validation |
| `bytecode_compiler_integration.rs` | ~20 | Compiler integration |

### Interpreter Tests

| File | Tests | Description |
|------|-------|-------------|
| `interpreter_integration_tests.rs` | — | Interpreter integration harness |
| `stdlib_integration_tests.rs` | ~50 | Standard library tests |
| `stdlib_parity_verification.rs` | ~40 | Interpreter/VM stdlib parity |

### Language Feature Tests

| File | Description |
|------|-------------|
| `first_class_functions_tests.rs` | Function values, closures |
| `pattern_matching_tests.rs` | Match expressions |
| `option_result_tests.rs` | Option/Result types |
| `generics_runtime_tests.rs` | Generic type system |
| `test_for_in_*.rs` | For-in loop tests |
| `collection_iteration_tests.rs` | Collection iteration |

### Infrastructure Tests

| File | Description |
|------|-------------|
| `lexer_tests.rs` | Lexer tokenization |
| `parser_tests.rs` | Parser AST generation |
| `security_tests.rs` | Security permissions |
| `module_*.rs` | Module system tests |

## Testing Patterns

### VM Test Helper

```rust
fn vm_eval(source: &str) -> Option<Value> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all()).expect("VM failed")
}
```

### Parity Testing

```rust
fn assert_parity(source: &str) {
    let vm_result = vm_eval(source);
    let interp_result = interp_eval(source);
    assert_eq!(vm_result.unwrap_or(Value::Null), interp_result);
}
```

### Parameterized Tests (rstest)

```rust
#[rstest]
#[case("1 + 2;", 3.0)]
#[case("4 * 5;", 20.0)]
fn test_arithmetic(#[case] source: &str, #[case] expected: f64) {
    assert_eq!(vm_number(source), expected);
}
```

## Benchmarks

```bash
# Run all benchmarks
cargo bench -p atlas-runtime

# Run specific benchmark
cargo bench -p atlas-runtime -- vm_performance
```

Benchmarks use Criterion for statistical analysis. See `benches/vm_performance_benches.rs`.
