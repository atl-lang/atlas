//! Source Map v3 integration tests.
//!
//! Tests VLQ encoding/decoding, source map generation, parsing,
//! lookup, and compiler integration.

use atlas_runtime::bytecode::{Bytecode, DebugSpan};
use atlas_runtime::sourcemap::encoder::{
    decode_mappings, MappingEntry, SourceMapBuilder, SourceMapV3,
};
use atlas_runtime::sourcemap::vlq;
use atlas_runtime::sourcemap::{
    generate_from_debug_spans, generate_inline_source_map, generate_source_map, SourceMapOptions,
};
use atlas_runtime::span::Span;

// ═══════════════════════════════════════════════════════════════════════════════
// VLQ Encoding Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_vlq_encode_zero() {
    assert_eq!(vlq::encode(0), "A");
}

#[test]
fn test_vlq_encode_positive_values() {
    // 1 → shifted to 2 → binary 000010 → base64 'C'
    assert_eq!(vlq::encode(1), "C");
    assert_eq!(vlq::encode(2), "E");
    assert_eq!(vlq::encode(3), "G");
}

#[test]
fn test_vlq_encode_negative_values() {
    // -1 → shifted to 3 → binary 000011 → base64 'D'
    assert_eq!(vlq::encode(-1), "D");
    assert_eq!(vlq::encode(-2), "F");
}

#[test]
fn test_vlq_roundtrip_small() {
    for v in -50..=50 {
        let encoded = vlq::encode(v);
        let (decoded, consumed) = vlq::decode(&encoded).unwrap();
        assert_eq!(decoded, v, "roundtrip failed for {v}");
        assert_eq!(consumed, encoded.len());
    }
}

#[test]
fn test_vlq_roundtrip_large() {
    for v in [100, 500, 1000, 5000, 10000, 65535, -100, -500, -65535] {
        let encoded = vlq::encode(v);
        let (decoded, _) = vlq::decode(&encoded).unwrap();
        assert_eq!(decoded, v);
    }
}

#[test]
fn test_vlq_decode_invalid_empty() {
    assert!(vlq::decode("").is_none());
}

#[test]
fn test_vlq_decode_invalid_chars() {
    assert!(vlq::decode("!!!").is_none());
}

#[test]
fn test_vlq_segment_roundtrip() {
    let values = vec![0, 5, -3, 10, 0, -1];
    let segment = vlq::encode_segment(&values);
    let decoded = vlq::decode_segment(&segment).unwrap();
    assert_eq!(decoded, values);
}

#[test]
fn test_vlq_segment_empty() {
    let decoded = vlq::decode_segment("").unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn test_vlq_segment_single_value() {
    let segment = vlq::encode_segment(&[42]);
    let decoded = vlq::decode_segment(&segment).unwrap();
    assert_eq!(decoded, vec![42]);
}

#[test]
fn test_vlq_multibyte_encoding() {
    // Values >= 16 require multiple VLQ digits
    let encoded = vlq::encode(16);
    assert!(encoded.len() > 1, "16 should need multi-byte VLQ");
    let (decoded, _) = vlq::decode(&encoded).unwrap();
    assert_eq!(decoded, 16);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Source Map Builder Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_builder_empty() {
    let mut builder = SourceMapBuilder::new();
    let map = builder.build();
    assert_eq!(map.version, 3);
    assert!(map.sources.is_empty());
    assert!(map.names.is_empty());
    assert!(map.mappings.is_empty());
}

#[test]
fn test_builder_single_source() {
    let mut builder = SourceMapBuilder::new();
    let idx = builder.add_source("main.atlas", None);
    assert_eq!(idx, 0);
    let map = builder.build();
    assert_eq!(map.sources, vec!["main.atlas"]);
    assert!(map.sources_content.is_none());
}

#[test]
fn test_builder_source_with_content() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", Some("let x = 1;".to_string()));
    let map = builder.build();
    assert_eq!(
        map.sources_content,
        Some(vec![Some("let x = 1;".to_string())])
    );
}

#[test]
fn test_builder_duplicate_source() {
    let mut builder = SourceMapBuilder::new();
    let idx1 = builder.add_source("a.atlas", None);
    let idx2 = builder.add_source("a.atlas", None);
    assert_eq!(idx1, idx2);
    let map = builder.build();
    assert_eq!(map.sources.len(), 1);
}

#[test]
fn test_builder_multiple_sources() {
    let mut builder = SourceMapBuilder::new();
    let idx0 = builder.add_source("a.atlas", None);
    let idx1 = builder.add_source("b.atlas", None);
    assert_eq!(idx0, 0);
    assert_eq!(idx1, 1);
    let map = builder.build();
    assert_eq!(map.sources.len(), 2);
}

#[test]
fn test_builder_names() {
    let mut builder = SourceMapBuilder::new();
    let idx = builder.add_name("myVar");
    assert_eq!(idx, 0);
    let idx2 = builder.add_name("myVar"); // duplicate
    assert_eq!(idx2, 0);
    let idx3 = builder.add_name("otherVar");
    assert_eq!(idx3, 1);
    let map = builder.build();
    assert_eq!(map.names, vec!["myVar", "otherVar"]);
}

#[test]
fn test_builder_single_mapping() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    let map = builder.build();
    assert!(!map.mappings.is_empty());

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].generated_line, 0);
    assert_eq!(entries[0].generated_column, 0);
    assert_eq!(entries[0].original_line, 0);
    assert_eq!(entries[0].original_column, 0);
}

