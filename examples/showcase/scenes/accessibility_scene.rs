//! Embedded Accessibility scene for the showcase.
//!
//! Demonstrates screen reader support (OSC 99) with visual focus rings,
//! accessibility tree, live announcements, and real input widgets.

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::{PasswordInput, SearchInput};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

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
        &[
            FocusTarget::UsernameField,
            FocusTarget::PasswordField,
            FocusTarget::LoginButton,
            FocusTarget::HelpLink,
            FocusTarget::RememberCheck,
        ]
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

/// Draw a rounded focus ring around a rectangular area.
/// Used to visually indicate the accessible focus target.
fn draw_focus_ring(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, color: Color) {
    // Corners
    let corners = [
        ('╭', x, y),
        ('╮', x + w.saturating_sub(1), y),
        ('╰', x, y + h.saturating_sub(1)),
        ('╯', x + w.saturating_sub(1), y + h.saturating_sub(1)),
    ];
    for (ch, cx, cy) in corners {
        if cy < plane.height && cx < plane.width {
            let idx = (cy * plane.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = color;
                plane.cells[idx].transparent = false;
            }
        }
    }
    // Top and bottom borders
    for dx in 1..w.saturating_sub(1) {
        let cx = x + dx;
        if cx >= plane.width { break; }
        if y < plane.height {
            let top = (y * plane.width + cx) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = color; plane.cells[top].transparent = false; }
        }
        let by = y + h.saturating_sub(1);
        if by < plane.height {
            let bot = (by * plane.width + cx) as usize;
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = color; plane.cells[bot].transparent = false; }
        }
    }
    // Left and right borders
    for dy in 1..h.saturating_sub(1) {
        let cy = y + dy;
        if cy >= plane.height { break; }
        if x < plane.width {
            let left = (cy * plane.width + x) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = color; plane.cells[left].transparent = false; }
        }
        let rx = x + w.saturating_sub(1);
        if rx < plane.width {
            let right = (cy * plane.width + rx) as usize;
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = color; plane.cells[right].transparent = false; }
        }
    }
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
    // Real input widgets
    username_input: SearchInput,
    password_input: PasswordInput,
}

// Layout constants — form elements positioned absolutely within the scene
const FORM_X: u16 = 2;
const FORM_Y: u16 = 2;
const USERNAME_LABEL_Y: u16 = FORM_Y;
const USERNAME_INPUT_Y: u16 = FORM_Y + 1;
const USERNAME_RING: (u16, u16, u16, u16) = (FORM_X, USERNAME_INPUT_Y, 24, 3); // x, y, w, h
const PASSWORD_LABEL_Y: u16 = USERNAME_INPUT_Y + 3 + 1; // 1 row gap after username ring
const PASSWORD_INPUT_Y: u16 = PASSWORD_LABEL_Y + 1;
const PASSWORD_RING: (u16, u16, u16, u16) = (FORM_X, PASSWORD_INPUT_Y, 24, 3);
const CHECK_Y: u16 = PASSWORD_INPUT_Y + 3 + 1;
const BUTTON_Y: u16 = CHECK_Y + 2;
const LINK_Y: u16 = BUTTON_Y + 2;

