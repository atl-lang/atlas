//! File I/O and stdin standard library functions
//!
//! Provides file and directory operations with security checks and UTF-8 validation,
//! plus stdin reading for interactive CLI programs.
//! All operations respect the SecurityContext permission model.

use super::{stdlib_arg_error, stdlib_arity_error};
use crate::security::SecurityContext;
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};

/// Read one line from stdin (strips trailing newline).
///
/// Blocks until the user presses Enter. Returns `Some(line)` on success,
/// `None` on EOF. This prevents infinite loops when stdin is exhausted.
pub fn io_read_line(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("io.readLine", 0, args.len(), span));
    }
    let mut line = String::new();
    let bytes_read =
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|e| RuntimeError::IoError {
                message: format!("Failed to read from stdin: {}", e),
                span,
            })?;

    // EOF: read_line returns Ok(0) when stdin is exhausted
    if bytes_read == 0 {
        return Ok(Value::Option(None));
    }

    // Strip trailing \n or \r\n
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(Value::Option(Some(Box::new(Value::string(line)))))
}

/// Print a prompt then read one line from stdin (strips trailing newline).
///
/// Flushes stdout before reading so the prompt appears immediately.
/// Returns `Some(line)` on success, `None` on EOF.
pub fn io_read_line_prompt(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("io.readLinePrompt", 1, args.len(), span));
    }
    let prompt = match &args[0] {
        Value::String(s) => s.as_ref().to_string(),
        _ => {
            return Err(stdlib_arg_error(
                "io.readLinePrompt",
                "string",
                &args[0],
                span,
            ))
        }
    };
    use std::io::Write;
    print!("{}", prompt);
    std::io::stdout()
        .flush()
        .map_err(|e| RuntimeError::IoError {
            message: format!("Failed to flush stdout: {}", e),
            span,
        })?;
    let mut line = String::new();
    let bytes_read =
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|e| RuntimeError::IoError {
                message: format!("Failed to read from stdin: {}", e),
                span,
            })?;

    // EOF: read_line returns Ok(0) when stdin is exhausted
    if bytes_read == 0 {
        return Ok(Value::Option(None));
    }

    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(Value::Option(Some(Box::new(Value::string(line)))))
}

/// Read entire file as UTF-8 string
///
/// Checks read permission, validates UTF-8 encoding.
/// Returns error if file not found, permission denied, or invalid UTF-8.
pub fn read_file(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("readFile", 1, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("readFile", "string", &args[0], span)),
    };

    let path = PathBuf::from(path_str);
    let abs_path = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "File.read: path '{}' not found or inaccessible: {}",
                path_str, e
            ))))));
        }
    };

    // Check permission
    if security.check_filesystem_read(&abs_path).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.read: permission denied for '{}'",
            abs_path.display()
        ))))));
    }

    // Read file
    match fs::read_to_string(&abs_path) {
        Ok(contents) => Ok(Value::Result(Ok(Box::new(Value::string(contents))))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.read: failed to read '{}': {}",
            abs_path.display(),
            e
        )))))),
    }
}

/// Write string to file (create or overwrite)
///
/// Checks write permission. Creates file if it doesn't exist.
pub fn write_file(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("writeFile", 2, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("writeFile", "string", &args[0], span)),
    };

    let contents = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("writeFile", "string", &args[1], span)),
    };

    let path = PathBuf::from(path_str);

    // For write operations, check permission on the parent directory if file doesn't exist
    let check_path = if path.exists() {
        match path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return Ok(Value::Result(Err(Box::new(Value::string(format!(
                    "File.write: cannot resolve path '{}': {}",
                    path_str, e
                ))))));
            }
        }
    } else {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        match parent.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return Ok(Value::Result(Err(Box::new(Value::string(format!(
                    "File.write: cannot resolve parent path: {}",
                    e
                ))))));
            }
        }
    };

    // Check permission
    if security.check_filesystem_write(&check_path).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.write: permission denied for '{}'",
            check_path.display()
        ))))));
    }

    // Write file
    match fs::write(&path, contents) {
        Ok(()) => Ok(Value::Result(Ok(Box::new(Value::Null)))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.write: failed to write '{}': {}",
            path_str, e
        )))))),
    }
}

