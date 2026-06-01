//! Embedded Form Demo scene for the showcase.
//!
//! Demonstrates form fields with validation, submit, and drag reorder.
//! Left panel: form fields with section headers.
//! Right panel: profile preview with live summary.

use crate::scenes::shared_helpers::{blit_to, draw_text};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, PasswordInput, SearchInput, Select, Toggle,
};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;

const FIELD_USERNAME: usize = 0;
const FIELD_EMAIL: usize = 1;
const FIELD_PASSWORD: usize = 2;
const FIELD_THEME: usize = 3;
const FIELD_NOTIFICATIONS: usize = 4;
const FIELD_SUBMIT: usize = 5;
const FIELD_COUNT: usize = 6;

pub struct FormDemoScene {
    theme: Theme,
    show_help: bool,
    dirty: bool,
    focused_field: usize,
    field_order: [usize; FIELD_COUNT],
    dragging: Option<usize>,
    drag_hover: Option<usize>,
    username: SearchInput,
    email: SearchInput,
    password: PasswordInput,
    theme_select: Select,
    notifications: Toggle,
    submit: Button,
    toast: Option<String>,
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
}

impl FormDemoScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme: theme.clone(),
            show_help: false,
            dirty: true,
            focused_field: 0,
            field_order: [
                FIELD_USERNAME,
                FIELD_EMAIL,
                FIELD_PASSWORD,
                FIELD_THEME,
                FIELD_NOTIFICATIONS,
                FIELD_SUBMIT,
            ],
            dragging: None,
            drag_hover: None,
            username: SearchInput::new(WidgetId::new(10)).with_theme(theme.clone()),
            email: SearchInput::new(WidgetId::new(11)).with_theme(theme.clone()),
            password: PasswordInput::new(WidgetId::new(12)).with_theme(theme.clone()),
            theme_select: Select::new(WidgetId::new(13))
                .with_theme(theme.clone())
                .with_options(vec!["Dark".into(), "Light".into(), "Cyberpunk".into()]),
            notifications: Toggle::new(WidgetId::new(14), "Enabled").with_theme(theme.clone()),
            submit: Button::with_id(WidgetId::new(15), "Submit").with_theme(theme.clone()),
            toast: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn cycle_focus(&mut self, forward: bool) {
        if forward {
            self.focused_field = (self.focused_field + 1) % FIELD_COUNT;
        } else {
            self.focused_field = (self.focused_field + FIELD_COUNT - 1) % FIELD_COUNT;
        }
    }

    fn validate(&self) -> Option<String> {
        if self.username.query().is_empty() {
            return Some("Username required".into());
        }
        if self.email.query().is_empty() || !self.email.query().contains('@') {
            return Some("Valid email required".into());
        }
        if self.password.password().len() < 6 {
            return Some("Password must be 6+ chars".into());
        }
        None
    }

    fn submit(&mut self) {
        if let Some(err) = self.validate() {
            self.toast = Some(format!("Error: {}", err));
        } else {
            self.toast = Some("Settings saved!".into());
        }
    }

    fn reset(&mut self) {
        self.username.clear();
        self.email.clear();
        self.password.clear();
        self.toast = None;
        self.focused_field = 0;
        self.dirty = true;
    }

    fn field_section(field_id: usize) -> &'static str {
        match field_id {
            FIELD_USERNAME | FIELD_EMAIL => "Account",
            FIELD_PASSWORD => "Security",
            FIELD_THEME | FIELD_NOTIFICATIONS => "Preferences",
            FIELD_SUBMIT => "Actions",
            _ => "",
        }
    }

    fn field_label(field_id: usize) -> &'static str {
        match field_id {
            FIELD_USERNAME => "Username",
            FIELD_EMAIL => "Email",
            FIELD_PASSWORD => "Password",
            FIELD_THEME => "Theme",
            FIELD_NOTIFICATIONS => "Notifications",
            FIELD_SUBMIT => "",
            _ => "",
        }
    }

    fn field_icon(field_id: usize) -> char {
        match field_id {
            FIELD_USERNAME => '👤',
            FIELD_EMAIL => '@',
            FIELD_PASSWORD => '🔒',
            FIELD_THEME => '◈',
            FIELD_NOTIFICATIONS => '◉',
            FIELD_SUBMIT => '▶',
            _ => '○',
        }
    }
}

