use crate::compositor::plane::{Cell, Color};

/// Applies a visual effect to a cell during compositing.
pub trait Filter {
    /// Applies the filter effect to `cell` at position `(x, y)` at the given time.
    fn apply(&self, cell: &mut Cell, x: u16, y: u16, time: f32);
}

fn dim_rgb(r: u8, g: u8, b: u8, factor: f32) -> Color {
    Color::Rgb(
        (r as f32 * factor).clamp(0.0, 255.0) as u8,
        (g as f32 * factor).clamp(0.0, 255.0) as u8,
        (b as f32 * factor).clamp(0.0, 255.0) as u8,
    )
}

fn dim_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => dim_rgb(r, g, b, factor),
        Color::Ansi(c) => {
            if c > 8 {
                Color::Ansi(8)
            } else {
                color
            }
        }
        Color::Reset => color,
    }
}

/// Dims colors by a given factor.
pub struct Dim {
    /// The dimming factor applied to colors (0.0 to 1.0).
    pub factor: f32,
}

impl Default for Dim {
    fn default() -> Self {
        Self { factor: 0.5 }
    }
}

impl Filter for Dim {
    fn apply(&self, cell: &mut Cell, _x: u16, _y: u16, _time: f32) {
        cell.fg = dim_color(cell.fg, self.factor);
        cell.bg = dim_color(cell.bg, self.factor);
    }
}

/// Swaps foreground and background colors, creating an inverted appearance.
pub struct Invert;

impl Filter for Invert {
    fn apply(&self, cell: &mut Cell, _x: u16, _y: u16, _time: f32) {
        std::mem::swap(&mut cell.fg, &mut cell.bg);
    }
}

/// Applies scanline effect by dimming every other row.
pub struct Scanline;

impl Filter for Scanline {
    fn apply(&self, cell: &mut Cell, _x: u16, y: u16, _time: f32) {
        if y.is_multiple_of(2) {
            cell.fg = dim_color(cell.fg, 0.8);
            cell.bg = dim_color(cell.bg, 0.8);
        }
    }
}

/// Pulses foreground brightness over time using a sine wave.
pub struct Pulse;

impl Filter for Pulse {
    fn apply(&self, cell: &mut Cell, _x: u16, _y: u16, time: f32) {
        let factor = (time.sin() * 0.2 + 0.8).clamp(0.0, 1.0);
        cell.fg = dim_color(cell.fg, factor);
    }
}

/// Applies glitch effect with random character corruption and row distortion.
pub struct Glitch;

impl Filter for Glitch {
    fn apply(&self, cell: &mut Cell, x: u16, y: u16, time: f32) {
        let seed = (x as f32 * 12.9898 + y as f32 * 78.233 + time).sin() * 43_758.547;
        let rand = seed - seed.floor();

        if rand > 0.98 {
            cell.char = if rand > 0.99 { '█' } else { '░' };
            cell.fg = Color::Rgb(255, 0, 85);
        } else if rand > 0.95 {
            let shift = (time * 10.0).sin() * 5.0;
            if (y as f32 - shift.abs()).abs() < 1.0 {
                cell.style.insert(crate::compositor::plane::Styles::REVERSE);
            }
        }
    }
}