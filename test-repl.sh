#!/bin/bash
# Test script for REPL functionality

echo "Testing Atlas REPL..."
echo ""

# Test 1: Simple expression
echo "=== Test 1: Simple expression ==="
echo "1 + 2;" | cargo run --package atlas-cli -- repl

echo ""
echo "=== Test 2: Variable declaration and use ==="
printf "let x = 42;\nx;\n:quit\n" | cargo run --package atlas-cli -- repl

echo ""
echo "=== Test 3: Function declaration and call ==="
printf "fn add(a: number, b: number) -> number { return a + b; }\nadd(10, 20);\n:quit\n" | cargo run --package atlas-cli -- repl

echo ""
echo "=== Test 4: String operations ==="
printf '"hello" + " world";\n:quit\n' | cargo run --package atlas-cli -- repl

echo ""
echo "=== Test 5: Array operations ==="
printf "let arr = [1, 2, 3];\narr[1];\n:quit\n" | cargo run --package atlas-cli -- repl

echo ""
echo "All tests completed!"
