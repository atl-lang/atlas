//! Library project template.
//!
//! Creates a library project with:
//! - Public API exports
//! - Documentation
//! - Test structure
//! - README with badges
//! - License file
//! - Contributing guide

use super::Template;

/// Generate the library project template.
pub fn template() -> Template {
    Template::builder("library")
        .description("A library project with documentation and tests")
        // Directories
        .directory("src")
        .directory("tests")
        .directory("examples")
        .directory("docs")
        // Main library file
        .file("src/lib.atl", LIB_ATL)
        // Internal modules
        .file("src/utils.atl", UTILS_ATL)
        // Test file
        .file("tests/lib_test.atl", LIB_TEST_ATL)
        // Example
        .file("examples/basic.atl", EXAMPLE_ATL)
        // Documentation
        .file("docs/api.md", API_MD)
        // Project files
        .file("atlas.toml", ATLAS_TOML)
        .file("README.md", README_MD)
        .file("LICENSE", LICENSE_MIT)
        .file("CONTRIBUTING.md", CONTRIBUTING_MD)
        .file(".gitignore", GITIGNORE)
        .build()
}

const ATLAS_TOML: &str = r#"[package]
name = "{{name}}"
version = "{{version}}"
description = "{{description}}"
authors = ["{{author}}"]
license = "MIT"
repository = ""
keywords = []
categories = []

[lib]
path = "src/lib.atl"

[dependencies]
# Add dependencies here
# example = "1.0"

[dev-dependencies]
# Add dev dependencies here

[features]
# Define optional features
default = []

[build]
# Build configuration
profile = "release"
"#;

const LIB_ATL: &str = r#"// {{name}} - {{description}}
//
// A library for {{description}}.

import { format_message } from "./utils"

/// Greet a user by name.
///
/// @param name The name to greet
/// @returns A greeting message
///
/// @example
/// ```
/// let msg = greet("World")
/// assert(msg == "Hello, World!")
/// ```
fn greet(name) {
    return format_message("Hello", name)
}

/// Calculate the sum of two numbers.
///
/// @param a First number
/// @param b Second number
/// @returns Sum of a and b
fn add(a, b) {
    return a + b
}

/// Calculate the product of two numbers.
///
/// @param a First number
/// @param b Second number
/// @returns Product of a and b
fn multiply(a, b) {
    return a * b
}

/// Check if a value is positive.
///
/// @param value The value to check
/// @returns true if positive, false otherwise
fn is_positive(value) {
    return value > 0
}

/// Get the library version.
///
/// @returns The current library version string
fn version() {
    return "{{version}}"
}

// Export public API
export { greet, add, multiply, is_positive, version }
"#;

const UTILS_ATL: &str = r#"// Internal utility functions for {{name}}

/// Format a message with a prefix and subject.
///
/// @param prefix The message prefix
/// @param subject The message subject
/// @returns Formatted message string
fn format_message(prefix, subject) {
    return prefix + ", " + subject + "!"
}

/// Clamp a value to a range.
///
/// @param value The value to clamp
/// @param min Minimum allowed value
/// @param max Maximum allowed value
/// @returns Clamped value
fn clamp(value, min, max) {
    if value < min {
        return min
    }
    if value > max {
        return max
    }
    return value
}

/// Check if a string is empty or whitespace only.
///
/// @param s The string to check
/// @returns true if empty or whitespace
fn is_blank(s) {
    return len(trim(s)) == 0
}

export { format_message, clamp, is_blank }
"#;

const LIB_TEST_ATL: &str = r#"// Tests for {{name}} library

import { greet, add, multiply, is_positive, version } from "../src/lib"

// Test greet function
fn test_greet_basic() {
    let result = greet("Atlas")
    assert(result == "Hello, Atlas!")
    return true
}

fn test_greet_empty_name() {
    let result = greet("")
    assert(result == "Hello, !")
    return true
}

// Test add function
fn test_add_positive() {
    assert(add(2, 3) == 5)
    return true
}

fn test_add_negative() {
    assert(add(-1, -2) == -3)
    return true
}

fn test_add_zero() {
    assert(add(0, 5) == 5)
    assert(add(5, 0) == 5)
    return true
}

// Test multiply function
fn test_multiply_positive() {
    assert(multiply(3, 4) == 12)
    return true
}

fn test_multiply_by_zero() {
    assert(multiply(5, 0) == 0)
    assert(multiply(0, 5) == 0)
    return true
}

fn test_multiply_negative() {
    assert(multiply(-2, 3) == -6)
    return true
}

// Test is_positive function
fn test_is_positive_true() {
    assert(is_positive(1) == true)
    assert(is_positive(100) == true)
    return true
}

fn test_is_positive_false() {
    assert(is_positive(0) == false)
    assert(is_positive(-1) == false)
    return true
}

// Test version function
fn test_version_format() {
    let v = version()
    assert(len(v) > 0)
    return true
}

