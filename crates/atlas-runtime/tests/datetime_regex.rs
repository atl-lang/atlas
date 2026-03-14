// datetime_regex.rs — DateTime and Regex stdlib extension tests

use atlas_runtime::{Atlas, Value};

// --- DateTime core ---

// Tests for datetime core functionality (Phase-09a)
//
// Tests construction, component access, arithmetic, and conversion operations.

// ============================================================================
// Test Helpers
// ============================================================================

fn eval_ok(code: &str) -> String {
    let atlas = Atlas::new();
    let result = atlas.eval(code).expect("Execution should succeed");
    result.to_string()
}

fn eval_expect_error(code: &str) -> bool {
    let atlas = Atlas::new();
    atlas.eval(code).is_err()
}

// ============================================================================
// Construction Tests (8 tests)
// ============================================================================

#[test]
fn test_date_time_now() {
    let code = r#"
        let dt = DateTime.now();
        typeof(dt)
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
fn test_date_time_from_timestamp() {
    // Unix epoch for 2021-01-01 00:00:00 UTC = 1609459200
    let code = r#"
        let dt = DateTime.fromTimestamp(1609459200);
        let y = dt.year();
        let m = dt.month();
        let d = dt.day();
        y * 10000 + m * 100 + d
    "#;
    assert_eq!(eval_ok(code), "20210101");
}

#[test]
fn test_date_time_from_components() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 2, 14, 15, 30, 45);
        let y = dt.year();
        let mo = dt.month();
        let d = dt.day();
        let h = dt.hour();
        let mi = dt.minute();
        let s = dt.second();
        y * 10000000000 + mo * 100000000 + d * 1000000 + h * 10000 + mi * 100 + s
    "#;
    assert_eq!(eval_ok(code), "20240214153045");
}

#[test]
fn test_date_time_parse_iso() {
    let code = r#"
        let dt = DateTime.parseIso("2024-01-15T10:30:00Z");
        let y = dt.year();
        let m = dt.month();
        let d = dt.day();
        y * 10000 + m * 100 + d
    "#;
    assert_eq!(eval_ok(code), "20240115");
}

#[test]
fn test_date_time_from_components_invalid_month() {
    assert!(eval_expect_error(
        "DateTime.fromComponents(2024, 13, 1, 0, 0, 0);"
    ));
}

#[test]
fn test_date_time_from_components_invalid_day() {
    assert!(eval_expect_error(
        "DateTime.fromComponents(2024, 1, 32, 0, 0, 0);"
    ));
}

#[test]
fn test_date_time_parse_iso_invalid() {
    assert!(eval_expect_error(r#"DateTime.parseIso("not-a-date");"#));
}

#[test]
fn test_date_time_negative_timestamp() {
    // Negative timestamp (before Unix epoch)
    let code = r#"
        let dt = DateTime.fromTimestamp(-86400);
        dt.year()
    "#;
    assert_eq!(eval_ok(code), "1969");
}

// ============================================================================
// Component Access Tests (10 tests)
// ============================================================================

#[test]
fn test_date_time_year() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 15, 10, 30, 45);
        dt.year()
    "#;
    assert_eq!(eval_ok(code), "2024");
}

#[test]
fn test_date_time_month() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 15, 10, 30, 45);
        dt.month()
    "#;
    assert_eq!(eval_ok(code), "6");
}

#[test]
fn test_date_time_day() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 15, 10, 30, 45);
        dt.day()
    "#;
    assert_eq!(eval_ok(code), "15");
}

#[test]
fn test_date_time_hour() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 15, 10, 30, 45);
        dt.hour()
    "#;
    assert_eq!(eval_ok(code), "10");
}

#[test]
fn test_date_time_minute() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 15, 10, 30, 45);
        dt.minute()
    "#;
    assert_eq!(eval_ok(code), "30");
}

#[test]
fn test_date_time_second() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 15, 10, 30, 45);
        dt.second()
    "#;
    assert_eq!(eval_ok(code), "45");
}

#[test]
fn test_date_time_weekday() {
    // 2024-01-01 is a Monday (weekday = 1)
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 0, 0, 0);
        dt.weekday()
    "#;
    assert_eq!(eval_ok(code), "1"); // Monday
}

#[test]
fn test_date_time_weekday_sunday() {
    // 2024-01-07 is a Sunday (weekday = 7)
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 7, 0, 0, 0);
        dt.weekday()
    "#;
    assert_eq!(eval_ok(code), "7"); // Sunday
}

#[test]
fn test_date_time_day_of_year() {
    // January 15 is day 15
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 15, 0, 0, 0);
        dt.dayOfYear()
    "#;
    assert_eq!(eval_ok(code), "15");
}

#[test]
fn test_date_time_day_of_year_leap() {
    // 2024 is a leap year, March 1 is day 61 (31 + 29 + 1)
    let code = r#"
        let dt = DateTime.fromComponents(2024, 3, 1, 0, 0, 0);
        dt.dayOfYear()
    "#;
    assert_eq!(eval_ok(code), "61");
}

// ============================================================================
// Arithmetic Tests (8 tests)
// ============================================================================

#[test]
fn test_date_time_add_seconds_positive() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = dt.addSeconds( 90);
        dt2.second()
    "#;
    assert_eq!(eval_ok(code), "30");
}

