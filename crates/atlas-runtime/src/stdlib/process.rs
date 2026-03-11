//! Process management stdlib functions
//!
//! This module provides Atlas stdlib functions for spawning external processes,
//! capturing output, managing environment variables, and controlling working directories.
//!
//! Command execution:
//! - exec: Execute command and wait for completion
//! - spawn: Spawn child process (non-blocking)
//! - shell: Execute shell command
//!
//! Standard I/O:
//! - execCapture: Execute and capture stdout/stderr
//! - execInherit: Execute with inherited stdio
//!
//! Environment variables:
//! - getEnv: Get environment variable
//! - setEnv: Set environment variable
//! - unsetEnv: Remove environment variable
//! - listEnv: List all environment variables
//!
//! Working directory:
//! - getCwd: Get current working directory
//! - setCwd: Set working directory for process
//!
//! Process control:
//! - processWait: Wait for process completion
//! - processKill: Kill running process
//! - processPid: Get current process ID

use super::stdlib_arity_error;
use crate::security::SecurityContext;
use crate::span::Span;
use crate::value::{RuntimeError, Value, ValueArray};
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

// ============================================================================
// ProcessOutput — typed result of exec() / shell()
// ============================================================================

/// Typed output from a completed process.
///
/// Returned by `Process.exec()` and `Process.shell()`.
/// Methods: `.stdout()`, `.stderr()`, `.exitCode()`, `.success()`
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl ProcessOutput {
    pub fn new(stdout: String, stderr: String, exit_code: i32, success: bool) -> Self {
        Self {
            stdout,
            stderr,
            exit_code,
            success,
        }
    }

    fn from_std_output(output: &Output) -> Self {
        let exit_code = output.status.code().unwrap_or(-1);
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            success: output.status.success(),
        }
    }
}

/// Process.exec() / Process.shell() output: .stdout() -> string
pub fn process_output_stdout(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error(
            "processOutputStdout",
            1,
            args.len(),
            span,
        ));
    }
    match &args[0] {
        Value::ProcessOutput(out) => Ok(Value::string(out.stdout.clone())),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "processOutputStdout: expected ProcessOutput, got {}",
                args[0].type_name()
            ),
            span,
        }),
    }
}

/// Process.exec() / Process.shell() output: .stderr() -> string
pub fn process_output_stderr(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error(
            "processOutputStderr",
            1,
            args.len(),
            span,
        ));
    }
    match &args[0] {
        Value::ProcessOutput(out) => Ok(Value::string(out.stderr.clone())),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "processOutputStderr: expected ProcessOutput, got {}",
                args[0].type_name()
            ),
            span,
        }),
    }
}

/// Process.exec() / Process.shell() output: .exitCode() -> number
pub fn process_output_exit_code(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error(
            "processOutputExitCode",
            1,
            args.len(),
            span,
        ));
    }
    match &args[0] {
        Value::ProcessOutput(out) => Ok(Value::Number(out.exit_code as f64)),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "processOutputExitCode: expected ProcessOutput, got {}",
                args[0].type_name()
            ),
            span,
        }),
    }
}

/// Process.exec() / Process.shell() output: .success() -> bool
pub fn process_output_success(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error(
            "processOutputSuccess",
            1,
            args.len(),
            span,
        ));
    }
    match &args[0] {
        Value::ProcessOutput(out) => Ok(Value::Bool(out.success)),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "processOutputSuccess: expected ProcessOutput, got {}",
                args[0].type_name()
            ),
            span,
        }),
    }
}

static PROCESS_REGISTRY: OnceLock<Arc<Mutex<HashMap<u32, Child>>>> = OnceLock::new();
static PROCESS_IO_REGISTRY: OnceLock<Mutex<HashMap<u32, ProcessIoHandles>>> = OnceLock::new();
static PROCESS_STDIN_HANDLES: OnceLock<Mutex<HashMap<u64, Arc<Mutex<ChildStdin>>>>> =
    OnceLock::new();
static PROCESS_STDOUT_HANDLES: OnceLock<Mutex<HashMap<u64, Arc<Mutex<ChildStdout>>>>> =
    OnceLock::new();
