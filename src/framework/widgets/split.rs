//! Split pane widget.

use crate::compositor::{Color, Plane, Styles};
use ratatui::layout::Rect;

/// The direction in which a split pane divides the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Split left to right.
    Horizontal,
    /// Split top to bottom.
    Vertical,
}

/// A widget that splits a rectangular area into two panes with a configurable ratio.
pub struct SplitPane {
    ratio: f32,
    orientation: Orientation,
    divider_char: char,
    min_size: u16,
}

impl SplitPane {
    /// Creates a new `SplitPane` in the given orientation with a 50/50 split.
    pub fn new(orientation: Orientation) -> Self {
        Self {
            ratio: 0.5,
            orientation,
            divider_char: '│',
            min_size: 10,
        }
    }

    /// Creates a new `SplitPane` inferring orientation from the aspect ratio of `rect`.
    /// Horizontal if width >= height, otherwise vertical.
    pub fn from_rect(rect: Rect) -> Self {
        let orientation = if rect.width >= rect.height {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        };
        Self {
            ratio: 0.5,
            orientation,
            divider_char: '│',
            min_size: 10,
        }
    }

    /// Sets the split ratio (0.1–0.9, default 0.5).
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.1, 0.9);
        self
    }

    /// Sets the divider character (default '│').
    pub fn with_divider(mut self, c: char) -> Self {
        self.divider_char = c;
        self
    }

    /// Sets the minimum size in cells for each pane (default 10).
    pub fn with_min_size(mut self, size: u16) -> Self {
        self.min_size = size;
        self
    }

    /// Splits `area` into two `Rect`s according to the current ratio and orientation.
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

    /// Returns the `Rect` occupied by the divider line between the two panes.
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

    /// Renders the divider as a `Plane` styled with the divider character.
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

    /// Handles a mouse drag event to interactively resize the split ratio.
    ///
    /// Returns `true` if the event was consumed.
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