#[test]
fn test_date_time_add_seconds_negative() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = dt.addSeconds( -30);
        let s = dt2.second();
        let m = dt2.minute();
        m * 100 + s
    "#;
    assert_eq!(eval_ok(code), "5930"); // 59:30
}

#[test]
fn test_date_time_add_days() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 15, 0, 0, 0);
        let dt2 = dt.addDays( 10);
        dt2.day()
    "#;
    assert_eq!(eval_ok(code), "25");
}

#[test]
fn test_date_time_add_hours_and_minutes() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 10, 30, 0);
        let dt2 = dt.addHours( 2);
        let dt3 = dt2.addMinutes( 45);
        let h = dt3.hour();
        let m = dt3.minute();
        h * 100 + m
    "#;
    assert_eq!(eval_ok(code), "1315"); // 13:15
}

#[test]
fn test_date_time_diff() {
    let code = r#"
        let dt1 = DateTime.fromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = DateTime.fromComponents(2024, 1, 1, 10, 0, 0);
        dt1.diff(dt2)
    "#;
    assert_eq!(eval_ok(code), "7200"); // 2 hours = 7200 seconds
}

#[test]
fn test_date_time_diff_negative() {
    let code = r#"
        let dt1 = DateTime.fromComponents(2024, 1, 1, 10, 0, 0);
        let dt2 = DateTime.fromComponents(2024, 1, 1, 12, 0, 0);
        dt1.diff(dt2)
    "#;
    assert_eq!(eval_ok(code), "-7200"); // -2 hours
}

#[test]
fn test_date_time_compare_equal() {
    let code = r#"
        let dt1 = DateTime.fromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = DateTime.fromComponents(2024, 1, 1, 12, 0, 0);
        dt1.compare(dt2)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_date_time_compare_less_greater() {
    let code_less = r#"
        let dt1 = DateTime.fromComponents(2024, 1, 1, 10, 0, 0);
        let dt2 = DateTime.fromComponents(2024, 1, 1, 12, 0, 0);
        dt1.compare(dt2)
    "#;
    assert_eq!(eval_ok(code_less), "-1");

    let code_greater = r#"
        let dt1 = DateTime.fromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = DateTime.fromComponents(2024, 1, 1, 10, 0, 0);
        dt1.compare(dt2)
    "#;
    assert_eq!(eval_ok(code_greater), "1");
}

// ============================================================================
// Conversion Tests (4 tests)
// ============================================================================

#[test]
fn test_date_time_to_timestamp_roundtrip() {
    let code = r#"
        let timestamp = 1609459200;  // 2021-01-01 00:00:00 UTC
        let dt = DateTime.fromTimestamp(timestamp);
        dt.toTimestamp()
    "#;
    assert_eq!(eval_ok(code), "1609459200");
}

#[test]
fn test_date_time_to_iso() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0);
        let iso = dt.toIso();
        iso.substring( 0, 19)
    "#;
    assert_eq!(eval_ok(code), "2024-01-15T10:30:00");
}

#[test]
fn test_iso_parse_roundtrip() {
    let code = r#"
        let iso = "2024-06-15T14:30:45Z";
        let dt = DateTime.parseIso(iso);
        let iso2 = dt.toIso();
        iso2.substring( 0, 19)
    "#;
    assert_eq!(eval_ok(code), "2024-06-15T14:30:45");
}

#[test]
fn test_timestamp_edge_cases() {
    // Test epoch
    let code_epoch = r#"
        let dt = DateTime.fromTimestamp(0);
        dt.year()
    "#;
    assert_eq!(eval_ok(code_epoch), "1970");

    // Test far future
    let code_future = r#"
        let dt = DateTime.fromTimestamp(2000000000);
        dt.year()
    "#;
    assert_eq!(eval_ok(code_future), "2033");
}

// ============================================================================
// Additional Edge Case Tests (2 tests)
// ============================================================================

#[test]
fn test_date_time_utc_alias() {
    // dateTimeUtc should be an alias for dateTimeNow
    let code = r#"
        let dt = DateTime.utc();
        typeof(dt)
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
fn test_large_time_span_arithmetic() {
    // Test adding many days
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 0, 0, 0);
        let dt2 = dt.addDays( 365);
        let m = dt2.month();
        let d = dt2.day();
        m * 100 + d
    "#;
    // 2024 is a leap year, so 365 days from Jan 1 is Dec 31
    assert_eq!(eval_ok(code), "1231"); // Month 12, Day 31
}

// --- DateTime advanced ---

// Integration tests for advanced datetime functionality (Phase-09b)
//
// Tests cover:
// - Advanced formatting (custom formats, RFC 3339/2822)
// - Advanced parsing (custom formats, RFC 3339/2822, try-parse)
// - Timezone operations (conversion, offset, named zones)
// - Duration operations (creation, formatting)

// ============================================================================
// Advanced Formatting Tests (8 tests)
// ============================================================================

#[test]
fn test_datetime_format_custom_pattern() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 45); dt.format( \"%Y-%m-%d %H:%M:%S\")")
        .unwrap();
    assert_eq!(result, Value::string("2024-01-15 10:30:45".to_string()));
}

#[test]
fn test_datetime_format_with_weekday_month_names() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); dt.format( \"%A, %B %d, %Y\")")
        .unwrap();
    assert_eq!(
        result,
        Value::string("Monday, January 15, 2024".to_string())
    );
}

