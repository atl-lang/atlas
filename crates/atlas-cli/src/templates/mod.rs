//! Template system for Atlas project scaffolding.
//!
//! This module provides a template engine for generating project files
//! with variable substitution and template rendering.

// Allow unused items that are part of the public API
#![allow(dead_code)]

pub mod binary;
pub mod library;
pub mod web;

use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Template variable context for substitution.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    /// Variables for substitution (e.g., "name" -> "my-project")
    pub variables: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new empty template context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable in the context.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Get a variable from the context.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Create context with standard project variables.
    pub fn for_project(name: &str, author: &str, description: &str) -> Self {
        let mut ctx = Self::new();
        ctx.set("name", name)
            .set("name_snake", to_snake_case(name))
            .set("name_pascal", to_pascal_case(name))
            .set("author", author)
            .set("description", description)
            .set("year", chrono::Local::now().format("%Y").to_string())
            .set("version", "0.1.0");
        ctx
    }
}

/// A single file to be generated from a template.
#[derive(Debug, Clone)]
pub struct TemplateFile {
    /// Relative path from project root.
    pub path: PathBuf,
    /// File content with placeholders.
    pub content: String,
    /// Whether to make the file executable (Unix only).
    pub executable: bool,
}

impl TemplateFile {
    /// Create a new template file.
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
            executable: false,
        }
    }

    /// Set the executable flag.
    pub fn executable(mut self, value: bool) -> Self {
        self.executable = value;
        self
    }

    /// Render the file content with the given context.
    pub fn render(&self, ctx: &TemplateContext) -> String {
        substitute_variables(&self.content, ctx)
    }
}

/// A directory to be created.
#[derive(Debug, Clone)]
pub struct TemplateDir {
    /// Relative path from project root.
    pub path: PathBuf,
}

impl TemplateDir {
    /// Create a new template directory.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

/// A complete project template.
#[derive(Debug, Clone)]
pub struct Template {
    /// Template name (e.g., "binary", "library", "web").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Directories to create.
    pub directories: Vec<TemplateDir>,
    /// Files to generate.
    pub files: Vec<TemplateFile>,
}

impl Template {
    /// Create a new template builder.
    pub fn builder(name: impl Into<String>) -> TemplateBuilder {
        TemplateBuilder::new(name)
    }

    /// Render all files with the given context.
    pub fn render(&self, ctx: &TemplateContext) -> Vec<(PathBuf, String, bool)> {
        self.files
            .iter()
            .map(|f| (f.path.clone(), f.render(ctx), f.executable))
            .collect()
    }

    /// Generate the project in the given directory.
    pub fn generate(&self, root: &Path, ctx: &TemplateContext, verbose: bool) -> Result<()> {
        // Validate target directory
        if root.exists() {
            let entries: Vec<_> = fs::read_dir(root)
                .context("Failed to read target directory")?
                .filter_map(|e| e.ok())
                .collect();
            if !entries.is_empty() {
                bail!(
                    "Directory '{}' is not empty. Use --force to overwrite.",
                    root.display()
                );
            }
        }

        // Create root directory
        fs::create_dir_all(root).context("Failed to create project directory")?;

        // Create subdirectories
        for dir in &self.directories {
            let dir_path = root.join(&dir.path);
            fs::create_dir_all(&dir_path)
                .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
            if verbose {
                println!("  Created directory: {}", dir.path.display());
            }
        }

        // Generate files
        for file in &self.files {
            let file_path = root.join(&file.path);

            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create parent directory: {}", parent.display())
                })?;
            }

            // Render and write content
            let content = file.render(ctx);
            fs::write(&file_path, &content)
                .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

            // Set executable permission on Unix
            #[cfg(unix)]
            if file.executable {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&file_path)?.permissions();
                perms.set_mode(perms.mode() | 0o111);
                fs::set_permissions(&file_path, perms)?;
            }

            if verbose {
                let exe_marker = if file.executable { " (executable)" } else { "" };
                println!("  Created: {}{}", file.path.display(), exe_marker);
            }
        }

        Ok(())
    }
}

/// Builder for constructing templates.
#[derive(Debug, Clone)]
pub struct TemplateBuilder {
    name: String,
    description: String,
    directories: Vec<TemplateDir>,
    files: Vec<TemplateFile>,
}

