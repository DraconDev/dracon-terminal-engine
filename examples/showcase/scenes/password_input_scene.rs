//! Embedded Password Input scene for the showcase.
//!
//! Demonstrates the PasswordInput widget in a login form context.

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::password_input::PasswordInput;
use dracon_terminal_engine::framework::widgets::search_input::SearchInput;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

pub struct PasswordInputScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    username_input: SearchInput,
    password_input: PasswordInput,
    confirm_input: PasswordInput,
    focused: usize, // 0=username, 1=password, 2=confirm
    submitted: bool,
    error_msg: Option<&'static str>,
    dirty: bool,
    area: std::cell::Cell<Rect>,
}

impl PasswordInputScene {
    pub fn new(theme: Theme) -> Self {
        let username_input = SearchInput::new(WidgetId::new(1))
            .with_theme(theme.clone());

        let password_input = PasswordInput::new(WidgetId::new(2))
            .with_theme(theme.clone());

        let confirm_input = PasswordInput::new(WidgetId::new(3))
            .with_theme(theme.clone())
            .with_mask_char('●');

        Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            username_input,
            password_input,
            confirm_input,
            focused: 0,
            submitted: false,
            error_msg: None,
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn render_field_label(&self, plane: &mut Plane, x: u16, y: u16, label: &str, is_focused: bool) {
        let t = &self.theme;
        let fg = if is_focused { t.primary } else { t.fg_muted };
        let style = if is_focused { Styles::BOLD } else { Styles::empty() };
        for (j, ch) in label.chars().enumerate() {
            let idx = (y * plane.width + x + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = fg;
                plane.cells[idx].style = style;
                plane.cells[idx].transparent = false;
            }
        }
        // Focus indicator
        if is_focused {
            draw_text(plane, x + label.len() as u16, y, " ◄", t.primary, t.bg, false);
        }
    }

    fn render_input_field(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, input_plane: &Plane) {
        // Blit input plane into position
        for iy in 0..h.min(input_plane.height) {
            for ix in 0..w.min(input_plane.width) {
                let src_idx = (iy * input_plane.width + ix) as usize;
                let dst_idx = ((y + iy) * plane.width + x + ix) as usize;
                if src_idx < input_plane.cells.len() && dst_idx < plane.cells.len() {
                    let src = &input_plane.cells[src_idx];
                    if !src.transparent {
                        plane.cells[dst_idx] = *src;
                    }
                }
            }
        }
    }

    fn try_submit(&mut self) {
        let username = self.username_input.query();
        let password = self.password_input.password();
        let confirm = self.confirm_input.password();

        if username.is_empty() {
            self.error_msg = Some("Username is required");
            return;
        }
        if password.is_empty() {
            self.error_msg = Some("Password is required");
            return;
        }
        if password.len() < 6 {
            self.error_msg = Some("Password must be at least 6 characters");
            return;
        }
        if password != confirm {
            self.error_msg = Some("Passwords do not match");
            return;
        }
        self.error_msg = None;
        self.submitted = true;
    }
}

impl Scene for PasswordInputScene {
    fn scene_id(&self) -> &str { "password_input" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(&mut plane, 2, 0, " Password Input ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        if self.submitted {
            // Success screen
            draw_text(&mut plane, 2, 4, "✓ Login Successful!", t.success, t.bg, true);
            draw_text(&mut plane, 2, 6, &format!("Welcome, {}!", self.username_input.query()), t.fg, t.bg, false);
            draw_text(&mut plane, 2, 8, "Press R to reset and try again", t.fg_muted, t.bg, false);

            // Password strength indicator
            let strength = match self.password_input.password().len() {
                0..=5 => ("Weak", t.error),
                6..=9 => ("Medium", t.warning),
                _ => ("Strong", t.success),
            };
            draw_text(&mut plane, 2, 10, &format!("Password strength: {}", strength.0), strength.1, t.bg, true);
        } else {
            // Login form
            draw_text(&mut plane, 2, 2, "Login Form", t.fg, t.bg, true);
            draw_text(&mut plane, 2, 3, "Tab to switch fields, Enter to submit", t.fg_muted, t.bg, false);

            // Username field
            self.render_field_label(&mut plane, 4, 5, "Username:", self.focused == 0);
            let username_area = Rect::new(4, 6, area.width.saturating_sub(8), 3);
            let username_plane = self.username_input.render(username_area);
            self.render_input_field(&mut plane, 4, 6, username_area.width, username_area.height, &username_plane);

            // Password field
            self.render_field_label(&mut plane, 4, 9, "Password:", self.focused == 1);
            let password_area = Rect::new(4, 10, area.width.saturating_sub(8), 3);
            let password_plane = self.password_input.render(password_area);
            self.render_input_field(&mut plane, 4, 10, password_area.width, password_area.height, &password_plane);

            // Confirm password field
            self.render_field_label(&mut plane, 4, 13, "Confirm:", self.focused == 2);
            let confirm_area = Rect::new(4, 14, area.width.saturating_sub(8), 3);
            let confirm_plane = self.confirm_input.render(confirm_area);
            self.render_input_field(&mut plane, 4, 14, confirm_area.width, confirm_area.height, &confirm_plane);

            // Password strength indicator (live)
            if !self.password_input.password().is_empty() {
                let strength = match self.password_input.password().len() {
                    0..=5 => ("Weak", t.error),
                    6..=9 => ("Medium", t.warning),
                    _ => ("Strong", t.success),
                };
                let strength_text = format!("Strength: {}", strength.0);
                draw_text(&mut plane, 4, 17, &strength_text, strength.1, t.bg, true);
            }

            // Error message
            if let Some(err) = self.error_msg {
                draw_text(&mut plane, 4, 18, &format!("⚠ {}", err), t.error, t.bg, true);
            }

            // Submit hint
            draw_text(&mut plane, 4, 19, "Enter: submit", t.fg_muted, t.bg, false);
        }

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " Tab:switch fields | {}:help | {}:back ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            self.render_help(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        // Reset
        if key.code == KeyCode::Char('r') && key.modifiers.is_empty() && self.submitted {
            self.submitted = false;
            self.error_msg = None;
            self.focused = 0;
            self.username_input.clear();
            self.password_input.clear();
            self.confirm_input.clear();
            self.dirty = true;
            return true;
        }

        if self.submitted { return false; }

        // Tab between fields
        match key.code {
            KeyCode::Tab => {
                self.focused = (self.focused + 1) % 3;
                self.dirty = true;
                return true;
            }
            KeyCode::BackTab => {
                self.focused = if self.focused == 0 { 2 } else { self.focused - 1 };
                self.dirty = true;
                return true;
            }
            KeyCode::Enter => {
                if self.focused == 2 {
                    self.try_submit();
                } else {
                    self.focused = (self.focused + 1) % 3;
                }
                self.dirty = true;
                return true;
            }
            _ => {}
        }

        // Forward to focused input
        let handled = match self.focused {
            0 => self.username_input.handle_key(key),
            1 => self.password_input.handle_key(key),
            2 => self.confirm_input.handle_key(key),
            _ => false,
        };
        if handled {
            self.error_msg = None;
            self.dirty = true;
        }
        handled
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.submitted { return false; }

        // Click on fields to focus them
        if matches!(kind, MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)) {
            if (6..=8).contains(&row) { self.focused = 0; self.dirty = true; return true; }
            if (10..=12).contains(&row) { self.focused = 1; self.dirty = true; return true; }
            if (14..=16).contains(&row) { self.focused = 2; self.dirty = true; return true; }
        }

