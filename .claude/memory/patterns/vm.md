# VM Patterns

## P-002: Upvalue Chaining via UpvalueCapture Enum
**Status:** Active

Closures use `UpvalueCapture::Local(abs_idx)` for direct parent captures and
`UpvalueCapture::Upvalue(parent_idx)` for grandparent+ captures. `register_upvalue_at_depth`
recursively chains through `UpvalueContext` entries. At definition site, emit `GetLocal` for
`Local` captures and `GetUpvalue` for `Upvalue` captures. `upvalue_stack: Vec<UpvalueContext>`
stores `parent_base` (= `prev_local_base` when pushed) + `captures`.

## P-001: .atb Bytecode Format with Debug Info
**Status:** Active

Binary bytecode (.atb) with embedded debug info. Source locations for errors.
