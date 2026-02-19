//! Interpreter vs VM head-to-head parity benchmarks
//!
//! Runs identical programs through both engines in the same benchmark file
//! to make performance divergence immediately visible.

use atlas_runtime::compiler::Compiler;
use atlas_runtime::vm::VM;
use atlas_runtime::{Interpreter, Lexer, Parser, SecurityContext};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn run_interpreter(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let security = SecurityContext::allow_all();
    let mut interp = Interpreter::new();
    let _ = interp.eval(&program, &security);
}

fn run_vm(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).expect("Compilation failed");
    let mut vm = VM::new(bytecode);
    let _ = vm.run(&SecurityContext::allow_all());
}

const PROGRAMS: &[(&str, &str)] = &[
    (
        "arithmetic_loop",
        "var sum = 0; var i = 0; while (i < 10000) { sum = sum + i; i = i + 1; } sum;",
    ),
    (
        "fibonacci_20",
        "fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(20);",
    ),
    (
        "function_calls_5k",
        "fn inc(x: number) -> number { return x + 1; } var r = 0; var i = 0; while (i < 5000) { r = inc(r); i = i + 1; } r;",
    ),
    (
        "nested_loops",
        "var c = 0; var i = 0; while (i < 100) { var j = 0; while (j < 100) { c = c + 1; j = j + 1; } i = i + 1; } c;",
    ),
    (
        "string_concat_200",
        r#"var s = ""; var i = 0; while (i < 200) { s = s + "x"; i = i + 1; } len(s);"#,
    ),
];

fn bench_parity(c: &mut Criterion) {
    let mut group = c.benchmark_group("parity");

    for (name, code) in PROGRAMS {
        group.bench_with_input(BenchmarkId::new("interpreter", name), code, |b, src| {
            b.iter(|| run_interpreter(black_box(src)));
        });
        group.bench_with_input(BenchmarkId::new("vm", name), code, |b, src| {
            b.iter(|| run_vm(black_box(src)));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_parity);
criterion_main!(benches);