static PROCESS_STDERR_HANDLES: OnceLock<Mutex<HashMap<u64, Arc<Mutex<ChildStderr>>>>> =
    OnceLock::new();
static NEXT_PROCESS_IO_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Default, Clone, Copy)]
struct ProcessIoHandles {
    stdin: Option<u64>,
    stdout: Option<u64>,
    stderr: Option<u64>,
}

fn process_registry() -> Arc<Mutex<HashMap<u32, Child>>> {
    PROCESS_REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

fn process_io_registry() -> &'static Mutex<HashMap<u32, ProcessIoHandles>> {
    PROCESS_IO_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn process_stdin_handles() -> &'static Mutex<HashMap<u64, Arc<Mutex<ChildStdin>>>> {
    PROCESS_STDIN_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn process_stdout_handles() -> &'static Mutex<HashMap<u64, Arc<Mutex<ChildStdout>>>> {
    PROCESS_STDOUT_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn process_stderr_handles() -> &'static Mutex<HashMap<u64, Arc<Mutex<ChildStderr>>>> {
    PROCESS_STDERR_HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn make_io_handle(tag: &str, id: u64) -> Value {
    Value::Array(ValueArray::from_vec(vec![
        Value::string(tag.to_string()),
        Value::Number(id as f64),
    ]))
}

const PROCESS_STDIN_TAG: &str = "__process_stdin__";
const PROCESS_STDOUT_TAG: &str = "__process_stdout__";
const PROCESS_STDERR_TAG: &str = "__process_stderr__";

// ============================================================================
// Command Execution
// ============================================================================

/// Execute a command and wait for completion
///
/// Atlas signature: `exec(command: string | []string, options?: object) -> Result<object, string>`
///
/// Options:
/// - env: object - Custom environment variables
/// - cwd: string - Working directory
/// - inherit: bool - Inherit parent stdio (default: false)
///
/// Returns: { exitCode: number, stdout: string, stderr: string }
pub fn exec(args: &[Value], span: Span, security: &SecurityContext) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(stdlib_arity_error("exec", 1, args.len(), span));
    }

    // Parse command
    let (program, command_args) = parse_command(&args[0], span)?;

    // Check permission
    security
        .check_process(&program)
        .map_err(|_| RuntimeError::ProcessPermissionDenied {
            command: program.clone(),
            span,
        })?;

    // Parse options
    let options = if args.len() == 2 {
        parse_exec_options(&args[1], span)?
    } else {
        ExecOptions::default()
    };

    // Build command
    let mut cmd = Command::new(&program);
    cmd.args(&command_args);

    // Set environment if provided
    if let Some(env_vars) = &options.env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }

    // Set working directory if provided
    if let Some(cwd) = &options.cwd {
        cmd.current_dir(cwd);
    }

    // Set stdio handling
    if options.inherit {
        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());
    } else {
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
    }

    // Execute command
    let output = cmd.output().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to execute command: {}", e),
        span,
    })?;

    // Return typed ProcessOutput
    let process_out = ProcessOutput::from_std_output(&output);
    Ok(Value::Result(Ok(Box::new(Value::ProcessOutput(Arc::new(
        process_out,
    ))))))
}

/// Execute a shell command and return stdout as a string
///
/// Simpler variant of `shell()` for cases where only stdout is needed.
/// Atlas signature: `shellOut(command: string) -> Result<string, string>`
/// Returns Ok(stdout) on exit code 0, Err(stderr) otherwise.
pub fn shell_out(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("shellOut", 1, args.len(), span));
    }

    let command_str = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected string for command, got {}", args[0].type_name()),
                span,
            })
        }
    };

    let (shell_cmd, shell_arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    security
        .check_process(shell_cmd)
        .map_err(|_| RuntimeError::ProcessPermissionDenied {
            command: shell_cmd.to_string(),
            span,
        })?;

    let output = std::process::Command::new(shell_cmd)
        .arg(shell_arg)
        .arg(&command_str)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| RuntimeError::IoError {
            message: format!("Failed to execute shell command: {}", e),
            span,
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Value::Result(Ok(Box::new(Value::string(stdout)))))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let msg = if stderr.trim().is_empty() {
            format!(
                "Shell command failed with exit code {}",
                output.status.code().unwrap_or(-1)
            )
        } else {
            stderr.trim().to_string()
        };
        Ok(Value::Result(Err(Box::new(Value::string(msg)))))
    }
}

