//! Embedded Login Screen scene for the showcase.
//!
//! Demonstrates a full login form with:
//!   - Centered card layout with border
//!   - Real SearchInput (username) + PasswordInput ×2
//!   - Password strength meter + requirements checklist
//!   - Show/hide password toggle
//!   - Error state simulation (wrong password)
//!   - Success state with welcome message

use crate::scenes::shared_helpers::{blit_to, draw_focus_ring, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::password_input::PasswordInput;
use dracon_terminal_engine::framework::widgets::search_input::SearchInput;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind, MouseButton};
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
    Requirement { label: "Contains special (!@#$)", check: |p| p.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:',.<>?/`~".contains(c)) },
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
        let username_input = SearchInput::new(WidgetId::new(1)).with_theme(theme.clone());
        let password_input = PasswordInput::new(WidgetId::new(2)).with_theme(theme.clone());
        let confirm_input = PasswordInput::new(WidgetId::new(3)).with_theme(theme.clone()).with_mask_char('●');

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

        // ── Header ─────────────────────────────────────────────
        draw_text(&mut plane, 2, 0, " Login Screen ", t.primary, t.bg, true);
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

        // ── Layout: form (left ~55%) | side panel (right ~40%) ───────────
        let form_w = (area.width * 55 / 100).max(30);
        let panel_x = form_w + 2;
        let panel_w = area.width.saturating_sub(panel_x + 2);

        if self.submitted {
            // ── Success Screen ──────────────────────────────────
            let card_x = 4u16;
            let card_w = form_w.saturating_sub(8);
            let card_h = 8;
            let card_y = 3;

            // Card background
            for y in card_y..card_y + card_h {
                for x in card_x..card_x + card_w {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            draw_focus_ring(&mut plane, card_x, card_y, card_w, card_h, t.primary);

            // Success content
            draw_text(&mut plane, card_x + 2, card_y + 1, "✓ Login Successful!", t.success, t.surface_elevated, true);
            draw_text(&mut plane, card_x + 2, card_y + 3, &format!("Welcome back, {}!", self.username_input.query()), t.fg, t.surface_elevated, false);
            draw_text(&mut plane, card_x + 2, card_y + 5, "Your session is now active.", t.fg_muted, t.surface_elevated, false);
            draw_text(&mut plane, card_x + 2, card_y + 6, "Press R to reset and try again", t.secondary, t.surface_elevated, false);

        } else {
            // ── Login Form Card ──────────────────────────────────
            let card_x = 4u16;
            let card_w = form_w.saturating_sub(8);
            let card_h = area.height.saturating_sub(8);
            let card_y = 2u16;

            // Card background (elevated surface)
            for y in card_y..card_y + card_h {
                for x in card_x..card_x + card_w {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            // Card border
            for dx in card_x..card_x + card_w {
                let top_idx = (card_y * area.width + dx) as usize;
                let bot_idx = ((card_y + card_h - 1) * area.width + dx) as usize;
                if top_idx < plane.cells.len() { plane.cells[top_idx].char = '─'; plane.cells[top_idx].fg = t.outline; }
                if bot_idx < plane.cells.len() { plane.cells[bot_idx].char = '─'; plane.cells[bot_idx].fg = t.outline; }
            }
            for dy in card_y..card_y + card_h {
                let left_idx = (dy * area.width + card_x) as usize;
                let right_idx = (dy * area.width + card_x + card_w - 1) as usize;
                if left_idx < plane.cells.len() { plane.cells[left_idx].char = '│'; plane.cells[left_idx].fg = t.outline; }
                if right_idx < plane.cells.len() { plane.cells[right_idx].char = '│'; plane.cells[right_idx].fg = t.outline; }
            }
            // Corners
            let corners = [
                (card_y, card_x, '╭'),
                (card_y, card_x + card_w - 1, '╮'),
                (card_y + card_h - 1, card_x, '╰'),
                (card_y + card_h - 1, card_x + card_w - 1, '╯'),
            ];
            for (cy, cx, ch) in corners {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
            }

            // Form title
            draw_text(&mut plane, card_x + 2, card_y + 1, "Sign In", t.primary, t.surface_elevated, true);
            draw_text(&mut plane, card_x + 2, card_y + 2, "Enter your credentials below", t.fg_muted, t.surface_elevated, false);

            // ── Username field ─────────────────────────────────
            let field_x = card_x + 2;
            let field_y = card_y + 4;
            let field_w = card_w.saturating_sub(4);

            let label_x = field_x;
            draw_text(&mut plane, label_x, field_y, "Username", if self.focused == 0 { t.primary } else { t.fg_muted }, t.surface_elevated, self.focused == 0);
            if self.focused == 0 {
                draw_text(&mut plane, label_x + 9, field_y, "◄", t.primary, t.surface_elevated, false);
            }

            let input_area = Rect::new(field_x, field_y + 1, field_w, 3);
            blit_to(&mut plane, &self.username_input.render(input_area), field_x as usize, (field_y + 1) as usize);

            // ── Password field ─────────────────────────────────
            let pwd_y = field_y + 5;
            draw_text(&mut plane, label_x, pwd_y, "Password", if self.focused == 1 { t.primary } else { t.fg_muted }, t.surface_elevated, self.focused == 1);
            if self.focused == 1 {
                draw_text(&mut plane, label_x + 9, pwd_y, "◄", t.primary, t.surface_elevated, false);
            }

            let pwd_input_area = Rect::new(field_x, pwd_y + 1, field_w, 3);
            blit_to(&mut plane, &self.password_input.render(pwd_input_area), field_x as usize, (pwd_y + 1) as usize);

            // Visibility hint
            let toggle_text = if self.show_password { "show" } else { "hide" };
            draw_text(&mut plane, label_x, pwd_y + 4, &format!("[Ctrl+H] {}", toggle_text), t.fg_muted, t.surface_elevated, false);

            // ── Confirm field ───────────────────────────────────
            let conf_y = pwd_y + 6;
            draw_text(&mut plane, label_x, conf_y, "Confirm", if self.focused == 2 { t.primary } else { t.fg_muted }, t.surface_elevated, self.focused == 2);
            if self.focused == 2 {
                draw_text(&mut plane, label_x + 8, conf_y, "◄", t.primary, t.surface_elevated, false);
            }

            let conf_input_area = Rect::new(field_x, conf_y + 1, field_w, 3);
            blit_to(&mut plane, &self.confirm_input.render(conf_input_area), field_x as usize, (conf_y + 1) as usize);

            // Match indicator
            let password = self.password_input.password();
            let confirm = self.confirm_input.password();
            if !confirm.is_empty() && password.len() >= 6 {
                let match_icon = if password == confirm { "✓ passwords match" } else { "✗ passwords don't match" };
                let match_color = if password == confirm { t.success } else { t.error };
                draw_text(&mut plane, label_x, conf_y + 4, match_icon, match_color, t.surface_elevated, false);
            }

            // Error message
            if let Some(err) = self.error_msg {
                draw_text(&mut plane, label_x, conf_y + 5, &format!("⚠ {}", err), t.error, t.surface_elevated, true);
            }

            // Submit hint
            draw_text(&mut plane, label_x, conf_y + 6, "Enter to submit  |  Tab to switch fields", t.fg_muted, t.surface_elevated, false);
        }

        // ── Side Panel ──────────────────────────────────────────
        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * area.width + form_w) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        let password = self.password_input.password();

        // Panel background (subtle)
        for y in 1..area.height.saturating_sub(1) {
            for x in panel_x..panel_x + panel_w {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    if plane.cells[idx].transparent {
                        plane.cells[idx].bg = t.surface;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // ── Strength meter ─────────────────────────────────────
        let strength_y = 2;
        let (label, ratio) = Self::password_strength(password);
        let color = match label {
            "Weak" => t.error,
            "Fair" => t.warning,
            "Good" => t.info,
            _ => t.success,
        };

        draw_text(&mut plane, panel_x, strength_y, "Strength", t.secondary, t.surface, true);
        draw_text(&mut plane, panel_x, strength_y + 1, label, color, t.surface, true);

        let bar_y = strength_y + 3;
        let filled = (panel_w as f32 * ratio) as u16;
        for dx in 0..panel_w {
            let idx = (bar_y * area.width + panel_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if dx < filled { '█' } else { '░' };
                plane.cells[idx].fg = if dx < filled { color } else { t.fg_muted };
                plane.cells[idx].transparent = false;
            }
        }

        // ── Requirements checklist ─────────────────────────────
        let req_y = strength_y + 6;
        draw_text(&mut plane, panel_x, req_y, "Requirements", t.secondary, t.surface, true);
        for dx in 0..panel_w {
            let idx = ((req_y + 1) * area.width + panel_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        for (i, req) in REQUIREMENTS.iter().enumerate() {
            let ry = req_y + 2 + i as u16;
            let passed = (req.check)(password);
            let icon = if passed { '✓' } else { '○' };
            let fg = if passed { t.success } else { t.fg_muted };

            let icon_idx = (ry * area.width + panel_x) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = icon;
                plane.cells[icon_idx].fg = fg;
            }
            draw_text(&mut plane, panel_x + 2, ry, req.label, fg, t.surface, false);
        }

        // ── Security tips ──────────────────────────────────────
        let tips_y = req_y + 2 + REQUIREMENTS.len() as u16 + 1;
        if tips_y + 8 < area.height.saturating_sub(2) {
            draw_text(&mut plane, panel_x, tips_y, "Security Tips", t.secondary, t.surface, true);
            for dx in 0..panel_w {
                let idx = ((tips_y + 1) * area.width + panel_x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }

            let tips = [
                ("Use unique passwords", t.fg),
                ("Longer is stronger", t.fg),
                ("Mix character types", t.fg),
                ("Consider a passphrase", t.fg),
                ("Never reuse passwords", t.fg),
            ];

            for (i, (tip_text, tip_color)) in tips.iter().enumerate() {
                let ty = tips_y + 2 + i as u16;
                let idx = (ty * area.width + panel_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '●';
                    plane.cells[idx].fg = t.fg_muted;
                }
                draw_text(&mut plane, panel_x + 2, ty, tip_text, *tip_color, t.surface, false);
            }
        }

        // ── Footer ────────────────────────────────────────────
        let fy = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (fy * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        let nav = " Tab: switch | Ctrl+H: toggle | Enter: submit | ?: help | Esc: back ";
        draw_text(&mut plane, 2, fy, nav, t.fg_muted, t.surface, false);

        if self.show_help {
            render_help_overlay(&mut plane, area, t, "Login Screen Help", &[
                ("Tab", "Switch between fields"),
                ("Enter", "Submit form"),
                ("Ctrl+H", "Toggle password visibility"),
                ("Click", "Focus a field"),
                ("R", "Reset after success"),
                ("Esc", "Back to showcase"),
            ]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
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
            return true;
        }

        if self.submitted { return false; }

        // Toggle password visibility
        if key.code == KeyCode::Char('h') && key.modifiers.contains(dracon_terminal_engine::input::event::KeyModifiers::CONTROL) {
            self.show_password = !self.show_password;
            return true;
        }

        // Tab between fields
        match key.code {
            KeyCode::Tab => {
                self.focused = (self.focused + 1) % 3;
                return true;
            }
            KeyCode::BackTab => {
                self.focused = if self.focused == 0 { 2 } else { self.focused - 1 };
                return true;
            }
            KeyCode::Enter => {
                if self.focused == 2 {
                    self.try_submit();
                } else {
                    self.focused = (self.focused + 1) % 3;
                }
                return true;
            }
            _ => {}
        }

        // Forward to focused input
        match self.focused {
            0 => self.username_input.handle_key(key),
            1 => self.password_input.handle_key(key),
            2 => self.confirm_input.handle_key(key),
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.submitted { return false; }

        if let MouseEventKind::Down(MouseButton::Left) = kind {
            // Check which field was clicked
            let area = self.area.get();
            let form_w = (area.width * 55 / 100).max(30);
            let card_x = 4u16;
            let card_w = form_w.saturating_sub(8);

            // Username field (row 7-9)
            if col >= card_x && col < card_x + card_w && (7..=9).contains(&row) {
                self.focused = 0;
                return true;
            }
            // Password field (row 12-14)
            if col >= card_x && col < card_x + card_w && (12..=14).contains(&row) {
                self.focused = 1;
                return true;
            }
            // Confirm field (row 18-20)
            if col >= card_x && col < card_x + card_w && (18..=20).contains(&row) {
                self.focused = 2;
                return true;
            }
        }

        // Forward to focused input
        match self.focused {
            0 => {
                let rel_col = col.saturating_sub(6);
                let rel_row = row.saturating_sub(7);
                self.username_input.handle_mouse(kind, rel_col, rel_row)
            }
            1 => {
                let rel_col = col.saturating_sub(6);
                let rel_row = row.saturating_sub(12);
                self.password_input.handle_mouse(kind, rel_col, rel_row)
            }
            2 => {
                let rel_col = col.saturating_sub(6);
                let rel_row = row.saturating_sub(18);
                self.confirm_input.handle_mouse(kind, rel_col, rel_row)
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.username_input.on_theme_change(theme);
        self.password_input.on_theme_change(theme);
        self.confirm_input.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}