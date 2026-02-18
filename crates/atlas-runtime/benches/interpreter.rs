//! Interpreter execution benchmarks
//!
//! Benchmarks the tree-walking interpreter on canonical programs
//! that stress different execution paths.

use atlas_runtime::{Interpreter, Lexer, Parser, SecurityContext};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn interp_run(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let security = SecurityContext::allow_all();
    let mut interp = Interpreter::new();
    let _ = interp.eval(&program, &security);
}

fn bench_interp_arithmetic_loop(c: &mut Criterion) {
    c.bench_function("interp_arithmetic_loop_10k", |b| {
        let code = "let sum = 0; let i = 0; while (i < 10000) { sum = sum + i; i = i + 1; } sum;";
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_fibonacci(c: &mut Criterion) {
    c.bench_function("interp_fibonacci_20", |b| {
        let code = "fn fib(n: number) -> number { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } fib(20);";
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_string_concat(c: &mut Criterion) {
    c.bench_function("interp_string_concat_500", |b| {
        let code = r#"let s = ""; let i = 0; while (i < 500) { s = s + "x"; i = i + 1; } len(s);"#;
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_collection_ops(c: &mut Criterion) {
    c.bench_function("interp_array_push_pop_1k", |b| {
        let code = r#"
            let arr: number[] = [];
            let i = 0;
            while (i < 1000) {
                arr = push(arr, i);
                i = i + 1;
            }
            len(arr);
        "#;
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_function_calls(c: &mut Criterion) {
    c.bench_function("interp_function_calls_10k", |b| {
        let code = "fn inc(x: number) -> number { return x + 1; } let r = 0; let i = 0; while (i < 10000) { r = inc(r); i = i + 1; } r;";
        b.iter(|| interp_run(black_box(code)));
    });
}

fn bench_interp_nested_loops(c: &mut Criterion) {
    c.bench_function("interp_nested_loops_100x100", |b| {
        let code = "let count = 0; let i = 0; while (i < 100) { let j = 0; while (j < 100) { count = count + 1; j = j + 1; } i = i + 1; } count;";
        b.iter(|| interp_run(black_box(code)));
    });
}

criterion_group!(
    benches,
    bench_interp_arithmetic_loop,
    bench_interp_fibonacci,
    bench_interp_string_concat,
    bench_interp_collection_ops,
    bench_interp_function_calls,
    bench_interp_nested_loops
);
criterion_main!(benches);
