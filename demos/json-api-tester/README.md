# JSON API Testing Tool 🧪

A comprehensive HTTP API testing tool built in Atlas that reads test suites from JSON files and validates API responses.

## What This Demonstrates

- **🌐 HTTP Client** - All HTTP methods (GET, POST, PUT, DELETE, PATCH)
- **📊 JSON** - Parsing test suites and validating responses
- **✅ Validation** - Multiple assertion types (status, headers, body, JSON fields)
- **💾 File I/O** - Reading test suites and saving results
- **📅 DateTime** - Timestamps for test duration tracking
- **🗂️ Collections** - Aggregating test results
- **✨ Result Types** - Comprehensive error handling
- **🎨 String Formatting** - Beautiful CLI output

## Features

- ✅ **Multiple HTTP methods** - GET, POST, PUT, DELETE, PATCH support
- 🎯 **Flexible assertions** - Status codes, headers, body content, JSON validation
- 📝 **JSON test suites** - Define tests in readable JSON format
- ⏱️ **Performance tracking** - Measure request duration
- 💾 **Result persistence** - Save test results to JSON files
- 🎨 **Beautiful output** - Clear, formatted test results
- 🔧 **Custom headers & body** - Full control over requests
- ⏰ **Timeout support** - Configure request timeouts

## Project Structure

```
json-api-tester/
├── atlas.toml          # Project configuration
├── main.atl            # Main entry point and orchestration
├── runner.atl          # Test execution engine
├── validator.atl       # Response validation logic
├── reporter.atl        # Results reporting and display
├── tests/              # Test suite JSON files
│   ├── example-api-tests.json  # JSONPlaceholder examples
│   └── httpbin-tests.json      # HTTPBin examples
├── results/            # Test results (auto-generated)
└── README.md           # This file
```

## How to Run (For Beginners)

### Prerequisites

Build the Atlas runtime:

```bash
cd /path/to/atlas
cargo build -p atlas-runtime
```

### Running the Demo

**Option 1: Run with default test suite**

```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/json-api-tester/main.atl
```

This runs the JSONPlaceholder API tests by default.

**Option 2: Run with different test suite**

1. Open `main.atl` in a text editor
2. Find this line:
   ```atlas
   let testSuite: string = "./tests/example-api-tests.json";
   ```
3. Change to another test suite:
   ```atlas
   let testSuite: string = "./tests/httpbin-tests.json";
   ```
4. Run using Option 1

### Example Output

```
═══════════════════════════════════════
  JSON API Testing Tool
═══════════════════════════════════════

╔═══════════════════════════════════════╗
║  API Test Suite: JSONPlaceholder API Tests
╠═══════════════════════════════════════╣
║  Example test suite for JSONPlaceholder public API
║  Tests: 7
╚═══════════════════════════════════════╝

🧪 [1/7] Running: Get all posts - should return 200
🧪 [2/7] Running: Get single post - should return post with id 1
🧪 [3/7] Running: Create new post - should return 201
🧪 [4/7] Running: Update post - should return 200
🧪 [5/7] Running: Delete post - should return 200
🧪 [6/7] Running: Get non-existent post - should return 404
🧪 [7/7] Running: Get users - should return array

Test Execution Complete
─────────────────────────────────────

1. ✅ PASS - Get all posts - should return 200 (234ms)
2. ✅ PASS - Get single post - should return post with id 1 (156ms)
3. ✅ PASS - Create new post - should return 201 (198ms)
4. ✅ PASS - Update post - should return 200 (167ms)
5. ✅ PASS - Delete post - should return 200 (145ms)
6. ✅ PASS - Get non-existent post - should return 404 (123ms)
7. ✅ PASS - Get users - should return array (189ms)

═══════════════════════════════════════
  Test Results Summary
═══════════════════════════════════════
  Total:    7
  Passed:   7 ✅
  Failed:   0 ❌
  Duration: 1212ms
═══════════════════════════════════════

📄 Results saved to: ./results/JSONPlaceholder-1708123456.json
✅ Test suite completed successfully
```

## Test Suite Format

Test suites are defined in JSON format:

