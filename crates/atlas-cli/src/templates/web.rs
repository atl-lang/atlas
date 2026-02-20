//! Web server project template.
//!
//! Creates a web server project with:
//! - HTTP server setup
//! - Routing structure
//! - Middleware examples
//! - Static file serving
//! - API endpoint structure
//! - Environment configuration

use super::Template;

/// Generate the web server project template.
pub fn template() -> Template {
    Template::builder("web")
        .description("A web server project with HTTP routing")
        // Directories
        .directory("src")
        .directory("src/routes")
        .directory("src/middleware")
        .directory("static")
        .directory("static/css")
        .directory("static/js")
        .directory("templates")
        .directory("tests")
        .directory("config")
        // Main source files
        .file("src/main.atl", MAIN_ATL)
        .file("src/server.atl", SERVER_ATL)
        .file("src/router.atl", ROUTER_ATL)
        // Routes
        .file("src/routes/mod.atl", ROUTES_MOD_ATL)
        .file("src/routes/api.atl", ROUTES_API_ATL)
        .file("src/routes/pages.atl", ROUTES_PAGES_ATL)
        // Middleware
        .file("src/middleware/mod.atl", MIDDLEWARE_MOD_ATL)
        .file("src/middleware/logger.atl", MIDDLEWARE_LOGGER_ATL)
        // Static files
        .file("static/css/style.css", STYLE_CSS)
        .file("static/js/app.js", APP_JS)
        // Templates
        .file("templates/index.html", INDEX_HTML)
        .file("templates/error.html", ERROR_HTML)
        // Configuration
        .file("config/default.toml", DEFAULT_CONFIG)
        .file(".env.example", ENV_EXAMPLE)
        // Tests
        .file("tests/server_test.atl", SERVER_TEST_ATL)
        // Project files
        .file("atlas.toml", ATLAS_TOML)
        .file("README.md", README_MD)
        .file("LICENSE", LICENSE_MIT)
        .file(".gitignore", GITIGNORE)
        .file("Dockerfile", DOCKERFILE)
        .build()
}

const ATLAS_TOML: &str = r#"[package]
name = "{{name}}"
version = "{{version}}"
description = "{{description}}"
authors = ["{{author}}"]
license = "MIT"
repository = ""
keywords = ["web", "server", "http"]
categories = ["web"]

[[bin]]
name = "{{name}}"
path = "src/main.atl"

[dependencies]
# Add dependencies here
# http = "1.0"

[dev-dependencies]
# Add dev dependencies here

[build]
profile = "release"

[server]
# Default server configuration
host = "127.0.0.1"
port = 8080
"#;

const MAIN_ATL: &str = r#"// {{name}} - {{description}}
//
// Web server entry point.

import { create_server, start_server } from "./server"
import { setup_routes } from "./routes/mod"
import { logger_middleware } from "./middleware/mod"

/// Application entry point.
fn main() {
    // Load configuration
    let config = load_config()

    // Create server instance
    let server = create_server(config)

    // Setup middleware
    server.use(logger_middleware)

    // Setup routes
    setup_routes(server)

    // Start the server
    print("Starting {{name}} server...")
    print("Listening on http://" + config.host + ":" + str(config.port))
    start_server(server)
}

/// Load server configuration.
fn load_config() {
    return {
        "host": env("HOST", "127.0.0.1"),
        "port": int(env("PORT", "8080")),
        "debug": env("DEBUG", "false") == "true",
        "static_dir": "static",
        "template_dir": "templates"
    }
}

/// Get environment variable with default.
fn env(name, default_value) {
    // Would be provided by Atlas runtime
    return default_value
}
"#;

const SERVER_ATL: &str = r#"// HTTP Server implementation for {{name}}

/// Server state and configuration.
let server_state = {
    "config": nil,
    "routes": [],
    "middleware": []
}

/// Create a new server instance.
///
/// @param config Server configuration
/// @returns Server instance
fn create_server(config) {
    let server = {
        "config": config,
        "routes": [],
        "middleware": []
    }
    return server
}

/// Register middleware.
///
/// @param middleware Middleware function
fn use(middleware) {
    server_state.middleware = push(server_state.middleware, middleware)
}

/// Register a route handler.
///
/// @param method HTTP method (GET, POST, etc.)
/// @param path URL path pattern
/// @param handler Request handler function
fn route(method, path, handler) {
    let r = {
        "method": method,
        "path": path,
        "handler": handler
    }
    server_state.routes = push(server_state.routes, r)
}

