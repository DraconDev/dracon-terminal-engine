//! Form Demo — demonstrates the framework's form widget with validation.
//!
//! This example shows:
//! - A `Form` widget containing multiple field types
//! - Focus cycling via `FocusManager` (Tab/Shift+Tab)
//! - Field validation with inline error messages
//! - Submit handling with validation of all fields
//! - Keyboard navigation (Enter to advance, Escape to clear)
//! - Success feedback via `Toast`
//!
//! # Fields
//!
//! | Field | Type | Validation |
//! |-------|------|------------|
//! | Username | TextInput | non-empty |
//! | Email | TextInput | must contain "@" |
//! | Password | PasswordInput | min 8 characters |
//! | Theme | Select | Dark/Light/Cyberpunk |
//! | Notifications | Toggle | on/off |
//! | Submit | Button | validates all fields |
//!
//! # Key Patterns
//!
//! 1. **Composition**: The `SettingsForm` struct wraps multiple widgets and
//!    implements `Widget` to compose them into a single form.
//!
//! 2. **Focus cycling**: `FocusManager::tab_next()` / `tab_prev()` cycle focus
//!    through the field widgets.
//!
//! 3. **Validation**: Each field validates on blur or submit. Errors are stored
//!    in the form and rendered inline below invalid fields.
//!
//! 4. **Submit flow**: Button click triggers validation of all fields. If valid,
//!    shows a success toast. If invalid, shows error states.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Button, PasswordInput, SearchInput, Select, Toast, ToastKind, Toggle,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

/// Field indices for focus management.
const FIELD_USERNAME: usize = 0;
const FIELD_EMAIL: usize = 1;
const FIELD_PASSWORD: usize = 2;
const FIELD_THEME: usize = 3;
const FIELD_NOTIFICATIONS: usize = 4;
const FIELD_SUBMIT: usize = 5;
const FIELD_COUNT: usize = 6;

/// Validation error messages.
struct ValidationErrors {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self {
            username: None,
            email: None,
            password: None,
        }
    }
}

/// A settings form widget composed of multiple input widgets.
struct SettingsForm {
    id: WidgetId,
    username: SearchInput,
    email: SearchInput,
    password: PasswordInput,
    theme: Select,
    notifications: Toggle,
    submit: Button,
    focused_field: usize,
    errors: ValidationErrors,
    show_toast: bool,
    toast_message: String,
    area: Rect,
    dirty: bool,
}

impl SettingsForm {
    fn new(id: WidgetId) -> Self {
        let theme = Select::new(WidgetId::new(id.0 + 3))
            .with_options(vec![
                "Dark".to_string(),
                "Light".to_string(),
                "Cyberpunk".to_string(),
            ]);

        Self {
            id,
            username: SearchInput::new(WidgetId::new(id.0 + 1)),
            email: SearchInput::new(WidgetId::new(id.0 + 2)),
            password: PasswordInput::new(WidgetId::new(id.0 + 3)),
            theme,
            notifications: Toggle::new(WidgetId::new(id.0 + 4), "Enable notifications"),
            submit: Button::with_id(WidgetId::new(id.0 + 5), "Save Settings"),
            focused_field: FIELD_USERNAME,
            errors: ValidationErrors::default(),
            show_toast: false,
            toast_message: String::new(),
            area: Rect::new(0, 0, 60, 12),
            dirty: true,
        }
    }

    fn validate_field(&mut self, field: usize) -> bool {
        match field {
            FIELD_USERNAME => {
                if self.username.query().is_empty() {
                    self.errors.username = Some("Username is required".to_string());
                    false
                } else {
                    self.errors.username = None;
                    true
                }
            }
            FIELD_EMAIL => {
                let email = self.email.query();
                if email.is_empty() {
                    self.errors.email = Some("Email is required".to_string());
                    false
                } else if !email.contains('@') {
                    self.errors.email = Some("Email must contain @".to_string());
                    false
                } else {
                    self.errors.email = None;
                    true
                }
            }
            FIELD_PASSWORD => {
                let pwd = self.password.password();
                if pwd.len() < 8 {
                    self.errors.password = Some("Password must be at least 8 characters".to_string());
                    false
                } else {
                    self.errors.password = None;
                    true
                }
            }
            _ => true,
        }
    }

    fn validate_all(&mut self) -> bool {
        let v1 = self.validate_field(FIELD_USERNAME);
        let v2 = self.validate_field(FIELD_EMAIL);
        let v3 = self.validate_field(FIELD_PASSWORD);
        v1 && v2 && v3
    }

    fn clear_form(&mut self) {
        self.username.clear();
        self.email.clear();
        self.password.clear();
        self.errors = ValidationErrors::default();
        self.focused_field = FIELD_USERNAME;
        self.dirty = true;
    }