/// Execute a shell command
///
/// Atlas signature: `shell(command: string, options?: object) -> Result<object, string>`
pub fn shell(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(stdlib_arity_error("shell", 1, args.len(), span));
    }

    let command_str = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected string for command, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Detect shell
    let (shell_cmd, shell_arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    // Check permission for shell
    security
        .check_process(shell_cmd)
        .map_err(|_| RuntimeError::ProcessPermissionDenied {
            command: shell_cmd.to_string(),
            span,
        })?;

    // Parse options
    let options = if args.len() == 2 {
        parse_exec_options(&args[1], span)?
    } else {
        ExecOptions::default()
    };

    // Build command
    let mut cmd = Command::new(shell_cmd);
    cmd.arg(shell_arg).arg(&command_str);

    // Set environment if provided
    if let Some(env_vars) = &options.env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }

    // Set working directory if provided
    if let Some(cwd) = &options.cwd {
        cmd.current_dir(cwd);
    }

    // Set stdio
    if options.inherit {
        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());
    } else {
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
    }

    // Execute
    let output = cmd.output().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to execute shell command: {}", e),
        span,
    })?;

    // Return typed ProcessOutput
    let process_out = ProcessOutput::from_std_output(&output);
    Ok(Value::Result(Ok(Box::new(Value::ProcessOutput(Arc::new(
        process_out,
    ))))))
}

// ============================================================================
// Environment Variables
// ============================================================================

/// Get an environment variable
///
/// Atlas signature: `getEnv(name: string) -> string | null`
pub fn get_env(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("getEnv", 1, args.len(), span));
    }

    let var_name = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string for variable name, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    // Check permission
    security.check_environment(&var_name).map_err(|_| {
        RuntimeError::EnvironmentPermissionDenied {
            var: var_name.clone(),
            span,
        }
    })?;

    // Get environment variable
    match env::var(&var_name) {
        Ok(value) => Ok(Value::Option(Some(Box::new(Value::string(value))))),
        Err(_) => Ok(Value::Option(None)),
    }
}

/// Set an environment variable
///
/// Atlas signature: `setEnv(name: string, value: string) -> null`
pub fn set_env(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("setEnv", 2, args.len(), span));
    }

    let var_name = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string for variable name, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    let var_value = match &args[1] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string for variable value, got {}",
                    args[1].type_name()
                ),
                span,
            })
        }
    };

    // Check permission
    security.check_environment(&var_name).map_err(|_| {
        RuntimeError::EnvironmentPermissionDenied {
            var: var_name.clone(),
            span,
        }
    })?;

    // Set environment variable
    env::set_var(&var_name, &var_value);

    Ok(Value::Null)
}

/// Remove an environment variable
///
/// Atlas signature: `unsetEnv(name: string) -> null`
pub fn unset_env(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("unsetEnv", 1, args.len(), span));
    }

    let var_name = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string for variable name, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    // Check permission
    security.check_environment(&var_name).map_err(|_| {
        RuntimeError::EnvironmentPermissionDenied {
            var: var_name.clone(),
            span,
        }
    })?;

    // Remove environment variable
    env::remove_var(&var_name);

    Ok(Value::Null)
}

/// List all environment variables
///
/// Atlas signature: `listEnv() -> object`
pub fn list_env(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("listEnv", 0, args.len(), span));
    }

    // Get all environment variables
    let env_vars: HashMap<String, crate::json_value::JsonValue> = env::vars()
        .map(|(key, value)| (key, crate::json_value::JsonValue::String(value)))
        .collect();

    Ok(Value::JsonValue(Arc::new(
        crate::json_value::JsonValue::Object(env_vars),
    )))
}

// ============================================================================
// Working Directory
// ============================================================================