/// Handle incoming request.
///
/// @param request HTTP request object
/// @returns HTTP response object
fn handle_request(request) {
    // Run middleware chain
    let ctx = {
        "request": request,
        "response": nil,
        "next": true
    }

    let i = 0
    while i < len(server_state.middleware) and ctx.next {
        let mw = server_state.middleware[i]
        ctx = mw(ctx)
        i = i + 1
    }

    // Find matching route
    let j = 0
    while j < len(server_state.routes) {
        let r = server_state.routes[j]
        if r.method == request.method and match_path(r.path, request.path) {
            return r.handler(request)
        }
        j = j + 1
    }

    // No route found - 404
    return {
        "status": 404,
        "body": "Not Found",
        "headers": {"Content-Type": "text/plain"}
    }
}

/// Check if path matches route pattern.
fn match_path(pattern, path) {
    // Simple exact match for now
    // Future: support parameters like /users/:id
    return pattern == path
}

/// Start the HTTP server.
///
/// @param server Server instance
fn start_server(server) {
    server_state.config = server.config
    server_state.routes = server.routes
    server_state.middleware = server.middleware

    // Server main loop would go here
    // This is a placeholder - actual HTTP handling
    // would be provided by Atlas runtime
    print("Server started successfully")
}

/// Helper: Push item to array (immutable).
fn push(arr, item) {
    let result = []
    let i = 0
    while i < len(arr) {
        result = result + [arr[i]]
        i = i + 1
    }
    result = result + [item]
    return result
}

export { create_server, start_server, route, use, handle_request }
"#;

const ROUTER_ATL: &str = r#"// Router utilities for {{name}}

/// Create a router group with a prefix.
///
/// @param prefix URL prefix for all routes in this group
/// @returns Router group object
fn router(prefix) {
    return {
        "prefix": prefix,
        "routes": []
    }
}

/// Add GET route.
fn get(path, handler) {
    return {"method": "GET", "path": path, "handler": handler}
}

/// Add POST route.
fn post(path, handler) {
    return {"method": "POST", "path": path, "handler": handler}
}

/// Add PUT route.
fn put(path, handler) {
    return {"method": "PUT", "path": path, "handler": handler}
}

/// Add DELETE route.
fn delete_route(path, handler) {
    return {"method": "DELETE", "path": path, "handler": handler}
}

/// JSON response helper.
///
/// @param data Data to serialize to JSON
/// @param status HTTP status code (default 200)
/// @returns Response object
fn json(data, status) {
    if status == nil {
        status = 200
    }
    return {
        "status": status,
        "body": stringify(data),
        "headers": {"Content-Type": "application/json"}
    }
}

/// HTML response helper.
///
/// @param html HTML content
/// @param status HTTP status code (default 200)
/// @returns Response object
fn html(content, status) {
    if status == nil {
        status = 200
    }
    return {
        "status": status,
        "body": content,
        "headers": {"Content-Type": "text/html; charset=utf-8"}
    }
}

/// Redirect response helper.
///
/// @param url URL to redirect to
/// @param status HTTP status code (default 302)
/// @returns Response object
fn redirect(url, status) {
    if status == nil {
        status = 302
    }
    return {
        "status": status,
        "body": "",
        "headers": {"Location": url}
    }
}

/// Stringify object to JSON.
fn stringify(data) {
    // Placeholder - would be provided by runtime
    return str(data)
}

export { router, get, post, put, delete_route, json, html, redirect }
"#;

const ROUTES_MOD_ATL: &str = r#"// Route definitions for {{name}}

import { route } from "../server"
import { api_routes } from "./api"
import { page_routes } from "./pages"

/// Setup all application routes.
///
/// @param server Server instance
fn setup_routes(server) {
    // Register page routes
    page_routes(server)

    // Register API routes
    api_routes(server)
}

export { setup_routes }
"#;

const ROUTES_API_ATL: &str = r#"// API route handlers for {{name}}

import { route } from "../server"
import { json } from "../router"

/// Setup API routes.
///
/// @param server Server instance
fn api_routes(server) {
    route("GET", "/api/health", handle_health)
    route("GET", "/api/info", handle_info)
    route("GET", "/api/users", handle_users)
    route("POST", "/api/users", handle_create_user)
}

