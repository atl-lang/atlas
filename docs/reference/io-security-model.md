# Atlas I/O Security Model

**Version:** 1.0
**Status:** Specification
**Last Updated:** 2026-02-13

---

## Overview

Atlas follows a **secure-by-default** model for all I/O operations. All file system, network, and process operations are **denied by default** and require explicit opt-in via runtime flags.

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

### Permission Flag Pattern

All permission flags follow this pattern:

```bash
# Deny (default - no flag)
atlas run script.atl

# Allow all (no restrictions)
atlas run script.atl --allow-<type>

# Allow specific target only
atlas run script.atl --allow-<type>=<target>

# Allow multiple specific targets
atlas run script.atl --allow-<type>=<target1> --allow-<type>=<target2>
```

### Permission Flags

| Flag | Default | Unrestricted | Restricted | Multiple Targets |
|------|---------|--------------|------------|------------------|
| `--allow-read[=<path>]` | DENY all reads | `--allow-read` | `--allow-read=/data` | `--allow-read=/data --allow-read=/tmp` |
| `--allow-write[=<path>]` | DENY all writes | `--allow-write` | `--allow-write=/output` | `--allow-write=/out --allow-write=/logs` |
| `--allow-net[=<domain>]` | DENY all network | `--allow-net` | `--allow-net=api.example.com` | `--allow-net=api.com --allow-net=cdn.com` |
| `--allow-run[=<program>]` | DENY all execution | `--allow-run` | `--allow-run=git` | `--allow-run=git --allow-run=npm` |
| `--allow-env[=<var>]` | DENY all env vars | `--allow-env` | `--allow-env=PATH` | `--allow-env=PATH --allow-env=HOME` |
| `--allow-all` | DENY all | Grants ALL permissions (dev only) | N/A | N/A |

**Permission Details:**

- **`--allow-read`**: File reads (path + subdirectories if restricted)
- **`--allow-write`**: File writes, creates, deletes (path + subdirectories if restricted)
- **`--allow-net`**: HTTP, HTTPS, TCP, UDP (domain restrictions apply)
- **`--allow-run`**: Subprocess execution (exact program name match if restricted)
- **`--allow-env`**: Environment variable reads (exact var name match if restricted)
- **`--allow-all`**: **Development mode only** - grants all permissions, emits warning

**Example Usage:**
```bash
# Read from /data, write to /output, access GitHub API
atlas run script.atl --allow-read=/data --allow-write=/output --allow-net=api.github.com

# Run git and npm commands, read PATH env var
atlas run script.atl --allow-run=git --allow-run=npm --allow-env=PATH

# Development mode (unrestricted)
atlas run script.atl --allow-all  # ⚠️  Warning emitted
```

---

## Permission Enforcement

### Runtime Checks

**When I/O function is called:**
1. **Check permission:** Does current permission set allow this operation?
2. **Check target:** If granular permission, does target match allowlist?
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

# Denied (path traversal):
file::read("/tmp/../etc/passwd")     ❌  (resolves to /etc/passwd)
file::read("/etc/passwd")            ❌
file::read("/tmp-backup/data.txt")   ❌  (not a subdirectory)
```

---

## Domain Validation

**Domain Matching Rules:**
- Exact match: `api.github.com` matches only `api.github.com`
- Subdomains NOT included: `api.github.com` does NOT match `v3.api.github.com`
- Ports ignored: `api.github.com` matches `api.github.com:443` and `api.github.com:8080`
- Protocols ignored: `api.github.com` matches both HTTP and HTTPS

**URL Validation:**
```bash
# Allow access to api.github.com
atlas run script.atl --allow-net=api.github.com

# Allowed:
http::get("https://api.github.com/users")      ✅
http::get("http://api.github.com:8080/data")   ✅

# Denied:
http::get("https://v3.api.github.com/users")   ❌  (subdomain)
http::get("https://github.com/users")          ❌  (different domain)
```

---

## Security Considerations

### Development vs Production

**Development:**
- Use `--allow-all` for convenience
- Fast iteration, no permission prompts
- **Never deploy with `--allow-all`**

**Production:**
- Use minimal, granular permissions
- `--allow-read=/app/data --allow-net=api.example.com`
- Principle of least privilege

### Third-Party Code

**Risk:** Third-party libraries may contain malicious I/O operations

**Mitigation:**
1. Audit dependencies before use
2. Run untrusted code with minimal permissions
3. Use granular permissions (not `--allow-all`)
4. Monitor permission denials in logs

### Permission Auditing

**Log all permission denials:**
```
[2026-02-13 14:30:45] Permission denied: file read access to "/etc/passwd"
[2026-02-13 14:30:46] Permission denied: network access to "evil.com"
```

**Helps identify:**
- Malicious activity
- Missing permissions (false positives)
- Over-permissive grants

---

## Implementation Notes

**For Atlas VM/Interpreter:**

1. **Initialize permission set** from CLI flags at startup
2. **Check permissions** before every I/O syscall
3. **Normalize paths/domains** before comparing
4. **Raise AT0300/AT0301** on denial
5. **Log denials** for audit trail

**Performance:**
- Permission checks are fast (hash set lookup)
- Path normalization done once per operation
- No performance impact for granted operations

---

## Future Enhancements

**v0.3+ Considerations:**
- **Prompt mode:** Interactive permission requests (like mobile apps)
- **Permission files:** Save granted permissions to config file
- **Fine-grained network:** Port-level restrictions
- **Time-limited permissions:** Temporary grants that expire
- **Capability-based:** Pass I/O capabilities as function arguments

---

## References

**Similar Models:**
- **Deno:** Pioneer of secure-by-default runtime permissions
- **Android:** App permission system
- **Browser:** Origin-based security model
- **Flatpak:** Linux app sandboxing

**Security Research:**
- Principle of least privilege
- Capability-based security
- Confused deputy problem
