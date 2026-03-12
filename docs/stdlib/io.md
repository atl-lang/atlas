# io namespace

Console input/output operations.

## Reading

### io.readLine

```atlas
io.readLine(): Option<string>
```

Read one line from stdin. Blocks until Enter is pressed.
Returns `None` on EOF (prevents infinite loops when stdin is exhausted).

```atlas
match io.readLine() {
    Some(line) => console.log("Got: " + line),
    None => console.log("EOF"),
}
```

### io.readLinePrompt

```atlas
io.readLinePrompt(prompt: string): Option<string>
```

Print prompt, then read one line from stdin.
Returns `None` on EOF.

```atlas
match io.readLinePrompt("Name: ") {
    Some(name) => console.log("Hello " + name),
    None => console.log("No input"),
}
```

### io.readAll

```atlas
io.readAll(): string
```

Read all of stdin until EOF. Returns entire content as string.

```atlas
let input = io.readAll();
console.log("Read " + input.length().toString() + " chars");
```

## Writing

### io.write

```atlas
io.write(text: string): void
```

Write to stdout without trailing newline.

```atlas
io.write("Loading");
io.write(".");
io.write(".");
io.write(" done!\n");
```

### io.writeLine

```atlas
io.writeLine(text: string): void
```

Write to stdout with trailing newline.

```atlas
io.writeLine("Hello");
```

### io.flush

```atlas
io.flush(): void
```

Flush stdout buffer. Useful after `io.write()` without newline.

```atlas
io.write("Processing...");
io.flush();
// ... work ...
io.writeLine(" done!");
```

## Input Loop Pattern

```atlas
// Safe input loop that exits on EOF
while true {
    match io.readLinePrompt("> ") {
        Some(line) => {
            if line == "quit" {
                break;
            }
            process(line);
        }
        None => {
            console.log("Goodbye!");
            break;
        }
    }
}
```
