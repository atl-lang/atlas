//! File I/O standard library functions
//!
//! Provides file and directory operations with security checks and UTF-8 validation.
//! All operations respect the SecurityContext permission model.

use crate::security::SecurityContext;
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::fs;
use std::path::{Path, PathBuf};

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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let path = PathBuf::from(path_str);
    let abs_path = path.canonicalize().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to resolve path '{}': {}", path_str, e),
        span,
    })?;

    // Check permission
    security.check_filesystem_read(&abs_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "file read".to_string(),
            path: abs_path.display().to_string(),
            span,
        }
    })?;

    // Read file
    let contents = fs::read_to_string(&abs_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read file '{}': {}", abs_path.display(), e),
        span,
    })?;

    Ok(Value::string(contents))
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let contents = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let path = PathBuf::from(path_str);

    // For write operations, check permission on the parent directory if file doesn't exist
    let check_path = if path.exists() {
        path.canonicalize().map_err(|e| RuntimeError::IoError {
            message: format!("Failed to resolve path '{}': {}", path_str, e),
            span,
        })?
    } else {
        // Check parent directory permission
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        parent.canonicalize().map_err(|e| RuntimeError::IoError {
            message: format!("Failed to resolve parent path: {}", e),
            span,
        })?
    };

    // Check permission
    security.check_filesystem_write(&check_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "file write".to_string(),
            path: check_path.display().to_string(),
            span,
        }
    })?;

    // Write file
    fs::write(&path, contents).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to write file '{}': {}", path_str, e),
        span,
    })?;

    Ok(Value::Null)
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let contents = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let path = PathBuf::from(path_str);

    // Check permission (same logic as write_file)
    let check_path = if path.exists() {
        path.canonicalize().map_err(|e| RuntimeError::IoError {
            message: format!("Failed to resolve path '{}': {}", path_str, e),
            span,
        })?
    } else {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        parent.canonicalize().map_err(|e| RuntimeError::IoError {
            message: format!("Failed to resolve parent path: {}", e),
            span,
        })?
    };

    security.check_filesystem_write(&check_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "file write".to_string(),
            path: check_path.display().to_string(),
            span,
        }
    })?;

    // Append to file
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| RuntimeError::IoError {
            message: format!("Failed to open file '{}': {}", path_str, e),
            span,
        })?;

    file.write_all(contents.as_bytes())
        .map_err(|e| RuntimeError::IoError {
            message: format!("Failed to append to file '{}': {}", path_str, e),
            span,
        })?;

    Ok(Value::Null)
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
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

    let abs_check = check_path
        .canonicalize()
        .map_err(|e| RuntimeError::IoError {
            message: format!("Failed to resolve path: {}", e),
            span,
        })?;

    // Check permission on existing ancestor
    security.check_filesystem_write(&abs_check).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "directory create".to_string(),
            path: abs_check.display().to_string(),
            span,
        }
    })?;

    // Create directory
    fs::create_dir_all(&path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to create directory '{}': {}", path_str, e),
        span,
    })?;

    Ok(Value::Null)
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let path = PathBuf::from(path_str);
    let abs_path = path.canonicalize().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to resolve path '{}': {}", path_str, e),
        span,
    })?;

    // Check permission
    security.check_filesystem_write(&abs_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "file delete".to_string(),
            path: abs_path.display().to_string(),
            span,
        }
    })?;

    // Remove file
    fs::remove_file(&abs_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to remove file '{}': {}", abs_path.display(), e),
        span,
    })?;

    Ok(Value::Null)
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
    };

    let path = PathBuf::from(path_str);
    let abs_path = path.canonicalize().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to resolve path '{}': {}", path_str, e),
        span,
    })?;

    // Check permission
    security.check_filesystem_write(&abs_path).map_err(|_| {
        RuntimeError::FilesystemPermissionDenied {
            operation: "directory delete".to_string(),
            path: abs_path.display().to_string(),
            span,
        }
    })?;

    // Remove directory (must be empty)
    fs::remove_dir(&abs_path).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to remove directory '{}': {}", abs_path.display(), e),
        span,
    })?;

    Ok(Value::Null)
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let path_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
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

    Ok(Value::JsonValue(std::rc::Rc::new(JsonValue::Object(info))))
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
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let mut path = PathBuf::new();
    for arg in args {
        match arg {
            Value::String(s) => path.push(s.as_ref()),
            _ => return Err(RuntimeError::InvalidStdlibArgument { span }),
        }
    }

    Ok(Value::string(path.to_string_lossy().to_string()))
}