impl AccessibilityScene {
    pub fn new(theme: Theme) -> Self {
        let username_input = SearchInput::new(WidgetId::new(200))
            .with_theme(theme.clone());
        let password_input = PasswordInput::new(WidgetId::new(201))
            .with_theme(theme.clone())
            .with_mask_char('•')
            .with_placeholder("type here…");

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
            username_input,
            password_input,
        }
    }

    fn advance_focus(&mut self) {
        let targets = FocusTarget::all();
        self.focus_idx = (self.focus_idx + 1) % targets.len();
        self.focused = targets[self.focus_idx];
        self.update_widget_focus();
        self.add_announcement("focus", self.focused.name(), "focused".to_string());
        self.dirty = true;
    }

    fn prev_focus(&mut self) {
        let targets = FocusTarget::all();
        self.focus_idx = if self.focus_idx == 0 { targets.len() - 1 } else { self.focus_idx - 1 };
        self.focused = targets[self.focus_idx];
        self.update_widget_focus();
        self.add_announcement("focus", self.focused.name(), "focused".to_string());
        self.dirty = true;
    }

    fn set_focus(&mut self, target: FocusTarget) {
        let targets = FocusTarget::all();
        if let Some(idx) = targets.iter().position(|t| *t == target) {
            self.focus_idx = idx;
            self.focused = target;
            self.update_widget_focus();
            self.add_announcement("focus", self.focused.name(), "focused".to_string());
            self.dirty = true;
        }
    }

    fn update_widget_focus(&mut self) {
        // Focus/blur the real input widgets based on current focus target
        match self.focused {
            FocusTarget::UsernameField => {
                self.username_input.on_focus();
                self.password_input.on_blur();
            }
            FocusTarget::PasswordField => {
                self.username_input.on_blur();
                self.password_input.on_focus();
            }
            _ => {
                self.username_input.on_blur();
                self.password_input.on_blur();
            }
        }
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

    fn render_form(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let x = FORM_X;
        let max_x = 30u16.min(area.width); // clip at divider

        // Title
        draw_text_clipped(plane, x, FORM_Y, "Login Form", max_x, t.primary, t.bg, true);
        for dx in 0..26 {
            let dx_pos = x + dx;
            if dx_pos >= max_x { break; }
            let idx = ((FORM_Y + 1) * plane.width + dx_pos) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // --- Username field ---
        let u_label_y = FORM_Y + 2;
        let u_input_y = FORM_Y + 3;
        let is_u_focused = self.focused == FocusTarget::UsernameField;
        let label_fg = if is_u_focused { t.primary } else { t.fg_muted };
        draw_text_clipped(plane, x, u_label_y, "Username:", max_x, label_fg, t.bg, false);

        // Render real SearchInput widget
        let u_area = Rect::new(x + 1, u_input_y, 22, 1);
        self.username_input.set_area(u_area);
        let u_plane = self.username_input.render(u_area);
        blit_to(plane, &u_plane, (x + 1) as usize, u_input_y as usize);

        // Focus ring around input (1 row above, 1 row below, 1 col on each side)
        if is_u_focused {
            draw_focus_ring(plane, x, u_input_y.saturating_sub(1), 24, 3, t.primary);
        }

        // --- Password field ---
        let p_label_y = u_input_y + 3;
        let p_input_y = p_label_y + 1;
        let is_p_focused = self.focused == FocusTarget::PasswordField;
        let label_fg = if is_p_focused { t.primary } else { t.fg_muted };
        draw_text_clipped(plane, x, p_label_y, "Password:", max_x, label_fg, t.bg, false);

        // Render real PasswordInput widget
        let p_area = Rect::new(x + 1, p_input_y, 22, 1);
        self.password_input.set_area(p_area);
        let p_plane = self.password_input.render(p_area);
        blit_to(plane, &p_plane, (x + 1) as usize, p_input_y as usize);

        // Focus ring
        if is_p_focused {
            draw_focus_ring(plane, x, p_input_y.saturating_sub(1), 24, 3, t.primary);
        }

        // --- Remember me checkbox ---
        let check_y = p_input_y + 2;
        let is_c_focused = self.focused == FocusTarget::RememberCheck;
        let check_fg = if is_c_focused { t.primary } else { t.fg_muted };
        let check_char = if self.checked { '☑' } else { '☐' };
        if check_y < plane.height && x < plane.width {
            let idx = (check_y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = check_char;
                plane.cells[idx].fg = check_fg;
                plane.cells[idx].transparent = false;
            }
        }
        draw_text_clipped(plane, x + 2, check_y, "Remember me", max_x, check_fg, t.bg, false);
        if is_c_focused {
            // Focus highlight on the checkbox row
            draw_focus_ring(plane, x, check_y, 14, 1, t.primary);
        }

        // --- Login button ---
        let btn_y = check_y + 2;
        let is_b_focused = self.focused == FocusTarget::LoginButton;
        let btn_fg = if is_b_focused { t.bg } else { t.fg };
        let btn_bg = if is_b_focused { t.primary } else { t.surface };
        for dx in 0..12u16 {
            let bx = x + dx;
            if bx >= max_x { break; }
            if btn_y < plane.height {
                let idx = (btn_y * plane.width + bx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = btn_bg;
                    plane.cells[idx].transparent = false;
                }
            }
        }
        draw_text_clipped(plane, x + 1, btn_y, "  Login  ", max_x, btn_fg, btn_bg, true);
        if is_b_focused {
            draw_focus_ring(plane, x.saturating_sub(1), btn_y.saturating_sub(1), 14, 3, t.primary);
        }

        // --- Help link ---
        let link_y = btn_y + 2;
        let is_l_focused = self.focused == FocusTarget::HelpLink;
        let link_fg = if is_l_focused { t.primary } else { t.secondary };
        draw_text_clipped(plane, x, link_y, "Need help?", max_x, link_fg, t.bg, false);
        if is_l_focused {
            // Underline for link focus
            for dx in 0..10u16 {
                let ux = x + dx;
                if ux >= max_x { break; }
                let uy = link_y + 1;
                if uy < plane.height {
                    let idx = (uy * plane.width + ux) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '▔';
                        plane.cells[idx].fg = t.primary;
                    }
                }
            }
        }
    }

    fn render_a11y_tree(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;
        let max_x = x + w;

        draw_text_clipped(plane, x, y, "Accessibility Tree", max_x, t.primary, t.bg, true);
        for dx in 0..w {
            let dx_pos = x + dx;
            if dx_pos >= max_x { break; }
            let idx = ((y + 1) * plane.width + dx_pos) as usize;
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
            draw_text_clipped(plane, x, ty, "│", max_x, t.outline, t.bg, false);
            draw_text_clipped(plane, x + 2, ty, connector, max_x, t.outline, t.bg, false);

            // Role badge
            let role_color = if is_focused { t.primary } else { t.fg_muted };
            let role_badge = format!("[{}]", target.role());
            draw_text_clipped(plane, x + 5, ty, &role_badge, max_x, role_color, t.bg, is_focused);

            // Label
            let label_color = if is_focused { t.primary } else { t.fg };
            draw_text_clipped(plane, x + 5 + role_badge.len() as u16, ty, target.name(), max_x, label_color, t.bg, is_focused);

            // Focus indicator
            if is_focused && x + w.saturating_sub(3) < max_x {
                draw_text_clipped(plane, x + w.saturating_sub(3), ty, " ◄", max_x, t.primary, t.bg, true);
            }
        }
    }

    fn render_announcement_log(&self, plane: &mut Plane, x: u16, y: u16, w: u16, area: Rect) {
        let t = &self.theme;
        let max_x = x + w;

        draw_text_clipped(plane, x, y, "Announcements", max_x, t.secondary, t.bg, true);
        for dx in 0..w {
            let dx_pos = x + dx;
            if dx_pos >= max_x { break; }
            let idx = ((y + 1) * plane.width + dx_pos) as usize;
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
            if ay < plane.height && x < plane.width {
                let icon_idx = (ay * plane.width + x) as usize;
                if icon_idx < plane.cells.len() {
                    plane.cells[icon_idx].char = icon;
                    plane.cells[icon_idx].fg = t.primary;
                    plane.cells[icon_idx].transparent = false;
                }
            }

            // Announcement text
            let text = format!("{}: {}", ann.label, ann.action);
            draw_text_clipped(plane, x + 2, ay, &text, max_x, t.fg, t.bg, false);
        }
    }

    fn render_focus_info(&self, plane: &mut Plane, x: u16, y: u16, w: u16) {
        let t = &self.theme;
        let max_x = x + w;

        draw_text_clipped(plane, x, y, "Focus Info", max_x, t.primary, t.bg, true);
        for dx in 0..w {
            let dx_pos = x + dx;
            if dx_pos >= max_x { break; }
            let idx = ((y + 1) * plane.width + dx_pos) as usize;
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
            draw_text_clipped(plane, x, iy, key, max_x, t.fg_muted, t.bg, false);
            draw_text_clipped(plane, x + 13, iy, val, max_x, t.fg, t.bg, false);
        }

        // State (for checkbox)
        let state = self.focused.state(self.checked);
        if !state.is_empty() {
            draw_text_clipped(plane, x, y + 6, "State:", max_x, t.fg_muted, t.bg, false);
            draw_text_clipped(plane, x + 13, y + 6, state, max_x, t.primary, t.bg, true);
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
        self.render_form(&mut plane, area);

        // Vertical divider
        let div_x = 30u16.min(area.width.saturating_sub(1));
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
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(&mut plane, area, &self.theme, "Accessibility — Help", &[
                ("Tab", "Next focus target"),
                ("Shift+Tab", "Previous focus target"),
                ("Enter/Space", "Activate focused element"),
                ("Type", "Enter text in input fields"),
                (back_key, "Back"),
            ]);
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
                if key.modifiers.contains(KeyModifiers::SHIFT) {
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
                if self.focused == FocusTarget::RememberCheck
                    || self.focused == FocusTarget::LoginButton
                    || self.focused == FocusTarget::HelpLink
                {
                    self.activate();
                } else {
                    // In text fields, space types a space
                    self.forward_key_to_input(key);
                }
                true
            }
            KeyCode::Char(c) => {
                // Allow typing with no modifiers or just SHIFT (uppercase)
                let mods = key.modifiers;
                if mods.is_empty() || mods == KeyModifiers::SHIFT {
                    self.forward_key_to_input(key);
                }
                true
            }
            KeyCode::Backspace if key.modifiers.is_empty() => {
                self.forward_key_to_input(key);
                true
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End => {
                self.forward_key_to_input(key);
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();

        // Username input: row FORM_Y+3, cols FORM_X+1..FORM_X+23
        let u_input_y = FORM_Y + 3;
        if col >= FORM_X + 1 && col < FORM_X + 23 && row == u_input_y {
            if let MouseEventKind::Down(_) = kind {
                self.set_focus(FocusTarget::UsernameField);
            }
            // Forward mouse to input widget
            let rel_col = col.saturating_sub(FORM_X + 1);
            let rel_row = 0u16;
            self.username_input.handle_mouse(kind, rel_col, rel_row);
            self.dirty = true;
            return true;
        }

        // Password input: row FORM_Y+5, cols FORM_X+1..FORM_X+23
        let p_label_y = u_input_y + 3;
        let p_input_y = p_label_y + 1;
        if col >= FORM_X + 1 && col < FORM_X + 23 && row == p_input_y {
            if let MouseEventKind::Down(_) = kind {
                self.set_focus(FocusTarget::PasswordField);
            }
            let rel_col = col.saturating_sub(FORM_X + 1);
            let rel_row = 0u16;
            self.password_input.handle_mouse(kind, rel_col, rel_row);
            self.dirty = true;
            return true;
        }

        if let MouseEventKind::Down(_) = kind {
            // Remember checkbox
            let check_y = p_input_y + 2;
            if col >= FORM_X && col < FORM_X + 14 && row == check_y {
                self.set_focus(FocusTarget::RememberCheck);
                self.activate();
                return true;
            }

            // Login button
            let btn_y = check_y + 2;
            if col >= FORM_X && col < FORM_X + 12 && row == btn_y {
                self.set_focus(FocusTarget::LoginButton);
                self.activate();
                return true;
            }

            // Help link
            let link_y = btn_y + 2;
            if col >= FORM_X && col < FORM_X + 10 && row == link_y {
                self.set_focus(FocusTarget::HelpLink);
                self.activate();
                return true;
            }

            // A11y tree items: right panel
            let panel_x = 30u16.min(area.width.saturating_sub(1)) + 2;
            if col >= panel_x && row >= 12 && row < 12 + FocusTarget::all().len() as u16 {
                let idx = (row - 12) as usize;
                let targets = FocusTarget::all();
                if idx < targets.len() {
                    self.set_focus(targets[idx]);
                    return true;
                }
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.username_input.on_theme_change(theme);
        self.password_input.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl AccessibilityScene {
    fn forward_key_to_input(&mut self, key: KeyEvent) {
        match self.focused {
            FocusTarget::UsernameField => {
                self.username_input.handle_key(key);
                self.dirty = true;
            }
            FocusTarget::PasswordField => {
                self.password_input.handle_key(key);
                self.dirty = true;
            }
            _ => {}
        }
    }
}
