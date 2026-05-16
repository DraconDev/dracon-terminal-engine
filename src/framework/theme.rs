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
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeKind {
    #[default]
    Dark,
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
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: std::sync::Arc<str>,
    /// Human-friendly display label (e.g. "Solarized Dark", "Rosé Pine").
    /// Used for theme cycling menus and the showcase launcher.
    pub display_name: std::sync::Arc<str>,
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
    /// **Deprecated:** Layout dimensions should not live in Theme.
    /// Use [`crate::framework::scroll::DEFAULT_SCROLLBAR_WIDTH`] instead.
    /// This field is kept for backward compatibility and will be removed in a future release.
    #[deprecated(since = "0.3.0", note = "Use framework::scroll::DEFAULT_SCROLLBAR_WIDTH instead")]
    #[doc(hidden)]
    pub scrollbar_width: u16,
}

#[allow(deprecated)]
impl Theme {
    #[allow(deprecated)]
    #[inline]
    fn default_scrollbar_width() -> u16 {
        crate::framework::scroll::DEFAULT_SCROLLBAR_WIDTH
    }

    /// Creates a dark theme with muted colors suitable for low-light environments.
    pub fn dark() -> Self {
        Self {
            name: "dark".into(),
            display_name: "Dark".into(),
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
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates a light theme with high contrast suitable for bright environments.
    pub fn light() -> Self {
        Self {
            name: "light".into(),
            display_name: "Light".into(),
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
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates a high-contrast theme optimized for accessibility.
    ///
    /// Uses pure black and white with vivid, distinguishable colors.
    /// Ideal for users with low vision or color blindness.
    pub fn high_contrast() -> Self {
        Self {
            name: "high_contrast".into(),
            display_name: "High Contrast".into(),
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
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates a cyberpunk-themed dark theme with neon green and hot pink accents.
    pub fn cyberpunk() -> Self {
        Self {
            name: "cyberpunk".into(),
            display_name: "Cyberpunk".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(0, 0, 0),
            surface: Color::Rgb(10, 10, 15),
            surface_elevated: Color::Rgb(20, 20, 30),
            fg: Color::Rgb(0, 255, 136),
            fg_muted: Color::Rgb(0, 180, 100),
            fg_subtle: Color::Rgb(0, 120, 70),
            fg_on_accent: Color::Rgb(0, 0, 0),
            primary: Color::Rgb(255, 0, 100),
            primary_hover: Color::Rgb(255, 50, 130),
            primary_active: Color::Rgb(200, 0, 80),
            secondary: Color::Rgb(0, 200, 255),
            secondary_hover: Color::Rgb(50, 220, 255),
            secondary_active: Color::Rgb(0, 160, 220),
            outline: Color::Rgb(0, 200, 100),
            outline_variant: Color::Rgb(0, 150, 80),
            divider: Color::Rgb(0, 100, 60),
            error: Color::Rgb(255, 0, 80),
            error_bg: Color::Rgb(50, 0, 20),
            success: Color::Rgb(0, 255, 180),
            success_bg: Color::Rgb(0, 40, 30),
            warning: Color::Rgb(255, 200, 0),
            warning_bg: Color::Rgb(50, 40, 0),
            info: Color::Rgb(100, 200, 255),
            info_bg: Color::Rgb(20, 40, 60),
            selection_bg: Color::Rgb(0, 50, 30),
            selection_fg: Color::Rgb(0, 255, 200),
            input_bg: Color::Rgb(10, 10, 20),
            input_fg: Color::Rgb(0, 255, 136),
            input_border: Color::Rgb(0, 200, 100),
            scrollbar_track: Color::Rgb(0, 30, 20),
            scrollbar_thumb: Color::Rgb(0, 150, 80),
            scrollbar_thumb_hover: Color::Rgb(0, 180, 100),
            disabled_fg: Color::Rgb(0, 80, 50),
            disabled_bg: Color::Rgb(20, 20, 30),
            hover_bg: Color::Rgb(25, 25, 40),
            focus_bg: Color::Rgb(35, 35, 55),
            focus_border: Color::Rgb(255, 0, 100),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Dracula theme — iconic dark purple aesthetic with vivid accents.
    pub fn dracula() -> Self {
        Self {
            name: "dracula".into(),
            display_name: "Dracula".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(40, 42, 54),
            surface: Color::Rgb(48, 50, 64),
            surface_elevated: Color::Rgb(60, 62, 78),
            fg: Color::Rgb(248, 248, 242),
            fg_muted: Color::Rgb(180, 180, 200),
            fg_subtle: Color::Rgb(120, 120, 140),
            fg_on_accent: Color::Rgb(255, 255, 255),
            primary: Color::Rgb(98, 114, 164),
            primary_hover: Color::Rgb(118, 134, 184),
            primary_active: Color::Rgb(78, 94, 144),
            secondary: Color::Rgb(255, 121, 198),
            secondary_hover: Color::Rgb(255, 141, 218),
            secondary_active: Color::Rgb(235, 101, 178),
            outline: Color::Rgb(98, 114, 164),
            outline_variant: Color::Rgb(68, 84, 124),
            divider: Color::Rgb(60, 65, 80),
            error: Color::Rgb(255, 85, 85),
            error_bg: Color::Rgb(60, 20, 20),
            success: Color::Rgb(80, 250, 123),
            success_bg: Color::Rgb(20, 50, 30),
            warning: Color::Rgb(241, 250, 140),
            warning_bg: Color::Rgb(50, 50, 20),
            info: Color::Rgb(139, 233, 253),
            info_bg: Color::Rgb(30, 50, 60),
            selection_bg: Color::Rgb(68, 71, 90),
            selection_fg: Color::Rgb(255, 255, 255),
            input_bg: Color::Rgb(30, 32, 42),
            input_fg: Color::Rgb(248, 248, 242),
            input_border: Color::Rgb(98, 114, 164),
            scrollbar_track: Color::Rgb(30, 32, 42),
            scrollbar_thumb: Color::Rgb(68, 71, 90),
            scrollbar_thumb_hover: Color::Rgb(88, 91, 110),
            disabled_fg: Color::Rgb(68, 71, 90),
            disabled_bg: Color::Rgb(50, 52, 64),
            hover_bg: Color::Rgb(58, 60, 76),
            focus_bg: Color::Rgb(68, 70, 86),
            focus_border: Color::Rgb(255, 121, 198),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Nord theme — arctic blue-gray palette.
    pub fn nord() -> Self {
        Self {
            name: "nord".into(),
            display_name: "Nord".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(46, 52, 64),
            surface: Color::Rgb(52, 58, 72),
            surface_elevated: Color::Rgb(62, 68, 84),
            fg: Color::Rgb(216, 222, 233),
            fg_muted: Color::Rgb(160, 170, 185),
            fg_subtle: Color::Rgb(120, 130, 145),
            fg_on_accent: Color::Rgb(46, 52, 64),
            primary: Color::Rgb(136, 192, 208),
            primary_hover: Color::Rgb(156, 212, 228),
            primary_active: Color::Rgb(116, 172, 188),
            secondary: Color::Rgb(163, 190, 140),
            secondary_hover: Color::Rgb(183, 210, 160),
            secondary_active: Color::Rgb(143, 170, 120),
            outline: Color::Rgb(67, 76, 94),
            outline_variant: Color::Rgb(57, 66, 84),
            divider: Color::Rgb(55, 63, 78),
            error: Color::Rgb(191, 97, 106),
            error_bg: Color::Rgb(60, 25, 30),
            success: Color::Rgb(163, 190, 140),
            success_bg: Color::Rgb(30, 50, 25),
            warning: Color::Rgb(235, 203, 139),
            warning_bg: Color::Rgb(55, 50, 25),
            info: Color::Rgb(136, 192, 208),
            info_bg: Color::Rgb(30, 50, 60),
            selection_bg: Color::Rgb(67, 76, 94),
            selection_fg: Color::Rgb(236, 240, 243),
            input_bg: Color::Rgb(35, 40, 52),
            input_fg: Color::Rgb(216, 222, 233),
            input_border: Color::Rgb(67, 76, 94),
            scrollbar_track: Color::Rgb(35, 40, 52),
            scrollbar_thumb: Color::Rgb(67, 76, 94),
            scrollbar_thumb_hover: Color::Rgb(87, 96, 114),
            disabled_fg: Color::Rgb(119, 128, 144),
            disabled_bg: Color::Rgb(50, 55, 68),
            hover_bg: Color::Rgb(56, 62, 78),
            focus_bg: Color::Rgb(66, 72, 88),
            focus_border: Color::Rgb(136, 192, 208),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Catppuccin Mocha theme — warm, soothing pastel dark theme.
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin_mocha".into(),
            display_name: "Catppuccin".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(30, 30, 46),
            surface: Color::Rgb(38, 38, 56),
            surface_elevated: Color::Rgb(48, 48, 68),
            fg: Color::Rgb(205, 214, 244),
            fg_muted: Color::Rgb(160, 168, 200),
            fg_subtle: Color::Rgb(110, 118, 150),
            fg_on_accent: Color::Rgb(30, 30, 46),
            primary: Color::Rgb(137, 180, 250),
            primary_hover: Color::Rgb(157, 200, 255),
            primary_active: Color::Rgb(117, 160, 230),
            secondary: Color::Rgb(166, 227, 161),
            secondary_hover: Color::Rgb(186, 247, 181),
            secondary_active: Color::Rgb(146, 207, 141),
            outline: Color::Rgb(88, 91, 112),
            outline_variant: Color::Rgb(68, 71, 92),
            divider: Color::Rgb(55, 58, 78),
            error: Color::Rgb(243, 139, 168),
            error_bg: Color::Rgb(60, 25, 40),
            success: Color::Rgb(166, 227, 161),
            success_bg: Color::Rgb(25, 50, 30),
            warning: Color::Rgb(249, 226, 175),
            warning_bg: Color::Rgb(55, 50, 30),
            info: Color::Rgb(137, 180, 250),
            info_bg: Color::Rgb(30, 40, 60),
            selection_bg: Color::Rgb(49, 50, 68),
            selection_fg: Color::Rgb(230, 233, 244),
            input_bg: Color::Rgb(24, 24, 37),
            input_fg: Color::Rgb(205, 214, 244),
            input_border: Color::Rgb(88, 91, 112),
            scrollbar_track: Color::Rgb(24, 24, 37),
            scrollbar_thumb: Color::Rgb(88, 91, 112),
            scrollbar_thumb_hover: Color::Rgb(108, 111, 132),
            disabled_fg: Color::Rgb(108, 112, 134),
            disabled_bg: Color::Rgb(40, 40, 58),
            hover_bg: Color::Rgb(52, 52, 72),
            focus_bg: Color::Rgb(62, 62, 82),
            focus_border: Color::Rgb(137, 180, 250),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Gruvbox Dark theme — retro warm dark theme with earthy tones.
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "gruvbox_dark".into(),
            display_name: "Gruvbox".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(40, 40, 40),
            surface: Color::Rgb(50, 48, 45),
            surface_elevated: Color::Rgb(60, 58, 55),
            fg: Color::Rgb(213, 196, 161),
            fg_muted: Color::Rgb(180, 165, 135),
            fg_subtle: Color::Rgb(140, 125, 100),
            fg_on_accent: Color::Rgb(40, 40, 40),
            primary: Color::Rgb(214, 93, 14),
            primary_hover: Color::Rgb(234, 113, 34),
            primary_active: Color::Rgb(194, 73, 0),
            secondary: Color::Rgb(152, 151, 26),
            secondary_hover: Color::Rgb(172, 171, 46),
            secondary_active: Color::Rgb(132, 131, 6),
            outline: Color::Rgb(120, 90, 60),
            outline_variant: Color::Rgb(90, 70, 50),
            divider: Color::Rgb(80, 75, 55),
            error: Color::Rgb(204, 36, 36),
            error_bg: Color::Rgb(50, 20, 20),
            success: Color::Rgb(152, 151, 26),
            success_bg: Color::Rgb(40, 45, 20),
            warning: Color::Rgb(215, 153, 33),
            warning_bg: Color::Rgb(50, 40, 20),
            info: Color::Rgb(131, 165, 152),
            info_bg: Color::Rgb(35, 45, 40),
            selection_bg: Color::Rgb(100, 70, 40),
            selection_fg: Color::Rgb(235, 219, 178),
            input_bg: Color::Rgb(30, 30, 30),
            input_fg: Color::Rgb(213, 196, 161),
            input_border: Color::Rgb(120, 90, 60),
            scrollbar_track: Color::Rgb(30, 30, 30),
            scrollbar_thumb: Color::Rgb(100, 70, 40),
            scrollbar_thumb_hover: Color::Rgb(120, 90, 60),
            disabled_fg: Color::Rgb(120, 90, 60),
            disabled_bg: Color::Rgb(50, 45, 40),
            hover_bg: Color::Rgb(65, 60, 55),
            focus_bg: Color::Rgb(75, 70, 65),
            focus_border: Color::Rgb(214, 93, 14),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Tokyo Night theme — vivid blue accents on a dark background.
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo_night".into(),
            display_name: "Tokyo Night".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(32, 34, 44),
            surface: Color::Rgb(38, 40, 52),
            surface_elevated: Color::Rgb(48, 50, 64),
            fg: Color::Rgb(192, 202, 245),
            fg_muted: Color::Rgb(140, 150, 180),
            fg_subtle: Color::Rgb(100, 110, 140),
            fg_on_accent: Color::Rgb(32, 34, 44),
            primary: Color::Rgb(98, 130, 234),
            primary_hover: Color::Rgb(118, 150, 254),
            primary_active: Color::Rgb(78, 110, 214),
            secondary: Color::Rgb(166, 227, 161),
            secondary_hover: Color::Rgb(186, 247, 181),
            secondary_active: Color::Rgb(146, 207, 141),
            outline: Color::Rgb(62, 64, 82),
            outline_variant: Color::Rgb(52, 54, 72),
            divider: Color::Rgb(48, 52, 68),
            error: Color::Rgb(255, 85, 85),
            error_bg: Color::Rgb(55, 20, 20),
            success: Color::Rgb(166, 227, 161),
            success_bg: Color::Rgb(25, 50, 30),
            warning: Color::Rgb(255, 184, 108),
            warning_bg: Color::Rgb(55, 45, 25),
            info: Color::Rgb(98, 130, 234),
            info_bg: Color::Rgb(30, 35, 55),
            selection_bg: Color::Rgb(52, 54, 70),
            selection_fg: Color::Rgb(202, 212, 254),
            input_bg: Color::Rgb(22, 24, 34),
            input_fg: Color::Rgb(192, 202, 245),
            input_border: Color::Rgb(62, 64, 82),
            scrollbar_track: Color::Rgb(22, 24, 34),
            scrollbar_thumb: Color::Rgb(62, 64, 82),
            scrollbar_thumb_hover: Color::Rgb(82, 84, 102),
            disabled_fg: Color::Rgb(113, 117, 138),
            disabled_bg: Color::Rgb(40, 42, 54),
            hover_bg: Color::Rgb(42, 44, 58),
            focus_bg: Color::Rgb(52, 54, 68),
            focus_border: Color::Rgb(98, 130, 234),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Solarized Dark theme — precision-engineered dark theme.
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized_dark".into(),
            display_name: "Solarized Dark".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(0, 43, 54),
            surface: Color::Rgb(0, 50, 62),
            surface_elevated: Color::Rgb(0, 60, 75),
            fg: Color::Rgb(131, 148, 150),
            fg_muted: Color::Rgb(100, 115, 120),
            fg_subtle: Color::Rgb(70, 85, 90),
            fg_on_accent: Color::Rgb(253, 246, 227),
            primary: Color::Rgb(38, 139, 210),
            primary_hover: Color::Rgb(58, 159, 230),
            primary_active: Color::Rgb(18, 119, 190),
            secondary: Color::Rgb(133, 153, 0),
            secondary_hover: Color::Rgb(153, 173, 20),
            secondary_active: Color::Rgb(113, 133, 0),
            outline: Color::Rgb(0, 80, 100),
            outline_variant: Color::Rgb(0, 65, 85),
            divider: Color::Rgb(0, 70, 88),
            error: Color::Rgb(220, 50, 47),
            error_bg: Color::Rgb(50, 20, 18),
            success: Color::Rgb(133, 153, 0),
            success_bg: Color::Rgb(20, 35, 0),
            warning: Color::Rgb(181, 137, 0),
            warning_bg: Color::Rgb(45, 35, 0),
            info: Color::Rgb(38, 139, 210),
            info_bg: Color::Rgb(15, 35, 55),
            selection_bg: Color::Rgb(0, 60, 76),
            selection_fg: Color::Rgb(147, 161, 161),
            input_bg: Color::Rgb(0, 33, 44),
            input_fg: Color::Rgb(131, 148, 150),
            input_border: Color::Rgb(0, 80, 100),
            scrollbar_track: Color::Rgb(0, 33, 44),
            scrollbar_thumb: Color::Rgb(0, 80, 100),
            scrollbar_thumb_hover: Color::Rgb(0, 100, 120),
            disabled_fg: Color::Rgb(88, 110, 117),
            disabled_bg: Color::Rgb(0, 55, 68),
            hover_bg: Color::Rgb(10, 55, 72),
            focus_bg: Color::Rgb(20, 65, 82),
            focus_border: Color::Rgb(38, 139, 210),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Solarized Light theme — precision-engineered light theme.
    pub fn solarized_light() -> Self {
        Self {
            name: "solarized_light".into(),
            display_name: "Solarized Light".into(),
            kind: ThemeKind::Light,
            bg: Color::Rgb(253, 246, 227),
            surface: Color::Rgb(253, 249, 240),
            surface_elevated: Color::Rgb(255, 255, 250),
            fg: Color::Rgb(101, 123, 131),
            fg_muted: Color::Rgb(120, 140, 145),
            fg_subtle: Color::Rgb(140, 155, 160),
            fg_on_accent: Color::Rgb(253, 246, 227),
            primary: Color::Rgb(38, 139, 210),
            primary_hover: Color::Rgb(28, 129, 200),
            primary_active: Color::Rgb(48, 149, 220),
            secondary: Color::Rgb(133, 153, 0),
            secondary_hover: Color::Rgb(113, 133, 0),
            secondary_active: Color::Rgb(153, 173, 20),
            outline: Color::Rgb(147, 161, 161),
            outline_variant: Color::Rgb(170, 180, 185),
            divider: Color::Rgb(200, 210, 215),
            error: Color::Rgb(220, 50, 47),
            error_bg: Color::Rgb(255, 240, 240),
            success: Color::Rgb(133, 153, 0),
            success_bg: Color::Rgb(245, 250, 230),
            warning: Color::Rgb(181, 137, 0),
            warning_bg: Color::Rgb(255, 250, 230),
            info: Color::Rgb(38, 139, 210),
            info_bg: Color::Rgb(235, 245, 255),
            selection_bg: Color::Rgb(181, 209, 240),
            selection_fg: Color::Rgb(0, 43, 54),
            input_bg: Color::Rgb(253, 246, 227),
            input_fg: Color::Rgb(101, 123, 131),
            input_border: Color::Rgb(147, 161, 161),
            scrollbar_track: Color::Rgb(220, 220, 215),
            scrollbar_thumb: Color::Rgb(147, 161, 161),
            scrollbar_thumb_hover: Color::Rgb(127, 141, 141),
            disabled_fg: Color::Rgb(147, 161, 161),
            disabled_bg: Color::Rgb(240, 238, 230),
            hover_bg: Color::Rgb(248, 245, 238),
            focus_bg: Color::Rgb(238, 235, 228),
            focus_border: Color::Rgb(38, 139, 210),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the One Dark theme — Atom editor's iconic dark theme.
    pub fn one_dark() -> Self {
        Self {
            name: "one_dark".into(),
            display_name: "One Dark".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(40, 44, 52),
            surface: Color::Rgb(48, 52, 62),
            surface_elevated: Color::Rgb(58, 62, 74),
            fg: Color::Rgb(220, 223, 228),
            fg_muted: Color::Rgb(160, 165, 175),
            fg_subtle: Color::Rgb(110, 115, 125),
            fg_on_accent: Color::Rgb(40, 44, 52),
            primary: Color::Rgb(97, 175, 239),
            primary_hover: Color::Rgb(117, 195, 255),
            primary_active: Color::Rgb(77, 155, 219),
            secondary: Color::Rgb(152, 195, 121),
            secondary_hover: Color::Rgb(172, 215, 141),
            secondary_active: Color::Rgb(132, 175, 101),
            outline: Color::Rgb(62, 66, 76),
            outline_variant: Color::Rgb(52, 56, 66),
            divider: Color::Rgb(50, 54, 64),
            error: Color::Rgb(224, 108, 108),
            error_bg: Color::Rgb(55, 25, 25),
            success: Color::Rgb(152, 195, 121),
            success_bg: Color::Rgb(30, 50, 25),
            warning: Color::Rgb(229, 192, 120),
            warning_bg: Color::Rgb(55, 45, 25),
            info: Color::Rgb(97, 175, 239),
            info_bg: Color::Rgb(25, 40, 55),
            selection_bg: Color::Rgb(62, 66, 76),
            selection_fg: Color::Rgb(230, 233, 239),
            input_bg: Color::Rgb(30, 34, 42),
            input_fg: Color::Rgb(220, 223, 228),
            input_border: Color::Rgb(62, 66, 76),
            scrollbar_track: Color::Rgb(30, 34, 42),
            scrollbar_thumb: Color::Rgb(62, 66, 76),
            scrollbar_thumb_hover: Color::Rgb(82, 86, 96),
            disabled_fg: Color::Rgb(124, 131, 141),
            disabled_bg: Color::Rgb(48, 52, 60),
            hover_bg: Color::Rgb(52, 56, 68),
            focus_bg: Color::Rgb(62, 66, 78),
            focus_border: Color::Rgb(97, 175, 239),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Rosé Pine theme — elegant muted rose tones.
    pub fn rose_pine() -> Self {
        Self {
            name: "rose_pine".into(),
            display_name: "Rosé Pine".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(30, 30, 46),
            surface: Color::Rgb(38, 36, 54),
            surface_elevated: Color::Rgb(48, 46, 64),
            fg: Color::Rgb(220, 200, 200),
            fg_muted: Color::Rgb(170, 155, 165),
            fg_subtle: Color::Rgb(120, 110, 125),
            fg_on_accent: Color::Rgb(30, 30, 46),
            primary: Color::Rgb(210, 160, 160),
            primary_hover: Color::Rgb(230, 180, 180),
            primary_active: Color::Rgb(190, 140, 140),
            secondary: Color::Rgb(235, 200, 180),
            secondary_hover: Color::Rgb(245, 210, 195),
            secondary_active: Color::Rgb(215, 190, 170),
            outline: Color::Rgb(60, 55, 75),
            outline_variant: Color::Rgb(50, 45, 65),
            divider: Color::Rgb(48, 44, 62),
            error: Color::Rgb(210, 160, 160),
            error_bg: Color::Rgb(50, 30, 40),
            success: Color::Rgb(204, 170, 140),
            success_bg: Color::Rgb(45, 38, 32),
            warning: Color::Rgb(230, 200, 160),
            warning_bg: Color::Rgb(52, 45, 35),
            info: Color::Rgb(190, 180, 210),
            info_bg: Color::Rgb(38, 35, 55),
            selection_bg: Color::Rgb(50, 45, 65),
            selection_fg: Color::Rgb(230, 210, 210),
            input_bg: Color::Rgb(24, 24, 37),
            input_fg: Color::Rgb(220, 200, 200),
            input_border: Color::Rgb(60, 55, 75),
            scrollbar_track: Color::Rgb(24, 24, 37),
            scrollbar_thumb: Color::Rgb(60, 55, 75),
            scrollbar_thumb_hover: Color::Rgb(80, 75, 95),
            disabled_fg: Color::Rgb(100, 90, 100),
            disabled_bg: Color::Rgb(40, 38, 55),
            hover_bg: Color::Rgb(52, 50, 68),
            focus_bg: Color::Rgb(62, 60, 78),
            focus_border: Color::Rgb(210, 160, 160),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Kanagawa theme — inspired by Hokusai's art with deep blues and golds.
    pub fn kanagawa() -> Self {
        Self {
            name: "kanagawa".into(),
            display_name: "Kanagawa".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(38, 40, 64),
            surface: Color::Rgb(44, 46, 72),
            surface_elevated: Color::Rgb(54, 56, 84),
            fg: Color::Rgb(220, 217, 201),
            fg_muted: Color::Rgb(170, 167, 155),
            fg_subtle: Color::Rgb(120, 117, 110),
            fg_on_accent: Color::Rgb(38, 40, 64),
            primary: Color::Rgb(166, 122, 102),
            primary_hover: Color::Rgb(186, 142, 122),
            primary_active: Color::Rgb(146, 102, 82),
            secondary: Color::Rgb(166, 206, 122),
            secondary_hover: Color::Rgb(186, 226, 142),
            secondary_active: Color::Rgb(146, 186, 102),
            outline: Color::Rgb(88, 90, 112),
            outline_variant: Color::Rgb(68, 70, 92),
            divider: Color::Rgb(60, 62, 85),
            error: Color::Rgb(219, 98, 98),
            error_bg: Color::Rgb(55, 25, 28),
            success: Color::Rgb(166, 206, 122),
            success_bg: Color::Rgb(30, 50, 25),
            warning: Color::Rgb(230, 186, 122),
            warning_bg: Color::Rgb(55, 45, 28),
            info: Color::Rgb(166, 122, 102),
            info_bg: Color::Rgb(40, 35, 50),
            selection_bg: Color::Rgb(60, 62, 90),
            selection_fg: Color::Rgb(230, 225, 205),
            input_bg: Color::Rgb(28, 30, 54),
            input_fg: Color::Rgb(220, 217, 201),
            input_border: Color::Rgb(88, 90, 112),
            scrollbar_track: Color::Rgb(28, 30, 54),
            scrollbar_thumb: Color::Rgb(88, 90, 112),
            scrollbar_thumb_hover: Color::Rgb(108, 110, 132),
            disabled_fg: Color::Rgb(100, 95, 110),
            disabled_bg: Color::Rgb(45, 47, 68),
            hover_bg: Color::Rgb(50, 52, 78),
            focus_bg: Color::Rgb(60, 62, 88),
            focus_border: Color::Rgb(166, 122, 102),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Everforest theme — comfortable forest green dark theme.
    pub fn everforest() -> Self {
        Self {
            name: "everforest".into(),
            display_name: "Everforest".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(43, 48, 40),
            surface: Color::Rgb(50, 55, 48),
            surface_elevated: Color::Rgb(58, 65, 58),
            fg: Color::Rgb(210, 191, 163),
            fg_muted: Color::Rgb(165, 150, 130),
            fg_subtle: Color::Rgb(115, 105, 90),
            fg_on_accent: Color::Rgb(43, 48, 40),
            primary: Color::Rgb(148, 181, 97),
            primary_hover: Color::Rgb(168, 201, 117),
            primary_active: Color::Rgb(128, 161, 77),
            secondary: Color::Rgb(147, 170, 200),
            secondary_hover: Color::Rgb(167, 190, 220),
            secondary_active: Color::Rgb(127, 150, 180),
            outline: Color::Rgb(80, 90, 70),
            outline_variant: Color::Rgb(65, 75, 58),
            divider: Color::Rgb(60, 68, 55),
            error: Color::Rgb(220, 110, 110),
            error_bg: Color::Rgb(50, 25, 28),
            success: Color::Rgb(148, 181, 97),
            success_bg: Color::Rgb(28, 45, 22),
            warning: Color::Rgb(230, 180, 110),
            warning_bg: Color::Rgb(52, 42, 28),
            info: Color::Rgb(147, 170, 200),
            info_bg: Color::Rgb(35, 42, 55),
            selection_bg: Color::Rgb(60, 68, 55),
            selection_fg: Color::Rgb(222, 211, 179),
            input_bg: Color::Rgb(33, 38, 30),
            input_fg: Color::Rgb(210, 191, 163),
            input_border: Color::Rgb(80, 90, 70),
            scrollbar_track: Color::Rgb(33, 38, 30),
            scrollbar_thumb: Color::Rgb(80, 90, 70),
            scrollbar_thumb_hover: Color::Rgb(100, 110, 90),
            disabled_fg: Color::Rgb(100, 110, 90),
            disabled_bg: Color::Rgb(48, 53, 45),
            hover_bg: Color::Rgb(55, 62, 52),
            focus_bg: Color::Rgb(65, 72, 62),
            focus_border: Color::Rgb(148, 181, 97),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Monokai theme — classic syntax highlighting colors.
    pub fn monokai() -> Self {
        Self {
            name: "monokai".into(),
            display_name: "Monokai".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(39, 40, 34),
            surface: Color::Rgb(46, 47, 40),
            surface_elevated: Color::Rgb(55, 56, 50),
            fg: Color::Rgb(248, 248, 242),
            fg_muted: Color::Rgb(190, 190, 175),
            fg_subtle: Color::Rgb(140, 140, 125),
            fg_on_accent: Color::Rgb(39, 40, 34),
            primary: Color::Rgb(102, 217, 239),
            primary_hover: Color::Rgb(122, 237, 255),
            primary_active: Color::Rgb(82, 197, 219),
            secondary: Color::Rgb(166, 226, 50),
            secondary_hover: Color::Rgb(186, 246, 70),
            secondary_active: Color::Rgb(146, 206, 30),
            outline: Color::Rgb(100, 95, 80),
            outline_variant: Color::Rgb(80, 75, 65),
            divider: Color::Rgb(70, 72, 60),
            error: Color::Rgb(249, 38, 114),
            error_bg: Color::Rgb(55, 15, 30),
            success: Color::Rgb(166, 226, 50),
            success_bg: Color::Rgb(40, 50, 20),
            warning: Color::Rgb(230, 200, 80),
            warning_bg: Color::Rgb(55, 48, 25),
            info: Color::Rgb(102, 217, 239),
            info_bg: Color::Rgb(30, 40, 50),
            selection_bg: Color::Rgb(80, 75, 60),
            selection_fg: Color::Rgb(250, 250, 250),
            input_bg: Color::Rgb(29, 30, 24),
            input_fg: Color::Rgb(248, 248, 242),
            input_border: Color::Rgb(100, 95, 80),
            scrollbar_track: Color::Rgb(29, 30, 24),
            scrollbar_thumb: Color::Rgb(100, 95, 80),
            scrollbar_thumb_hover: Color::Rgb(120, 115, 100),
            disabled_fg: Color::Rgb(130, 125, 110),
            disabled_bg: Color::Rgb(45, 46, 38),
            hover_bg: Color::Rgb(58, 59, 48),
            focus_bg: Color::Rgb(68, 69, 58),
            focus_border: Color::Rgb(166, 226, 50),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Warm theme — cozy amber and bronze tones.
    pub fn warm() -> Self {
        Self {
            name: "warm".into(),
            display_name: "Warm".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(28, 26, 24),
            surface: Color::Rgb(36, 34, 30),
            surface_elevated: Color::Rgb(44, 42, 38),
            fg: Color::Rgb(240, 228, 210),
            fg_muted: Color::Rgb(190, 175, 155),
            fg_subtle: Color::Rgb(140, 128, 110),
            fg_on_accent: Color::Rgb(28, 26, 24),
            primary: Color::Rgb(224, 164, 90),
            primary_hover: Color::Rgb(240, 190, 120),
            primary_active: Color::Rgb(200, 140, 70),
            secondary: Color::Rgb(94, 199, 178),
            secondary_hover: Color::Rgb(114, 219, 198),
            secondary_active: Color::Rgb(74, 179, 158),
            outline: Color::Rgb(86, 80, 72),
            outline_variant: Color::Rgb(72, 66, 58),
            divider: Color::Rgb(60, 56, 50),
            error: Color::Rgb(220, 100, 100),
            error_bg: Color::Rgb(50, 20, 20),
            success: Color::Rgb(120, 200, 120),
            success_bg: Color::Rgb(20, 45, 20),
            warning: Color::Rgb(230, 180, 80),
            warning_bg: Color::Rgb(50, 40, 15),
            info: Color::Rgb(94, 199, 178),
            info_bg: Color::Rgb(20, 45, 40),
            selection_bg: Color::Rgb(80, 72, 60),
            selection_fg: Color::Rgb(250, 245, 235),
            input_bg: Color::Rgb(22, 20, 18),
            input_fg: Color::Rgb(240, 228, 210),
            input_border: Color::Rgb(86, 80, 72),
            scrollbar_track: Color::Rgb(22, 20, 18),
            scrollbar_thumb: Color::Rgb(86, 80, 72),
            scrollbar_thumb_hover: Color::Rgb(106, 100, 92),
            disabled_fg: Color::Rgb(110, 100, 90),
            disabled_bg: Color::Rgb(40, 38, 34),
            hover_bg: Color::Rgb(50, 46, 40),
            focus_bg: Color::Rgb(60, 56, 50),
            focus_border: Color::Rgb(224, 164, 90),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Cool theme — purple and ice blue tones.
    pub fn cool() -> Self {
        Self {
            name: "cool".into(),
            display_name: "Cool".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(24, 26, 32),
            surface: Color::Rgb(30, 32, 40),
            surface_elevated: Color::Rgb(38, 40, 50),
            fg: Color::Rgb(220, 222, 240),
            fg_muted: Color::Rgb(170, 172, 195),
            fg_subtle: Color::Rgb(120, 122, 145),
            fg_on_accent: Color::Rgb(24, 26, 32),
            primary: Color::Rgb(160, 118, 255),
            primary_hover: Color::Rgb(180, 138, 255),
            primary_active: Color::Rgb(140, 98, 235),
            secondary: Color::Rgb(116, 184, 255),
            secondary_hover: Color::Rgb(136, 204, 255),
            secondary_active: Color::Rgb(96, 164, 235),
            outline: Color::Rgb(82, 86, 104),
            outline_variant: Color::Rgb(68, 72, 90),
            divider: Color::Rgb(56, 60, 76),
            error: Color::Rgb(255, 120, 140),
            error_bg: Color::Rgb(50, 15, 25),
            success: Color::Rgb(120, 220, 160),
            success_bg: Color::Rgb(18, 45, 30),
            warning: Color::Rgb(255, 200, 100),
            warning_bg: Color::Rgb(50, 40, 15),
            info: Color::Rgb(116, 184, 255),
            info_bg: Color::Rgb(20, 38, 55),
            selection_bg: Color::Rgb(70, 64, 95),
            selection_fg: Color::Rgb(240, 238, 255),
            input_bg: Color::Rgb(20, 22, 28),
            input_fg: Color::Rgb(220, 222, 240),
            input_border: Color::Rgb(82, 86, 104),
            scrollbar_track: Color::Rgb(20, 22, 28),
            scrollbar_thumb: Color::Rgb(82, 86, 104),
            scrollbar_thumb_hover: Color::Rgb(102, 106, 124),
            disabled_fg: Color::Rgb(100, 102, 120),
            disabled_bg: Color::Rgb(36, 38, 46),
            hover_bg: Color::Rgb(44, 46, 58),
            focus_bg: Color::Rgb(54, 56, 68),
            focus_border: Color::Rgb(160, 118, 255),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Forest theme — moss green and pine tones.
    pub fn forest() -> Self {
        Self {
            name: "forest".into(),
            display_name: "Forest".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(24, 30, 26),
            surface: Color::Rgb(30, 38, 32),
            surface_elevated: Color::Rgb(38, 48, 40),
            fg: Color::Rgb(210, 225, 210),
            fg_muted: Color::Rgb(160, 180, 160),
            fg_subtle: Color::Rgb(110, 130, 110),
            fg_on_accent: Color::Rgb(24, 30, 26),
            primary: Color::Rgb(126, 196, 102),
            primary_hover: Color::Rgb(146, 216, 122),
            primary_active: Color::Rgb(106, 176, 82),
            secondary: Color::Rgb(86, 168, 142),
            secondary_hover: Color::Rgb(106, 188, 162),
            secondary_active: Color::Rgb(66, 148, 122),
            outline: Color::Rgb(66, 80, 70),
            outline_variant: Color::Rgb(54, 68, 58),
            divider: Color::Rgb(48, 60, 52),
            error: Color::Rgb(220, 110, 110),
            error_bg: Color::Rgb(50, 20, 20),
            success: Color::Rgb(126, 196, 102),
            success_bg: Color::Rgb(22, 45, 18),
            warning: Color::Rgb(220, 180, 80),
            warning_bg: Color::Rgb(50, 42, 15),
            info: Color::Rgb(86, 168, 142),
            info_bg: Color::Rgb(18, 40, 35),
            selection_bg: Color::Rgb(58, 72, 52),
            selection_fg: Color::Rgb(230, 240, 230),
            input_bg: Color::Rgb(20, 26, 22),
            input_fg: Color::Rgb(210, 225, 210),
            input_border: Color::Rgb(66, 80, 70),
            scrollbar_track: Color::Rgb(20, 26, 22),
            scrollbar_thumb: Color::Rgb(66, 80, 70),
            scrollbar_thumb_hover: Color::Rgb(86, 100, 90),
            disabled_fg: Color::Rgb(100, 115, 100),
            disabled_bg: Color::Rgb(34, 42, 36),
            hover_bg: Color::Rgb(40, 50, 42),
            focus_bg: Color::Rgb(50, 60, 52),
            focus_border: Color::Rgb(126, 196, 102),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Sunset theme — orange coral and pink tones.
    pub fn sunset() -> Self {
        Self {
            name: "sunset".into(),
            display_name: "Sunset".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(32, 24, 26),
            surface: Color::Rgb(40, 30, 32),
            surface_elevated: Color::Rgb(50, 38, 42),
            fg: Color::Rgb(240, 220, 215),
            fg_muted: Color::Rgb(190, 170, 165),
            fg_subtle: Color::Rgb(140, 120, 115),
            fg_on_accent: Color::Rgb(32, 24, 26),
            primary: Color::Rgb(236, 146, 98),
            primary_hover: Color::Rgb(255, 170, 120),
            primary_active: Color::Rgb(216, 126, 78),
            secondary: Color::Rgb(236, 99, 141),
            secondary_hover: Color::Rgb(255, 119, 161),
            secondary_active: Color::Rgb(216, 79, 121),
            outline: Color::Rgb(94, 78, 82),
            outline_variant: Color::Rgb(78, 64, 68),
            divider: Color::Rgb(68, 56, 60),
            error: Color::Rgb(255, 100, 120),
            error_bg: Color::Rgb(50, 15, 20),
            success: Color::Rgb(120, 200, 140),
            success_bg: Color::Rgb(18, 45, 25),
            warning: Color::Rgb(255, 200, 100),
            warning_bg: Color::Rgb(55, 45, 15),
            info: Color::Rgb(236, 146, 98),
            info_bg: Color::Rgb(45, 30, 20),
            selection_bg: Color::Rgb(80, 60, 55),
            selection_fg: Color::Rgb(255, 245, 240),
            input_bg: Color::Rgb(26, 20, 22),
            input_fg: Color::Rgb(240, 220, 215),
            input_border: Color::Rgb(94, 78, 82),
            scrollbar_track: Color::Rgb(26, 20, 22),
            scrollbar_thumb: Color::Rgb(94, 78, 82),
            scrollbar_thumb_hover: Color::Rgb(114, 98, 102),
            disabled_fg: Color::Rgb(120, 105, 100),
            disabled_bg: Color::Rgb(44, 36, 38),
            hover_bg: Color::Rgb(50, 40, 42),
            focus_bg: Color::Rgb(60, 50, 52),
            focus_border: Color::Rgb(236, 146, 98),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }

    /// Creates the Mono theme — soft silver monochrome.
    pub fn mono() -> Self {
        Self {
            name: "mono".into(),
            display_name: "Mono".into(),
            kind: ThemeKind::Dark,
            bg: Color::Rgb(26, 28, 32),
            surface: Color::Rgb(34, 36, 42),
            surface_elevated: Color::Rgb(42, 44, 52),
            fg: Color::Rgb(228, 232, 240),
            fg_muted: Color::Rgb(178, 182, 190),
            fg_subtle: Color::Rgb(128, 132, 140),
            fg_on_accent: Color::Rgb(26, 28, 32),
            primary: Color::Rgb(210, 214, 224),
            primary_hover: Color::Rgb(230, 234, 244),
            primary_active: Color::Rgb(190, 194, 204),
            secondary: Color::Rgb(162, 172, 188),
            secondary_hover: Color::Rgb(182, 192, 208),
            secondary_active: Color::Rgb(142, 152, 168),
            outline: Color::Rgb(72, 78, 92),
            outline_variant: Color::Rgb(60, 66, 78),
            divider: Color::Rgb(52, 58, 68),
            error: Color::Rgb(220, 110, 110),
            error_bg: Color::Rgb(45, 18, 18),
            success: Color::Rgb(140, 200, 140),
            success_bg: Color::Rgb(20, 42, 20),
            warning: Color::Rgb(220, 190, 100),
            warning_bg: Color::Rgb(48, 42, 18),
            info: Color::Rgb(162, 172, 188),
            info_bg: Color::Rgb(25, 32, 42),
            selection_bg: Color::Rgb(62, 66, 78),
            selection_fg: Color::Rgb(240, 242, 250),
            input_bg: Color::Rgb(22, 24, 28),
            input_fg: Color::Rgb(228, 232, 240),
            input_border: Color::Rgb(72, 78, 92),
            scrollbar_track: Color::Rgb(22, 24, 28),
            scrollbar_thumb: Color::Rgb(72, 78, 92),
            scrollbar_thumb_hover: Color::Rgb(92, 98, 112),
            disabled_fg: Color::Rgb(110, 114, 122),
            disabled_bg: Color::Rgb(38, 40, 46),
            hover_bg: Color::Rgb(44, 46, 54),
            focus_bg: Color::Rgb(54, 56, 64),
            focus_border: Color::Rgb(210, 214, 224),
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }
}

#[allow(deprecated)]
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
    pub fn all() -> &'static [Theme] {
        use std::sync::OnceLock;
        static ALL_THEMES: OnceLock<Vec<Theme>> = OnceLock::new();
        ALL_THEMES.get_or_init(|| {
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
        })
    }

    /// Return an iterator over all built-in themes.
    ///
    /// Convenience wrapper around `Theme::all().iter()`.
    pub fn iter() -> std::slice::Iter<'static, Theme> {
        Self::all().iter()
    }

    /// Create a custom theme with the essential fields. All remaining fields
    /// get sensible defaults (muted variants, standard borders, neutral scrollbar).
    ///
    /// Use this instead of struct literals when you need a custom theme
    /// (e.g., for theming demos or app-specific color schemes).
    ///
    /// # Example
    ///
    /// ```ignore
    /// Theme::custom(
    ///     "vscode-dark",
    ///     "VS Code Dark",
    ///     ThemeKind::Dark,
    ///     Color::Rgb(30, 30, 30),  // bg
    ///     Color::Rgb(204, 204, 204), // fg
    ///     Color::Rgb(0, 122, 204),  // primary
    /// )
    /// ```
    pub fn custom(
        name: impl Into<std::sync::Arc<str>>,
        display_name: impl Into<std::sync::Arc<str>>,
        kind: ThemeKind,
        bg: Color,
        fg: Color,
        primary: Color,
    ) -> Self {
        let (bg_r, bg_g, bg_b) = match bg {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Ansi(n) => {
                let (r, g, b) = ansi_to_rgb(n);
                (r, g, b)
            }
            Color::Reset => (0, 0, 0),
        };
        let (fg_r, fg_g, fg_b) = match fg {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Ansi(n) => {
                let (r, g, b) = ansi_to_rgb(n);
                (r, g, b)
            }
            Color::Reset => (204, 204, 204),
        };
        let (primary_r, primary_g, primary_b) = match primary {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Ansi(n) => {
                let (r, g, b) = ansi_to_rgb(n);
                (r, g, b)
            }
            Color::Reset => (0, 122, 204),
        };

        let surface_r = bg_r.saturating_add(12.min((255 - bg_r) / 8));
        let surface_g = bg_g.saturating_add(12.min((255 - bg_g) / 8));
        let surface_b = bg_b.saturating_add(12.min((255 - bg_b) / 8));
        let surface_elevated_r = bg_r.saturating_add(20.min((255 - bg_r) / 6));
        let surface_elevated_g = bg_g.saturating_add(20.min((255 - bg_g) / 6));
        let surface_elevated_b = bg_b.saturating_add(20.min((255 - bg_b) / 6));
        let fg_muted_r = fg_r.saturating_sub((fg_r - bg_r) / 3);
        let fg_muted_g = fg_g.saturating_sub((fg_g - bg_g) / 3);
        let fg_muted_b = fg_b.saturating_sub((fg_b - bg_b) / 3);
        let fg_subtle_r = fg_r.saturating_sub((fg_r - bg_r) / 6);
        let fg_subtle_g = fg_g.saturating_sub((fg_g - bg_g) / 6);
        let fg_subtle_b = fg_b.saturating_sub((fg_b - bg_b) / 6);
        let fg_on_accent = if kind == ThemeKind::Dark {
            Color::Rgb(255, 255, 255)
        } else {
            Color::Rgb(0, 0, 0)
        };
        let primary_hover = Color::Rgb(
            primary_r.saturating_add(16.min((255 - primary_r) / 8)),
            primary_g.saturating_add(16.min((255 - primary_g) / 8)),
            primary_b.saturating_add(16.min((255 - primary_b) / 8)),
        );
        let primary_active = Color::Rgb(
            primary_r.saturating_add(30.min((255 - primary_r) / 4)),
            primary_g.saturating_add(30.min((255 - primary_g) / 4)),
            primary_b.saturating_add(30.min((255 - primary_b) / 4)),
        );
        let secondary = fg;
        let secondary_hover = Color::Rgb(
            fg_r.saturating_add(12.min((255 - fg_r) / 8)),
            fg_g.saturating_add(12.min((255 - fg_g) / 8)),
            fg_b.saturating_add(12.min((255 - fg_b) / 8)),
        );
        let secondary_active = Color::Rgb(
            fg_r.saturating_add(20.min((255 - fg_r) / 6)),
            fg_g.saturating_add(20.min((255 - fg_g) / 6)),
            fg_b.saturating_add(20.min((255 - fg_b) / 6)),
        );
        let (outline_r, outline_g, outline_b) = (
            bg_r.saturating_add(30.min((255 - bg_r) / 4)),
            bg_g.saturating_add(30.min((255 - bg_g) / 4)),
            bg_b.saturating_add(30.min((255 - bg_b) / 4)),
        );
        let outline = Color::Rgb(outline_r, outline_g, outline_b);
        let outline_variant = Color::Rgb(
            bg_r.saturating_add(20.min((255 - bg_r) / 6)),
            bg_g.saturating_add(20.min((255 - bg_g) / 6)),
            bg_b.saturating_add(20.min((255 - bg_b) / 6)),
        );
        let divider = Color::Rgb(
            bg_r.saturating_add(25.min((255 - bg_r) / 5)),
            bg_g.saturating_add(25.min((255 - bg_g) / 5)),
            bg_b.saturating_add(25.min((255 - bg_b) / 5)),
        );
        let error = Color::Rgb(235, 75, 75);
        let error_bg = Color::Rgb(
            bg_r.saturating_add(20),
            (bg_g as u32 * 2 / 3) as u8,
            (bg_b as u32 * 2 / 3) as u8,
        );
        let success = Color::Rgb(73, 201, 73);
        let success_bg = Color::Rgb(20, 50, 20);
        let warning = Color::Rgb(227, 180, 60);
        let warning_bg = Color::Rgb(55, 45, 20);
        let info = primary;
        let info_bg = Color::Rgb(
            bg_r.saturating_add(10),
            bg_g.saturating_add(10),
            bg_b.saturating_add(15),
        );
        let selection_bg = Color::Rgb(
            bg_r.saturating_add(primary_r / 4),
            bg_g.saturating_add(primary_g / 4),
            bg_b.saturating_add(primary_b / 4),
        );
        let selection_fg = fg_on_accent;
        let input_bg = Color::Rgb(surface_r, surface_g, surface_b);
        let input_fg = fg;
        let input_border = outline;
        let scrollbar_track = Color::Rgb(
            bg_r.saturating_add(10),
            bg_g.saturating_add(10),
            bg_b.saturating_add(10),
        );
        let scrollbar_thumb = Color::Rgb(
            outline_r.saturating_add(30.min((255 - outline_r) / 4)),
            outline_g.saturating_add(30.min((255 - outline_g) / 4)),
            outline_b.saturating_add(30.min((255 - outline_b) / 4)),
        );
        let scrollbar_thumb_hover = outline;
        let disabled_fg = Color::Rgb(fg_muted_r, fg_muted_g, fg_muted_b);
        let disabled_bg = Color::Rgb(surface_r, surface_g, surface_b);
        let hover_bg = Color::Rgb(surface_r, surface_g, surface_b);
        let focus_bg = Color::Rgb(surface_elevated_r, surface_elevated_g, surface_elevated_b);
        let focus_border = primary;

        Self {
            name: name.into(),
            display_name: display_name.into(),
            kind,
            bg,
            surface: Color::Rgb(surface_r, surface_g, surface_b),
            surface_elevated: Color::Rgb(surface_elevated_r, surface_elevated_g, surface_elevated_b),
            fg,
            fg_muted: Color::Rgb(fg_muted_r, fg_muted_g, fg_muted_b),
            fg_subtle: Color::Rgb(fg_subtle_r, fg_subtle_g, fg_subtle_b),
            fg_on_accent,
            primary,
            primary_hover,
            primary_active,
            secondary,
            secondary_hover,
            secondary_active,
            outline,
            outline_variant,
            divider,
            error,
            error_bg,
            success,
            success_bg,
            warning,
            warning_bg,
            info,
            info_bg,
            selection_bg,
            selection_fg,
            input_bg,
            input_fg,
            input_border,
            scrollbar_track,
            scrollbar_thumb,
            scrollbar_thumb_hover,
            disabled_fg,
            disabled_bg,
            hover_bg,
            focus_bg,
            focus_border,
            scrollbar_width: Self::default_scrollbar_width(),
        }
    }
}

fn ansi_to_rgb(n: u8) -> (u8, u8, u8) {
    match n {
        // Standard colors (0-7)
        0 => (0, 0, 0),
        1 => (205, 0, 0),
        2 => (0, 205, 0),
        3 => (205, 205, 0),
        4 => (0, 0, 205),
        5 => (205, 0, 205),
        6 => (0, 205, 205),
        7 => (229, 229, 229),
        // Bright colors (8-15)
        8 => (127, 127, 127),
        9 => (255, 0, 0),
        10 => (0, 255, 0),
        11 => (255, 255, 0),
        12 => (0, 0, 255),
        13 => (255, 0, 255),
        14 => (0, 255, 255),
        15 => (255, 255, 255),
        // Grayscale ramp (232-255)
        232..=255 => {
            let gray = (n - 232) * 10 + 8;
            (gray, gray, gray)
        }
        // 6×6×6 color cube (16-231)
        _ => {
            let idx = n - 16;
            let r = ((idx / 36) % 6) * 51;
            let g = ((idx / 6) % 6) * 51;
            let b = (idx % 6) * 51;
            (r, g, b)
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
