//! Audit logging tests
//!
//! Tests that all security events are properly logged for monitoring and compliance.

use atlas_runtime::{AuditEvent, AuditLogger, MemoryAuditLogger, SecurityContext};
use std::path::Path;
use std::sync::Arc;

// ============================================================================
// Audit Logging Integration Tests
// ============================================================================

#[test]
fn test_audit_logger_logs_filesystem_read_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_filesystem_read(Path::new("/etc/passwd"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::FilesystemReadDenied { .. }
    ));
}

#[test]
fn test_audit_logger_logs_filesystem_read_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_filesystem_read(Path::new("/data"), true);
    let _ = ctx.check_filesystem_read(Path::new("/data/file.txt"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

#[test]
fn test_audit_logger_logs_filesystem_write_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_filesystem_write(Path::new("/etc/passwd"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::FilesystemWriteDenied { .. }
    ));
}

#[test]
fn test_audit_logger_logs_filesystem_write_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_filesystem_write(Path::new("/output"), true);
    let _ = ctx.check_filesystem_write(Path::new("/output/file.txt"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

#[test]
fn test_audit_logger_logs_network_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("api.example.com");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(&entries[0].event, AuditEvent::NetworkDenied { .. }));
}

#[test]
fn test_audit_logger_logs_network_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_network("api.example.com");
    let _ = ctx.check_network("api.example.com");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

#[test]
fn test_audit_logger_logs_process_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_process("git");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(&entries[0].event, AuditEvent::ProcessDenied { .. }));
}

#[test]
fn test_audit_logger_logs_process_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_process("git");
    let _ = ctx.check_process("git");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

#[test]
fn test_audit_logger_logs_environment_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_environment("PATH");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::EnvironmentDenied { .. }
    ));
}

#[test]
fn test_audit_logger_logs_environment_granted() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_environment("PATH");
    let _ = ctx.check_environment("PATH");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));
}

// ============================================================================
// Multiple Events Tests
// ============================================================================

#[test]
fn test_audit_logger_logs_multiple_events() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_filesystem_read(Path::new("/data"), true);

    // Multiple permission checks
    let _ = ctx.check_filesystem_read(Path::new("/data/file1.txt"));
    let _ = ctx.check_filesystem_read(Path::new("/data/file2.txt"));
    let _ = ctx.check_filesystem_read(Path::new("/etc/passwd"));
    let _ = ctx.check_network("api.example.com");

    let entries = logger.entries();
    assert_eq!(entries.len(), 4);
}

#[test]
fn test_audit_logger_logs_granted_and_denied() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let mut ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    ctx.grant_filesystem_read(Path::new("/data"), true);

    // Granted
    let _ = ctx.check_filesystem_read(Path::new("/data/file.txt"));

    // Denied
    let _ = ctx.check_filesystem_read(Path::new("/etc/passwd"));

    let entries = logger.entries();
    assert_eq!(entries.len(), 2);

    // First should be granted
    assert!(matches!(
        &entries[0].event,
        AuditEvent::PermissionCheck { granted: true, .. }
    ));

    // Second should be denied
    assert!(matches!(
        &entries[1].event,
        AuditEvent::FilesystemReadDenied { .. }
    ));
}

// ============================================================================
// Audit Entry Format Tests
// ============================================================================

#[test]
fn test_audit_entry_has_timestamp() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("api.example.com");

    let entries = logger.entries();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].timestamp > 0);
}

#[test]
fn test_audit_entry_log_line_format() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_filesystem_read(Path::new("/etc/passwd"));

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();

    assert!(log_line.contains("Permission denied"));
    assert!(log_line.contains("file read"));
    assert!(log_line.contains("/etc/passwd"));
    assert!(log_line.starts_with('[')); // Has timestamp
}

#[test]
fn test_audit_event_display_filesystem_read_denied() {
    let event = AuditEvent::FilesystemReadDenied {
        path: Path::new("/etc/passwd").to_path_buf(),
    };
    let display = event.to_string();

    assert!(display.contains("Permission denied"));
    assert!(display.contains("file read"));
    assert!(display.contains("/etc/passwd"));
}

