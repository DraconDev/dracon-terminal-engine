use crate::compositor::plane::{Cell, Color, Plane, Styles};
use std::io::{self, Write};

/// Composites multiple planes into a single render target.
pub struct Compositor {
    /// The planes to composite, ordered by z-index.
    pub planes: Vec<Plane>,
    width: u16,
    height: u16,
    last_frame: Vec<Cell>,
}

impl Compositor {
    /// Creates a new Compositor with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            planes: Vec::new(),
            width,
            height,
            last_frame: vec![Cell::default(); (width as u32 * height as u32) as usize],
        }
    }

    /// Advances the compositor state by one frame.
    pub fn tick(&mut self, _delta: f32) {
    }

    /// Returns the topmost visible plane at the given coordinates, if any.
    pub fn hit_test(&self, x: u16, y: u16) -> Option<&Plane> {
        for plane in self.planes.iter().rev() {
            if !plane.visible {
                continue;
            }
            if x >= plane.x
                && x < plane.x.saturating_add(plane.width)
                && y >= plane.y
                && y < plane.y.saturating_add(plane.height)
            {
                let lx = x - plane.x;
                let ly = y - plane.y;
                let idx = (ly * plane.width + lx) as usize;
                if !plane.cells[idx].transparent {
                    return Some(plane);
                }
            }
        }
        None
    }

    /// Returns the width and height of the compositor.
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Adds a plane to the compositor, inserting it at the correct z-index position.
    pub fn add_plane(&mut self, plane: Plane) {
        self.planes.push(plane);
        self.sort_planes();
    }

    /// Draws text at the specified position with the given colors and style.
    pub fn draw_text(&mut self, text: &str, x: u16, y: u16, fg: Color, bg: Color, style: Styles) {
        let mut plane = Plane::new(0, text.len() as u16, 1);
        plane.x = x;
        plane.y = y;
        plane.z_index = 10;

        for (i, c) in text.chars().enumerate() {
            if i < plane.cells.len() {
                plane.cells[i] = Cell {
                    char: c,
                    fg,
                    bg,
                    style,
                    transparent: false,
                    skip: false,
                };
            }
        }
        self.add_plane(plane);
    }

    /// Draws a filled rectangle at the specified position with the given character, colors, and style.
    pub fn draw_rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        char: char,
        fg: Color,
        bg: Color,
        style: Styles,
    ) {
        let mut plane = Plane::new(0, width, height);
        plane.x = x;
        plane.y = y;
        plane.z_index = 5;

        let cell = Cell {
            char,
            fg,
            bg,
            style,
            transparent: false,
            skip: false,
        };

        for i in 0..plane.cells.len() {
            plane.cells[i] = cell.clone();
        }
        self.add_plane(plane);
    }

    /// Clears the terminal and resets the internal frame buffer.
    pub fn force_clear(&mut self) {
        if let Some(base) = self.planes.first_mut() {
            base.clear();
        }
        for cell in &mut self.last_frame {
            cell.char = '\x01';
        }
    }

    /// Draws a ratatui Line at the specified position.
    pub fn draw_ratatui_line(&mut self, line: &ratatui::text::Line, x: u16, y: u16) {
        let total_len: usize = line.spans.iter().map(|s| s.content.len()).sum();
        if total_len == 0 {
            return;
        }

        let mut plane = Plane::new(0, total_len as u16, 1);
        plane.x = x;
        plane.y = y;
        plane.z_index = 10;

        let mut idx = 0;
        for span in &line.spans {
            let fg = map_color(span.style.fg.unwrap_or(ratatui::style::Color::Reset));
            let bg = map_color(span.style.bg.unwrap_or(ratatui::style::Color::Reset));
            let mut style = crate::compositor::plane::Styles::empty();
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::BOLD)
            {
                style.insert(crate::compositor::plane::Styles::BOLD);
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::ITALIC)
            {
                style.insert(crate::compositor::plane::Styles::ITALIC);
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::UNDERLINED)
            {
                style.insert(crate::compositor::plane::Styles::UNDERLINE);
            }

            for c in span.content.chars() {
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg,
                        style,
                        transparent: false,
                        skip: false,
                    };
                    idx += 1;
                }
            }
        }
        self.add_plane(plane);
    }

    /// Resizes the compositor to the given dimensions, resetting the frame buffer.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.last_frame = vec![Cell::default(); (width * height) as usize];
    }

    fn sort_planes(&mut self) {
        self.planes.sort_by_key(|a| a.z_index);
    }

    /// Renders the compositor state to the given writer, outputting terminal escape codes.
    pub fn render<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let mut final_buffer = vec![
            Cell {
                bg: Color::Rgb(0, 0, 0),
                transparent: false,
                ..Cell::default()
            };
            (self.width * self.height) as usize
        ];

        let mut layers = Vec::new();
        for i in 0..self.planes.len() {
            layers.push((self.planes[i].z_index, i));
        }
        layers.sort_by_key(|a| a.0);

        for (_, plane_idx) in layers {
            let plane = &self.planes[plane_idx];
            if !plane.visible {
                continue;
            }
            for py in 0..plane.height {
                for px in 0..plane.width {
                    let abs_x = plane.x.saturating_add(px);
                    let abs_y = plane.y.saturating_add(py);
                    if abs_x >= self.width || abs_y >= self.height {
                        continue;
                    }

                    let src_idx = (py * plane.width + px) as usize;
                    let dest_idx = (abs_y * self.width + abs_x) as usize;
                    let mut src_cell = plane.cells[src_idx].clone();

                    if let Some(filter) = &plane.filter {
                        filter.apply(&mut src_cell, abs_x, abs_y, 0.0);
                    }

                    blend_cells(&mut final_buffer[dest_idx], &src_cell, plane.opacity);
                }
            }
        }

        write!(writer, "\x1b[?2026h")?;

        let mut current_fg = Color::Reset;
        let mut current_bg = Color::Reset;
        let mut current_style = Styles::empty();

        write!(writer, "\x1b[?7l")?;

        for y in 0..self.height {
            let mut line_cursor_moved = false;
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                let cell = &final_buffer[idx];
                let last_cell = &self.last_frame[idx];

                if cell.skip {
                    continue;
                }

                if cell == last_cell {
                    line_cursor_moved = false;
                    continue;
                }

                if !line_cursor_moved {
                    write!(writer, "\x1b[{};{}H", y + 1, x + 1)?;
                    line_cursor_moved = true;
                }

                if cell.style != current_style {
                    let diff = cell.style ^ current_style;
                    if diff.contains(Styles::BOLD) {
                        if cell.style.contains(Styles::BOLD) {
                            write!(writer, "\x1b[1m")?;
                        } else {
                            write!(writer, "\x1b[22m")?;
                        }
                    }
                    if diff.contains(Styles::ITALIC) {
                        if cell.style.contains(Styles::ITALIC) {
                            write!(writer, "\x1b[3m")?;
                        } else {
                            write!(writer, "\x1b[23m")?;
                        }
                    }
                    if diff.contains(Styles::UNDERLINE) {
                        if cell.style.contains(Styles::UNDERLINE) {
                            write!(writer, "\x1b[4m")?;
                        } else {
                            write!(writer, "\x1b[24m")?;
                        }
                    }
                    current_style = cell.style;
                }

                if cell.fg != current_fg {
                    match cell.fg {
                        Color::Reset => write!(writer, "\x1b[39m")?,
                        Color::Ansi(c) => write!(writer, "\x1b[38;5;{}m", c)?,
                        Color::Rgb(r, g, b) => write!(writer, "\x1b[38;2;{};{};{}m", r, g, b)?,
                    }
                    current_fg = cell.fg;
                }
                if cell.bg != current_bg {
                    match cell.bg {
                        Color::Reset => write!(writer, "\x1b[49m")?,
                        Color::Ansi(c) => write!(writer, "\x1b[48;5;{}m", c)?,
                        Color::Rgb(r, g, b) => write!(writer, "\x1b[48;2;{};{};{}m", r, g, b)?,
                    }
                    current_bg = cell.bg;
                }
                write!(writer, "{}", cell.char)?;
            }
        }

        write!(writer, "\x1b[?7h")?;
        write!(writer, "\x1b[?2026l")?;

        self.last_frame = final_buffer;
        writer.flush()?;
        Ok(())
    }
}

