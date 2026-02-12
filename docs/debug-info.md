# Atlas Debug Info

## Purpose
Ensure VM and bytecode errors can map back to source spans with high fidelity.

## Requirements
- Each bytecode instruction optionally carries a `Span` reference.
- Spans refer to the original source file and line/column.
- Debug info is preserved in `.atb` files.

## Policy
- Debug info is enabled by default in v0.1.
- Optional flag may strip debug info for size in future releases.

## Format (v0.1)
- Instruction stream stores a parallel `Vec<SpanId>`.
- `SpanId` indexes a table of unique spans.
- Spans include file, line, column, length.