/// Append string to end of file (create if doesn't exist)
///
/// Checks write permission. Creates file if it doesn't exist.
pub fn append_file(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("appendFile", 2, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("appendFile", "string", &args[0], span)),
    };

    let contents = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("appendFile", "string", &args[1], span)),
    };

    let path = PathBuf::from(path_str);

    // Check permission (same logic as write_file)
    let check_path = if path.exists() {
        match path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return Ok(Value::Result(Err(Box::new(Value::string(format!(
                    "File.append: cannot resolve path '{}': {}",
                    path_str, e
                ))))));
            }
        }
    } else {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        match parent.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return Ok(Value::Result(Err(Box::new(Value::string(format!(
                    "File.append: cannot resolve parent path: {}",
                    e
                ))))));
            }
        }
    };

    if security.check_filesystem_write(&check_path).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.append: permission denied for '{}'",
            check_path.display()
        ))))));
    }

    // Append to file
    use std::io::Write;
    let mut file = match fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "File.append: failed to open '{}': {}",
                path_str, e
            ))))));
        }
    };

    match file.write_all(contents.as_bytes()) {
        Ok(()) => Ok(Value::Result(Ok(Box::new(Value::Null)))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.append: failed to write '{}': {}",
            path_str, e
        )))))),
    }
}

/// Check if file or directory exists
///
/// No permission check needed - just checks existence without reading.
pub fn file_exists(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("fileExists", 1, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("fileExists", "string", &args[0], span)),
    };

    let path = Path::new(path_str);
    Ok(Value::Bool(path.exists()))
}

/// Read directory contents as array of filenames
///
/// Checks read permission. Returns array of strings (filenames only, not full paths).
pub fn read_dir(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("readDir", 1, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("readDir", "string", &args[0], span)),
    };

    let path = PathBuf::from(path_str);
    let abs_path = path.canonicalize().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to resolve path '{}': {}", path_str, e),
        span,
    })?;

    // Check permission
    security.check_filesystem_read(&abs_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "directory read".to_string(),
            path: abs_path.display().to_string(),
            span,
        }
    })?;

    // Read directory
    let entries = fs::read_dir(&abs_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read directory '{}': {}", abs_path.display(), e),
        span,
    })?;

    let mut filenames = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read directory entry: {}", e),
            span,
        })?;

        let filename = entry.file_name().to_string_lossy().to_string();
        filenames.push(Value::string(filename));
    }

    Ok(Value::array(filenames))
}

/// Create directory (mkdir -p behavior - creates parents if needed)
///
/// Checks write permission. For nested paths, finds the first existing ancestor.
pub fn create_dir(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("createDir", 1, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("createDir", "string", &args[0], span)),
    };

    let path = PathBuf::from(path_str);

    // Find first existing ancestor for permission check
    let mut check_path = path.as_path();
    while !check_path.exists() {
        check_path = match check_path.parent() {
            Some(p) => p,
            None => Path::new("."),
        };
    }

    let abs_check = match check_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "File.createDir: cannot resolve ancestor path: {}",
                e
            ))))));
        }
    };

    // Check permission on existing ancestor
    if security.check_filesystem_write(&abs_check).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.createDir: permission denied for '{}'",
            abs_check.display()
        ))))));
    }

    // Create directory
    match fs::create_dir_all(&path) {
        Ok(()) => Ok(Value::Result(Ok(Box::new(Value::Null)))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.createDir: failed to create '{}': {}",
            path_str, e
        )))))),
    }
}

