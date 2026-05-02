//! Color themes for terminal UI.
//!
//! Provides [`crate::framework::theme::Theme`] with 15 built-in themes: `dark`, `light`, `cyberpunk`,
//! `dracula`, `nord`, `catppuccin_mocha`, `gruvbox_dark`, `tokyo_night`,
//! `solarized_dark`, `solarized_light`, `one_dark`, `rose_pine`, `kanagawa`,
//! `everforest`, `monokai`.
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
            scrollbar_width: 1,
        }
    }

    /// Creates a cyberpunk-themed dark theme with neon green and hot pink accents.
    pub fn cyberpunk() -> Self {
        Self {
            name: "cyberpunk",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Dracula theme — iconic dark purple aesthetic with vivid accents.
    pub fn dracula() -> Self {
        Self {
            name: "dracula",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Nord theme — arctic blue-gray palette.
    pub fn nord() -> Self {
        Self {
            name: "nord",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Catppuccin Mocha theme — warm, soothing pastel dark theme.
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin-mocha",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Gruvbox Dark theme — retro warm dark theme with earthy tones.
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "gruvbox-dark",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Tokyo Night theme — vivid blue accents on a dark background.
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Solarized Dark theme — precision-engineered dark theme.
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Solarized Light theme — precision-engineered light theme.
    pub fn solarized_light() -> Self {
        Self {
            name: "solarized-light",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the One Dark theme — Atom editor's iconic dark theme.
    pub fn one_dark() -> Self {
        Self {
            name: "one-dark",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Rosé Pine theme — elegant muted rose tones.
    pub fn rose_pine() -> Self {
        Self {
            name: "rose-pine",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Kanagawa theme — inspired by Hokusai's art with deep blues and golds.
    pub fn kanagawa() -> Self {
        Self {
            name: "kanagawa",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Everforest theme — comfortable forest green dark theme.
    pub fn everforest() -> Self {
        Self {
            name: "everforest",
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
            scrollbar_width: 1,
        }
    }

    /// Creates the Monokai theme — classic syntax highlighting colors.
    pub fn monokai() -> Self {
        Self {
            name: "monokai",
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
            scrollbar_width: 1,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}