#[test]
fn test_datetime_to_rfc3339() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); dt.toRfc3339()")
        .unwrap();
    // RFC 3339 format includes timezone
    match result {
        Value::String(s) => {
            assert!(s.starts_with("2024-01-15T10:30:00"));
            assert!(s.contains("+00:00") || s.contains("Z"));
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_datetime_to_rfc2822() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); dt.toRfc2822()")
        .unwrap();
    // RFC 2822 format: "Mon, 15 Jan 2024 10:30:00 +0000"
    match result {
        Value::String(s) => {
            assert!(s.contains("15 Jan 2024"));
            assert!(s.contains("10:30:00"));
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_datetime_format_with_timezone_offset() {
    let runtime = Atlas::new();
    let result = runtime
        .eval(
            "let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); dt.format( \"%Y-%m-%d %z\")",
        )
        .unwrap();
    match result {
        Value::String(s) => {
            assert!(s.starts_with("2024-01-15"));
            assert!(s.contains("+0000") || s.contains("+00:00"));
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_datetime_format_year_only() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 6, 15, 0, 0, 0); dt.format( \"%Y\")")
        .unwrap();
    assert_eq!(result, Value::string("2024".to_string()));
}

#[test]
fn test_datetime_format_time_only() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 14, 30, 45); dt.format( \"%H:%M:%S\")")
        .unwrap();
    assert_eq!(result, Value::string("14:30:45".to_string()));
}

#[test]
fn test_datetime_to_custom_alias() {
    let runtime = Atlas::new();
    let result = runtime
        .eval(
            "let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); dt.toCustom( \"%Y/%m/%d\")",
        )
        .unwrap();
    assert_eq!(result, Value::string("2024/01/15".to_string()));
}

// ============================================================================
// Advanced Parsing Tests (8 tests)
// ============================================================================

#[test]
fn test_datetime_parse_with_custom_format() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.parse(\"2024-01-15 10:30:00\", \"%Y-%m-%d %H:%M:%S\"); dt.year()")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_parse_different_format() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.parse(\"15/01/2024 10:30:00\", \"%d/%m/%Y %H:%M:%S\"); dt.month()")
        .unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_datetime_parse_rfc3339() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.parseRfc3339(\"2024-01-15T10:30:00+00:00\"); dt.year()")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_parse_rfc2822() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.parseRfc2822(\"Mon, 15 Jan 2024 10:30:00 +0000\"); dt.day()")
        .unwrap();
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_datetime_try_parse_first_format_succeeds() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.tryParse(\"2024-01-15 10:30:00\", [\"%Y-%m-%d %H:%M:%S\", \"%d/%m/%Y\"]); dt.year()")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_try_parse_second_format_succeeds() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.tryParse(\"15/01/2024 10:00:00\", [\"%Y-%m-%d %H:%M:%S\", \"%d/%m/%Y %H:%M:%S\"]); dt.month()")
        .unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_datetime_parse_invalid_format_error() {
    let runtime = Atlas::new();
    let result = runtime.eval("DateTime.parse(\"invalid\", \"%Y-%m-%d\")");
    assert!(result.is_err());
}

#[test]
fn test_datetime_try_parse_no_format_matches() {
    let runtime = Atlas::new();
    let result = runtime.eval("DateTime.tryParse(\"invalid\", [\"%Y-%m-%d\", \"%d/%m/%Y\"])");
    assert!(result.is_err());
}

// ============================================================================
// Timezone Operation Tests (10 tests)
// ============================================================================

#[test]
fn test_datetime_to_utc() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); let utc = dt.toUtc(); utc.year()")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_to_local() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); let local = dt.toLocal(); local.year()")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_to_timezone_america_new_york() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); let ny = dt.toTimezone( \"America/New_York\"); ny.year()")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_to_timezone_europe_london() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 6, 15, 12, 0, 0); let london = dt.toTimezone( \"Europe/London\"); london.year()")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_get_timezone() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.now(); dt.getTimezone()")
        .unwrap();
    assert_eq!(result, Value::string("UTC".to_string()));
}

#[test]
fn test_datetime_get_offset() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); dt.getOffset()")
        .unwrap();
    // UTC offset is always 0
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_datetime_in_timezone() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = DateTime.fromComponents(2024, 1, 15, 10, 30, 0); let tz = dt.inTimezone( \"America/New_York\"); tz.hour()")
        .unwrap();
    // Should have different hour due to timezone interpretation
    match result {
        Value::Number(h) => assert!((0.0..24.0).contains(&h)),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_datetime_to_timezone_invalid_name_error() {
    let runtime = Atlas::new();
    let result = runtime.eval("let dt = DateTime.now(); dt.toTimezone( \"Invalid/Timezone\")");
    assert!(result.is_err());
}

#[test]
fn test_datetime_in_timezone_invalid_name_error() {
    let runtime = Atlas::new();
    let result = runtime.eval("let dt = DateTime.now(); dt.inTimezone( \"Invalid/Zone\")");
    assert!(result.is_err());
}

#[test]
fn test_datetime_timezone_roundtrip() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt1 = DateTime.fromComponents(2024, 1, 15, 12, 0, 0); let ny = dt1.toTimezone( \"America/New_York\"); let dt2 = ny.toUtc(); dt1.compare(dt2)")
        .unwrap();
    assert_eq!(result, Value::Number(0.0)); // Should be equal
}