/// Health check endpoint.
fn handle_health(request) {
    return json({"status": "healthy", "uptime": 0}, nil)
}

/// Application info endpoint.
fn handle_info(request) {
    return json({
        "name": "{{name}}",
        "version": "{{version}}",
        "description": "{{description}}"
    }, nil)
}

/// List users endpoint.
fn handle_users(request) {
    // Example response
    let users = [
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ]
    return json({"users": users}, nil)
}

/// Create user endpoint.
fn handle_create_user(request) {
    // Would parse request.body and create user
    return json({"id": 3, "name": "New User"}, 201)
}

export { api_routes }
"#;

const ROUTES_PAGES_ATL: &str = r#"// Page route handlers for {{name}}

import { route } from "../server"
import { html, redirect } from "../router"

/// Setup page routes.
///
/// @param server Server instance
fn page_routes(server) {
    route("GET", "/", handle_home)
    route("GET", "/about", handle_about)
    route("GET", "/contact", handle_contact)
}

/// Home page handler.
fn handle_home(request) {
    let content = render_template("index.html", {
        "title": "{{name}}",
        "message": "Welcome to {{name}}!"
    })
    return html(content, nil)
}

/// About page handler.
fn handle_about(request) {
    let content = "<html><body><h1>About {{name}}</h1><p>{{description}}</p></body></html>"
    return html(content, nil)
}

/// Contact page handler.
fn handle_contact(request) {
    let content = "<html><body><h1>Contact</h1><p>Contact us at example@example.com</p></body></html>"
    return html(content, nil)
}

/// Render template with data.
fn render_template(name, data) {
    // Placeholder - would load and render template
    return "<html><body><h1>" + data.title + "</h1><p>" + data.message + "</p></body></html>"
}

export { page_routes }
"#;

const MIDDLEWARE_MOD_ATL: &str = r#"// Middleware definitions for {{name}}

import { logger } from "./logger"

/// Logger middleware.
let logger_middleware = logger

export { logger_middleware }
"#;

const MIDDLEWARE_LOGGER_ATL: &str = r#"// Request logging middleware for {{name}}

/// Logger middleware function.
///
/// @param ctx Request context
/// @returns Modified context
fn logger(ctx) {
    let request = ctx.request
    let start_time = now()

    // Log request
    print("[" + timestamp() + "] " + request.method + " " + request.path)

    // Continue to next middleware
    ctx.next = true
    return ctx
}

/// Get current timestamp string.
fn timestamp() {
    // Placeholder - would get actual timestamp
    return "2024-01-01 12:00:00"
}

/// Get current time in milliseconds.
fn now() {
    // Placeholder - would get actual time
    return 0
}

export { logger }
"#;

const STYLE_CSS: &str = r#"/* {{name}} - Main Stylesheet */

:root {
    --primary-color: #3498db;
    --secondary-color: #2ecc71;
    --text-color: #333;
    --bg-color: #fff;
    --border-color: #ddd;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    background-color: var(--bg-color);
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

header {
    background: var(--primary-color);
    color: white;
    padding: 1rem 0;
}

header h1 {
    font-size: 1.5rem;
}

nav {
    display: flex;
    gap: 1rem;
}

nav a {
    color: white;
    text-decoration: none;
}

nav a:hover {
    text-decoration: underline;
}

main {
    padding: 2rem 0;
}

.hero {
    text-align: center;
    padding: 4rem 0;
}

.hero h2 {
    font-size: 2.5rem;
    margin-bottom: 1rem;
}

.btn {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    background: var(--primary-color);
    color: white;
    text-decoration: none;
    border-radius: 4px;
    border: none;
    cursor: pointer;
}

.btn:hover {
    opacity: 0.9;
}

footer {
    border-top: 1px solid var(--border-color);
    padding: 2rem 0;
    text-align: center;
    color: #666;
}
"#;

const APP_JS: &str = r#"// {{name}} - Main JavaScript

document.addEventListener('DOMContentLoaded', function() {
    console.log('{{name}} loaded');

    // Example: Fetch API data
    async function fetchHealth() {
        try {
            const response = await fetch('/api/health');
            const data = await response.json();
            console.log('Health check:', data);
        } catch (error) {
            console.error('Health check failed:', error);
        }
    }

    // Run health check
    fetchHealth();
});
"#;

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{name}}</title>
    <link rel="stylesheet" href="/static/css/style.css">
