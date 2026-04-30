//! Progress bar widget.
//!
//! A horizontal progress bar showing completion percentage.

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A progress bar widget.
pub struct ProgressBar {
    id: WidgetId,
    progress: f32,
    theme: Theme,
    area: std::cell::Cell<Rect>,
}

impl ProgressBar {
    /// Creates a new progress bar with the given id.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            progress: 0.0,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 1)),
        }
    }

    /// Sets the theme for this progress bar.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the progress value (0.0 to 1.0).
    pub fn set_progress(&mut self, value: f32) {
        self.progress = value.clamp(0.0, 1.0);
    }

    /// Returns the current progress value.
    pub fn progress(&self) -> f32 {
        self.progress
    }
}

impl crate::framework::widget::Widget for ProgressBar {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;
        let height = plane.height as usize;

        let fill_width = (self.progress * width as f32).round() as usize;
        let fill_width = fill_width.min(width.saturating_sub(2));

        for x in 1..fill_width + 1 {
            let idx = ((height / 2) as u16 * plane.width + x as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: ' ',
                    fg: Color::Reset,
                    bg: self.theme.accent,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        let left_bracket = Cell {
            char: '[',
            fg: self.theme.fg,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        let right_bracket = Cell {
            char: ']',
            fg: self.theme.fg,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };

        let left_idx = ((height / 2) as u16 * plane.width) as usize;
        let right_idx = ((height / 2) as u16 * plane.width + (width - 1) as u16) as usize;
        if left_idx < plane.cells.len() {
            plane.cells[left_idx] = left_bracket;
        }
        if right_idx < plane.cells.len() {
            plane.cells[right_idx] = right_bracket;
        }

        plane
    }
}