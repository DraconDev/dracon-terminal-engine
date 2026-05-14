//! Form widget for grouping labeled input fields.
//!
//! A vertical layout of labeled form fields with validation support.

use regex::Regex;
use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A validation rule for form field values.
pub enum ValidationRule {
    /// Field must not be empty.
    Required,
    /// Minimum character count.
    MinLength(usize),
    /// Maximum character count.
    MaxLength(usize),
    /// Basic email format validation.
    Email,
    /// Regex pattern match.
    Regex(String),
    /// Custom validator returning an error message if invalid.
    Custom(ValidatorFn),
}

/// Callback for custom validation.
pub type ValidatorFn = Box<dyn Fn(&str) -> Option<String>>;

impl ValidationRule {
    /// Validates a value against this rule, returning an error message if invalid.
    pub fn validate(&self, value: &str) -> Option<String> {
        match self {
            ValidationRule::Required => {
                if value.trim().is_empty() {
                    Some("This field is required".to_string())
                } else {
                    None
                }
            }
            ValidationRule::MinLength(min) => {
                if value.chars().count() < *min {
                    Some(format!("Must be at least {} characters", min))
                } else {
                    None
                }
            }
            ValidationRule::MaxLength(max) => {
                if value.chars().count() > *max {
                    Some(format!("Must be at most {} characters", max))
                } else {
                    None
                }
            }
            ValidationRule::Email => {
                let parts: Vec<&str> = value.split('@').collect();
                if parts.len() == 2
                    && !parts[0].is_empty()
                    && parts[1].contains('.')
                    && !parts[1].starts_with('.')
                    && !parts[1].ends_with('.')
                {
                    None
                } else {
                    Some("Invalid email format".to_string())
                }
            }
            ValidationRule::Regex(pattern) => match Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(value) {
                        None
                    } else {
                        Some("Does not match required pattern".to_string())
                    }
                }
                Err(_) => Some("Invalid validation pattern".to_string()),
            },
            ValidationRule::Custom(validator) => validator(value),
        }
    }
}

/// A single field within a form with a label, value, and optional error.
pub struct FormField {
    /// The label text displayed before the input.
    pub label: String,
    /// The current value of the field.
    pub value: String,
    /// An optional error message shown below the field.
    pub error: Option<String>,
    /// Validation rules for this field.
    pub rules: Vec<ValidationRule>,
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
            rules: Vec::new(),
        });
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets validation rules for a field by index.
    pub fn with_validation(mut self, index: usize, rules: Vec<ValidationRule>) -> Self {
        if let Some(ref mut field) = self.fields.get_mut(index) {
            field.rules = rules;
        }
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

    /// Validates all fields and returns all errors as (field_index, message) tuples.
    pub fn validate(&mut self) -> Result<(), Vec<(usize, String)>> {
        let mut errors = Vec::new();
        for i in 0..self.fields.len() {
            if let Some(error) = self.validate_field(i) {
                errors.push((i, error));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validates a single field by index, setting its error if invalid.
    pub fn validate_field(&mut self, index: usize) -> Option<String> {
        let value = self.fields.get(index)?.value.clone();

        let mut error = None;
        if let Some(field) = self.fields.get(index) {
            for rule in &field.rules {
                if let Some(err) = rule.validate(&value) {
                    error = Some(err);
                    break;
                }
            }
        }

        if let Some(f) = self.fields.get_mut(index) {
            f.error = error.clone();
            self.dirty = true;
        }

        error
    }

    /// Clears the error for a field by index.
    pub fn clear_field_error(&mut self, index: usize) {
        if let Some(ref mut field) = self.fields.get_mut(index) {
            field.error = None;
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
        plane.fill_bg(self.theme.bg);

        let width = plane.cells.len() / plane.height as usize;

        let mut row = 0u16;
        for (i, field) in self.fields.iter().enumerate() {
            if row >= plane.height {
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
            let bg = if is_focused {
                self.theme.focus_bg
            } else {
                self.theme.bg
            };
            let err_fg = self.theme.error;

            // Fill the entire row background first
            for x in 0..width {
                let idx = (row * plane.width + x as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                }
            }

            for (j, c) in label_text.chars().take(width).enumerate() {
                let idx = (row * plane.width + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg,
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
                let idx = (row * plane.width + (value_start + j) as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }

            if let Some(ref error) = field.error {
                if row + 1 < plane.height {
                    let error_row = row + 1;
                    // Fill error row background
                    for x in 0..width {
                        let idx = (error_row * plane.width + x as u16) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].bg = bg;
                        }
                    }
                    // Render error text with small indent
                    let indent = 2usize;
                    for (j, c) in error
                        .chars()
                        .take(width.saturating_sub(indent))
                        .enumerate()
                    {
                        let idx =
                            (error_row * plane.width + (indent + j) as u16) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx] = Cell {
                                char: c,
                                fg: err_fg,
                                bg,
                                style: Styles::empty(),
                                transparent: false,
                                skip: false,
                            };
                        }
                    }
                }
                row += 2;
            } else {
                row += 1;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind, KeyModifiers};
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
            KeyCode::Tab => {
                self.validate_field(self.focused_field);
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    if self.focused_field > 0 {
                        self.focused_field -= 1;
                    }
                } else {
                    if self.focused_field < self.fields.len().saturating_sub(1) {
                        self.focused_field += 1;
                    }
                }
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                self.validate_field(self.focused_field);
                self.dirty = true;
                true
            }
            KeyCode::Char(ch) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => {
                if let Some(ref mut field) = self.fields.get_mut(self.focused_field) {
                    field.value.push(ch);
                    field.error = None;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Backspace => {
                if let Some(ref mut field) = self.fields.get_mut(self.focused_field) {
                    field.value.pop();
                    field.error = None;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                if let Some(ref mut _field) = self.fields.get_mut(self.focused_field) {
                    // Home should move cursor to start in future cursor-tracking impl
                    // For now, no-op (was incorrectly clearing the field)
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
        let mut current_row = 0u16;
        for (i, field) in self.fields.iter().enumerate() {
            let height = if field.error.is_some() { 2 } else { 1 };
            if row >= current_row && row < current_row + height {
                self.focused_field = i;
                self.dirty = true;
                return true;
            }
            current_row += height;
        }
        false
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
    }
}

impl crate::framework::widget::WidgetState for Form {
    fn state_id(&self) -> Option<&str> {
        Some("form")
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        let field_values: Vec<serde_json::Value> = self
            .fields
            .iter()
            .map(|f| json!({"label": f.label, "value": f.value}))
            .collect();
        json!({
            "focused_field": self.focused_field,
            "fields": field_values,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(focused) = json.get("focused_field").and_then(|v| v.as_u64()) {
            self.focused_field = focused as usize;
        }
        if let Some(fields) = json.get("fields").and_then(|v| v.as_array()) {
            for (i, field_json) in fields.iter().enumerate() {
                if let (Some(value), Some(_field)) = (
                    field_json.get("value").and_then(|v| v.as_str()),
                    self.fields.get_mut(i),
                ) {
                    _field.value = value.to_string();
                }
            }
        }
        self.dirty = true;
        Ok(())
    }
}
