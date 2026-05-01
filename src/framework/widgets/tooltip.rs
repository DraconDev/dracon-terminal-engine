//! Tooltip widget for displaying contextual help text.
//!
//! A small overlay that appears near a widget with descriptive text.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A tooltip overlay that displays informational text.
pub struct Tooltip {
    id: WidgetId,
    text: String,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Tooltip {
    /// Creates a new tooltip with the given ID and text.
    pub fn new(id: WidgetId, text: &str) -> Self {
        Self {
            id,
            text: text.to_string(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 3)),
            dirty: true,
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Returns the tooltip text.
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl crate::framework::widget::Widget for Tooltip {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        100
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 100;

        let lines: Vec<&str> = self.text.lines().collect();
        let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);
        let width = max_len.min(area.width as usize).max(1);
        let height = lines.len().min(area.height as usize).max(1);

        for (i, line) in lines.iter().take(height).enumerate() {
            for (j, c) in line.chars().take(width).enumerate() {
                let idx = (i as u16 * plane.width + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: self.theme.bg,
                        bg: self.theme.fg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        plane
    }
}
