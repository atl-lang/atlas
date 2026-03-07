//! Type conversions between Atlas and LSP types

use atlas_runtime::{Diagnostic, DiagnosticLevel};
use tower_lsp::lsp_types;

/// Convert a byte offset to an LSP Position (line/column)
pub fn offset_to_position(text: &str, offset: usize) -> lsp_types::Position {
    let mut line = 0;
    let mut col = 0;
    let mut current_offset = 0;

    for ch in text.chars() {
        if current_offset >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        current_offset += ch.len_utf8();
    }

    lsp_types::Position {
        line: line as u32,
        character: col as u32,
    }
}

/// Convert an LSP Position (line/column) to a byte offset
pub fn position_to_offset(text: &str, position: lsp_types::Position) -> usize {
    let mut current_line = 0;
    let mut current_col = 0;
    let mut offset = 0;

    for ch in text.chars() {
        if current_line == position.line as usize && current_col == position.character as usize {
            return offset;
        }

        if ch == '\n' {
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }

        offset += ch.len_utf8();
    }

    offset
}

/// Unused-symbol warning codes that should render as dimmed (UNNECESSARY) in editors,
/// not as red squiggles. Matches D-031.
const UNUSED_CODES: &[&str] = &["AT2001", "AT2004", "AT2008"];

/// Convert an Atlas diagnostic to an LSP diagnostic.
///
/// Forwards all diagnostic data per D-031:
/// - `notes` + `help` appended to message so editors display them
/// - `related` → `related_information` with proper LSP locations
/// - Unused-symbol codes tagged `UNNECESSARY` (dim, not red)
/// - `DiagnosticLevel::Hint` → `DiagnosticSeverity::HINT`
pub fn diagnostic_to_lsp(diag: &Diagnostic) -> lsp_types::Diagnostic {
    // Build full message: main + notes + help
    let mut message = diag.message.clone();
    for note in &diag.notes {
        message.push_str(&format!("\nnote: {note}"));
    }
    if let Some(help) = &diag.help {
        message.push_str(&format!("\nhelp: {help}"));
    }

    // Convert related locations → LSP DiagnosticRelatedInformation
    let related_information = if diag.related.is_empty() {
        None
    } else {
        Some(
            diag.related
                .iter()
                .map(|r| lsp_types::DiagnosticRelatedInformation {
                    location: lsp_types::Location {
                        uri: file_path_to_uri(&r.file),
                        range: lsp_types::Range {
                            start: lsp_types::Position {
                                line: (r.line.saturating_sub(1)) as u32,
                                character: (r.column.saturating_sub(1)) as u32,
                            },
                            end: lsp_types::Position {
                                line: (r.line.saturating_sub(1)) as u32,
                                character: (r.column.saturating_sub(1) + r.length) as u32,
                            },
                        },
                    },
                    message: r.message.clone(),
                })
                .collect(),
        )
    };

    // Tag unused-symbol warnings as UNNECESSARY (dim) not ERROR (red)
    let tags = if UNUSED_CODES.contains(&diag.code.as_str()) {
        Some(vec![lsp_types::DiagnosticTag::UNNECESSARY])
    } else {
        None
    };

    lsp_types::Diagnostic {
        range: lsp_types::Range {
            start: lsp_types::Position {
                line: (diag.line.saturating_sub(1)) as u32,
                character: (diag.column.saturating_sub(1)) as u32,
            },
            end: lsp_types::Position {
                line: (diag.line.saturating_sub(1)) as u32,
                character: (diag.column.saturating_sub(1) + diag.length) as u32,
            },
        },
        severity: Some(match diag.level {
            DiagnosticLevel::Error => lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticLevel::Warning => lsp_types::DiagnosticSeverity::WARNING,
            DiagnosticLevel::Hint => lsp_types::DiagnosticSeverity::HINT,
        }),
        code: Some(lsp_types::NumberOrString::String(diag.code.clone())),
        source: Some("atlas".to_string()),
        message,
        related_information,
        tags,
        ..Default::default()
    }
}

/// Convert a file path to an LSP URI.
/// Falls back to a placeholder for synthetic paths like `<input>` or `<unknown>`.
fn file_path_to_uri(path: &str) -> lsp_types::Url {
    if path.starts_with('<') {
        placeholder_uri()
    } else {
        lsp_types::Url::from_file_path(path).unwrap_or_else(|_| placeholder_uri())
    }
}

/// Placeholder URI for synthetic/unknown file paths.
fn placeholder_uri() -> lsp_types::Url {
    // "file:///unknown" is a statically valid URI — parse cannot fail
    lsp_types::Url::parse("file:///unknown").expect("hardcoded valid URI")
}
