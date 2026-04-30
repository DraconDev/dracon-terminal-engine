//! Checkbox widget.
//!
//! A checkbox is a two-state on/off control with a check mark.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A checkbox widget.
pub struct Checkbox {
    id: WidgetId,
    checked: bool,
    label: String,
    theme: Theme,
    on_change: Option<Box<dyn FnMut(bool)>>,
    area: std::cell::Cell<Rect>,
}

impl Checkbox {
    /// Creates a new checkbox with the given id and label.
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            checked: false,
            label: label.to_string(),
            theme: Theme::default(),
            on_change: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 20, 1)),
        }
    }

    /// Sets the theme for this checkbox.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the callback for when the checkbox state changes.
    pub fn on_change(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Checks the checkbox.
    pub fn check(&mut self) {
        self.checked = true;
    }

    /// Unchecks the checkbox.
    pub fn uncheck(&mut self) {
        self.checked = false;
    }

    /// Toggles the checkbox state.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    /// Returns whether the checkbox is checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }
}

impl crate::framework::widget::Widget for Checkbox {
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

        let check_str = if self.checked { "[x]" } else { "[ ]" };
        let full_text = format!("{} {}", check_str, self.label);

        let cell_width = full_text.width().min(width);
        let start_x = (width.saturating_sub(cell_width)) / 2;
        let start_y = height.saturating_sub(1) / 2;

        let fg = if self.checked {
            self.theme.success_fg
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
                self.toggle();
                if let Some(ref mut cb) = self.on_change {
                    cb(self.checked);
                }
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
                    cb(self.checked);
                }
                true
            }
            _ => false,
        }
    }
}