#[test]
fn test_builder_multiple_mappings_same_line() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(0, 5, 0, 0, 4, None);
    builder.add_mapping(0, 10, 0, 1, 0, None);
    let map = builder.build();

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].generated_column, 0);
    assert_eq!(entries[1].generated_column, 5);
    assert_eq!(entries[2].generated_column, 10);
}

#[test]
fn test_builder_multiple_lines() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(1, 0, 0, 1, 0, None);
    builder.add_mapping(2, 0, 0, 2, 0, None);
    let map = builder.build();

    // Should have semicolons separating lines
    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].generated_line, 0);
    assert_eq!(entries[1].generated_line, 1);
    assert_eq!(entries[2].generated_line, 2);
}

#[test]
fn test_builder_with_names() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    let name_idx = builder.add_name("myFunc");
    builder.add_mapping(0, 0, 0, 0, 0, Some(name_idx));
    let map = builder.build();

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries[0].name_index, Some(0));
}

#[test]
fn test_builder_set_file() {
    let mut builder = SourceMapBuilder::new();
    builder.set_file("output.atlas.bc");
    let map = builder.build();
    assert_eq!(map.file, Some("output.atlas.bc".to_string()));
}

#[test]
fn test_builder_set_source_root() {
    let mut builder = SourceMapBuilder::new();
    builder.set_source_root("/src/");
    let map = builder.build();
    assert_eq!(map.source_root, Some("/src/".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Source Map JSON Serialization Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_source_map_to_json() {
    let mut builder = SourceMapBuilder::new();
    builder.set_file("out.js");
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    let map = builder.build();

    let json = map.to_json().unwrap();
    assert!(json.contains("\"version\":3"));
    assert!(json.contains("\"file\":\"out.js\""));
    assert!(json.contains("\"sources\":[\"main.atlas\"]"));
}

#[test]
fn test_source_map_roundtrip_json() {
    let mut builder = SourceMapBuilder::new();
    builder.set_file("test.bc");
    builder.add_source("a.atlas", Some("let x = 1;".to_string()));
    builder.add_source("b.atlas", None);
    builder.add_name("x");
    builder.add_mapping(0, 0, 0, 0, 4, Some(0));
    builder.add_mapping(0, 5, 1, 2, 0, None);
    let map = builder.build();

    let json = map.to_json().unwrap();
    let parsed = SourceMapV3::from_json(&json).unwrap();

    assert_eq!(parsed.version, 3);
    assert_eq!(parsed.file, Some("test.bc".to_string()));
    assert_eq!(parsed.sources, vec!["a.atlas", "b.atlas"]);
    assert_eq!(parsed.names, vec!["x"]);
    assert_eq!(parsed.mappings, map.mappings);
}

#[test]
fn test_source_map_pretty_json() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    let map = builder.build();
    let json = map.to_json_pretty().unwrap();
    assert!(json.contains('\n'), "pretty JSON should have newlines");
}

#[test]
fn test_source_map_no_optional_fields() {
    let map = SourceMapV3 {
        version: 3,
        file: None,
        source_root: None,
        sources: vec![],
        sources_content: None,
        names: vec![],
        mappings: String::new(),
    };
    let json = map.to_json().unwrap();
    assert!(!json.contains("file"));
    assert!(!json.contains("sourceRoot"));
    assert!(!json.contains("sourcesContent"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mappings Encoding/Decoding Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_decode_empty_mappings() {
    let entries = decode_mappings("").unwrap();
    assert!(entries.is_empty());
}

#[test]
fn test_encode_decode_roundtrip() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("test.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(0, 10, 0, 2, 5, None);
    builder.add_mapping(1, 0, 0, 5, 0, None);
    let map = builder.build();

    let entries = decode_mappings(&map.mappings).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].generated_column, 0);
    assert_eq!(entries[0].original_line, 0);
    assert_eq!(entries[1].generated_column, 10);
    assert_eq!(entries[1].original_line, 2);
    assert_eq!(entries[1].original_column, 5);
    assert_eq!(entries[2].generated_line, 1);
    assert_eq!(entries[2].generated_column, 0);
    assert_eq!(entries[2].original_line, 5);
}

#[test]
fn test_mappings_with_semicolons() {
    // Multiple lines separated by semicolons
    let mut builder = SourceMapBuilder::new();
    builder.add_source("test.atlas", None);
    // Line 0 has 2 entries, line 1 is empty, line 2 has 1 entry
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(0, 5, 0, 0, 5, None);
    builder.add_mapping(2, 0, 0, 3, 0, None);
    let map = builder.build();

    // Should contain semicolons for line separation
    assert!(map.mappings.contains(';'));

    let entries = decode_mappings(&map.mappings).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[2].generated_line, 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Source Map Lookup Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_lookup_exact_match() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 5, 10, None);
    let map = builder.build();

    let loc = map.lookup(0, 0).unwrap();
    assert_eq!(loc.source, "main.atlas");
    assert_eq!(loc.line, 5);
    assert_eq!(loc.column, 10);
}

#[test]
fn test_lookup_closest_column() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    builder.add_mapping(0, 10, 0, 1, 0, None);
    let map = builder.build();

    // Column 5 is between 0 and 10; should match column 0's mapping
    let loc = map.lookup(0, 5).unwrap();
    assert_eq!(loc.line, 0);
}

#[test]
fn test_lookup_no_match() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 5, 0, 0, 0, None);
    let map = builder.build();

    // Column 3 is before the first mapping on line 0
    assert!(map.lookup(0, 3).is_none());
}

