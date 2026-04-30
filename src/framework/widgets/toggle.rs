//! Toggle switch widget.
//!
//! A toggle switch is a two-state on/off control.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A toggle switch widget.
pub struct Toggle {
    id: WidgetId,
    state: bool,
    label: String,
    theme: Theme,
    on_change: Option<Box<dyn FnMut(bool)>>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Toggle {
    /// Creates a new toggle switch with the given id and label.
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            state: false,
            label: label.to_string(),
            theme: Theme::default(),
            on_change: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 20, 1)),
            dirty: true,
        }
    }

    /// Sets the theme for this toggle switch.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the callback for when the toggle state changes.
    pub fn on_change(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Toggles the switch state.
    pub fn toggle(&mut self) {
        self.state = !self.state;
        self.dirty = true;
    }

    /// Returns whether the toggle is on.
    pub fn is_on(&self) -> bool {
        self.state
    }
}

impl crate::framework::widget::Widget for Toggle {
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
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;
        let height = plane.height as usize;

        let on_text = if self.state { "[*]" } else { "[ ]" };
        let full_text = format!("{} {}", on_text, self.label);

        let cell_width = full_text.width().min(width);
        let start_x = (width.saturating_sub(cell_width)) / 2;
        let start_y = height.saturating_sub(1) / 2;

        let bg = if self.state {
            self.theme.success_fg
        } else {
            self.theme.inactive_fg
        };

        for (i, c) in full_text.chars().take(width).enumerate() {
            let idx = (start_y as u16 * plane.width + (start_x as u16 + i as u16)) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                        char: c,
                        fg: self.theme.fg,
                        bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Enter => {
                self.toggle();
                if let Some(ref mut cb) = self.on_change {
                    cb(self.state);
                }
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, _col: u16, _row: u16) -> bool {
        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                self.toggle();
                if let Some(ref mut cb) = self.on_change {
                    cb(self.state);
                }
                self.dirty = true;
                true
            }
            _ => false,
        }
    }
}