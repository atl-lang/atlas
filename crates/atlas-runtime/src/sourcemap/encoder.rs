//! Source Map v3 encoder — generates standard JSON source maps.
//!
//! Produces source maps compliant with the Source Map v3 specification,
//! mapping compiled bytecode positions back to original Atlas source code.

use serde::{Deserialize, Serialize};

use super::vlq;

/// A single mapping entry: generated position → original position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingEntry {
    /// Generated (bytecode) line (0-based).
    pub generated_line: u32,
    /// Generated (bytecode) column (0-based).
    pub generated_column: u32,
    /// Index into `sources` array.
    pub source_index: u32,
    /// Original source line (0-based).
    pub original_line: u32,
    /// Original source column (0-based).
    pub original_column: u32,
    /// Optional index into `names` array.
    pub name_index: Option<u32>,
}

/// Source Map v3 JSON structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceMapV3 {
    pub version: u32,
    /// Output file name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Source root prefix.
    #[serde(rename = "sourceRoot", skip_serializing_if = "Option::is_none")]
    pub source_root: Option<String>,
    /// Original source file names.
    pub sources: Vec<String>,
    /// Optional inline source content (parallel to `sources`).
    #[serde(rename = "sourcesContent", skip_serializing_if = "Option::is_none")]
    pub sources_content: Option<Vec<Option<String>>>,
    /// Identifier names referenced in mappings.
    pub names: Vec<String>,
    /// VLQ-encoded mappings string.
    pub mappings: String,
}

impl SourceMapV3 {
    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize to pretty JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Decode the mappings string into individual entries.
    pub fn decode_mappings(&self) -> Option<Vec<MappingEntry>> {
        decode_mappings(&self.mappings)
    }

    /// Look up the original source location for a generated position.
    ///
    /// Uses binary search on decoded mappings for efficient lookup.
    pub fn lookup(&self, generated_line: u32, generated_column: u32) -> Option<OriginalLocation> {
        let mappings = self.decode_mappings()?;
        // Find the best match: same line, closest column ≤ requested
        let mut best: Option<&MappingEntry> = None;
        for entry in &mappings {
            if entry.generated_line == generated_line && entry.generated_column <= generated_column
            {
                match best {
                    Some(prev) if entry.generated_column > prev.generated_column => {
                        best = Some(entry);
                    }
                    None => best = Some(entry),
                    _ => {}
                }
            }
        }
        let entry = best?;
        Some(OriginalLocation {
            source: self.sources.get(entry.source_index as usize)?.clone(),
            line: entry.original_line,
            column: entry.original_column,
            name: entry
                .name_index
                .and_then(|i| self.names.get(i as usize).cloned()),
        })
    }
}

/// Result of a source map lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginalLocation {
    /// Source file name.
    pub source: String,
    /// Original line (0-based).
    pub line: u32,
    /// Original column (0-based).
    pub column: u32,
    /// Optional identifier name.
    pub name: Option<String>,
}

/// Builder for constructing source maps incrementally.
pub struct SourceMapBuilder {
    file: Option<String>,
    source_root: Option<String>,
    sources: Vec<String>,
    sources_content: Vec<Option<String>>,
    names: Vec<String>,
    mappings: Vec<MappingEntry>,
    /// Cache: source name → index
    source_index: std::collections::HashMap<String, u32>,
    /// Cache: name → index
    name_index: std::collections::HashMap<String, u32>,
}

impl SourceMapBuilder {
    pub fn new() -> Self {
        Self {
            file: None,
            source_root: None,
            sources: Vec::new(),
            sources_content: Vec::new(),
            names: Vec::new(),
            mappings: Vec::new(),
            source_index: std::collections::HashMap::new(),
            name_index: std::collections::HashMap::new(),
        }
    }

    pub fn set_file(&mut self, file: impl Into<String>) -> &mut Self {
        self.file = Some(file.into());
        self
    }

    pub fn set_source_root(&mut self, root: impl Into<String>) -> &mut Self {
        self.source_root = Some(root.into());
        self
    }

    /// Add a source file and return its index.
    pub fn add_source(&mut self, name: impl Into<String>, content: Option<String>) -> u32 {
        let name = name.into();
        if let Some(&idx) = self.source_index.get(&name) {
            return idx;
        }
        let idx = self.sources.len() as u32;
        self.source_index.insert(name.clone(), idx);
        self.sources.push(name);
        self.sources_content.push(content);
        idx
    }

    /// Add an identifier name and return its index.
    pub fn add_name(&mut self, name: impl Into<String>) -> u32 {
        let name = name.into();
        if let Some(&idx) = self.name_index.get(&name) {
            return idx;
        }
        let idx = self.names.len() as u32;
        self.name_index.insert(name.clone(), idx);
        self.names.push(name);
        idx
    }

