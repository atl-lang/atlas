# Atlas - AI-first systems language
# Rust project commands

# Default: show available commands
default:
    @just --list

# Run all tests
test:
    cargo nextest run -p atlas-runtime

# Run specific test file
test-file FILE:
    cargo nextest run -p atlas-runtime --test {{FILE}}

# Build all crates
build:
    cargo build --all

# Build release
build-release:
    cargo build --all --release

# Format code
fmt:
    cargo fmt --all

# Lint with clippy
lint:
    cargo clippy --all -- -D warnings

# Run formatter + linter
check: fmt lint

# Full CI locally (fmt + clippy + test + coverage)
ci:
    cargo fmt --all --check
    cargo clippy --all -- -D warnings
    cargo nextest run -p atlas-runtime
    cargo tarpaulin --out Html --output-dir coverage

# Clean build artifacts
clean:
    cargo clean

# Run cargo deny (supply chain audit)
audit:
    cargo deny check

# Run benchmarks
bench:
    cargo bench

# Dev: watch and run tests on file changes
dev:
    watchexec -e rs "cargo nextest run -p atlas-runtime"
