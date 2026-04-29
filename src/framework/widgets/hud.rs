use crate::compositor::{Cell, Color, Plane, Styles};

pub struct Hud {
    z_index: i32,
    visible: bool,
    width: u16,
    height: u16,
}

impl Hud {
    pub fn new(z_index: i32) -> Self {
        Self {
            z_index,
            visible: true,
            width: 30,
            height: 10,
        }
    }

    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn position(&self) -> (u16, u16) {
        (0, 0)
    }

    pub fn z_index(&self) -> i32 {
        self.z_index
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn render_text(&self, x: u16, y: u16, text: &str, fg: Color, bg: Color) -> Plane {
        let mut plane = Plane::new(0, self.width, self.height);
        plane.z_index = self.z_index;

        let text_len = text.len().min(self.width as usize - x as usize);
        let start_idx = (y * self.width + x) as usize;

        for (i, ch) in text.chars().take(text_len).enumerate() {
            let idx = start_idx + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: ch,
                    fg,
                    bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }

    pub fn render_gauge(&self, x: u16, y: u16, label: &str, value: f32, max: f32, width: u16) -> Plane {
        let mut plane = Plane::new(0, self.width, self.height);
        plane.z_index = self.z_index;

        let label_len = label.len().min(width as usize);
        let start_idx = (y * self.width + x) as usize;

        for (i, ch) in label.chars().take(label_len).enumerate() {
            let idx = start_idx + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = Color::Rgb(200, 200, 220);
                plane.cells[idx].bg = Color::Reset;
            }
        }

        let bar_start = start_idx + label_len + 1;
        let filled = if max > 0.0 {
            ((value / max) * (width as f32 - label_len as f32 - 2.0).max(0.0)) as usize
        } else {
            0
        };

        for i in 0..(width as usize - label_len - 2) {
            let idx = bar_start + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if i < filled { '█' } else { '░' };
                plane.cells[idx].fg = if i < filled {
                    Color::Rgb(0, 200, 120)
                } else {
                    Color::Rgb(60, 60, 80)
                };
                plane.cells[idx].bg = Color::Reset;
            }
        }

        plane
    }
}