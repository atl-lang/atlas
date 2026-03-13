# Security Model

Atlas is secure by default. All I/O operations — filesystem access, network connections, process execution, environment variable reads, and FFI — are **denied** unless explicitly granted. Programs that do not require I/O run with zero capability and cannot be exploited via I/O injection.

## Design Principles

- **Capability-based:** permissions are specific grants (a path, a host, a command), not role flags.
- **Deny by default:** the absence of a grant is a denial, not an error to handle.
- **Auditable:** every permission check, denial, and sandbox event is optionally logged.
- **Composable:** multiple named policies can be loaded and merged.
- **Enforceable at runtime:** the `SecurityContext` is checked at every I/O call site inside the runtime.

## Permission Types

There are five permission categories:

| Category | Grants access to |
|----------|-----------------|
| `FilesystemRead` | Reading files and directories at a path |
| `FilesystemWrite` | Writing files and directories at a path |
| `Network` | Connecting to a host (domain or IP) |
| `Process` | Executing a specific command |
| `Environment` | Reading a specific environment variable |

Each permission is a specific capability, not a broad flag. Granting read access to `/data` does not grant write access to `/data`, and does not grant access to `/etc`.

## SecurityContext

The `SecurityContext` is the runtime's permission authority. It holds the active permission sets for all five categories and is queried before every I/O operation.

### Default State

```atlas
// All permissions denied — no I/O possible
let ctx = SecurityContext.new();
```

The default context has zero permissions. Any I/O attempt against it returns a `SecurityError`.

### Granting Permissions

```atlas
// Grant filesystem read to a specific directory (recursive = includes subdirectories)
ctx.grant_filesystem_read("/data", true);

// Grant filesystem write to a specific file only (recursive = false)
ctx.grant_filesystem_write("/tmp/output.txt", false);

// Grant network access to a specific host
ctx.grant_network("api.example.com");

// Grant network access to a whole domain via wildcard
ctx.grant_network("*.example.com");

// Grant network access to all hosts (use carefully)
ctx.grant_network("*");

// Grant process execution for a specific command
ctx.grant_process("git");

// Grant process execution for any command
ctx.grant_process("*");

// Grant environment variable access
ctx.grant_environment("HOME");
ctx.grant_environment("*");  // all env vars
```

Path permissions are canonicalized before storage. Symlinks are resolved when the path exists; if it does not yet exist (e.g. a write target), the path is made absolute without symlink resolution.

### Checking Permissions

The runtime calls check methods before performing I/O. User code does not call these directly — they are enforced internally. The methods are:

```
ctx.check_filesystem_read(path)   -> Ok(()) or SecurityError
ctx.check_filesystem_write(path)  -> Ok(()) or SecurityError
ctx.check_network(host)           -> Ok(()) or SecurityError
ctx.check_process(command)        -> Ok(()) or SecurityError
ctx.check_environment(var)        -> Ok(()) or SecurityError
```

A denied check returns a typed `SecurityError` with the denied path/host/command as context.

### Preset Contexts

Two presets are available for development and testing:

```atlas
// Allow everything — for development only, not secure
let ctx = SecurityContext.allow_all();

// Allow filesystem, process, and env — network still denied
// Used in test suites
let ctx = SecurityContext.test_mode();
```

Never use `allow_all()` in production or in code that handles untrusted input.

## Filesystem Permission Matching

- **Exact match:** granting `/data/file.txt` allows only that exact file.
- **Recursive:** granting `/data` with `recursive = true` allows all files under `/data/` including subdirectories.
- Non-recursive grants to a directory do not grant access to any file inside it — only the directory entry itself.

## Network Permission Matching

- **Exact host:** `"api.example.com"` allows only that host.
- **Wildcard subdomain:** `"*.example.com"` allows `api.example.com`, `cdn.example.com`, etc. but not `example.com` itself.
- **All hosts:** `"*"` allows any connection.

## Policy Files

Permissions can be expressed declaratively in TOML or JSON policy files and loaded at runtime.

### TOML Policy Format

```toml
name = "my-app-policy"
description = "Permissions for the data processing app"
default_action = "deny"

[[allow]]
resource = "file-read"
pattern = "/data"
scope = "recursive"

[[allow]]
resource = "file-write"
pattern = "/tmp"
scope = "recursive"

[[allow]]
resource = "network-connect"
pattern = "*.example.com"

[[allow]]
resource = "environment"
pattern = "DATABASE_URL"

# Explicit deny rules override allow rules
[[deny]]
resource = "file-read"
pattern = "/data/secrets"
```

### JSON Policy Format

```json
{
  "name": "my-app-policy",
  "default_action": "deny",
  "allow": [
    { "resource": "file-read", "pattern": "/data", "scope": "recursive" },
    { "resource": "network-connect", "pattern": "api.example.com" }
  ],
  "deny": [
    { "resource": "file-read", "pattern": "/data/secrets" }
  ]
}
```

### Resource Types in Policies

