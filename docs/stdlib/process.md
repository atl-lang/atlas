# Process Functions

Process spawning, execution, and management.

## Process Spawning

### spawn

```atlas
fn spawn(command: string) : Result<Process, string>
```

Spawns a subprocess and returns process handle.

**Parameters:**
- `command` - Shell command to execute

**Returns:**
- `Ok(Process)` on success
- `Err(string)` if spawn fails

**Alias:** `spawnProcess`

### shell

```atlas
fn shell(command: string) : Result<string, string>
```

Executes shell command and returns output.

**Parameters:**
- `command` - Shell command to execute

**Returns:**
- `Ok(string)` - Command output
- `Err(string)` on failure

**Alias:** `exec`

## Process Properties

### processStdout

```atlas
fn processStdout(process: Process) : string
```

Gets stdout from completed process.

**Parameters:**
- `process` - Process handle

**Returns:** `string` - All stdout output

### processStderr

```atlas
fn processStderr(process: Process) : string
```

Gets stderr from completed process.

**Parameters:**
- `process` - Process handle

**Returns:** `string` - All stderr output

### processStdin

```atlas
fn processStdin(process: Process) : string
```

Gets stdin from process (if captured).

**Parameters:**
- `process` - Process handle

**Returns:** `string` - Stdin data

### processOutput

```atlas
fn processOutput(process: Process) : object
```

Gets all output from process as object.

**Parameters:**
- `process` - Process handle

**Returns:** `object` with fields:
- `stdout: string` - Standard output
- `stderr: string` - Standard error
- `status: number` - Exit code
- `success: bool` - True if exit code 0

## Process Lifecycle

### processWait

```atlas
fn processWait(process: Process) : Result<number, string>
```

Waits for process to complete and returns exit code.

**Parameters:**
- `process` - Process handle

**Returns:**
- `Ok(number)` - Exit code
- `Err(string)` on error

### processKill

```atlas
fn processKill(process: Process) : Result<Null, string>
```

Kills a running process.

**Parameters:**
- `process` - Process handle

**Returns:**
- `Ok(Null)` on success
- `Err(string)` if already terminated

### processIsRunning

```atlas
fn processIsRunning(process: Process) : bool
```

Checks if process is still running.

**Parameters:**
- `process` - Process handle

**Returns:** `bool`

## Environment

### getEnv

```atlas
fn getEnv(name: string) : Option<string>
```

Gets environment variable value.

**Parameters:**
- `name` - Variable name

**Returns:** `Option<string>` - Variable value or None if not set

### setEnv

```atlas
fn setEnv(name: string, value: string) : Result<Null, string>
```

Sets environment variable.

**Parameters:**
- `name` - Variable name
- `value` - Variable value

**Returns:**
- `Ok(Null)` on success
- `Err(string)` on failure

### unsetEnv

```atlas
fn unsetEnv(name: string) : Result<Null, string>
```

Unsets environment variable.

**Parameters:**
- `name` - Variable name

**Returns:**
- `Ok(Null)` on success
- `Err(string)` on error

### listEnv

```atlas
fn listEnv() : object
```

Gets all environment variables.

**Returns:** `object` - HashMap of all env vars

## System Information

### getPid

```atlas
fn getPid() : number
```

Gets process ID of current process.

**Returns:** `number` - PID

### getCwd

```atlas
fn getCwd() : Result<string, string>
```

Gets current working directory.

**Returns:**
- `Ok(string)` - Current directory path
- `Err(string)` on error

## Example Usage

```atlas
// Simple shell execution
let output = shell("ls -la")?;
print(output);

// Spawn process and wait
let proc = spawn("cargo build")?;
let code = processWait(proc)?;
if (code != 0) {
  print("Build failed");
}

// Get output
let proc = spawn("echo hello")?;
processWait(proc)?;
print(processStdout(proc)); // "hello\n"

// Environment
print(getEnv("HOME"));
setEnv("MY_VAR", "value")?;
```
