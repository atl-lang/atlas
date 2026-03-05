# DateTime Functions

Date and time creation, arithmetic, conversion, and formatting.

## Construction

### dateTimeNow

```atlas
fn dateTimeNow() -> DateTime
```

Gets current UTC time.

**Returns:** `DateTime` - Current moment in UTC

### dateTimeFromTimestamp

```atlas
fn dateTimeFromTimestamp(timestamp: number) -> DateTime
```

Creates DateTime from Unix timestamp (seconds since epoch).

**Parameters:**
- `timestamp` - Seconds since Unix epoch

**Returns:** `DateTime` - UTC datetime

### dateTimeFromComponents

```atlas
fn dateTimeFromComponents(year: number, month: number, day: number,
                          hour: number, minute: number, second: number) -> DateTime
```

Creates DateTime from date/time components.

**Parameters:**
- `year` - Year (e.g., 2024)
- `month` - Month (1-12)
- `day` - Day of month (1-31)
- `hour` - Hour (0-23)
- `minute` - Minute (0-59)
- `second` - Second (0-59)

**Returns:** `DateTime` - UTC datetime

**Errors:** Invalid ranges for month, day, hour, minute, or second

## Parsing

### dateTimeParseIso

```atlas
fn dateTimeParseIso(text: string) -> Result<DateTime, string>
```

Parses ISO 8601 formatted datetime.

**Parameters:**
- `text` - ISO format string (e.g., "2024-01-15T10:30:00Z")

**Returns:**
- `Ok(DateTime)` on success
- `Err(string)` if invalid format

### dateTimeParse

```atlas
fn dateTimeParse(text: string, format: string) -> Result<DateTime, string>
```

Parses datetime with custom format string.

**Parameters:**
- `text` - DateTime string
- `format` - Format pattern (strftime format)

**Returns:**
- `Ok(DateTime)` on success
- `Err(string)` if invalid

### dateTimeParseRfc3339

```atlas
fn dateTimeParseRfc3339(text: string) -> Result<DateTime, string>
```

Parses RFC 3339 formatted datetime.

**Parameters:**
- `text` - RFC 3339 string

**Returns:**
- `Ok(DateTime)` on success
- `Err(string)` if invalid format

### dateTimeParseRfc2822

```atlas
fn dateTimeParseRfc2822(text: string) -> Result<DateTime, string>
```

Parses RFC 2822 formatted datetime.

**Parameters:**
- `text` - RFC 2822 string (email format)

**Returns:**
- `Ok(DateTime)` on success
- `Err(string)` if invalid format

### dateTimeTryParse

```atlas
fn dateTimeTryParse(text: string) -> Result<DateTime, string>
```

Attempts to parse datetime with automatic format detection.

**Parameters:**
- `text` - DateTime string

**Returns:**
- `Ok(DateTime)` on success
- `Err(string)` if cannot parse

## Component Access

### dateTimeYear

```atlas
fn dateTimeYear(dt: DateTime) -> number
```

Gets year component.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - Year (e.g., 2024)

### dateTimeMonth

```atlas
fn dateTimeMonth(dt: DateTime) -> number
```

Gets month component.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - Month (1-12)

### dateTimeDay

```atlas
fn dateTimeDay(dt: DateTime) -> number
```

Gets day of month component.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - Day (1-31)

### dateTimeHour

```atlas
fn dateTimeHour(dt: DateTime) -> number
```

Gets hour component.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - Hour (0-23)

### dateTimeMinute

```atlas
fn dateTimeMinute(dt: DateTime) -> number
```

Gets minute component.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - Minute (0-59)

### dateTimeSecond

```atlas
fn dateTimeSecond(dt: DateTime) -> number
```

Gets second component.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - Second (0-59)

### dateTimeWeekday

```atlas
fn dateTimeWeekday(dt: DateTime) -> number
```

Gets day of week.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - 0=Monday, 1=Tuesday, ..., 6=Sunday

### dateTimeDayOfYear

```atlas
fn dateTimeDayOfYear(dt: DateTime) -> number
```

Gets day of year.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - 1-366

## Arithmetic

### dateTimeAddSeconds

```atlas
fn dateTimeAddSeconds(dt: DateTime, seconds: number) -> DateTime
```

Adds seconds to datetime.

**Parameters:**
- `dt` - DateTime to modify
- `seconds` - Number of seconds to add

**Returns:** `DateTime` - New datetime

### dateTimeAddMinutes

```atlas
fn dateTimeAddMinutes(dt: DateTime, minutes: number) -> DateTime
```

Adds minutes to datetime.

**Parameters:**
- `dt` - DateTime to modify
- `minutes` - Number of minutes to add

**Returns:** `DateTime` - New datetime

### dateTimeAddHours

```atlas
fn dateTimeAddHours(dt: DateTime, hours: number) -> DateTime
```

Adds hours to datetime.

**Parameters:**
- `dt` - DateTime to modify
- `hours` - Number of hours to add

**Returns:** `DateTime` - New datetime

### dateTimeAddDays

```atlas
fn dateTimeAddDays(dt: DateTime, days: number) -> DateTime
```

Adds days to datetime.

**Parameters:**
- `dt` - DateTime to modify
- `days` - Number of days to add

**Returns:** `DateTime` - New datetime

## Comparison

### dateTimeCompare

```atlas
fn dateTimeCompare(dt1: DateTime, dt2: DateTime) -> number
```

Compares two datetimes.

**Parameters:**
- `dt1` - First datetime
- `dt2` - Second datetime

