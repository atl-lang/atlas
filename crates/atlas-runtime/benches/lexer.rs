//! Lexer throughput benchmarks
//!
//! Measures tokenization speed at multiple input sizes.
//! Reports throughput as MB/s via Criterion's Throughput::Bytes.

use atlas_runtime::Lexer;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn generate_arithmetic(n: usize) -> String {
    // Generates: let x = 1 + 2 + 3 + ... + n;
    let mut s = String::from("let x = 1");
    for i in 2..=n {
        s.push_str(&format!(" + {}", i));
    }
    s.push(';');
    s
}

fn generate_complex_program(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        s.push_str(&format!(
            "fn func_{}(a: number, b: string) -> number {{ let x = a + {}; return x; }}\n",
            i, i
        ));
    }
    s
}

fn bench_lexer_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_throughput");

    // Simple arithmetic at increasing sizes
    for token_count in [100, 1000, 10000] {
        let source = generate_arithmetic(token_count);
        let bytes = source.len() as u64;
        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("arithmetic", token_count),
            &source,
            |b, src| {
                b.iter(|| {
                    let mut lexer = Lexer::new(black_box(src.clone()));
                    let _ = black_box(lexer.tokenize());
                });
            },
        );
    }

    group.finish();
}

fn bench_lexer_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_complex");

    for func_count in [10, 100, 500] {
        let source = generate_complex_program(func_count);
        let bytes = source.len() as u64;
        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(
            BenchmarkId::new("functions", func_count),
            &source,
            |b, src| {
                b.iter(|| {
                    let mut lexer = Lexer::new(black_box(src.clone()));
                    let _ = black_box(lexer.tokenize());
                });
            },
        );
    }

    group.finish();
}

fn bench_lexer_string_heavy(c: &mut Criterion) {
    // String-heavy input (lots of string literals)
    let mut source = String::new();
    for i in 0..200 {
        source.push_str(&format!(
            "let s{} = \"hello world string literal number {}\";\n",
            i, i
        ));
    }
    let bytes = source.len() as u64;

    let mut group = c.benchmark_group("lexer_strings");
    group.throughput(Throughput::Bytes(bytes));
    group.bench_function("200_string_literals", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source.clone()));
            let _ = black_box(lexer.tokenize());
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_lexer_throughput,
    bench_lexer_complex,
    bench_lexer_string_heavy
);
criterion_main!(benches);