    fn focus_next(&mut self) {
        self.validate_field(self.focused_field);
        self.focused_field = (self.focused_field + 1) % FIELD_COUNT;
        self.dirty = true;
    }

    fn focus_prev(&mut self) {
        self.validate_field(self.focused_field);
        self.focused_field = if self.focused_field == 0 {
            FIELD_COUNT - 1
        } else {
            self.focused_field - 1
        };
        self.dirty = true;
    }

    fn get_current_widget_mut(&mut self) -> Option<&mut dyn Widget> {
        match self.focused_field {
            FIELD_USERNAME => Some(&mut self.username),
            FIELD_EMAIL => Some(&mut self.email),
            FIELD_PASSWORD => Some(&mut self.password),
            FIELD_THEME => Some(&mut self.theme),
            FIELD_NOTIFICATIONS => Some(&mut self.notifications),
            FIELD_SUBMIT => Some(&mut self.submit),
            _ => None,
        }
    }
}

impl Widget for SettingsForm {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
        self.dirty = true;
    }

    fn focusable(&self) -> bool {
        true
    }

    fn z_index(&self) -> u16 {
        10
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.username.needs_render() || self.email.needs_render()
            || self.password.needs_render() || self.theme.needs_render()
            || self.notifications.needs_render() || self.submit.needs_render()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
        self.username.clear_dirty();
        self.email.clear_dirty();
        self.password.clear_dirty();
        self.theme.clear_dirty();
        self.notifications.clear_dirty();
        self.submit.clear_dirty();
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = self.z_index();

        let left_col = 15u16;
        let input_width = 40u16;
        let error_offset = 30u16;

        let mut y = 0u16;

        plane.cells[0].char = 'U';
        for (i, c) in "Username: ".chars().take(left_col as usize).enumerate() {
            let idx = (y * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.focused_field == FIELD_USERNAME {
                    self.theme.accent
                } else {
                    self.theme.fg
                };
                plane.cells[idx].bg = self.theme.bg;
            }
        }
        let username_plane = self.username.render(Rect::new(left_col, y, input_width, 1));
        for cell in &username_plane.cells[..username_plane.cells.len().min(plane.cells.len())] {
            let idx = (y * plane.width + left_col + cell.char as usize) as usize;
            if idx < plane.cells.len() && cell.char != '\0' {
                plane.cells[idx] = *cell;
            }
        }
        if let Some(ref err) = self.errors.username {
            for (i, c) in err.chars().take(input_width as usize).enumerate() {
                let idx = (y * plane.width + left_col + 2 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.error_fg;
                }
            }
        }

        y += 2;

        for (i, c) in "Email: ".chars().take(left_col as usize).enumerate() {
            let idx = (y * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.focused_field == FIELD_EMAIL {
                    self.theme.accent
                } else {
                    self.theme.fg
                };
                plane.cells[idx].bg = self.theme.bg;
            }
        }
        let email_plane = self.email.render(Rect::new(left_col, y, input_width, 1));
        for cell in &email_plane.cells[..email_plane.cells.len().min(plane.cells.len())] {
            let idx = (y * plane.width + left_col + cell.char as usize) as usize;
            if idx < plane.cells.len() && cell.char != '\0' {
                plane.cells[idx] = *cell;
            }
        }
        if let Some(ref err) = self.errors.email {
            for (i, c) in err.chars().take(input_width as usize).enumerate() {
                let idx = (y * plane.width + left_col + 2 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.error_fg;
                }
            }
        }

        y += 2;

        for (i, c) in "Password: ".chars().take(left_col as usize).enumerate() {
            let idx = (y * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.focused_field == FIELD_PASSWORD {
                    self.theme.accent
                } else {
                    self.theme.fg
                };
                plane.cells[idx].bg = self.theme.bg;
            }
        }
        let password_plane = self.password.render(Rect::new(left_col, y, input_width, 1));
        for cell in &password_plane.cells[..password_plane.cells.len().min(plane.cells.len())] {
            let idx = (y * plane.width + left_col + cell.char as usize) as usize;
            if idx < plane.cells.len() && cell.char != '\0' {
                plane.cells[idx] = *cell;
            }
        }
        if let Some(ref err) = self.errors.password {
            for (i, c) in err.chars().take(input_width as usize).enumerate() {
                let idx = (y * plane.width + left_col + 2 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.error_fg;
                }
            }
        }

        y += 2;

        for (i, c) in "Theme: ".chars().take(left_col as usize).enumerate() {
            let idx = (y * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.focused_field == FIELD_THEME {
                    self.theme.accent
                } else {
                    self.theme.fg
                };
                plane.cells[idx].bg = self.theme.bg;
            }
        }
        let theme_plane = self.theme.render(Rect::new(left_col, y, 20, 4));
        for cell in &theme_plane.cells[..theme_plane.cells.len().min(plane.cells.len())] {
            let idx = (y * plane.width + left_col + cell.char as usize) as usize;
            if idx < plane.cells.len() && cell.char != '\0' {
                plane.cells[idx] = *cell;
            }
        }

        y += 2;

        let toggle_plane = self.notifications.render(Rect::new(left_col, y, 30, 1));
        for cell in &toggle_plane.cells[..toggle_plane.cells.len().min(plane.cells.len())] {
            let idx = (y * plane.width + left_col + cell.char as usize) as usize;
            if idx < plane.cells.len() && cell.char != '\0' {
                plane.cells[idx] = *cell;
            }
        }

        y += 3;

        let submit_plane = self.submit.render(Rect::new(left_col, y, 20, 1));
        for cell in &submit_plane.cells[..submit_plane.cells.len().min(plane.cells.len())] {
            let idx = (y * plane.width + left_col + cell.char as usize) as usize;
            if idx < plane.cells.len() && cell.char != '\0' {
                plane.cells[idx] = *cell;
            }
        }

        y += 3;

        for (i, c) in "[Tab] next  [Shift+Tab] prev  [Enter] advance  [Esc] clear"
            .chars()
            .take(area.width as usize)
            .enumerate()
        {
            let idx = (y * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.inactive_fg;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.focus_prev();
                } else {
                    self.focus_next();
                }
                true
            }
            KeyCode::Esc => {
                self.clear_form();
                true
            }
            KeyCode::Enter => {
                if self.focused_field == FIELD_SUBMIT {
                    if self.validate_all() {
                        self.show_toast = true;
                        self.toast_message = format!(
                            "Settings saved! (user={}, theme={})",
                            self.username.query(),
                            self.theme.selected_label().unwrap_or("unknown")
                        );
                        self.dirty = true;
                    } else {
                        self.dirty = true;
                    }
                } else {
                    self.focus_next();
                }
                true
            }
            _ => {
                if let Some(widget) = self.get_current_widget_mut() {
                    widget.handle_key(key)
                } else {
                    false
                }
            }
        }
    }

    fn handle_mouse(
        &mut self,
        kind: dracon_terminal_engine::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        let left_col = 15u16;
        let input_width = 40u16;

        if row == 0 || row == 1 {
            self.focused_field = FIELD_USERNAME;
            self.dirty = true;
            return true;
        } else if row == 2 || row == 3 {
            self.focused_field = FIELD_EMAIL;
            self.dirty = true;
            return true;
        } else if row == 4 || row == 5 {
            self.focused_field = FIELD_PASSWORD;
            self.dirty = true;
            return true;
        } else if row >= 6 && row < 10 {
            self.focused_field = FIELD_THEME;
            self.dirty = true;
            return true;
        } else if row == 10 || row == 11 {
            self.focused_field = FIELD_NOTIFICATIONS;
            self.dirty = true;
            return true;
        }

        if col >= left_col && col < left_col + input_width {
            if row <= 1 {
                return self.username.handle_mouse(kind, col - left_col, 0);
            } else if row <= 3 {
                return self.email.handle_mouse(kind, col - left_col, 0);
            } else if row <= 5 {
                return self.password.handle_mouse(kind, col - left_col, 0);
            }
        }

        false
    }

    fn on_focus(&mut self) {
        self.dirty = true;
    }

    fn on_blur(&mut self) {
        self.validate_field(self.focused_field);
        self.dirty = true;
    }

    fn cursor_position(&self) -> Option<(u16, u16)> {
        match self.focused_field {
            FIELD_USERNAME => self.username.cursor_position(),
            FIELD_EMAIL => self.email.cursor_position(),
            FIELD_PASSWORD => self.password.cursor_position(),
            _ => None,
        }
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    App::new()?
        .title("Form Demo - Settings")
        .fps(30)
        .theme(theme)
        .on_tick(|ctx, _tick| {
            ctx.mark_dirty();
        })
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();

            let form_width = 60;
            let form_height = 15;
            let form_x = (w.saturating_sub(form_width)) / 2;
            let form_y = (h.saturating_sub(form_height)) / 2;

            let mut form = SettingsForm::new(WidgetId::new(1));
            form.set_area(Rect::new(form_x, form_y, form_width, form_height));

            let form_plane = form.render(form.area());
            ctx.add_plane(form_plane);

            if form.show_toast {
                let toast = Toast::new(WidgetId::new(100), &form.toast_message)
                    .with_kind(ToastKind::Success)
                    .with_duration(std::time::Duration::from_secs(3))
                    .with_theme(theme.clone());

                let toast_area = Rect::new(
                    (w.saturating_sub(40)) / 2,
                    h.saturating_sub(3),
                    40,
                    1,
                );
                ctx.add_plane(toast.render(toast_area));
            }
        })
}