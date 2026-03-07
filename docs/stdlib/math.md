# Math Functions

Complete math API following IEEE 754 semantics.

## Basic Operations

### abs

```atlas
fn abs(x: number) -> number
```

Returns absolute value of x. Preserves signed zero: `abs(-0) = +0`.

**Returns:**
- `number` - Absolute value
- Preserves: `abs(±∞) = +∞`, `abs(NaN) = NaN`

### floor

```atlas
fn floor(x: number) -> number
```

Returns largest integer ≤ x.

**Examples:**
- `floor(1.9) = 1`
- `floor(-1.1) = -2`

**Returns:** Largest integer ≤ x

### ceil

```atlas
fn ceil(x: number) -> number
```

Returns smallest integer ≥ x.

**Examples:**
- `ceil(1.1) = 2`
- `ceil(-1.9) = -1`

**Returns:** Smallest integer ≥ x

### round

```atlas
fn round(x: number) -> number
```

Rounds to nearest integer using banker's rounding (ties-to-even).

**Examples:**
- `round(2.5) = 2` (rounds to even)
- `round(3.5) = 4` (rounds to even)

**Returns:** Rounded number

### min

```atlas
fn min(a: number, b: number) -> number
```

Returns smaller of two numbers.

**Note:** If either is NaN, returns NaN

### max

```atlas
fn max(a: number, b: number) -> number
```

Returns larger of two numbers.

**Note:** If either is NaN, returns NaN

## Exponential and Power Operations

### sqrt

```atlas
fn sqrt(x: number) -> Result<number, string>
```

Returns square root of x.

> **⚠️ AI Generation Note:** `sqrt` returns `Result<number, string>`, **not** `number`.
> Writing `let x = sqrt(16)` fails — the typechecker sees `Result`, not a `number`.
> **Correct pattern:** `let x = unwrap(sqrt(16));`
> Use `match` for safe unwrapping when input may be negative.

**Returns:**
- `Ok(sqrt)` for non-negative x
- `Err(string)` for negative x

**Note:** `sqrt(+∞) = Ok(+∞)`, `sqrt(NaN) = Err`

### pow

```atlas
fn pow(base: number, exponent: number) -> number
```

Returns base raised to exponent power.

**Special cases:**
- `pow(x, 0) = 1` for any x (including NaN)
- `pow(1, y) = 1` for any y (including NaN)
- `pow(NaN, y) = NaN` (except y=0)

### log

```atlas
fn log(x: number) -> Result<number, string>
```

Returns natural logarithm (ln) of x.

**Returns:**
- `Ok(ln(x))` for x > 0
- `Err(string)` for x ≤ 0

## Trigonometric Functions (in radians)

### sin

```atlas
fn sin(x: number) -> number
```

Returns sine of x (x in radians).

**Note:** `sin(±∞) = NaN`, `sin(NaN) = NaN`

### cos

```atlas
fn cos(x: number) -> number
```

Returns cosine of x (x in radians).

**Note:** `cos(±∞) = NaN`, `cos(NaN) = NaN`

### tan

```atlas
fn tan(x: number) -> number
```

Returns tangent of x (x in radians).

**Note:** `tan(±∞) = NaN`, `tan(NaN) = NaN`

### asin

```atlas
fn asin(x: number) -> Result<number, string>
```

Returns arcsine of x.

**Returns:**
- `Ok(asin(x))` for x in [-1, 1]
- `Err(string)` otherwise

### acos

```atlas
fn acos(x: number) -> Result<number, string>
```

Returns arccosine of x.

**Returns:**
- `Ok(acos(x))` for x in [-1, 1]
- `Err(string)` otherwise

### atan

```atlas
fn atan(x: number) -> number
```

Returns arctangent of x in radians.

**Range:** (-π/2, π/2)

**Note:** `atan(±∞) = ±π/2`, `atan(NaN) = NaN`

## Utility Functions

### clamp

```atlas
fn clamp(value: number, min: number, max: number) -> Result<number, string>
```

Restricts value to [min, max] range.

**Returns:**
- `Ok(clamped)` if valid
- `Err(string)` if min > max or any argument is NaN

### sign

```atlas
fn sign(x: number) -> number
```

Returns sign of x: -1 for negative, 0 for zero, 1 for positive.

**Special cases:**
- Preserves signed zero: `sign(-0) = -0`, `sign(+0) = +0`
- `sign(NaN) = NaN`

### random

```atlas
fn random() -> number
```

Returns pseudo-random number in [0, 1) with uniform distribution.

**Note:** Uses thread-local RNG for randomness

## Constants

```atlas
PI = 3.141592653589793
E = 2.718281828459045
SQRT2 = 1.414213562373095
LN2 = 0.6931471805599453
LN10 = 2.302585092994046
```

These are available as module-level constants.
