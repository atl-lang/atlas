# Doc-Sync PR Summary

Scope: Align specification and guides with current runtime implementation (CoW memory model + trait error codes), and remove outdated Arc<Mutex> references.

## Changes
1. Trait error codes in spec updated to match runtime (`AT3030+` series).
2. Language semantics updated to CoW array semantics and write-back model.
3. Interpreter status updated to CoW value representation.
4. Embedding guide updated to CoW value types and current `Value` enum usage.
5. JIT status updated to reflect CoW array model.

## Files Updated
- /Users/proxikal/dev/projects/atlas/docs/specification/types.md
- /Users/proxikal/dev/projects/atlas/docs/specification/language-semantics.md
- /Users/proxikal/dev/projects/atlas/docs/interpreter-status.md
- /Users/proxikal/dev/projects/atlas/docs/embedding-guide.md
- /Users/proxikal/dev/projects/atlas/crates/atlas-jit/JIT_STATUS.md

## Rationale
These changes remove incorrect Arc<Mutex> references and fix error-code mismatches that would mislead AI agents and tooling, directly supporting the AI-first correctness and predictability goals.
