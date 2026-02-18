# Phase 08b: Regex Operations (Replace + Advanced)

## 🚨 BLOCKERS
**REQUIRED:** Phase-08a complete (regex core)
**Verify:** `cargo test regex_core_tests`

## Objective
Implement regex replacement operations (replace_first, replace_all), callback-based replacements, regex-based string splitting, and integration tests. Completes the regex stdlib module.

## Files
**Update:** `crates/atlas-runtime/src/stdlib/regex.rs` (~300 lines added)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (register new functions)
**Update:** `docs/api/stdlib.md` (~300 lines regex documentation)
**Tests:** `crates/atlas-runtime/tests/regex_operations_tests.rs` (~300 lines)

## Dependencies
- Phase-08a (regex core - compilation, matching, captures)
- String stdlib (for results)
- Array type (for split results)
- Function type (for callback replacements)

## Implementation

### 1. Basic Replacement (regex.rs)
```
regexReplace(regex: Regex, text: string, replacement: string) -> string
regexReplaceAll(regex: Regex, text: string, replacement: string) -> string
```

**replace:** Replace first match with replacement string. Capture group refs: `$1`, `$2`, `$&` (full match), `$`` (before match), `$'` (after match).
**replaceAll:** Replace all matches with replacement string. Same capture group ref syntax.

Use `regex.replace()` and `regex.replace_all()` from regex crate.

### 2. Callback Replacement (regex.rs)
```
regexReplaceWith(regex: Regex, text: string, callback: function) -> string
regexReplaceAllWith(regex: Regex, text: string, callback: function) -> string
```

Callback signature: `fn(match: HashMap) -> string`
HashMap contains: `{text: string, start: number, end: number, groups: Array}`

Iterate matches, call callback for each, build result string. Return Result if callback returns non-string.

### 3. String Splitting (regex.rs)
```
regexSplit(regex: Regex, text: string) -> Array<string>
regexSplitN(regex: Regex, text: string, limit: number) -> Array<string>
```

Split string at regex matches. Return array of substrings. Empty strings included (e.g., splitting "a,b,,c" on "," gives ["a", "b", "", "c"]). splitN limits number of splits.

Use `regex.split()` API.

### 4. Advanced Features (regex.rs)
```
regexMatchIndices(regex: Regex, text: string) -> Array<Array<number>>
regexTest(pattern: string, text: string) -> boolean
```

**matchIndices:** Return array of `[start, end]` pairs for all matches.
**test:** Convenience function - compile pattern + test match in one call. Return false on compile error.

### 5. Documentation (docs/api/stdlib.md)
Document all regex functions with examples:
- Pattern syntax overview (link to regex crate docs)
- Compilation and flags
- Matching functions with examples
- Capture groups with examples
- Replacement with capture group refs
- Callback replacements with function examples
- Splitting with edge cases
- Error handling (compile errors, runtime errors)

## Tests (TDD - rstest + insta)

### Replacement Tests (10 tests)
- replace replaces first match only
- replaceAll replaces all matches
- Capture group references in replacement ($1, $2)
- Special refs ($&, $`, $') work correctly
- Empty replacement deletes matches
- No match returns original string
- Unicode in replacement strings
- Escaped dollar signs in replacement
- Multiple capture groups in replacement
- Replacement at start/end of string

### Callback Replacement Tests (8 tests)
- replaceWith calls callback for first match
- replaceAllWith calls callback for all matches
- Callback receives correct match HashMap
- Callback return value used as replacement
- Callback with capture groups in match data
- Error when callback returns non-string
- Callback can use match positions
- Callback can access groups array

### Splitting Tests (8 tests)
- split divides string at matches
- split with empty matches (zero-width)
- split includes empty strings
- split with no matches returns original array
- splitN limits number of splits
- splitN with limit 0 returns empty array
- split on complex pattern
- split preserves unicode boundaries

### Integration Tests (6 tests)
- Email validation pattern
- URL extraction from text
- Phone number formatting
- HTML tag stripping (replace with empty)
- CSV parsing with regex split
- Complex text processing pipeline (find + replace + split)

**Minimum:** 25 tests
**Target:** 35+ tests for comprehensive coverage

## Acceptance Criteria
- ✅ regexReplace replaces first match
- ✅ regexReplaceAll replaces all matches
- ✅ Capture group references work ($1, $2, $&, etc.)
- ✅ regexReplaceWith calls callback correctly
- ✅ regexReplaceAllWith processes all matches
- ✅ Callback receives proper match data (text, positions, groups)
- ✅ regexSplit divides string at matches
- ✅ regexSplitN limits splits correctly
- ✅ regexMatchIndices returns all match positions
- ✅ regexTest convenience function works
- ✅ Documentation complete with examples
- ✅ 25+ tests pass (target 35+)
- ✅ Integration tests verify real-world usage
- ✅ Interpreter/VM parity verified
- ✅ cargo test passes (full suite)
- ✅ cargo clippy clean
- ✅ cargo fmt clean

## Quality Gates
**GATE -1:** Sanity check (verify phase-08a complete)
```bash
cargo test regex_core_tests && cargo check -p atlas-runtime
```

**GATE 0:** Verify phase-08a functions work correctly
**GATE 1:** Implement basic replacement (replace, replaceAll)
**GATE 2:** Implement callback replacement (replaceWith, replaceAllWith)
**GATE 3:** Implement splitting (split, splitN)
**GATE 4:** Implement advanced features (matchIndices, test)
**GATE 5:** Write and verify 25+ tests (all passing)
**GATE 6:** Write documentation, verify parity, run full suite, clippy, fmt

## Notes
- Capture group refs: `$1` (group 1), `$&` (full match), `$`` (before), `$'` (after)
- Callbacks must return strings - enforce with type checking
- Split includes empty strings (matches POSIX behavior)
- regexTest is a convenience - compiles + tests in one call
- Error handling: compile errors, callback errors, type errors
- Integration tests should use realistic patterns (email, URL, etc.)
- Document byte vs char index behavior clearly
