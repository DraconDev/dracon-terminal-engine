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

/// A color scheme defining the visual appearance of the terminal UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// The theme's display name.
    pub name: &'static str,
    /// Background color.
    pub bg: Color,
    /// Foreground (text) color.
    pub fg: Color,
    /// Accent color for highlights and emphasis.
    pub accent: Color,
    /// Selection background color.
    pub selection_bg: Color,
    /// Selection foreground (text) color.
    pub selection_fg: Color,
    /// Border color.
    pub border: Color,
    /// Scrollbar track color.
    pub scrollbar_track: Color,
    /// Scrollbar thumb (handle) color.
    pub scrollbar_thumb: Color,
    /// Background color for hoverable/interactive elements.
    pub hover_bg: Color,
    /// Background color for active/pressed elements.
    pub active_bg: Color,
    /// Foreground color for inactive/disabled elements.
    pub inactive_fg: Color,
    /// Input field background color.
    pub input_bg: Color,
    /// Input field foreground (text) color.
    pub input_fg: Color,
    /// Width of scrollbars in pixels.
    pub scrollbar_width: u16,
    /// Foreground color for error states (validation failures, critical alerts).
    pub error_fg: Color,
    /// Foreground color for success states (confirmations, completed actions).
    pub success_fg: Color,
    /// Foreground color for warning states (cautions, important notices).
    pub warning_fg: Color,
    /// Foreground color for disabled/inactive interactive elements.
    pub disabled_fg: Color,
}

impl Theme {
    /// Creates a dark theme with muted colors suitable for low-light environments.
    pub fn dark() -> Self {
        Self {
            name: "dark",
            bg: Color::Rgb(16, 16, 24),
            fg: Color::Rgb(200, 200, 220),
            accent: Color::Rgb(0, 200, 120),
            selection_bg: Color::Rgb(50, 80, 60),
            selection_fg: Color::Rgb(200, 255, 220),
            border: Color::Rgb(60, 60, 80),
            scrollbar_track: Color::Rgb(30, 30, 40),
            scrollbar_thumb: Color::Rgb(80, 80, 100),
            hover_bg: Color::Rgb(30, 30, 45),
            active_bg: Color::Rgb(40, 40, 60),
            inactive_fg: Color::Rgb(100, 100, 120),
            input_bg: Color::Rgb(20, 20, 30),
            input_fg: Color::Rgb(220, 220, 240),
            scrollbar_width: 1,
            error_fg: Color::Rgb(255, 80, 80),
            success_fg: Color::Rgb(80, 255, 120),
            warning_fg: Color::Rgb(255, 180, 80),
            disabled_fg: Color::Rgb(80, 80, 100),
        }
    }

    /// Creates a light theme with high contrast suitable for bright environments.
    pub fn light() -> Self {
        Self {
            name: "light",
            bg: Color::Rgb(250, 250, 250),
            fg: Color::Rgb(30, 30, 40),
            accent: Color::Rgb(0, 120, 180),
            selection_bg: Color::Rgb(180, 220, 240),
            selection_fg: Color::Rgb(0, 0, 0),
            border: Color::Rgb(180, 180, 180),
            scrollbar_track: Color::Rgb(220, 220, 220),
            scrollbar_thumb: Color::Rgb(150, 150, 150),
            hover_bg: Color::Rgb(220, 220, 235),
            active_bg: Color::Rgb(200, 200, 220),
            inactive_fg: Color::Rgb(150, 150, 150),
            input_bg: Color::Rgb(255, 255, 255),
            input_fg: Color::Rgb(30, 30, 40),
            scrollbar_width: 1,
            error_fg: Color::Rgb(200, 40, 40),
            success_fg: Color::Rgb(40, 160, 40),
            warning_fg: Color::Rgb(200, 140, 40),
            disabled_fg: Color::Rgb(150, 150, 150),
        }
    }

    /// Creates a cyberpunk-themed dark theme with neon green and pink accents.
    pub fn cyberpunk() -> Self {
        Self {
            name: "cyberpunk",
            bg: Color::Rgb(0, 0, 0),
            fg: Color::Rgb(0, 255, 136),
            accent: Color::Rgb(255, 0, 100),
            selection_bg: Color::Rgb(0, 50, 30),
            selection_fg: Color::Rgb(0, 255, 200),
            border: Color::Rgb(0, 200, 100),
            scrollbar_track: Color::Rgb(0, 30, 20),
            scrollbar_thumb: Color::Rgb(0, 150, 80),
            hover_bg: Color::Rgb(0, 40, 25),
            active_bg: Color::Rgb(0, 60, 40),
            inactive_fg: Color::Rgb(0, 100, 60),
            input_bg: Color::Rgb(10, 10, 20),
            input_fg: Color::Rgb(0, 255, 136),
            scrollbar_width: 1,
            error_fg: Color::Rgb(255, 0, 80),
            success_fg: Color::Rgb(0, 255, 180),
            warning_fg: Color::Rgb(255, 200, 0),
            disabled_fg: Color::Rgb(0, 80, 50),
        }
    }

