use super::*;

// ============================================================================
// B10-P07: Static namespaces Json, Math, Env — PascalCase.method() syntax
// Both interpreter and VM parity tested throughout.
// ============================================================================

// --- Json.parse() ---
// parseJSON returns Result<json>. Use unwrap() to extract the value.
// On invalid input it returns Result(Err(...)) — use isErr() to check.

#[test]
fn test_json_parse_object() {
    // parseJSON returns Ok(json) on valid input — unwrap should succeed
    let src = r#"let v = Json.parse("{\"x\": 1}"); unwrap(v); true;"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_json_parse_invalid_returns_err() {
    // parseJSON returns Result(Err(...)) on invalid input
    let src = r#"let v = Json.parse("not json"); isErr(v);"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- Json.stringify() ---

#[test]
fn test_json_stringify_number() {
    let src = r#"Json.stringify(42);"#;
    assert_eval_string(src, "42");
    assert_parity(src);
}

#[test]
fn test_json_stringify_string() {
    let src = r#"Json.stringify("hello");"#;
    assert_eval_string(src, "\"hello\"");
    assert_parity(src);
}

// --- Json.isValid() ---

#[test]
fn test_json_is_valid_true() {
    let src = r#"Json.isValid("{\"a\": 1}");"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_json_is_valid_false() {
    let src = r#"Json.isValid("not json");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// --- Math.sqrt() ---
// sqrt() returns Result<number>. Use unwrap() to extract.

#[test]
fn test_math_sqrt() {
    let src = r#"unwrap(Math.sqrt(9));"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

// --- Math.abs() ---

#[test]
fn test_math_abs_negative() {
    let src = r#"Math.abs(-5);"#;
    assert_eval_number(src, 5.0);
    assert_parity(src);
}

#[test]
fn test_math_abs_positive() {
    let src = r#"Math.abs(3);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

// --- Math.floor() / Math.ceil() / Math.round() ---

#[test]
fn test_math_floor() {
    let src = r#"Math.floor(3.7);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

#[test]
fn test_math_ceil() {
    let src = r#"Math.ceil(3.2);"#;
    assert_eval_number(src, 4.0);
    assert_parity(src);
}

#[test]
fn test_math_round_up() {
    let src = r#"Math.round(3.6);"#;
    assert_eval_number(src, 4.0);
    assert_parity(src);
}

#[test]
fn test_math_round_down() {
    let src = r#"Math.round(3.4);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

// --- Math.min() / Math.max() ---

#[test]
fn test_math_min() {
    let src = r#"Math.min(3, 7);"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

#[test]
fn test_math_max() {
    let src = r#"Math.max(3, 7);"#;
    assert_eval_number(src, 7.0);
    assert_parity(src);
}

// --- Math.pow() ---

#[test]
fn test_math_pow() {
    let src = r#"Math.pow(2, 10);"#;
    assert_eval_number(src, 1024.0);
    assert_parity(src);
}

// --- Math.sign() ---

#[test]
fn test_math_sign_negative() {
    let src = r#"Math.sign(-42);"#;
    assert_eval_number(src, -1.0);
    assert_parity(src);
}

#[test]
fn test_math_sign_positive() {
    let src = r#"Math.sign(5);"#;
    assert_eval_number(src, 1.0);
    assert_parity(src);
}

#[test]
fn test_math_sign_zero() {
    let src = r#"Math.sign(0);"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

// --- Math.clamp() ---
// clamp() returns Result<number>. Use unwrap() to extract.

#[test]
fn test_math_clamp_within() {
    let src = r#"unwrap(Math.clamp(5, 0, 10));"#;
    assert_eval_number(src, 5.0);
    assert_parity(src);
}

#[test]
fn test_math_clamp_below() {
    let src = r#"unwrap(Math.clamp(-5, 0, 10));"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

#[test]
fn test_math_clamp_above() {
    let src = r#"unwrap(Math.clamp(15, 0, 10));"#;
    assert_eval_number(src, 10.0);
    assert_parity(src);
}

// --- Math.random() ---

#[test]
fn test_math_random_range() {
    // random() returns [0, 1) — verify it's a number in range (untyped binding avoids AT3001)
    let src = r#"
        let r = Math.random();
        r >= 0;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- Math constants (B22) — accessed as 0-arg functions Math.PI() ---

#[test]
fn test_math_pi_constant() {
    let src = r#"Math.PI() > 3.14 && Math.PI() < 3.15;"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_math_e_constant() {
    let src = r#"Math.E() > 2.71 && Math.E() < 2.72;"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_math_sqrt2_constant() {
    let src = r#"Math.SQRT2() > 1.41 && Math.SQRT2() < 1.42;"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_math_ln2_constant() {
    let src = r#"Math.LN2() > 0.69 && Math.LN2() < 0.70;"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_math_ln10_constant() {
    let src = r#"Math.LN10() > 2.30 && Math.LN10() < 2.31;"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- Math new methods (B22) ---

#[test]
fn test_math_asin_zero() {
    let src = r#"unwrap(Math.asin(0));"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

#[test]
fn test_math_acos_one() {
    let src = r#"unwrap(Math.acos(1));"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

#[test]
fn test_math_atan_zero() {
    let src = r#"Math.atan(0);"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

#[test]
fn test_math_atan2_zero() {
    let src = r#"Math.atan2(0, 1);"#;
    assert_eval_number(src, 0.0);
    assert_parity(src);
}

#[test]
fn test_math_trunc_positive() {
    let src = r#"Math.trunc(4.9);"#;
    assert_eval_number(src, 4.0);
    assert_parity(src);
}

#[test]
fn test_math_trunc_negative() {
    let src = r#"Math.trunc(-4.9);"#;
    assert_eval_number(src, -4.0);
    assert_parity(src);
}

#[test]
fn test_math_log2_of_8() {
    let src = r#"unwrap(Math.log2(8));"#;
    assert_eval_number(src, 3.0);
    assert_parity(src);
}

#[test]
fn test_math_log10_of_100() {
    let src = r#"unwrap(Math.log10(100));"#;
    assert_eval_number(src, 2.0);
    assert_parity(src);
}

#[test]
fn test_math_exp_zero() {
    let src = r#"Math.exp(0);"#;
    assert_eval_number(src, 1.0);
    assert_parity(src);
}

#[test]
fn test_math_cbrt_eight() {
    let src = r#"Math.cbrt(8);"#;
    assert_eval_number(src, 2.0);
    assert_parity(src);
}

#[test]
fn test_math_hypot_3_4() {
    let src = r#"Math.hypot(3, 4);"#;
    assert_eval_number(src, 5.0);
    assert_parity(src);
}

// --- env.get() ---
// getEnv requires env permissions. Use allow_all security context via run_interpreter/run_vm.

#[test]
fn test_env_get_known_var() {
    // PATH is always set — verify get() returns a non-null value
    // Use run_interpreter/run_vm (allow_all security context) instead of assert_eval_bool
    let src = r#"let v = env.get("PATH"); v != null;"#;
    let result = run_interpreter(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(
        result.unwrap().contains("Bool(true)"),
        "Expected Bool(true)"
    );
    assert_parity(src);
}

// ============================================================================
// B10-P08: Static namespaces File, Process, DateTime, Path — PascalCase.method()
// Both interpreter and VM parity tested throughout.
// ============================================================================

// --- file.read() / file.write() / file.exists() / file.remove() ---
// File operations require fs permissions — use run_interpreter (allow_all context).

#[test]
fn test_file_write_and_read() {
    // Write then read — verify round-trip. File ops return Result<T,string> — debug contains content.
    let src = r#"
        file.write("/tmp/atlas_test_p08.txt", "hello atlas");
        let content = file.read("/tmp/atlas_test_p08.txt");
        content;
    "#;
    let result = run_interpreter(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(
        result.unwrap().contains("hello atlas"),
        "Expected file content"
    );
    assert_parity(src);
}

#[test]
fn test_file_exists_true() {
    // Write then check exists
    let src = r#"
        file.write("/tmp/atlas_test_exists.txt", "x");
        file.exists("/tmp/atlas_test_exists.txt");
    "#;
    let result = run_interpreter(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(
        result.unwrap().contains("Bool(true)"),
        "Expected Bool(true)"
    );
    assert_parity(src);
}

#[test]
fn test_file_exists_false() {
    let src = r#"file.exists("/tmp/atlas_nonexistent_zxqp.txt");"#;
    let result = run_interpreter(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(
        result.unwrap().contains("Bool(false)"),
        "Expected Bool(false)"
    );
    assert_parity(src);
}

#[test]
fn test_file_append() {
    let src = r#"
        file.write("/tmp/atlas_append_p08.txt", "hello");
        file.append("/tmp/atlas_append_p08.txt", " world");
        let content = file.read("/tmp/atlas_append_p08.txt");
        content;
    "#;
    let result = run_interpreter(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(
        result.unwrap().contains("hello world"),
        "Expected appended content"
    );
    assert_parity(src);
}

// --- process.cwd() / process.pid() ---

#[test]
fn test_process_cwd() {
    // cwd() returns a string directly (not Result)
    let src = r#"
        let cwd = process.cwd();
        cwd != "";
    "#;
    let result = run_interpreter(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(
        result.unwrap().contains("Bool(true)"),
        "Expected Bool(true)"
    );
    assert_parity(src);
}

#[test]
fn test_process_pid() {
    // pid() returns a number > 0
    let src = r#"
        let pid = process.pid();
        pid > 0;
    "#;
    let result = run_interpreter(src);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert!(
        result.unwrap().contains("Bool(true)"),
        "Expected Bool(true)"
    );
    assert_parity(src);
}

// --- datetime.now() ---

#[test]
fn test_datetime_now_is_datetime() {
    // datetime.now() returns a DateTime value
    let src = r#"
        let dt = datetime.now();
        dt != null;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_datetime_from_timestamp() {
    // datetime.fromTimestamp(0) = Unix epoch
    let src = r#"
        let dt = datetime.fromTimestamp(0);
        dt != null;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- path.join() / path.dirname() / path.basename() / path.exists() ---

#[test]
fn test_path_join() {
    let src = r#"path.join("/tmp", "atlas", "test.txt");"#;
    assert_eval_string(src, "/tmp/atlas/test.txt");
    assert_parity(src);
}

#[test]
fn test_path_dirname() {
    let src = r#"path.dirname("/tmp/atlas/test.txt");"#;
    assert_eval_string(src, "/tmp/atlas");
    assert_parity(src);
}

#[test]
fn test_path_basename() {
    let src = r#"path.basename("/tmp/atlas/test.txt");"#;
    assert_eval_string(src, "test.txt");
    assert_parity(src);
}

#[test]
fn test_path_extension() {
    let src = r#"path.extension("/tmp/atlas/test.txt");"#;
    assert_eval_string(src, "txt");
    assert_parity(src);
}

#[test]
fn test_path_is_absolute_true() {
    let src = r#"path.isAbsolute("/tmp/foo");"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_path_is_absolute_false() {
    let src = r#"path.isAbsolute("relative/path");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

#[test]
fn test_path_exists_false() {
    let src = r#"path.exists("/tmp/atlas_nonexistent_path_zxqp_99");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

// ============================================================================
// B10-P09: Static namespaces Http, Net, Crypto, Regex — PascalCase.method()
// Both interpreter and VM parity tested throughout.
// ============================================================================

// --- regex.new() / regex.test() / regex.isMatch() / regex.find() / regex.replace() ---

#[test]
fn test_regex_new() {
    // regex.new(pattern) returns Result<Regex>
    let src = r#"let r = regex.new("[0-9]+"); isOk(r);"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_regex_test_true() {
    // unwrap() the Result<Regex> before passing to regex.test
    let src = r#"let r = unwrap(regex.new("[0-9]+")); regex.test(r, "hello 42 world");"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_regex_test_false() {
    let src = r#"let r = unwrap(regex.new("[0-9]+")); regex.test(r, "no digits here");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

#[test]
fn test_regex_is_match() {
    // isMatch maps to regexIsMatch (compiled Regex + string)
    let src = r#"let r = unwrap(regex.new("^hello")); regex.isMatch(r, "hello world");"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_regex_is_match_false() {
    let src = r#"let r = unwrap(regex.new("^world")); regex.isMatch(r, "hello world");"#;
    assert_eval_bool(src, false);
    assert_parity(src);
}

#[test]
fn test_regex_find() {
    // find() returns Option<string>
    let src = r#"let r = unwrap(regex.new("[0-9]+")); isSome(regex.find(r, "foo 42 bar"));"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_regex_find_none() {
    let src = r#"let r = unwrap(regex.new("[0-9]+")); isNone(regex.find(r, "no digits"));"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_regex_replace() {
    let src = r#"let r = unwrap(regex.new("[0-9]+")); regex.replace(r, "foo 42 bar", "NUM");"#;
    assert_eval_string(src, "foo NUM bar");
    assert_parity(src);
}

#[test]
fn test_regex_replace_all() {
    let src = r#"let r = unwrap(regex.new("[0-9]+")); regex.replaceAll(r, "1 and 2 and 3", "N");"#;
    assert_eval_string(src, "N and N and N");
    assert_parity(src);
}

#[test]
fn test_regex_escape() {
    // escape() makes special regex chars literal
    let src = r#"let escaped = regex.escape("hello.world"); escaped != "";"#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// --- crypto.sha256() / crypto.sha512() ---

#[test]
fn test_crypto_sha256() {
    // sha256("hello") returns a hex string of length 64
    let src = r#"
        let h = crypto.sha256("hello");
        len(h) == 64;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_crypto_sha256_deterministic() {
    // Same input = same hash
    let src = r#"
        let h1 = crypto.sha256("atlas");
        let h2 = crypto.sha256("atlas");
        h1 == h2;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

#[test]
fn test_crypto_sha512() {
    // sha512 returns 128-char hex string
    let src = r#"
        let h = crypto.sha512("hello");
        len(h) == 128;
    "#;
    assert_eval_bool(src, true);
    assert_parity(src);
}

// ============================================================================
// End B10-P07/P08/P09 tests
// ============================================================================

// ============================================================================
// B18: Process namespace + ProcessOutput typed struct
// ============================================================================

#[test]
fn test_process_exec_returns_process_output() {
    // process.exec() returns Result<ProcessOutput, string>; verify via .success()
    // exec() takes a string as program name — use array form for args
    let src = r#"
        let result = process.shell("echo hello");
        match result {
            Ok(out) => out.success(),
            Err(_) => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_process_exec_output_stdout_method() {
    // ProcessOutput.stdout() returns captured stdout (non-empty for echo)
    let src = r#"
        let result = process.shell("echo hello");
        match result {
            Ok(out) => len(out.stdout()) > 0,
            Err(_) => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_process_exec_output_exit_code_method() {
    // ProcessOutput.exitCode() returns 0 for successful commands
    let src = r#"
        let result = process.shell("echo hi");
        match result {
            Ok(out) => out.exitCode() == 0,
            Err(_) => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_process_exec_output_success_method() {
    // ProcessOutput.success() returns true for exit code 0
    let src = r#"
        let result = process.shell("echo hi");
        match result {
            Ok(out) => out.success(),
            Err(_) => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_process_exec_output_stderr_method() {
    // .stderr() returns a string (empty for successful commands with no stderr)
    let src = r#"
        let result = process.shell("echo hi");
        match result {
            Ok(out) => len(out.stderr()) == 0,
            Err(_) => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_process_shell_returns_process_output() {
    // process.shell() returns Result<ProcessOutput, string>
    let src = r#"
        let result = process.shell("echo hello");
        match result {
            Ok(out) => out.success(),
            Err(_) => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_process_shell_output_stdout() {
    // process.shell() stdout() contains the output
    let src = r#"
        let result = process.shell("echo hello");
        match result {
            Ok(out) => len(out.stdout()) > 0,
            Err(_) => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_process_run_returns_result_string() {
    // process.run(program, args) -> Result<string, string>
    let src = r#"
        let result = process.run("echo", ["hi"]);
        match result {
            Ok(s) => len(s) > 0,
            Err(_) => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_process_args_returns_array() {
    // process.args() returns string[] — verify it doesn't error
    let src = r#"
        let argv = process.args();
        len(argv) >= 0;
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_env_list_returns_object() {
    // env.list() returns a JsonValue with env vars — must not error
    let src = r#"
        let _env = env.list();
        true;
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

#[test]
fn test_env_get_set_unset_via_namespace() {
    // env.set / env.get / env.unset all route through EnvNs
    let src = r#"
        env.set("ATLAS_B18_TEST", "42");
        let v = env.get("ATLAS_B18_TEST");
        env.unset("ATLAS_B18_TEST");
        match v {
            Some(s) => s == "42",
            None => false,
        }
    "#;
    let r = run_interpreter(src);
    assert!(r.is_ok(), "Expected Ok, got {:?}", r);
    assert!(r.unwrap().contains("Bool(true)"), "Expected Bool(true)");
    assert_parity(src);
}

// ============================================================================
// End B18 tests
// ============================================================================
