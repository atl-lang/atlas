//! Security Configuration
//!
//! Defines security policies and permission settings for Atlas projects.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct SecurityConfig {
    /// Security mode ("none", "standard", "strict")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// Filesystem permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem: Option<FilesystemPermissions>,

    /// Network permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkPermissions>,

    /// Process execution permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<ProcessPermissions>,

    /// Environment variable permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<EnvironmentPermissions>,
}

/// Filesystem permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct FilesystemPermissions {
    /// Paths allowed for reading
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub read: Vec<PathBuf>,

    /// Paths allowed for writing
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub write: Vec<PathBuf>,

    /// Paths explicitly denied
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<PathBuf>,
}

/// Network permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct NetworkPermissions {
    /// Hosts/domains allowed
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,

    /// Hosts/domains explicitly denied
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
}

/// Process execution permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct ProcessPermissions {
    /// Commands allowed to execute
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,

    /// Commands explicitly denied
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
}

/// Environment variable permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentPermissions {
    /// Environment variables allowed to read
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,

    /// Environment variables explicitly denied
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
}

impl SecurityConfig {
    /// Get the security mode (default: "standard")
    pub fn mode(&self) -> &str {
        self.mode.as_deref().unwrap_or("standard")
    }

    /// Check if mode is valid
    pub fn is_valid_mode(mode: &str) -> bool {
        matches!(mode, "none" | "standard" | "strict")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_security_config() {
        let toml = r#"
mode = "strict"

[filesystem]
read = ["./data", "./config"]
write = ["./output"]

[network]
allow = ["api.example.com"]
deny = ["*"]

[process]
deny = ["*"]

[environment]
allow = ["PATH", "HOME"]
"#;

        let config: SecurityConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.mode(), "strict");
        assert_eq!(config.filesystem.as_ref().unwrap().read.len(), 2);
        assert_eq!(config.network.as_ref().unwrap().allow.len(), 1);
    }

    #[test]
    fn test_security_mode_default() {
        let config = SecurityConfig::default();
        assert_eq!(config.mode(), "standard");
    }

    #[test]
    fn test_valid_modes() {
        assert!(SecurityConfig::is_valid_mode("none"));
        assert!(SecurityConfig::is_valid_mode("standard"));
        assert!(SecurityConfig::is_valid_mode("strict"));
        assert!(!SecurityConfig::is_valid_mode("invalid"));
    }
}