```json
{
  "name": "My API Tests",
  "description": "Description of what these tests do",
  "tests": [
    {
      "name": "Test name",
      "method": "GET",
      "url": "https://api.example.com/endpoint",
      "headers": {
        "Content-Type": "application/json",
        "Authorization": "Bearer token"
      },
      "body": {
        "key": "value"
      },
      "timeout": 5000,
      "assertions": [
        {
          "type": "status",
          "value": 200
        },
        {
          "type": "json_field",
          "field": "id",
          "value": 1
        }
      ]
    }
  ]
}
```

## Supported Assertions

### Status Code
```json
{
  "type": "status",
  "value": 200
}
```

### Header Match
```json
{
  "type": "header",
  "name": "Content-Type",
  "value": "application/json"
}
```

### Header Contains
```json
{
  "type": "header_contains",
  "name": "Content-Type",
  "value": "json"
}
```

### Body Contains
```json
{
  "type": "body_contains",
  "value": "success"
}
```

### Body Regex Match
```json
{
  "type": "body_regex",
  "value": "^\\{.*\\}$"
}
```

### JSON Field Exists
```json
{
  "type": "json_field_exists",
  "field": "id"
}
```

### JSON Field Value
```json
{
  "type": "json_field",
  "field": "status",
  "value": "active"
}
```

## Creating Your Own Test Suites

1. Create a new JSON file in the `tests/` directory
2. Define your test suite using the format above
3. Update `main.atl` to point to your test suite
4. Run the tool

### Example: Testing Your Own API

```json
{
  "name": "My API Tests",
  "description": "Tests for my backend API",
  "tests": [
    {
      "name": "Health check",
      "method": "GET",
      "url": "http://localhost:3000/health",
      "assertions": [
        {
          "type": "status",
          "value": 200
        },
        {
          "type": "json_field",
          "field": "status",
          "value": "healthy"
        }
      ]
    },
    {
      "name": "Create user",
      "method": "POST",
      "url": "http://localhost:3000/users",
      "headers": {
        "Content-Type": "application/json"
      },
      "body": {
        "username": "testuser",
        "email": "test@example.com"
      },
      "assertions": [
        {
          "type": "status",
          "value": 201
        },
        {
          "type": "json_field_exists",
          "field": "id"
        }
      ]
    }
  ]
}
```

## Use Cases

- **API Development** - Test your API during development
- **Regression Testing** - Ensure API behavior doesn't break
- **Integration Testing** - Verify third-party API integrations
- **Documentation** - Test suites serve as API documentation
- **CI/CD** - Automated API testing in pipelines

## Troubleshooting

**Error: "Failed to read test suite"**
- Check that the file path is correct
- Ensure the test suite file exists in the `tests/` directory

**Error: "Failed to parse test suite JSON"**
- Validate your JSON syntax (use a JSON validator)
- Check for missing commas or brackets

**Error: "HTTP request failed"**
- Check your internet connection
- Verify the API URL is correct and accessible
- Check if the API requires authentication

**Tests failing unexpectedly**
- Review the assertion error messages
- Check if the API response format has changed
- Verify expected values in assertions

## Extending the Tool

Try adding:
- **Command-line arguments** - Specify test suite from CLI
- **Parallel test execution** - Run tests concurrently
- **Test fixtures** - Setup and teardown logic
- **Environment variables** - Configure URLs per environment
- **HTML reports** - Generate visual test reports
- **Test filtering** - Run specific tests by name/tag
- **Retry logic** - Automatically retry failed tests

## Learning Points

This demo showcases:
- **HTTP testing patterns** - How to test REST APIs
- **Validation strategies** - Multiple ways to verify responses
- **Result aggregation** - Collecting and summarizing test results
- **File-based configuration** - Using JSON for test definitions
- **Error handling** - Graceful failure handling with Result types
- **Modular architecture** - Separation of concerns across modules

## API Documentation

- **JSONPlaceholder**: https://jsonplaceholder.typicode.com/
- **HTTPBin**: https://httpbin.org/

## Next Steps

After exploring this demo, check out:
- `github-stats` - Real-world API integration
- `weather-dashboard` - Multi-city API data aggregation
- `web-crawler` - Link extraction and analysis
