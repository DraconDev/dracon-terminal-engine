//! Color themes for terminal UI.
//!
//! Provides [`crate::framework::theme::Theme`] with 21 built-in themes: `dark`, `light`, `high_contrast`,
//! `cyberpunk`, `dracula`, `nord`, `catppuccin_mocha`, `gruvbox_dark`, `tokyo_night`,
//! `solarized_dark`, `solarized_light`, `one_dark`, `rose_pine`, `kanagawa`,
//! `everforest`, `monokai`, `warm`, `cool`, `forest`, `sunset`, `mono`.
//!
//! Apply a theme with [`App::set_theme`](crate::framework::app::App::set_theme),
//! which propagates it to all registered widgets via `on_theme_change`.

use crate::compositor::Color;

/// Whether this is a dark or light theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeKind {
    Dark,
    #[default]
    Light,
}

/// A color scheme defining the visual appearance of the terminal UI.
///
/// This uses a semantic color system inspired by Material Design, adapted for terminal UI.
/// Colors are organized by purpose: surfaces (elevation), text (hierarchy), interactive (states),
/// semantic (feedback), and specialized (selection, input, scrollbar).
///
/// # Surface/Elevation System
/// - `bg` — Root viewport background
/// - `surface` — Panel/card surface (slightly elevated from bg)
/// - `surface_elevated` — Dropdowns, dialogs (highest surface)
///
/// # Text Hierarchy
/// - `fg` — Primary text
/// - `fg_muted` — Secondary text (labels, descriptions)
/// - `fg_subtle` — Tertiary text (placeholders, hints)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub name: &'static str,
    pub kind: ThemeKind,

    // Surface / Elevation
    pub bg: Color,
    pub surface: Color,
    pub surface_elevated: Color,

    // Text hierarchy
    pub fg: Color,
    pub fg_muted: Color,
    pub fg_subtle: Color,
    pub fg_on_accent: Color,

    // Interactive / Primary action
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,

    // Secondary interactive
    pub secondary: Color,
    pub secondary_hover: Color,
    pub secondary_active: Color,

    // Borders and dividers
    pub outline: Color,
    pub outline_variant: Color,
    pub divider: Color,

    // Semantic states
    pub error: Color,
    pub error_bg: Color,
    pub success: Color,
    pub success_bg: Color,
    pub warning: Color,
    pub warning_bg: Color,
    pub info: Color,
    pub info_bg: Color,

    // Selection
    pub selection_bg: Color,
    pub selection_fg: Color,

    // Input fields
    pub input_bg: Color,
    pub input_fg: Color,
    pub input_border: Color,

    // Scrollbar
    pub scrollbar_track: Color,
    pub scrollbar_thumb: Color,
    pub scrollbar_thumb_hover: Color,

    // Disabled
    pub disabled_fg: Color,
    pub disabled_bg: Color,

    /// Background color for hovered interactive items (rows, list items, nodes).
    /// Used by `Table`, `List`, `Tree`, and `CommandPalette` widgets.
    pub hover_bg: Color,

    /// Background color for focused elements (e.g., input fields with keyboard focus).
    /// Available for custom widgets that track focus state.
    pub focus_bg: Color,

    /// Border color for focused widgets or sections.
    /// Available for custom widgets that track focus state.
    pub focus_border: Color,

    // Scrollbar width
    pub scrollbar_width: u16,
}

impl Theme {
    /// Creates a dark theme with muted colors suitable for low-light environments.
    pub fn dark() -> Self {
        Self {
            name: "dark",
            kind: ThemeKind::Dark,
            bg: Color::Rgb(16, 16, 24),
            surface: Color::Rgb(24, 24, 36),
            surface_elevated: Color::Rgb(32, 32, 48),
            fg: Color::Rgb(200, 200, 220),
            fg_muted: Color::Rgb(140, 140, 160),
            fg_subtle: Color::Rgb(100, 100, 120),
            fg_on_accent: Color::Rgb(0, 0, 0),
            primary: Color::Rgb(0, 200, 120),
            primary_hover: Color::Rgb(0, 220, 140),
            primary_active: Color::Rgb(0, 180, 100),
            secondary: Color::Rgb(100, 150, 200),
            secondary_hover: Color::Rgb(120, 170, 220),
            secondary_active: Color::Rgb(80, 130, 180),
            outline: Color::Rgb(60, 60, 80),
            outline_variant: Color::Rgb(45, 45, 65),
            divider: Color::Rgb(50, 50, 70),
            error: Color::Rgb(255, 80, 80),
            error_bg: Color::Rgb(50, 20, 20),
            success: Color::Rgb(80, 255, 120),
            success_bg: Color::Rgb(20, 50, 30),
            warning: Color::Rgb(255, 180, 80),
            warning_bg: Color::Rgb(50, 40, 20),
            info: Color::Rgb(100, 180, 255),
            info_bg: Color::Rgb(20, 40, 60),
            selection_bg: Color::Rgb(50, 80, 60),
            selection_fg: Color::Rgb(200, 255, 220),
            input_bg: Color::Rgb(20, 20, 30),
            input_fg: Color::Rgb(220, 220, 240),
            input_border: Color::Rgb(60, 60, 80),
            scrollbar_track: Color::Rgb(30, 30, 40),
            scrollbar_thumb: Color::Rgb(80, 80, 100),
            scrollbar_thumb_hover: Color::Rgb(100, 100, 120),
            disabled_fg: Color::Rgb(80, 80, 100),
            disabled_bg: Color::Rgb(35, 35, 50),
            hover_bg: Color::Rgb(40, 40, 56),
            focus_bg: Color::Rgb(50, 50, 70),
            focus_border: Color::Rgb(0, 200, 120),
            scrollbar_width: 1,
        }
    }