    /// Creates the Dracula theme — iconic dark purple aesthetic with vivid accents.
    pub fn dracula() -> Self {
        Self {
            name: "dracula",
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(98, 114, 164),
            selection_bg: Color::Rgb(68, 71, 90),
            selection_fg: Color::Rgb(255, 255, 255),
            border: Color::Rgb(98, 114, 164),
            scrollbar_track: Color::Rgb(30, 32, 42),
            scrollbar_thumb: Color::Rgb(68, 71, 90),
            hover_bg: Color::Rgb(50, 52, 64),
            active_bg: Color::Rgb(60, 62, 74),
            inactive_fg: Color::Rgb(139, 233, 253),
            input_bg: Color::Rgb(30, 32, 42),
            input_fg: Color::Rgb(248, 248, 242),
            scrollbar_width: 1,
            error_fg: Color::Rgb(255, 85, 85),
            success_fg: Color::Rgb(80, 250, 123),
            warning_fg: Color::Rgb(241, 250, 140),
            disabled_fg: Color::Rgb(68, 71, 90),
        }
    }

    /// Creates the Nord theme — arctic blue-gray palette.
    pub fn nord() -> Self {
        Self {
            name: "nord",
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(216, 222, 233),
            accent: Color::Rgb(136, 192, 208),
            selection_bg: Color::Rgb(67, 76, 94),
            selection_fg: Color::Rgb(236, 240, 243),
            border: Color::Rgb(67, 76, 94),
            scrollbar_track: Color::Rgb(35, 40, 52),
            scrollbar_thumb: Color::Rgb(67, 76, 94),
            hover_bg: Color::Rgb(56, 61, 73),
            active_bg: Color::Rgb(66, 71, 84),
            inactive_fg: Color::Rgb(146, 155, 168),
            input_bg: Color::Rgb(35, 40, 52),
            input_fg: Color::Rgb(216, 222, 233),
            scrollbar_width: 1,
            error_fg: Color::Rgb(191, 97, 106),
            success_fg: Color::Rgb(163, 190, 140),
            warning_fg: Color::Rgb(235, 203, 139),
            disabled_fg: Color::Rgb(119, 128, 144),
        }
    }

