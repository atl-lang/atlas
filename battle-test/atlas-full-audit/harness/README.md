# Battle Test Harness

The Rust harness is in `crates/atlas-runtime/tests/battle_audit.rs`.

It runs every `.atlas` file in `battle-test/atlas-full-audit/domains/` through
both engines (interpreter + VM) and asserts:

1. Program compiles and runs without error
2. Output matches `*.expected` file (if present)
3. Interpreter and VM produce identical output (parity)

Run:
```bash
cargo nextest run -p atlas-runtime -E 'test(battle_audit)'
```
