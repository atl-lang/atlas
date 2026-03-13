# process namespace

Spawn external processes, capture output, manage environment variables, and query the current process.

All operations check the security context. Attempts to execute disallowed programs or read restricted environment variables throw permission errors.

---

## ProcessOutput

`process.exec()` and `process.shell()` return `Result<ProcessOutput, string>`. `ProcessOutput` is an opaque value with the following methods:

| Method         | Signature                          | Description                               |
|----------------|------------------------------------|-------------------------------------------|
| `.stdout()`    | `stdout(out: ProcessOutput): string` | Captured standard output                |
| `.stderr()`    | `stderr(out: ProcessOutput): string` | Captured standard error                 |
| `.exitCode()`  | `exitCode(out: ProcessOutput): number` | Exit code (−1 if process was killed)  |
| `.success()`   | `success(out: ProcessOutput): bool`  | `true` if exit code is 0               |

```atlas
let result = process.exec("ls");
match result {
    Ok(out) => {
        console.log(out.stdout());
        console.log("exit: " + out.exitCode().toString());
    }
    Err(e) => console.log("error: " + e),
}
```

---

## Command Execution

### process.exec

```atlas
process.exec(command: string | string[], options?: object): Result<ProcessOutput, string>
```

Execute a command and wait for it to complete. `command` may be a string (program name or path) or an array of strings `["program", "arg1", "arg2"]`.

Options object fields (all optional):

| Field     | Type     | Description                                  |
|-----------|----------|----------------------------------------------|
| `env`     | `object` | Additional environment variables             |
| `cwd`     | `string` | Working directory for the child process      |
| `inherit` | `bool`   | Inherit parent stdio (default: `false`)      |

Returns `Ok(ProcessOutput)` on success (any exit code), `Err(string)` if the process could not be started.

```atlas
// String command
let result = process.exec("git status");

// Array command (preferred — avoids shell injection)
let result = process.exec(["git", "log", "--oneline", "-10"]);

// With options
let result = process.exec(
    ["npm", "test"],
    { cwd: "/home/user/project", inherit: true }
);
```

### process.shell

```atlas
process.shell(command: string, options?: object): Result<ProcessOutput, string>
```

Execute a shell command string via `sh -c` (Unix) or `cmd /C` (Windows). Supports shell features like pipes, redirects, and glob expansion. Same options as `process.exec`.

```atlas
let result = process.shell("find . -name '*.atl' | wc -l");
match result {
    Ok(out) => console.log(out.stdout().trim()),
    Err(e) => console.log(e),
}
```

### process.shellOut

```atlas
process.shellOut(command: string): Result<string, string>
```

Simpler variant of `process.shell` for when only stdout is needed. Returns `Ok(stdout)` if exit code is 0, `Err(stderr or message)` otherwise.

```atlas
match process.shellOut("git rev-parse HEAD") {
    Ok(sha) => console.log("commit: " + sha.trim()),
    Err(e) => console.log("not a git repo: " + e),
}
```

---

## Environment Variables

### process.getEnv

```atlas
process.getEnv(name: string): Option<string>
```

Read an environment variable. Returns `Some(value)` if set, `None` if not present. Throws on permission denial.

```atlas
match process.getEnv("HOME") {
    Some(home) => console.log(home),
    None => console.log("HOME not set"),
}
```

### process.setEnv

```atlas
process.setEnv(name: string, value: string): null
```

Set an environment variable for the current process and any child processes spawned afterward. Throws on permission denial.

```atlas
process.setEnv("MY_APP_ENV", "production");
```

### process.unsetEnv

```atlas
process.unsetEnv(name: string): null
```

Remove an environment variable. Throws on permission denial.

```atlas
process.unsetEnv("DEBUG");
```

### process.listEnv

```atlas
process.listEnv(): object
```

Return all environment variables as a plain object (keys and values are strings).

```atlas
let env = process.listEnv();
```

---

## Working Directory

### process.getCwd

```atlas
process.getCwd(): string
```

Return the current working directory as an absolute path.

```atlas
let cwd = process.getCwd();
console.log(cwd);
```

---

## Process Identity

### process.getPid

```atlas
process.getPid(): number
```

Return the numeric PID of the current process.

```atlas
let pid = process.getPid();
console.log("PID: " + pid.toString());
```

### process.platform

```atlas
process.platform(): string
```

Return the operating system identifier: `"darwin"` (macOS), `"linux"`, `"windows"`, or another OS string.

```atlas
let os = process.platform();
if os == "windows" {
    console.log("running on Windows");
}
```

### process.arch

```atlas
process.arch(): string
```

Return the CPU architecture: `"x86_64"`, `"aarch64"`, `"arm"`, etc.

```atlas
let arch = process.arch();
console.log("architecture: " + arch);
```

### process.exit

```atlas
process.exit(code: number): never
```

Terminate the process immediately with the given exit code. `0` signals success; any other value signals failure. This function does not return.

```atlas
process.exit(0);   // success
process.exit(1);   // failure
```

---

## Patterns

### Run and check result

```atlas
fn runGitCommand(borrow args: string[]): Result<string, string> {
    let fullArgs = ["git"];
    // (append args...)
    let result = process.exec(fullArgs);
    match result {
        Ok(out) => {
            if out.success() {
                return Ok(out.stdout());
            }
            return Err(out.stderr());
        }
        Err(e) => return Err(e),
    }
}
```

### Environment-aware configuration

```atlas
fn isDevelopment(): bool {
    match process.getEnv("APP_ENV") {
        Some(env) => return env == "development",
        None => return true,
    }
}
```

### Cross-platform script runner

```atlas
fn runScript(borrow script: string): Result<ProcessOutput, string> {
    if process.platform() == "windows" {
        return process.shell("powershell -Command " + script);
    }
    return process.shell(script);
}
```

### Capture and parse output

```atlas
let result = process.shellOut("git log --oneline -5");
match result {
    Ok(output) => {
        let lines = output.split("\n");
        for line in lines {
            if line != "" {
                console.log(line);
            }
        }
    }
    Err(e) => {
        console.log("git error: " + e);
        process.exit(1);
    }
}
```