</head>
<body>
    <header>
        <div class="container">
            <h1>{{name}}</h1>
            <nav>
                <a href="/">Home</a>
                <a href="/about">About</a>
                <a href="/contact">Contact</a>
            </nav>
        </div>
    </header>

    <main>
        <div class="container">
            <section class="hero">
                <h2>Welcome to {{name}}</h2>
                <p>{{description}}</p>
                <a href="/api/info" class="btn">API Info</a>
            </section>
        </div>
    </main>

    <footer>
        <div class="container">
            <p>&copy; {{year}} {{author}}. All rights reserved.</p>
        </div>
    </footer>

    <script src="/static/js/app.js"></script>
</body>
</html>
"#;

const ERROR_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Error - {{name}}</title>
    <link rel="stylesheet" href="/static/css/style.css">
</head>
<body>
    <header>
        <div class="container">
            <h1>{{name}}</h1>
        </div>
    </header>

    <main>
        <div class="container">
            <section class="hero">
                <h2>Error</h2>
                <p>Something went wrong. Please try again later.</p>
                <a href="/" class="btn">Go Home</a>
            </section>
        </div>
    </main>

    <footer>
        <div class="container">
            <p>&copy; {{year}} {{author}}. All rights reserved.</p>
        </div>
    </footer>
</body>
</html>
"#;

const DEFAULT_CONFIG: &str = r#"# {{name}} Server Configuration

[server]
host = "127.0.0.1"
port = 8080

[logging]
level = "info"
format = "json"

[static]
# Static file directory
dir = "static"
# Cache duration in seconds
cache = 3600

[templates]
# Template directory
dir = "templates"
# Enable template caching
cache = true
"#;

const ENV_EXAMPLE: &str = r#"# {{name}} Environment Variables
# Copy this file to .env and customize

# Server settings
HOST=127.0.0.1
PORT=8080

# Debug mode (true/false)
DEBUG=false

# Logging level (debug, info, warn, error)
LOG_LEVEL=info

# Database URL (if applicable)
# DATABASE_URL=postgres://user:pass@localhost/db

# Secret key for sessions (generate a random string)
# SECRET_KEY=your-secret-key-here
"#;

const SERVER_TEST_ATL: &str = r#"// Server tests for {{name}}

import { create_server, handle_request } from "../src/server"
import { json, html, redirect } from "../src/router"

// Router helper tests

fn test_json_response() {
    let resp = json({"status": "ok"}, nil)
    assert(resp.status == 200)
    assert(resp.headers["Content-Type"] == "application/json")
    return true
}

fn test_json_response_custom_status() {
    let resp = json({"error": "not found"}, 404)
    assert(resp.status == 404)
    return true
}

fn test_html_response() {
    let resp = html("<h1>Hello</h1>", nil)
    assert(resp.status == 200)
    assert(resp.headers["Content-Type"] == "text/html; charset=utf-8")
    return true
}

fn test_redirect_response() {
    let resp = redirect("/new-location", nil)
    assert(resp.status == 302)
    assert(resp.headers["Location"] == "/new-location")
    return true
}

fn test_redirect_custom_status() {
    let resp = redirect("/permanent", 301)
    assert(resp.status == 301)
    return true
}

// Server tests

fn test_create_server() {
    let config = {"host": "localhost", "port": 3000}
    let server = create_server(config)
    assert(server != nil)
    assert(server.config.port == 3000)
    return true
}

fn test_handle_request_404() {
    // Without registered routes, should return 404
    let request = {"method": "GET", "path": "/nonexistent"}
    let response = handle_request(request)
    assert(response.status == 404)
    return true
}

export {
    test_json_response,
    test_json_response_custom_status,
    test_html_response,
    test_redirect_response,
    test_redirect_custom_status,
    test_create_server,
    test_handle_request_404
}
"#;

const README_MD: &str = r#"# {{name}}

{{description}}

## Quick Start

```bash
# Start the development server
atlas run src/main.atl

# The server will be available at http://localhost:8080
```

## Project Structure

