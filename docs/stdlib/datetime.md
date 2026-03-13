# DateTime — Date and Time Operations

Namespace: `DateTime` (PascalCase, D-049)

The `DateTime` namespace provides construction, parsing, component access, arithmetic,
formatting, timezone conversion, and duration operations. All Atlas `DateTime` values are
internally stored in UTC using `chrono::DateTime<Utc>`.

---

## Import

No import required. `DateTime` is a built-in namespace.

---

## Construction Functions

### `DateTime.now() -> DateTime`

Get the current time in UTC.

```atlas
let now = DateTime.now();
```

---

### `DateTime.utc() -> DateTime`

Alias for `DateTime.now()`. Returns the current UTC time.

```atlas
let utc = DateTime.utc();
```

---

### `DateTime.fromTimestamp(timestamp: number) -> DateTime`

Create a `DateTime` from a Unix timestamp (seconds since the Unix epoch, Jan 1 1970 UTC).
The `timestamp` is truncated to an integer.

```atlas
let dt = DateTime.fromTimestamp(1609459200); // 2021-01-01 00:00:00 UTC
```

**Errors:** `TypeError` if the timestamp value is out of the valid range.

---

### `DateTime.fromComponents(year: number, month: number, day: number, hour: number, minute: number, second: number) -> DateTime`

Create a `DateTime` from individual date and time components. All values are in UTC.

| Parameter | Range |
|-----------|-------|
| `year` | any valid calendar year |
| `month` | 1–12 |
| `day` | 1–31 |
| `hour` | 0–23 |
| `minute` | 0–59 |
| `second` | 0–59 |

```atlas
let dt = DateTime.fromComponents(2024, 6, 15, 10, 30, 0);
// 2024-06-15 10:30:00 UTC
```

**Errors:** `TypeError` if any component is out of range, or if the combination does not
form a valid date (e.g., February 30).

---

### `DateTime.parseIso(text: string) -> DateTime`

Parse an ISO 8601 datetime string. Accepts formats recognized by the `chrono` crate
(e.g., `"2024-01-15T10:30:00Z"`).

```atlas
let dt = DateTime.parseIso("2024-01-15T10:30:00Z");
```

**Errors:** `TypeError` if the string cannot be parsed.

---

### `DateTime.parse(text: string, format: string) -> DateTime`

Parse a datetime string using a `strftime`-compatible format string. The parsed result is
treated as UTC.

```atlas
let dt = DateTime.parse("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S");
```

---

### `DateTime.parseRfc3339(text: string) -> DateTime`

Parse an RFC 3339 formatted datetime string (e.g., `"2024-01-15T10:30:00+00:00"`).
Converts the result to UTC.

```atlas
let dt = DateTime.parseRfc3339("2024-01-15T10:30:00+05:30");
```

---

### `DateTime.parseRfc2822(text: string) -> DateTime`

Parse an RFC 2822 formatted datetime string (email/HTTP date format,
e.g., `"Mon, 15 Jan 2024 10:30:00 +0000"`). Converts the result to UTC.

```atlas
let dt = DateTime.parseRfc2822("Mon, 15 Jan 2024 10:30:00 +0000");
```

---

### `DateTime.tryParse(text: string, formats: string[]) -> DateTime`

Try parsing a datetime string against multiple format strings in order. Returns the first
successful parse. Throws `TypeError` if no format matches.

```atlas
let dt = DateTime.tryParse("2024-01-15", ["%Y-%m-%d", "%d/%m/%Y", "%m-%d-%Y"]);
```

---

## Component Access

All component functions take a `DateTime` and return a `number`.

| Function | Returns | Range |
|----------|---------|-------|
| `DateTime.year(dt)` | Year number | e.g., 2024 |
| `DateTime.month(dt)` | Month | 1–12 |
| `DateTime.day(dt)` | Day of month | 1–31 |
| `DateTime.hour(dt)` | Hour (24h) | 0–23 |
| `DateTime.minute(dt)` | Minute | 0–59 |
| `DateTime.second(dt)` | Second | 0–59 |
| `DateTime.weekday(dt)` | ISO 8601 weekday | 1=Monday, 7=Sunday |
| `DateTime.dayOfYear(dt)` | Day of year | 1–366 |

```atlas
let now = DateTime.now();
let year = DateTime.year(now);
let month = DateTime.month(now);
let day = DateTime.day(now);
let hour = DateTime.hour(now);
let weekday = DateTime.weekday(now); // 1 = Monday

console.log(year.toString() + "-" + month.toString() + "-" + day.toString());
```

---

## DateTime Instance Methods

`DateTime` values also support instance method syntax:

