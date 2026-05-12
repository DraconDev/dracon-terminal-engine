#![allow(missing_docs)]
//! Rich Form Demo — settings form with validation, icons, and visual polish.
//!
//! Features:
//! - Rounded border card layout with header bar
//! - Nerd font icons on every field label
//! - Focus row highlighting with `focus_bg`
//! - Validation errors with error icons and red text
//! - Success toast with rounded border and icon
//! - Theme cycling (`t`)
//! - Help overlay (`?`)
//! - Mouse support for all fields
//!
//! Controls:
//!   Tab/Shift+Tab  — cycle focus
//!   Enter          — advance field / submit
//!   Esc            — clear form / dismiss help
//!   t              — cycle theme
//!   ?              — help overlay
//!   q              — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, PasswordInput, SearchInput, Select, Toggle,
};
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const FIELD_USERNAME: usize = 0;
const FIELD_EMAIL: usize = 1;
const FIELD_PASSWORD: usize = 2;
const FIELD_THEME: usize = 3;
const FIELD_NOTIFICATIONS: usize = 4;
const FIELD_SUBMIT: usize = 5;
const FIELD_COUNT: usize = 6;

#[derive(Default)]
struct ValidationErrors {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

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
    toast_timer: u32,
    form_theme: Theme,
    show_help: bool,
    area: Rect,
    dirty: bool,
    keybindings: KeybindingSet,
}