```
{{name}}/
├── src/
│   ├── main.atl          # Application entry point
│   ├── server.atl        # HTTP server implementation
│   ├── router.atl        # Routing utilities
│   ├── routes/
│   │   ├── mod.atl       # Route setup
│   │   ├── api.atl       # API endpoints
│   │   └── pages.atl     # Page handlers
│   └── middleware/
│       ├── mod.atl       # Middleware exports
│       └── logger.atl    # Request logging
├── static/
│   ├── css/style.css     # Stylesheet
│   └── js/app.js         # Client-side JavaScript
├── templates/
│   ├── index.html        # Home page template
│   └── error.html        # Error page template
├── config/
│   └── default.toml      # Default configuration
├── tests/
│   └── server_test.atl   # Server tests
├── atlas.toml            # Project manifest
└── README.md
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/info` | GET | Application info |
| `/api/users` | GET | List users |
| `/api/users` | POST | Create user |

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and customize:

```bash
cp .env.example .env
```

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | 127.0.0.1 | Server bind address |
| `PORT` | 8080 | Server port |
| `DEBUG` | false | Enable debug mode |
| `LOG_LEVEL` | info | Logging level |

### Config File

Server configuration in `config/default.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080

[logging]
level = "info"
format = "json"
```

## Development

### Running Tests

```bash
atlas test
```

### Building for Production

```bash
atlas build --release
```

### Docker

```bash
# Build image
docker build -t {{name}} .

# Run container
docker run -p 8080:8080 {{name}}
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

# Environment files
.env
.env.local
.env.*.local

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

# Node modules (if using any JS tooling)
node_modules/
"#;

const DOCKERFILE: &str = r#"# {{name}} Dockerfile
FROM ubuntu:22.04 AS builder

# Install Atlas compiler (placeholder)
# RUN curl -sSf https://atlas-lang.org/install.sh | sh

WORKDIR /app
COPY . .

# Build the application
# RUN atlas build --release

FROM ubuntu:22.04

WORKDIR /app

# Copy built application
COPY --from=builder /app/target/release/{{name}} /app/
COPY --from=builder /app/static /app/static
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/config /app/config

# Expose port
EXPOSE 8080

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=8080

# Run the application
CMD ["./{{name}}"]
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::TemplateContext;

    #[test]
    fn test_web_template_structure() {
        let tmpl = template();
        assert_eq!(tmpl.name, "web");

        // Check required directories
        let dir_names: Vec<_> = tmpl.directories.iter().map(|d| &d.path).collect();
        assert!(dir_names.iter().any(|p| p.to_str() == Some("src")));
        assert!(dir_names.iter().any(|p| p.to_str() == Some("src/routes")));
        assert!(dir_names.iter().any(|p| p.to_str() == Some("static")));
        assert!(dir_names.iter().any(|p| p.to_str() == Some("templates")));
    }

    #[test]
    fn test_web_template_files() {
        let tmpl = template();

        let file_names: Vec<_> = tmpl.files.iter().map(|f| &f.path).collect();
        assert!(file_names
            .iter()
            .any(|p| p.to_str() == Some("src/main.atl")));
        assert!(file_names
            .iter()
            .any(|p| p.to_str() == Some("src/server.atl")));
        assert!(file_names.iter().any(|p| p.to_str() == Some("atlas.toml")));
        assert!(file_names.iter().any(|p| p.to_str() == Some("Dockerfile")));
    }

    #[test]
    fn test_web_template_render() {
        let tmpl = template();
        let ctx = TemplateContext::for_project("my-server", "Test Author", "A web server");
        let files = tmpl.render(&ctx);

        // Find atlas.toml and check substitution
        let atlas_toml = files
            .iter()
            .find(|(p, _, _)| p.to_str() == Some("atlas.toml"));
        assert!(atlas_toml.is_some());

        let content = &atlas_toml.unwrap().1;
        assert!(content.contains("name = \"my-server\""));
        assert!(content.contains("[server]"));
    }

    #[test]
    fn test_web_has_docker_support() {
        let tmpl = template();
        let has_dockerfile = tmpl
            .files
            .iter()
            .any(|f| f.path.to_str() == Some("Dockerfile"));
        assert!(has_dockerfile);
    }

    #[test]
    fn test_web_has_static_files() {
        let tmpl = template();
        let has_css = tmpl
            .files
            .iter()
            .any(|f| f.path.to_str() == Some("static/css/style.css"));
        let has_js = tmpl
            .files
            .iter()
            .any(|f| f.path.to_str() == Some("static/js/app.js"));
        assert!(has_css);
        assert!(has_js);
    }
}
