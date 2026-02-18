//! Typechecker speed benchmarks
//!
//! Benchmarks the full frontend pipeline (lex + parse + bind + typecheck)
//! on representative programs of varying complexity.

use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn typecheck(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut binder = Binder::new();
    let (mut sym_table, _) = binder.bind(&program);
    let mut tc = TypeChecker::new(&mut sym_table);
    let _ = tc.check(&program);
}

fn generate_function_heavy(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!(
            "fn compute{}(a: number, b: number) -> number {{ return a * b + {}; }}\n",
            i, i
        ));
    }
    // Call them all
    for i in 0..n {
        s.push_str(&format!("let r{} = compute{}({}, {});\n", i, i, i, i + 1));
    }
    s
}

fn generate_scope_heavy(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!("let x{}: number = {};\n", i, i));
    }
    // Reference all variables
    s.push_str("let total: number = ");
    for i in 0..n {
        if i > 0 {
            s.push_str(" + ");
        }
        s.push_str(&format!("x{}", i));
    }
    s.push_str(";\n");
    s
}

fn generate_control_flow(n: usize) -> String {
    let mut s = String::new();
    s.push_str("fn process(x: number) -> number {\n");
    for i in 0..n {
        s.push_str(&format!("  if (x > {}) {{ return x + {}; }}\n", i * 10, i));
    }
    s.push_str("  return 0;\n}\n");
    for i in 0..n {
        s.push_str(&format!("let r{} = process({});\n", i, i * 5));
    }
    s
}

fn bench_typecheck_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("typecheck_functions");
    for count in [10, 50, 200] {
        let source = generate_function_heavy(count);
        group.bench_with_input(BenchmarkId::from_parameter(count), &source, |b, src| {
            b.iter(|| typecheck(black_box(src)));
        });
    }
    group.finish();
}

fn bench_typecheck_scopes(c: &mut Criterion) {
    let mut group = c.benchmark_group("typecheck_scopes");
    for count in [50, 200, 500] {
        let source = generate_scope_heavy(count);
        group.bench_with_input(BenchmarkId::from_parameter(count), &source, |b, src| {
            b.iter(|| typecheck(black_box(src)));
        });
    }
    group.finish();
}

fn bench_typecheck_control_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("typecheck_control_flow");
    for count in [10, 50, 100] {
        let source = generate_control_flow(count);
        group.bench_with_input(BenchmarkId::from_parameter(count), &source, |b, src| {
            b.iter(|| typecheck(black_box(src)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_typecheck_functions,
    bench_typecheck_scopes,
    bench_typecheck_control_flow
);
criterion_main!(benches);