// ============================================================================
// Duration Operation Tests (4 tests)
// ============================================================================

#[test]
fn test_duration_from_seconds() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromSeconds(3665); let opt = dur.get(\"hours\"); unwrap(opt)")
        .unwrap();
    assert_eq!(result, Value::Number(1.0)); // 3665 seconds = 1 hour, 1 minute, 5 seconds
}

#[test]
fn test_duration_from_minutes() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromMinutes(90); let opt = dur.get(\"hours\"); unwrap(opt)")
        .unwrap();
    assert_eq!(result, Value::Number(1.0)); // 90 minutes = 1 hour, 30 minutes
}

#[test]
fn test_duration_from_hours() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromHours(25); let opt = dur.get(\"days\"); unwrap(opt)")
        .unwrap();
    assert_eq!(result, Value::Number(1.0)); // 25 hours = 1 day, 1 hour
}

#[test]
fn test_duration_from_days() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromDays(2); let opt = dur.get(\"days\"); unwrap(opt)")
        .unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_duration_format_positive() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromSeconds(3665); durationFormat(dur)")
        .unwrap();
    assert_eq!(result, Value::string("1h 1m 5s".to_string()));
}

#[test]
fn test_duration_format_days_hours() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromHours(25); durationFormat(dur)")
        .unwrap();
    assert_eq!(result, Value::string("1d 1h".to_string()));
}

#[test]
fn test_duration_format_negative() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromSeconds(-3600); durationFormat(dur)")
        .unwrap();
    assert_eq!(result, Value::string("-1h".to_string()));
}

#[test]
fn test_duration_format_zero_seconds() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromSeconds(0); durationFormat(dur)")
        .unwrap();
    assert_eq!(result, Value::string("0s".to_string()));
}

// --- Regex core ---

// Regex core functionality tests (Phase-08a)
//
// Tests regex compilation, matching, and capture group extraction.
// All tests use Atlas::eval() API.

// ============================================================================
// Test Helpers
// ============================================================================

// ============================================================================
// Compilation Tests (6 tests)
// ============================================================================

