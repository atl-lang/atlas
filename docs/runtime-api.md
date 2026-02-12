# Atlas Runtime API (Design Sketch)

## Purpose
Define a stable minimal API for embedding Atlas in host applications later.

## v0.1 Status
- Not implemented, but API shape is defined to avoid refactors.

## Proposed API
- `Atlas::new()` -> runtime instance
- `Atlas::eval(source: &str) -> Result<Value, Diagnostic>`
- `Atlas::eval_file(path: &str) -> Result<Value, Diagnostic>`
- `Atlas::set_stdout(writer)` (optional)

## Value Interop
- Expose `Value` as a tagged enum for host usage.

## Future
- C-ABI wrapper for embedding in non-Rust hosts.