#[test]
fn test_lookup_with_name() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    let name_idx = builder.add_name("foo");
    builder.add_mapping(0, 0, 0, 3, 4, Some(name_idx));
    let map = builder.build();

    let loc = map.lookup(0, 0).unwrap();
    assert_eq!(loc.name, Some("foo".to_string()));
}

#[test]
fn test_lookup_wrong_line() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    let map = builder.build();

    assert!(map.lookup(1, 0).is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Source Map Generation from Bytecode Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn make_bytecode(spans: Vec<(usize, usize, usize)>) -> Bytecode {
    Bytecode {
        instructions: vec![0; spans.last().map(|(o, _, _)| o + 1).unwrap_or(0)],
        constants: Vec::new(),
        debug_info: spans
            .into_iter()
            .map(|(offset, start, end)| DebugSpan {
                instruction_offset: offset,
                span: Span::new(start, end),
            })
            .collect(),
    }
}

#[test]
fn test_generate_simple_program() {
    let source = "let x = 1;\nlet y = 2;\n";
    let bytecode = make_bytecode(vec![(0, 0, 10), (5, 11, 21)]);
    let options = SourceMapOptions::default();

    let map = generate_source_map(&bytecode, "main.atlas", Some(source), &options);
    assert_eq!(map.version, 3);
    assert_eq!(map.sources, vec!["main.atlas"]);

    let entries = map.decode_mappings().unwrap();
    assert!(!entries.is_empty());

    // First entry should map to line 0
    assert_eq!(entries[0].original_line, 0);
    // Second entry should map to line 1
    let last = entries.last().unwrap();
    assert_eq!(last.original_line, 1);
}

#[test]
fn test_generate_skips_dummy_spans() {
    let bytecode = make_bytecode(vec![(0, 0, 0), (1, 5, 10)]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some("hello world"), &options);

    let entries = map.decode_mappings().unwrap();
    // The dummy span (0,0) should be skipped
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].generated_column, 1);
}