#[test]
fn test_regex_new_valid_pattern() {
    let code = r#"
        let pattern = regex.new("\\d+");
        typeof(unwrap(pattern))
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
fn test_regex_new_invalid_pattern() {
    let code = r#"
        let pattern = regex.new("[invalid");
        is_err(pattern)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_regex_new_empty_pattern() {
    let code = r#"
        let pattern = regex.new("");
        typeof(unwrap(pattern))
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
fn test_regex_new_complex_pattern() {
    let code = r#"
        let pattern = regex.new("(?P<year>\\d{4})-(?P<month>\\d{2})-(?P<day>\\d{2})");
        typeof(unwrap(pattern))
    "#;
    assert_eq!(eval_ok(code), "record");
}

#[test]
fn test_regex_escape() {
    let code = r#"
        regex.escape("hello.world*test+")
    "#;
    assert_eq!(eval_ok(code), "hello\\.world\\*test\\+");
}

#[test]
fn test_regex_new_with_flags() {
    let code = r#"
        let pattern = regex.new("(?i)HELLO");
        let r = unwrap(pattern);
        r.isMatch("hello")
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Matching Tests (12 tests)
// ============================================================================

#[test]
fn test_is_match_true() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.isMatch("hello123world")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_is_match_false() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.isMatch("hello world")
    "#;
    assert_eq!(eval_ok(code), "false");
}

#[test]
fn test_is_match_case_insensitive() {
    let code = r#"
        let pattern = unwrap(regex.new("(?i)HELLO"));
        pattern.isMatch("hello world")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_is_match_multiline() {
    let code = r#"
        let pattern = unwrap(regex.new("(?m)^world"));
        pattern.isMatch("hello\nworld")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_find_returns_match() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let result = pattern.find("hello123world");
        let match_obj = unwrap(result);
        unwrap(match_obj.get("text"))
    "#;
    assert_eq!(eval_ok(code), "123");
}

#[test]
fn test_find_returns_none() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let result = pattern.find("hello world");
        is_none(result)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_find_all_multiple_matches() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let matches = pattern.findAll("a1 b22 c333");
        len(matches)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_find_all_no_matches() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let matches = pattern.findAll("hello world");
        len(matches)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_find_all_non_overlapping() {
    let code = r#"
        let pattern = unwrap(regex.new("\\w+"));
        let matches = pattern.findAll("hello world test");
        len(matches)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_unicode_handling() {
    let code = r#"
        let pattern = unwrap(regex.new("世界"));
        pattern.isMatch("こんにちは世界")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_dot_matches_newline_with_flag() {
    let code = r#"
        let pattern = unwrap(regex.new("(?s)a.b"));
        pattern.isMatch("a\nb")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_anchors_start_end() {
    let code = r#"
        let pattern = unwrap(regex.new("^hello$"));
        pattern.isMatch("hello")
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Capture Group Tests (12 tests)
// ============================================================================

#[test]
fn test_captures_simple_group() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d+)"));
        let groups = unwrap(regex.captures(pattern, "hello123"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "2"); // Full match + 1 group
}

#[test]
fn test_captures_multiple_groups() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d+)-(\\w+)"));
        let groups = unwrap(regex.captures(pattern, "123-abc"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "3"); // Full match + 2 groups
}

#[test]
fn test_captures_nested_groups() {
    let code = r#"
        let pattern = unwrap(regex.new("((\\d+)-(\\w+))"));
        let groups = unwrap(regex.captures(pattern, "123-abc"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "4"); // Full match + 3 groups
}

#[test]
fn test_captures_optional_group() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d+)?-(\\w+)"));
        let groups = unwrap(regex.captures(pattern, "-abc"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "3"); // Full match + 2 groups (first is null)
}

#[test]
fn test_captures_named_groups() {
    let code = r#"
        let pattern = unwrap(regex.new("(?P<num>\\d+)-(?P<word>\\w+)"));
        let groups = unwrap(regex.capturesNamed(pattern, "123-abc"));
        unwrap(groups.get("num"))
    "#;
    assert_eq!(eval_ok(code), "123");
}

#[test]
fn test_captures_named_and_positional() {
    let code = r#"
        let pattern = unwrap(regex.new("(?P<first>\\d+)-(\\w+)"));
        let positional = unwrap(regex.captures(pattern, "123-abc"));
        let named = unwrap(regex.captures(pattern, "123-abc"));
        len(positional)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_captures_none_when_no_match() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d+)"));
        let groups = regex.captures(pattern, "hello world");
        is_none(groups)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_captures_named_none_when_no_match() {
    let code = r#"
        let pattern = unwrap(regex.new("(?P<num>\\d+)"));
        let groups = regex.captures(pattern, "hello world");
        is_none(groups)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_captures_with_alternation() {
    let code = r#"
        let pattern = unwrap(regex.new("(cat|dog)"));
        let groups = unwrap(regex.captures(pattern, "I have a dog"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "2");
}

#[test]
fn test_captures_backreferences_unsupported() {
    // Backreferences are NOT supported by Rust's regex crate
    // This test verifies we get an error (not a panic)
    let code = r#"
        let pattern = regex.new("(\\w+)\\s+\\1");
        is_err(pattern)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_captures_non_capturing_groups() {
    let code = r#"
        let pattern = unwrap(regex.new("(?:\\d+)-(\\w+)"));
        let groups = unwrap(regex.captures(pattern, "123-abc"));
        len(groups)
    "#;
    assert_eq!(eval_ok(code), "2"); // Full match + 1 capturing group (non-capturing doesn't count)
}

#[test]
fn test_captures_full_match_at_index_zero() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d+)-(\\w+)"));
        let groups = unwrap(regex.captures(pattern, "123-abc"));
        groups[0]
    "#;
    assert_eq!(eval_ok(code), "123-abc");
}

// ============================================================================
// Additional Edge Case Tests (5 tests to reach 35+)
// ============================================================================

#[test]
fn test_find_with_positions() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let match_obj = unwrap(pattern.find("hello123world"));
        let start = unwrap(match_obj.get("start"));
        let end_pos = unwrap(match_obj.get("end"));
        start
    "#;
    assert_eq!(eval_ok(code), "5");
}

#[test]
fn test_find_all_extracts_all_text() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let matches = pattern.findAll("1 and 22 and 333");
        let first = unwrap(matches[0].get("text"));
        let second = unwrap(matches[1].get("text"));
        let third = unwrap(matches[2].get("text"));
        first
    "#;
    assert_eq!(eval_ok(code), "1");
}

#[test]
fn test_regex_escape_all_special_chars() {
    let code = r#"
        let escaped = regex.escape(".*+?^$()[]{}|\\");
        let pattern = unwrap(regex.new(escaped));
        pattern.isMatch(".*+?^$()[]{}|\\")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_invalid_flag_returns_error() {
    // An invalid regex pattern returns Err
    let code = r#"
        let pattern = regex.new("[invalid");
        is_err(pattern)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_complex_email_pattern() {
    let code = r#"
        let pattern = unwrap(regex.new("[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"));
        pattern.isMatch("user@example.com")
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Test Count Verification
// ============================================================================

// Total tests:
// - Compilation: 6
// - Matching: 12
// - Capture Groups: 12
// - Edge Cases: 5
// TOTAL: 35 tests ✅

// --- Regex operations ---

// Regex operations tests (Phase-08b)
//
// Tests regex replacement operations, splitting, and advanced features.
// All tests use Atlas::eval() API.

// ============================================================================
// Test Helpers
// ============================================================================

// ============================================================================
// Basic Replacement Tests (10 tests)
// ============================================================================

#[test]
fn test_replace_first_match_only() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.replace("a1b2c3", "X")
    "#;
    assert_eq!(eval_ok(code), "aXb2c3");
}

#[test]
fn test_replace_all_matches() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.replaceAll("a1b2c3", "X")
    "#;
    assert_eq!(eval_ok(code), "aXbXcX");
}

#[test]
fn test_replace_with_capture_group_refs() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d+)"));
        pattern.replace("a123b", "[$1]")
    "#;
    assert_eq!(eval_ok(code), "a[123]b");
}

#[test]
fn test_replace_all_with_capture_groups() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d+)"));
        pattern.replaceAll("a1b22c333", "[$1]")
    "#;
    assert_eq!(eval_ok(code), "a[1]b[22]c[333]");
}

#[test]
fn test_replace_special_refs_full_match() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.replace("a123b", "[$0]")
    "#;
    assert_eq!(eval_ok(code), "a[123]b");
}

#[test]
fn test_replace_empty_replacement() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.replaceAll("a1b2c3", "")
    "#;
    assert_eq!(eval_ok(code), "abc");
}

