//! CLI diagnostic formatting helpers

use atlas_runtime::diagnostic::formatter::{
    enrich_diagnostic, extract_snippet, DiagnosticFormatter,
};
use atlas_runtime::{Diagnostic, DiagnosticLevel};
use std::path::Path;
use termcolor::{ColorChoice, StandardStream, WriteColor};

pub fn emit_diagnostics_stderr(
    diagnostics: &[Diagnostic],
    source: Option<&str>,
    fallback_file: Option<&str>,
) {
    let formatter = DiagnosticFormatter::auto();
    let mut stream = StandardStream::stderr(color_choice());
    emit_all(&formatter, &mut stream, diagnostics, source, fallback_file);
}

/// Emit all diagnostics as a single JSON object: `{"errors": [...], "warnings": [...]}`.
/// This is the structured machine-readable format (B14-P06, D-043).
pub fn emit_diagnostics_json(
    diagnostics: &[Diagnostic],
    source: Option<&str>,
    fallback_file: Option<&str>,
) {
    let prepared: Vec<Diagnostic> = diagnostics
        .iter()
        .map(|d| prepare_diagnostic(d, source, fallback_file))
        .collect();

    let errors: Vec<&Diagnostic> = prepared
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    let warnings: Vec<&Diagnostic> = prepared
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect();

    let errors_json: Vec<serde_json::Value> = errors
        .iter()
        .filter_map(|d| d.to_json_compact().ok())
        .filter_map(|s| serde_json::from_str(&s).ok())
        .collect();
    let warnings_json: Vec<serde_json::Value> = warnings
        .iter()
        .filter_map(|d| d.to_json_compact().ok())
        .filter_map(|s| serde_json::from_str(&s).ok())
        .collect();

    let output = serde_json::json!({
        "errors": errors_json,
        "warnings": warnings_json,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );
}

pub fn emit_diagnostics_stdout(
    diagnostics: &[Diagnostic],
    source: Option<&str>,
    fallback_file: Option<&str>,
) {
    let formatter = DiagnosticFormatter::auto();
    let mut stream = StandardStream::stdout(color_choice());
    emit_all(&formatter, &mut stream, diagnostics, source, fallback_file);
}

pub fn format_diagnostic_plain(
    diagnostic: &Diagnostic,
    source: Option<&str>,
    fallback_file: Option<&str>,
) -> String {
    let formatter = DiagnosticFormatter::plain();
    let prepared = prepare_diagnostic(diagnostic, source, fallback_file);
    formatter.format_to_string(&prepared)
}

fn emit_all(
    formatter: &DiagnosticFormatter,
    stream: &mut impl WriteColor,
    diagnostics: &[Diagnostic],
    source: Option<&str>,
    fallback_file: Option<&str>,
) {
    for diag in diagnostics {
        let prepared = prepare_diagnostic(diag, source, fallback_file);
        let _ = formatter.write_diagnostic(stream, &prepared);
    }
}

fn prepare_diagnostic(
    diagnostic: &Diagnostic,
    source: Option<&str>,
    fallback_file: Option<&str>,
) -> Diagnostic {
    let mut diag = diagnostic.clone();

    if diag.file.is_empty() || diag.file == "<unknown>" || diag.file == "<input>" {
        if let Some(file) = fallback_file {
            diag.file = file.to_string();
        }
    }

    let source_text = resolve_source(&diag, source, fallback_file);
    if let Some(src) = source_text.as_deref() {
        if diag.snippet.is_empty() {
            let enriched = enrich_diagnostic(diag.clone(), src);
            if diag.line == 0 {
                diag.line = enriched.line;
            }
            if diag.snippet.is_empty() {
                diag.snippet = enriched.snippet;
            }
            if diag.snippet.is_empty() {
                if let Some(snippet) = extract_snippet(src, diag.line) {
                    diag.snippet = snippet;
                }
            }
        }

        if diag.length == 0 {
            diag.length = 1;
        }
    }

    // NOTE: help text is NOT auto-injected from the registry here (D-043 rule 3).
    // Help must come from the diagnostic emitting site via error_at_with_code_and_help()
    // or error_with_dynamic_help(). The registry is reserved for `atlas explain`.

    diag
}

fn resolve_source(
    diagnostic: &Diagnostic,
    source: Option<&str>,
    fallback_file: Option<&str>,
) -> Option<String> {
    if let Some(src) = source {
        let file_matches = fallback_file.map(|f| diagnostic.file == f).unwrap_or(false);
        if diagnostic.file.is_empty()
            || diagnostic.file == "<unknown>"
            || diagnostic.file == "<input>"
            || file_matches
        {
            return Some(src.to_string());
        }
    }

    let file = if diagnostic.file.is_empty() || diagnostic.file == "<unknown>" {
        fallback_file?
    } else {
        diagnostic.file.as_str()
    };

    let path = Path::new(file);
    if path.exists() {
        std::fs::read_to_string(path).ok()
    } else {
        None
    }
}

fn color_choice() -> ColorChoice {
    if std::env::var("NO_COLOR").is_ok() {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    }
}
