//! Button widget for clickable actions.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A clickable button widget.
pub struct Button {
    id: WidgetId,
    label: String,
    theme: Theme,
    on_click: Option<Box<dyn FnMut()>>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Button {
    /// Creates a new Button with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            id: WidgetId::default_id(),
            label: label.to_string(),
            theme: Theme::default(),
            on_click: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 10, 1)),
            dirty: true,
        }
    }

    /// Creates a new Button with the given ID and label.
    pub fn with_id(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_string(),
            theme: Theme::default(),
            on_click: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 10, 1)),
            dirty: true,
        }
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the click callback.
    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl crate::framework::widget::Widget for Button {
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

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 0;

        let display = if self.label.is_empty() {
            "Button"
        } else {
            &self.label
        };
        let max_width = area.width.saturating_sub(2);

        plane.cells[0] = Cell {
            char: '[',
            fg: self.theme.fg,
            bg: self.theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };

        for (i, c) in display.chars().take(max_width as usize).enumerate() {
            let idx = 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: self.theme.fg,
                    bg: self.theme.bg,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        let end_idx = 1 + display.width().min(max_width as usize);
        if end_idx < plane.cells.len() {
            plane.cells[end_idx] = Cell {
                char: ']',
                fg: self.theme.fg,
                bg: self.theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
            if let Some(ref mut cb) = self.on_click {
                cb();
            }
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        let area = self.area.get();
        if col >= area.x && col < area.x + area.width && row >= area.y && row < area.y + area.height
        {
            if let crate::input::event::MouseEventKind::Down(_) = kind {
                if let Some(ref mut cb) = self.on_click {
                    cb();
                }
                self.dirty = true;
                return true;
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}
