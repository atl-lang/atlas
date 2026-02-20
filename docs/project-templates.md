# Atlas Project Templates

This guide documents the project templating system for Atlas. The `atlas new` command creates new projects from templates with complete structure, tests, and documentation.

## Quick Start

```bash
# Create a binary application
atlas new my-app

# Create a library
atlas new my-lib --lib

# Create a web server
atlas new my-api --web

# List available templates
atlas new dummy --list
```

## Available Templates

### Binary Template

Creates an executable project with CLI support.

```bash
atlas new my-app
# or
atlas new my-app --template=binary
```

**Structure:**
```
my-app/
├── atlas.toml           # Project manifest
├── src/
│   ├── main.atl         # Entry point
│   ├── cli.atl          # CLI argument parsing
│   └── config.atl       # Configuration handling
├── config/
│   └── default.toml     # Default configuration
├── tests/
│   └── main_test.atl    # Tests
├── README.md
├── LICENSE
└── .gitignore
```

**Features:**
- Command-line argument parsing example
- Configuration file support (TOML)
- Error handling patterns
- Logging setup
- Test structure

### Library Template

Creates a library project with documentation and tests.

```bash
atlas new my-lib --lib
# or
atlas new my-lib --template=library
```

**Structure:**
```
my-lib/
├── atlas.toml           # Project manifest
├── src/
│   ├── lib.atl          # Library entry point
│   └── utils.atl        # Utility functions
├── tests/
│   └── lib_test.atl     # Library tests
├── examples/
│   └── basic.atl        # Usage example
├── docs/
│   └── api.md           # API documentation
├── README.md
├── LICENSE
├── CONTRIBUTING.md
└── .gitignore
```

**Features:**
- Public API with exports
- Documentation examples
- Comprehensive test suite
- Example code
- Contributing guidelines
- API documentation

### Web Server Template

Creates a web server project with HTTP routing.

```bash
atlas new my-api --web
# or
atlas new my-api --template=web
```

**Structure:**
```
my-api/
├── atlas.toml           # Project manifest
├── src/
│   ├── main.atl         # Entry point
│   ├── server.atl       # HTTP server
│   ├── router.atl       # Routing utilities
│   ├── routes/
│   │   ├── mod.atl      # Route setup
│   │   ├── api.atl      # API endpoints
│   │   └── pages.atl    # Page handlers
│   └── middleware/
│       ├── mod.atl      # Middleware exports
│       └── logger.atl   # Request logging
├── static/
│   ├── css/style.css    # Stylesheet
│   └── js/app.js        # Client JavaScript
├── templates/
│   ├── index.html       # Home page
│   └── error.html       # Error page
├── config/
│   └── default.toml     # Server configuration
├── tests/
│   └── server_test.atl  # Server tests
├── .env.example         # Environment variables template
├── Dockerfile           # Docker support
├── README.md
├── LICENSE
└── .gitignore
```

**Features:**
- HTTP server implementation
- URL routing
- Middleware support
- Static file serving
- HTML templates
- API endpoints (health, users)
- Docker support
- Environment configuration

## Command Reference

### `atlas new`

Create a new Atlas project from a template.

```
USAGE:
    atlas new <NAME> [OPTIONS]

ARGUMENTS:
    <NAME>    Project name (creates directory with this name)

OPTIONS:
    --lib                  Create a library project
    --web                  Create a web server project
    -t, --template <TYPE>  Template type (binary, library, web)
    --author <NAME>        Author name
    --description <TEXT>   Project description
    --no-git               Skip git initialization
    --no-commit            Skip initial commit
    --force                Overwrite existing directory
    --list                 List available templates
    -v, --verbose          Verbose output
    -h, --help             Print help

ALIASES:
    atlas n                Short alias for 'new'
```

### Examples

```bash
# Create binary project with all options
atlas new my-app \
  --author "Jane Doe" \
  --description "My awesome application"

# Create library with verbose output
atlas new my-lib --lib --verbose

# Create web server without git
atlas new my-api --web --no-git

# Force overwrite existing project
atlas new existing-project --force

# Use explicit template flag
atlas new my-project --template=binary
```

## Template Variables

