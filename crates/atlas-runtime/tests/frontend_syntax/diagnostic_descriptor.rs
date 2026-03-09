//! Tests for DiagnosticDescriptor + DiagnosticBuilder (B17-P01)

use atlas_runtime::{
    diagnostic::{
        descriptor::{DiagnosticDescriptor, DiagnosticDomain},
        DiagnosticLevel,
    },
    span::{intern_file, register_source, Span},
};

fn test_span() -> Span {
    let fid = intern_file("<desc-test>");
    register_source(fid, "let x: str = 42;");
    Span {
        start: 0,
        end: 3,
        file: fid,
    }
}

static DESC_BASIC: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT9001",
    level: DiagnosticLevel::Error,
    title: "Type mismatch",
    message_template: "expected {expected}, found {found}",
    static_help: None,
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

static DESC_WITH_STATIC: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT9002",
    level: DiagnosticLevel::Error,
    title: "Undefined symbol",
    message_template: "`{name}` is not defined",
    static_help: Some("check your imports"),
    static_note: Some("symbols must be declared before use"),
    domain: DiagnosticDomain::Typechecker,
};

static DESC_WARNING: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AW9001",
    level: DiagnosticLevel::Warning,
    title: "Unused variable",
    message_template: "variable `{name}` is never used",
    static_help: None,
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

static DESC_NO_HOLES: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT9003",
    level: DiagnosticLevel::Error,
    title: "Syntax error",
    message_template: "unexpected end of file",
    static_help: Some("make sure all blocks are closed"),
    static_note: None,
    domain: DiagnosticDomain::Parser,
};

static DESC_MULTI_HOLES: DiagnosticDescriptor = DiagnosticDescriptor {
    code: "AT9004",
    level: DiagnosticLevel::Error,
    title: "Arity mismatch",
    message_template: "function `{name}` expects {expected} args, found {found}",
    static_help: None,
    static_note: None,
    domain: DiagnosticDomain::Typechecker,
};

// ── template substitution ─────────────────────────────────────────────────────

#[test]
fn descriptor_substitutes_named_holes() {
    let diag = DESC_BASIC
        .emit(test_span())
        .arg("expected", "str")
        .arg("found", "int")
        .build();
    assert_eq!(diag.message, "expected str, found int");
    assert_eq!(diag.code, "AT9001");
}

#[test]
fn descriptor_leaves_missing_hole_as_is() {
    let diag = DESC_BASIC
        .emit(test_span())
        .arg("expected", "str")
        // "found" intentionally omitted
        .build();
    assert!(
        diag.message.contains("{found}"),
        "unfilled hole should remain"
    );
}

#[test]
fn descriptor_substitutes_multiple_holes() {
    let diag = DESC_MULTI_HOLES
        .emit(test_span())
        .arg("name", "foo")
        .arg("expected", "2")
        .arg("found", "3")
        .build();
    assert_eq!(diag.message, "function `foo` expects 2 args, found 3");
}

#[test]
fn descriptor_no_holes_message_unchanged() {
    let diag = DESC_NO_HOLES.emit(test_span()).build();
    assert_eq!(diag.message, "unexpected end of file");
}

// ── static_help / static_note prepend ────────────────────────────────────────

#[test]
fn descriptor_prepends_static_help() {
    let diag = DESC_WITH_STATIC
        .emit(test_span())
        .arg("name", "foo")
        .with_help("try `use foo;`")
        .build();
    assert_eq!(diag.help.len(), 2);
    assert_eq!(diag.help[0], "check your imports");
    assert_eq!(diag.help[1], "try `use foo;`");
}

#[test]
fn descriptor_prepends_static_note() {
    let diag = DESC_WITH_STATIC
        .emit(test_span())
        .arg("name", "bar")
        .with_note("declared at line 5")
        .build();
    assert_eq!(diag.notes.len(), 2);
    assert_eq!(diag.notes[0], "symbols must be declared before use");
    assert_eq!(diag.notes[1], "declared at line 5");
}

#[test]
fn descriptor_no_static_help_note_when_none() {
    let diag = DESC_BASIC
        .emit(test_span())
        .arg("expected", "int")
        .arg("found", "str")
        .build();
    assert!(diag.help.is_empty());
    assert!(diag.notes.is_empty());
}

// ── extra help / notes merge ──────────────────────────────────────────────────

#[test]
fn descriptor_multiple_extra_helps_appended() {
    let diag = DESC_BASIC
        .emit(test_span())
        .arg("expected", "int")
        .arg("found", "str")
        .with_help("help line 1")
        .with_help("help line 2")
        .build();
    assert_eq!(diag.help, vec!["help line 1", "help line 2"]);
}

#[test]
fn descriptor_multiple_extra_notes_appended() {
    let diag = DESC_BASIC
        .emit(test_span())
        .arg("expected", "int")
        .arg("found", "str")
        .with_note("note 1")
        .with_note("note 2")
        .build();
    assert_eq!(diag.notes, vec!["note 1", "note 2"]);
}

// ── level propagation ─────────────────────────────────────────────────────────

#[test]
fn descriptor_error_level_propagated() {
    let diag = DESC_BASIC
        .emit(test_span())
        .arg("expected", "int")
        .arg("found", "str")
        .build();
    assert!(diag.is_error());
}

#[test]
fn descriptor_warning_level_propagated() {
    let diag = DESC_WARNING
        .emit(test_span())
        .arg("name", "unused_var")
        .build();
    assert!(diag.is_warning());
    assert_eq!(diag.code, "AW9001");
}

// ── static_help-only (no extra) ───────────────────────────────────────────────

#[test]
fn descriptor_static_help_only_no_extras() {
    let diag = DESC_NO_HOLES.emit(test_span()).build();
    assert_eq!(diag.help.len(), 1);
    assert_eq!(diag.help[0], "make sure all blocks are closed");
    assert!(diag.notes.is_empty());
}
