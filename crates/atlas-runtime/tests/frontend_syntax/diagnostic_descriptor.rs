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

// ── descriptor_coverage (P02 AC) ──────────────────────────────────────────────

#[test]
fn descriptor_coverage_all_codes_have_title_and_help() {
    use atlas_runtime::diagnostic::error_codes::DESCRIPTOR_REGISTRY;
    for desc in DESCRIPTOR_REGISTRY {
        assert!(!desc.code.is_empty(), "descriptor has empty code");
        assert!(
            !desc.title.is_empty(),
            "descriptor {} has empty title",
            desc.code
        );
        assert!(
            !desc.message_template.is_empty(),
            "descriptor {} has empty message_template",
            desc.code
        );
        // AT9999 and AW9999 are intentionally generic — allow them to have help
        assert!(
            desc.static_help.is_some(),
            "descriptor {} ('{}') has no static_help",
            desc.code,
            desc.title
        );
    }
}

#[test]
fn descriptor_lookup_at1000_returns_descriptor() {
    use atlas_runtime::diagnostic::error_codes::lookup;
    let d = lookup("AT1000").expect("AT1000 should be registered");
    assert_eq!(d.code, "AT1000");
    assert!(!d.title.is_empty());
    assert!(d.static_help.is_some());
}

#[test]
fn descriptor_lookup_unknown_code_returns_none() {
    use atlas_runtime::diagnostic::error_codes::lookup;
    assert!(lookup("AT0000").is_none());
    assert!(lookup("").is_none());
    assert!(lookup("NOTACODE").is_none());
}

#[test]
fn descriptor_registry_has_no_duplicate_codes() {
    use atlas_runtime::diagnostic::error_codes::DESCRIPTOR_REGISTRY;
    let mut seen = std::collections::HashSet::new();
    for desc in DESCRIPTOR_REGISTRY {
        assert!(
            seen.insert(desc.code),
            "duplicate code in DESCRIPTOR_REGISTRY: {}",
            desc.code
        );
    }
}

#[test]
fn descriptor_new_codes_at0304_at0500_at0501_registered() {
    use atlas_runtime::diagnostic::error_codes::lookup;
    let ffi = lookup("AT0304").expect("AT0304 FFI_PERMISSION_DENIED should be registered");
    assert!(ffi.static_help.is_some());
    let timeout = lookup("AT0500").expect("AT0500 EXECUTION_TIMEOUT should be registered");
    assert!(timeout.static_help.is_some());
    let mem = lookup("AT0501").expect("AT0501 MEMORY_LIMIT_EXCEEDED should be registered");
    assert!(mem.static_help.is_some());
}

#[test]
fn descriptor_parser_e_codes_no_longer_conflict() {
    use atlas_runtime::diagnostic::error_codes::lookup;
    // AT1003 = INVALID_ESCAPE (not missing brace)
    let esc = lookup("AT1003").expect("AT1003 should be registered");
    assert_eq!(esc.title, "Invalid escape sequence");
    // AT1004 = UNTERMINATED_COMMENT (not unexpected token)
    let comment = lookup("AT1004").expect("AT1004 should be registered");
    assert_eq!(comment.title, "Unterminated block comment");
    // AT1021 = MISSING_CLOSING_DELIMITER (new)
    let delim = lookup("AT1021").expect("AT1021 should be registered");
    assert!(delim.title.contains("Missing closing"));
    // AT1022 = RESERVED_KEYWORD_AS_IDENTIFIER (new)
    let kw = lookup("AT1022").expect("AT1022 should be registered");
    assert!(kw.title.contains("Reserved"));
}