    /// Add a mapping entry.
    pub fn add_mapping(
        &mut self,
        generated_line: u32,
        generated_column: u32,
        source_index: u32,
        original_line: u32,
        original_column: u32,
        name_index: Option<u32>,
    ) -> &mut Self {
        self.mappings.push(MappingEntry {
            generated_line,
            generated_column,
            source_index,
            original_line,
            original_column,
            name_index,
        });
        self
    }

    /// Build the final source map.
    pub fn build(&mut self) -> SourceMapV3 {
        // Sort mappings by generated position
        self.mappings
            .sort_by_key(|m| (m.generated_line, m.generated_column));

        let has_content = self.sources_content.iter().any(|c| c.is_some());

        SourceMapV3 {
            version: 3,
            file: self.file.clone(),
            source_root: self.source_root.clone(),
            sources: self.sources.clone(),
            sources_content: if has_content {
                Some(self.sources_content.clone())
            } else {
                None
            },
            names: self.names.clone(),
            mappings: encode_mappings(&self.mappings),
        }
    }
}

impl Default for SourceMapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode mapping entries into the VLQ mappings string.
fn encode_mappings(entries: &[MappingEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut prev_generated_line: u32 = 0;
    let mut prev_generated_column: u32 = 0;
    let mut prev_source: u32 = 0;
    let mut prev_original_line: u32 = 0;
    let mut prev_original_column: u32 = 0;
    let mut prev_name: u32 = 0;
    let mut first_in_line = true;

    for entry in entries {
        // Emit semicolons for skipped lines
        while prev_generated_line < entry.generated_line {
            result.push(';');
            prev_generated_line += 1;
            prev_generated_column = 0;
            first_in_line = true;
        }

        if !first_in_line {
            result.push(',');
        }
        first_in_line = false;

        // Field 1: generated column (relative to previous in same line)
        let col_diff = entry.generated_column as i64 - prev_generated_column as i64;
        result.push_str(&vlq::encode(col_diff));
        prev_generated_column = entry.generated_column;

        // Field 2: source index (relative)
        let src_diff = entry.source_index as i64 - prev_source as i64;
        result.push_str(&vlq::encode(src_diff));
        prev_source = entry.source_index;

        // Field 3: original line (relative)
        let line_diff = entry.original_line as i64 - prev_original_line as i64;
        result.push_str(&vlq::encode(line_diff));
        prev_original_line = entry.original_line;

        // Field 4: original column (relative)
        let ocol_diff = entry.original_column as i64 - prev_original_column as i64;
        result.push_str(&vlq::encode(ocol_diff));
        prev_original_column = entry.original_column;

        // Field 5: name index (optional, relative)
        if let Some(name_idx) = entry.name_index {
            let name_diff = name_idx as i64 - prev_name as i64;
            result.push_str(&vlq::encode(name_diff));
            prev_name = name_idx;
        }
    }

    result
}

/// Decode VLQ mappings string into mapping entries.
pub fn decode_mappings(mappings: &str) -> Option<Vec<MappingEntry>> {
    let mut entries = Vec::new();
    let mut generated_line: u32 = 0;
    let mut generated_column: u32;
    let mut source_index: u32 = 0;
    let mut original_line: u32 = 0;
    let mut original_column: u32 = 0;
    let mut name_index: u32 = 0;

    for line_segment in mappings.split(';') {
        generated_column = 0;

        if !line_segment.is_empty() {
            for segment in line_segment.split(',') {
                if segment.is_empty() {
                    continue;
                }
                let values = vlq::decode_segment(segment)?;
                if values.len() < 4 {
                    // Segments with only 1 field (generated column only) are valid but
                    // we skip them as they don't map to an original position
                    if values.len() == 1 {
                        generated_column = (generated_column as i64 + values[0]) as u32;
                        continue;
                    }
                    return None;
                }

                generated_column = (generated_column as i64 + values[0]) as u32;
                source_index = (source_index as i64 + values[1]) as u32;
                original_line = (original_line as i64 + values[2]) as u32;
                original_column = (original_column as i64 + values[3]) as u32;

                let name = if values.len() >= 5 {
                    name_index = (name_index as i64 + values[4]) as u32;
                    Some(name_index)
                } else {
                    None
                };

                entries.push(MappingEntry {
                    generated_line,
                    generated_column,
                    source_index,
                    original_line,
                    original_column,
                    name_index: name,
                });
            }
        }

        generated_line += 1;
    }

    Some(entries)
}
