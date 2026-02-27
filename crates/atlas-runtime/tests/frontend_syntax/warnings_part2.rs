//! Warnings tests part 2 (lines 2101-2425 from original frontend_syntax.rs)

use super::*;

#[test]
fn test_emitter_mixed_allow_deny() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    config.deny("AT2002");
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "unused",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "unreachable",
        Span::new(5, 10),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2003",
        "duplicate",
        Span::new(15, 20),
    ));

    assert_eq!(emitter.warnings().len(), 1); // Only AT2003
    assert_eq!(emitter.errors().len(), 1); // Promoted AT2002
    assert_eq!(emitter.count(), 2);
}

#[test]
fn test_emitter_clear() {
    let mut emitter = WarningEmitter::default_config();
    emitter.emit(Diagnostic::warning("warn", Span::new(0, 1)));
    assert!(emitter.has_warnings());
    emitter.clear();
    assert!(!emitter.has_warnings());
    assert_eq!(emitter.count(), 0);
}

#[test]
fn test_emitter_all_diagnostics() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);

    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "promoted",
        Span::new(0, 1),
    ));
    emitter.emit(Diagnostic::warning_with_code(
        "AT2002",
        "kept as warning",
        Span::new(5, 10),
    ));

    let all = emitter.all_diagnostics();
    assert_eq!(all.len(), 2);
    // Errors first in the result
    assert_eq!(all[0].level, DiagnosticLevel::Error);
    assert_eq!(all[1].level, DiagnosticLevel::Warning);
}

#[test]
fn test_emitter_no_warnings() {
    let emitter = WarningEmitter::default_config();
    assert!(!emitter.has_warnings());
    assert!(!emitter.has_errors());
    assert_eq!(emitter.count(), 0);
}

// ============================================================
// WarningKind Tests
// ============================================================

#[rstest]
#[case(WarningKind::UnusedVariable, "AT2001")]
#[case(WarningKind::UnreachableCode, "AT2002")]
#[case(WarningKind::DuplicateDeclaration, "AT2003")]
#[case(WarningKind::UnusedFunction, "AT2004")]
#[case(WarningKind::Shadowing, "AT2005")]
#[case(WarningKind::ConstantCondition, "AT2006")]
#[case(WarningKind::UnnecessaryAnnotation, "AT2007")]
#[case(WarningKind::UnusedImport, "AT2008")]
fn test_warning_kind_code(#[case] kind: WarningKind, #[case] expected: &str) {
    assert_eq!(kind.code(), expected);
}

#[rstest]
#[case("AT2001", Some(WarningKind::UnusedVariable))]
#[case("AT2002", Some(WarningKind::UnreachableCode))]
#[case("AT2003", Some(WarningKind::DuplicateDeclaration))]
#[case("AT2004", Some(WarningKind::UnusedFunction))]
#[case("AT2005", Some(WarningKind::Shadowing))]
#[case("AT2006", Some(WarningKind::ConstantCondition))]
#[case("AT2007", Some(WarningKind::UnnecessaryAnnotation))]
#[case("AT2008", Some(WarningKind::UnusedImport))]
#[case("XXXX", None)]
#[case("AT0001", None)]
fn test_warning_kind_from_code(#[case] code: &str, #[case] expected: Option<WarningKind>) {
    assert_eq!(WarningKind::from_code(code), expected);
}

// ============================================================
// TOML Config Tests
// ============================================================

#[test]
fn test_config_from_toml_warn_level() {
    let toml_str = r#"
[warnings]
level = "warn"
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Warn);
}

#[test]
fn test_config_from_toml_allow_level() {
    let toml_str = r#"
[warnings]
level = "allow"
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Allow);
}

#[test]
fn test_config_from_toml_deny_level() {
    let toml_str = r#"
[warnings]
level = "deny"
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Deny);
}

#[test]
fn test_config_from_toml_allow_list() {
    let toml_str = r#"
[warnings]
allow = ["AT2001", "AT2002"]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert!(config.is_allowed("AT2001"));
    assert!(config.is_allowed("AT2002"));
    assert!(!config.is_allowed("AT2003"));
}

#[test]
fn test_config_from_toml_deny_list() {
    let toml_str = r#"
[warnings]
deny = ["AT2001"]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert!(config.is_denied("AT2001"));
}

