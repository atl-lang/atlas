//! Comprehensive security and permission tests

use atlas_runtime::security::{Permission, PermissionSet, SecurityContext, SecurityError};
use std::path::{Path, PathBuf};

// ============================================================================
// Permission Matching Tests
// ============================================================================

#[test]
fn test_filesystem_read_exact_match() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_filesystem_read_recursive_allows_subdirs() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/subdir/file.txt"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_filesystem_read_recursive_allows_nested() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/a/b/c/file.txt"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_filesystem_read_non_recursive_denies_subdirs() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: false,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_filesystem_read_different_paths() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/other/file.txt"),
        recursive: false,
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_filesystem_write_exact_match() {
    let allowed = Permission::FilesystemWrite {
        path: PathBuf::from("/output/result.txt"),
        recursive: false,
    };
    let requested = Permission::FilesystemWrite {
        path: PathBuf::from("/output/result.txt"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_filesystem_write_recursive() {
    let allowed = Permission::FilesystemWrite {
        path: PathBuf::from("/output"),
        recursive: true,
    };
    let requested = Permission::FilesystemWrite {
        path: PathBuf::from("/output/logs/app.log"),
        recursive: false,
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_exact_host_match() {
    let allowed = Permission::Network {
        host: "api.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "api.example.com".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_wildcard_subdomain() {
    let allowed = Permission::Network {
        host: "*.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "api.example.com".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_wildcard_nested_subdomain() {
    let allowed = Permission::Network {
        host: "*.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "api.v2.example.com".to_string(),
    };
    // *.example.com matches any subdomains (including nested)
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_wildcard_all() {
    let allowed = Permission::Network {
        host: "*".to_string(),
    };
    let requested = Permission::Network {
        host: "any.host.com".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_network_different_hosts() {
    let allowed = Permission::Network {
        host: "api.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "other.com".to_string(),
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_process_exact_command() {
    let allowed = Permission::Process {
        command: "git".to_string(),
    };
    let requested = Permission::Process {
        command: "git".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_process_wildcard() {
    let allowed = Permission::Process {
        command: "*".to_string(),
    };
    let requested = Permission::Process {
        command: "git".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_process_different_commands() {
    let allowed = Permission::Process {
        command: "git".to_string(),
    };
    let requested = Permission::Process {
        command: "npm".to_string(),
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_environment_exact_var() {
    let allowed = Permission::Environment {
        var: "PATH".to_string(),
    };
    let requested = Permission::Environment {
        var: "PATH".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_environment_wildcard() {
    let allowed = Permission::Environment {
        var: "*".to_string(),
    };
    let requested = Permission::Environment {
        var: "HOME".to_string(),
    };
    assert!(allowed.allows(&requested));
}

#[test]
fn test_environment_different_vars() {
    let allowed = Permission::Environment {
        var: "PATH".to_string(),
    };
    let requested = Permission::Environment {
        var: "HOME".to_string(),
    };
    assert!(!allowed.allows(&requested));
}

// ============================================================================
// Permission Type Mismatch Tests
// ============================================================================

#[test]
fn test_permission_type_mismatch_fs_vs_network() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::Network {
        host: "example.com".to_string(),
    };
    assert!(!allowed.allows(&requested));
}

#[test]
fn test_permission_type_mismatch_read_vs_write() {
    let allowed = Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    };
    let requested = Permission::FilesystemWrite {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(!allowed.allows(&requested));
}

// ============================================================================
// PermissionSet Tests
// ============================================================================

#[test]
fn test_permission_set_empty_denies_all() {
    let set = PermissionSet::new();
    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(!set.is_granted(&requested));
}

#[test]
fn test_permission_set_grant_and_check() {
    let mut set = PermissionSet::new();
    set.grant(Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    });

    let requested = Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    };
    assert!(set.is_granted(&requested));
}

#[test]
fn test_permission_set_multiple_permissions() {
    let mut set = PermissionSet::new();
    set.grant(Permission::FilesystemRead {
        path: PathBuf::from("/data"),
        recursive: true,
    });
    set.grant(Permission::FilesystemRead {
        path: PathBuf::from("/config.txt"),
        recursive: false,
    });

    assert!(set.is_granted(&Permission::FilesystemRead {
        path: PathBuf::from("/data/file.txt"),
        recursive: false,
    }));
    assert!(set.is_granted(&Permission::FilesystemRead {
        path: PathBuf::from("/config.txt"),
        recursive: false,
    }));
    assert!(!set.is_granted(&Permission::FilesystemRead {
        path: PathBuf::from("/other.txt"),
        recursive: false,
    }));
}

// ============================================================================
// SecurityContext Tests
// ============================================================================

#[test]
fn test_security_context_default_denies_everything() {
    let ctx = SecurityContext::new();

    assert!(ctx
        .check_filesystem_read(Path::new("/data/file.txt"))
        .is_err());
    assert!(ctx
        .check_filesystem_write(Path::new("/output/file.txt"))
        .is_err());
    assert!(ctx.check_network("api.example.com").is_err());
    assert!(ctx.check_process("git").is_err());
    assert!(ctx.check_environment("PATH").is_err());
}

#[test]
fn test_security_context_grant_filesystem_read() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_read(Path::new("/data"), true);

    assert!(ctx.check_filesystem_read(Path::new("/data/file.txt")).is_ok());
    assert!(ctx
        .check_filesystem_read(Path::new("/data/subdir/file.txt"))
        .is_ok());
    assert!(ctx.check_filesystem_read(Path::new("/other/file.txt")).is_err());
}

#[test]
fn test_security_context_grant_filesystem_write() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_write(Path::new("/output"), true);

    assert!(ctx.check_filesystem_write(Path::new("/output/file.txt")).is_ok());
    assert!(ctx.check_filesystem_write(Path::new("/output/logs/app.log")).is_ok());
    assert!(ctx.check_filesystem_write(Path::new("/other/file.txt")).is_err());
}

#[test]
fn test_security_context_grant_network() {
    let mut ctx = SecurityContext::new();
    ctx.grant_network("api.example.com");

    assert!(ctx.check_network("api.example.com").is_ok());
    assert!(ctx.check_network("other.com").is_err());
}

#[test]
fn test_security_context_grant_network_wildcard() {
    let mut ctx = SecurityContext::new();
    ctx.grant_network("*.example.com");

    assert!(ctx.check_network("api.example.com").is_ok());
    assert!(ctx.check_network("cdn.example.com").is_ok());
    assert!(ctx.check_network("other.com").is_err());
}

#[test]
fn test_security_context_grant_process() {
    let mut ctx = SecurityContext::new();
    ctx.grant_process("git");

    assert!(ctx.check_process("git").is_ok());
    assert!(ctx.check_process("npm").is_err());
}

#[test]
fn test_security_context_grant_environment() {
    let mut ctx = SecurityContext::new();
    ctx.grant_environment("PATH");

    assert!(ctx.check_environment("PATH").is_ok());
    assert!(ctx.check_environment("HOME").is_err());
}

#[test]
fn test_security_context_allow_all() {
    let ctx = SecurityContext::allow_all();

    assert!(ctx.check_filesystem_read(Path::new("/any/path")).is_ok());
    assert!(ctx.check_filesystem_write(Path::new("/any/path")).is_ok());
    assert!(ctx.check_network("any.host.com").is_ok());
    assert!(ctx.check_process("any-command").is_ok());
    assert!(ctx.check_environment("ANY_VAR").is_ok());
}

// ============================================================================
// SecurityError Tests
// ============================================================================

#[test]
fn test_security_error_filesystem_read() {
    let ctx = SecurityContext::new();
    let result = ctx.check_filesystem_read(Path::new("/data/file.txt"));

    assert!(matches!(
        result,
        Err(SecurityError::FilesystemReadDenied { .. })
    ));
}

#[test]
fn test_security_error_filesystem_write() {
    let ctx = SecurityContext::new();
    let result = ctx.check_filesystem_write(Path::new("/output/file.txt"));

    assert!(matches!(
        result,
        Err(SecurityError::FilesystemWriteDenied { .. })
    ));
}

#[test]
fn test_security_error_network() {
    let ctx = SecurityContext::new();
    let result = ctx.check_network("api.example.com");

    assert!(matches!(result, Err(SecurityError::NetworkDenied { .. })));
}

#[test]
fn test_security_error_process() {
    let ctx = SecurityContext::new();
    let result = ctx.check_process("git");

    assert!(matches!(result, Err(SecurityError::ProcessDenied { .. })));
}

#[test]
fn test_security_error_environment() {
    let ctx = SecurityContext::new();
    let result = ctx.check_environment("PATH");

    assert!(matches!(
        result,
        Err(SecurityError::EnvironmentDenied { .. })
    ));
}

// ============================================================================
// Edge Cases and Security Tests
// ============================================================================

#[test]
fn test_empty_path_handling() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_read(Path::new(""), true);

    // Empty path should be treated as current directory
    assert!(ctx.check_filesystem_read(Path::new("")).is_ok());
}

#[test]
fn test_multiple_grants_same_type() {
    let mut ctx = SecurityContext::new();
    ctx.grant_filesystem_read(Path::new("/data"), true);
    ctx.grant_filesystem_read(Path::new("/config"), true);

    assert!(ctx.check_filesystem_read(Path::new("/data/file.txt")).is_ok());
    assert!(ctx.check_filesystem_read(Path::new("/config/app.toml")).is_ok());
    assert!(ctx.check_filesystem_read(Path::new("/other/file.txt")).is_err());
}

#[test]
fn test_network_base_domain_match() {
    let allowed = Permission::Network {
        host: "*.example.com".to_string(),
    };
    let requested = Permission::Network {
        host: "example.com".to_string(),
    };
    // *.example.com should also match example.com (base domain)
    assert!(allowed.allows(&requested));
}

#[test]
fn test_security_context_isolation() {
    let mut ctx1 = SecurityContext::new();
    let ctx2 = SecurityContext::new();

    ctx1.grant_filesystem_read(Path::new("/data"), true);

    // ctx2 should not have ctx1's permissions
    assert!(ctx1.check_filesystem_read(Path::new("/data/file.txt")).is_ok());
    assert!(ctx2.check_filesystem_read(Path::new("/data/file.txt")).is_err());
}
