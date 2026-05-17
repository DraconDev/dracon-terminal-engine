//! Embedded Accessibility scene for the showcase.
//!
//! Demonstrates screen reader support (OSC 99) with visual focus rings,
//! accessibility tree, live announcements, and contrast checker.

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

#[derive(Clone, Copy, PartialEq)]
enum FocusTarget {
    UsernameField,
    PasswordField,
    LoginButton,
    HelpLink,
    RememberCheck,
}

impl FocusTarget {
    fn all() -> &'static [FocusTarget] {
        &[FocusTarget::UsernameField, FocusTarget::PasswordField, FocusTarget::LoginButton, FocusTarget::HelpLink, FocusTarget::RememberCheck]
    }

    fn name(&self) -> &'static str {
        match self {
            FocusTarget::UsernameField => "Username input",
            FocusTarget::PasswordField => "Password input",
            FocusTarget::LoginButton => "Login button",
            FocusTarget::HelpLink => "Help link",
            FocusTarget::RememberCheck => "Remember me checkbox",
        }
    }

    fn role(&self) -> &'static str {
        match self {
            FocusTarget::UsernameField => "textbox",
            FocusTarget::PasswordField => "textbox",
            FocusTarget::LoginButton => "button",
            FocusTarget::HelpLink => "link",
            FocusTarget::RememberCheck => "checkbox",
        }
    }

    fn shortcut(&self) -> &'static str {
        match self {
            FocusTarget::UsernameField => "Tab",
            FocusTarget::PasswordField => "Tab",
            FocusTarget::LoginButton => "Enter",
            FocusTarget::HelpLink => "Enter",
            FocusTarget::RememberCheck => "Space",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            FocusTarget::UsernameField => "Enter your username",
            FocusTarget::PasswordField => "Enter your password (masked)",
            FocusTarget::LoginButton => "Submit login form",
            FocusTarget::HelpLink => "Open help documentation",
            FocusTarget::RememberCheck => "Toggle remember-me option",
        }
    }

    fn state(&self, checked: bool) -> &'static str {
        match self {
            FocusTarget::RememberCheck => if checked { "checked" } else { "unchecked" },
            _ => "",
        }
    }
}

struct Announcement {
    role: String,
    label: String,
    action: String,
_timestamp: u64,
}

pub struct AccessibilityScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    focus_idx: usize,
    focused: FocusTarget,
    checked: bool,
    announcements: Vec<Announcement>,
    tick: u64,
    dirty: bool,
    area: std::cell::Cell<Rect>,
}

