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

/// Convert an Atlas diagnostic to an LSP diagnostic
pub fn diagnostic_to_lsp(diag: &Diagnostic) -> lsp_types::Diagnostic {
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
        }),
        code: Some(lsp_types::NumberOrString::String(diag.code.clone())),
        source: Some("atlas".to_string()),
        message: diag.message.clone(),
        ..Default::default()
    }
}
