# Atlas Tests

## Layout

### Modular Stdlib Tests
- `tests/stdlib/` - Interpreter stdlib tests (modular by API)
  - `mod.rs` - Common utilities (eval_ok, eval_err)
  - `array_pure.rs` - Non-callback array functions (pop, shift, concat, etc.)
  - `array_intrinsics.rs` - Callback-based array functions (map, filter, reduce, etc.)
  - `math_basic.rs` - Basic math operations (abs, floor, sqrt, pow, etc.)
  - `math_trig.rs` - Trigonometric functions (sin, cos, tan, etc.)
  - `math_utils_constants.rs` - Math utilities (clamp, sign, random) & constants (PI, E, etc.)

### Modular VM Tests (Parity)
- `tests/vm/` - VM tests (mirrors stdlib/ structure for interpreter/VM parity)
  - `mod.rs` - Common utilities (execute_vm_ok, execute_vm_err)
  - Same module structure as stdlib/

### Component Tests
- `tests/*_tests.rs` - Component-specific tests (lexer, parser, typechecker, etc.)

### Test Data
- `tests/snapshots/` - Auto-generated snapshot files (insta)

## Golden Tests
- Each `.atl` test file has a corresponding `.out` or `.diag` file.
- Outputs must be deterministic.

## Running
- `cargo test` runs unit and golden tests.