fn blend_cells(dest: &mut Cell, src: &Cell, alpha: f32) {
    if src.transparent || alpha <= 0.0 {
        return;
    }

    if alpha >= 1.0 {
        if src.bg != Color::Reset {
            dest.bg = src.bg;
        }

        if src.skip {
            dest.skip = true;
            dest.char = ' ';
        } else if src.char != '\0' {
            if is_braille(dest.char) && is_braille(src.char) {
                dest.char = merge_braille(dest.char, src.char);
            } else {
                dest.char = src.char;
            }
            dest.fg = src.fg;
            dest.style = src.style;
            dest.skip = false;
        }
    } else {
        let blend = |c1: Color, c2: Color, a: f32| -> Color {
            match (c1, c2) {
                (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => Color::Rgb(
                    ((r1 as f32 * (1.0 - a)) + (r2 as f32 * a)) as u8,
                    ((g1 as f32 * (1.0 - a)) + (g2 as f32 * a)) as u8,
                    ((b1 as f32 * (1.0 - a)) + (b2 as f32 * a)) as u8,
                ),
                (_, c) => {
                    if a > 0.5 {
                        c
                    } else {
                        c1
                    }
                }
            }
        };

        if src.bg != Color::Reset {
            dest.bg = blend(dest.bg, src.bg, alpha);
        }

        if src.skip {
            if alpha > 0.5 {
                dest.skip = true;
                dest.char = ' ';
            }
        } else if src.char != '\0' {
            dest.fg = blend(dest.fg, src.fg, alpha);

            if alpha > 0.5 {
                dest.char = src.char;
                dest.style = src.style;
                dest.skip = false;
            }
        }
    }

    dest.transparent = false;
}

fn is_braille(c: char) -> bool {
    let u = c as u32;
    (0x2800..=0x28FF).contains(&u)
}

fn merge_braille(c1: char, c2: char) -> char {
    let b1 = (c1 as u32) & 0xFF;
    let b2 = (c2 as u32) & 0xFF;
    std::char::from_u32(0x2800 | (b1 | b2)).unwrap_or(c1)
}

/// Converts a ratatui Color into a compositor Color.
pub fn map_color(c: ratatui::style::Color) -> Color {
    use ratatui::style::Color as RColor;
    match c {
        RColor::Reset => Color::Reset,
        RColor::Black => Color::Rgb(0, 0, 0),
        RColor::Red => Color::Rgb(255, 0, 85),
        RColor::Green => Color::Rgb(0, 255, 150),
        RColor::Yellow => Color::Rgb(255, 255, 0),
        RColor::Blue => Color::Rgb(0, 150, 255),
        RColor::Magenta => Color::Rgb(255, 0, 255),
        RColor::Cyan => Color::Rgb(0, 255, 200),
        RColor::Gray => Color::Rgb(180, 180, 180),
        RColor::DarkGray => Color::Rgb(60, 60, 70),
        RColor::LightRed => Color::Rgb(255, 100, 100),
        RColor::LightGreen => Color::Rgb(100, 255, 100),
        RColor::LightYellow => Color::Rgb(255, 255, 150),
        RColor::LightBlue => Color::Rgb(150, 150, 255),
        RColor::LightMagenta => Color::Rgb(255, 150, 255),
        RColor::LightCyan => Color::Rgb(150, 255, 255),
        RColor::White => Color::Rgb(255, 255, 255),
        RColor::Indexed(i) => Color::Ansi(i),
        RColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}