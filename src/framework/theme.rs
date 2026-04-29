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
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}