#[test]
fn test_generate_with_inlined_sources() {
    let source = "let x = 42;";
    let bytecode = make_bytecode(vec![(0, 0, 11)]);
    let options = SourceMapOptions {
        include_sources: true,
        ..Default::default()
    };

    let map = generate_source_map(&bytecode, "main.atlas", Some(source), &options);
    assert_eq!(
        map.sources_content,
        Some(vec![Some("let x = 42;".to_string())])
    );
}

#[test]
fn test_generate_without_source_text() {
    let bytecode = make_bytecode(vec![(0, 5, 10)]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "main.atlas", None, &options);

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 1);
    // Without source text, positions are computed from offset 0 only
    assert_eq!(entries[0].original_line, 0);
}

#[test]
fn test_generate_with_file_option() {
    let bytecode = make_bytecode(vec![(0, 0, 5)]);
    let options = SourceMapOptions {
        file: Some("output.bc".to_string()),
        ..Default::default()
    };
    let map = generate_source_map(&bytecode, "main.atlas", Some("hello"), &options);
    assert_eq!(map.file, Some("output.bc".to_string()));
}

#[test]
fn test_generate_with_source_root() {
    let bytecode = make_bytecode(vec![(0, 0, 5)]);
    let options = SourceMapOptions {
        source_root: Some("/src/".to_string()),
        ..Default::default()
    };
    let map = generate_source_map(&bytecode, "main.atlas", Some("hello"), &options);
    assert_eq!(map.source_root, Some("/src/".to_string()));
}

#[test]
fn test_generate_removes_redundant_entries() {
    // Multiple instructions mapping to the same source position
    let bytecode = make_bytecode(vec![(0, 0, 10), (1, 0, 10), (2, 0, 10), (5, 11, 20)]);
    let source = "let x = 1;\nlet y = 2;\n";
    let options = SourceMapOptions::default();

    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);
    let entries = map.decode_mappings().unwrap();

    // Redundant entries with same original position should be deduped
    assert!(entries.len() <= 3, "Should remove redundant mappings");
}