impl Scene for FormDemoScene {
    fn scene_id(&self) -> &str {
        "form_demo"
    }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title
        draw_text(&mut plane, 2, 0, " Settings Form ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Validation summary bar (top-right) ───────────────────────
        let username = self.username.query();
        let email_q = self.email.query();
        let password = self.password.password();
        let mut errors: Vec<&str> = Vec::new();
        if username.is_empty() {
            errors.push("username required");
        } else if username.len() < 3 {
            errors.push("username ≥3 chars");
        }
        if email_q.is_empty() {
            errors.push("email required");
        } else if !email_q.contains('@') || !email_q.contains('.') {
            errors.push("invalid email");
        }
        if password.len() < 6 {
            errors.push("password ≥6 chars");
        }
        let summary_text = if errors.is_empty() {
            "✓ All fields valid".to_string()
        } else {
            format!("✗ {}", errors.join(" · "))
        };
        let summary_color = if errors.is_empty() {
            t.success
        } else {
            t.error
        };
        draw_text(
            &mut plane,
            area.width.saturating_sub(summary_text.len() as u16 + 2),
            1,
            &summary_text,
            summary_color,
            t.bg,
            true,
        );

        // ── Left panel: Form fields ──────────────────────────────────────
        let form_w = (area.width * 55 / 100).max(30);
        let start_y = 2u16;
        let field_h = 2u16;

        // Build position map
        let mut field_to_row = [0usize; FIELD_COUNT];
        for (row_idx, &field_id) in self.field_order.iter().enumerate() {
            field_to_row[field_id] = row_idx;
        }

        // Track current section for headers
        let mut last_section = "";
        let mut y_offset = 0u16;

        for (row_idx, &field_id) in self.field_order.iter().enumerate() {
            let section = Self::field_section(field_id);

            // Section header
            if section != last_section && !section.is_empty() {
                let sy = start_y + y_offset;
                draw_text(&mut plane, 2, sy, section, t.secondary, t.bg, true);
                for dx in 0..form_w.saturating_sub(4) {
                    let idx = ((sy + 1) * plane.width + 2 + dx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '─';
                        plane.cells[idx].fg = t.outline;
                    }
                }
                y_offset += 2;
                last_section = section;
            }

            let y = start_y + y_offset;
            let is_focused = self.focused_field == field_id;
            let is_dragged = self.dragging == Some(row_idx);
            let is_hover_target = self.drag_hover == Some(row_idx) && self.dragging.is_some();

            let row_bg = if is_dragged {
                t.selection_bg
            } else if is_hover_target {
                t.primary_hover
            } else if is_focused {
                t.focus_bg
            } else {
                t.surface
            };

            // Row background
            for row in y..y + field_h {
                for col in 0..form_w {
                    let idx = (row * plane.width + col) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = row_bg;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Drag handle
            if !is_dragged {
                let handle = if is_hover_target { ">" } else { "=" };
                draw_text(&mut plane, 0, y, handle, t.fg_muted, row_bg, false);
            }

            // Icon + label
            let icon = Self::field_icon(field_id);
            let label = Self::field_label(field_id);
            let icon_idx = (y * plane.width + 2) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = icon;
                plane.cells[icon_idx].fg = if is_focused { t.primary } else { t.fg_muted };
            }
            if !label.is_empty() {
                draw_text(
                    &mut plane,
                    4,
                    y,
                    label,
                    if is_focused { t.primary } else { t.fg },
                    row_bg,
                    is_focused,
                );
            }

            // Validation indicator + inline error
            let (valid, error_msg): (bool, &str) = match field_id {
                FIELD_USERNAME => {
                    let q = self.username.query();
                    if q.is_empty() {
                        (false, "required")
                    } else if q.len() < 3 {
                        (false, "≥3 chars")
                    } else {
                        (true, "")
                    }
                }
                FIELD_EMAIL => {
                    let q = self.email.query();
                    if q.is_empty() {
                        (false, "required")
                    } else if !q.contains('@') || !q.contains('.') {
                        (false, "must contain @ and .")
                    } else {
                        (true, "")
                    }
                }
                FIELD_PASSWORD => {
                    let p = self.password.password();
                    if p.is_empty() {
                        (false, "required")
                    } else if p.len() < 6 {
                        (false, "≥6 chars")
                    } else {
                        (true, "")
                    }
                }
                _ => (true, ""),
            };
            if valid {
                draw_text(
                    &mut plane,
                    form_w.saturating_sub(2),
                    y,
                    "✓",
                    t.success,
                    row_bg,
                    false,
                );
            } else {
                // Show inline error message in red next to the field
                draw_text(&mut plane, 16, y + 1, error_msg, t.error, t.bg, false);
            }

            // Widget
            let widget_x = 16u16;
            let widget_w = form_w.saturating_sub(widget_x + 4);
            let widget_area = Rect::new(widget_x, y, widget_w, 1);
            if widget_area.width > 0 {
                let w_plane = match field_id {
                    FIELD_USERNAME => self.username.render(widget_area),
                    FIELD_EMAIL => self.email.render(widget_area),
                    FIELD_PASSWORD => self.password.render(widget_area),
                    FIELD_THEME => self.theme_select.render(widget_area),
                    FIELD_NOTIFICATIONS => self.notifications.render(widget_area),
                    FIELD_SUBMIT => self.submit.render(widget_area),
                    _ => Plane::new(0, 0, 0),
                };
                blit_to(&mut plane, &w_plane, widget_x as usize, y as usize);
            }

            y_offset += field_h;
        }

        // Reset button (below form)
        let reset_y = start_y + y_offset + 1;
        if reset_y < area.height.saturating_sub(2) {
            draw_text(
                &mut plane,
                2,
                reset_y,
                "r: reset form",
                t.fg_muted,
                t.bg,
                false,
            );
        }

        // ── Vertical divider ──────────────────────────────────────────────
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * plane.width + form_w) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Right panel: Profile Preview ──────────────────────────────────
        let panel_x = form_w + 2;
        let panel_w = area.width.saturating_sub(panel_x + 2);

        draw_text(
            &mut plane,
            panel_x,
            2,
            "Profile Preview",
            t.primary,
            t.bg,
            true,
        );
        for dx in 0..panel_w {
            let idx = (3 * plane.width + panel_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Avatar placeholder
        let avatar_y = 4;
        for dy in 0..3 {
            for dx in 0..5 {
                let idx = ((avatar_y + dy) * plane.width + panel_x + 2 + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                }
            }
        }
        let avatar_ch = if self.username.query().is_empty() {
            '?'
        } else {
            self.username.query().chars().next().unwrap_or('?')
        };
        let av_idx = ((avatar_y + 1) * plane.width + panel_x + 3) as usize;
        if av_idx < plane.cells.len() {
            plane.cells[av_idx].char = avatar_ch;
            plane.cells[av_idx].fg = t.primary;
            plane.cells[av_idx].style = dracon_terminal_engine::compositor::plane::Styles::BOLD;
        }

        // Profile info
        let username = self.username.query();
        let display_name: &str = if username.is_empty() {
            "Not set"
        } else {
            username
        };
        draw_text(
            &mut plane,
            panel_x + 9,
            avatar_y,
            display_name,
            if username.is_empty() {
                t.fg_muted
            } else {
                t.fg
            },
            t.bg,
            true,
        );

        let email = self.email.query();
        let display_email: &str = if email.is_empty() { "Not set" } else { email };
        draw_text(
            &mut plane,
            panel_x + 9,
            avatar_y + 1,
            display_email,
            if email.is_empty() {
                t.fg_muted
            } else {
                t.secondary
            },
            t.bg,
            false,
        );

        // Status badge
        let validation = self.validate();
        let (badge_text, badge_color) = if validation.is_none() {
            ("✓ Valid", t.success)
        } else {
            ("✗ Incomplete", t.error)
        };
        draw_text(
            &mut plane,
            panel_x + 9,
            avatar_y + 2,
            badge_text,
            badge_color,
            t.bg,
            true,
        );

        // Settings summary
        let summary_y = avatar_y + 4;
        draw_text(
            &mut plane,
            panel_x,
            summary_y,
            "Settings",
            t.secondary,
            t.bg,
            true,
        );
        for dx in 0..panel_w {
            let idx = ((summary_y + 1) * plane.width + panel_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let settings = [
            (
                "Theme",
                self.theme_select.selected_label().unwrap_or("Dark"),
            ),
            (
                "Notifications",
                if self.notifications.is_on() {
                    "On"
                } else {
                    "Off"
                },
            ),
            (
                "Password",
                if self.password.password().len() >= 6 {
                    "Set"
                } else if self.password.password().is_empty() {
                    "Not set"
                } else {
                    "Too short"
                },
            ),
            (
                "Email",
                if self.email.query().contains('@') {
                    "Valid"
                } else if self.email.query().is_empty() {
                    "Not set"
                } else {
                    "Invalid"
                },
            ),
        ];
        for (i, (label, value)) in settings.iter().enumerate() {
            let sy = summary_y + 2 + i as u16;
            if sy >= area.height.saturating_sub(2) {
                break;
            }
            draw_text(&mut plane, panel_x, sy, label, t.fg_muted, t.bg, false);
            let val_color = match *value {
                "Valid" | "Set" | "On" => t.success,
                "Invalid" | "Too short" | "Off" => t.warning,
                _ => t.fg,
            };
            draw_text(&mut plane, panel_x + 14, sy, value, val_color, t.bg, false);
        }

        // Keyboard shortcuts
        let shortcuts_y = summary_y + 7;
        if shortcuts_y + 5 < area.height.saturating_sub(2) {
            draw_text(
                &mut plane,
                panel_x,
                shortcuts_y,
                "Keyboard",
                t.secondary,
                t.bg,
                true,
            );
            for dx in 0..panel_w {
                let idx = ((shortcuts_y + 1) * plane.width + panel_x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }
            let shortcuts = [
                ("Tab", "Next field"),
                ("Enter", "Submit"),
                ("r", "Reset form"),
                ("Drag =", "Reorder"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let ky = shortcuts_y + 2 + i as u16;
                if ky >= area.height.saturating_sub(2) {
                    break;
                }
                draw_text(&mut plane, panel_x, ky, key, t.primary, t.bg, false);
                draw_text(&mut plane, panel_x + 8, ky, desc, t.fg_muted, t.bg, false);
            }
        }

        // Toast
        if let Some(ref msg) = self.toast {
            let toast_y = area.height.saturating_sub(3);
            let toast_x = (area.width.saturating_sub(msg.len() as u16 + 4)) / 2;
            for x in toast_x..toast_x + msg.len() as u16 + 4 {
                let idx = (toast_y * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = if msg.starts_with("Error") {
                        t.error_bg
                    } else {
                        t.success_bg
                    };
                    plane.cells[idx].fg = if msg.starts_with("Error") {
                        t.error
                    } else {
                        t.success
                    };
                }
            }
            draw_text(
                &mut plane,
                toast_x + 2,
                toast_y,
                msg,
                if msg.starts_with("Error") {
                    t.error
                } else {
                    t.success
                },
                if msg.starts_with("Error") {
                    t.error_bg
                } else {
                    t.success_bg
                },
                true,
            );
        }

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        let nav = " Tab:next | Enter:submit | Drag=:reorder | r:reset | ?:help | B:back ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.surface, false);

        if self.show_help {
            crate::scenes::shared_helpers::render_help_overlay(
                &mut plane,
                area,
                t,
                "Form Demo Help",
                &[
                    ("Tab", "Next field"),
                    ("Shift+Tab", "Previous field"),
                    ("Enter", "Submit form"),
                    ("r", "Reset form"),
                    ("Live validation", "Errors shown in red under fields"),
                    ("Drag =", "Reorder fields"),
                    ("Esc", "Back"),
                ],
            );
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
            }
            return true;
        }

        if self.toast.is_some() {
            self.toast = None;
            self.dirty = true;
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Tab => {
                self.cycle_focus(true);
                self.dirty = true;
                true
            }
            KeyCode::BackTab => {
                self.cycle_focus(false);
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                if self.focused_field == FIELD_SUBMIT {
                    self.submit();
                }
                self.dirty = true;
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.reset();
                true
            }
            _ => {
                let handled = match self.focused_field {
                    FIELD_USERNAME => self.username.handle_key(key),
                    FIELD_EMAIL => self.email.handle_key(key),
                    FIELD_PASSWORD => self.password.handle_key(key),
                    FIELD_THEME => self.theme_select.handle_key(key),
                    FIELD_NOTIFICATIONS => self.notifications.handle_key(key),
                    FIELD_SUBMIT => self.submit.handle_key(key),
                    _ => false,
                };
                if handled {
                    self.dirty = true;
                }
                handled
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.show_help {
            return true;
        }

        let area = self.area.get();
        let form_w = (area.width * 55 / 100).max(30);
        let start_y = 2u16;
        let field_h = 2u16;

        // Calculate approximate row index (section headers shift positions)
        let row_idx = if row >= start_y && col < form_w {
            Some(((row - start_y) / field_h).min((FIELD_COUNT - 1) as u16) as usize)
        } else {
            None
        };

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(idx) = row_idx {
                    if col < 2 && self.dragging.is_none() {
                        self.dragging = Some(idx);
                        self.drag_hover = Some(idx);
                        self.dirty = true;
                        return true;
                    }
                    if self.dragging.is_some() {
                        if let Some(drag_idx) = self.dragging {
                            if let Some(hover_idx) = self.drag_hover {
                                if drag_idx != hover_idx {
                                    self.field_order.swap(drag_idx, hover_idx);
                                }
                            }
                        }
                        self.dragging = None;
                        self.drag_hover = None;
                        self.dirty = true;
                        return true;
                    }
                    let field_id = self.field_order[idx.min(FIELD_COUNT - 1)];
                    if field_id == FIELD_SUBMIT {
                        self.submit();
                    } else {
                        self.focused_field = field_id;
                    }
                    self.dirty = true;
                    return true;
                }
                false
            }
            MouseEventKind::Moved if self.dragging.is_some() => {
                if let Some(idx) = row_idx {
                    if self.drag_hover != Some(idx) {
                        self.drag_hover = Some(idx);
                        self.dirty = true;
                    }
                }
                true
            }
            MouseEventKind::Moved => false,
            MouseEventKind::Down(MouseButton::Right) if self.dragging.is_some() => {
                self.dragging = None;
                self.drag_hover = None;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.username.on_theme_change(theme);
        self.email.on_theme_change(theme);
        self.password.on_theme_change(theme);
        self.theme_select.on_theme_change(theme);
        self.notifications.on_theme_change(theme);
        self.submit.on_theme_change(theme);
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
}