Templates support variable substitution using the `{{variable}}` syntax.

| Variable | Description | Example |
|----------|-------------|---------|
| `{{name}}` | Project name | `my-project` |
| `{{name_snake}}` | Snake case name | `my_project` |
| `{{name_pascal}}` | Pascal case name | `MyProject` |
| `{{author}}` | Author name | `Jane Doe` |
| `{{description}}` | Project description | `A cool project` |
| `{{version}}` | Initial version | `0.1.0` |
| `{{year}}` | Current year | `2026` |

**Example in template:**
```toml
[package]
name = "{{name}}"
version = "{{version}}"
authors = ["{{author}}"]
```

**Rendered:**
```toml
[package]
name = "my-project"
version = "0.1.0"
authors = ["Jane Doe"]
```

## Project Naming Rules

Valid project names must:
- Start with a letter (a-z, A-Z)
- Contain only letters, numbers, hyphens, and underscores
- Be 64 characters or less
- Not be a reserved name

**Reserved names:**
- `atlas`, `std`, `core`, `test`, `debug`, `self`, `super`, `crate`

**Valid examples:**
- `my-project`
- `my_project`
- `project123`
- `MyAwesomeApp`

**Invalid examples:**
- `-invalid` (starts with hyphen)
- `123project` (starts with number)
- `has space` (contains space)
- `atlas` (reserved)

## Git Integration

By default, `atlas new` initializes a Git repository and creates an initial commit.

```bash
# Default behavior
atlas new my-project  # Git initialized + initial commit

# Skip git entirely
atlas new my-project --no-git

# Initialize git but skip initial commit
atlas new my-project --no-commit
```

The generated `.gitignore` includes:
- `/target/` - Build artifacts
- `/dist/` - Distribution files
- `/.atlas/` - Atlas cache
- Editor files (`.swp`, `.idea/`, `.vscode/`)
- OS files (`.DS_Store`, `Thumbs.db`)
- Logs and local configuration

## Generated Files

### atlas.toml

Project manifest with package metadata:

```toml
[package]
name = "my-project"
version = "0.1.0"
description = "Project description"
authors = ["Author Name"]
license = "MIT"

[[bin]]
name = "my-project"
path = "src/main.atl"

[dependencies]
# Add dependencies here

[dev-dependencies]
# Add dev dependencies here
```

### README.md

Project documentation with:
- Title and description
- Installation instructions
- Usage examples
- Development guide
- License information

### LICENSE

MIT License by default, with author name and year substituted.

## After Creating a Project

After creating a project, you can:

**Binary project:**
```bash
cd my-app
atlas run src/main.atl
```

**Library project:**
```bash
cd my-lib
atlas test
atlas run examples/basic.atl
```

**Web project:**
```bash
cd my-api
atlas run src/main.atl
# Server starts at http://localhost:8080
```

## Building and Testing

```bash
# Build the project
atlas build

# Build for release
atlas build --release

# Run tests
atlas test

# Format code
atlas fmt .
```

## Tips and Best Practices

1. **Choose the right template**: Start with the template closest to your needs, then customize.

2. **Use descriptive names**: Project names become directory names and appear in generated files.

3. **Set author information**: Use `--author` for proper attribution in generated files.

4. **Add a description**: Use `--description` for meaningful README and manifest content.

5. **Review generated structure**: Templates provide a starting point—modify as needed.

6. **Keep git history**: The initial commit provides a clean baseline for changes.

## Troubleshooting

### "Directory already exists"

Use `--force` to overwrite:
```bash
atlas new existing-project --force
```

### "Invalid project name"

Ensure the name:
- Starts with a letter
- Contains only letters, numbers, hyphens, underscores
- Is not a reserved name

### "Git initialization failed"

Git may not be installed. Use `--no-git`:
```bash
atlas new my-project --no-git
```

### Interactive prompts in scripts

Pass `--author` and `--description` to skip prompts:
```bash
atlas new my-project --author "Bot" --description "Automated"
```

## See Also

- [Package Manager CLI](package-manager-cli.md) - Managing dependencies
- [Atlas Language Specification](specification/) - Language reference
- [CLI Reference](cli-reference.md) - Full CLI documentation