/// Get current working directory
///
/// Atlas signature: `getCwd() -> string`
pub fn get_cwd(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("getCwd", 0, args.len(), span));
    }

    let cwd = env::current_dir().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to get current directory: {}", e),
        span,
    })?;

    Ok(Value::string(cwd.to_string_lossy()))
}

/// Get current process ID
///
/// Atlas signature: `getPid() -> number`
pub fn get_pid(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("getPid", 0, args.len(), span));
    }

    Ok(Value::Number(std::process::id() as f64))
}

/// Get command-line arguments passed to the Atlas program (H-213)
///
/// Atlas signature: `getProcessArgs() -> string[]`
pub fn get_process_args(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(stdlib_arity_error("getProcessArgs", 0, args.len(), span));
    }

    // Skip the first two args: the atlas binary and the source file path.
    // User programs see only the arguments after the file name.
    let argv: Vec<Value> = std::env::args().skip(2).map(Value::string).collect();
    Ok(Value::Array(ValueArray::from_vec(argv)))
}

/// Direct process execution — no shell, returns stdout as string (H-212)
///
/// Atlas signature: `processRun(program: string, args: string[]) -> Result<string, string>`
pub fn process_run(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("processRun", 2, args.len(), span));
    }

    let program = match &args[0] {
        Value::String(s) => s.as_ref().clone(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "processRun: first argument (program) must be a string".to_string(),
                span,
            })
        }
    };

    security
        .check_process(&program)
        .map_err(|_| RuntimeError::ProcessPermissionDenied {
            command: program.clone(),
            span,
        })?;

    let cmd_args: Vec<String> = match &args[1] {
        Value::Array(arr) => arr
            .iter()
            .map(|v| match v {
                Value::String(s) => Ok(s.as_ref().clone()),
                _ => Err(RuntimeError::TypeError {
                    msg: "processRun: args array must contain strings".to_string(),
                    span,
                }),
            })
            .collect::<Result<_, _>>()?,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "processRun: second argument (args) must be a string array".to_string(),
                span,
            })
        }
    };

    let output = Command::new(&program)
        .args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()
        .map_err(|e| RuntimeError::IoError {
            message: format!("processRun: failed to execute '{}': {}", program, e),
            span,
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string();
        Ok(Value::Result(Ok(Box::new(Value::string(stdout)))))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string();
        let msg = if stderr.is_empty() {
            format!(
                "process '{}' exited with code {}",
                program,
                output.status.code().unwrap_or(-1)
            )
        } else {
            stderr
        };
        Ok(Value::Result(Err(Box::new(Value::string(msg)))))
    }
}

// ============================================================================
// Async Process Management
// ============================================================================

/// Spawn a process in the background
///
/// Atlas signature: `spawnProcess(command: []string) -> ProcessHandle`
pub fn spawn_process(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("spawnProcess", 1, args.len(), span));
    }

    let (program, command_args) = parse_command_array(&args[0], span, "spawnProcess")?;

    security
        .check_process(&program)
        .map_err(|_| RuntimeError::ProcessPermissionDenied {
            command: program.clone(),
            span,
        })?;

    let mut cmd = Command::new(&program);
    cmd.args(&command_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| RuntimeError::IoError {
        message: format!("Failed to spawn process: {}", e),
        span,
    })?;

    let pid = child.id();
    process_registry().lock().unwrap().insert(pid, child);

    Ok(Value::Number(pid as f64))
}

/// Get a writer handle for a process stdin stream
///
/// Atlas signature: `processStdin(handle: ProcessHandle) -> Writer`
pub fn process_stdin(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("processStdin", 1, args.len(), span));
    }

    let pid = extract_process_handle(&args[0], "processStdin", span)?;
    let registry = process_registry();
    let mut registry = registry.lock().unwrap();
    let child = registry
        .get_mut(&pid)
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "processStdin(): unknown process handle".to_string(),
            span,
        })?;

    let mut io_registry = process_io_registry().lock().unwrap();
    let entry = io_registry.entry(pid).or_default();
    if let Some(id) = entry.stdin {
        return Ok(make_io_handle(PROCESS_STDIN_TAG, id));
    }

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "processStdin(): stdin not available".to_string(),
            span,
        })?;

    let id = NEXT_PROCESS_IO_ID.fetch_add(1, Ordering::Relaxed);
    process_stdin_handles()
        .lock()
        .unwrap()
        .insert(id, Arc::new(Mutex::new(stdin)));
    entry.stdin = Some(id);

    Ok(make_io_handle(PROCESS_STDIN_TAG, id))
}

