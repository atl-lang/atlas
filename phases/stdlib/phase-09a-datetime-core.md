# Phase 09a: DateTime Core & Arithmetic

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Stdlib infrastructure must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/stdlib/mod.rs
cargo test stdlib
```

**What's needed:**
- Stdlib infrastructure from v0.1
- chrono crate for time handling
- String formatting support

**If missing:** Basic stdlib should exist from v0.1

---

## Objective
Implement core datetime functionality including value type, construction, component access, and basic arithmetic operations - providing foundation for time-based computations.

## Scope
**This phase (09a):** DateTime value type, construction, components, arithmetic
**Next phase (09b):** Advanced formatting, parsing, timezone operations

## Files
**Create:** `crates/atlas-runtime/src/stdlib/datetime.rs` (~400 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~100 lines DateTime value)
**Update:** `crates/atlas-runtime/Cargo.toml` (add chrono dependency)
**Update:** `crates/atlas-runtime/src/stdlib/mod.rs` (register functions)
**Tests:** `crates/atlas-runtime/tests/datetime_core_tests.rs` (~350 lines)

## Dependencies
- chrono = "0.4" (datetime handling)
- Stdlib infrastructure
- Value enum support

## Implementation

### 1. DateTime Value Type
Add DateTime variant to Value enum. Wrap chrono::DateTime<Utc> in Rc<RefCell<>> for consistency. Display implementation for DateTime values. Clone and comparison support.

### 2. DateTime Construction
**Functions:**
- `dateTimeNow()` - Current UTC time
- `dateTimeFromTimestamp(seconds: number)` - Unix timestamp to DateTime
- `dateTimeFromComponents(year, month, day, hour, min, sec)` - Build from parts
- `dateTimeParseIso(text: string)` - Parse ISO 8601 string (RFC 3339)
- `dateTimeUtc()` - Current UTC time (alias for now)

**Behavior:**
- Validate components (month 1-12, day 1-31, hour 0-23, etc.)
- Handle invalid dates with runtime errors
- Support Unix timestamps (seconds since epoch)
- ISO 8601 parsing for basic format (YYYY-MM-DDTHH:MM:SSZ)

### 3. Component Access
**Functions:**
- `dateTimeYear(dt: DateTime)` - Extract year
- `dateTimeMonth(dt: DateTime)` - Extract month (1-12)
- `dateTimeDay(dt: DateTime)` - Extract day (1-31)
- `dateTimeHour(dt: DateTime)` - Extract hour (0-23)
- `dateTimeMinute(dt: DateTime)` - Extract minute (0-59)
- `dateTimeSecond(dt: DateTime)` - Extract second (0-59)
- `dateTimeWeekday(dt: DateTime)` - Day of week (1=Monday, 7=Sunday)
- `dateTimeDayOfYear(dt: DateTime)` - Day of year (1-366)

**Behavior:**
- All accessors return number values
- Immutable operations (don't modify DateTime)
- Follow ISO 8601 conventions (Monday=1)

### 4. Time Arithmetic
**Functions:**
- `dateTimeAddSeconds(dt: DateTime, seconds: number)` - Add seconds
- `dateTimeAddMinutes(dt: DateTime, minutes: number)` - Add minutes
- `dateTimeAddHours(dt: DateTime, hours: number)` - Add hours
- `dateTimeAddDays(dt: DateTime, days: number)` - Add days
- `dateTimeDiff(dt1: DateTime, dt2: DateTime)` - Difference in seconds
- `dateTimeCompare(dt1: DateTime, dt2: DateTime)` - Compare (-1, 0, 1)

**Behavior:**
- Arithmetic returns new DateTime (immutable)
- Diff returns signed number (dt1 - dt2 in seconds)
- Compare returns -1 (dt1 < dt2), 0 (equal), 1 (dt1 > dt2)
- Negative values subtract time

### 5. Conversion
**Functions:**
- `dateTimeToTimestamp(dt: DateTime)` - Convert to Unix timestamp
- `dateTimeToIso(dt: DateTime)` - Convert to ISO 8601 string

**Behavior:**
- Timestamp is seconds since Unix epoch
- ISO format: "YYYY-MM-DDTHH:MM:SSZ" (UTC)

## Tests (60% of phase-09 total, ~30 tests)

### Construction Tests (8)
1. Create current time with dateTimeNow
2. Create from valid timestamp
3. Create from valid components
4. Parse valid ISO 8601 string
5. Invalid components error (month 13)
6. Invalid components error (day 32)
7. Invalid ISO format error
8. Negative timestamp handling

### Component Access Tests (10)
1. Extract year from datetime
2. Extract month from datetime
3. Extract day from datetime
4. Extract hour from datetime
5. Extract minute from datetime
6. Extract second from datetime
7. Extract weekday (Monday=1)
8. Extract day of year
9. Leap year day of year
10. Edge case: year boundary

### Arithmetic Tests (8)
1. Add positive seconds
2. Add negative seconds (subtract)
3. Add days
4. Add hours and minutes
5. Difference between datetimes
6. Compare equal datetimes
7. Compare earlier vs later
8. Large time span arithmetic

### Conversion Tests (4)
1. DateTime to timestamp roundtrip
2. DateTime to ISO string
3. ISO string parse roundtrip
4. Timestamp edge cases (epoch, far future)

**Minimum test count:** 30 tests (all must pass)

## Acceptance Criteria
- ✅ DateTime value type added to Value enum
- ✅ chrono dependency added
- ✅ dateTimeNow returns current UTC time
- ✅ dateTimeFromTimestamp creates DateTime from Unix timestamp
- ✅ dateTimeFromComponents validates and creates DateTime
- ✅ dateTimeParseIso parses basic ISO 8601
- ✅ All 8 component accessors work correctly
- ✅ All 6 arithmetic functions work correctly
- ✅ dateTimeDiff returns correct difference in seconds
- ✅ dateTimeCompare returns correct comparison result
- ✅ Conversion functions roundtrip correctly
- ✅ Invalid inputs produce runtime errors
- ✅ 30+ tests pass (100% pass rate)
- ✅ Interpreter/VM parity maintained
- ✅ cargo test -p atlas-runtime passes (full suite)
- ✅ cargo clippy clean (zero warnings)
- ✅ cargo fmt clean

## Implementation Notes
- Use chrono::DateTime<Utc> internally
- Wrap in Rc<RefCell<DateTime<Utc>>> for Value enum
- All functions are pure stdlib functions (not intrinsics)
- Errors use RuntimeError::new(ErrorCode::ATXXXX, msg)
- Follow existing stdlib patterns from string, array, regex modules

## Handoff to Phase-09b
Phase-09b will add:
- Advanced formatting with custom format strings
- Multiple format parsing
- Timezone operations (local, named zones, conversion)
- Duration type and operations
- Locale-aware formatting

Dependencies: Requires all phase-09a functions and DateTime value type.
