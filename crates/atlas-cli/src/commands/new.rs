//! Project creation command (atlas new)
//!
//! Creates a new Atlas project from a template in a new directory.

use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::templates::{
    git_initial_commit, init_git, validate_name, TemplateContext, TemplateType,
};

/// Arguments for the new command.
#[derive(Debug, Clone)]
pub struct NewArgs {
    /// Project name (required).
    pub name: String,
    /// Template type to use.
    pub template: TemplateType,
    /// Author name for project.
    pub author: Option<String>,
    /// Project description.
    pub description: Option<String>,
    /// Initialize git repository.
    pub git: bool,
    /// Create initial commit.
    pub commit: bool,
    /// Force creation even if directory exists.
    pub force: bool,
    /// Parent directory for the project.
    pub path: PathBuf,
    /// Skip interactive prompts.
    pub non_interactive: bool,
    /// Verbose output.
    pub verbose: bool,
}

impl Default for NewArgs {
    fn default() -> Self {
        Self {
            name: String::new(),
            template: TemplateType::Binary,
            author: None,
            description: None,
            git: true,
            commit: true,
            force: false,
            path: PathBuf::from("."),
            non_interactive: false,
            verbose: false,
        }
    }
}

/// Run the new command.
pub fn run(args: NewArgs) -> Result<()> {
    // Validate project name
    validate_name(&args.name)?;

    // Calculate project directory
    let project_dir = args.path.join(&args.name);

    // Check if directory exists
    if project_dir.exists() {
        if args.force {
            // Remove existing directory
            fs::remove_dir_all(&project_dir).context("Failed to remove existing directory")?;
            if args.verbose {
                println!("Removed existing directory: {}", project_dir.display());
            }
        } else {
            let entries: Vec<_> = fs::read_dir(&project_dir)
                .context("Failed to read target directory")?
                .filter_map(|e| e.ok())
                .collect();
            if !entries.is_empty() {
                bail!(
                    "Directory '{}' already exists and is not empty. Use --force to overwrite.",
                    project_dir.display()
                );
            }
        }
    }

    // Get author name
    let author = if let Some(ref a) = args.author {
        a.clone()
    } else if args.non_interactive {
        get_git_user_name().unwrap_or_else(|| "Unknown Author".to_string())
    } else {
        let default = get_git_user_name().unwrap_or_default();
        prompt_for_value("Author", &default)?
    };

    // Get description
    let description = if let Some(ref d) = args.description {
        d.clone()
    } else if args.non_interactive {
        format!("A {} Atlas project", args.template.name())
    } else {
        let default = format!("A {} Atlas project", args.template.name());
        prompt_for_value("Description", &default)?
    };

    // Create template context
    let ctx = TemplateContext::for_project(&args.name, &author, &description);

    // Get the template
    let template = args.template.template();

    if args.verbose {
        println!("Creating {} project: {}", args.template.name(), args.name);
        println!("Template: {}", template.description);
        println!("Directory: {}", project_dir.display());
    }

    // Generate the project
    template
        .generate(&project_dir, &ctx, args.verbose)
        .context("Failed to generate project")?;

    // Initialize git if requested
    if args.git {
        let git_ok = init_git(&project_dir, args.verbose)?;

        // Create initial commit if requested and git init succeeded
        if git_ok && args.commit {
            git_initial_commit(&project_dir, args.verbose)?;
        }
    }

    // Print success message
    println!();
    println!(
        "{} Created {} project '{}'",
        green_check(),
        args.template.name(),
        args.name
    );
    println!("  Path: {}", project_dir.display());
    println!();
    println!("To get started:");
    println!("  cd {}", args.name);

    match args.template {
        TemplateType::Binary => {
            println!("  atlas run src/main.atl");
        }
        TemplateType::Library => {
            println!("  atlas test");
            println!("  atlas run examples/basic.atl");
        }
        TemplateType::Web => {
            println!("  atlas run src/main.atl");
            println!("  # Server starts at http://localhost:8080");
        }
    }

    Ok(())
}

/// Prompt user for a value with a default.
fn prompt_for_value(prompt: &str, default: &str) -> Result<String> {
    if default.is_empty() {
        print!("{}: ", prompt);
    } else {
        print!("{} [{}]: ", prompt, default);
    }
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

/// Get the git user.name from global config.
fn get_git_user_name() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["config", "--global", "user.name"])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

/// Green checkmark for success messages.
fn green_check() -> &'static str {
    "\u{2713}"
}