    /// Creates the Catppuccin Mocha theme — warm, soothing pastel dark theme.
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin-mocha",
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            accent: Color::Rgb(137, 180, 250),
            selection_bg: Color::Rgb(49, 50, 68),
            selection_fg: Color::Rgb(230, 233, 244),
            border: Color::Rgb(88, 91, 112),
            scrollbar_track: Color::Rgb(24, 24, 37),
            scrollbar_thumb: Color::Rgb(88, 91, 112),
            hover_bg: Color::Rgb(40, 40, 57),
            active_bg: Color::Rgb(50, 50, 68),
            inactive_fg: Color::Rgb(108, 112, 134),
            input_bg: Color::Rgb(24, 24, 37),
            input_fg: Color::Rgb(205, 214, 244),
            scrollbar_width: 1,
            error_fg: Color::Rgb(243, 139, 168),
            success_fg: Color::Rgb(166, 227, 161),
            warning_fg: Color::Rgb(249, 226, 175),
            disabled_fg: Color::Rgb(108, 112, 134),
        }
    }

    /// Creates the Gruvbox Dark theme — retro warm dark theme with earthy tones.
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "gruvbox-dark",
            bg: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(213, 196, 161),
            accent: Color::Rgb(214, 93, 14),
            selection_bg: Color::Rgb(100, 70, 40),
            selection_fg: Color::Rgb(235, 219, 178),
            border: Color::Rgb(120, 90, 60),
            scrollbar_track: Color::Rgb(30, 30, 30),
            scrollbar_thumb: Color::Rgb(100, 70, 40),
            hover_bg: Color::Rgb(55, 45, 35),
            active_bg: Color::Rgb(70, 55, 40),
            inactive_fg: Color::Rgb(150, 130, 100),
            input_bg: Color::Rgb(30, 30, 30),
            input_fg: Color::Rgb(213, 196, 161),
            scrollbar_width: 1,
            error_fg: Color::Rgb(204, 36, 36),
            success_fg: Color::Rgb(152, 151, 26),
            warning_fg: Color::Rgb(215, 153, 33),
            disabled_fg: Color::Rgb(120, 90, 60),
        }
    }

    /// Creates the Tokyo Night theme — vivid blue accents on a dark background.
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night",
            bg: Color::Rgb(32, 34, 44),
            fg: Color::Rgb(192, 202, 245),
            accent: Color::Rgb(98, 130, 234),
            selection_bg: Color::Rgb(52, 54, 70),
            selection_fg: Color::Rgb(202, 212, 254),
            border: Color::Rgb(62, 64, 82),
            scrollbar_track: Color::Rgb(22, 24, 34),
            scrollbar_thumb: Color::Rgb(62, 64, 82),
            hover_bg: Color::Rgb(42, 44, 54),
            active_bg: Color::Rgb(52, 54, 70),
            inactive_fg: Color::Rgb(113, 117, 138),
            input_bg: Color::Rgb(22, 24, 34),
            input_fg: Color::Rgb(192, 202, 245),
            scrollbar_width: 1,
            error_fg: Color::Rgb(255, 85, 85),
            success_fg: Color::Rgb(166, 227, 161),
            warning_fg: Color::Rgb(255, 184, 108),
            disabled_fg: Color::Rgb(113, 117, 138),
        }
    }

    /// Creates the Solarized Dark theme — precision-engineered dark theme.
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark",
            bg: Color::Rgb(0, 43, 54),
            fg: Color::Rgb(131, 148, 150),
            accent: Color::Rgb(38, 139, 210),
            selection_bg: Color::Rgb(0, 60, 76),
            selection_fg: Color::Rgb(147, 161, 161),
            border: Color::Rgb(0, 80, 100),
            scrollbar_track: Color::Rgb(0, 33, 44),
            scrollbar_thumb: Color::Rgb(0, 80, 100),
            hover_bg: Color::Rgb(0, 53, 66),
            active_bg: Color::Rgb(0, 66, 82),
            inactive_fg: Color::Rgb(88, 110, 117),
            input_bg: Color::Rgb(0, 33, 44),
            input_fg: Color::Rgb(131, 148, 150),
            scrollbar_width: 1,
            error_fg: Color::Rgb(220, 50, 47),
            success_fg: Color::Rgb(133, 153, 0),
            warning_fg: Color::Rgb(181, 137, 0),
            disabled_fg: Color::Rgb(88, 110, 117),
        }
    }

    /// Creates the Solarized Light theme — precision-engineered light theme.
    pub fn solarized_light() -> Self {
        Self {
            name: "solarized-light",
            bg: Color::Rgb(253, 246, 227),
            fg: Color::Rgb(101, 123, 131),
            accent: Color::Rgb(38, 139, 210),
            selection_bg: Color::Rgb(181, 209, 240),
            selection_fg: Color::Rgb(0, 43, 54),
            border: Color::Rgb(147, 161, 161),
            scrollbar_track: Color::Rgb(253, 246, 227),
            scrollbar_thumb: Color::Rgb(147, 161, 161),
            hover_bg: Color::Rgb(250, 243, 224),
            active_bg: Color::Rgb(247, 240, 221),
            inactive_fg: Color::Rgb(147, 161, 161),
            input_bg: Color::Rgb(253, 246, 227),
            input_fg: Color::Rgb(101, 123, 131),
            scrollbar_width: 1,
            error_fg: Color::Rgb(220, 50, 47),
            success_fg: Color::Rgb(133, 153, 0),
            warning_fg: Color::Rgb(181, 137, 0),
            disabled_fg: Color::Rgb(147, 161, 161),
        }
    }

    /// Creates the One Dark theme — Atom editor's iconic dark theme.
    pub fn one_dark() -> Self {
        Self {
            name: "one-dark",
            bg: Color::Rgb(40, 44, 52),
            fg: Color::Rgb(220, 223, 228),
            accent: Color::Rgb(97, 175, 239),
            selection_bg: Color::Rgb(62, 66, 76),
            selection_fg: Color::Rgb(230, 233, 239),
            border: Color::Rgb(62, 66, 76),
            scrollbar_track: Color::Rgb(30, 34, 42),
            scrollbar_thumb: Color::Rgb(62, 66, 76),
            hover_bg: Color::Rgb(50, 54, 62),
            active_bg: Color::Rgb(60, 64, 74),
            inactive_fg: Color::Rgb(124, 131, 141),
            input_bg: Color::Rgb(30, 34, 42),
            input_fg: Color::Rgb(220, 223, 228),
            scrollbar_width: 1,
            error_fg: Color::Rgb(224, 108, 108),
            success_fg: Color::Rgb(152, 195, 121),
            warning_fg: Color::Rgb(229, 192, 120),
            disabled_fg: Color::Rgb(124, 131, 141),
        }
    }

    /// Creates the Rosé Pine theme — elegant muted rose tones.
    pub fn rose_pine() -> Self {
        Self {
            name: "rose-pine",
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(220, 200, 200),
            accent: Color::Rgb(210, 160, 160),
            selection_bg: Color::Rgb(50, 45, 65),
            selection_fg: Color::Rgb(230, 210, 210),
            border: Color::Rgb(60, 55, 75),
            scrollbar_track: Color::Rgb(24, 24, 37),
            scrollbar_thumb: Color::Rgb(60, 55, 75),
            hover_bg: Color::Rgb(40, 37, 52),
            active_bg: Color::Rgb(50, 45, 65),
            inactive_fg: Color::Rgb(150, 130, 150),
            input_bg: Color::Rgb(24, 24, 37),
            input_fg: Color::Rgb(220, 200, 200),
            scrollbar_width: 1,
            error_fg: Color::Rgb(210, 160, 160),
            success_fg: Color::Rgb(204, 170, 140),
            warning_fg: Color::Rgb(230, 200, 160),
            disabled_fg: Color::Rgb(100, 90, 100),
        }
    }

    /// Creates the Kanagawa theme — inspired by Hokusai's art with deep blues and golds.
    pub fn kanagawa() -> Self {
        Self {
            name: "kanagawa",
            bg: Color::Rgb(38, 40, 64),
            fg: Color::Rgb(220, 217, 201),
            accent: Color::Rgb(166, 122, 102),
            selection_bg: Color::Rgb(60, 62, 90),
            selection_fg: Color::Rgb(230, 225, 205),
            border: Color::Rgb(88, 90, 112),
            scrollbar_track: Color::Rgb(28, 30, 54),
            scrollbar_thumb: Color::Rgb(88, 90, 112),
            hover_bg: Color::Rgb(48, 50, 74),
            active_bg: Color::Rgb(58, 60, 84),
            inactive_fg: Color::Rgb(128, 125, 115),
            input_bg: Color::Rgb(28, 30, 54),
            input_fg: Color::Rgb(220, 217, 201),
            scrollbar_width: 1,
            error_fg: Color::Rgb(219, 98, 98),
            success_fg: Color::Rgb(166, 206, 122),
            warning_fg: Color::Rgb(230, 186, 122),
            disabled_fg: Color::Rgb(100, 95, 110),
        }
    }

    /// Creates the Everforest theme — comfortable forest green dark theme.
    pub fn everforest() -> Self {
        Self {
            name: "everforest",
            bg: Color::Rgb(43, 48, 40),
            fg: Color::Rgb(210, 191, 163),
            accent: Color::Rgb(148, 181, 97),
            selection_bg: Color::Rgb(60, 68, 55),
            selection_fg: Color::Rgb(222, 211, 179),
            border: Color::Rgb(80, 90, 70),
            scrollbar_track: Color::Rgb(33, 38, 30),
            scrollbar_thumb: Color::Rgb(80, 90, 70),
            hover_bg: Color::Rgb(53, 58, 50),
            active_bg: Color::Rgb(63, 68, 60),
            inactive_fg: Color::Rgb(130, 140, 115),
            input_bg: Color::Rgb(33, 38, 30),
            input_fg: Color::Rgb(210, 191, 163),
            scrollbar_width: 1,
            error_fg: Color::Rgb(220, 110, 110),
            success_fg: Color::Rgb(148, 181, 97),
            warning_fg: Color::Rgb(230, 180, 110),
            disabled_fg: Color::Rgb(100, 110, 90),
        }
    }

    /// Creates the Monokai theme — classic syntax highlighting colors.
    pub fn monokai() -> Self {
        Self {
            name: "monokai",
            bg: Color::Rgb(39, 40, 34),
            fg: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(102, 217, 239),
            selection_bg: Color::Rgb(80, 75, 60),
            selection_fg: Color::Rgb(250, 250, 250),
            border: Color::Rgb(100, 95, 80),
            scrollbar_track: Color::Rgb(29, 30, 24),
            scrollbar_thumb: Color::Rgb(100, 95, 80),
            hover_bg: Color::Rgb(49, 50, 44),
            active_bg: Color::Rgb(60, 55, 50),
            inactive_fg: Color::Rgb(150, 145, 130),
            input_bg: Color::Rgb(29, 30, 24),
            input_fg: Color::Rgb(248, 248, 242),
            scrollbar_width: 1,
            error_fg: Color::Rgb(249, 38, 114),
            success_fg: Color::Rgb(166, 226, 50),
            warning_fg: Color::Rgb(230, 200, 80),
            disabled_fg: Color::Rgb(130, 125, 110),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