impl SettingsForm {
    fn new(id: WidgetId, theme: Theme, keybindings: KeybindingSet) -> Self {
        let theme_select = Select::new(WidgetId::new(id.0 + 3)).with_options(vec![
            "Dark".to_string(),
            "Light".to_string(),
            "Cyberpunk".to_string(),
            "Nord".to_string(),
            "Dracula".to_string(),
        ]);

        Self {
            id,
            username: SearchInput::new(WidgetId::new(id.0 + 1)),
            email: SearchInput::new(WidgetId::new(id.0 + 2)),
            password: PasswordInput::new(WidgetId::new(id.0 + 3)),
            theme: theme_select,
            notifications: Toggle::new(WidgetId::new(id.0 + 4), "Enable notifications"),
            submit: Button::with_id(WidgetId::new(id.0 + 5), "Save Settings"),
            focused_field: FIELD_USERNAME,
            errors: ValidationErrors::default(),
            show_toast: false,
            toast_message: String::new(),
            toast_timer: 0,
            form_theme: theme,
            show_help: false,
            area: Rect::new(0, 0, 70, 18),
            dirty: true,
            keybindings,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::dark(),
            Theme::light(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::nord(),
            Theme::catppuccin_mocha(),
            Theme::gruvbox_dark(),
            Theme::tokyo_night(),
            Theme::solarized_dark(),
            Theme::solarized_light(),
            Theme::one_dark(),
            Theme::rose_pine(),
            Theme::kanagawa(),
            Theme::everforest(),
            Theme::monokai(),
            Theme::warm(),
            Theme::cool(),
            Theme::forest(),
            Theme::sunset(),
            Theme::mono(),
        ];
        let idx = themes
            .iter()
            .position(|t| t.name == self.form_theme.name)
            .unwrap_or(0);
        self.form_theme = themes[(idx + 1) % themes.len()];
        self.username.on_theme_change(&self.form_theme);
        self.email.on_theme_change(&self.form_theme);
        self.password.on_theme_change(&self.form_theme);
        self.theme.on_theme_change(&self.form_theme);
        self.notifications.on_theme_change(&self.form_theme);
        self.submit.on_theme_change(&self.form_theme);
        self.dirty = true;
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
                    self.errors.password =
                        Some("Password must be at least 8 characters".to_string());
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
        self.show_toast = false;
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

    fn field_label(field: usize) -> &'static str {
        match field {
            FIELD_USERNAME => "󰀄 Username",
            FIELD_EMAIL => "󰇮 Email",
            FIELD_PASSWORD => "󰌆 Password",
            FIELD_THEME => "󰔎 Theme",
            FIELD_NOTIFICATIONS => "",
            FIELD_SUBMIT => "",
            _ => "",
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
        self.dirty
            || self.username.needs_render()
            || self.email.needs_render()
            || self.password.needs_render()
            || self.theme.needs_render()
            || self.notifications.needs_render()
            || self.submit.needs_render()
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
        let t = self.form_theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = self.z_index() as i32;

        // Background
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let margin = 2u16;
        let card_w = area.width.saturating_sub(margin * 2);
        let card_h = area.height.saturating_sub(2);

        // Card border
        draw_rounded_border(&mut plane, margin, 0, card_w, card_h, t);

        // Fill card background
        for y in 1..card_h.saturating_sub(1) {
            for x in 1..card_w.saturating_sub(1) {
                let idx = (y * area.width + margin + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                }
            }
        }

        // === HEADER ===
        let title = " 󰒓 Settings ";
        let title_x = margin + (card_w - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (area.width + title_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].bg = t.surface;
            }
        }

        let subtitle = "Configure your account preferences";
        let sub_x = margin + (card_w - subtitle.len() as u16) / 2;
        for (i, c) in subtitle.chars().enumerate() {
            let idx = (2 * area.width + sub_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
            }
        }

        // === FIELDS ===
        let label_col = margin + 2;
        let input_col = margin + 18;
        let input_width = card_w.saturating_sub(22);

        // Username
        {
            let field = FIELD_USERNAME;
            let y = 4u16;
            let is_focused = self.focused_field == field;
            let row_bg = if is_focused { t.focus_bg } else { t.surface };
            for dx in 1..card_w.saturating_sub(1) {
                let idx = (y * area.width + margin + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = row_bg;
                }
            }
            if is_focused {
                for dx in 1..card_w.saturating_sub(1) {
                    let top_idx = ((y - 1) * area.width + margin + dx) as usize;
                    let bot_idx = ((y + 1) * area.width + margin + dx) as usize;
                    if top_idx < plane.cells.len() {
                        plane.cells[top_idx].bg = t.focus_border;
                    }
                    if bot_idx < plane.cells.len() {
                        plane.cells[bot_idx].bg = t.focus_border;
                    }
                }
            }
            for (i, c) in Self::field_label(field).chars().enumerate() {
                let idx = (y * area.width + label_col + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if is_focused { t.primary } else { t.fg };
                    plane.cells[idx].bg = row_bg;
                    plane.cells[idx].style = if is_focused {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                }
            }
            if self.errors.username.is_some() {
                let err_x = margin + card_w - 4;
                let idx = (y * area.width + err_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '󰅙';
                    plane.cells[idx].fg = t.error;
                    plane.cells[idx].bg = row_bg;
                }
            }
        }
        let username_plane = self
            .username
            .render(Rect::new(input_col, 4, input_width, 1));
        blit(&mut plane, &username_plane, input_col, 4);
        if let Some(ref err) = self.errors.username {
            let err_text = format!("  󰅙 {}", err);
            for (i, c) in err_text.chars().take(input_width as usize).enumerate() {
                let idx = (5 * area.width + input_col + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.error;
                    plane.cells[idx].bg = t.surface;
                }
            }
        }

        // Email
        {
            let field = FIELD_EMAIL;
            let y = 6u16;
            let is_focused = self.focused_field == field;
            let row_bg = if is_focused { t.focus_bg } else { t.surface };
            for dx in 1..card_w.saturating_sub(1) {
                let idx = (y * area.width + margin + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = row_bg;
                }
            }
            if is_focused {
                for dx in 1..card_w.saturating_sub(1) {
                    let top_idx = ((y - 1) * area.width + margin + dx) as usize;
                    let bot_idx = ((y + 1) * area.width + margin + dx) as usize;
                    if top_idx < plane.cells.len() {
                        plane.cells[top_idx].bg = t.focus_border;
                    }
                    if bot_idx < plane.cells.len() {
                        plane.cells[bot_idx].bg = t.focus_border;
                    }
                }
            }
            for (i, c) in Self::field_label(field).chars().enumerate() {
                let idx = (y * area.width + label_col + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if is_focused { t.primary } else { t.fg };
                    plane.cells[idx].bg = row_bg;
                    plane.cells[idx].style = if is_focused {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                }
            }
            if self.errors.email.is_some() {
                let err_x = margin + card_w - 4;
                let idx = (y * area.width + err_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '󰅙';
                    plane.cells[idx].fg = t.error;
                    plane.cells[idx].bg = row_bg;
                }
            }
        }
        let email_plane = self.email.render(Rect::new(input_col, 6, input_width, 1));
        blit(&mut plane, &email_plane, input_col, 6);
        if let Some(ref err) = self.errors.email {
            let err_text = format!("  󰅙 {}", err);
            for (i, c) in err_text.chars().take(input_width as usize).enumerate() {
                let idx = (7 * area.width + input_col + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.error;
                    plane.cells[idx].bg = t.surface;
                }
            }
        }

        // Password
        {
            let field = FIELD_PASSWORD;
            let y = 8u16;
            let is_focused = self.focused_field == field;
            let row_bg = if is_focused { t.focus_bg } else { t.surface };
            for dx in 1..card_w.saturating_sub(1) {
                let idx = (y * area.width + margin + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = row_bg;
                }
            }
            if is_focused {
                for dx in 1..card_w.saturating_sub(1) {
                    let top_idx = ((y - 1) * area.width + margin + dx) as usize;
                    let bot_idx = ((y + 1) * area.width + margin + dx) as usize;
                    if top_idx < plane.cells.len() {
                        plane.cells[top_idx].bg = t.focus_border;
                    }
                    if bot_idx < plane.cells.len() {
                        plane.cells[bot_idx].bg = t.focus_border;
                    }
                }
            }
            for (i, c) in Self::field_label(field).chars().enumerate() {
                let idx = (y * area.width + label_col + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if is_focused { t.primary } else { t.fg };
                    plane.cells[idx].bg = row_bg;
                    plane.cells[idx].style = if is_focused {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                }
            }
            if self.errors.password.is_some() {
                let err_x = margin + card_w - 4;
                let idx = (y * area.width + err_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '󰅙';
                    plane.cells[idx].fg = t.error;
                    plane.cells[idx].bg = row_bg;
                }
            }
        }
        let password_plane = self
            .password
            .render(Rect::new(input_col, 8, input_width, 1));
        blit(&mut plane, &password_plane, input_col, 8);
        if let Some(ref err) = self.errors.password {
            let err_text = format!("  󰅙 {}", err);
            for (i, c) in err_text.chars().take(input_width as usize).enumerate() {
                let idx = (9 * area.width + input_col + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.error;
                    plane.cells[idx].bg = t.surface;
                }
            }
        }

        // Theme
        {
            let field = FIELD_THEME;
            let y = 10u16;
            let is_focused = self.focused_field == field;
            let row_bg = if is_focused { t.focus_bg } else { t.surface };
            for dx in 1..card_w.saturating_sub(1) {
                let idx = (y * area.width + margin + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = row_bg;
                }
            }
            if is_focused {
                for dx in 1..card_w.saturating_sub(1) {
                    let top_idx = ((y - 1) * area.width + margin + dx) as usize;
                    let bot_idx = ((y + 1) * area.width + margin + dx) as usize;
                    if top_idx < plane.cells.len() {
                        plane.cells[top_idx].bg = t.focus_border;
                    }
                    if bot_idx < plane.cells.len() {
                        plane.cells[bot_idx].bg = t.focus_border;
                    }
                }
            }
            for (i, c) in Self::field_label(field).chars().enumerate() {
                let idx = (y * area.width + label_col + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if is_focused { t.primary } else { t.fg };
                    plane.cells[idx].bg = row_bg;
                    plane.cells[idx].style = if is_focused {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                }
            }
        }
        let theme_plane = self.theme.render(Rect::new(input_col, 10, 25, 4));
        blit(&mut plane, &theme_plane, input_col, 10);

        // Notifications
        let notif_label = "󰂚 Notifications";
        let is_focused = self.focused_field == FIELD_NOTIFICATIONS;
        let row_bg = if is_focused { t.focus_bg } else { t.surface };
        for dx in 1..card_w.saturating_sub(1) {
            let idx = (12 * area.width + margin + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = row_bg;
            }
        }
        if is_focused {
            for dx in 1..card_w.saturating_sub(1) {
                let top_idx = (11 * area.width + margin + dx) as usize;
                let bot_idx = (13 * area.width + margin + dx) as usize;
                if top_idx < plane.cells.len() {
                    plane.cells[top_idx].bg = t.focus_border;
                }
                if bot_idx < plane.cells.len() {
                    plane.cells[bot_idx].bg = t.focus_border;
                }
            }
        }
        for (i, c) in notif_label.chars().enumerate() {
            let idx = (12 * area.width + label_col + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if is_focused { t.primary } else { t.fg };
                plane.cells[idx].bg = row_bg;
                plane.cells[idx].style = if is_focused {
                    Styles::BOLD
                } else {
                    Styles::empty()
                };
            }
        }
        let toggle_plane = self.notifications.render(Rect::new(input_col, 12, 30, 1));
        blit(&mut plane, &toggle_plane, input_col, 12);

        // Submit button
        let is_focused = self.focused_field == FIELD_SUBMIT;
        let row_bg = if is_focused { t.focus_bg } else { t.surface };
        for dx in 1..card_w.saturating_sub(1) {
            let idx = (14 * area.width + margin + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = row_bg;
            }
        }
        if is_focused {
            for dx in 1..card_w.saturating_sub(1) {
                let top_idx = (13 * area.width + margin + dx) as usize;
                let bot_idx = (15 * area.width + margin + dx) as usize;
                if top_idx < plane.cells.len() {
                    plane.cells[top_idx].bg = t.focus_border;
                }
                if bot_idx < plane.cells.len() {
                    plane.cells[bot_idx].bg = t.focus_border;
                }
            }
        }
        let submit_plane = self.submit.render(Rect::new(input_col, 14, 20, 1));
        blit(&mut plane, &submit_plane, input_col, 14);

        // === STATUS BAR ===
        let status_y = card_h.saturating_sub(1);
        let hint = self.keybindings.format_hint(&[
            ("Tab", "next"),
            ("Shift+Tab", "prev"),
            ("Enter", "submit"),
            (actions::DISMISS, "clear"),
            (actions::THEME, "theme"),
            (actions::HELP, "help"),
            (actions::QUIT, "quit"),
        ]);
        for (i, c) in hint.chars().take(card_w as usize - 2).enumerate() {
            let idx = (status_y * area.width + margin + 1 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_subtle;
                plane.cells[idx].bg = t.surface;
            }
        }

        // === TOAST ===
        if self.show_toast {
            let toast_w = self.toast_message.len() as u16 + 6;
            let toast_h = 3u16;
            let toast_x = margin + (card_w - toast_w) / 2;
            let toast_y = card_h / 2;

            // Toast background
            for py in 0..toast_h {
                for px in 0..toast_w {
                    let idx = ((toast_y + py) * area.width + toast_x + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.success_bg;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            draw_rounded_border(&mut plane, toast_x, toast_y, toast_w, toast_h, t);

            let toast_text = format!(" 󰄬 {} ", self.toast_message);
            let text_x = toast_x + (toast_w - toast_text.len() as u16) / 2;
            for (i, c) in toast_text.chars().enumerate() {
                let idx = ((toast_y + 1) * area.width + text_x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.success;
                    plane.cells[idx].bg = t.success_bg;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }
        }

        // === HELP OVERLAY ===
        if self.show_help {
            render_help_overlay(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.show_toast {
            self.show_toast = false;
            self.dirty = true;
        }

        if self.keybindings.matches(actions::QUIT, &key) {
            return false; // Let app handle quit
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::DISMISS, &key) {
            self.clear_form();
            return true;
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
            KeyCode::Enter => {
                if self.focused_field == FIELD_SUBMIT {
                    if self.validate_all() {
                        self.show_toast = true;
                        self.toast_message = format!(
                            "Settings saved! (user={}, theme={})",
                            self.username.query(),
                            self.theme.selected_label().unwrap_or("unknown")
                        );
                        self.toast_timer = 120;
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

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let margin = 2u16;
        let input_col = margin + 18;
        let input_width = self.area.width.saturating_sub(margin * 2 + 22);

        // Map rows to fields
        if row == 4 && col >= input_col && col < input_col + input_width {
            self.focused_field = FIELD_USERNAME;
            self.dirty = true;
            return self.username.handle_mouse(kind, col - input_col, 0);
        } else if row == 6 && col >= input_col && col < input_col + input_width {
            self.focused_field = FIELD_EMAIL;
            self.dirty = true;
            return self.email.handle_mouse(kind, col - input_col, 0);
        } else if row == 8 && col >= input_col && col < input_col + input_width {
            self.focused_field = FIELD_PASSWORD;
            self.dirty = true;
            return self.password.handle_mouse(kind, col - input_col, 0);
        } else if (10..=13).contains(&row) && col >= input_col {
            self.focused_field = FIELD_THEME;
            self.dirty = true;
            return self.theme.handle_mouse(kind, col - input_col, row - 10);
        } else if row == 12 && col >= input_col {
            self.focused_field = FIELD_NOTIFICATIONS;
            self.dirty = true;
            return self.notifications.handle_mouse(kind, col - input_col, 0);
        } else if row == 14 && col >= input_col && col < input_col + 20 {
            self.focused_field = FIELD_SUBMIT;
            self.dirty = true;
            return self.submit.handle_mouse(kind, col - input_col, 0);
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

fn render_help_overlay(plane: &mut Plane, area: Rect, t: Theme) {
    let w = 50u16.min(area.width - 4);
    let h = 13u16.min(area.height - 4);
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;

    for py in 0..h {
        for px in 0..w {
            let idx = ((y + py) * area.width + x + px) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }

    draw_rounded_border(plane, x, y, w, h, t);

    let title = "Form Help";
    let title_x = x + (w - title.len() as u16) / 2;
    draw_text(
        plane,
        title_x,
        y + 1,
        title,
        t.primary,
        t.surface_elevated,
        true,
    );

    let shortcuts = [
        ("Tab / Shift+Tab", "Cycle focus"),
        ("Enter", "Advance / Submit"),
        ("Esc", "Clear form"),
        ("t", "Cycle theme"),
        ("?", "Toggle help"),
        ("q", "Quit"),
    ];

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = y + 3 + i as u16;
        draw_text(plane, x + 3, row, key, t.primary, t.surface_elevated, true);
        draw_text(plane, x + 22, row, desc, t.fg, t.surface_elevated, false);
    }

    let hint = "Press ? or Esc to close";
    draw_text(
        plane,
        x + 3,
        y + h - 1,
        hint,
        t.fg_muted,
        t.surface_elevated,
        false,
    );
}

fn draw_rounded_border(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: Theme) {
    if w < 3 || h < 2 {
        return;
    }
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() {
                continue;
            }
            let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
            if is_border {
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].char = if row == y && col == x {
                    '╭'
                } else if row == y && col == x + w - 1 {
                    '╮'
                } else if row == y + h - 1 && col == x {
                    '╰'
                } else if row == y + h - 1 && col == x + w - 1 {
                    '╯'
                } else if row == y || row == y + h - 1 {
                    '─'
                } else {
                    '│'
                };
                plane.cells[idx].transparent = false;
            }
        }
    }
}

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };
        }
    }
}

fn blit(dst: &mut Plane, src: &Plane, dx: u16, dy: u16) {
    for (i, cell) in src.cells.iter().enumerate() {
        if cell.transparent {
            continue;
        }
        let x = (i % src.width as usize) as u16 + dx;
        let y = (i / src.width as usize) as u16 + dy;
        let idx = (y * dst.width + x) as usize;
        if idx < dst.cells.len() && x < dst.width && y < dst.height {
            dst.cells[idx] = cell.clone();
        }
    }
}

fn main() -> std::io::Result<()> {
    let (_w, _h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());
    let form = SettingsForm::new(WidgetId::new(0), Theme::dracula(), keybindings.clone());

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?
        .title("Settings Form")
        .fps(30)
        .theme(Theme::from_env_or(Theme::dracula()));
    app.add_widget(Box::new(form), Rect::new(0, 0, 70, 18));
    app.on_tick(move |ctx, _tick| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(|_ctx| {})
}
