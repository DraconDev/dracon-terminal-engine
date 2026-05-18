//! Embedded Password Input scene for the showcase.
//!
//! Demonstrates PasswordInput in a login form with:
//!   - Live password strength meter
//!   - Requirements checklist with live checkmarks
//!   - Show/hide password toggle
//!   - Side panel with security tips

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

struct Requirement {
    label: &'static str,
    check: fn(&str) -> bool,
}

const REQUIREMENTS: &[Requirement] = &[
    Requirement { label: "At least 8 characters", check: |p| p.len() >= 8 },
    Requirement { label: "Contains uppercase (A-Z)", check: |p| p.chars().any(|c| c.is_ascii_uppercase()) },
    Requirement { label: "Contains lowercase (a-z)", check: |p| p.chars().any(|c| c.is_ascii_lowercase()) },
    Requirement { label: "Contains digit (0-9)", check: |p| p.chars().any(|c| c.is_ascii_digit()) },
    Requirement { label: "Contains special char (!@#)", check: |p| p.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:',.<>?/`~".contains(c)) },
];

pub struct PasswordInputScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    username_input: SearchInput,
    password_input: PasswordInput,
    confirm_input: PasswordInput,
    focused: usize,
    submitted: bool,
    error_msg: Option<&'static str>,
    show_password: bool,
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
            show_password: false,
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn password_strength(password: &str) -> (&'static str, f32) {
        if password.is_empty() { return ("None", 0.0); }
        let mut score = 0u32;
        for req in REQUIREMENTS {
            if (req.check)(password) { score += 1; }
        }
        match score {
            0..=1 => ("Weak", 0.2),
            2 => ("Fair", 0.4),
            3 => ("Good", 0.6),
            4 => ("Strong", 0.8),
            _ => ("Excellent", 1.0),
        }
    }

    fn render_field_label(&self, plane: &mut Plane, x: u16, y: u16, label: &str, is_focused: bool) {
        let t = &self.theme;
        let fg = if is_focused { t.primary } else { t.fg_muted };
        for (j, ch) in label.chars().enumerate() {
            let idx = (y * plane.width + x + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = fg;
                plane.cells[idx].style = if is_focused { Styles::BOLD } else { Styles::empty() };
                plane.cells[idx].transparent = false;
            }
        }
        if is_focused {
            draw_text(plane, x + label.len() as u16, y, " ◄", t.primary, t.bg, false);
        }
    }

    fn render_input_field(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, input_plane: &Plane) {
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

    fn render_strength_bar(&self, plane: &mut Plane, x: u16, y: u16, w: u16, password: &str) {
        let t = &self.theme;
        let (label, ratio) = Self::password_strength(password);

        // Strength label
        let color = match label {
            "Weak" => t.error,
            "Fair" => t.warning,
            "Good" => t.info,
            "Strong" => t.success,
            "Excellent" => t.success,
            _ => t.fg_muted,
        };
        draw_text(plane, x, y, &format!("Strength: {}", label), color, t.bg, true);

        // Strength bar
        let bar_y = y + 1;
        let filled = (w as f32 * ratio) as u16;
        for dx in 0..w {
            let idx = (bar_y * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if dx < filled { '█' } else { '░' };
                plane.cells[idx].fg = if dx < filled { color } else { t.fg_muted };
                plane.cells[idx].transparent = false;
            }
        }
    }

    fn render_requirements(&self, plane: &mut Plane, x: u16, y: u16, w: u16, password: &str) {
        let t = &self.theme;

        draw_text(plane, x, y, "Requirements", t.secondary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        for (i, req) in REQUIREMENTS.iter().enumerate() {
            let ry = y + 2 + i as u16;
            let passed = (req.check)(password);
            let icon = if passed { '✓' } else { '○' };
            let fg = if passed { t.success } else { t.fg_muted };

            let icon_idx = (ry * plane.width + x) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = icon;
                plane.cells[icon_idx].fg = fg;
            }

            draw_text(plane, x + 2, ry, req.label, fg, t.bg, false);
        }
    }

    fn render_tips_panel(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16) {
        let t = &self.theme;

        draw_text(plane, x, y, "Security Tips", t.primary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let tips = [
            ("●", "Use unique passwords per site", t.fg),
            ("●", "Longer passwords are stronger", t.fg),
            ("●", "Avoid dictionary words alone", t.fg),
            ("●", "Mix character types", t.fg),
            ("●", "Consider a passphrase", t.fg),
            ("", "", t.fg),
            ("ℹ", "Tab cycles through fields", t.info),
            ("ℹ", "Ctrl+H toggles password visibility", t.info),
        ];

        for (i, (icon, tip, color)) in tips.iter().enumerate() {
            let ty = y + 2 + i as u16;
            if ty >= y + h { break; }
            if !icon.is_empty() {
                let idx = (ty * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = icon.chars().next().unwrap_or(' ');
                    plane.cells[idx].fg = *color;
                }
            }
            if !tip.is_empty() {
                draw_text(plane, x + 2, ty, tip, *color, t.bg, false);
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

        // Layout: form (left ~55%) | side panel (right ~40%)
        let form_w = (area.width * 55 / 100).max(30);
        let panel_x = form_w + 2;
        let panel_w = area.width.saturating_sub(panel_x + 2);

        if self.submitted {
            // Success screen
            draw_text(&mut plane, 4, 3, "✓ Login Successful!", t.success, t.bg, true);
            draw_text(&mut plane, 4, 5, &format!("Welcome, {}!", self.username_input.query()), t.fg, t.bg, false);
            draw_text(&mut plane, 4, 7, "Press R to reset and try again", t.fg_muted, t.bg, false);
            self.render_strength_bar(&mut plane, 4, 9, form_w.saturating_sub(8), self.password_input.password());
        } else {
            // Login form
            draw_text(&mut plane, 4, 2, "Login Form", t.fg, t.bg, true);
            draw_text(&mut plane, 4, 3, "Tab to switch, Enter to submit", t.fg_muted, t.bg, false);

            // Username field
            self.render_field_label(&mut plane, 4, 5, "Username:", self.focused == 0);
            let username_area = Rect::new(4, 6, form_w.saturating_sub(8), 3);
            let username_plane = self.username_input.render(username_area);
            self.render_input_field(&mut plane, 4, 6, username_area.width, username_area.height, &username_plane);

            // Password field
            self.render_field_label(&mut plane, 4, 9, "Password:", self.focused == 1);
            let password_area = Rect::new(4, 10, form_w.saturating_sub(8), 3);
            let password_plane = self.password_input.render(password_area);
            self.render_input_field(&mut plane, 4, 10, password_area.width, password_area.height, &password_plane);

            // Show/hide toggle hint
            let toggle_text = if self.show_password { "visible" } else { "hidden" };
            draw_text(&mut plane, 4, 13, &format!("visibility: {} (Ctrl+H)", toggle_text), t.fg_muted, t.bg, false);

            // Confirm password field
            self.render_field_label(&mut plane, 4, 14, "Confirm:", self.focused == 2);
            let confirm_area = Rect::new(4, 15, form_w.saturating_sub(8), 3);
            let confirm_plane = self.confirm_input.render(confirm_area);
            self.render_input_field(&mut plane, 4, 15, confirm_area.width, confirm_area.height, &confirm_plane);

            // Match indicator
            let password = self.password_input.password();
            let confirm = self.confirm_input.password();
            if !confirm.is_empty() {
                let match_icon = if password == confirm { "✓ Match" } else { "✗ Mismatch" };
                let match_color = if password == confirm { t.success } else { t.error };
                draw_text(&mut plane, 4, 18, match_icon, match_color, t.bg, true);
            }

            // Error message
            if let Some(err) = self.error_msg {
                draw_text(&mut plane, 4, 19, &format!("⚠ {}", err), t.error, t.bg, true);
            }

            // Submit hint
            let submit_y = if self.error_msg.is_some() { 20 } else { 19 };
            draw_text(&mut plane, 4, submit_y, "Enter: submit", t.fg_muted, t.bg, false);
        }

        // ── Side panel: strength + requirements + tips ─────────────────────
        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * plane.width + form_w) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        let password = self.password_input.password();

        // Strength bar
        self.render_strength_bar(&mut plane, panel_x, 2, panel_w, password);

        // Requirements checklist
        self.render_requirements(&mut plane, panel_x, 5, panel_w, password);

        // Tips panel
        let tips_y = 5 + 2 + REQUIREMENTS.len() as u16 + 1;
        if tips_y + 8 < area.height.saturating_sub(2) {
            self.render_tips_panel(&mut plane, panel_x, tips_y, panel_w, area.height.saturating_sub(tips_y + 2));
        }

        // Footer
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " Tab:switch | Ctrl+H:toggle | {}:help | {}:back ",
            self.keybindings.display(actions::HELP).unwrap_or("f1"),
            back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * plane.width + i as u16) as usize;
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

        // Toggle password visibility
        if key.code == KeyCode::Char('h') && key.modifiers.contains(dracon_terminal_engine::input::event::KeyModifiers::CONTROL) {
            self.show_password = !self.show_password;
            self.dirty = true;
            return true;
        }

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

        if matches!(kind, MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)) {
            if (6..=8).contains(&row) { self.focused = 0; self.dirty = true; return true; }
            if (10..=12).contains(&row) { self.focused = 1; self.dirty = true; return true; }
            if (15..=17).contains(&row) { self.focused = 2; self.dirty = true; return true; }
        }

        match self.focused {
            0 => self.username_input.handle_mouse(kind, col.saturating_sub(4), row.saturating_sub(6)),
            1 => self.password_input.handle_mouse(kind, col.saturating_sub(4), row.saturating_sub(10)),
            2 => self.confirm_input.handle_mouse(kind, col.saturating_sub(4), row.saturating_sub(15)),
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
        let hw = 44u16.min(area.width.saturating_sub(4));
        let hh = 12u16.min(area.height.saturating_sub(4));
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
        for x in hx + 1..hx + hw - 1 {
            let top = (hy * plane.width + x) as usize;
            let bot = ((hy + hh - 1) * plane.width + x) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * plane.width + hx) as usize;
            let right = (y * plane.width + hx + hw - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }
        for (ch, cx, cy) in [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)] {
            let idx = (cy * plane.width + cx) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
        }

        let title = "Password Input Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

        let shortcuts = [
            ("Tab", "Cycle through fields"),
            ("Enter", "Submit (or next field)"),
            ("Ctrl+H", "Toggle password visibility"),
            ("Click", "Focus a field"),
            ("R", "Reset after submit"),
            ("B/Esc", "Back to showcase"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
            draw_text(plane, hx + 16, row, desc, t.fg, t.surface_elevated, false);
        }
    }
}
