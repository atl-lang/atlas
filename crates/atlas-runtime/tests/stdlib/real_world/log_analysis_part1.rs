use super::super::*;
// ============================================================================

#[test]
fn test_log_parse_basic() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 10:00:00 INFO: Application started\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let first: string = lines[0];
        includes(first, "INFO")
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_filter_errors() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: Started\nERROR: Failed\nWARN: Warning\nERROR: Crashed\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let errors: string[] = filter(lines, isError);
        len(errors)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_extract_timestamps() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 ERROR: Test\n2024-01-02 INFO: OK\n").unwrap();

    let code = format!(
        r#"
        fn getTimestamp(line: string) -> string {{
            return substring(line, 0.0, 10.0);
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        getTimestamp(line1)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "2024-01-01");
}

#[test]
fn test_log_count_by_level() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: msg1\nERROR: msg2\nINFO: msg3\nWARN: msg4\nINFO: msg5\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isInfo(line: string) -> bool {{
            return includes(line, "INFO");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let infos: string[] = filter(dataLines, isInfo);
        len(infos)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_extract_error_messages() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "[2024-01-01] ERROR: Connection failed\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line: string = lines[0];
        let parts: string[] = split(line, "ERROR: ");
        var msg: string = "";
        if (len(parts) >= 2.0) {{
            msg = parts[1];
        }}
        msg
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "Connection failed");
}

#[test]
fn test_log_filter_by_date() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "2024-01-01 INFO: Old\n2024-01-15 ERROR: New\n2024-01-20 INFO: Newer\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isAfterJan10(line: string) -> bool {{
            let date: string = substring(line, 0.0, 10.0);
            return !startsWith(date, "2024-01-0");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let recent: string[] = filter(dataLines, isAfterJan10);
        len(recent)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_severity_ordering() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "DEBUG: d\nINFO: i\nWARN: w\nERROR: e\n").unwrap();

    let code = format!(
        r#"
        fn isHighSeverity(line: string) -> bool {{
            return includes(line, "ERROR") || includes(line, "WARN");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let high: string[] = filter(dataLines, isHighSeverity);
        len(high)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_multi_line_error() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "ERROR: Failed\nStack trace line 1\nStack trace line 2\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let first: string = lines[0];
        let second: string = lines[1];
        includes(first, "ERROR") && includes(second, "Stack")
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_empty_lines_filter() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "INFO: msg1\n\nERROR: msg2\n\nWARN: msg3\n").unwrap();

    let code = format!(
        r#"
        fn isNotEmpty(line: string) -> bool {{
            return len(line) > 0.0;
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let nonEmpty: string[] = filter(lines, isNotEmpty);
        len(nonEmpty)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_contains_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "User alice logged in\nUser bob failed\nUser alice logged out\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn mentionsAlice(line: string) -> bool {{
            return includes(line, "alice");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let aliceLogs: string[] = filter(dataLines, mentionsAlice);
        len(aliceLogs)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_case_insensitive_search() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Error: test\nerror: test2\nERROR: test3\n").unwrap();

    let code = format!(
        r#"
        fn hasError(line: string) -> bool {{
            let lower: string = toLowerCase(line);
            return includes(lower, "error");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let errors: string[] = filter(dataLines, hasError);
        len(errors)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_extract_user_actions() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "User:alice Action:login\nUser:bob Action:logout\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn extractUser(line: string) -> string {{
            let parts: string[] = split(line, " ");
            let userPart: string = parts[0];
            let userFields: string[] = split(userPart, ":");
            return userFields[1];
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        extractUser(line1)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "alice");
}

#[test]
fn test_log_count_occurrences() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "login\nlogout\nlogin\nlogin\nlogout\n").unwrap();

    let code = format!(
        r#"
        fn isLogin(line: string) -> bool {{
            return line == "login";
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let logins: string[] = filter(dataLines, isLogin);
        len(logins)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_trim_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "  ERROR: Test  \n  WARN: Alert  \n").unwrap();

    let code = format!(
        r#"
        fn cleanLine(line: string) -> string {{
            return trim(line);
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        let cleaned: string = cleanLine(line1);
        cleaned
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "ERROR: Test");
}

#[test]
fn test_log_starts_with_timestamp() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 INFO: msg\n2024-01-02 ERROR: err\n").unwrap();

    let code = format!(
        r#"
        fn hasTimestamp(line: string) -> bool {{
            return startsWith(line, "2024");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let timestamped: string[] = filter(dataLines, hasTimestamp);
        len(timestamped)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}