#[test]
fn test_audit_event_display_network_denied() {
    let event = AuditEvent::NetworkDenied {
        host: "evil.com".to_string(),
    };
    let display = event.to_string();

    assert!(display.contains("Permission denied"));
    assert!(display.contains("network access"));
    assert!(display.contains("evil.com"));
}

#[test]
fn test_audit_event_display_permission_check_granted() {
    let event = AuditEvent::PermissionCheck {
        operation: "file read".to_string(),
        target: "/data/file.txt".to_string(),
        granted: true,
    };
    let display = event.to_string();

    assert!(display.contains("GRANTED"));
    assert!(display.contains("file read"));
    assert!(display.contains("/data/file.txt"));
}

#[test]
fn test_audit_event_display_permission_check_denied() {
    let event = AuditEvent::PermissionCheck {
        operation: "network".to_string(),
        target: "evil.com".to_string(),
        granted: false,
    };
    let display = event.to_string();

    assert!(display.contains("DENIED"));
    assert!(display.contains("network"));
    assert!(display.contains("evil.com"));
}

// ============================================================================
// Audit Logger Clear Tests
// ============================================================================

#[test]
fn test_audit_logger_clear() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("api.example.com");
    assert_eq!(logger.entries().len(), 1);

    logger.clear();
    assert_eq!(logger.entries().len(), 0);
}

#[test]
fn test_audit_logger_clear_and_continue() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("api1.example.com");
    logger.clear();

    let _ = ctx.check_network("api2.example.com");
    assert_eq!(logger.entries().len(), 1);
}

// ============================================================================
// Default Context (No Audit Logger) Tests
// ============================================================================

#[test]
fn test_default_context_has_null_logger() {
    // Default context should have NullAuditLogger (no logging overhead)
    let ctx = SecurityContext::new();

    // Should not panic or cause errors
    let _ = ctx.check_filesystem_read(Path::new("/etc/passwd"));
    let _ = ctx.check_network("api.example.com");
}

#[test]
fn test_default_context_audit_logger_is_null() {
    let ctx = SecurityContext::new();

    // Perform some checks
    let _ = ctx.check_filesystem_read(Path::new("/etc/passwd"));
    let _ = ctx.check_network("api.example.com");

    // Get audit logger and verify it returns no entries
    let logger = ctx.audit_logger();
    assert_eq!(logger.entries().len(), 0); // NullAuditLogger returns empty
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[test]
fn test_audit_logger_thread_safe() {
    use std::thread;

    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = Arc::new(SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let ctx = Arc::clone(&ctx);
            thread::spawn(move || {
                let _ = ctx.check_network(&format!("api{}.example.com", i));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // All 10 events should be logged
    assert_eq!(logger.entries().len(), 10);
}

#[test]
fn test_security_context_clone_shares_logger() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx1 = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let ctx2 = ctx1.clone();

    let _ = ctx1.check_network("api1.example.com");
    let _ = ctx2.check_network("api2.example.com");

    // Both contexts share the same logger
    assert_eq!(logger.entries().len(), 2);
}

// ============================================================================
// Event Details Tests
// ============================================================================

#[test]
fn test_filesystem_read_denied_event_includes_path() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_filesystem_read(Path::new("/secret/data.txt"));

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();
    assert!(log_line.contains("/secret/data.txt"));
}

#[test]
fn test_network_denied_event_includes_host() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_network("malicious.com");

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();
    assert!(log_line.contains("malicious.com"));
}

#[test]
fn test_process_denied_event_includes_command() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_process("rm");

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();
    assert!(log_line.contains("rm"));
}

#[test]
fn test_environment_denied_event_includes_var() {
    let logger = Arc::new(MemoryAuditLogger::new());
    let ctx = SecurityContext::with_audit_logger(logger.clone() as Arc<dyn AuditLogger>);

    let _ = ctx.check_environment("SECRET_KEY");

    let entries = logger.entries();
    let log_line = entries[0].to_log_line();
    assert!(log_line.contains("SECRET_KEY"));
}
