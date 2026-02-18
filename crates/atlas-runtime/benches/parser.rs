//! Parser throughput benchmarks
//!
//! Measures parsing speed at multiple complexity levels.
//! Uses pre-tokenized input to isolate parser cost from lexer cost.

use atlas_runtime::{Lexer, Parser};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn tokenize(source: &str) -> Vec<atlas_runtime::Token> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    tokens
}

fn generate_nested_expr(depth: usize) -> String {
    let mut s = "let x = ".to_string();
    for _ in 0..depth {
        s.push('(');
    }
    s.push('1');
    for i in 0..depth {
        s.push_str(&format!(" + {})", i + 2));
    }
    s.push(';');
    s
}

fn generate_functions(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!(
            "fn f{}(x: number, y: number) -> number {{ return x + y + {}; }}\n",
            i, i
        ));
    }
    s
}

fn generate_typed_program(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!(
            "let v{}: number = {};\nlet s{}: string = \"hello{}\";\n",
            i, i, i, i
        ));
    }
    s
}

fn bench_parser_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_complexity");

    // Deeply nested expressions
    for depth in [10, 50, 200] {
        let source = generate_nested_expr(depth);
        let tokens = tokenize(&source);
        group.bench_with_input(
            BenchmarkId::new("nested_expr", depth),
            &tokens,
            |b, toks| {
                b.iter(|| {
                    let mut parser = Parser::new(black_box(toks.clone()));
                    let _ = black_box(parser.parse());
                });
            },
        );
    }

    group.finish();
}

fn bench_parser_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_functions");

    for count in [10, 100, 500] {
        let source = generate_functions(count);
        let tokens = tokenize(&source);
        group.bench_with_input(
            BenchmarkId::new("function_defs", count),
            &tokens,
            |b, toks| {
                b.iter(|| {
                    let mut parser = Parser::new(black_box(toks.clone()));
                    let _ = black_box(parser.parse());
                });
            },
        );
    }

    group.finish();
}

fn bench_parser_typed(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_typed");

    for count in [50, 200, 500] {
        let source = generate_typed_program(count);
        let tokens = tokenize(&source);
        group.bench_with_input(
            BenchmarkId::new("typed_decls", count),
            &tokens,
            |b, toks| {
                b.iter(|| {
                    let mut parser = Parser::new(black_box(toks.clone()));
                    let _ = black_box(parser.parse());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parser_complexity,
    bench_parser_functions,
    bench_parser_typed
);
criterion_main!(benches);
