# Atlas Runtime Fuzz Testing

Fuzz testing for the Atlas compiler frontend and interpreter using [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer).

## Prerequisites

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
```

## Fuzz Targets

| Target | Pipeline | What it tests |
|--------|----------|---------------|
| `fuzz_lexer` | bytes → lexer | Tokenization of arbitrary input |
| `fuzz_parser` | bytes → lexer → parser | AST construction from arbitrary tokens |
| `fuzz_typechecker` | bytes → lexer → parser → binder → typechecker | Full frontend type analysis |
| `fuzz_eval` | bytes → `Atlas::eval()` | Complete pipeline including interpretation |

## Running Locally

Run a specific target (runs indefinitely until stopped with Ctrl+C):

```bash
cd crates/atlas-runtime
cargo +nightly fuzz run fuzz_lexer
cargo +nightly fuzz run fuzz_parser
cargo +nightly fuzz run fuzz_typechecker
cargo +nightly fuzz run fuzz_eval
```

Run with a time limit (in seconds):

```bash
cargo +nightly fuzz run fuzz_lexer -- -max_total_time=60
```

Run all targets for 60 seconds each:

```bash
for target in fuzz_lexer fuzz_parser fuzz_typechecker fuzz_eval; do
    echo "=== Fuzzing $target ==="
    cargo +nightly fuzz run "$target" -- -max_total_time=60
done
```

## Reproducing a Crash

When the fuzzer finds a crash, it saves the input to `fuzz/artifacts/<target>/`. Reproduce it with:

```bash
cargo +nightly fuzz run fuzz_parser fuzz/artifacts/fuzz_parser/crash-<hash>
```

To get a backtrace:

```bash
RUST_BACKTRACE=1 cargo +nightly fuzz run fuzz_parser fuzz/artifacts/fuzz_parser/crash-<hash>
```

## Seed Corpus

Each target has a seed corpus in `fuzz/corpus/<target>/`. Seeds are committed to the repository and provide the fuzzer with representative starting inputs. The fuzzer evolves these seeds by mutation to explore new code paths.

To add a new seed, create a file in the appropriate corpus directory:

```bash
echo 'let x = [1, 2, 3];' > fuzz/corpus/fuzz_eval/array_literal
```

## Adding a New Fuzz Target

1. Create `fuzz/fuzz_targets/fuzz_<name>.rs` following the existing pattern
2. Add a `[[bin]]` entry to `fuzz/Cargo.toml`
3. Create `fuzz/corpus/fuzz_<name>/` with at least 5 seed files
4. Add a step to `.github/workflows/ci.yml` in the `fuzz` job
5. Run for at least 60 seconds to verify no immediate crashes

## CI Integration

The fuzz job runs nightly (04:00 UTC) via GitHub Actions, not on every PR. Each target runs for 120 seconds. Any crash fails the CI job. See `.github/workflows/ci.yml` for the configuration.

## Directory Structure

```
fuzz/
├── Cargo.toml              # Fuzz workspace (cargo-fuzz)
├── README.md               # This file
├── fuzz_targets/           # Fuzz target binaries
│   ├── fuzz_lexer.rs
│   ├── fuzz_parser.rs
│   ├── fuzz_typechecker.rs
│   └── fuzz_eval.rs
├── corpus/                 # Seed corpus (committed)
│   ├── fuzz_lexer/
│   ├── fuzz_parser/
│   ├── fuzz_typechecker/
│   └── fuzz_eval/
└── artifacts/              # Crash artifacts (gitignored)
```