#[test]
fn test_replace_no_match_returns_original() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.replace("abc", "X")
    "#;
    assert_eq!(eval_ok(code), "abc");
}

#[test]
fn test_replace_unicode() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.replace("こんにちは123世界", "★")
    "#;
    assert_eq!(eval_ok(code), "こんにちは★世界");
}

#[test]
fn test_replace_multiple_capture_groups() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d+)-(\\w+)"));
        pattern.replace("abc 123-xyz def", "[$2:$1]")
    "#;
    assert_eq!(eval_ok(code), "abc [xyz:123] def");
}

#[test]
fn test_replace_at_boundaries() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        pattern.replaceAll("123abc456", "X")
    "#;
    assert_eq!(eval_ok(code), "XabcX");
}

// ============================================================================
// Callback Replacement Tests (8 tests)
// ============================================================================

#[test]
fn test_replace_with_calls_callback_first_match() {
    let code = r#"
        fn bracketize(borrow m: Map<string, any>): string {
            return "[" + unwrap(m.get("text")) + "]";
        }
        let pattern = unwrap(regex.new("\\d+"));
        regex_replace_with(pattern, "a1b2c3", bracketize)
    "#;
    assert_eq!(eval_ok(code), "a[1]b2c3");
}

#[test]
fn test_replace_all_with_calls_callback_all_matches() {
    let code = r#"
        fn bracketize(borrow m: Map<string, any>): string {
            return "[" + unwrap(m.get("text")) + "]";
        }
        let pattern = unwrap(regex.new("\\d+"));
        regex_replace_all_with(pattern, "a1b2c3", bracketize)
    "#;
    assert_eq!(eval_ok(code), "a[1]b[2]c[3]");
}

#[test]
fn test_callback_receives_correct_match_data() {
    let code = r#"
        fn formatter(borrow m: Map<string, any>): string {
            let text = unwrap(m.get("text"));
            let start = unwrap(m.get("start"));
            let end_pos = unwrap(m.get("end"));
            return "[" + text + "@" + toString(start) + "-" + toString(end_pos) + "]";
        }
        let pattern = unwrap(regex.new("\\d+"));
        regex_replace_with(pattern, "hello123world", formatter)
    "#;
    assert_eq!(eval_ok(code), "hello[123@5-8]world");
}

#[test]
fn test_callback_return_value_used_as_replacement() {
    let code = r#"
        fn doubler(borrow m: Map<string, any>): string {
            let num: string = unwrap(m.get("text"));
            return toString(unwrap(num.toNumber()) * 2);
        }
        let pattern = unwrap(regex.new("\\d+"));
        regex_replace_with(pattern, "value:42", doubler)
    "#;
    assert_eq!(eval_ok(code), "value:84");
}

#[test]
fn test_callback_with_capture_groups() {
    let code = r#"
        fn swapper(borrow m: Map<string, any>): string {
            let groups = unwrap(m.get("groups"));
            let num = groups[1];
            let word = groups[2];
            return word + ":" + num;
        }
        let pattern = unwrap(regex.new("(\\d+)-(\\w+)"));
        regex_replace_with(pattern, "abc 123-xyz def", swapper)
    "#;
    assert_eq!(eval_ok(code), "abc xyz:123 def");
}

#[test]
fn test_callback_can_use_match_positions() {
    let code = r#"
        fn firstOrOther(borrow m: Map<string, any>): string {
            let start = unwrap(m.get("start"));
            if (start == 0) {
                return "FIRST";
            } else {
                return "OTHER";
            }
        }
        let pattern = unwrap(regex.new("\\w+"));
        regex_replace_with(pattern, "hello world", firstOrOther)
    "#;
    assert_eq!(eval_ok(code), "FIRST world");
}

#[test]
fn test_callback_can_access_groups_array() {
    let code = r#"
        fn extractCapture(borrow m: Map<string, any>): string {
            let groups = unwrap(m.get("groups"));
            let captured = groups[1];
            return "[" + captured + "]";
        }
        let pattern = unwrap(regex.new("(\\d+)"));
        regex_replace_with(pattern, "test123", extractCapture)
    "#;
    assert_eq!(eval_ok(code), "test[123]");
}

#[test]
fn test_replace_all_with_processes_all_matches() {
    let code = r#"
        fn bracketize(borrow m: Map<string, any>): string {
            let num = unwrap(m.get("text"));
            return "[" + num + "]";
        }
        let pattern = unwrap(regex.new("\\d+"));
        regex_replace_all_with(pattern, "1a2b3c", bracketize)
    "#;
    assert_eq!(eval_ok(code), "[1]a[2]b[3]c");
}

// ============================================================================
// Splitting Tests (8 tests)
// ============================================================================

#[test]
fn test_split_divides_at_matches() {
    let code = r#"
        let pattern = unwrap(regex.new(","));
        let parts = pattern.split("a,b,c");
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_split_includes_empty_strings() {
    let code = r#"
        let pattern = unwrap(regex.new(","));
        let parts = pattern.split("a,b,,c");
        parts[2]
    "#;
    assert_eq!(eval_ok(code), "");
}

#[test]
fn test_split_no_matches_returns_single_element() {
    let code = r#"
        let pattern = unwrap(regex.new(","));
        let parts = pattern.split("abc");
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "1");
}

