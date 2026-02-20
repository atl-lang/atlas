//! Binary executable project template.
//!
//! Creates an executable project with:
//! - Main entry point
//! - CLI argument parsing example
//! - Configuration file support
//! - Logging setup
//! - Error handling patterns

use super::Template;

/// Generate the binary project template.
pub fn template() -> Template {
    Template::builder("binary")
        .description("A binary executable project with CLI support")
        // Directories
        .directory("src")
        .directory("tests")
        .directory("config")
        // Main source files
        .file("src/main.atl", MAIN_ATL)
        .file("src/cli.atl", CLI_ATL)
        .file("src/config.atl", CONFIG_ATL)
        // Configuration
        .file("config/default.toml", DEFAULT_CONFIG)
        // Tests
        .file("tests/main_test.atl", MAIN_TEST_ATL)
        // Project files
        .file("atlas.toml", ATLAS_TOML)
        .file("README.md", README_MD)
        .file("LICENSE", LICENSE_MIT)
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

[[bin]]
name = "{{name}}"
path = "src/main.atl"

[dependencies]
# Add dependencies here
# example = "1.0"

[dev-dependencies]
# Add dev dependencies here

[build]
# Build configuration
profile = "release"
"#;

const MAIN_ATL: &str = r#"// {{name}} - {{description}}
//
// Main entry point for the {{name}} application.

import { parse_args, print_help, print_version } from "./cli"
import { load_config } from "./config"

/// Application entry point.
fn main() {
    // Parse command line arguments
    let args = parse_args()

    // Handle help flag
    if args.help {
        print_help()
        return
    }

    // Handle version flag
    if args.version {
        print_version()
        return
    }

    // Load configuration
    let config = load_config(args.config_path)

    // Display startup message
    if config.verbose {
        print("Starting {{name}} v{{version}}...")
        print("Config loaded from: " + str(args.config_path))
    }

    // Run the main application logic
    run(args, config)

    if config.verbose {
        print("{{name}} completed successfully!")
    }
}

/// Main application logic.
///
/// @param args Parsed command line arguments
/// @param config Loaded configuration
fn run(args, config) {
    // Your application logic here
    print("Hello from {{name}}!")

    // Example: process input if provided
    if args.input != nil {
        print("Processing input: " + str(args.input))
    }
}
"#;

const CLI_ATL: &str = r#"// Command line argument handling for {{name}}

/// Parsed command line arguments.
let default_args = {
    "help": false,
    "version": false,
    "verbose": false,
    "config_path": "config/default.toml",
    "input": nil,
    "output": nil
}

/// Parse command line arguments.
///
/// @returns Object containing parsed arguments
fn parse_args() {
    let args = default_args

    // Get command line arguments (skip program name)
    let argv = get_args()
    let i = 0

    while i < len(argv) {
        let arg = argv[i]

        if arg == "-h" or arg == "--help" {
            args.help = true
        } else if arg == "-V" or arg == "--version" {
            args.version = true
        } else if arg == "-v" or arg == "--verbose" {
            args.verbose = true
        } else if arg == "-c" or arg == "--config" {
            i = i + 1
            if i < len(argv) {
                args.config_path = argv[i]
            }
        } else if arg == "-i" or arg == "--input" {
            i = i + 1
            if i < len(argv) {
                args.input = argv[i]
            }
        } else if arg == "-o" or arg == "--output" {
            i = i + 1
            if i < len(argv) {
                args.output = argv[i]
            }
        } else if not starts_with(arg, "-") {
            // Positional argument
            if args.input == nil {
                args.input = arg
            }
        }

        i = i + 1
    }

    return args
}

/// Print help message.
fn print_help() {
    print("{{name}} v{{version}}")
    print("{{description}}")
    print("")
    print("USAGE:")
    print("    {{name}} [OPTIONS] [INPUT]")
    print("")
    print("OPTIONS:")
    print("    -h, --help       Print help information")
    print("    -V, --version    Print version information")
    print("    -v, --verbose    Enable verbose output")
    print("    -c, --config     Configuration file path")
    print("    -i, --input      Input file or value")
    print("    -o, --output     Output file path")
    print("")
    print("EXAMPLES:")
    print("    {{name}} input.txt")
    print("    {{name}} --verbose -c config.toml")
}

/// Print version information.
fn print_version() {
    print("{{name}} {{version}}")
}

/// Helper: Check if string starts with prefix.
fn starts_with(s, prefix) {
    if len(s) < len(prefix) {
        return false
    }
    return slice(s, 0, len(prefix)) == prefix
}

/// Helper: Get command line arguments.
/// Returns an array of argument strings.
fn get_args() {
    // This would be provided by the Atlas runtime
    // For now, return empty array
    return []
}

export { parse_args, print_help, print_version, default_args }
"#;

const CONFIG_ATL: &str = r#"// Configuration handling for {{name}}

/// Default configuration values.
let default_config = {
    "verbose": false,
    "debug": false,
    "log_level": "info",
    "max_retries": 3,
    "timeout": 30
}

/// Load configuration from file.
///
/// @param path Path to configuration file
/// @returns Configuration object
fn load_config(path) {
    // Start with defaults
    let config = default_config

    // Try to load from file
    // In a real implementation, this would parse the TOML file
    // For now, return defaults
    return config
}

/// Validate configuration values.
///
/// @param config Configuration to validate
/// @returns true if valid, false otherwise
fn validate_config(config) {
    // Check log level
    let valid_levels = ["debug", "info", "warn", "error"]
    if not contains(valid_levels, config.log_level) {
        print("Warning: Invalid log level: " + config.log_level)
        return false
    }

    // Check timeout
    if config.timeout <= 0 {
        print("Warning: Timeout must be positive")
        return false
    }

    // Check retries
    if config.max_retries < 0 {
        print("Warning: max_retries cannot be negative")
        return false
    }

    return true
}

/// Helper: Check if array contains value.
fn contains(arr, value) {
    let i = 0
    while i < len(arr) {
        if arr[i] == value {
            return true
        }
        i = i + 1
    }
    return false
}

export { load_config, validate_config, default_config }
"#;

const DEFAULT_CONFIG: &str = r#"# {{name}} Configuration
# Default configuration file

[general]
# Enable verbose output
verbose = false

# Debug mode
debug = false

# Logging level: debug, info, warn, error
log_level = "info"

[performance]
# Maximum retry attempts
max_retries = 3

# Operation timeout in seconds
timeout = 30

[paths]
# Input directory
# input_dir = "."

# Output directory
# output_dir = "./output"
"#;

const MAIN_TEST_ATL: &str = r#"// Tests for {{name}}

import { parse_args, default_args } from "../src/cli"
import { load_config, validate_config, default_config } from "../src/config"

// CLI Tests

fn test_default_args() {
    assert(default_args.help == false)
    assert(default_args.version == false)
    assert(default_args.verbose == false)
    return true
}

fn test_parse_args_empty() {
    let args = parse_args()
    assert(args.help == false)
    assert(args.version == false)
    return true
}

// Config Tests

fn test_default_config() {
    assert(default_config.verbose == false)
    assert(default_config.debug == false)
    assert(default_config.log_level == "info")
    return true
}

fn test_load_config() {
    let config = load_config("config/default.toml")
    assert(config != nil)
    return true
}

fn test_validate_config_valid() {
    let config = {
        "verbose": true,
        "debug": false,
        "log_level": "info",
        "max_retries": 3,
        "timeout": 30
    }
    assert(validate_config(config) == true)
    return true
}

fn test_validate_config_invalid_timeout() {
    let config = default_config
    config.timeout = -1
    assert(validate_config(config) == false)
    return true
}

export {
    test_default_args,
    test_parse_args_empty,
    test_default_config,
    test_load_config,
    test_validate_config_valid,
    test_validate_config_invalid_timeout
}
"#;

const README_MD: &str = r#"# {{name}}

{{description}}

## Installation

Build from source:

```bash
atlas build --release
```

## Usage

```bash
{{name}} [OPTIONS] [INPUT]
```

### Options

| Option | Description |
|--------|-------------|
| `-h, --help` | Print help information |
| `-V, --version` | Print version |
| `-v, --verbose` | Enable verbose output |
| `-c, --config FILE` | Configuration file path |
| `-i, --input FILE` | Input file |
| `-o, --output FILE` | Output file |

### Examples

```bash
# Basic usage
{{name}} input.txt

# With verbose output
{{name}} --verbose input.txt

# Custom configuration
{{name}} -c custom.toml input.txt

# Specify output
{{name}} -i input.txt -o output.txt
```

## Configuration

Configuration is loaded from `config/default.toml` by default.

```toml
[general]
verbose = false
debug = false
log_level = "info"

[performance]
max_retries = 3
timeout = 30
```

## Development

### Running Tests

```bash
atlas test
```

### Building

```bash
# Development build
atlas build

# Release build
atlas build --release
```

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

# Log files
*.log
/logs/

# Local configuration
config/local.toml
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::TemplateContext;

    #[test]
    fn test_binary_template_structure() {
        let tmpl = template();
        assert_eq!(tmpl.name, "binary");

        // Check required directories
        let dir_names: Vec<_> = tmpl.directories.iter().map(|d| &d.path).collect();
        assert!(dir_names.iter().any(|p| p.to_str() == Some("src")));
        assert!(dir_names.iter().any(|p| p.to_str() == Some("tests")));
        assert!(dir_names.iter().any(|p| p.to_str() == Some("config")));
    }

    #[test]
    fn test_binary_template_files() {
        let tmpl = template();

        let file_names: Vec<_> = tmpl.files.iter().map(|f| &f.path).collect();
        assert!(file_names
            .iter()
            .any(|p| p.to_str() == Some("src/main.atl")));
        assert!(file_names.iter().any(|p| p.to_str() == Some("src/cli.atl")));
        assert!(file_names.iter().any(|p| p.to_str() == Some("atlas.toml")));
        assert!(file_names.iter().any(|p| p.to_str() == Some("README.md")));
    }

    #[test]
    fn test_binary_template_render() {
        let tmpl = template();
        let ctx = TemplateContext::for_project("my-app", "Test Author", "A test app");
        let files = tmpl.render(&ctx);

        // Find atlas.toml and check substitution
        let atlas_toml = files
            .iter()
            .find(|(p, _, _)| p.to_str() == Some("atlas.toml"));
        assert!(atlas_toml.is_some());

        let content = &atlas_toml.unwrap().1;
        assert!(content.contains("name = \"my-app\""));
        assert!(content.contains("[[bin]]"));
    }

    #[test]
    fn test_binary_has_config_directory() {
        let tmpl = template();
        let has_config_dir = tmpl
            .directories
            .iter()
            .any(|d| d.path.to_str() == Some("config"));
        assert!(has_config_dir);
    }
}