    /// Creates a light theme with high contrast suitable for bright environments.
    pub fn light() -> Self {
        Self {
            name: "light",
            kind: ThemeKind::Light,
            bg: Color::Rgb(250, 250, 250),
            surface: Color::Rgb(255, 255, 255),
            surface_elevated: Color::Rgb(245, 245, 250),
            fg: Color::Rgb(30, 30, 40),
            fg_muted: Color::Rgb(100, 100, 110),
            fg_subtle: Color::Rgb(140, 140, 150),
            fg_on_accent: Color::Rgb(255, 255, 255),
            primary: Color::Rgb(0, 100, 180),
            primary_hover: Color::Rgb(0, 120, 200),
            primary_active: Color::Rgb(0, 80, 160),
            secondary: Color::Rgb(100, 100, 180),
            secondary_hover: Color::Rgb(120, 120, 200),
            secondary_active: Color::Rgb(80, 80, 160),
            outline: Color::Rgb(180, 180, 190),
            outline_variant: Color::Rgb(210, 210, 215),
            divider: Color::Rgb(200, 200, 205),
            error: Color::Rgb(200, 40, 40),
            error_bg: Color::Rgb(255, 235, 235),
            success: Color::Rgb(40, 160, 40),
            success_bg: Color::Rgb(235, 250, 235),
            warning: Color::Rgb(200, 140, 40),
            warning_bg: Color::Rgb(255, 245, 220),
            info: Color::Rgb(40, 120, 200),
            info_bg: Color::Rgb(220, 240, 255),
            selection_bg: Color::Rgb(180, 220, 240),
            selection_fg: Color::Rgb(0, 0, 0),
            input_bg: Color::Rgb(255, 255, 255),
            input_fg: Color::Rgb(30, 30, 40),
            input_border: Color::Rgb(180, 180, 190),
            scrollbar_track: Color::Rgb(220, 220, 225),
            scrollbar_thumb: Color::Rgb(150, 150, 155),
            scrollbar_thumb_hover: Color::Rgb(120, 120, 125),
            disabled_fg: Color::Rgb(150, 150, 155),
            disabled_bg: Color::Rgb(235, 235, 240),
            hover_bg: Color::Rgb(240, 240, 245),
            focus_bg: Color::Rgb(230, 230, 240),
            focus_border: Color::Rgb(0, 100, 180),
            scrollbar_width: 1,
        }
    }