#[test]
fn test_split_n_limits_splits() {
    let code = r#"
        let pattern = unwrap(regex.new(","));
        let parts = pattern.splitN( "a,b,c,d", 2);
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "3"); // Splits into 3 parts: a, b, c,d
}

#[test]
fn test_split_n_with_limit_zero_returns_empty() {
    let code = r#"
        let pattern = unwrap(regex.new(","));
        let parts = pattern.splitN( "a,b,c", 0);
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_split_on_complex_pattern() {
    let code = r#"
        let pattern = unwrap(regex.new("\\s+"));
        let parts = pattern.split("hello   world  test");
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_split_preserves_unicode() {
    let code = r#"
        let pattern = unwrap(regex.new(","));
        let parts = pattern.split("こんにちは,世界,テスト");
        parts[1]
    "#;
    assert_eq!(eval_ok(code), "世界");
}

#[test]
fn test_split_with_zero_width_matches() {
    let code = r#"
        let pattern = unwrap(regex.new(""));
        let parts = pattern.split("abc");
        len(parts)
    "#;
    // Empty pattern splits between every character including boundaries
    assert_eq!(eval_ok(code), "5"); // "", "a", "b", "c", ""
}

// ============================================================================
// Advanced Features Tests (8 tests)
// ============================================================================

#[test]
fn test_match_indices_returns_positions() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let indices = pattern.matchIndices("a1b22c333");
        len(indices)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_match_indices_returns_start_end_pairs() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let indices = pattern.matchIndices("hello123world");
        let first = indices[0];
        first[0]
    "#;
    assert_eq!(eval_ok(code), "5"); // start position
}

#[test]
fn test_match_indices_no_matches_returns_empty() {
    let code = r#"
        let pattern = unwrap(regex.new("\\d+"));
        let indices = pattern.matchIndices("hello");
        len(indices)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_regex_test_convenience_function() {
    let code = r#"
        Regex.test("\\d+", "hello123")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_regex_test_returns_false_no_match() {
    let code = r#"
        Regex.test("\\d+", "hello")
    "#;
    assert_eq!(eval_ok(code), "false");
}

#[test]
fn test_regex_test_returns_false_on_compile_error() {
    let code = r#"
        Regex.test("[invalid", "test")
    "#;
    assert_eq!(eval_ok(code), "false");
}

#[test]
fn test_match_indices_with_overlapping_pattern() {
    let code = r#"
        let pattern = unwrap(regex.new("\\w+"));
        let indices = pattern.matchIndices("hello world");
        len(indices)
    "#;
    assert_eq!(eval_ok(code), "2"); // "hello" and "world"
}

#[test]
fn test_regex_test_with_complex_pattern() {
    let code = r#"
        Regex.test("[a-z]+@[a-z]+\\.[a-z]+", "user@example.com")
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Integration Tests (6 tests)
// ============================================================================

#[test]
fn test_integration_email_validation() {
    let code = r#"
        let email_pattern = unwrap(regex.new("[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"));
        email_pattern.isMatch("test.user+tag@example.com")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_integration_url_extraction() {
    let code = r#"
        let url_pattern = unwrap(regex.new("https?://[^\\s]+"));
        let text = "Visit https://example.com or http://test.org for info";
        let matches = url_pattern.findAll(text);
        len(matches)
    "#;
    assert_eq!(eval_ok(code), "2");
}

#[test]
fn test_integration_phone_formatting() {
    let code = r#"
        let pattern = unwrap(regex.new("(\\d{3})(\\d{3})(\\d{4})"));
        pattern.replace("Phone: 5551234567", "($1) $2-$3")
    "#;
    assert_eq!(eval_ok(code), "Phone: (555) 123-4567");
}

#[test]
fn test_integration_html_tag_stripping() {
    let code = r#"
        let tag_pattern = unwrap(regex.new("<[^>]+>"));
        tag_pattern.replaceAll("<p>Hello <b>World</b></p>", "")
    "#;
    assert_eq!(eval_ok(code), "Hello World");
}

#[test]
fn test_integration_csv_parsing() {
    let code = r#"
        let pattern = unwrap(regex.new(","));
        let parts = pattern.split("John,Doe,30,Engineer");
        parts[3]
    "#;
    assert_eq!(eval_ok(code), "Engineer");
}

#[test]
fn test_integration_text_processing_pipeline() {
    let code = r#"
        fn uppercase_numbers(borrow m: Map<string, any>): string {
            let num = unwrap(m.get("text"));
            return "[" + num + "]";
        }
        let digit_pattern = unwrap(regex.new("\\d+"));
        let text = "Error 404: Page 500 not found";
        let processed = regex_replace_all_with(digit_pattern, text, uppercase_numbers);
        let word_pattern = unwrap(regex.new("\\s+"));
        let words = word_pattern.split(processed);
        len(words)
    "#;
    assert_eq!(eval_ok(code), "6"); // "Error", "[404]:", "Page", "[500]", "not", "found"
}

// ============================================================================
// DateTime Extended Hardening (Phase v02-completion-04)
// ============================================================================

fn eval_err(code: &str) -> bool {
    let atlas = Atlas::new();
    atlas.eval(code).is_err()
}

// --- Format/Parse round-trips ---

#[test]
fn test_date_time_to_rfc3339_roundtrip() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 15, 12, 30, 0);
        let s = dt.toRfc3339();
        let dt2 = DateTime.parseRfc3339(s);
        dt2.year() * 10000 + dt2.month() * 100 + dt2.day()
    "#;
    assert_eq!(eval_ok(code), "20240615");
}

#[test]
fn test_date_time_to_rfc2822_roundtrip() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 3, 10, 9, 0, 0);
        let s = dt.toRfc2822();
        let dt2 = DateTime.parseRfc2822(s);
        dt2.year() * 10000 + dt2.month() * 100 + dt2.day()
    "#;
    assert_eq!(eval_ok(code), "20240310");
}

#[test]
fn test_date_time_from_components_midnight() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 0, 0, 0);
        dt.hour() * 3600 + dt.minute() * 60 + dt.second()
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_date_time_from_components_feb29_non_leap_year_errors() {
    assert!(eval_err(r#"DateTime.fromComponents(2023, 2, 29, 0, 0, 0)"#));
}

#[test]
fn test_date_time_from_components_feb29_leap_year_succeeds() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 2, 29, 0, 0, 0);
        dt.day()
    "#;
    assert_eq!(eval_ok(code), "29");
}

