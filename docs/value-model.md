# Atlas Value Model (Detailed)

## Goals
- Single `Value` type shared by interpreter and VM.
- Clear ownership and mutation rules.
- Deterministic behavior for AI and tests.

## Rust Type Sketch
```rust
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Null,
    Array(Rc<RefCell<Vec<Value>>>),
    Function(FunctionRef),
}
```

## Ownership Rules
- Numbers/bools/null are immediate values.
- Strings and arrays are heap allocated and reference-counted.
- Assignment copies references, not deep copies.
- Function arguments follow the same rule as assignment.

## Mutability
- Strings are immutable.
- Arrays are mutable through `RefCell` and shared by reference.
- Modifying an array is visible to all references.

## Equality
- `==` and `!=` are defined only for same-type operands.
- Arrays compare by reference identity (v0.1) to avoid deep compare cost.

## Future Extensions
- Optional `Option<T>` or `Result<T, E>` types.
- GC or arena allocation if refcount overhead becomes a bottleneck.
