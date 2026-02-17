# Atlas Source Maps

Source maps enable mapping compiled bytecode positions back to original Atlas source code, supporting accurate debugging, error reporting, and development tool integration.

## Overview

Atlas implements the [Source Map v3 specification](https://sourcemaps.info/spec.html), producing standard JSON source maps with VLQ-encoded mappings. Source maps are generated from the compiler's debug info (instruction offset to source span mappings) that are already embedded during compilation.

## Architecture

```
sourcemap/
  mod.rs       — Generation API, compiler integration, inline source maps
  encoder.rs   — SourceMapV3 struct, SourceMapBuilder, VLQ mapping encode/decode
  vlq.rs       — Base64-VLQ encoding/decoding primitives
```

### Module Relationships

- **`vlq`** — Low-level: encodes/decodes signed integers as base64-VLQ strings
- **`encoder`** — Mid-level: builds Source Map v3 JSON with mappings, sources, names
- **`mod`** — High-level: generates source maps from compiled bytecode, integrates with debugger

## Usage

### Generate a Source Map from Bytecode

```rust
use atlas_runtime::sourcemap::{generate_source_map, SourceMapOptions};

let options = SourceMapOptions {
    file: Some("output.atlas.bc".to_string()),
    source_root: Some("/src/".to_string()),
    include_sources: true,
};
let source_map = generate_source_map(&bytecode, "main.atlas", Some(source_code), &options);
let json = source_map.to_json().unwrap();
```

### Look Up Original Location

```rust
// Given a bytecode offset, find the original source location
let location = source_map.lookup(0, instruction_offset as u32);
if let Some(loc) = location {
    println!("{}:{}:{}", loc.source, loc.line + 1, loc.column + 1);
}
```

### Build Source Maps Incrementally

```rust
use atlas_runtime::sourcemap::SourceMapBuilder;

let mut builder = SourceMapBuilder::new();
builder.set_file("output.bc");
let src_idx = builder.add_source("main.atlas", Some(source.to_string()));
let name_idx = builder.add_name("myFunction");
builder.add_mapping(0, 0, src_idx, 5, 10, Some(name_idx));
let map = builder.build();
```

### Inline Source Maps

```rust
use atlas_runtime::sourcemap::generate_inline_source_map;

let comment = generate_inline_source_map(&source_map).unwrap();
// Returns: //# sourceMappingURL=data:application/json;base64,...
```

## Source Map v3 Format

Generated source maps follow the standard JSON format:

```json
{
  "version": 3,
  "file": "output.atlas.bc",
  "sourceRoot": "/src/",
  "sources": ["main.atlas"],
  "sourcesContent": ["let x = 42;\nprint(x);"],
  "names": ["x"],
  "mappings": "AAAA,KACA"
}
```

### Fields

| Field | Description |
|-------|-------------|
| `version` | Always `3` |
| `file` | Output file name (optional) |
| `sourceRoot` | Root path prefix for sources (optional) |
| `sources` | Array of original source file names |
| `sourcesContent` | Inline source text, parallel to `sources` (optional) |
| `names` | Identifier names referenced in mappings |
| `mappings` | VLQ-encoded position mappings |

### VLQ Encoding

Each mapping segment encodes 4-5 relative values:
1. Generated column (relative to previous in same line)
2. Source file index (relative to previous)
3. Original line (relative to previous)
4. Original column (relative to previous)
5. Name index (optional, relative to previous)

Segments on the same generated line are separated by commas. Lines are separated by semicolons.

## Integration Points

### Compiler

The compiler already embeds `DebugSpan` entries mapping each bytecode instruction to its source `Span`. The source map generator converts these into v3 format:

- Instruction offset becomes the "generated column" (bytecode is a flat stream on line 0)
- Source spans are converted to 0-based line/column using the original source text
- Redundant mappings (same original position) are automatically deduplicated

### Debugger

The existing `debugger::source_map::SourceMap` provides offset-to-location lookup for the debugger. The v3 `SourceMapV3` provides the same data in the standard interchange format for external tools.

Both systems work from the same `debug_info` in the compiled bytecode:

```rust
// Internal debugger use
let dbg_map = SourceMap::from_debug_spans(&bytecode.debug_info, "file.atlas", Some(source));

// External tool use (standard v3 format)
let v3_map = generate_source_map(&bytecode, "file.atlas", Some(source), &options);
```

## Options

| Option | Default | Description |
|--------|---------|-------------|
| `file` | `None` | Name of the output file |
| `source_root` | `None` | Root prefix for source paths |
| `include_sources` | `false` | Embed source content inline |

## Testing

57 tests cover:
- VLQ encoding/decoding (roundtrip, edge cases, invalid input)
- Builder API (sources, names, mappings, options)
- JSON serialization/deserialization roundtrip
- Mapping encode/decode roundtrip
- Source map lookup (exact, closest, missing)
- Compiler integration (simple programs, functions, multi-statement)
- Debugger integration (v3 and internal source map parity)
- Edge cases (unicode, large offsets, empty bytecode, redundant entries)
