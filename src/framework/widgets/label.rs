//! Label widget for static text display.

use crate::compositor::{Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A widget that displays static text.
pub struct Label {
    id: WidgetId,
    text: String,
    theme: Theme,
    style: Styles,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Label {
    /// Creates a new Label with the given text.
    pub fn new(text: &str) -> Self {
        Self {
            id: WidgetId::default_id(),
            text: text.to_string(),
            theme: Theme::default(),
            style: Styles::empty(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 1)),
            dirty: true,
        }
    }

    /// Creates a new Label with the given ID and text.
    pub fn with_id(id: WidgetId, text: &str) -> Self {
        Self {
            id,
            text: text.to_string(),
            theme: Theme::default(),
            style: Styles::empty(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 1)),
            dirty: true,
        }
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the text style (e.g., bold, italic).
    pub fn with_style(mut self, style: Styles) -> Self {
        self.style = style;
        self
    }

    /// Sets the area for this label.
    pub fn with_area(self, area: Rect) -> Self {
        self.area.set(area);
        self
    }

    /// Updates the label text.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl crate::framework::widget::Widget for Label {
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

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn focusable(&self) -> bool {
        false
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 0;

        for (i, c) in self.text.chars().take(area.width as usize).enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx] = crate::compositor::Cell {
                    char: c,
                    fg: self.theme.fg,
                    bg: self.theme.bg,
                    style: self.style,
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }
}
