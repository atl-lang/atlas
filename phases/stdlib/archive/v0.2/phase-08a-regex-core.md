# Phase 08a: Regex Core (Compilation + Matching)

## 🚨 BLOCKERS
**REQUIRED:** String stdlib, Result types, regex crate
**Verify:** `cargo test stdlib_string && ls crates/atlas-runtime/src/result_type.rs`

## Objective
Implement core regex functionality: pattern compilation, validation, matching (is_match, find, find_all), and capture groups. This phase establishes the foundational Regex type and matching operations.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/regex.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~100 lines - add Regex variant)
**Update:** `crates/atlas-runtime/src/stdlib/prelude.rs` (register regex functions)
**Update:** `Cargo.toml` (add `regex = "1"` dependency)
**Tests:** `crates/atlas-runtime/tests/regex_core_tests.rs` (~350 lines)

## Dependencies
- `regex` crate (external)
- String stdlib (complete)
- Result<T,E> types (foundation/phase-09, complete)
- Array type (for match results)

## Implementation

### 1. Regex Value Type (value.rs)
Add `Regex(Rc<regex::Regex>)` variant to Value enum. Pattern stored as compiled regex for efficiency. Implement Debug, Display, PartialEq for Regex values. Hash implementation (error - regex not hashable). Clone wraps in new Rc.

### 2. Regex Construction (regex.rs)
```
regexNew(pattern: string) -> Result<Regex, string>
regexNewWithFlags(pattern: string, flags: string) -> Result<Regex, string>
regexEscape(text: string) -> string
```

Compile pattern using `regex::Regex::new()`. Validate pattern syntax (return Err on invalid). Flags: "i" (case-insensitive), "m" (multi-line), "s" (dot-all), "x" (extended/verbose). Build regex with `RegexBuilder` for flags. Escape special chars: `. * + ? ^ $ ( ) [ ] { } | \`.

### 3. Pattern Matching (regex.rs)
```
regexIsMatch(regex: Regex, text: string) -> boolean
regexFind(regex: Regex, text: string) -> Option<HashMap>
regexFindAll(regex: Regex, text: string) -> Array<HashMap>
```

**isMatch:** Call `regex.is_match()`, return boolean.
**find:** Return Option with HashMap: `{text: string, start: number, end: number}`.
**findAll:** Return array of match HashMaps, all matches in string.

### 4. Capture Groups (regex.rs)
```
regexCaptures(regex: Regex, text: string) -> Option<Array>
regexCapturesNamed(regex: Regex, text: string) -> Option<HashMap>
```

**captures:** Return array where index 0 = full match, index 1+ = groups.
**capturesNamed:** Return HashMap with named group mappings.

Use `regex.captures()` API. Extract groups by index or name. Return None if no match.

## Tests (TDD - rstest + insta)

### Compilation Tests (6 tests)
- Valid pattern compiles successfully
- Invalid pattern returns error with message
- Empty pattern compiles (matches empty string)
- Complex pattern compiles (nested groups, alternation)
- regexEscape escapes all special characters
- Flags modify compilation (case-insensitive, multiline)

### Matching Tests (12 tests)
- isMatch returns true on match
- isMatch returns false on no match
- isMatch with case-insensitive flag
- isMatch with multiline mode (^ and $ behavior)
- find returns first match with positions
- find returns None when no match
- findAll returns all matches
- findAll returns empty array when no matches
- findAll with overlapping patterns (non-overlapping results)
- Unicode handling (matching unicode chars)
- Dot matches newline with dot-all flag
- Anchors work correctly (^, $, \b)

### Capture Group Tests (12 tests)
- Simple capture group extraction (one group)
- Multiple capture groups (2+ groups)
- Nested capture groups
- Optional capture groups (may not match)
- Named capture groups
- Named + positional groups (mixed)
- captures returns None when no match
- capturesNamed returns None when no match
- Capture group with alternation
- Backreferences in pattern
- Non-capturing groups (?:...)
- Full match always at index 0

**Minimum:** 30 tests
**Target:** 35+ tests for comprehensive coverage

## Acceptance Criteria
- ✅ Regex value type added to Value enum
- ✅ regexNew compiles valid patterns, returns errors for invalid
- ✅ regexNewWithFlags supports i, m, s, x flags
- ✅ regexEscape escapes all special characters
- ✅ regexIsMatch returns correct boolean
- ✅ regexFind returns match with text/start/end
- ✅ regexFindAll returns all matches as array
- ✅ regexCaptures extracts groups by index
- ✅ regexCapturesNamed extracts named groups
- ✅ Unicode support works correctly
- ✅ 30+ tests pass (target 35+)
- ✅ Interpreter/VM parity verified
- ✅ cargo test passes (full suite)
- ✅ cargo clippy clean
- ✅ cargo fmt clean

## Quality Gates
**GATE -1:** Sanity check
```bash
cargo clean && cargo check -p atlas-runtime
```

**GATE 0:** Add regex dependency to Cargo.toml
**GATE 1:** Implement Regex value type in value.rs
**GATE 2:** Implement regex construction functions (new, newWithFlags, escape)
**GATE 3:** Implement pattern matching functions (isMatch, find, findAll)
**GATE 4:** Implement capture group functions (captures, capturesNamed)
**GATE 5:** Write and verify 30+ tests (all passing)
**GATE 6:** Verify parity, run full suite, clippy, fmt

## Notes
- Regex patterns use Rust's `regex` crate (fast, well-tested)
- Pattern compilation may fail - always return Result
- Match positions are byte indices (not char indices) - document this
- Named groups use `(?P<name>...)` syntax
- Regex values are NOT hashable (runtime error if attempted)
- Cache compiled regexes using `Rc<regex::Regex>` for efficiency
