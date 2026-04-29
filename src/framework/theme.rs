use crate::compositor::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border: Color,
    pub scrollbar_track: Color,
    pub scrollbar_thumb: Color,
    pub hover_bg: Color,
    pub active_bg: Color,
    pub inactive_fg: Color,
    pub input_bg: Color,
    pub input_fg: Color,
    pub scrollbar_width: u16,
}

impl Theme {
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