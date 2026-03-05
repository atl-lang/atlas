## H-047: Test runner aborts on non-test .atlas files
**Verification:** Ran `NO_COLOR=false atlas test` and observed parse errors reported from `crates/atlas-runtime/tests/corpus/fail/syntax_errors/*.atlas`. Read `crates/atlas-cli/src/testing/discovery.rs` (only scans `*.test.atl`).
**Reality:** The CLI reports parse errors from non-test `.atlas` files even though discovery code is intended to scan only `*.test.atl`.
**Classification:** Code bug
**Action:** Abandoned

## H-048: NO_COLOR=1 breaks atlas test
**Verification:** `atlas test` fails with `invalid value '1' for '--no-color'` when `NO_COLOR=1` is set in the environment. `NO_COLOR=1 atlas test --help` renders help. Read CLI arg definition for `--no-color` in `crates/atlas-cli/src/main.rs`.
**Reality:** `NO_COLOR=1` causes argument parsing failure for `atlas test`.
**Classification:** Code bug
**Action:** Abandoned

## H-049: Test function signature mismatch
**Verification:** `atlas test --help` says tests "return true on success". Read `crates/atlas-cli/src/testing/discovery.rs` (only enforces no parameters; return value ignored). Reviewed `docs/specification/testing.md`.
**Reality:** Test functions must have no parameters; return values are ignored. CLI help text was incorrect and spec implied stricter return rules.
**Classification:** Doc issue
**Action:** Fixed

## H-050: Test filter flag mismatch
**Verification:** `NO_COLOR=false atlas test --filter foo` errors with unexpected argument. `NO_COLOR=false atlas test foo` runs with a positional pattern. Reviewed `docs/specification/testing.md` and `docs/cli-reference.md`.
**Reality:** The CLI uses a positional pattern argument, not `--filter`.
**Classification:** Doc issue
**Action:** Fixed

## H-051: Enum constructor syntax undocumented
**Verification:** Read enum parsing and enum variant expression code in `crates/atlas-runtime/src/parser/mod.rs` and `crates/atlas-runtime/src/parser/expr.rs`. Ran `cargo run -p atlas-cli -- run /tmp/enum_test.atl` (unit/tuple variants parse) and `cargo run -p atlas-cli -- run /tmp/enum_struct_variant.atl` (struct variant constructor is a syntax error).
**Reality:** Enums support unit/tuple/struct variants in declarations. Construction syntax exists only for unit/tuple variants (`Enum::Variant` / `Enum::Variant(args)`); struct variant construction is not supported.
**Classification:** Doc issue
**Action:** Fixed

## H-052: Type export/import semantics unclear
**Verification:** Read `crates/atlas-runtime/src/parser/mod.rs` (export supports only `fn`, `let`, `type`) and `crates/atlas-runtime/src/module_executor.rs` (exports only runtime values). Ran `cargo run -p atlas-cli -- run /tmp/main.atl` importing a type alias and saw `'Point' is not exported` error; ran `cargo run -p atlas-cli -- run /tmp/export_struct.atl` and `/tmp/export_enum.atl` and saw "Expected 'fn', 'let', or 'type' after 'export'".
**Reality:** `export type` is compile-time only and does not create runtime exports. `import { name }` imports runtime values only. `export struct` / `export enum` are invalid.
**Classification:** Doc issue
**Action:** Fixed

## H-053: assertThrows/assertNoThrow unusable
**Verification:** Read `crates/atlas-runtime/src/stdlib/test.rs` implementations for `assertThrows`/`assertNoThrow`.
**Reality:** These functions accept only `NativeFunction` values and reject Atlas-defined functions, making them unusable from normal Atlas tests.
**Classification:** Code bug
**Action:** Abandoned

## H-054: Multiline string syntax not specified
**Verification:** Read `crates/atlas-runtime/src/lexer/literals.rs` string scanning (allows `\n` inside `"..."`). Ran `cargo run -p atlas-cli -- run /tmp/multiline.atl` with a multiline string literal.
**Reality:** Multiline strings are standard double-quoted strings containing literal newlines; no special triple-quote syntax.
**Classification:** Doc issue
**Action:** Fixed
