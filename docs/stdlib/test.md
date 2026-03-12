# test namespace

Testing assertions. See [testing.md](../testing.md) for full test framework guide.

## Functions

### test.assert

```atlas
test.assert(condition: bool, message?: string): void
```

Assert condition is true. Fails with message if false.

```atlas
test.assert(x > 0, "x must be positive");
test.assert(isValid(input));  // message optional
```

### test.equal

```atlas
test.equal(actual: T, expected: T, message?: string): void
```

Assert deep equality. Works with primitives, arrays, options, results.

```atlas
test.equal(2 + 2, 4);
test.equal([1, 2], [1, 2]);  // deep comparison
test.equal(user.name, "alice", "wrong user");
```

### test.notEqual

```atlas
test.notEqual(actual: T, expected: T, message?: string): void
```

Assert values are not equal.

```atlas
test.notEqual(a, b);
```

### test.ok

```atlas
test.ok(result: Result<T, E>): T
```

Assert Result is Ok. Returns unwrapped value.

```atlas
let value = test.ok(parseNumber("42"));
test.equal(value, 42);
```

### test.err

```atlas
test.err(result: Result<T, E>): E
```

Assert Result is Err. Returns unwrapped error.

```atlas
let error = test.err(parseNumber("abc"));
test.equal(error, "invalid number");
```

### test.contains

```atlas
test.contains(array: T[], value: T): void
```

Assert array contains value (deep equality).

```atlas
test.contains([1, 2, 3], 2);
test.contains(users, targetUser);
```

### test.empty

```atlas
test.empty(array: T[]): void
```

Assert array is empty.

```atlas
test.empty([]);
```

### test.approx

```atlas
test.approx(actual: number, expected: number, epsilon: number): void
```

Assert numbers are approximately equal: |actual - expected| <= epsilon.

```atlas
test.approx(3.14159, 3.14, 0.01);
test.approx(calculatePi(), 3.14159265, 0.0001);
```

### test.throws

```atlas
test.throws(fn: () -> any, message?: string): void
```

Assert function throws (returns Err or panics).

```atlas
test.throws(fn(): void { panic("boom"); });
```

### test.noThrow

```atlas
test.noThrow(fn: () -> any, message?: string): void
```

Assert function does not throw.

```atlas
test.noThrow(fn(): void { return; });
```