impl TemplateBuilder {
    /// Create a new template builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            directories: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Set the template description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a directory to create.
    pub fn directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.directories.push(TemplateDir::new(path));
        self
    }

    /// Add a file to generate.
    pub fn file(mut self, path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        self.files.push(TemplateFile::new(path, content));
        self
    }

    /// Add an executable file to generate.
    pub fn executable_file(mut self, path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        self.files
            .push(TemplateFile::new(path, content).executable(true));
        self
    }

    /// Build the template.
    pub fn build(self) -> Template {
        Template {
            name: self.name,
            description: self.description,
            directories: self.directories,
            files: self.files,
        }
    }
}

/// Available template types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    /// Binary executable project.
    Binary,
    /// Library project.
    Library,
    /// Web server project.
    Web,
}

impl TemplateType {
    /// Get the template name.
    pub fn name(&self) -> &'static str {
        match self {
            TemplateType::Binary => "binary",
            TemplateType::Library => "library",
            TemplateType::Web => "web",
        }
    }

    /// Get the template description.
    pub fn description(&self) -> &'static str {
        match self {
            TemplateType::Binary => "A binary executable project with CLI support",
            TemplateType::Library => "A library project with documentation and tests",
            TemplateType::Web => "A web server project with HTTP routing",
        }
    }

    /// Get the corresponding template.
    pub fn template(&self) -> Template {
        match self {
            TemplateType::Binary => binary::template(),
            TemplateType::Library => library::template(),
            TemplateType::Web => web::template(),
        }
    }

    /// List all available template types.
    pub fn all() -> &'static [TemplateType] {
        &[
            TemplateType::Binary,
            TemplateType::Library,
            TemplateType::Web,
        ]
    }
}

impl std::str::FromStr for TemplateType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bin" | "binary" => Ok(TemplateType::Binary),
            "lib" | "library" => Ok(TemplateType::Library),
            "web" | "server" => Ok(TemplateType::Web),
            _ => Err(format!(
                "Unknown template type: '{}'. Available: binary, library, web",
                s
            )),
        }
    }
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Substitute template variables in content.
///
/// Variables are specified as `{{variable_name}}` in the content.
pub fn substitute_variables(content: &str, ctx: &TemplateContext) -> String {
    let mut result = content.to_string();

    for (key, value) in &ctx.variables {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }

    result
}

/// Convert a string to snake_case.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if *c == '-' || *c == '_' {
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
        } else if c.is_uppercase() {
            // Add underscore before uppercase if:
            // 1. Not at the start
            // 2. Previous char was lowercase, OR
            // 3. Next char exists and is lowercase (end of acronym)
            let prev_lowercase = i > 0 && chars[i - 1].is_lowercase();
            let next_lowercase = i + 1 < chars.len() && chars[i + 1].is_lowercase();

            if i > 0 && !result.ends_with('_') && (prev_lowercase || next_lowercase) {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(*c);
        }
    }

    // Clean up double underscores
    while result.contains("__") {
        result = result.replace("__", "_");
    }

    result.trim_matches('_').to_string()
}

/// Convert a string to PascalCase.
pub fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Validate a package/project name.
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Project name cannot be empty");
    }

    if name.len() > 64 {
        bail!("Project name must be 64 characters or less");
    }

    // First character must be alphanumeric
    if !name
        .chars()
        .next()
        .map(|c| c.is_alphabetic())
        .unwrap_or(false)
    {
        bail!("Project name must start with a letter");
    }

    // Only alphanumeric, hyphen, and underscore allowed
    for c in name.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' {
            bail!("Project name can only contain letters, numbers, hyphens, and underscores");
        }
    }

    // Reserved names
    let reserved = [
        "atlas", "std", "core", "test", "debug", "self", "super", "crate",
    ];
    if reserved.contains(&name.to_lowercase().as_str()) {
        bail!("'{}' is a reserved name and cannot be used", name);
    }

    Ok(())
}

/// Initialize git repository in the given directory.
pub fn init_git(path: &Path, verbose: bool) -> Result<bool> {
    // Check if already a git repo
    if path.join(".git").exists() {
        if verbose {
            println!("  Git repository already exists");
        }
        return Ok(true);
    }

    // Try to initialize git
    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(path)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            if verbose {
                println!("  Initialized git repository");
            }
            Ok(true)
        }
        Ok(_) => {
            if verbose {
                println!("  Warning: Failed to initialize git repository");
            }
            Ok(false)
        }
        Err(_) => {
            if verbose {
                println!("  Note: git not found, skipping repository initialization");
            }
            Ok(false)
        }
    }
}

