//! Debug overlay widget for rendering diagnostics.
//!
//! A transparent overlay showing internal state for debugging.

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A debug overlay displaying key-value diagnostic pairs.
pub struct DebugOverlay {
    /// The widget ID for this overlay.
    id: WidgetId,
    /// The lines of debug text to display.
    lines: Vec<String>,
    /// The theme for this widget.
    theme: Theme,
    area: std::cell::Cell<Rect>,
}

impl DebugOverlay {
    /// Creates a new debug overlay with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            lines: Vec::new(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 60, 20)),
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Adds a line of debug text.
    pub fn add_line(&mut self, line: &str) {
        self.lines.push(line.to_string());
    }

    /// Sets all debug lines at once.
    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.lines = lines;
    }

    /// Clears all debug lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

impl crate::framework::widget::Widget for DebugOverlay {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn z_index(&self) -> u16 {
        200
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 200;

        let width = plane.cells.len() / plane.height as usize;

        for (i, line) in self.lines.iter().take(area.height as usize).enumerate() {
            for (j, c) in line.chars().take(width).enumerate() {
                let idx = (i as u16 * plane.width + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: self.theme.error_fg,
                        bg: Color::Reset,
                        style: Styles::empty(),
                        transparent: true,
                        skip: false,
                    };
                }
            }
        }

        plane
    }
}