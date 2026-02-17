//! Warning configuration and collection system
//!
//! Provides configurable warning levels (allow/warn/deny) per warning code,
//! with support for global warning level and per-code overrides.

use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use std::collections::{HashMap, HashSet};

/// Warning severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningLevel {
    /// Suppress the warning entirely
    Allow,
    /// Emit as a warning (default)
    Warn,
    /// Promote to an error
    Deny,
}

/// Warning kind classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WarningKind {
    /// Unused variable or parameter (AT2001)
    UnusedVariable,
    /// Unreachable code after return (AT2002)
    UnreachableCode,
    /// Duplicate declaration (AT2003)
    DuplicateDeclaration,
    /// Unused function (AT2004)
    UnusedFunction,
    /// Variable shadowing (AT2005)
    Shadowing,
    /// Constant condition (AT2006)
    ConstantCondition,
    /// Unnecessary type annotation (AT2007)
    UnnecessaryAnnotation,
    /// Unused import (AT2008)
    UnusedImport,
}

impl WarningKind {
    /// Get the error code for this warning kind
    pub fn code(&self) -> &'static str {
        match self {
            WarningKind::UnusedVariable => "AT2001",
            WarningKind::UnreachableCode => "AT2002",
            WarningKind::DuplicateDeclaration => "AT2003",
            WarningKind::UnusedFunction => "AT2004",
            WarningKind::Shadowing => "AT2005",
            WarningKind::ConstantCondition => "AT2006",
            WarningKind::UnnecessaryAnnotation => "AT2007",
            WarningKind::UnusedImport => "AT2008",
        }
    }

    /// Parse from error code string
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "AT2001" => Some(WarningKind::UnusedVariable),
            "AT2002" => Some(WarningKind::UnreachableCode),
            "AT2003" => Some(WarningKind::DuplicateDeclaration),
            "AT2004" => Some(WarningKind::UnusedFunction),
            "AT2005" => Some(WarningKind::Shadowing),
            "AT2006" => Some(WarningKind::ConstantCondition),
            "AT2007" => Some(WarningKind::UnnecessaryAnnotation),
            "AT2008" => Some(WarningKind::UnusedImport),
            _ => None,
        }
    }
}

/// Warning configuration controlling which warnings are emitted
#[derive(Debug, Clone)]
pub struct WarningConfig {
    /// Global warning level (default: Warn)
    pub default_level: WarningLevel,
    /// Per-code overrides
    overrides: HashMap<String, WarningLevel>,
    /// Codes explicitly allowed (suppressed)
    allow_set: HashSet<String>,
    /// Codes explicitly denied (promoted to errors)
    deny_set: HashSet<String>,
}

impl WarningConfig {
    /// Create a default warning config (all warnings enabled)
    pub fn new() -> Self {
        Self {
            default_level: WarningLevel::Warn,
            overrides: HashMap::new(),
            allow_set: HashSet::new(),
            deny_set: HashSet::new(),
        }
    }

    /// Create a config that suppresses all warnings
    pub fn allow_all() -> Self {
        Self {
            default_level: WarningLevel::Allow,
            overrides: HashMap::new(),
            allow_set: HashSet::new(),
            deny_set: HashSet::new(),
        }
    }

    /// Create a config that denies all warnings (treats as errors)
    pub fn deny_all() -> Self {
        Self {
            default_level: WarningLevel::Deny,
            overrides: HashMap::new(),
            allow_set: HashSet::new(),
            deny_set: HashSet::new(),
        }
    }

    /// Allow (suppress) a specific warning code
    pub fn allow(&mut self, code: impl Into<String>) {
        let code = code.into();
        self.deny_set.remove(&code);
        self.allow_set.insert(code.clone());
        self.overrides.insert(code, WarningLevel::Allow);
    }

    /// Deny (promote to error) a specific warning code
    pub fn deny(&mut self, code: impl Into<String>) {
        let code = code.into();
        self.allow_set.remove(&code);
        self.deny_set.insert(code.clone());
        self.overrides.insert(code, WarningLevel::Deny);
    }

    /// Set a specific warning code to warn level
    pub fn warn(&mut self, code: impl Into<String>) {
        let code = code.into();
        self.allow_set.remove(&code);
        self.deny_set.remove(&code);
        self.overrides.insert(code, WarningLevel::Warn);
    }

