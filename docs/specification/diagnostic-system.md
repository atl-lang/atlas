# Atlas Diagnostic System

**Purpose:** Comprehensive reference for Atlas diagnostic system including errors, warnings, formatting, and stability rules

**For AI Agents:** This is your single source of truth for diagnostic behavior. All errors and warnings follow these rules consistently across compiler, interpreter, and VM.

---

## Table of Contents
1. [Overview](#overview)
2. [Diagnostic Schema](#diagnostic-schema)
3. [Output Formats](#output-formats)
4. [Error Codes](#error-codes)
5. [Warning Codes](#warning-codes)
6. [Emission Policy](#emission-policy)
7. [Ordering Rules](#ordering-rules)
8. [Normalization Rules](#normalization-rules)

---

## Overview

All errors and warnings in Atlas are represented as `Diagnostic` objects. This ensures consistency across:
- Compiler (lexer, parser, type checker)
- Interpreter
- Bytecode VM
- REPL
- CLI tools

**Core Principles:**
- Every diagnostic includes precise location information (file, line, column, length)
- Every diagnostic has a unique error code
- Diagnostics are deterministic and reproducible
- Both human-readable and machine-readable formats are supported

---

## Diagnostic Schema

All diagnostics follow this schema:

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `diag_version` | integer | Diagnostic schema version (currently 1) |
| `level` | string | Either `"error"` or `"warning"` |
| `code` | string | Unique diagnostic code (e.g., `"AT0001"`) |
| `message` | string | Short summary of the diagnostic |
| `file` | string | File path (normalized) |
| `line` | integer | 1-based line number |
| `column` | integer | 1-based column number |
| `length` | integer | Length of the error span in characters |
| `snippet` | string | Source line containing the error |
| `label` | string | Short label for the caret range |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `notes` | string[] | Additional explanatory notes |
| `related` | object[] | Secondary locations providing context |
| `help` | string | Suggested fix or resolution |

### Related Location Schema

Each entry in `related` array:
```json
{
  "file": "path/to/file.atl",
  "line": 10,
  "column": 5,
  "length": 6,
  "message": "context message"
}
```

---

## Output Formats

### Human-Readable Format

**Error Example:**
```
error[AT0001]: Type mismatch
  --> path/to/file.atl:12:9
   |
12 | let x: number = "hello";
   |         ^^^^^ expected number, found string
   |
help: convert the value to number or change the variable type
```

**Warning Example:**
```
warning[AT2001]: Unused variable
  --> path/to/file.atl:5:9
   |
 5 | let unused = 42;
   |     ^^^^^^ variable declared but never used
   |
help: remove the variable or prefix with underscore: _unused
```

**Format Rules:**
- Level and code on first line: `error[CODE]: Message`
- Location on second line: `--> file:line:column`
- Source snippet with line number
- Caret line showing span with label
- Optional help text at end

### Machine-Readable (JSON) Format

**Full Example:**
```json
{
  "diag_version": 1,
  "level": "error",
  "code": "AT0001",
  "message": "Type mismatch",
  "file": "path/to/file.atl",
  "line": 12,
  "column": 9,
  "length": 5,
  "snippet": "let x: number = \"hello\";",
  "label": "expected number, found string",
  "notes": [
    "inferred type of \"hello\" is string"
  ],
  "related": [
    {
      "file": "path/to/file.atl",
      "line": 10,
      "column": 5,
      "length": 6,
      "message": "variable declared here"
    }
  ],
  "help": "convert the value to number or change the variable type"
}
```

**JSON Output:**
- One JSON object per diagnostic
- Newline-delimited for multiple diagnostics
- Suitable for parsing by tools and AI agents
- All fields guaranteed to be present (except optional ones)

---

## Error Codes

### Error Code Format
- Prefix: `AT` (Atlas)
- Category digit: Error type (0-9)
- Sequential number: 001-999 within category
- Example: `AT0001`, `AT0005`, `AT3003`

### Error Code Categories

**AT0xxx - Type Errors:**
- `AT0001`: Type mismatch
- `AT0002`: Undefined symbol
- `AT0005`: Divide by zero (runtime)
- `AT0006`: Out-of-bounds array access (runtime)
- `AT0007`: NaN or Infinity result (runtime)

**AT1xxx - Syntax Errors:**
- Parse errors, malformed expressions, invalid syntax

**AT2xxx - Warnings:**
- `AT2001`: Unused variable
- `AT2002`: Unreachable code

**AT3xxx - Semantic Errors:**
- `AT3003`: Immutability violation (assigning to `let`)

**AT4xxx - Runtime Errors:**
- Stack overflow, invalid stdlib arguments, etc.

**AT5xxx - Module Errors:**
- `AT5003`: Circular dependency detected
- `AT5004`: Cannot export (symbol not found)
- `AT5005`: Module not found
- `AT5006`: Module does not export symbol
- `AT5007`: Namespace imports not yet supported
- `AT5008`: Duplicate export

### Error Code Policy

**Rules:**
- Every diagnostic MUST use a defined error code
- New error codes MUST be added to `Atlas-SPEC.md` first
- Error codes are stable (never reused for different errors)
- Same error condition always produces same error code

**For complete error code listing, see:** `Atlas-SPEC.md`

---

## Warning Codes

### Warning Categories

Warnings are non-fatal diagnostics that indicate potential issues without blocking execution.

### Current Warning Codes

**AT2001: Unused Variable**
- **Triggered when:** Variable declared but never read
- **Example:**
  ```atlas
  let unused = 42;  // AT2001 warning
  let x = 5;
  print(x);
  ```
- **Suppression:** Prefix variable name with `_`
  ```atlas
  let _unused = 42;  // No warning
  ```

**AT2002: Unreachable Code**
- **Triggered when:** Code after `return`, `break`, or `continue` that cannot execute
- **Example:**
  ```atlas
  fn foo() -> number {
      return 42;
      print("never executed");  // AT2002 warning
  }
  ```

### Warning Emission Rules

**Behavior:**
- Warnings do NOT block execution
- Programs with warnings can still run
- Warnings are emitted even if errors exist

**Unused Variable Rules:**
- Variables starting with `_` do NOT trigger warnings
- Unused function parameters ARE warned unless prefixed with `_`
- Unused function return values do NOT trigger warnings

---

## Emission Policy

### Error Emission

**Compile-Time Errors:**
- Stop after **25 errors** (prevent overwhelming output)
- Errors are emitted as soon as detected
- First 25 errors are guaranteed to be reported

**Runtime Errors:**
- First error stops execution
- Runtime error produces diagnostic with stack trace
- REPL reports runtime error but continues accepting input

**REPL-Specific:**
- REPL reports **first error** per input line
- Multiple errors in one input only show first
- Avoids overwhelming user during interactive development

### Warning Emission

**Compile-Time Warnings:**
- ALL warnings are emitted (no limit)
- Warnings emitted even if errors exist
- Warnings do not block compilation or execution

**Warning vs Error Priority:**
- Errors are shown before warnings
- Warnings are supplementary information

---

## Ordering Rules

### Purpose
Ensure deterministic, predictable diagnostic output. Same input always produces same output order.

### Ordering Algorithm

**Primary Sort:** Level
1. All errors first
2. Then all warnings

**Secondary Sort:** Location (within same level)
1. Sort by file path (lexicographic)
2. Then by line number (ascending)
3. Then by column number (ascending)

**Example:**
```
Input with diagnostics at:
- error at file.atl:15:5
- warning at file.atl:10:3
- error at file.atl:10:5

Output order:
1. error at file.atl:10:5  (error, line 10, col 5)
2. error at file.atl:15:5  (error, line 15, col 5)
3. warning at file.atl:10:3 (warning after all errors)
```

### Multiple Files

When diagnostics span multiple files:
```
file_a.atl:10:5 error
file_b.atl:5:3 error
file_a.atl:20:1 error
file_b.atl:15:7 warning

Output order:
1. file_a.atl:10:5 error
2. file_a.atl:20:1 error
3. file_b.atl:5:3 error
4. file_b.atl:15:7 warning
```

---

## Normalization Rules

### Purpose
Ensure diagnostics are stable and reproducible across different machines, operating systems, and development environments.

### Path Normalization

**Absolute to Relative:**
- Strip absolute path prefixes
- Use relative paths from project root
- Example: `/Users/me/projects/atlas/src/main.atl` → `src/main.atl`

**Fallback to Filename:**
- If relative path unavailable, use filename only
- Example: `main.atl`

**Consistency:**
- Same source file always produces same path in diagnostics
- No machine-specific path information leaks into diagnostics

### Line Ending Normalization

**Rule:**
- All line endings normalized to `\n` (Unix style)
- Applies to both source input and diagnostic output
- `\r\n` (Windows) converted to `\n`
- `\r` (old Mac) converted to `\n`

**Rationale:**
- Consistent diagnostics regardless of source file line endings
- Identical JSON output across platforms

### Volatile Field Removal

**Remove:**
- Timestamps
- OS-specific paths
- Machine-specific information
- Temporary directory paths
- User-specific information

**Keep:**
- Error codes
- Messages
- Relative file paths
- Line/column numbers
- Source snippets

### Cross-Machine Stability

**Test:**
- Same input on two different machines yields **identical** JSON output
- Bit-for-bit identical diagnostic objects
- Enables reproducible testing and CI/CD

**Applications:**
- Snapshot testing with `insta`
- Cross-platform CI verification
- Deterministic test suites

---

## Integration with Testing

### Snapshot Testing

**Using `insta` crate:**
```rust
let diagnostics = compile("invalid.atl");
insta::assert_json_snapshot!(diagnostics);
```

**Benefits:**
- Catch unintended diagnostic changes
- Verify diagnostic message quality
- Ensure consistent error codes

### Property Testing

**Invariants to test:**
- All diagnostics have valid error codes
- All diagnostics have non-empty messages
- Line/column numbers are positive
- Length is non-negative
- Ordering rules are followed

---

## Best Practices

### For Compiler Authors

**DO:**
- ✅ Always include precise span information
- ✅ Provide helpful error messages
- ✅ Include `help` text when possible
- ✅ Use `related` locations to show context
- ✅ Follow ordering rules consistently

**DON'T:**
- ❌ Create new error codes without spec update
- ❌ Skip span information
- ❌ Write vague error messages
- ❌ Emit diagnostics with invalid fields

### For AI Agents

**When generating code:**
- Parse JSON diagnostics programmatically
- Extract error code and location
- Use `help` text for fix suggestions
- Follow `related` locations for context

**When reading diagnostics:**
- Check `diag_version` for compatibility
- Look for `help` field for guidance
- Use `related` to understand full error context
- Error codes are stable - safe to pattern match

---

## Diagnostic Examples

### Type Mismatch with Help

```json
{
  "diag_version": 1,
  "level": "error",
  "code": "AT0001",
  "message": "Type mismatch",
  "file": "example.atl",
  "line": 5,
  "column": 14,
  "length": 7,
  "snippet": "let x: number = \"hello\";",
  "label": "expected number, found string",
  "help": "convert the value using str() or change the variable type"
}
```

### Undefined Symbol with Related Location

```json
{
  "diag_version": 1,
  "level": "error",
  "code": "AT0002",
  "message": "Undefined symbol",
  "file": "example.atl",
  "line": 10,
  "column": 9,
  "length": 3,
  "snippet": "let y = foo;",
  "label": "symbol 'foo' not found",
  "related": [
    {
      "file": "example.atl",
      "line": 15,
      "column": 4,
      "length": 3,
      "message": "similar function 'bar' defined here"
    }
  ],
  "help": "did you mean 'bar'?"
}
```

### Unused Variable Warning

```json
{
  "diag_version": 1,
  "level": "warning",
  "code": "AT2001",
  "message": "Unused variable",
  "file": "example.atl",
  "line": 3,
  "column": 5,
  "length": 6,
  "snippet": "let unused = 42;",
  "label": "variable declared but never used",
  "help": "remove the variable or prefix with underscore: _unused"
}
```

---

## Implementation References

**For implementation details, see:**
- `docs/implementation/08-diagnostics.md` - Diagnostic implementation guide
- `Atlas-SPEC.md` - Complete error code listing
- `docs/implementation/07-typechecker.md` - Type error generation
- `docs/implementation/03-lexer.md` - Lexer error generation
- `docs/implementation/04-parser.md` - Parser error generation

---

**Summary:** Atlas diagnostics are precise, consistent, deterministic, and designed for both human and AI consumption. Every diagnostic follows strict rules for schema, ordering, and normalization to ensure reproducibility across all environments.
