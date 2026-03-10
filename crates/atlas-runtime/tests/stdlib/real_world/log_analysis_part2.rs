use super::super::*;
#[test]
fn test_log_extract_ip_addresses() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("access.log");
    std::fs::write(&log_path, "192.168.1.1 GET /page\n10.0.0.1 POST /api\n").unwrap();

    let code = format!(
        r#"
        fn extractIP(borrow line: string): string {{
            let parts: []string = split(line, " ");
            return parts[0];
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let line1: string = lines[0];
        extractIP(line1)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "192.168.1.1");
}

#[test]
fn test_log_group_by_category() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "DB: query\nAPI: request\nDB: update\nDB: delete\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isDatabase(borrow line: string): bool {{
            return starts_with(line, "DB:");
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        let dbLogs: []string = filter(dataLines, isDatabase);
        len(dbLogs)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_parse_structured() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "level=error msg=\"Failed to connect\" code=500\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let line: string = lines[0];
        let parts: []string = split(line, " ");
        let levelPart: string = parts[0];
        starts_with(levelPart, "level=error")
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_count_warnings() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "INFO\nWARN\nERROR\nWARN\nINFO\nWARN\n").unwrap();

    let code = format!(
        r#"
        fn countWarnings(borrow total: number, borrow line: string): number {{
            if (line == "WARN") {{
                return total + 1.0;
            }}
            return total;
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        reduce(dataLines, countWarnings, 0.0)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_find_first_error() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: ok\nWARN: warning\nERROR: failure\nERROR: another\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(borrow line: string): bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        let firstError: string = unwrap(find(dataLines, isError));
        firstError
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "ERROR: failure");
}

#[test]
fn test_log_reverse_chronological() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Line1\nLine2\nLine3\n").unwrap();

    let code = format!(
        r#"
        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        let reversed: []string = reverse(dataLines);
        reversed[0]
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "Line3");
}

#[test]
fn test_log_summary_report() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "ERROR:e1\nINFO:i1\nERROR:e2\nWARN:w1\nERROR:e3\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(borrow line: string): bool {{ return includes(line, "ERROR"); }}
        fn isWarn(borrow line: string): bool {{ return includes(line, "WARN"); }}
        fn isInfo(borrow line: string): bool {{ return includes(line, "INFO"); }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);

        let errors: number = len(filter(dataLines, isError));
        let warns: number = len(filter(dataLines, isWarn));
        let infos: number = len(filter(dataLines, isInfo));

        "E:" + str(errors) + ",W:" + str(warns) + ",I:" + str(infos)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "E:3,W:1,I:1");
}

#[test]
fn test_log_filter_time_range() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "08:00 Start\n09:30 Middle\n12:00 End\n").unwrap();

    let code = format!(
        r#"
        fn isMorning(borrow line: string): bool {{
            let time: string = substring(line, 0.0, 2.0);
            return time == "08" || time == "09";
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        let morning: []string = filter(dataLines, isMorning);
        len(morning)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_extract_http_codes() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("access.log");
    std::fs::write(&log_path, "GET /page 200\nPOST /api 404\nGET /home 200\n").unwrap();

    let code = format!(
        r#"
        fn is404(borrow line: string): bool {{
            return includes(line, "404");
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        let notFound: []string = filter(dataLines, is404);
        len(notFound)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 1.0);
}

#[test]
fn test_log_parse_json_lines() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("json.log");
    std::fs::write(
        &log_path,
        "{\"level\":\"error\",\"msg\":\"failed\"}\n{\"level\":\"info\",\"msg\":\"ok\"}\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let line1: string = lines[0];
        let json: json = parse_json(line1)?;
        let level: string = json["level"].as_string();
        level
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "error");
}

#[test]
fn test_log_aggregate_metrics() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("metrics.log");
    std::fs::write(&log_path, "latency:100\nlatency:150\nlatency:200\n").unwrap();

    let code = format!(
        r#"
        fn sumLatency(borrow total: number, borrow line: string): number {{
            let parts: []string = split(line, ":");
            let value: number = unwrap(parse_float(parts[1]));
            return total + value;
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        let total: number = reduce(dataLines, sumLatency, 0.0);
        let avg: number = total / len(dataLines);
        avg
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 150.0);
}

#[test]
fn test_log_detect_anomalies() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Normal\nNormal\nANOMALY\nNormal\n").unwrap();

    let code = format!(
        r#"
        fn isAnomaly(borrow line: string): bool {{
            return line == "ANOMALY";
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        let anomalies: []string = filter(dataLines, isAnomaly);
        len(anomalies) > 0.0
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_combine_multiline() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "ERROR: Start\nContinue\nEnd\n").unwrap();

    let code = format!(
        r#"
        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let combined: string = lines[0] + " " + lines[1] + " " + lines[2];
        includes(combined, "Start") && includes(combined, "Continue") && includes(combined, "End")
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_write_filtered() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.log");
    let output_path = temp_dir.path().join("errors.log");
    std::fs::write(
        &input_path,
        "INFO: ok\nERROR: failed\nWARN: warn\nERROR: bad\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(borrow line: string): bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = read_file("{}");
        let lines: []string = split(logs, "\n");
        let dataLines: []string = slice(lines, 0.0, len(lines) - 1.0);
        let errors: []string = filter(dataLines, isError);
        let output: string = join(errors, "\n") + "\n";
        write_file("{}", output);

        let result: string = read_file("{}");
        let resultLines: []string = split(result, "\n");
        len(resultLines) - 1.0
    "#,
        path_for_atlas(&input_path),
        path_for_atlas(&output_path),
        path_for_atlas(&output_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}