/// Get a reader handle for a process stdout stream
///
/// Atlas signature: `processStdout(handle: ProcessHandle) -> Reader`
pub fn process_stdout(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("processStdout", 1, args.len(), span));
    }

    let pid = extract_process_handle(&args[0], "processStdout", span)?;
    let registry = process_registry();
    let mut registry = registry.lock().unwrap();
    let child = registry
        .get_mut(&pid)
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "processStdout(): unknown process handle".to_string(),
            span,
        })?;

    let mut io_registry = process_io_registry().lock().unwrap();
    let entry = io_registry.entry(pid).or_default();
    if let Some(id) = entry.stdout {
        return Ok(make_io_handle(PROCESS_STDOUT_TAG, id));
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "processStdout(): stdout not available".to_string(),
            span,
        })?;

    let id = NEXT_PROCESS_IO_ID.fetch_add(1, Ordering::Relaxed);
    process_stdout_handles()
        .lock()
        .unwrap()
        .insert(id, Arc::new(Mutex::new(stdout)));
    entry.stdout = Some(id);

    Ok(make_io_handle(PROCESS_STDOUT_TAG, id))
}

/// Get a reader handle for a process stderr stream
///
/// Atlas signature: `processStderr(handle: ProcessHandle) -> Reader`
pub fn process_stderr(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("processStderr", 1, args.len(), span));
    }

    let pid = extract_process_handle(&args[0], "processStderr", span)?;
    let registry = process_registry();
    let mut registry = registry.lock().unwrap();
    let child = registry
        .get_mut(&pid)
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "processStderr(): unknown process handle".to_string(),
            span,
        })?;

    let mut io_registry = process_io_registry().lock().unwrap();
    let entry = io_registry.entry(pid).or_default();
    if let Some(id) = entry.stderr {
        return Ok(make_io_handle(PROCESS_STDERR_TAG, id));
    }

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "processStderr(): stderr not available".to_string(),
            span,
        })?;

    let id = NEXT_PROCESS_IO_ID.fetch_add(1, Ordering::Relaxed);
    process_stderr_handles()
        .lock()
        .unwrap()
        .insert(id, Arc::new(Mutex::new(stderr)));
    entry.stderr = Some(id);

    Ok(make_io_handle(PROCESS_STDERR_TAG, id))
}

/// Poll a process handle for completion (non-blocking)
///
/// Atlas signature: `processWait(handle: ProcessHandle) -> Result<number, string>`
pub fn process_wait(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("processWait", 1, args.len(), span));
    }

    let pid = extract_process_handle(&args[0], "processWait", span)?;
    let registry = process_registry();
    let mut registry = registry.lock().unwrap();
    let child = match registry.get_mut(&pid) {
        Some(child) => child,
        None => {
            return Ok(Value::Result(Err(Box::new(Value::string(
                "Unknown process handle",
            )))))
        }
    };

    match child.try_wait() {
        Ok(Some(status)) => Ok(Value::Result(Ok(Box::new(Value::Number(
            exit_status_code(status) as f64,
        ))))),
        Ok(None) => Ok(Value::Result(Err(Box::new(Value::string("still running"))))),
        Err(e) => Ok(Value::Result(Err(Box::new(Value::string(format!(
            "Failed to poll process: {}",
            e
        )))))),
    }
}