/// Create an initial git commit.
pub fn git_initial_commit(path: &Path, verbose: bool) -> Result<bool> {
    // Add all files
    let add_output = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output();

    if !add_output.map(|o| o.status.success()).unwrap_or(false) {
        if verbose {
            println!("  Warning: Failed to stage files for git commit");
        }
        return Ok(false);
    }

    // Create initial commit
    let commit_output = std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output();

    match commit_output {
        Ok(out) if out.status.success() => {
            if verbose {
                println!("  Created initial commit");
            }
            Ok(true)
        }
        _ => {
            if verbose {
                println!("  Warning: Failed to create initial commit");
            }
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("my-project"), "my_project");
        assert_eq!(to_snake_case("MyProject"), "my_project");
        assert_eq!(to_snake_case("my_project"), "my_project");
        assert_eq!(to_snake_case("myProject"), "my_project");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my-project"), "MyProject");
        assert_eq!(to_pascal_case("my_project"), "MyProject");
        assert_eq!(to_pascal_case("myproject"), "Myproject");
        assert_eq!(to_pascal_case("http-server"), "HttpServer");
    }

    #[test]
    fn test_substitute_variables() {
        let mut ctx = TemplateContext::new();
        ctx.set("name", "test-project");
        ctx.set("version", "1.0.0");

        let content = "Project: {{name}}, Version: {{version}}";
        let result = substitute_variables(content, &ctx);
        assert_eq!(result, "Project: test-project, Version: 1.0.0");
    }

    #[test]
    fn test_substitute_variables_missing() {
        let ctx = TemplateContext::new();
        let content = "Project: {{name}}";
        let result = substitute_variables(content, &ctx);
        assert_eq!(result, "Project: {{name}}");
    }

    #[test]
    fn test_template_context_for_project() {
        let ctx = TemplateContext::for_project("my-app", "Test Author", "A test project");
        assert_eq!(ctx.get("name"), Some(&"my-app".to_string()));
        assert_eq!(ctx.get("name_snake"), Some(&"my_app".to_string()));
        assert_eq!(ctx.get("name_pascal"), Some(&"MyApp".to_string()));
        assert_eq!(ctx.get("author"), Some(&"Test Author".to_string()));
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("my-project").is_ok());
        assert!(validate_name("my_project").is_ok());
        assert!(validate_name("project123").is_ok());
        assert!(validate_name("a").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_name("").is_err());
        assert!(validate_name("-invalid").is_err());
        assert!(validate_name("123start").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name("has.dot").is_err());
    }

    #[test]
    fn test_validate_name_reserved() {
        assert!(validate_name("atlas").is_err());
        assert!(validate_name("std").is_err());
        assert!(validate_name("core").is_err());
        assert!(validate_name("self").is_err());
    }

    #[test]
    fn test_template_type_from_str() {
        assert_eq!("bin".parse::<TemplateType>().unwrap(), TemplateType::Binary);
        assert_eq!(
            "binary".parse::<TemplateType>().unwrap(),
            TemplateType::Binary
        );
        assert_eq!(
            "lib".parse::<TemplateType>().unwrap(),
            TemplateType::Library
        );
        assert_eq!(
            "library".parse::<TemplateType>().unwrap(),
            TemplateType::Library
        );
        assert_eq!("web".parse::<TemplateType>().unwrap(), TemplateType::Web);
        assert!("invalid".parse::<TemplateType>().is_err());
    }

    #[test]
    fn test_template_builder() {
        let template = Template::builder("test")
            .description("Test template")
            .directory("src")
            .file("README.md", "# {{name}}")
            .build();

        assert_eq!(template.name, "test");
        assert_eq!(template.description, "Test template");
        assert_eq!(template.directories.len(), 1);
        assert_eq!(template.files.len(), 1);
    }

    #[test]
    fn test_template_render() {
        let template = Template::builder("test")
            .file("README.md", "# {{name}}\n\n{{description}}")
            .file("src/main.atl", "// {{name}}")
            .build();

        let ctx = TemplateContext::for_project("my-app", "Author", "Description");
        let files = template.render(&ctx);

        assert_eq!(files.len(), 2);
        assert!(files[0].1.contains("# my-app"));
        assert!(files[1].1.contains("// my-app"));
    }
}
