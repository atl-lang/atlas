# Diagnostic Snapshots

This directory contains golden test fixtures for diagnostic output.

## Structure

Each test case consists of two files:
- `<CODE>_<description>.atl` - Atlas source code that triggers the diagnostic
- `<CODE>_<description>.json` - Expected normalized diagnostic JSON output

## Error Codes

### Errors (AT)
- `AT0001` - Type mismatch
- `AT0002` - Undefined variable
- `AT0003` - Invalid operation
- `AT0004` - Function not found
- `AT0005` - Argument count mismatch

### Warnings (AW)
- `AW0001` - Unused variable

## Normalization

All snapshot files use normalized paths (filename only, no absolute paths).
This ensures tests are stable across different machines and environments.

## Adding New Snapshots

1. Create the `.atl` file with code that triggers the diagnostic
2. Create the `.json` file with the expected normalized output
3. Ensure `diag_version` is set to the current version
4. Use relative or filename-only paths in the `file` field
