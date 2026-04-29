use crate::compositor::{Cell, Color, Plane, Styles};
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct SplitPane {
    ratio: f32,
    orientation: Orientation,
    divider_char: char,
    min_size: u16,
}

impl SplitPane {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            ratio: 0.5,
            orientation,
            divider_char: '│',
            min_size: 10,
        }
    }

    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.1, 0.9);
        self
    }

    pub fn with_divider(mut self, c: char) -> Self {
        self.divider_char = c;
        self
    }

    pub fn with_min_size(mut self, size: u16) -> Self {
        self.min_size = size;
        self
    }

    pub fn split(&self, area: Rect) -> (Rect, Rect) {
        match self.orientation {
            Orientation::Horizontal => {
                let w1 = ((area.width as f32 * self.ratio).round() as u16).max(self.min_size);
                let w2 = area.width.saturating_sub(w1).max(self.min_size);
                let w1 = area.width.saturating_sub(w2);
                (Rect::new(area.x, area.y, w1, area.height),
                 Rect::new(area.x + w1, area.y, w2, area.height))
            }
            Orientation::Vertical => {
                let h1 = ((area.height as f32 * self.ratio).round() as u16).max(self.min_size);
                let h2 = area.height.saturating_sub(h1).max(self.min_size);
                let h1 = area.height.saturating_sub(h2);
                (Rect::new(area.x, area.y, area.width, h1),
                 Rect::new(area.x, area.y + h1, area.width, h2))
            }
        }
    }

    pub fn divider_rect(&self, area: Rect) -> Rect {
        match self.orientation {
            Orientation::Horizontal => {
                let w1 = (area.width as f32 * self.ratio).round() as u16;
                Rect::new(area.x + w1, area.y, 1, area.height)
            }
            Orientation::Vertical => {
                let h1 = (area.height as f32 * self.ratio).round() as u16;
                Rect::new(area.x, area.y + h1, area.width, 1)
            }
        }
    }

    pub fn render_divider(&self, area: Rect) -> Plane {
        let rect = self.divider_rect(area);
        let mut plane = Plane::new(0, rect.width, rect.height);
        plane.x = rect.x;
        plane.y = rect.y;

        for cell in &mut plane.cells {
            cell.char = self.divider_char;
            cell.fg = Color::Rgb(80, 80, 100);
            cell.bg = Color::Reset;
            cell.style = Styles::empty();
            cell.transparent = false;
            cell.skip = false;
        }

        plane
    }

    pub fn handle_resize(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16, area: Rect) -> bool {
        match kind {
            crate::input::event::MouseEventKind::Drag(_) => {
                match self.orientation {
                    Orientation::Horizontal => {
                        let total_w = area.width as f32;
                        self.ratio = (col as f32 / total_w).clamp(0.1, 0.9);
                    }
                    Orientation::Vertical => {
                        let total_h = area.height as f32;
                        self.ratio = (row as f32 / total_h).clamp(0.1, 0.9);
                    }
                }
                true
            }
            _ => false,
        }
    }
}