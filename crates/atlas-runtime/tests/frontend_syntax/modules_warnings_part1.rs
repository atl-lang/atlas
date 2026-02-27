//! Module syntax and warnings tests part 1 (lines 1672-1714, 1733-2100 from original frontend_syntax.rs)

use super::*;


#[test]
fn test_export_without_item() {
    let source = r#"export"#;
    let (success, _) = parse(source);
    assert!(!success, "Should fail: export without fn/let/var");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_import_with_trailing_comma() {
    let source = r#"import { x, y, } from "./mod";"#;
    let (success, msgs) = parse(source);
    assert!(
        success,
        "Should parse import with trailing comma: {:?}",
        msgs
    );
}

#[test]
fn test_import_empty_list() {
    let source = r#"import { } from "./mod""#;
    let (success, msgs) = parse(source);
    // This should parse but might be semantically invalid
    // For now, just check it doesn't crash the parser
    let _ = (success, msgs);
}

#[test]
fn test_complex_nested_paths() {
    let source = r#"import { x } from "../../utils/helpers/math";"#;
    let (success, msgs) = parse(source);
    assert!(success, "Should parse complex nested paths: {:?}", msgs);
}

// ============================================================================
// Warning Detection Tests (from warning_tests.rs)
// ============================================================================

}

// ============================================================================
// Unused Variable Warnings (AT2001)
// ============================================================================

#[test]
fn test_unused_variable_warning() {
    let source = r#"fn main() -> number { let x: number = 42; return 5; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2001 warning");
    assert!(warnings[0].message.contains("Unused variable 'x'"));
}

#[test]
fn test_used_variable_no_warning() {
    let source = r#"fn main() -> number { let x: number = 42; return x; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 0, "Expected no AT2001 warnings");
}

#[test]
fn test_underscore_prefix_suppresses_warning() {
    let source = r#"fn main() -> number { let _unused: number = 42; return 5; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Underscore prefix should suppress warnings"
    );
}

#[test]
fn test_multiple_unused_variables() {
    let source = r#"fn main() -> number {
        let x: number = 1;
        let y: number = 2;
        let z: number = 3;
        return 0;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 3, "Expected 3 AT2001 warnings");
}

// ============================================================================
// Unused Parameter Warnings
// ============================================================================

#[test]
fn test_unused_parameter_warning() {
    let source = r#"fn add(a: number, b: number) -> number { return a; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        1,
        "Expected 1 AT2001 warning for unused param"
    );
    assert!(warnings[0].message.contains("Unused parameter 'b'"));
}

#[test]
fn test_used_parameter_in_callback_no_warning() {
    // Bug reproduction: parameter is used in function body, but function is passed as callback
    let source = r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        let result: number[] = map([1,2,3], double);
    "#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'x' is used in function body - should not warn even when function is passed as callback"
    );
}

#[test]
fn test_used_parameters_in_sort_callback_no_warning() {
    // Bug reproduction: both parameters are used, function passed to sort
    let source = r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }
        let sorted: number[] = sort([3,1,2], compare);
    "#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameters 'a' and 'b' are used in function body - should not warn when function is passed to sort"
    );
}

#[test]
fn test_minimal_callback_parameter_usage() {
    // Minimal reproduction: parameter used in intrinsic function call
    let source = r#"
        fn numToStr(n: number) -> string {
            return toString(n);
        }
        let x: string = numToStr(5);
    "#;

    let diags = get_all_diagnostics(source);

    // Debug output
    for diag in &diags {
        eprintln!("{:?}: {} (code: {})", diag.level, diag.message, diag.code);
    }

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'n' is used in toString call - should not warn"
    );
}

#[test]
fn test_parameter_used_in_user_function_call() {
    // Control test: parameter used in regular user function call
    let source = r#"
        fn helper(x: number) -> number {
            return x + 1;
        }
        fn wrapper(n: number) -> number {
            return helper(n);
        }
        let x: number = wrapper(5);
    "#;

    let diags = get_all_diagnostics(source);

    for diag in &diags {
        eprintln!("{:?}: {} (code: {})", diag.level, diag.message, diag.code);
    }

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'n' is used in helper call - should not warn"
    );
}

// ============================================================================
// Unreachable Code Warnings (AT2002)
// ============================================================================

#[test]
fn test_unreachable_code_after_return() {
    let source = r#"fn main() -> number {
        return 42;
        let x: number = 10;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2002").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2002 warning");
    assert!(warnings[0].message.contains("Unreachable code"));
}