| Method | Signature | Description |
|--------|-----------|-------------|
| `.year()` | `() -> number` | Extract year |
| `.month()` | `() -> number` | Extract month (1–12) |
| `.day()` | `() -> number` | Extract day (1–31) |
| `.hour()` | `() -> number` | Extract hour (0–23) |
| `.minute()` | `() -> number` | Extract minute (0–59) |
| `.second()` | `() -> number` | Extract second (0–59) |
| `.timestamp()` | `() -> number` | Convert to Unix timestamp |
| `.format(fmt)` | `(string) -> string` | Format with strftime pattern |
| `.addDays(n)` | `(number) -> DateTime` | Add/subtract days |
| `.addHours(n)` | `(number) -> DateTime` | Add/subtract hours |

---

## Arithmetic Functions

All arithmetic functions accept negative values for subtraction.

### `DateTime.addSeconds(dt: DateTime, seconds: number) -> DateTime`

```atlas
let later = DateTime.addSeconds(now, 3600); // 1 hour from now
let earlier = DateTime.addSeconds(now, -60); // 1 minute ago
```

### `DateTime.addMinutes(dt: DateTime, minutes: number) -> DateTime`

```atlas
let dt = DateTime.addMinutes(now, 30);
```

### `DateTime.addHours(dt: DateTime, hours: number) -> DateTime`

```atlas
let tomorrow = DateTime.addHours(now, 24);
```

### `DateTime.addDays(dt: DateTime, days: number) -> DateTime`

```atlas
let nextWeek = DateTime.addDays(now, 7);
let lastMonth = DateTime.addDays(now, -30);
```

**Errors:** `TypeError` on overflow.

---

### `DateTime.diff(dt1: DateTime, dt2: DateTime) -> number`

Calculate the difference between two `DateTime` values in seconds. Returns
`dt1 - dt2`. Result is negative if `dt1` is before `dt2`.

```atlas
let start = DateTime.parseIso("2024-01-01T00:00:00Z");
let end = DateTime.parseIso("2024-01-02T00:00:00Z");
let diffSeconds = DateTime.diff(end, start); // 86400
```

---

### `DateTime.compare(dt1: DateTime, dt2: DateTime) -> number`

Compare two `DateTime` values. Returns:
- `-1` if `dt1` is before `dt2`
- `0` if they are equal
- `1` if `dt1` is after `dt2`

```atlas
let cmp = DateTime.compare(start, end); // -1
```

---

## Conversion / Formatting Functions

### `DateTime.toTimestamp(dt: DateTime) -> number`

Convert a `DateTime` to a Unix timestamp (seconds since epoch).

```atlas
let ts = DateTime.toTimestamp(DateTime.now());
```

---

### `DateTime.toIso(dt: DateTime) -> string`

Format as ISO 8601 / RFC 3339 string (e.g., `"2024-01-15T10:30:00+00:00"`).

```atlas
let iso = DateTime.toIso(now); // "2024-01-15T10:30:00+00:00"
```

---

### `DateTime.toRfc3339(dt: DateTime) -> string`

Format as RFC 3339 string. Equivalent to `DateTime.toIso()`.

```atlas
let rfc = DateTime.toRfc3339(now);
```

---

### `DateTime.toRfc2822(dt: DateTime) -> string`

Format as RFC 2822 string (email/HTTP format,
e.g., `"Mon, 15 Jan 2024 10:30:00 +0000"`).

```atlas
let email = DateTime.toRfc2822(now);
```

---

### `DateTime.format(dt: DateTime, format: string) -> string`

Format a `DateTime` using a `strftime`-compatible format string.

**Common format specifiers:**

| Specifier | Output |
|-----------|--------|
| `%Y` | 4-digit year (2024) |
| `%m` | Zero-padded month (01–12) |
| `%d` | Zero-padded day (01–31) |
| `%H` | 24-hour hour (00–23) |
| `%M` | Minute (00–59) |
| `%S` | Second (00–59) |
| `%A` | Full weekday name (Monday) |
| `%B` | Full month name (January) |
| `%z` | UTC offset (+0000) |
| `%Z` | Timezone abbreviation (UTC) |

```atlas
let formatted = DateTime.format(now, "%Y-%m-%d %H:%M:%S");
// "2024-06-15 10:30:00"

let pretty = DateTime.format(now, "%A, %B %d %Y");
// "Saturday, June 15 2024"
```

---

### `DateTime.toCustom(dt: DateTime, format: string) -> string`

Alias for `DateTime.format()`. Uses the same strftime format syntax.

---

## Timezone Operations

All `DateTime` values are internally stored in UTC. Timezone conversion functions compute
the offset and store the result back in UTC.

