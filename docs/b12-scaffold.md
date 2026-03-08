# B12 Scaffold: AI-First Error Messages

Block registered in pt. See `pt block B12` and `pt phases B12` for full phase detail.

## Phase Dependency Graph

```
P01 (Audit)
├── P02 (Cross-Language Parser)
├── P03 (Edit-Distance Engine)
│   └── P04 (Typechecker Help)
│       └── P05 (Stdlib Errors)
├── P06 (Ownership Edges)
├── P07 (Runtime Errors)
│   └── P08 (REPL Guidance)
│       └── P09 (Battle Test) ← depends on P02-P08
│           └── P10 (Docs)
```

## Quality Bar

AT1007 is the reference. Every error must reach:
- Named error code (e.g. `AT1007`)
- Specific problem description
- Atlas fix shown inline

## Key Files

- `crates/atlas-runtime/src/diagnostic/error_codes.rs` — central help text registry
- `crates/atlas-runtime/src/typechecker/suggestions.rs` — edit-distance engine (unwired)
- `crates/atlas-runtime/src/parser/mod.rs` — cross-language pattern detection target
