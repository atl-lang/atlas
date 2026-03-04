# Hydra Battle Test Audit (Round 2)

## Build Results
Partial. `atlas test` failed during discovery because the runner parses non-test `.atlas` corpus files (expected-to-fail syntax fixtures) and aborts, so no tests were executed. Command used: `NO_COLOR=false atlas test hydra` (also fails without pattern) and the runner reported parse errors in `crates/atlas-runtime/tests/corpus/fail/**` and `battle-test/hydra/supervisor/supervisor.atlas` before finding any tests.

## Comparison: Atlas vs Python
Atlas can express a clean, functional web framework with strong typing, pattern matching, and first-class tests in a single file. It’s better than Python at enforcing consistency (types, purity by default) and makes middleware composition explicit and auditable. However, building Hydra in Atlas is harder today due to missing/unclear enum constructor syntax, insufficiently documented module/type export semantics, and test runner behavior that prevents running focused tests in real-world repos. Python wins on dynamic ergonomics, mature HTTP ecosystem, and quick iteration. Atlas wins on deterministic behavior, compiler-assisted refactors, and AI-parseable diagnostics (when they surface). To be “perfect for AI codegen,” Atlas needs: fully specified surface syntax (especially enums/structs), reliable and isolated test execution, and a stdlib test API that works for pure Atlas code without embedding hooks.

## Issues Found

### H-047: Test runner aborts on non-test `.atlas` files
**Priority:** P0
**Component:** cli
**Problem:** `atlas test` scans and parses non-test `.atlas` files (including corpus fail fixtures), then aborts the entire run on expected parse errors. This prevents any `.test.atl` tests from running in repositories that contain intentional failure fixtures.
**Impact:** AI agents cannot reliably verify changes or isolate tests; test validation becomes impossible in real repos with corpus fixtures.
**Solution:** Restrict discovery to `*.test.atl` by default and ignore non-test files. Add an opt-in flag if full-project parse is desired.

### H-048: `NO_COLOR=1` breaks `atlas test`
**Priority:** P1
**Component:** cli
**Problem:** The CLI’s `--no-color` flag expects `true|false`, but the standard `NO_COLOR=1` environment value causes an error: “invalid value '1' for '--no-color'.”
**Impact:** Common CI environments set `NO_COLOR=1`, which breaks test runs and undermines automation.
**Solution:** Treat `NO_COLOR` as a boolean flag: accept `1`, `true`, or any value as “enabled.”

### H-049: Test function signature mismatch between CLI help and spec
**Priority:** P1
**Component:** docs/cli
**Problem:** CLI help says tests “return true on success,” but the testing spec says tests return `void` and use assertion functions.
**Impact:** AI and human users write incorrect tests or add spurious returns to satisfy conflicting guidance.
**Solution:** Align CLI help with the spec (void return + assertions), or update the spec to match actual behavior.

### H-050: Test filter flag mismatch with documentation
**Priority:** P1
**Component:** docs/cli
**Problem:** The spec documents `atlas test --filter`, but the CLI only accepts a positional `[PATTERN]` and rejects `--filter`.
**Impact:** AI agents follow docs and hit CLI errors, wasting cycles and causing brittle automation scripts.
**Solution:** Either implement `--filter` as documented or update docs and examples to use positional patterns.

### H-051: Enum constructor syntax is undocumented and ambiguous
**Priority:** P1
**Component:** parser/docs
**Problem:** The syntax for constructing and matching user-defined enum variants (e.g., `Status.OK`, `OK()`, or `Status::OK`) is not specified. Existing examples only cover `Option`/`Result`.
**Impact:** AI code generation becomes unreliable for enums; generated code is likely to be syntactically wrong.
**Solution:** Specify and enforce a single constructor syntax, add examples, and add compiler errors with fix-it hints.

### H-052: Type export/import semantics are unclear
**Priority:** P1
**Component:** docs
**Problem:** Module import docs show function/var imports but do not clarify how types (structs/enums/traits) are exported and referenced across modules (e.g., `hydra.Request`).
**Impact:** AI agents guess and frequently produce non-compiling code when referencing types in other modules.
**Solution:** Document type export/import rules and include examples for structs, enums, and traits.

### H-053: `assertThrows`/`assertNoThrow` unusable for pure Atlas tests
**Priority:** P2
**Component:** stdlib/testing
**Problem:** These assertions only accept `NativeFunction` (embedding API), so you can’t test exceptions from Atlas-defined functions inside `.test.atl` files.
**Impact:** Test coverage for error paths is impaired; AI-generated tests will fail or be forced into awkward workarounds.
**Solution:** Allow bytecode functions or provide a test wrapper that executes Atlas closures in the interpreter/VM context.

### H-054: Multiline string literal syntax is not specified
**Priority:** P2
**Component:** docs/parser
**Problem:** Parser improvements mention multiline strings, but there is no canonical syntax documented (`"""`, raw strings, or embedded newlines).
**Impact:** AI agents cannot reliably generate valid multiline strings; codegen becomes trial-and-error.
**Solution:** Document the exact grammar and add explicit examples to `docs/specification/syntax.md`.