**Returns:**
- `number` - -1 if dt1 < dt2, 0 if equal, 1 if dt1 > dt2

### dateTimeDiff

```atlas
fn dateTimeDiff(dt1: DateTime, dt2: DateTime) -> number
```

Gets difference between two datetimes in seconds.

**Parameters:**
- `dt1` - First datetime
- `dt2` - Second datetime

**Returns:** `number` - Time difference in seconds (dt1 - dt2)

## Conversion

### dateTimeToTimestamp

```atlas
fn dateTimeToTimestamp(dt: DateTime) -> number
```

Converts datetime to Unix timestamp.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - Seconds since epoch

### dateTimeToIso

```atlas
fn dateTimeToIso(dt: DateTime) -> string
```

Formats datetime as ISO 8601 string.

**Parameters:**
- `dt` - DateTime

**Returns:** `string` - ISO format (e.g., "2024-01-15T10:30:00Z")

### dateTimeToRfc3339

```atlas
fn dateTimeToRfc3339(dt: DateTime) -> string
```

Formats datetime as RFC 3339 string.

**Parameters:**
- `dt` - DateTime

**Returns:** `string` - RFC 3339 format

### dateTimeToRfc2822

```atlas
fn dateTimeToRfc2822(dt: DateTime) -> string
```

Formats datetime as RFC 2822 string.

**Parameters:**
- `dt` - DateTime

**Returns:** `string` - RFC 2822 format (email)

### dateTimeFormat

```atlas
fn dateTimeFormat(dt: DateTime, format: string) -> string
```

Formats datetime with custom format pattern.

**Parameters:**
- `dt` - DateTime
- `format` - Format pattern (strftime format)

**Returns:** `string` - Formatted datetime

### dateTimeToCustom

```atlas
fn dateTimeToCustom(dt: DateTime, format: string) -> string
```

Formats datetime with custom format (alias for dateTimeFormat).

**Parameters:**
- `dt` - DateTime
- `format` - Format pattern

**Returns:** `string` - Formatted datetime

## Timezone Functions

### dateTimeUtc

```atlas
fn dateTimeUtc(dt: DateTime) -> DateTime
```

Converts datetime to UTC.

**Parameters:**
- `dt` - DateTime

**Returns:** `DateTime` - UTC version

### dateTimeToUtc

```atlas
fn dateTimeToUtc(dt: DateTime) -> DateTime
```

Converts datetime to UTC (alias for dateTimeUtc).

**Parameters:**
- `dt` - DateTime

**Returns:** `DateTime` - UTC version

### dateTimeToLocal

```atlas
fn dateTimeToLocal(dt: DateTime) -> DateTime
```

Converts datetime to local system timezone.

**Parameters:**
- `dt` - DateTime

**Returns:** `DateTime` - Local timezone version

### dateTimeToTimezone

```atlas
fn dateTimeToTimezone(dt: DateTime, tz: string) -> Result<DateTime, string>
```

Converts datetime to specified timezone.

**Parameters:**
- `dt` - DateTime
- `tz` - Timezone name (e.g., "America/New_York", "Europe/London")

**Returns:**
- `Ok(DateTime)` in specified timezone
- `Err(string)` if timezone invalid

### dateTimeGetTimezone

```atlas
fn dateTimeGetTimezone(dt: DateTime) -> string
```

Gets timezone of datetime.

**Parameters:**
- `dt` - DateTime

**Returns:** `string` - Timezone name

### dateTimeGetOffset

```atlas
fn dateTimeGetOffset(dt: DateTime) -> number
```

Gets UTC offset in seconds.

**Parameters:**
- `dt` - DateTime

**Returns:** `number` - Seconds offset from UTC

### dateTimeInTimezone

```atlas
fn dateTimeInTimezone(dt: DateTime, tz: string) -> Result<DateTime, string>
```

Creates datetime in specific timezone.

**Parameters:**
- `dt` - DateTime
- `tz` - Timezone name

**Returns:**
- `Ok(DateTime)` in specified timezone
- `Err(string)` if timezone invalid

## Duration Functions

### durationFromSeconds

```atlas
fn durationFromSeconds(seconds: number) -> HashMap<string, number>
```

Creates duration from seconds.

**Parameters:**
- `seconds` - Number of seconds

**Returns:** `HashMap<string, number>` - `{days, hours, minutes, seconds}`

### durationFromMinutes

```atlas
fn durationFromMinutes(minutes: number) -> HashMap<string, number>
```

Creates duration from minutes.

**Parameters:**
- `minutes` - Number of minutes

**Returns:** `HashMap<string, number>` - `{days, hours, minutes, seconds}`

### durationFromHours

```atlas
fn durationFromHours(hours: number) -> HashMap<string, number>
```

Creates duration from hours.

**Parameters:**
- `hours` - Number of hours

**Returns:** `HashMap<string, number>` - `{days, hours, minutes, seconds}`

### durationFromDays

```atlas
fn durationFromDays(days: number) -> HashMap<string, number>
```

Creates duration from days.

**Parameters:**
- `days` - Number of days

**Returns:** `HashMap<string, number>` - `{days, hours, minutes, seconds}`

### durationFormat

```atlas
fn durationFormat(duration: HashMap<string, number>) -> string
```

Formats duration as string.

**Parameters:**
- `duration` - HashMap with `days`, `hours`, `minutes`, `seconds`

**Returns:** `string` - Formatted (e.g., "1h 30m 45s")