#[test]
fn test_generate_from_debug_spans_direct() {
    let spans = vec![
        DebugSpan {
            instruction_offset: 0,
            span: Span::new(0, 5),
        },
        DebugSpan {
            instruction_offset: 3,
            span: Span::new(6, 11),
        },
    ];
    let options = SourceMapOptions::default();
    let map = generate_from_debug_spans(&spans, "test.atlas", Some("hello\nworld"), &options);

    assert_eq!(map.sources, vec!["test.atlas"]);
    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_generate_empty_bytecode() {
    let bytecode = Bytecode {
        instructions: Vec::new(),
        constants: Vec::new(),
        debug_info: Vec::new(),
    };
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "empty.atlas", Some(""), &options);

    assert_eq!(map.version, 3);
    assert!(map.decode_mappings().unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Inline Source Map Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_inline_source_map() {
    let mut builder = SourceMapBuilder::new();
    builder.add_source("main.atlas", None);
    builder.add_mapping(0, 0, 0, 0, 0, None);
    let map = builder.build();

    let inline = generate_inline_source_map(&map).unwrap();
    assert!(inline.starts_with("//# sourceMappingURL=data:application/json;base64,"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compiler Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn compile_source(source: &str) -> Bytecode {
    let mut lexer = atlas_runtime::lexer::Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = atlas_runtime::parser::Parser::new(tokens);
    let (program, _) = parser.parse();
    let mut compiler = atlas_runtime::compiler::Compiler::new();
    compiler.compile(&program).unwrap()
}

#[test]
fn test_compiler_generates_source_map() {
    let source = "let x = 42;";
    let bytecode = compile_source(source);

    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    assert_eq!(map.version, 3);
    assert_eq!(map.sources, vec!["test.atlas"]);
    assert!(!map.decode_mappings().unwrap().is_empty());
}

#[test]
fn test_compiler_source_map_with_function() {
    let source = "fn add(a, b) {\n  return a + b;\n}\nlet result = add(1, 2);";
    let bytecode = compile_source(source);

    let options = SourceMapOptions {
        include_sources: true,
        file: Some("add.atlas.bc".to_string()),
        ..Default::default()
    };
    let map = generate_source_map(&bytecode, "add.atlas", Some(source), &options);

    assert_eq!(map.file, Some("add.atlas.bc".to_string()));
    assert!(map.sources_content.is_some());
    assert!(!map.decode_mappings().unwrap().is_empty());
}

#[test]
fn test_compiler_source_map_lookup() {
    let source = "let x = 1;\nlet y = 2;\nlet z = x + y;";
    let bytecode = compile_source(source);

    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    let entries = map.decode_mappings().unwrap();
    assert!(
        entries.len() >= 3,
        "expected >=3 mappings, got {}",
        entries.len()
    );

    for entry in &entries {
        assert_eq!(entry.source_index, 0);
    }
}

#[test]
fn test_compiler_source_map_json_roundtrip() {
    let source = "let x = 42;\nprint(x);";
    let bytecode = compile_source(source);

    let options = SourceMapOptions {
        include_sources: true,
        ..Default::default()
    };
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);
    let json = map.to_json().unwrap();
    let parsed = SourceMapV3::from_json(&json).unwrap();

    assert_eq!(parsed.version, map.version);
    assert_eq!(parsed.sources, map.sources);
    assert_eq!(parsed.mappings, map.mappings);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Debugger Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_debugger_source_map_from_bytecode() {
    let source = "let a = 1;\nlet b = 2;";
    let bytecode = compile_source(source);

    let options = SourceMapOptions::default();
    let v3_map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    let dbg_map = atlas_runtime::debugger::source_map::SourceMap::from_debug_spans(
        &bytecode.debug_info,
        "test.atlas",
        Some(source),
    );

    assert!(!v3_map.decode_mappings().unwrap().is_empty());
    assert!(!dbg_map.is_empty());
}

#[test]
fn test_source_map_stack_trace_lookup() {
    let source = "fn greet() {\n  print(\"hello\");\n}\ngreet();";
    let bytecode = compile_source(source);

    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    let entries = map.decode_mappings().unwrap();
    if !entries.is_empty() {
        let first = &entries[0];
        let loc = map.lookup(first.generated_line, first.generated_column);
        assert!(loc.is_some());
        assert_eq!(loc.unwrap().source, "test.atlas");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_source_map_multiline_source() {
    let source = "let a = 1;\nlet b = 2;\nlet c = 3;\nlet d = 4;\nlet e = 5;";
    let bytecode = make_bytecode(vec![
        (0, 0, 10),
        (3, 11, 21),
        (6, 22, 32),
        (9, 33, 43),
        (12, 44, 54),
    ]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "test.atlas", Some(source), &options);

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 5);
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.original_line, i as u32, "line mismatch at entry {i}");
    }
}

#[test]
fn test_source_map_unicode_source() {
    let source = "let emoji = \"🎉\";\nlet kanji = \"漢字\";";
    let bytecode = make_bytecode(vec![(0, 0, 20), (5, 21, 40)]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "unicode.atlas", Some(source), &options);

    assert_eq!(map.sources, vec!["unicode.atlas"]);
    assert!(!map.decode_mappings().unwrap().is_empty());
}

#[test]
fn test_source_map_very_large_offsets() {
    let bytecode = make_bytecode(vec![(0, 0, 10), (10000, 50000, 50010)]);
    let options = SourceMapOptions::default();
    let map = generate_source_map(&bytecode, "big.atlas", None, &options);

    let entries = map.decode_mappings().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].generated_column, 10000);
}

#[test]
fn test_mapping_entry_equality() {
    let a = MappingEntry {
        generated_line: 0,
        generated_column: 0,
        source_index: 0,
        original_line: 1,
        original_column: 5,
        name_index: None,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_source_map_options_default() {
    let opts = SourceMapOptions::default();
    assert!(opts.file.is_none());
    assert!(opts.source_root.is_none());
    assert!(!opts.include_sources);
}

#[test]
fn test_source_map_options_new() {
    let opts = SourceMapOptions::new();
    assert!(opts.file.is_none());
}
