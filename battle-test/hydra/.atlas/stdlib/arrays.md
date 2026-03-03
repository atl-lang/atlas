# Array Operations

## ✅ Available Functions

### len()
Get array length:

```atlas
let items: array = [1, 2, 3];
let count: number = len(items);  // 3
```

### split()
Convert string to array:

```atlas
let text: string = "a,b,c";
let parts: array = split(text, ",");  // ["a", "b", "c"]

let lines: string = "line1\nline2";
let line_array: array = split(lines, "\n");
```

### for...in
Iterate over array:

```atlas
let items: array = [1, 2, 3];

for item in items {
    print(str(item));
}
```

## ❌ NOT Available / Unconfirmed

### push() / append()
**Status**: Not confirmed

**Workaround**: Build new array or use string concatenation

### pop() / remove()
**Status**: Not confirmed

### map() / filter() / reduce()
**Status**: Functional operations not observed

**Workaround**: Use for loops

```atlas
// Instead of map:
fn process_items(items: array) -> array {
    // Manual iteration and building
    for item in items {
        // process
    }
}
```

### sort()
**Status**: Not confirmed

### indexOf() / contains()
**Status**: Not confirmed

**Workaround**: Manual loop

```atlas
fn array_contains(arr: array, target: string) -> bool {
    for item in arr {
        if (item == target) {
            return true;
        }
    }
    return false;
}
```

### slice() / subarray()
**Status**: Not confirmed

## Array Creation

```atlas
// Literal:
let numbers: array = [1, 2, 3];

// From split:
let items: array = split("a,b,c", ",");

// Empty array:
let empty: array = [];
```

## Working Patterns

### Filter Array

```atlas
fn filter_lines(input: string) -> string {
    let lines: array = split(input, "\n");
    let mut result: string = "";
    let mut first: bool = true;

    for line in lines {
        if (should_keep(line)) {
            if (!first) {
                result = result + "\n";
            }
            result = result + line;
            first = false;
        }
    }

    return result;
}
```

### Process All Items

```atlas
fn process_all(items: array) -> void {
    for item in items {
        let text: string = item;  // Assuming string array
        print("Processing: " + text);
    }
}
```

### Count Matching

```atlas
fn count_matches(items: array, target: string) -> number {
    let mut count: number = 0;

    for item in items {
        if (item == target) {
            count = count + 1;
        }
    }

    return count;
}
```

## Limitations

- **No type parameters**: Can't specify `array<string>`, just `array`
- **No mutation helpers**: No push/pop/remove observed
- **No functional ops**: No map/filter/reduce
- **Manual processing**: Use for loops for everything

## Summary

- **Creation**: ✅ Literals and split()
- **Iteration**: ✅ for...in loops
- **Length**: ✅ len()
- **Mutation**: ❌ Not confirmed
- **Functional**: ❌ Not available
- **Workaround**: Use for loops and build results manually