        // Forward to focused input
        match self.focused {
            0 => self.username_input.handle_mouse(kind, col.saturating_sub(4), row.saturating_sub(6)),
            1 => self.password_input.handle_mouse(kind, col.saturating_sub(4), row.saturating_sub(10)),
            2 => self.confirm_input.handle_mouse(kind, col.saturating_sub(4), row.saturating_sub(14)),
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.username_input.on_theme_change(theme);
        self.password_input.on_theme_change(theme);
        self.confirm_input.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl PasswordInputScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 40u16.min(area.width.saturating_sub(4));
        let hh = 11u16.min(area.height.saturating_sub(4));
        let hx = (area.width - hw) / 2;
        let hy = (area.height - hh) / 2;

        for y in hy..hy + hh {
            for x in hx..hx + hw {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let lines = [
            ("╭────────────────────────────────────╮", true),
            ("│     Password Input Help           │", true),
            ("├────────────────────────────────────┤", true),
            ("│  Tab       Cycle through fields    │", false),
            ("│  Enter     Submit (or next field)   │", false),
            ("│  Click     Focus a field           │", false),
            (&format!("│  {:<10} Dismiss / go back      │", back_key), false),
            ("╰────────────────────────────────────╯", true),
        ];
        for (i, (line, is_border)) in lines.iter().enumerate() {
            let ly = hy + i as u16;
            let lx = (area.width - line.len() as u16) / 2;
            for (j, ch) in line.chars().enumerate() {
                let px = lx + j as u16;
                if px < area.width && ly < area.height {
                    let idx = (ly * area.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if *is_border || "│╭╮├┤╰╯─".contains(ch) { t.outline } else { t.fg };
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }
    }
}
