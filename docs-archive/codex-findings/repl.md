# REPL Output Capture Gap

Target: atlas-runtime REPL
Severity: Low
Status: Open

## Finding: REPL does not capture stdout

Evidence:
- /Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/repl.rs:232-239

What/Why:
- `ReplResult.stdout` is always empty because stdout capture is TODO. This limits programmatic REPL tooling and AI-agent evaluation of REPL output.

Impact:
- REPL integration tests or AI-driven flows cannot observe output consistently.

Recommendation:
- Implement output capture using the existing `OutputWriter` abstraction (e.g., an `Arc<Mutex<Vec<u8>>>` buffer) and return it in `ReplResult`.