/// Send a signal to a process
///
/// Atlas signature: `processKill(handle: ProcessHandle, signal: number) -> Result<null, string>`
pub fn process_kill(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(stdlib_arity_error("processKill", 2, args.len(), span));
    }

    let pid = extract_process_handle(&args[0], "processKill", span)?;
    let signal = match &args[1] {
        Value::Number(n) => *n as i32,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "processKill(): expected signal number, got {}",
                    args[1].type_name()
                ),
                span,
            })
        }
    };

    let registry = process_registry();
    let mut registry = registry.lock().unwrap();
    let child = match registry.get_mut(&pid) {
        Some(child) => child,
        None => {
            return Ok(Value::Result(Err(Box::new(Value::string(
                "Unknown process handle",
            )))))
        }
    };

    if signal != 9 && signal != 15 {
        return Ok(Value::Result(Err(Box::new(Value::string(
            "Unsupported signal (use 9 or 15)",
        )))));
    }

    let result = if cfg!(unix) {
        #[cfg(unix)]
        unsafe {
            if libc::kill(pid as i32, signal) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error().to_string())
            }
        }
        #[cfg(not(unix))]
        {
            let _ = child;
            Err("Signals are not supported on this platform".to_string())
        }
    } else {
        let _ = signal;
        child.kill().map_err(|e| e.to_string())
    };

    match result {
        Ok(()) => Ok(Value::Result(Ok(Box::new(Value::Null)))),
        Err(message) => Ok(Value::Result(Err(Box::new(Value::string(message))))),
    }
}

/// Check if a process is still running
///
/// Atlas signature: `processIsRunning(handle: ProcessHandle) -> bool`
pub fn process_is_running(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("processIsRunning", 1, args.len(), span));
    }

    let pid = extract_process_handle(&args[0], "processIsRunning", span)?;
    let registry = process_registry();
    let mut registry = registry.lock().unwrap();
    let child = registry
        .get_mut(&pid)
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: "processIsRunning(): unknown process handle".to_string(),
            span,
        })?;

    match child.try_wait() {
        Ok(Some(_)) => Ok(Value::Bool(false)),
        Ok(None) => Ok(Value::Bool(true)),
        Err(e) => Err(RuntimeError::IoError {
            message: format!("Failed to poll process: {}", e),
            span,
        }),
    }
}

/// Get stdout/stderr output from a finished process
///
/// Atlas signature: `processOutput(handle: ProcessHandle) -> Result<string, string>`
pub fn process_output(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(stdlib_arity_error("processOutput", 1, args.len(), span));
    }

    let pid = extract_process_handle(&args[0], "processOutput", span)?;
    let registry = process_registry();
    let mut registry = registry.lock().unwrap();
    let mut child = match registry.remove(&pid) {
        Some(child) => child,
        None => {
            return Ok(Value::Result(Err(Box::new(Value::string(
                "Unknown process handle",
            )))))
        }
    };

    let status = match child.try_wait() {
        Ok(Some(status)) => status,
        Ok(None) => {
            registry.insert(pid, child);
            return Ok(Value::Result(Err(Box::new(Value::string("still running")))));
        }
        Err(e) => {
            registry.insert(pid, child);
            return Ok(Value::Result(Err(Box::new(Value::string(format!(
                "Failed to poll process: {}",
                e
            ))))));
        }
    };

    let output = read_child_output(&mut child, status).map_err(|e| RuntimeError::IoError {
        message: format!("Failed to read process output: {}", e),
        span,
    })?;

    let mut combined = String::from_utf8_lossy(&output.stdout).to_string();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(Value::Result(Ok(Box::new(Value::string(combined)))))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse command from string or array
fn parse_command(value: &Value, span: Span) -> Result<(String, Vec<String>), RuntimeError> {
    match value {
        Value::String(s) => {
            // Single command, no arguments
            Ok((s.as_ref().clone(), vec![]))
        }
        Value::Array(arr) => {
            let arr_slice = arr.as_slice();
            if arr_slice.is_empty() {
                return Err(RuntimeError::TypeError {
                    msg: "Command array cannot be empty".to_string(),
                    span,
                });
            }

            // First element is the program
            let program = match &arr_slice[0] {
                Value::String(s) => s.as_ref().clone(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: "Command program must be a string".to_string(),
                        span,
                    })
                }
            };

            // Rest are arguments
            let mut args = Vec::new();
            for arg_val in &arr_slice[1..] {
                match arg_val {
                    Value::String(s) => args.push(s.as_ref().clone()),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: "Command arguments must be strings".to_string(),
                            span,
                        })
                    }
                }
            }

            Ok((program, args))
        }
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "Expected string or array for command, got {}",
                value.type_name()
            ),
            span,
        }),
    }
}

