# Bytecode Serialization Gap

Target: atlas-runtime + atlas-build
Severity: High
Status: Open

## Finding: Bytecode serialization is explicitly TODO

Evidence:
- `/Users/proxikal/dev/projects/atlas/crates/atlas-runtime/src/bytecode/serialize.rs` (TODO note)
- `/Users/proxikal/dev/projects/atlas/crates/atlas-build/src/builder.rs:881` (TODO: proper bytecode serialization format)

What/Why:
- There is no stable bytecode serialization format yet. Build outputs cannot reliably be cached, shipped, or compared.

Impact:
- Breaks reproducible builds and blocks any persistent cache or artifact-based workflow.

Recommendation:
- Define and implement a stable bytecode format (versioned + checksummed), add round-trip tests, and integrate into build outputs.

