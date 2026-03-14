# GATE 3: Verify Parity (100% REQUIRED)

**Condition:** Implementation and tests complete

---

## ⚠️ PARITY IS NON-NEGOTIABLE

**D-052: Single execution path — compiler + VM only. The interpreter is removed.**

Compiler output MUST match the spec. Parity = "Atlas program produces correct output per spec". No exceptions.

---

## Verification

### Use corpus tests (preferred)

The preferred pattern is a corpus test — it runs the program through the compiler+VM and asserts exact output:

```bash
# Create corpus test
echo 'let x = 42; console.log(x.toString());' > crates/atlas-runtime/tests/corpus/pass/my_feature.atlas
echo '42' > crates/atlas-runtime/tests/corpus/pass/my_feature.stdout
```

### Use `assert_eval_*` helpers in Rust tests

```rust
#[test]
fn test_feature_parity() {
    assert_eval_number(r#"let x = 42; x"#, 42.0);
    assert_eval_string(r#""hello""#, "hello");
    assert_eval_bool(r#"typeof([]) == "array""#, true);
}
```

### Run parity tests

```bash
# Run all parity tests
cargo nextest run -p atlas-runtime -E 'test(parity)'

# Run domain file that contains parity tests
cargo nextest run -p atlas-runtime --test <domain_file>
```

### Common parity violations

- Compiler output doesn't match spec (wrong value, missing output)
- Wrong error code or message vs spec expectation
- Method dispatch returns wrong type or crashes
- Snapshot/corpus `.stdout` file out of date with current output

---

## Decision

- All parity tests pass → GATE 4
- Any fail → **BLOCKING** → Fix → Retry

**100% parity or the phase is INCOMPLETE.**

---

**Next:** GATE 4