/// Remove file (not directory)
///
/// Checks write permission. Error if path is a directory.
pub fn remove_file(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("removeFile", 1, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("removeFile", "string", &args[0], span)),
    };

    let path = PathBuf::from(path_str);
    let abs_path = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "File.remove: path '{}' not found: {}",
                path_str, e
            ))))));
        }
    };

    // Check permission
    if security.check_filesystem_write(&abs_path).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.remove: permission denied for '{}'",
            abs_path.display()
        ))))));
    }

    // Remove file
    match fs::remove_file(&abs_path) {
        Ok(()) => Ok(Value::Result(Ok(Box::new(Value::Null)))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.remove: failed to remove '{}': {}",
            abs_path.display(),
            e
        )))))),
    }
}

/// Remove empty directory
///
/// Checks write permission. Error if directory is not empty or path is a file.
pub fn remove_dir(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("removeDir", 1, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("removeDir", "string", &args[0], span)),
    };

    let path = PathBuf::from(path_str);
    let abs_path = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "File.removeDir: path '{}' not found: {}",
                path_str, e
            ))))));
        }
    };

    // Check permission
    if security.check_filesystem_write(&abs_path).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.removeDir: permission denied for '{}'",
            abs_path.display()
        ))))));
    }

    // Remove directory (must be empty)
    match fs::remove_dir(&abs_path) {
        Ok(()) => Ok(Value::Result(Ok(Box::new(Value::Null)))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "File.removeDir: failed to remove '{}': {}",
            abs_path.display(),
            e
        )))))),
    }
}

/// Rename/move a file or directory
///
/// Checks write permission on both source and destination.
/// Works across directories on the same filesystem.
pub fn rename_file(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("file.rename", 2, args.len(), span));
    }

    let src_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("file.rename", "string", &args[0], span)),
    };

    let dst_str = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("file.rename", "string", &args[1], span)),
    };

    let src_path = PathBuf::from(src_str);
    let dst_path = PathBuf::from(dst_str);

    // Source must exist
    let abs_src = match src_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "file.rename: source '{}' not found: {}",
                src_str, e
            ))))));
        }
    };

    // Check write permission on source
    if security.check_filesystem_write(&abs_src).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "file.rename: permission denied for source '{}'",
            abs_src.display()
        ))))));
    }

    // Check write permission on destination parent
    let dst_parent = dst_path.parent().unwrap_or_else(|| Path::new("."));
    let abs_dst_parent = match dst_parent.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "file.rename: destination parent not found: {}",
                e
            ))))));
        }
    };

    if security.check_filesystem_write(&abs_dst_parent).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "file.rename: permission denied for destination '{}'",
            abs_dst_parent.display()
        ))))));
    }

    // Perform rename
    match fs::rename(&src_path, &dst_path) {
        Ok(()) => Ok(Value::Result(Ok(Box::new(Value::Null)))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "file.rename: failed to rename '{}' to '{}': {}",
            src_str, dst_str, e
        )))))),
    }
}

/// Copy a file
///
/// Checks read permission on source, write permission on destination.
/// Copies file contents and permissions. Does not copy directories.
pub fn copy_file(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("file.copy", 2, args.len(), span));
    }

    let src_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("file.copy", "string", &args[0], span)),
    };

    let dst_str = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("file.copy", "string", &args[1], span)),
    };

    let src_path = PathBuf::from(src_str);
    let dst_path = PathBuf::from(dst_str);

    // Source must exist
    let abs_src = match src_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "file.copy: source '{}' not found: {}",
                src_str, e
            ))))));
        }
    };

    // Check read permission on source
    if security.check_filesystem_read(&abs_src).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "file.copy: permission denied for source '{}'",
            abs_src.display()
        ))))));
    }

    // Check write permission on destination parent
    let dst_parent = dst_path.parent().unwrap_or_else(|| Path::new("."));
    let abs_dst_parent = match dst_parent.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "file.copy: destination parent not found: {}",
                e
            ))))));
        }
    };

    if security.check_filesystem_write(&abs_dst_parent).is_err() {
        return Ok(Value::Result(Err(Box::new(Value::string(format!(
            "file.copy: permission denied for destination '{}'",
            abs_dst_parent.display()
        ))))));
    }

    // Perform copy
    match fs::copy(&src_path, &dst_path) {
        Ok(_) => Ok(Value::Result(Ok(Box::new(Value::Null)))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "file.copy: failed to copy '{}' to '{}': {}",
            src_str, dst_str, e
        )))))),
    }
}