impl AccessibilityScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            focus_idx: 0,
            focused: FocusTarget::UsernameField,
            checked: false,
            announcements: Vec::new(),
            tick: 0,
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn advance_focus(&mut self) {
        let targets = FocusTarget::all();
        self.focus_idx = (self.focus_idx + 1) % targets.len();
        self.focused = targets[self.focus_idx];
        self.add_announcement("focus", self.focused.name(), "focused".to_string());
        self.dirty = true;
    }

    fn prev_focus(&mut self) {
        let targets = FocusTarget::all();
        self.focus_idx = if self.focus_idx == 0 { targets.len() - 1 } else { self.focus_idx - 1 };
        self.focused = targets[self.focus_idx];
        self.add_announcement("focus", self.focused.name(), "focused".to_string());
        self.dirty = true;
    }

    fn activate(&mut self) {
        match self.focused {
            FocusTarget::LoginButton => {
                self.add_announcement("button", "Login", "pressed".to_string());
            }
            FocusTarget::HelpLink => {
                self.add_announcement("link", "Help", "activated".to_string());
            }
            FocusTarget::RememberCheck => {
                self.checked = !self.checked;
                let state = self.focused.state(self.checked);
                self.add_announcement("checkbox", "Remember me", format!("toggled {}", state));
            }
            _ => {
                self.add_announcement(self.focused.role(), self.focused.name(), "activated".to_string());
            }
        }
        self.dirty = true;
    }

    fn add_announcement(&mut self, role: &str, label: &str, action: String) {
        self.announcements.push(Announcement {
            role: role.to_string(),
            label: label.to_string(),
            action,
            _timestamp: self.tick,
        });
        if self.announcements.len() > 8 {
            self.announcements.remove(0);
        }
    }

    fn render_form(&self, plane: &mut Plane, x: u16, y: u16) {
        let t = &self.theme;

        // Simulated login form
        draw_text(plane, x, y, "Login Form", t.primary, t.bg, true);
        for dx in 0..30 {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Username field
        let is_focused = self.focused == FocusTarget::UsernameField;
        let field_bg = if is_focused { t.hover_bg } else { t.surface };
        let field_fg = if is_focused { t.primary } else { t.fg_muted };
        draw_text(plane, x, y + 2, "Username:", field_fg, t.bg, false);
        // Input box
        for dx in 0..18 {
            let idx = ((y + 3) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = field_bg;
                plane.cells[idx].transparent = false;
            }
        }
        // Focus ring (colored border when focused)
        if is_focused {
            for dx in 0..18 {
                let top = ((y + 2) * plane.width + x + dx) as usize;
                let bot = ((y + 4) * plane.width + x + dx) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.primary; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.primary; }
            }
            for dy in 2u16..5 {
                let left = (dy * plane.width + x - 1) as usize;
                let right = (dy * plane.width + x + 18) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.primary; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.primary; }
            }
        }
        draw_text(plane, x + 1, y + 3, "admin", t.fg, field_bg, false);

        // Password field
        let is_focused = self.focused == FocusTarget::PasswordField;
        let field_bg = if is_focused { t.hover_bg } else { t.surface };
        let field_fg = if is_focused { t.primary } else { t.fg_muted };
        draw_text(plane, x, y + 6, "Password:", field_fg, t.bg, false);
        for dx in 0..18 {
            let idx = ((y + 7) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = field_bg;
                plane.cells[idx].transparent = false;
            }
        }
        if is_focused {
            for dx in 0..18 {
                let top = ((y + 6) * plane.width + x + dx) as usize;
                let bot = ((y + 8) * plane.width + x + dx) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.primary; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.primary; }
            }
        }
        draw_text(plane, x + 1, y + 7, "••••••", t.fg_muted, field_bg, false);

        // Remember me checkbox
        let is_focused = self.focused == FocusTarget::RememberCheck;
        let check_fg = if is_focused { t.primary } else { t.fg_muted };
        let check_char = if self.checked { '☑' } else { '☐' };
        let check_idx = ((y + 9) * plane.width + x) as usize;
        if check_idx < plane.cells.len() {
            plane.cells[check_idx].char = check_char;
            plane.cells[check_idx].fg = check_fg;
            plane.cells[check_idx].transparent = false;
        }
        draw_text(plane, x + 2, y + 9, "Remember me", check_fg, t.bg, false);

        // Login button
        let is_focused = self.focused == FocusTarget::LoginButton;
        let btn_fg = if is_focused { t.bg } else { t.fg };
        let btn_bg = if is_focused { t.primary } else { t.surface };
        for dx in 0..12 {
            let idx = ((y + 11) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = btn_bg;
                plane.cells[idx].transparent = false;
            }
        }
        draw_text(plane, x + 2, y + 11, "  Login  ", btn_fg, btn_bg, true);

        // Help link
        let is_focused = self.focused == FocusTarget::HelpLink;
        let link_fg = if is_focused { t.primary } else { t.secondary };
        draw_text(plane, x, y + 13, "Need help?", link_fg, t.bg, false);
        if is_focused {
            // Underline for link focus
            for dx in 0..10 {
                let idx = ((y + 14) * plane.width + x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '▔';
                    plane.cells[idx].fg = t.primary;
                }
            }
        }
    }

    fn render_a11y_tree(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;

        draw_text(plane, x, y, "Accessibility Tree", t.primary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let targets = FocusTarget::all();
        for (i, target) in targets.iter().enumerate() {
            let ty = y + 2 + i as u16;
            let is_focused = *target == self.focused;

            // Tree connector
            let connector = if i + 1 < targets.len() { "├─ " } else { "└─ " };
            let indent = "│  ";
            if i + 1 < targets.len() {
                draw_text(plane, x, ty, indent, t.outline, t.bg, false);
            }
            draw_text(plane, x + 2, ty, connector, t.outline, t.bg, false);

            // Role badge
            let role_color = if is_focused { t.primary } else { t.fg_muted };
            let role_badge = format!("[{}]", target.role());
            draw_text(plane, x + 5, ty, &role_badge, role_color, t.bg, is_focused);

            // Label
            let label_color = if is_focused { t.primary } else { t.fg };
            draw_text(plane, x + 5 + role_badge.len() as u16, ty, target.name(), label_color, t.bg, is_focused);

            // Focus indicator
            if is_focused {
                draw_text(plane, x + w.saturating_sub(3), ty, " ◄", t.primary, t.bg, true);
            }
        }
    }

    fn render_announcement_log(&self, plane: &mut Plane, x: u16, y: u16, w: u16, area: Rect) {
        let t = &self.theme;

        draw_text(plane, x, y, "Announcements", t.secondary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        for (i, ann) in self.announcements.iter().rev().take(5).enumerate() {
            let ay = y + 2 + i as u16;
            if ay >= area.height.saturating_sub(2) { break; }

            // Role icon
            let icon = match ann.role.as_str() {
                "focus" => '◉',
                "button" => '▶',
                "link" => '↗',
                "checkbox" => '☑',
                _ => '•',
            };
            let icon_idx = (ay * plane.width + x) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = icon;
                plane.cells[icon_idx].fg = t.primary;
                plane.cells[icon_idx].transparent = false;
            }

            // Announcement text
            let text = format!("{}: {}", ann.label, ann.action);
            draw_text(plane, x + 2, ay, &text, t.fg, t.bg, false);
        }
    }

    fn render_focus_info(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;

        draw_text(plane, x, y, "Focus Info", t.primary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Current focus details
        let info = [
            ("Role:", self.focused.role()),
            ("Label:", self.focused.name()),
            ("Shortcut:", self.focused.shortcut()),
            ("Description:", self.focused.description()),
        ];
        for (i, (key, val)) in info.iter().enumerate() {
            let iy = y + 2 + i as u16;
            draw_text(plane, x, iy, key, t.fg_muted, t.bg, false);
            draw_text(plane, x + 13, iy, val, t.fg, t.bg, false);
        }

        // State (for checkbox)
        let state = self.focused.state(self.checked);
        if !state.is_empty() {
            draw_text(plane, x, y + 6, "State:", t.fg_muted, t.bg, false);
            draw_text(plane, x + 13, y + 6, state, t.primary, t.bg, true);
        }
    }
}

impl Scene for AccessibilityScene {
    fn scene_id(&self) -> &str { "accessibility" }

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
        draw_text(&mut plane, 2, 0, " Accessibility ", t.primary, t.bg, true);
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

        // Layout: Form (left ~30) | A11y tree + info (right)
        self.render_form(&mut plane, 2, 2);

        // Vertical divider
        let div_x = 30u16;
        for y in 2..area.height.saturating_sub(2) {
            let idx = (y * area.width + div_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Right panel
        let panel_x = div_x + 2;
        let panel_w = area.width.saturating_sub(panel_x + 2);

        // Focus info (top)
        self.render_focus_info(&mut plane, panel_x, 2, panel_w);

        // A11y tree (middle)
        self.render_a11y_tree(&mut plane, panel_x, 10, panel_w);

        // Announcement log (bottom)
        let log_y = 10 + FocusTarget::all().len() as u16 + 2;
        self.render_announcement_log(&mut plane, panel_x, log_y, panel_w, area);

        // OSC 99 hint
        draw_text(&mut plane, 2, area.height.saturating_sub(3),
                  "OSC 99: Screen reader announcements via terminal escape sequences", t.fg_muted, t.bg, false);

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(" Tab:next focus | Shift+Tab:prev | Enter:activate | {}:help | {}:back ", help_key, back_key);
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

        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(dracon_terminal_engine::input::event::KeyModifiers::SHIFT) {
                    self.prev_focus();
                } else {
                    self.advance_focus();
                }
                true
            }
            KeyCode::Enter => {
                self.activate();
                true
            }
            KeyCode::Char(' ') => {
                self.activate();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl AccessibilityScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 48u16.min(area.width.saturating_sub(4));
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
            let top = (hy * area.width + x) as usize;
            let bot = ((hy + hh - 1) * area.width + x) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * area.width + hx) as usize;
            let right = (y * area.width + hx + hw - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }
        let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
        for (ch, cx, cy) in corners {
            let idx = (cy * area.width + cx) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
        }

        let title = "Accessibility Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let shortcuts = [
            ("Tab", "Next focus target"),
            ("Shift+Tab", "Previous focus target"),
            ("Enter/Space", "Activate focused element"),
            (back_key, "Back to showcase"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
            draw_text(plane, hx + 16, row, desc, t.fg, t.surface_elevated, false);
        }
    }
}