    /// Creates a high-contrast theme optimized for accessibility.
    ///
    /// Uses pure black and white with vivid, distinguishable colors.
    /// Ideal for users with low vision or color blindness.
    pub fn high_contrast() -> Self {
        Self {
            name: "high_contrast",
            kind: ThemeKind::Dark,
            bg: Color::Rgb(0, 0, 0),
            surface: Color::Rgb(20, 20, 20),
            surface_elevated: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(255, 255, 255),
            fg_muted: Color::Rgb(200, 200, 200),
            fg_subtle: Color::Rgb(160, 160, 160),
            fg_on_accent: Color::Rgb(0, 0, 0),
            primary: Color::Rgb(0, 160, 255),
            primary_hover: Color::Rgb(50, 180, 255),
            primary_active: Color::Rgb(0, 130, 220),
            secondary: Color::Rgb(255, 200, 0),
            secondary_hover: Color::Rgb(255, 220, 50),
            secondary_active: Color::Rgb(220, 180, 0),
            outline: Color::Rgb(255, 255, 255),
            outline_variant: Color::Rgb(180, 180, 180),
            divider: Color::Rgb(120, 120, 120),
            error: Color::Rgb(255, 50, 50),
            error_bg: Color::Rgb(60, 0, 0),
            success: Color::Rgb(50, 255, 50),
            success_bg: Color::Rgb(0, 40, 0),
            warning: Color::Rgb(255, 255, 0),
            warning_bg: Color::Rgb(60, 60, 0),
            info: Color::Rgb(0, 200, 255),
            info_bg: Color::Rgb(0, 30, 60),
            selection_bg: Color::Rgb(0, 100, 200),
            selection_fg: Color::Rgb(255, 255, 255),
            input_bg: Color::Rgb(0, 0, 0),
            input_fg: Color::Rgb(255, 255, 255),
            input_border: Color::Rgb(255, 255, 255),
            scrollbar_track: Color::Rgb(60, 60, 60),
            scrollbar_thumb: Color::Rgb(255, 255, 255),
            scrollbar_thumb_hover: Color::Rgb(0, 200, 255),
            disabled_fg: Color::Rgb(120, 120, 120),
            disabled_bg: Color::Rgb(40, 40, 40),
            hover_bg: Color::Rgb(60, 60, 60),
            focus_bg: Color::Rgb(0, 60, 120),
            focus_border: Color::Rgb(0, 200, 255),
            scrollbar_width: 1,
        }
    }
}

impl Theme {
    /// Look up a theme by its `.name` field (case-insensitive).
    ///
    /// Supports all built-in themes: nord, dark, light, high_contrast,
    /// cyberpunk, dracula, catppuccin_mocha, gruvbox_dark, tokyo_night,
    /// solarized_dark, solarized_light, one_dark, rose_pine, kanagawa,
    /// everforest, monokai, warm, cool, forest, sunset, mono.
    pub fn from_name(name: &str) -> Option<Theme> {
        // Normalize: lowercase, and treat hyphens as underscores so
        // "catppuccin-mocha" resolves the same as "catppuccin_mocha".
        let normalized = name.to_ascii_lowercase().replace('-', "_");
        match normalized.as_str() {
            "dark" => Some(Self::dark()),
            "light" => Some(Self::light()),
            "high_contrast" => Some(Self::high_contrast()),
            "cyberpunk" => Some(Self::cyberpunk()),
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "catppuccin_mocha" | "catppuccin" => Some(Self::catppuccin_mocha()),
            "gruvbox_dark" | "gruvbox" => Some(Self::gruvbox_dark()),
            "tokyo_night" => Some(Self::tokyo_night()),
            "solarized_dark" => Some(Self::solarized_dark()),
            "solarized_light" => Some(Self::solarized_light()),
            "one_dark" => Some(Self::one_dark()),
            "rose_pine" => Some(Self::rose_pine()),
            "kanagawa" => Some(Self::kanagawa()),
            "everforest" => Some(Self::everforest()),
            "monokai" => Some(Self::monokai()),
            "warm" => Some(Self::warm()),
            "cool" => Some(Self::cool()),
            "forest" => Some(Self::forest()),
            "sunset" => Some(Self::sunset()),
            "mono" => Some(Self::mono()),
            _ => None,
        }
    }

    /// Resolve a theme from the `DTRON_THEME` environment variable, or fall
    /// back to `default`. Call this at the top of any example's `main()` so
    /// it inherits the showcase's selected theme:
    ///
    /// ```ignore
    /// .theme(Theme::from_env_or(Theme::nord()))
    /// ```
    pub fn from_env_or(default: Theme) -> Theme {
        std::env::var("DTRON_THEME")
            .ok()
            .and_then(|n| Self::from_name(&n))
            .unwrap_or(default)
    }

    /// Return all built-in themes.
    ///
    /// Useful for theme cycling UIs, tests, and iterating over every
    /// available color scheme.
    pub fn all() -> Vec<Theme> {
        vec![
            Self::dark(),
            Self::light(),
            Self::high_contrast(),
            Self::cyberpunk(),
            Self::dracula(),
            Self::nord(),
            Self::catppuccin_mocha(),
            Self::gruvbox_dark(),
            Self::tokyo_night(),
            Self::solarized_dark(),
            Self::solarized_light(),
            Self::one_dark(),
            Self::rose_pine(),
            Self::kanagawa(),
            Self::everforest(),
            Self::monokai(),
            Self::warm(),
            Self::cool(),
            Self::forest(),
            Self::sunset(),
            Self::mono(),
        ]
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