#[test]
fn test_no_unreachable_warning_without_return() {
    let source = r#"fn main() -> number {
        let x: number = 42;
        let y: number = 10;
        return x;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2002").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Should not have unreachable code warning"
    );
}

// ============================================================================
// Warnings Combined with Errors
// ============================================================================

#[test]
fn test_warnings_with_errors() {
    let source = r#"fn main() -> number { let x: number = "bad"; return 5; }"#;
    let diags = get_all_diagnostics(source);

    // Should have both error (type mismatch) and warning (unused variable)
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect();

    assert!(!errors.is_empty(), "Expected type error");
    assert!(!warnings.is_empty(), "Expected unused warning");
}

// ============================================================================
// Warning System Tests (from warnings_tests.rs)
// ============================================================================

// ============================================================
// WarningLevel Tests
// ============================================================

#[test]
fn test_warning_level_default() {
    let config = WarningConfig::new();
    assert_eq!(config.default_level, WarningLevel::Warn);
}

#[test]
fn test_warning_level_allow_all() {
    let config = WarningConfig::allow_all();
    assert_eq!(config.default_level, WarningLevel::Allow);
}

#[test]
fn test_warning_level_deny_all() {
    let config = WarningConfig::deny_all();
    assert_eq!(config.default_level, WarningLevel::Deny);
}

// ============================================================
// WarningConfig Tests
// ============================================================

#[test]
fn test_config_allow_specific() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    assert!(config.is_allowed("AT2001"));
    assert!(!config.is_allowed("AT2002"));
}

#[test]
fn test_config_deny_specific() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    assert!(config.is_denied("AT2001"));
    assert!(!config.is_denied("AT2002"));
}

#[test]
fn test_config_warn_specific() {
    let mut config = WarningConfig::deny_all();
    config.warn("AT2001");
    assert_eq!(config.level_for("AT2001"), WarningLevel::Warn);
    assert!(config.is_denied("AT2002")); // Others still denied
}

#[test]
fn test_config_allow_overrides_deny() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    config.allow("AT2001");
    assert!(config.is_allowed("AT2001"));
}

#[test]
fn test_config_deny_overrides_allow() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    config.deny("AT2001");
    assert!(config.is_denied("AT2001"));
}

#[test]
fn test_config_per_code_override_global() {
    let mut config = WarningConfig::allow_all();
    config.deny("AT2001");
    assert!(config.is_denied("AT2001"));
    assert!(config.is_allowed("AT2002"));
}

#[test]
fn test_config_multiple_overrides() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    config.deny("AT2002");
    config.warn("AT2003");
    assert!(config.is_allowed("AT2001"));
    assert!(config.is_denied("AT2002"));
    assert_eq!(config.level_for("AT2003"), WarningLevel::Warn);
    assert_eq!(config.level_for("AT2004"), WarningLevel::Warn); // Default
}

// ============================================================
// WarningEmitter Tests
// ============================================================

#[test]
fn test_emitter_collect_warnings() {
    let mut emitter = WarningEmitter::default_config();
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused variable 'x'",
        Span::new(0, 1),
    ));
    assert!(emitter.has_warnings());
    assert_eq!(emitter.warnings().len(), 1);
    assert_eq!(emitter.count(), 1);
}

#[test]
fn test_emitter_suppress_allowed() {
    let mut config = WarningConfig::new();
    config.allow("AT2001");
    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    assert!(!emitter.has_warnings());
    assert_eq!(emitter.count(), 0);
}

#[test]
fn test_emitter_promote_denied() {
    let mut config = WarningConfig::new();
    config.deny("AT2001");
    let mut emitter = WarningEmitter::new(config);
    emitter.emit(Diagnostic::warning_with_code(
        "AT2001",
        "Unused",
        Span::new(0, 1),
    ));
    assert!(!emitter.has_warnings());
    assert!(emitter.has_errors());
    assert_eq!(emitter.errors().len(), 1);
    assert_eq!(emitter.errors()[0].level, DiagnosticLevel::Error);
}

#[test]
fn test_emitter_multiple_warnings() {
    let mut emitter = WarningEmitter::default_config();
    for i in 0..5 {
        emitter.emit(Diagnostic::warning_with_code(
            "AT2001",
            format!("warn {}", i),
            Span::new(i, i + 1),
        ));
    }
    assert_eq!(emitter.warnings().len(), 5);
    assert_eq!(emitter.count(), 5);
}