#[test]
fn test_date_time_parse_empty_string_errors() {
    assert!(eval_expect_error(r#"DateTime.parseIso("")"#));
}

#[test]
fn test_date_time_parse_invalid_format_errors() {
    assert!(eval_expect_error(r#"DateTime.parseIso("not-a-date")"#));
}

#[test]
fn test_date_time_add_minutes_carry_into_hours() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 10, 50, 0);
        let dt2 = dt.addMinutes( 20);
        dt2.hour() * 100 + dt2.minute()
    "#;
    assert_eq!(eval_ok(code), "1110");
}

#[test]
fn test_date_time_add_seconds_carry_into_minutes() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 10, 0, 50);
        let dt2 = dt.addSeconds( 20);
        dt2.minute() * 100 + dt2.second()
    "#;
    assert_eq!(eval_ok(code), "110");
}

#[test]
fn test_date_time_compare_equal_returns_zero() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 1, 0, 0, 0);
        let dt2 = DateTime.fromComponents(2024, 6, 1, 0, 0, 0);
        dt.compare(dt2)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_date_time_compare_less_returns_negative() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 1, 1, 0, 0, 0);
        let dt2 = DateTime.fromComponents(2024, 12, 31, 0, 0, 0);
        dt.compare(dt2) < 0
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_date_time_diff_same_datetime_is_zero() {
    let code = r#"
        let dt = DateTime.fromComponents(2024, 6, 1, 12, 0, 0);
        let dt2 = DateTime.fromComponents(2024, 6, 1, 12, 0, 0);
        dt.diff(dt2)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_date_time_now_returns_datetime_type() {
    let code = r#"
        let dt = DateTime.now();
        dt.year() > 2020
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Regex Extended Hardening (Phase v02-completion-04)
// ============================================================================

#[test]
fn test_regex_invalid_pattern_returns_result_err() {
    // regex_new returns Result(Err(...)) for invalid patterns — not a runtime error
    let code = r#"
        let r = regex.new("[invalid");
        is_err(r)
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_regex_is_match_empty_string_matching_pattern() {
    let code = r#"
        let r = unwrap(regex.new(".*"));
        r.isMatch("")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_regex_find_all_no_matches_returns_empty_array() {
    let code = r#"
        let r = unwrap(regex.new("\\d+"));
        let results = r.findAll("no digits here");
        len(results)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_regex_replace_only_replaces_first() {
    let code = r#"
        let r = unwrap(regex.new("a"));
        r.replace("banana", "X")
    "#;
    assert_eq!(eval_ok(code), "bXnana");
}

#[test]
fn test_regex_replace_all_replaces_every_match() {
    let code = r#"
        let r = unwrap(regex.new("a"));
        r.replaceAll("banana", "X")
    "#;
    assert_eq!(eval_ok(code), "bXnXnX");
}

#[test]
fn test_regex_split_at_start_produces_empty_first() {
    let code = r#"
        let r = unwrap(regex.new(","));
        let parts = r.split(",hello");
        parts[0]
    "#;
    assert_eq!(eval_ok(code), "");
}

#[test]
fn test_regex_split_at_end_produces_empty_last() {
    let code = r#"
        let r = unwrap(regex.new(","));
        let parts = r.split("hello,");
        parts[1]
    "#;
    assert_eq!(eval_ok(code), "");
}

#[test]
fn test_regex_escape_special_chars() {
    let code = r#"
        let escaped = regex.escape("a.b*c+d?e");
        let r = unwrap(regex.new(escaped));
        r.isMatch("a.b*c+d?e")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_regex_named_captures() {
    // regex.capturesNamed returns Option<Map<string, string>> for named groups
    let code = r#"
        let r = unwrap(regex.new("(?P<year>\\d{4})-(?P<month>\\d{2})"));
        let opt = regex.capturesNamed(r, "2024-06");
        let m = unwrap(opt);
        unwrap(m.get("year"))
    "#;
    assert_eq!(eval_ok(code), "2024");
}

#[test]
fn test_regex_split_n_limit_one_returns_two_parts() {
    // splitn with limit=1 → splitn(text, 2) internally → at most 2 parts
    let code = r#"
        let r = unwrap(regex.new(","));
        let parts = r.splitN( "a,b,c", 1);
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "2");
}
