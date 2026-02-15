# DR-003: Hash Function Design for Collections

**Date:** 2026-02-15
**Status:** Accepted
**Component:** Standard Library - Collections

## Context
HashMap and HashSet require hash function infrastructure for key storage and lookup. Atlas needs deterministic, AI-friendly hashing that works with Atlas value types.

## Decision
**Implement deterministic hash function using Rust's `std::hash::Hash` trait with `DefaultHasher`:**

**Hashable types:**
- Number: Hash IEEE 754 bits (convert NaN to canonical representation)
- String: Hash UTF-8 bytes directly
- Bool: Hash as 0 (false) or 1 (true)
- Null: Hash as fixed constant

**Non-hashable types (runtime error AT0140):**
- Array: Mutable, no structural equality
- Function: Non-comparable, no meaningful hash
- JsonValue: Mutable, use explicit key extraction
- Option/Result: Complex, defer to v0.3

**Hash collision strategy:**
- Separate chaining (like Rust HashMap pre-1.36)
- Automatic resizing at 0.75 load factor
- Initial capacity: 16 buckets

## Rationale
**Deterministic hashing:** No RandomState/SipHash - ensures reproducible behavior for AI testing and debugging. Production security not a v0.2 concern.

**Type restrictions match JavaScript Map behavior:**
- JS Map allows primitives as keys, rejects mutable objects without explicit conversion
- Python dict allows hashable types (immutable): str, int, float, bool, None, tuple
- Rust HashMap requires `Hash + Eq` trait bounds

**AI-first principle:** Predictable, testable, debuggable hash values for development/testing.

**Industry precedent:**
- **Rust:** Hash trait with separate chaining, robin hood hashing
- **Go:** Built-in map with compiler magic, deterministic within session
- **Python:** `__hash__` protocol, deterministic hashing for primitives

## Alternatives Considered
- **RandomState with SipHash:** Rejected - non-deterministic breaks reproducible testing, unnecessary for v0.2
- **Allow array hashing (structural):** Rejected - requires deep equality, expensive, mutable arrays problematic
- **Custom hash algorithm:** Rejected - Rust's DefaultHasher is proven, fast, simple

## Consequences
- ✅ **Benefits:** Deterministic, reproducible behavior (AI-friendly)
- ✅ **Benefits:** Simple implementation using Rust std library
- ✅ **Benefits:** Matches Python/JS patterns (familiar to developers)
- ⚠️  **Trade-offs:** No DoS protection from hash flooding (acceptable for v0.2)
- ⚠️  **Trade-offs:** Cannot hash arrays/objects (need explicit key extraction)
- ❌ **Costs:** NaN handling requires special case (canonicalization)

## Implementation Notes
**Phase:** `stdlib/phase-07a-hash-infrastructure-hashmap.md` (v0.2)

**Hash function location:** `crates/atlas-runtime/src/stdlib/collections/hash.rs`

**Value enum additions:**
```rust
// In value.rs
Value::HashMap(Rc<RefCell<HashMap<HashKey, Value>>>)
Value::HashSet(Rc<RefCell<HashSet<HashKey>>>)
```

**HashKey wrapper:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum HashKey {
    Number(OrderedFloat<f64>),  // Canonical NaN handling
    String(Rc<String>),
    Bool(bool),
    Null,
}
```

**Error codes:**
- AT0140: "Cannot hash type {type} - only number, string, bool, null are hashable"
- AT0141: "HashMap key must be hashable type"

## References
- Spec: `docs/api/stdlib.md` (Collections section - to be added)
- Related: DR-002 (Array Intrinsics)
- External: Rust HashMap internals, Python dict implementation, JS Map specification