### `DateTime.toUtc(dt: DateTime) -> DateTime`

Convert to UTC (no-op for Atlas DateTimes which are already UTC).

### `DateTime.toLocal(dt: DateTime) -> DateTime`

Convert to the system's local timezone, then store back as UTC equivalent.

### `DateTime.toTimezone(dt: DateTime, tz: string) -> DateTime`

Convert a `DateTime` to a named IANA timezone. The result is stored in UTC.
Uses `chrono_tz` for timezone data.

```atlas
let ny = DateTime.toTimezone(now, "America/New_York");
let tokyo = DateTime.toTimezone(now, "Asia/Tokyo");
```

**Errors:** `TypeError` if the timezone name is not recognized.

### `DateTime.inTimezone(dt: DateTime, tz: string) -> DateTime`

Interpret a `DateTime`'s naive (wall clock) components as if they were in the named
timezone, then convert to UTC. Useful when you have a local time and know its timezone.

```atlas
let local_noon = DateTime.fromComponents(2024, 6, 15, 12, 0, 0);
let utc_noon_ny = DateTime.inTimezone(local_noon, "America/New_York");
```

**Errors:** `TypeError` if the timezone is ambiguous (DST gap) or invalid.

### `DateTime.getTimezone(dt: DateTime) -> string`

Returns `"UTC"` — all Atlas DateTimes are stored in UTC.

### `DateTime.getOffset(dt: DateTime) -> number`

Returns `0` — all Atlas DateTimes are stored in UTC.

---

## Duration Functions

Duration values are `HashMap<string, number>` with keys `days`, `hours`, `minutes`,
`seconds`. All components have the same sign (negative if the duration is negative).

### `DateTime.durationFromSeconds(seconds: number) -> {days, hours, minutes, seconds}`

```atlas
let dur = DateTime.durationFromSeconds(3665);
// {days: 0, hours: 1, minutes: 1, seconds: 5}
```

### `DateTime.durationFromMinutes(minutes: number) -> {days, hours, minutes, seconds}`

```atlas
let dur = DateTime.durationFromMinutes(90);
// {days: 0, hours: 1, minutes: 30, seconds: 0}
```

### `DateTime.durationFromHours(hours: number) -> {days, hours, minutes, seconds}`

```atlas
let dur = DateTime.durationFromHours(25);
// {days: 1, hours: 1, minutes: 0, seconds: 0}
```

### `DateTime.durationFromDays(days: number) -> {days, hours, minutes, seconds}`

```atlas
let dur = DateTime.durationFromDays(2);
// {days: 2, hours: 0, minutes: 0, seconds: 0}
```

### `DateTime.durationFormat(duration: {days, hours, minutes, seconds}) -> string`

Format a duration map as a human-readable string. Omits zero components (except when all
are zero, in which case just seconds are shown). Prefix with `-` for negative durations.

```atlas
let dur = DateTime.durationFromSeconds(5400);
let label = DateTime.durationFormat(dur); // "1h 30m"

let dur2 = DateTime.durationFromSeconds(3661);
let label2 = DateTime.durationFormat(dur2); // "1h 1m 1s"
```

---

## Common Patterns

### Get current date components

```atlas
let now = DateTime.now();
let y = DateTime.year(now);
let m = DateTime.month(now);
let d = DateTime.day(now);
console.log(y.toString() + "-" + m.toString() + "-" + d.toString());
```

### Calculate time elapsed

```atlas
let start = DateTime.now();
// ... do work ...
let end = DateTime.now();
let elapsed = DateTime.diff(end, start); // seconds
let fmt = DateTime.durationFormat(DateTime.durationFromSeconds(elapsed));
console.log("Elapsed: " + fmt);
```

### Parse multiple possible date formats

```atlas
let raw = "15/06/2024";
let dt = DateTime.tryParse(raw, ["%d/%m/%Y", "%Y-%m-%d", "%m-%d-%Y"]);
console.log(DateTime.toIso(dt));
```

### Convert a timestamp to a formatted date

```atlas
let ts = 1718448000;
let dt = DateTime.fromTimestamp(ts);
let label = DateTime.format(dt, "%d %B %Y");
// "15 June 2024"
```

---

## Error Behavior

| Condition | Error |
|-----------|-------|
| Invalid components in `fromComponents` | `TypeError` |
| Invalid timestamp in `fromTimestamp` | `TypeError` |
| Unparseable string | `TypeError` |
| Unknown timezone name | `TypeError` |
| Ambiguous DST datetime in `inTimezone` | `TypeError` |
| Overflow in arithmetic | `TypeError` |
