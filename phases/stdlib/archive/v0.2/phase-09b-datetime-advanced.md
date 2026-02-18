# Phase 09b: DateTime Advanced Features

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase-09a must be complete.

**Verification:**
```bash
grep -r "dateTimeNow" crates/atlas-runtime/src/stdlib/
cargo test -p atlas-runtime datetime_core_tests
```

**What's needed:**
- Phase-09a complete (DateTime value type, core functions)
- chrono crate already added
- chrono-tz for timezone support

**If missing:** Complete phase-09a first.

---

## Objective
Implement advanced datetime features including custom formatting, multiple format parsing, timezone operations, and duration handling - enabling sophisticated time-based applications.

## Scope
**Previous phase (09a):** DateTime value type, construction, components, arithmetic
**This phase (09b):** Advanced formatting, parsing, timezones, duration

## Files
**Update:** `crates/atlas-runtime/src/stdlib/datetime.rs` (+500 lines)
**Update:** `crates/atlas-runtime/Cargo.toml` (add chrono-tz dependency)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register functions)
**Create:** `crates/atlas-runtime/tests/datetime_advanced_tests.rs` (~350 lines)

## Dependencies
- chrono-tz = "0.8" (timezone database)
- Phase-09a complete
- HashMap support for timezone info

## Implementation

### 1. Advanced Formatting
**Functions:**
- `dateTimeFormat(dt: DateTime, format: string)` - Custom format string
- `dateTimeToRfc3339(dt: DateTime)` - RFC 3339 format
- `dateTimeToRfc2822(dt: DateTime)` - RFC 2822 format (email)
- `dateTimeToCustom(dt: DateTime, format: string)` - Alias for format

**Format Specifiers (chrono strftime):**
- `%Y` - Year (2024)
- `%m` - Month (01-12)
- `%d` - Day (01-31)
- `%H` - Hour 24h (00-23)
- `%M` - Minute (00-59)
- `%S` - Second (00-59)
- `%A` - Weekday name (Monday)
- `%B` - Month name (January)
- `%z` - Timezone offset (+0000)
- `%Z` - Timezone name (UTC)

**Behavior:**
- Invalid format strings return runtime errors
- Unicode support in format strings
- Timezone-aware formatting

### 2. Advanced Parsing
**Functions:**
- `dateTimeParse(text: string, format: string)` - Parse with custom format
- `dateTimeParseRfc3339(text: string)` - Parse RFC 3339
- `dateTimeParseRfc2822(text: string)` - Parse RFC 2822
- `dateTimeTryParse(text: string, formats: Array)` - Try multiple formats

**Behavior:**
- Parse errors return runtime errors with details
- Support multiple format attempts (tryParse)
- Timezone-aware parsing (preserve offset)

### 3. Timezone Operations
**Functions:**
- `dateTimeToUtc(dt: DateTime)` - Convert to UTC
- `dateTimeToLocal(dt: DateTime)` - Convert to local timezone
- `dateTimeToTimezone(dt: DateTime, tz: string)` - Convert to named timezone
- `dateTimeGetTimezone(dt: DateTime)` - Get timezone name
- `dateTimeGetOffset(dt: DateTime)` - Get UTC offset in seconds
- `dateTimeInTimezone(dt: DateTime, tz: string)` - Create in specific timezone

**Timezone Names:**
- IANA timezone database (America/New_York, Europe/London, etc.)
- UTC, GMT
- Fixed offsets (+05:00, -08:00)

**Behavior:**
- Invalid timezone names return runtime errors
- Daylight saving time handled automatically
- Timezone info stored in DateTime

### 4. Duration Operations
**Functions:**
- `durationFromSeconds(seconds: number)` - Create duration
- `durationFromMinutes(minutes: number)` - Create duration
- `durationFromHours(hours: number)` - Create duration
- `durationFromDays(days: number)` - Create duration
- `durationFormat(duration: Duration)` - Format duration (e.g., "2h 30m")

**Behavior:**
- Duration is a HashMap with {days, hours, minutes, seconds}
- Can be negative
- Format as human-readable string

## Tests (40% of phase-09 total, ~30 tests)

### Formatting Tests (8)
1. Format with custom pattern
2. Format to RFC 3339
3. Format to RFC 2822
4. Format with weekday/month names
5. Format with timezone offset
6. Invalid format string error
7. Unicode in format strings
8. Edge cases (leap second, etc.)

### Parsing Tests (8)
1. Parse with custom format
2. Parse RFC 3339 string
3. Parse RFC 2822 string
4. Try multiple formats (first succeeds)
5. Try multiple formats (second succeeds)
6. Invalid format error
7. Malformed datetime error
8. Timezone in parsed string

### Timezone Tests (10)
1. Convert DateTime to UTC
2. Convert DateTime to local timezone
3. Convert to named timezone (America/New_York)
4. Convert to named timezone (Europe/London)
5. Get timezone name from DateTime
6. Get UTC offset from DateTime
7. Create DateTime in specific timezone
8. Invalid timezone name error
9. Daylight saving time handling
10. Timezone conversion roundtrip

### Duration Tests (4)
1. Create duration from seconds
2. Create duration from days
3. Format duration as string
4. Negative duration

**Minimum test count:** 30 tests (all must pass)

## Acceptance Criteria
- ✅ dateTimeFormat works with custom format strings
- ✅ RFC 3339 and RFC 2822 formatting work
- ✅ Advanced parsing with custom formats works
- ✅ dateTimeTryParse tries multiple formats
- ✅ Timezone conversion functions work correctly
- ✅ Named timezone support (IANA database)
- ✅ Timezone offset extraction works
- ✅ Duration creation and formatting work
- ✅ Invalid timezone names produce errors
- ✅ Invalid format strings produce errors
- ✅ 30+ tests pass (100% pass rate)
- ✅ Interpreter/VM parity maintained
- ✅ cargo test -p atlas-runtime passes (full suite)
- ✅ cargo clippy clean (zero warnings)
- ✅ cargo fmt clean

## Implementation Notes
- Use chrono-tz for timezone database
- Duration can be HashMap or custom struct (TBD based on pattern analysis)
- All functions are pure stdlib functions (not intrinsics)
- Errors use RuntimeError::new(ErrorCode::ATXXXX, msg)
- Follow existing stdlib patterns
- Format strings use chrono's strftime format

## Integration with Phase-09a
This phase builds on:
- DateTime value type from phase-09a
- Basic construction functions
- Component accessors
- Basic arithmetic

Together, phase-09a + phase-09b provide complete datetime API with 35+ functions.