export {
    test_greet_basic,
    test_greet_empty_name,
    test_add_positive,
    test_add_negative,
    test_add_zero,
    test_multiply_positive,
    test_multiply_by_zero,
    test_multiply_negative,
    test_is_positive_true,
    test_is_positive_false,
    test_version_format
}
"#;

const EXAMPLE_ATL: &str = r#"// Example usage of {{name}} library

import { greet, add, multiply } from "../src/lib"

fn main() {
    // Use the greet function
    let message = greet("World")
    print(message)

    // Use math functions
    let sum = add(10, 20)
    print("10 + 20 = " + str(sum))

    let product = multiply(5, 6)
    print("5 * 6 = " + str(product))

    print("Example completed successfully!")
}
"#;

const API_MD: &str = r#"# {{name}} API Documentation

## Overview

{{description}}

## Installation

Add to your `atlas.toml`:

```toml
[dependencies]
{{name}} = "{{version}}"
```

## Functions

### `greet(name)`

Greet a user by name.

**Parameters:**
- `name` - The name to greet

**Returns:** A greeting message string

**Example:**
```atlas
import { greet } from "{{name}}"

let msg = greet("World")
print(msg)  // Output: Hello, World!
```

### `add(a, b)`

Calculate the sum of two numbers.

**Parameters:**
- `a` - First number
- `b` - Second number

**Returns:** Sum of a and b

### `multiply(a, b)`

Calculate the product of two numbers.

**Parameters:**
- `a` - First number
- `b` - Second number

**Returns:** Product of a and b

### `is_positive(value)`

Check if a value is positive.

**Parameters:**
- `value` - The value to check

**Returns:** `true` if positive, `false` otherwise

### `version()`

Get the library version.

**Returns:** The current library version string

## License

MIT License - see LICENSE file for details.
"#;

const README_MD: &str = r#"# {{name}}

{{description}}

## Installation

Add to your `atlas.toml`:

```toml
[dependencies]
{{name}} = "{{version}}"
```

## Quick Start

```atlas
import { greet, add } from "{{name}}"

fn main() {
    let message = greet("World")
    print(message)

    let sum = add(2, 3)
    print("2 + 3 = " + str(sum))
}
```

## Features

- Simple, well-documented API
- Comprehensive test suite
- Example code included

## Documentation

See the [API documentation](docs/api.md) for detailed information.

## Running Tests

```bash
atlas test
```

## Running Examples

```bash
atlas run examples/basic.atl
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

{{author}}
"#;

const LICENSE_MIT: &str = r#"MIT License

Copyright (c) {{year}} {{author}}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;

const CONTRIBUTING_MD: &str = r#"# Contributing to {{name}}

Thank you for your interest in contributing to {{name}}!

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch

## Development

### Running Tests

```bash
atlas test
```

### Code Style

- Use 4 spaces for indentation
- Keep lines under 100 characters
- Add documentation for public functions

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- Keep the first line under 50 characters

## Pull Request Process

1. Update documentation if needed
2. Add tests for new features
3. Ensure all tests pass
4. Update the README if needed

## Code of Conduct

- Be respectful and inclusive
- Give constructive feedback
- Focus on the code, not the person

## Questions?

Open an issue if you have questions or need help.
"#;

const GITIGNORE: &str = r#"# Atlas build artifacts
/target/
/dist/
/.atlas/

# Lock file (uncomment to track)
# atlas.lock

# Editor files
*.swp
*.swo
*~
.idea/
.vscode/

# OS files
.DS_Store
Thumbs.db

# Test output
/coverage/
*.log
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::TemplateContext;

    #[test]
    fn test_library_template_structure() {
        let tmpl = template();
        assert_eq!(tmpl.name, "library");

        // Check required directories
        let dir_names: Vec<_> = tmpl.directories.iter().map(|d| &d.path).collect();
        assert!(dir_names.iter().any(|p| p.to_str() == Some("src")));
        assert!(dir_names.iter().any(|p| p.to_str() == Some("tests")));
        assert!(dir_names.iter().any(|p| p.to_str() == Some("examples")));
    }

    #[test]
    fn test_library_template_files() {
        let tmpl = template();

        let file_names: Vec<_> = tmpl.files.iter().map(|f| &f.path).collect();
        assert!(file_names.iter().any(|p| p.to_str() == Some("src/lib.atl")));
        assert!(file_names.iter().any(|p| p.to_str() == Some("atlas.toml")));
        assert!(file_names.iter().any(|p| p.to_str() == Some("README.md")));
        assert!(file_names.iter().any(|p| p.to_str() == Some("LICENSE")));
    }

    #[test]
    fn test_library_template_render() {
        let tmpl = template();
        let ctx = TemplateContext::for_project("my-lib", "Test Author", "A test library");
        let files = tmpl.render(&ctx);

        // Find atlas.toml and check substitution
        let atlas_toml = files
            .iter()
            .find(|(p, _, _)| p.to_str() == Some("atlas.toml"));
        assert!(atlas_toml.is_some());

        let content = &atlas_toml.unwrap().1;
        assert!(content.contains("name = \"my-lib\""));
        assert!(content.contains("Test Author"));
    }
}
