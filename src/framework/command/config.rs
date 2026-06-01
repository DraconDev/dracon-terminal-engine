//! TOML app config: declarative widget/command layout loaded from disk.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::exec::BoundCommand;

// ═══════════════════════════════════════════════════════════════
// TOML APP CONFIG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub title: String,
    pub theme: Option<String>,
    pub fps: Option<u32>,
    #[serde(default)]
    pub layout: Option<LayoutConfig>,
    #[serde(default)]
    pub widgets: Vec<WidgetConfig>,
    #[serde(default)]
    pub commands: Vec<BoundCommand>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Dracon App".to_string(),
            theme: None,
            fps: None,
            layout: None,
            widgets: Vec::new(),
            commands: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WidgetConfig {
    #[serde(default)]
    pub id: Option<usize>,
    #[serde(default, rename = "type", alias = "type")]
    pub widget_type: Option<String>,
    #[serde(default)]
    pub area: Option<AreaConfig>,
    #[serde(default)]
    pub bind: Option<String>,
    #[serde(default)]
    pub parser: Option<ParserConfig>,
    #[serde(default)]
    pub refresh_seconds: Option<u64>,
    #[serde(default)]
    pub confirm: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutConfig {
    #[serde(default)]
    pub header_height: Option<u16>,
    #[serde(default)]
    pub sidebar_width: Option<u16>,
    #[serde(default)]
    pub footer_height: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaConfig {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    #[serde(rename = "type")]
    pub parser_type: String,
    pub key: Option<String>,
    pub path: Option<String>,
    pub item_key: Option<String>,
    pub pattern: Option<String>,
    pub group: Option<usize>,
    pub patterns: Option<HashMap<String, String>>,
}

impl AppConfig {
    pub fn from_toml(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn from_toml_str(content: &str) -> std::io::Result<Self> {
        toml::from_str(content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn load_user_config(name: &str) -> std::io::Result<Self> {
        let config_path = std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
            .join(".config")
            .join("dracon")
            .join(format!("{}.toml", name));
        if config_path.exists() {
            Self::from_toml(&config_path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("config not found at {:?}", config_path),
            ))
        }
    }

    /// Validates the configuration and returns warnings for unknown or invalid fields.
    ///
    /// This preserves forward compatibility by warning about unknown fields rather than failing.
    /// Warnings are collected and returned as a vector of strings.
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Validate theme name if specified
        if let Some(ref theme_name) = self.theme {
            if !Self::is_valid_theme(theme_name) {
                warnings.push(format!(
                    "unknown theme '{}'; falling back to default",
                    theme_name
                ));
            }
        }

        // Validate FPS range
        if let Some(fps) = self.fps {
            if fps == 0 {
                warnings.push("fps set to 0; using default (no limit)".to_string());
            } else if fps > 120 {
                warnings.push(format!(
                    "fps {} exceeds recommended max (120); capping",
                    fps
                ));
            }
        }

        // Validate widget IDs
        let mut seen_ids: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for widget in &self.widgets {
            if let Some(id) = widget.id {
                if !seen_ids.insert(id) {
                    warnings.push(format!(
                        "duplicate widget id {}; each widget should have a unique id",
                        id
                    ));
                }
            }
        }

        warnings
    }

    /// Checks if a theme name is valid.
    fn is_valid_theme(name: &str) -> bool {
        matches!(
            name.to_lowercase().as_str(),
            "default"
                | "nord"
                | "dracula"
                | "monokai"
                | "gruvbox"
                | "one-dark"
                | "catppuccin"
                | "tokyo-night"
                | "github-dark"
                | "solarized-dark"
                | "solarized-light"
                | "nord-light"
                | "nord-polar-night"
                | "nord-snow-storm"
                | "catppuccin-mocha"
                | "catppuccin-macchiato"
                | "catppuccin-frappe"
                | "catppuccin-latte"
                | "cyberpunk"
                | "synthwave"
                | "retro"
                | "matrix"
        )
    }
}