/// List available templates.
pub fn list_templates() {
    println!("Available templates:");
    println!();
    for template_type in TemplateType::all() {
        println!(
            "  {} - {}",
            template_type.name(),
            template_type.description()
        );
    }
    println!();
    println!("Use: atlas new <name> --template <type>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_binary_project() {
        let temp = TempDir::new().unwrap();

        let args = NewArgs {
            name: "test-app".to_string(),
            template: TemplateType::Binary,
            author: Some("Test Author".to_string()),
            description: Some("Test description".to_string()),
            git: false,
            commit: false,
            force: false,
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        run(args).unwrap();

        let project_dir = temp.path().join("test-app");
        assert!(project_dir.exists());
        assert!(project_dir.join("atlas.toml").exists());
        assert!(project_dir.join("src/main.atl").exists());
        assert!(project_dir.join("src/cli.atl").exists());
        assert!(project_dir.join("README.md").exists());
    }

    #[test]
    fn test_new_library_project() {
        let temp = TempDir::new().unwrap();

        let args = NewArgs {
            name: "test-lib".to_string(),
            template: TemplateType::Library,
            author: Some("Test Author".to_string()),
            description: Some("Test library".to_string()),
            git: false,
            commit: false,
            force: false,
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        run(args).unwrap();

        let project_dir = temp.path().join("test-lib");
        assert!(project_dir.exists());
        assert!(project_dir.join("atlas.toml").exists());
        assert!(project_dir.join("src/lib.atl").exists());
        assert!(project_dir.join("tests/lib_test.atl").exists());
        assert!(project_dir.join("examples/basic.atl").exists());
        assert!(project_dir.join("LICENSE").exists());
    }

    #[test]
    fn test_new_web_project() {
        let temp = TempDir::new().unwrap();

        let args = NewArgs {
            name: "test-web".to_string(),
            template: TemplateType::Web,
            author: Some("Test Author".to_string()),
            description: Some("Test web server".to_string()),
            git: false,
            commit: false,
            force: false,
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        run(args).unwrap();

        let project_dir = temp.path().join("test-web");
        assert!(project_dir.exists());
        assert!(project_dir.join("atlas.toml").exists());
        assert!(project_dir.join("src/main.atl").exists());
        assert!(project_dir.join("src/server.atl").exists());
        assert!(project_dir.join("src/routes/api.atl").exists());
        assert!(project_dir.join("static/css/style.css").exists());
        assert!(project_dir.join("Dockerfile").exists());
    }

    #[test]
    fn test_new_fails_existing_directory() {
        let temp = TempDir::new().unwrap();

        // Create existing directory with content
        let existing = temp.path().join("existing");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("file.txt"), "content").unwrap();

        let args = NewArgs {
            name: "existing".to_string(),
            template: TemplateType::Binary,
            author: Some("Test".to_string()),
            description: Some("Test".to_string()),
            git: false,
            commit: false,
            force: false,
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn test_new_force_overwrites() {
        let temp = TempDir::new().unwrap();

        // Create existing directory with content
        let existing = temp.path().join("existing");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("old-file.txt"), "old content").unwrap();

        let args = NewArgs {
            name: "existing".to_string(),
            template: TemplateType::Binary,
            author: Some("Test".to_string()),
            description: Some("Test".to_string()),
            git: false,
            commit: false,
            force: true,
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        run(args).unwrap();

        // Old file should be gone, new files should exist
        assert!(!existing.join("old-file.txt").exists());
        assert!(existing.join("atlas.toml").exists());
    }

    #[test]
    fn test_new_invalid_name() {
        let temp = TempDir::new().unwrap();

        let args = NewArgs {
            name: "-invalid".to_string(),
            template: TemplateType::Binary,
            author: Some("Test".to_string()),
            description: Some("Test".to_string()),
            git: false,
            commit: false,
            force: false,
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn test_new_reserved_name() {
        let temp = TempDir::new().unwrap();

        let args = NewArgs {
            name: "atlas".to_string(),
            template: TemplateType::Binary,
            author: Some("Test".to_string()),
            description: Some("Test".to_string()),
            git: false,
            commit: false,
            force: false,
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn test_variable_substitution_in_files() {
        let temp = TempDir::new().unwrap();

        let args = NewArgs {
            name: "my-awesome-app".to_string(),
            template: TemplateType::Binary,
            author: Some("Jane Doe".to_string()),
            description: Some("An awesome application".to_string()),
            git: false,
            commit: false,
            force: false,
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        run(args).unwrap();

        let project_dir = temp.path().join("my-awesome-app");

        // Check atlas.toml has correct substitutions
        let manifest = fs::read_to_string(project_dir.join("atlas.toml")).unwrap();
        assert!(manifest.contains("name = \"my-awesome-app\""));
        assert!(manifest.contains("Jane Doe"));
        assert!(manifest.contains("An awesome application"));

        // Check README
        let readme = fs::read_to_string(project_dir.join("README.md")).unwrap();
        assert!(readme.contains("my-awesome-app"));
    }

    #[test]
    fn test_template_type_all() {
        let all = TemplateType::all();
        assert!(all.len() >= 3);
        assert!(all.contains(&TemplateType::Binary));
        assert!(all.contains(&TemplateType::Library));
        assert!(all.contains(&TemplateType::Web));
    }
}