    /// Get the effective level for a warning code
    pub fn level_for(&self, code: &str) -> WarningLevel {
        if let Some(level) = self.overrides.get(code) {
            *level
        } else {
            self.default_level
        }
    }

    /// Check if a code is allowed (suppressed)
    pub fn is_allowed(&self, code: &str) -> bool {
        self.level_for(code) == WarningLevel::Allow
    }

    /// Check if a code is denied (promoted to error)
    pub fn is_denied(&self, code: &str) -> bool {
        self.level_for(code) == WarningLevel::Deny
    }
}

impl Default for WarningConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Warning emitter that collects and filters warnings
#[derive(Debug, Clone)]
pub struct WarningEmitter {
    config: WarningConfig,
    warnings: Vec<Diagnostic>,
    errors: Vec<Diagnostic>,
}

impl WarningEmitter {
    /// Create a new warning emitter with the given config
    pub fn new(config: WarningConfig) -> Self {
        Self {
            config,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(WarningConfig::new())
    }

    /// Emit a warning diagnostic, applying config rules
    pub fn emit(&mut self, diag: Diagnostic) {
        let level = self.config.level_for(&diag.code);
        match level {
            WarningLevel::Allow => {
                // Suppressed, don't collect
            }
            WarningLevel::Warn => {
                self.warnings.push(diag);
            }
            WarningLevel::Deny => {
                // Promote to error
                let error = Diagnostic {
                    level: DiagnosticLevel::Error,
                    ..diag
                };
                self.errors.push(error);
            }
        }
    }

    /// Get collected warnings
    pub fn warnings(&self) -> &[Diagnostic] {
        &self.warnings
    }

    /// Get warnings promoted to errors
    pub fn errors(&self) -> &[Diagnostic] {
        &self.errors
    }

    /// Check if any warnings were collected
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if any warnings were promoted to errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get all diagnostics (warnings + promoted errors)
    pub fn all_diagnostics(&self) -> Vec<Diagnostic> {
        let mut all = self.errors.clone();
        all.extend(self.warnings.clone());
        all
    }

    /// Get the warning config
    pub fn config(&self) -> &WarningConfig {
        &self.config
    }

    /// Clear all collected diagnostics
    pub fn clear(&mut self) {
        self.warnings.clear();
        self.errors.clear();
    }

    /// Total count of warnings + errors
    pub fn count(&self) -> usize {
        self.warnings.len() + self.errors.len()
    }
}

/// Build a WarningConfig from atlas.toml [warnings] section
pub fn config_from_toml(table: &toml::Value) -> WarningConfig {
    let mut config = WarningConfig::new();

    if let Some(warnings) = table.get("warnings").and_then(|v| v.as_table()) {
        // Global level
        if let Some(level) = warnings.get("level").and_then(|v| v.as_str()) {
            config.default_level = match level {
                "allow" => WarningLevel::Allow,
                "deny" => WarningLevel::Deny,
                _ => WarningLevel::Warn,
            };
        }

        // Allow list
        if let Some(allow) = warnings.get("allow").and_then(|v| v.as_array()) {
            for code in allow {
                if let Some(s) = code.as_str() {
                    config.allow(s);
                }
            }
        }

        // Deny list
        if let Some(deny) = warnings.get("deny").and_then(|v| v.as_array()) {
            for code in deny {
                if let Some(s) = code.as_str() {
                    config.deny(s);
                }
            }
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_default_config() {
        let config = WarningConfig::new();
        assert_eq!(config.default_level, WarningLevel::Warn);
        assert_eq!(config.level_for("AT2001"), WarningLevel::Warn);
    }

    #[test]
    fn test_allow_all() {
        let config = WarningConfig::allow_all();
        assert_eq!(config.level_for("AT2001"), WarningLevel::Allow);
        assert!(config.is_allowed("AT2001"));
    }

    #[test]
    fn test_deny_all() {
        let config = WarningConfig::deny_all();
        assert_eq!(config.level_for("AT2001"), WarningLevel::Deny);
        assert!(config.is_denied("AT2001"));
    }

    #[test]
    fn test_allow_specific() {
        let mut config = WarningConfig::new();
        config.allow("AT2001");
        assert!(config.is_allowed("AT2001"));
        assert!(!config.is_allowed("AT2002"));
    }

    #[test]
    fn test_deny_specific() {
        let mut config = WarningConfig::new();
        config.deny("AT2001");
        assert!(config.is_denied("AT2001"));
        assert!(!config.is_denied("AT2002"));
    }

    #[test]
    fn test_override_precedence() {
        let mut config = WarningConfig::allow_all();
        config.deny("AT2001");
        assert!(config.is_denied("AT2001"));
        assert!(config.is_allowed("AT2002")); // Still globally allowed
    }

    #[test]
    fn test_allow_overrides_deny() {
        let mut config = WarningConfig::new();
        config.deny("AT2001");
        config.allow("AT2001");
        assert!(config.is_allowed("AT2001"));
    }

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
    }

    #[test]
    fn test_emitter_suppress_allowed() {
        let mut config = WarningConfig::new();
        config.allow("AT2001");
        let mut emitter = WarningEmitter::new(config);
        emitter.emit(Diagnostic::warning_with_code(
            "AT2001",
            "Unused variable 'x'",
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
            "Unused variable 'x'",
            Span::new(0, 1),
        ));
        assert!(!emitter.has_warnings());
        assert!(emitter.has_errors());
        assert_eq!(emitter.errors()[0].level, DiagnosticLevel::Error);
    }

    #[test]
    fn test_emitter_multiple_warnings() {
        let mut emitter = WarningEmitter::default_config();
        emitter.emit(Diagnostic::warning_with_code(
            "AT2001",
            "warn1",
            Span::new(0, 1),
        ));
        emitter.emit(Diagnostic::warning_with_code(
            "AT2002",
            "warn2",
            Span::new(5, 10),
        ));
        emitter.emit(Diagnostic::warning_with_code(
            "AT2001",
            "warn3",
            Span::new(15, 20),
        ));
        assert_eq!(emitter.warnings().len(), 3);
        assert_eq!(emitter.count(), 3);
    }

    #[test]
    fn test_emitter_clear() {
        let mut emitter = WarningEmitter::default_config();
        emitter.emit(Diagnostic::warning_with_code(
            "AT2001",
            "warn",
            Span::new(0, 1),
        ));
        assert!(emitter.has_warnings());
        emitter.clear();
        assert!(!emitter.has_warnings());
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
            "kept",
            Span::new(5, 10),
        ));

        let all = emitter.all_diagnostics();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].level, DiagnosticLevel::Error); // Promoted
        assert_eq!(all[1].level, DiagnosticLevel::Warning); // Kept
    }

    #[test]
    fn test_warning_kind_codes() {
        assert_eq!(WarningKind::UnusedVariable.code(), "AT2001");
        assert_eq!(WarningKind::UnreachableCode.code(), "AT2002");
        assert_eq!(WarningKind::UnusedFunction.code(), "AT2004");
        assert_eq!(WarningKind::Shadowing.code(), "AT2005");
        assert_eq!(WarningKind::ConstantCondition.code(), "AT2006");
    }

    #[test]
    fn test_warning_kind_from_code() {
        assert_eq!(
            WarningKind::from_code("AT2001"),
            Some(WarningKind::UnusedVariable)
        );
        assert_eq!(WarningKind::from_code("XXXX"), None);
    }

    #[test]
    fn test_config_from_toml() {
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
    fn test_config_from_toml_deny_level() {
        let toml_str = r#"
[warnings]
level = "deny"
allow = ["AT2001"]
"#;
        let table: toml::Value = toml_str.parse().unwrap();
        let config = config_from_toml(&table);

        assert!(config.is_allowed("AT2001"));
        assert!(config.is_denied("AT2002")); // Default deny
    }

    #[test]
    fn test_config_from_toml_allow_level() {
        let toml_str = r#"
[warnings]
level = "allow"
"#;
        let table: toml::Value = toml_str.parse().unwrap();
        let config = config_from_toml(&table);

        assert!(config.is_allowed("AT2001"));
        assert!(config.is_allowed("AT2002"));
    }

    #[test]
    fn test_config_from_toml_missing_section() {
        let toml_str = r#"
[package]
name = "test"
"#;
        let table: toml::Value = toml_str.parse().unwrap();
        let config = config_from_toml(&table);
        // Should return default config
        assert_eq!(config.default_level, WarningLevel::Warn);
    }

    #[test]
    fn test_warn_override() {
        let mut config = WarningConfig::deny_all();
        config.warn("AT2001");
        assert_eq!(config.level_for("AT2001"), WarningLevel::Warn);
        assert!(!config.is_denied("AT2001"));
        assert!(!config.is_allowed("AT2001"));
    }
}