fn parse_command_array(
    value: &Value,
    span: Span,
    func_name: &str,
) -> Result<(String, Vec<String>), RuntimeError> {
    match value {
        Value::Array(arr) => {
            let arr_slice = arr.as_slice();
            if arr_slice.is_empty() {
                return Err(RuntimeError::TypeError {
                    msg: format!("{}(): command array cannot be empty", func_name),
                    span,
                });
            }

            let program = match &arr_slice[0] {
                Value::String(s) => s.as_ref().clone(),
                _ => {
                    return Err(RuntimeError::TypeError {
                        msg: format!("{}(): command program must be a string", func_name),
                        span,
                    })
                }
            };

            let mut args = Vec::new();
            for arg_val in &arr_slice[1..] {
                match arg_val {
                    Value::String(s) => args.push(s.as_ref().clone()),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: format!("{}(): command arguments must be strings", func_name),
                            span,
                        })
                    }
                }
            }

            Ok((program, args))
        }
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "{}(): expected array of strings, got {}",
                func_name,
                value.type_name()
            ),
            span,
        }),
    }
}

fn extract_process_handle(value: &Value, func_name: &str, span: Span) -> Result<u32, RuntimeError> {
    match value {
        Value::Number(n) if *n >= 0.0 => Ok(*n as u32),
        _ => Err(RuntimeError::TypeError {
            msg: format!("{}(): expected process handle number", func_name),
            span,
        }),
    }
}

fn exit_status_code(status: std::process::ExitStatus) -> i32 {
    status.code().unwrap_or(-1)
}

fn read_child_output(
    child: &mut Child,
    status: std::process::ExitStatus,
) -> std::io::Result<Output> {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    if let Some(mut out) = child.stdout.take() {
        out.read_to_end(&mut stdout)?;
    }
    if let Some(mut err) = child.stderr.take() {
        err.read_to_end(&mut stderr)?;
    }

    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

/// Options for exec command
#[derive(Default)]
struct ExecOptions {
    env: Option<HashMap<String, String>>,
    cwd: Option<String>,
    inherit: bool,
}

/// Parse execution options from object
fn parse_exec_options(value: &Value, span: Span) -> Result<ExecOptions, RuntimeError> {
    let json_obj = match value {
        Value::JsonValue(j) => j,
        Value::Null => return Ok(ExecOptions::default()),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected object for options, got {}", value.type_name()),
                span,
            })
        }
    };

    let mut options = ExecOptions::default();

    // Parse env
    if let crate::json_value::JsonValue::Object(obj_map) = json_obj.as_ref() {
        if let Some(crate::json_value::JsonValue::Object(env_obj)) = obj_map.get("env") {
            let mut env_map = HashMap::new();
            for (key, val) in env_obj {
                if let crate::json_value::JsonValue::String(s) = val {
                    env_map.insert(key.clone(), s.clone());
                } else {
                    return Err(RuntimeError::TypeError {
                        msg: "Environment variable values must be strings".to_string(),
                        span,
                    });
                }
            }
            options.env = Some(env_map);
        }

        // Parse cwd
        if let Some(cwd_val) = obj_map.get("cwd") {
            if let crate::json_value::JsonValue::String(s) = cwd_val {
                options.cwd = Some(s.clone());
            } else {
                return Err(RuntimeError::TypeError {
                    msg: "Working directory must be a string".to_string(),
                    span,
                });
            }
        }

        // Parse inherit
        if let Some(inherit_val) = obj_map.get("inherit") {
            if let crate::json_value::JsonValue::Bool(b) = inherit_val {
                options.inherit = *b;
            } else {
                return Err(RuntimeError::TypeError {
                    msg: "Inherit option must be a boolean".to_string(),
                    span,
                });
            }
        }
    }

    Ok(options)
}