/// Get file metadata (size, modified time, is_file, is_dir)
///
/// Checks read permission. Returns object with metadata fields.
pub fn file_info(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("fileInfo", 1, args.len(), span));
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(stdlib_arg_error("fileInfo", "string", &args[0], span)),
    };

    let path = PathBuf::from(path_str);
    let abs_path = path.canonicalize().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to resolve path '{}': {}", path_str, e),
        span,
    })?;

    // Check permission
    security.check_filesystem_read(&abs_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "file metadata".to_string(),
            path: abs_path.display().to_string(),
            span,
        }
    })?;

    // Get metadata
    let metadata = fs::metadata(&abs_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get metadata for '{}': {}", abs_path.display(), e),
        span,
    })?;

    // Convert modified time to seconds since Unix epoch
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|dur| dur.as_secs() as f64)
        .unwrap_or(0.0);

    use crate::json_value::JsonValue;
    let mut info = std::collections::HashMap::new();
    info.insert("size".to_string(), JsonValue::Number(metadata.len() as f64));
    info.insert("modified".to_string(), JsonValue::Number(modified));
    info.insert("isFile".to_string(), JsonValue::Bool(metadata.is_file()));
    info.insert("isDir".to_string(), JsonValue::Bool(metadata.is_dir()));

    Ok(Value::JsonValue(std::sync::Arc::new(JsonValue::Object(
        info,
    ))))
}

/// Join path components with OS-specific separator
///
/// No permission check needed - just string manipulation.
/// Cross-platform: uses \ on Windows, / on Unix.
pub fn path_join(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(stdlib_arity_error("pathJoin", 1, args.len(), span));
    }

    let mut path = PathBuf::new();
    for (i, arg) in args.iter().enumerate() {
        match arg {
            Value::String(s) => path.push(s.as_ref()),
            _ => return Err(stdlib_arg_error("pathJoin", "string", &args[i], span)),
        }
    }

    Ok(Value::string(path.to_string_lossy().to_string()))
}

// ── B26: io namespace completion ──────────────────────────────────────────────

/// Write a string to stdout without a trailing newline.
///
/// io.write(str) — maps to ioNsWrite in the stdlib registry.
pub fn io_write(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("io.write", 1, args.len(), span));
    }
    let s = match &args[0] {
        Value::String(s) => s.as_ref().to_string(),
        _ => return Err(stdlib_arg_error("io.write", "string", &args[0], span)),
    };
    use std::io::Write;
    print!("{}", s);
    std::io::stdout().flush().ok();
    Ok(Value::Null)
}

/// Write a string to stdout followed by a newline.
///
/// io.writeLine(str) — maps to ioNsWriteLine in the stdlib registry.
pub fn io_write_line(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("io.writeLine", 1, args.len(), span));
    }
    let s = match &args[0] {
        Value::String(s) => s.as_ref().to_string(),
        _ => return Err(stdlib_arg_error("io.writeLine", "string", &args[0], span)),
    };
    println!("{}", s);
    Ok(Value::Null)
}

/// Read all of stdin until EOF and return it as a string.
///
/// io.readAll() — maps to ioNsReadAll in the stdlib registry.
pub fn io_read_all(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("io.readAll", 0, args.len(), span));
    }
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut buf)
        .map_err(|e| RuntimeError::IoError {
            message: format!("Failed to read all from stdin: {}", e),
            span,
        })?;
    Ok(Value::string(buf))
}

/// Flush the stdout buffer.
///
/// io.flush() — maps to ioNsFlush in the stdlib registry.
pub fn io_flush(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("io.flush", 0, args.len(), span));
    }
    use std::io::Write;
    std::io::stdout()
        .flush()
        .map_err(|e| RuntimeError::IoError {
            message: format!("Failed to flush stdout: {}", e),
            span,
        })?;
    Ok(Value::Null)
}