| Policy resource name | Permission category |
|---------------------|---------------------|
| `file-read` | FilesystemRead |
| `file-write` | FilesystemWrite |
| `file-delete` | (future) |
| `network-connect` | Network |
| `network-listen` | (future) |
| `ffi` | FFI |
| `process` | Process |
| `environment` | Environment |
| `reflection` | (future) |

### Evaluation Order

Deny rules have higher priority than allow rules. Evaluation:

1. Check deny rules — if any match, deny.
2. Check allow rules — if any match, allow.
3. Apply `default_action` (default is `"deny"`).

### Policy Inheritance

Policies can inherit from other named policies:

```toml
name = "extended-policy"
inherits = ["base-policy"]

[[allow]]
resource = "network-connect"
pattern = "extra.example.com"
```

Inherited permissions are merged into the inheriting policy's permission set.

### PolicyManager

The `PolicyManager` loads and stores named policies and resolves inheritance:

```atlas
// Load a policy
let manager = PolicyManager.new();
manager.load_policy(policy);

// Get resolved permission set (with inheritance)
let perms = manager.get_permissions("my-app-policy");
```

## Sandbox

The `Sandbox` combines a `PermissionSet` with resource quotas for running untrusted code with hard resource limits.

### Resource Quotas

Two preset quota levels are available:

**Restrictive** (default for untrusted code):
- Memory: 64 MB
- CPU time: 5 seconds
- Stack depth: 1000 frames
- File descriptors: 10
- Network connections: 5
- Disk I/O: 10 MB

**Permissive** (for semi-trusted code):
- Memory: 1 GB
- CPU time: 5 minutes
- Stack depth: 10000 frames
- File descriptors: 1000
- Network connections: 100
- Disk I/O: 1 GB

**Unlimited** (development/testing only — no limits enforced).

### Creating a Sandbox

```atlas
// Restrictive sandbox — no permissions, tight quotas
let sb = Sandbox.restrictive("my-sandbox");

// Grant specific permissions to the sandbox
sb.grant_permission(Permission.FilesystemRead { path: "/data", recursive: true });
```

Quota violations produce `SandboxError` variants: `MemoryQuotaExceeded`, `CpuTimeQuotaExceeded`, `StackDepthExceeded`, `FileDescriptorQuotaExceeded`, `NetworkConnectionQuotaExceeded`, `DiskIOQuotaExceeded`.

## Audit Logging

Every permission check and sandbox event can be logged. Audit events are structured and typed.

### Audit Events

| Event | Triggered by |
|-------|-------------|
| `PermissionCheck` | Any permission check (granted or denied) |
| `FilesystemReadDenied` | Denied filesystem read |
| `FilesystemWriteDenied` | Denied filesystem write |
| `NetworkDenied` | Denied network access |
| `ProcessDenied` | Denied process execution |
| `EnvironmentDenied` | Denied env var access |
| `SandboxCreated` | New sandbox instantiated |
| `SandboxDestroyed` | Sandbox dropped |
| `PolicyViolation` | Policy rule violated |
| `QuotaViolation` | Resource quota exceeded |
| `PrivilegeEscalation` | Escalation attempt detected |
| `CapabilityGranted` | Capability added to context |
| `CapabilityRevoked` | Capability removed |

### Logger Implementations

**`NullAuditLogger`** — no-op (default). Zero overhead for production deployments that do not require audit trails.

**`MemoryAuditLogger`** — stores all entries in memory. Suitable for testing and short-lived programs. Entries are accessible for inspection.

Custom loggers can be implemented by satisfying the `AuditLogger` trait (which requires `log`, `entries`, and `clear` methods and is `Send + Sync`).

### Attaching a Logger

```atlas
let logger = MemoryAuditLogger.new();
let ctx = SecurityContext.with_audit_logger(logger);
```

Each audit entry carries a Unix millisecond timestamp and the full event details. Log lines are formatted as:

```
[<timestamp>] Permission DENIED: file read access to /etc/passwd
[<timestamp>] Sandbox created: my-sandbox (memory: Some(67108864), cpu: Some(5000))
```

## Security Error Types

All permission denials produce a `SecurityError` with a descriptive message and the denied resource:

| Error variant | Produced when |
|--------------|---------------|
| `FilesystemReadDenied { path }` | Read check fails |
| `FilesystemWriteDenied { path }` | Write check fails |
| `NetworkDenied { host }` | Network check fails |
| `ProcessDenied { command }` | Process check fails |
| `EnvironmentDenied { var }` | Env check fails |
| `InvalidPath(msg)` | Malformed path |
| `InvalidPattern(msg)` | Malformed permission pattern |

## Summary

| Concept | Description |
|---------|-------------|
| `SecurityContext` | Holds active permission sets; queried before I/O |
| `Permission` | A specific capability grant (path, host, command, variable) |
| `PermissionSet` | Collection of granted permissions |
| `SecurityPolicy` | Declarative allow/deny rules loadable from TOML or JSON |
| `PolicyManager` | Loads and resolves named policies with inheritance |
| `Sandbox` | Combines permissions with hard resource quotas |
| `AuditLogger` | Structured logging of all security events |
| Default | All I/O denied; must be explicitly granted |
