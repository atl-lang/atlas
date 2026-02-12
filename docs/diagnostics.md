# Atlas Diagnostics

## Overview
All errors and warnings are represented as `Diagnostic` objects. This ensures consistency across compiler, interpreter, and VM.

## Diagnostic Schema
Fields (required unless noted):
- `diag_version`: diagnostic schema version (integer)
- `level`: `error` | `warning`
- `code`: short string (e.g., `AT0001`)
- `message`: short summary
- `file`: file path
- `line`: 1-based
- `column`: 1-based
- `length`: length of error span
- `snippet`: source line string
- `label`: short label for caret range
- `notes`: array of strings (optional)
- `related`: array of secondary locations (optional)
- `help`: suggested fix (optional)

## Human Format
```
error[AT0001]: Type mismatch
  --> path/to/file.atl:12:9
   |
12 | let x: number = "hello";
   |         ^^^^^ expected number, found string
   |
help: convert the value to number or change the variable type
```

## JSON Format
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

## Error Code Policy
- Every diagnostic must use a defined error code from `Atlas-SPEC.md`.
- New error codes must be added to the spec first.

## Emission Policy
- Stop after 25 compile-time errors.
- Continue emitting warnings even if errors exist.
- REPL reports the first error per input.
- Use warning codes from `docs/warnings.md`.
- Ordering: errors are emitted before warnings for the same input.
- Ordering within same level: sort by file, then line, then column.