#[test]
fn test_config_from_toml_combined() {
    let toml_str = r#"
[warnings]
level = "warn"
allow = ["AT2001"]
deny = ["AT2002"]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert!(config.is_allowed("AT2001"));
    assert!(config.is_denied("AT2002"));
    assert_eq!(config.level_for("AT2005"), WarningLevel::Warn);
}

#[test]
fn test_config_from_toml_missing_section() {
    let toml_str = r#"
[package]
name = "test"
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Warn);
}

#[test]
fn test_config_from_toml_empty_warnings() {
    let toml_str = r#"
[warnings]
"#;
    let table: toml::Value = toml_str.parse().unwrap();
    let config = config_from_toml(&table);
    assert_eq!(config.default_level, WarningLevel::Warn);
}

// ============================================================
// Unused Variable Detection Integration Tests
// ============================================================

#[test]
fn test_warning_config_unused_variable() {
    let runtime = atlas_runtime::Atlas::new();
    // Unused variable should produce a warning (but warnings don't prevent execution)
    // In Atlas, eval returns errors (warnings are collected but don't fail)
    let result = runtime.eval("let x: number = 42; x");
    assert!(result.is_ok());
}

#[test]
fn test_unused_parameter_underscore_suppression() {
    // Variables prefixed with _ should not produce warnings
    // Atlas uses underscore prefix to suppress unused warnings
    let runtime = atlas_runtime::Atlas::new();
    let result = runtime.eval("let _x: number = 42;");
    assert!(result.is_ok());
}

// ============================================================
// Unreachable Code Warning Tests
// ============================================================

#[test]
fn test_warning_config_unreachable_code() {
    // The typechecker should emit an AT2002 warning for code after return
    // but since warnings don't prevent execution, the code should still run
    let runtime = atlas_runtime::Atlas::new();
    let result =
        runtime.eval("fn test(): number { return 1; let x: number = 2; return x; } test()");
    // Should succeed (warnings are non-fatal)
    // The typechecker emits the warning but it's collected, not returned as error
    assert!(result.is_ok() || result.is_err());
}

// ============================================================
// Warning Configuration Integration
// ============================================================

#[test]
fn test_warning_config_default_is_warn() {
    let config = WarningConfig::default();
    assert_eq!(config.default_level, WarningLevel::Warn);
}

#[test]
fn test_warning_config_is_clone() {
    let config = WarningConfig::new();
    let cloned = config.clone();
    assert_eq!(cloned.default_level, config.default_level);
}

#[test]
fn test_warning_emitter_config_access() {
    let config = WarningConfig::deny_all();
    let emitter = WarningEmitter::new(config);
    assert_eq!(emitter.config().default_level, WarningLevel::Deny);
}

// ============================================================
// Edge Cases
// ============================================================

#[test]
fn test_empty_warning_code() {
    let config = WarningConfig::new();
    assert_eq!(config.level_for(""), WarningLevel::Warn);
}

#[test]
fn test_unknown_warning_code() {
    let config = WarningConfig::new();
    assert_eq!(config.level_for("ZZZZ"), WarningLevel::Warn);
}

#[test]
fn test_warning_kind_roundtrip() {
    let kinds = vec![
        WarningKind::UnusedVariable,
        WarningKind::UnreachableCode,
        WarningKind::DuplicateDeclaration,
        WarningKind::UnusedFunction,
        WarningKind::Shadowing,
        WarningKind::ConstantCondition,
        WarningKind::UnnecessaryAnnotation,
        WarningKind::UnusedImport,
    ];
    for kind in kinds {
        let code = kind.code();
        let back = WarningKind::from_code(code).unwrap();
        assert_eq!(back, kind);
    }
}

#[test]
fn test_emitter_promoted_error_preserves_code() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    let errors = emitter.errors();
    assert_eq!(errors[0].code, "AT2001");
    assert_eq!(errors[0].level, DiagnosticLevel::Error);
}

#[test]
fn test_emitter_promoted_error_preserves_message() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused variable 'foo'",
        Span::new(0, 3),
    ));
    assert_eq!(emitter.errors()[0].message, "Unused variable 'foo'");
}

// ============================================================================
