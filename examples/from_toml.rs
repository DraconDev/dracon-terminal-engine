//! Example: loading app config from TOML
//!
//! Run with: cargo run --example from_toml
//!
//! This demonstrates loading a complete app configuration from a TOML file.
//! See `from_toml.toml` for the configuration format.

use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let toml_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/from_toml.toml".to_string());

    println!("Loading TOML config from: {}", toml_path);

    let content = std::fs::read_to_string(&toml_path)?;
    let config = dracon_terminal_engine::framework::command::AppConfig::from_toml_str(&content)?;

    println!(
        "Loaded config: {} with {} widgets",
        config.title,
        config.widgets.len()
    );

    // Show what we parsed
    for (i, widget) in config.widgets.iter().enumerate() {
        println!(
            "  widget[{}]: type={:?}, id={:?}, bind={:?}",
            i, widget.widget_type, widget.id, widget.bind
        );
    }

    // For demo purposes, just print the config rather than running
    // (full App::run requires a real TTY)
    println!("\nConfig parsed successfully!");
    println!("Note: Full app requires a real TTY. Use framework_demo for interactive use.");

    Ok(())
}
