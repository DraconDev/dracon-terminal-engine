//! Form widget for grouping labeled input fields.
//!
//! A vertical layout of labeled form fields with validation support.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A single field within a form with a label, value, and optional error.
pub struct FormField {
    /// The label text displayed before the input.
    pub label: String,
    /// The current value of the field.
    pub value: String,
    /// An optional error message shown below the field.
    pub error: Option<String>,
}

/// A form widget that displays a vertical list of labeled input fields.
pub struct Form {
    id: WidgetId,
    fields: Vec<FormField>,
    focused_field: usize,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Form {
    /// Creates a new form with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            fields: Vec::new(),
            focused_field: 0,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 10)),
            dirty: true,
        }
    }

    /// Adds a new field with the given label and returns self for chaining.
    pub fn add_field(mut self, label: &str) -> Self {
        self.fields.push(FormField {
            label: label.to_string(),
            value: String::new(),
            error: None,
        });
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the value of a field by index.
    pub fn set_field_value(&mut self, index: usize, value: &str) {
        if let Some(ref mut field) = self.fields.get_mut(index) {
            field.value = value.to_string();
            self.dirty = true;
        }
    }

    /// Sets an error message on a field by index.
    pub fn set_field_error(&mut self, index: usize, error: &str) {
        if let Some(ref mut field) = self.fields.get_mut(index) {
            field.error = Some(error.to_string());
            self.dirty = true;
        }
    }
}

impl crate::framework::widget::Widget for Form {
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

        for (i, field) in self.fields.iter().enumerate() {
            if i >= plane.height as usize {
                break;
            }
            let is_focused = i == self.focused_field;
            let label_text = format!("{}: ", field.label);
            let value_text = if field.value.is_empty() {
                "_".to_string()
            } else {
                field.value.clone()
            };

            let fg = if is_focused {
                self.theme.primary
            } else {
                self.theme.fg
            };
            let err_fg = self.theme.error;

            for (j, c) in label_text.chars().take(width).enumerate() {
                let idx = (i as u16 * plane.width + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg: self.theme.bg,
                        style: Styles::BOLD,
                        transparent: false,
                        skip: false,
                    };
                }
            }

            let value_start = label_text.width();
            for (j, c) in value_text
                .chars()
                .take(width.saturating_sub(value_start))
                .enumerate()
            {
                let idx = (i as u16 * plane.width + (value_start + j) as u16) as usize;
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

            if let Some(ref error) = field.error {
                for (j, c) in error
                    .chars()
                    .take(width.saturating_sub(value_start))
                    .enumerate()
                {
                    let idx = (i as u16 * plane.width + (value_start + j) as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = Cell {
                            char: c,
                            fg: err_fg,
                            bg: self.theme.bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
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
            KeyCode::Down => {
                if self.focused_field < self.fields.len().saturating_sub(1) {
                    self.focused_field += 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Up => {
                if self.focused_field > 0 {
                    self.focused_field -= 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Char(ch) => {
                if let Some(ref mut field) = self.fields.get_mut(self.focused_field) {
                    field.value.push(ch);
                    self.dirty = true;
                }
                true
            }
            KeyCode::Backspace => {
                if let Some(ref mut field) = self.fields.get_mut(self.focused_field) {
                    field.value.pop();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                if let Some(ref mut field) = self.fields.get_mut(self.focused_field) {
                    field.value.clear();
                    self.dirty = true;
                }
                true
            }
_ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        _kind: crate::input::event::MouseEventKind,
        _col: u16,
        row: u16,
    ) -> bool {
        if (row as usize) < self.fields.len() {
            self.focused_field = row as usize;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}
