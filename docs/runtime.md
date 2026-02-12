# Atlas Runtime Model (v0.1)

## Value Representation
- Single `Value` enum shared by interpreter and VM.
- Variants: `Number(f64)`, `String(Rc<String>)`, `Bool(bool)`, `Null`, `Array(Rc<RefCell<Vec<Value>>>)`, `Function(FunctionRef)`.

## Memory Model
- Reference counting (`Rc`/`Arc`) for heap values.
- No garbage collector in v0.1.
- All heap values are immutable by default except arrays.

## Mutability Rules
- Strings are immutable.
- Arrays are mutable, reference-counted, and shared by reference.
- Assignment copies references, not deep copies.

## Strings
- Strings are UTF-8.

## Execution Model
- Interpreter and VM share identical semantics.
- Evaluation order is left-to-right.
- Function arguments are evaluated left-to-right and passed by value (heap types are shared by reference).

## Error Semantics
- Runtime errors produce diagnostics with stack traces.
- Runtime errors do not abort the REPL.
- Array indexing requires integer values; non-integer indices are runtime errors.
- Invalid numeric results (NaN/Infinity) are runtime errors.
