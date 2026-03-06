# Oracle Testing — Reference Language Verification (Lazy-Loaded)

**Load when:** Verifying Atlas runtime behavior, debugging parity issues, or validating new language features.

---

## Purpose

Use established languages as behavioral oracles to verify Atlas produces correct output.
If Atlas and the oracle disagree, Atlas is wrong until proven otherwise.

---

## Oracle Selection

| Atlas Feature | Primary Oracle | Why |
|--------------|---------------|-----|
| Structs, enums, pattern matching | **Rust** | Nearly identical syntax and semantics |
| Traits, generics, bounds | **Rust** | Same trait system design |
| Ownership (own/borrow/shared) | **Rust** | Same ownership model (relaxed) |
| Error handling (Result/Option) | **Rust** | Identical pattern |
| Modules, import/export | **TypeScript** | Same module system |
| Union/intersection types | **TypeScript** | Same type algebra |
| Structural types, type guards | **TypeScript** | Same semantics |
| Closures, upvalue capture | **Rust** (primary), **JS** (secondary) | Rust for move semantics, JS for dynamic scoping edge cases |
| Math, string ops, stdlib | **Rust** | Both produce deterministic numeric output |

---

## How to Use

### Quick Oracle Check (during development)

Write the equivalent program in Rust or TypeScript, run it, compare output:

```bash
# 1. Write Atlas program
cat > /tmp/test.atlas << 'EOF'
let x = [1, 2, 3];
let y = arrayMap(x, fn(n) { n * 2 });
print(y);
EOF

# 2. Write Rust equivalent
cat > /tmp/test.rs << 'EOF'
fn main() {
    let x = vec![1, 2, 3];
    let y: Vec<_> = x.iter().map(|n| n * 2).collect();
    println!("{:?}", y);
}
EOF

# 3. Compare
atlas run /tmp/test.atlas
rustc /tmp/test.rs -o /tmp/test_rs && /tmp/test_rs
```

### Systematic Oracle Testing (for new features)

1. Identify 5-10 representative programs that exercise the feature
2. Write equivalents in the oracle language
3. Both must produce identical stdout
4. Differences → investigate Atlas, not the oracle

### When Oracle Disagrees with Atlas

1. **Check if it's a deliberate Atlas difference** (e.g., CoW arrays, unified `number` type)
2. **Check the spec** (`docs/language/`) — spec is authority
3. **If spec is silent:** Atlas should match the oracle's behavior. Log decision via `atlas-track add-decision`.
4. **If Atlas is wrong:** File bug, fix it

---

## Known Deliberate Differences from Oracles

| Behavior | Atlas | Rust/TS | Why |
|----------|-------|---------|-----|
| Number type | Unified `number` (f64) | i32/f64 separate | AI simplicity — one numeric type |
| Array mutation | CoW (rebind required) | In-place (Rust Vec) | Safety + parity |
| HashMap mutation | Shared (Arc<Mutex>) | In-place | Concurrent safety |
| String interpolation | `` `Hello {name}` `` | `format!("{name}")` | Cleaner syntax |
| Error propagation | `?` on Result/Option | Same in Rust | Identical |
| No exceptions | Result/Option only | try/catch (TS) | Explicit error handling |

---

## When NOT to Oracle Test

- Pure Atlas-specific features with no equivalent (CoW semantics, `shared` keyword)
- Internal compiler behavior (AST structure, bytecode format)
- Stdlib functions with no direct oracle equivalent
- Performance characteristics
