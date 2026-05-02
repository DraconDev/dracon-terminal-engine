//! Radio button widget.
//!
//! A radio button for mutually exclusive selection within a group.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A radio button widget.
pub struct Radio {
    id: WidgetId,
    selected: bool,
    label: String,
    theme: Theme,
    on_change: Option<Box<dyn FnMut(bool)>>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Radio {
    /// Creates a new radio button with the given id and label.
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            selected: false,
            label: label.to_string(),
            theme: Theme::default(),
            on_change: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 20, 1)),
            dirty: true,
        }
    }

    /// Sets the theme for this radio button.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the callback for when the radio button state changes.
    pub fn on_change(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Selects the radio button.
    pub fn select(&mut self) {
        self.selected = true;
        self.dirty = true;
    }

    /// Deselects the radio button.
    pub fn deselect(&mut self) {
        self.selected = false;
        self.dirty = true;
    }

    /// Returns whether the radio button is selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

impl crate::framework::widget::Widget for Radio {
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

        let radio_str = if self.selected { "(o)" } else { "( )" };
        let full_text = format!("{} {}", radio_str, self.label);

        let cell_width = full_text.width().min(width);
        let start_x = (width.saturating_sub(cell_width)) / 2;
        let start_y = height.saturating_sub(1) / 2;

        let fg = if self.selected {
            self.theme.primary
        } else {
            self.theme.fg
        };

        for (i, c) in full_text.chars().take(width).enumerate() {
            let idx = (start_y as u16 * plane.width + (start_x as u16 + i as u16)) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg,
                    bg: self.theme.bg,
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
                if !self.selected {
                    self.selected = true;
                    if let Some(ref mut cb) = self.on_change {
                        cb(true);
                    }
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        _col: u16,
        _row: u16,
    ) -> bool {
        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                if !self.selected {
                    self.selected = true;
                    if let Some(ref mut cb) = self.on_change {
                        cb(true);
                    }
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}
