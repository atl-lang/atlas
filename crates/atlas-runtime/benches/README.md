# Atlas Runtime Benchmarks

Criterion-based benchmark suite covering all critical performance paths.

## Benchmark Files

| File | What it measures |
|------|-----------------|
| `lexer.rs` | Tokenization throughput (MB/s) at multiple input sizes |
| `parser.rs` | Parse speed across expression depth, function count, type annotations |
| `typechecker.rs` | Full frontend pipeline (lex + parse + bind + typecheck) |
| `interpreter.rs` | Tree-walking interpreter on canonical programs |
| `vm_performance_benches.rs` | VM execution across arithmetic, functions, loops, arrays, scaling |
| `stdlib_benchmarks.rs` | Stdlib function performance (string, array, math, JSON, file I/O) |
| `parity.rs` | Interpreter vs VM head-to-head on identical programs |

## Running Benchmarks

```bash
# Full suite
cargo bench -p atlas-runtime

# Single benchmark file
cargo bench -p atlas-runtime --bench lexer
cargo bench -p atlas-runtime --bench parity

# Filter by benchmark name
cargo bench -p atlas-runtime -- "fibonacci"

# Quick mode (fewer iterations, faster feedback)
cargo bench -p atlas-runtime -- --quick
```

## Interpreting Output

Criterion reports three values: `[low est  mean  high est]`
- **mean** is the primary metric
- Throughput benchmarks (lexer) also report MB/s
- On subsequent runs, Criterion shows change vs previous: `[-2.5% -1.0% +0.5%]`
- Changes beyond ~5% are likely real; within noise otherwise

## Adding a New Benchmark

1. Add a function: `fn bench_my_thing(c: &mut Criterion) { ... }`
2. Add it to the appropriate `criterion_group!`
3. If creating a new file, add `[[bench]] name = "..." harness = false` to `Cargo.toml`
4. Run `cargo bench -p atlas-runtime --bench <name>` to verify

## Baseline

`baseline.txt` contains the initial benchmark numbers captured on the build machine.
This is the performance contract — regressions should be investigated.

## CI

Benchmarks run on pushes to `main` (not PRs). Results are stored as CI artifacts.
