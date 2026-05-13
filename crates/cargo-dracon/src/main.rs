//! cargo-dracon CLI tool for creating and managing Dracon Terminal Engine projects.
//!
//! Usage:
//!   cargo dracon new <name>     - Create a new Dracon project
//!   cargo dracon add <widget>   - Add a widget to dependencies
//!   cargo dracon init            - Initialize dracon.toml in existing project

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process;

/// Create a new Dracon Terminal Engine project.
fn create_project(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    // Check if directory already exists
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    println!("Creating new Dracon project: {}", name);

    // Create project directory
    fs::create_dir_all(project_dir).context("Failed to create project directory")?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A Dracon Terminal Engine application"

[dependencies]
dracon-terminal-engine = "0.1"

[profile.release]
opt-level = 3
lto = "thin"
"#,
        name = name
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)
        .context("Failed to write Cargo.toml")?;

    // Create src directory
    fs::create_dir_all(project_dir.join("src"))
        .context("Failed to create src directory")?;

    // Create main.rs based on template
    let main_rs = match template {
        "simple" => include_str!("templates/simple.rs"),
        "widget" => include_str!("templates/widget.rs"),
        "list" => include_str!("templates/list.rs"),
        _ => include_str!("templates/simple.rs"),
    };
    fs::write(project_dir.join("src/main.rs"), main_rs)
        .context("Failed to write main.rs")?;

    // Create dracon.toml
    let dracon_toml = r#"[app]
name = "My App"
fps = 30

[keybindings]
quit = "ctrl+q"
help = "f1"

[theme]
name = "nord"
"#;
    fs::write(project_dir.join("dracon.toml"), dracon_toml)
        .context("Failed to write dracon.toml")?;

    println!("Successfully created project '{}'", name);
    println!("\nTo get started:");
    println!("  cd {}", name);
    println!("  cargo run");

    Ok(())
}

/// Add a widget to the project dependencies.
fn add_widget(widget: &str) -> Result<()> {
    println!("Adding widget: {}", widget);

    // Find Cargo.toml
    let cargo_path = Path::new("Cargo.toml");
    if !cargo_path.exists() {
        anyhow::bail!("No Cargo.toml found in current directory");
    }

    // Read current Cargo.toml
    let content = fs::read_to_string(cargo_path)
        .context("Failed to read Cargo.toml")?;

    // Check if dracon-terminal-engine is already in dependencies
    if content.contains("dracon-terminal-engine") {
        println!("dracon-terminal-engine already in dependencies");
        println!("Widget '{}' is included in the base crate", widget);
    } else {
        // Add dracon-terminal-engine
        let new_content = content.replace(
            "[dependencies]",
            "[dependencies]\ndracon-terminal-engine = \"0.1\"",
        );
        fs::write(cargo_path, new_content)
            .context("Failed to update Cargo.toml")?;
        println!("Added dracon-terminal-engine to dependencies");
    }

    println!("Widget '{}' is ready to use!", widget);
    println!("\nIn your code:");
    println!("  use dracon_terminal_engine::prelude::*;");

    Ok(())
}

/// Initialize dracon.toml in existing project.
fn init_config() -> Result<()> {
    let config_path = Path::new("dracon.toml");

    if config_path.exists() {
        println!("dracon.toml already exists, skipping...");
        return Ok(());
    }

    println!("Initializing Dracon configuration...");

    let config = r#"[app]
name = "My Dracon App"
fps = 30

[keybindings]
quit = "ctrl+q"
help = "f1"
back = "esc"
theme = "ctrl+t"
search = "ctrl+f"
new = "ctrl+n"
close = "ctrl+w"
save = "ctrl+s"

[theme]
# Available themes: nord, dracula, gruvbox, solarized, monokai, cyberpunk,
# catppuccin_mocha, catppuccin_macchiato, catppuccin_frappe, catppuccin_latte,
# tokyo_night, tokyo_day, tokyo_storm, rose_pine, one_dark, github_dark,
# github_light, material_darker, material_palenight, nord_accent
name = "nord"

[window]
# Terminal window configuration
width = 80
height = 24

[logging]
# Enable structured logging (requires 'tracing' feature)
enabled = false
level = "info"
"#;

    fs::write(config_path, config).context("Failed to write dracon.toml")?;

    println!("Successfully created dracon.toml");
    println!("\nEdit dracon.toml to customize your application.");

    Ok(())
}

#[derive(Parser)]
#[command(
    name = "cargo-dracon",
    about = "CLI tool for creating and managing Dracon Terminal Engine projects",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Dracon project
    New {
        /// Name of the project to create
        name: String,

        /// Template to use (simple, widget, list)
        #[arg(short, long, default_value = "simple")]
        template: String,
    },
    /// Add a widget to dependencies
    Add {
        /// Name of the widget to add
        name: String,
    },
    /// Initialize dracon.toml in existing project
    Init,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New { name, template } => create_project(&name, &template),
        Commands::Add { name } => add_widget(&name),
        Commands::Init => init_config(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}