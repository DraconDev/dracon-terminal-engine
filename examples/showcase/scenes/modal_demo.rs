//! Embedded Modal Demo scene for the showcase.
//!
//! Demonstrates ConfirmDialog, Modal, and Toast with z-index layering.
//! Rich settings panel base screen with dimmed backdrop when modals open.

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

struct SettingItem {
    label: &'static str,
    value: &'static str,
    category: &'static str,
}

const SETTINGS: &[SettingItem] = &[
    SettingItem { label: "Theme", value: "Nord", category: "Appearance" },
    SettingItem { label: "Font Size", value: "14pt", category: "Appearance" },
    SettingItem { label: "Line Numbers", value: "On", category: "Editor" },
    SettingItem { label: "Word Wrap", value: "Off", category: "Editor" },
    SettingItem { label: "Auto Save", value: "30s", category: "Editor" },
    SettingItem { label: "Notifications", value: "Enabled", category: "System" },
    SettingItem { label: "Telemetry", value: "Disabled", category: "System" },
    SettingItem { label: "Update Channel", value: "Stable", category: "System" },
    SettingItem { label: "Shell", value: "/bin/bash", category: "Terminal" },
    SettingItem { label: "Scrollback", value: "10000", category: "Terminal" },
];

struct ToastEntry {
    message: String,
    kind: &'static str, // "success", "warning", "error", "info"
    created: u64,
}

pub struct ModalDemoScene {
    theme: Theme,
    show_help: bool,
    show_confirm: bool,
    dirty: bool,
    toasts: Vec<ToastEntry>,
    confirm_title: String,
    confirm_message: String,
    confirm_action: &'static str, // what action triggered the confirm
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
    tick: u64,
    selected_setting: usize,
}

impl ModalDemoScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            show_help: false,
            show_confirm: false,
            dirty: true,
            toasts: Vec::new(),
            confirm_title: String::new(),
            confirm_message: String::new(),
            confirm_action: "",
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            tick: 0,
            selected_setting: 0,
        }
    }

    fn add_toast(&mut self, message: &str, kind: &'static str) {
        self.toasts.push(ToastEntry {
            message: message.to_string(),
            kind,
            created: self.tick,
        });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
        self.dirty = true;
    }

    fn request_confirm(&mut self, title: &str, message: &str, action: &'static str) {
        self.confirm_title = title.to_string();
        self.confirm_message = message.to_string();
        self.confirm_action = action;
        self.show_confirm = true;
        self.dirty = true;
    }

    fn dim_color(&self, base: Color) -> Color {
        // Dim a color by blending toward bg
        let bg = self.theme.bg;
        match (base, bg) {
            (Color::Rgb(r, g, b), Color::Rgb(br, bg_, bb)) => {
                Color::Rgb(
                    r / 3 + br * 2 / 3,
                    g / 3 + bg_ * 2 / 3,
                    b / 3 + bb * 2 / 3,
                )
            }
            _ => base,
        }
    }

    fn render_settings(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, dimmed: bool) {
        let t = &self.theme;

        // Title
        draw_text(plane, x, y, "Settings", t.primary, t.bg, true);
        draw_text(plane, x + 10, y, &format!("({} items)", SETTINGS.len()), t.fg_muted, t.bg, false);

        // Divider
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
                if dimmed { plane.cells[idx].fg = self.dim_color(t.outline); }
            }
        }

        // Group by category
        let mut last_cat = "";
        let mut row = y + 2;
        for (i, setting) in SETTINGS.iter().enumerate() {
            if row >= y + h - 1 { break; }

            // Category header
            if setting.category != last_cat {
                if !last_cat.is_empty() { row += 1; }
                draw_text(plane, x, row, setting.category, t.secondary, t.bg, true);
                if dimmed { draw_text(plane, x, row, setting.category, self.dim_color(t.secondary), t.bg, true); }
                row += 1;
                last_cat = setting.category;
            }

            // Setting row
            let is_selected = i == self.selected_setting && !dimmed;
            let row_bg = if is_selected { t.hover_bg } else if dimmed { self.dim_color(t.surface) } else { t.surface };
            let label_fg = if dimmed { self.dim_color(t.fg) } else if is_selected { t.primary } else { t.fg };
            let value_fg = if dimmed { self.dim_color(t.fg_muted) } else { t.fg_muted };

            // Row background
            for dx in 0..w {
                let idx = (row * plane.width + x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = row_bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Selection indicator
            if is_selected {
                let arrow_idx = (row * plane.width + x) as usize;
                if arrow_idx < plane.cells.len() {
                    plane.cells[arrow_idx].char = '►';
                    plane.cells[arrow_idx].fg = t.primary;
                }
            }

            // Label + value
            draw_text(plane, x + 2, row, setting.label, label_fg, row_bg, is_selected);
            let val_x = x + w.saturating_sub(setting.value.len() as u16 + 2);
            draw_text(plane, val_x, row, setting.value, value_fg, row_bg, false);

            row += 1;
        }
    }

    fn render_toasts(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;

        // Stack toasts in bottom-right
        for (i, toast) in self.toasts.iter().rev().take(3).enumerate() {
            let ty = area.height.saturating_sub(3 + i as u16 * 2);
            let tw = (toast.message.len() as u16 + 4).min(area.width.saturating_sub(4));
            let tx = area.width.saturating_sub(tw + 2);

            let (fg, bg) = match toast.kind {
                "success" => (t.success, t.success_bg),
                "warning" => (t.warning, Color::Rgb(60, 50, 10)),
                "error" => (t.error, Color::Rgb(60, 10, 10)),
                _ => (t.info, Color::Rgb(10, 40, 60)),
            };

            // Toast background
            for dx in 0..tw {
                let idx = (ty * plane.width + tx + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Toast icon
            let icon = match toast.kind {
                "success" => '✓',
                "warning" => '⚠',
                "error" => '✗',
                _ => 'ℹ',
            };
            let icon_idx = (ty * plane.width + tx) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = icon;
                plane.cells[icon_idx].fg = fg;
                plane.cells[icon_idx].bg = bg;
            }

            // Toast text
            draw_text(plane, tx + 2, ty, &toast.message, fg, bg, true);
        }
    }

    fn render_dimmed_backdrop(&self, plane: &mut Plane, area: Rect, exclude: Option<(u16, u16, u16, u16)>) {
        let t = &self.theme;
        let dim_bg = self.dim_color(t.bg);

        for y in 0..area.height {
            for x in 0..area.width {
                // Skip the excluded region (the modal itself)
                if let Some((ex, ey, ew, eh)) = exclude {
                    if x >= ex && x < ex + ew && y >= ey && y < ey + eh {
                        continue;
                    }
                }
                let idx = (y * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    // Blend current cell toward dim bg
                    plane.cells[idx].fg = self.dim_color(plane.cells[idx].fg);
                    plane.cells[idx].bg = dim_bg;
                }
            }
        }
    }
}

impl Scene for ModalDemoScene {
    fn scene_id(&self) -> &str { "modal_demo" }

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
        draw_text(&mut plane, 2, 0, " Modal Dialogs ", t.primary, t.bg, true);
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

        // Settings panel (left ~45w)
        let settings_w = 40u16.min(area.width.saturating_sub(20));
        let has_modal = self.show_help || self.show_confirm;
        self.render_settings(&mut plane, 2, 2, settings_w, area.height.saturating_sub(4), has_modal);

        // Right panel: action buttons + description
        let rp_x = settings_w + 4;
        let rp_w = area.width.saturating_sub(rp_x + 2);

        draw_text(&mut plane, rp_x, 2, "Actions", t.primary, t.bg, true);
        for dx in 0..rp_w {
            let idx = ((3) * plane.width + rp_x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Action buttons
        let actions = [
            ("c", "Delete setting", "Opens confirm dialog", t.error),
            ("t", "Test toast", "Shows info toast", t.info),
            ("w", "Warning toast", "Shows warning toast", t.warning),
            ("e", "Error toast", "Shows error toast", t.error),
            ("s", "Success toast", "Shows success toast", t.success),
        ];
        for (i, (key, label, desc, color)) in actions.iter().enumerate() {
            let by = 4 + i as u16 * 2;
            if by + 1 >= area.height.saturating_sub(3) { break; }

            // Key badge
            let badge = format!(" {} ", key);
            for (j, ch) in badge.chars().enumerate() {
                let idx = (by * plane.width + rp_x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = *color;
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            draw_text(&mut plane, rp_x + 4, by, label, t.fg, t.bg, false);
            draw_text(&mut plane, rp_x + 4, by + 1, desc, t.fg_muted, t.bg, false);
        }

        // Description area (below actions)
        let desc_y = 4 + actions.len() as u16 * 2 + 1;
        if desc_y + 4 < area.height.saturating_sub(3) {
            draw_text(&mut plane, rp_x, desc_y, "Modal Stack", t.secondary, t.bg, true);
            for dx in 0..rp_w {
                let idx = ((desc_y + 1) * plane.width + rp_x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }

            let stack = [
                ("z=100", "Help overlay", t.fg_muted),
                ("z=110", "Confirm dialog", t.fg_muted),
                ("z=200", "Toast notifications", t.fg_muted),
            ];
            for (i, (z, desc, color)) in stack.iter().enumerate() {
                let sy = desc_y + 2 + i as u16;
                draw_text(&mut plane, rp_x, sy, z, t.primary, t.bg, true);
                draw_text(&mut plane, rp_x + 8, sy, desc, *color, t.bg, false);
            }

            // Current state
            let state_y = desc_y + 6;
            if state_y < area.height.saturating_sub(3) {
                draw_text(&mut plane, rp_x, state_y, "State", t.secondary, t.bg, true);
                let state = if self.show_confirm { "Confirm dialog open" }
                           else if self.show_help { "Help overlay open" }
                           else { "Base screen" };
                draw_text(&mut plane, rp_x, state_y + 1, state, t.fg, t.bg, false);
            }
        }

        // ── Modals (rendered on top) ─────────────────────────────────────

        if self.show_help {
            // Dim backdrop first
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 14u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;
            self.render_dimmed_backdrop(&mut plane, area, Some((hx, hy, hw, hh)));

            // Help modal content
            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * plane.width + x) as usize;
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

            let title = "Help Overlay (z=100)";
            let tx = hx + (hw - title.len() as u16) / 2;
            draw_text(&mut plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

            let shortcuts = [
                ("?", "Toggle this help"),
                ("c", "Show confirm dialog"),
                ("t/w/e/s", "Toast variants"),
                ("↑/↓", "Navigate settings"),
                ("Enter", "Toggle setting value"),
                ("B/Esc", "Back to showcase"),
                ("", "Click outside to dismiss"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                if !key.is_empty() {
                    draw_text(&mut plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
                }
                draw_text(&mut plane, hx + 16, row, desc, t.fg, t.surface_elevated, false);
            }
        }

        if self.show_confirm {
            let dw = 34u16;
            let dh = 8u16;
            let dx = (area.width - dw) / 2;
            let dy = (area.height - dh) / 2;
            self.render_dimmed_backdrop(&mut plane, area, Some((dx, dy, dw, dh)));

            // Confirm dialog content
            for y in dy..dy + dh {
                for x in dx..dx + dw {
                    let idx = (y * plane.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            for x in dx..dx + dw {
                let top = (dy * plane.width + x) as usize;
                let bot = ((dy + dh - 1) * plane.width + x) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.error; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.error; }
            }
            for y in dy..dy + dh {
                let left = (y * plane.width + dx) as usize;
                let right = (y * plane.width + dx + dw - 1) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.error; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.error; }
            }
            for (ch, cx, cy) in [('╭', dx, dy), ('╮', dx + dw - 1, dy), ('╰', dx, dy + dh - 1), ('╯', dx + dw - 1, dy + dh - 1)] {
                let idx = (cy * plane.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.error; }
            }

            let title = &self.confirm_title;
            let tx = dx + (dw - title.len() as u16) / 2;
            draw_text(&mut plane, tx, dy + 1, title, t.error, t.surface_elevated, true);

            draw_text(&mut plane, dx + 2, dy + 3, &self.confirm_message, t.fg, t.surface_elevated, false);

            // Yes button
            for (i, c) in " Yes ".chars().enumerate() {
                let idx = ((dy + 5) * plane.width + dx + 4 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].bg = t.error;
                    plane.cells[idx].fg = t.fg_on_accent;
                    plane.cells[idx].transparent = false;
                }
            }
            // No button
            for (i, c) in " No ".chars().enumerate() {
                let idx = ((dy + 5) * plane.width + dx + dw - 8 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].fg = t.fg;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Toasts (always on top)
        self.render_toasts(&mut plane, area);

        // Footer
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(" c:confirm | t/w/e/s:toasts | ↑↓:nav | ?:help | {}:back ", back_key);
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

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        // Confirm dialog takes priority
        if self.show_confirm {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') if key.modifiers.is_empty() => {
                    self.show_confirm = false;
                    let action = self.confirm_action;
                    self.add_toast(&format!("{} completed", action), "success");
                    true
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') if key.modifiers.is_empty() => {
                    self.show_confirm = false;
                    self.add_toast("Action cancelled", "info");
                    self.dirty = true;
                    true
                }
                _ => true,
            }
        } else if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) || key.code == KeyCode::Char('?') {
                self.show_help = false;
                self.dirty = true;
            }
            true
        } else {
            if self.keybindings.matches(actions::HELP, &key) || key.code == KeyCode::Char('?') {
                self.show_help = true;
                self.dirty = true;
                return true;
            }
            if self.keybindings.matches(actions::BACK, &key) {
                return false;
            }

            match key.code {
                KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers.is_empty() => {
                    if self.selected_setting < SETTINGS.len() {
                        self.request_confirm("Delete Setting",
                            &format!("Remove '{}' ({})?", SETTINGS[self.selected_setting].label, SETTINGS[self.selected_setting].value),
                            "Delete");
                    }
                    true
                }
                KeyCode::Char('t') if key.modifiers.is_empty() => {
                    self.add_toast("This is an info toast", "info");
                    true
                }
                KeyCode::Char('w') if key.modifiers.is_empty() => {
                    self.add_toast("Disk space running low", "warning");
                    true
                }
                KeyCode::Char('e') if key.modifiers.is_empty() => {
                    self.add_toast("Connection failed", "error");
                    true
                }
                KeyCode::Char('s') if key.modifiers.is_empty() => {
                    self.add_toast("File saved successfully", "success");
                    true
                }
                KeyCode::Up => {
                    if self.selected_setting > 0 {
                        self.selected_setting -= 1;
                        self.dirty = true;
                    }
                    true
                }
                KeyCode::Down => {
                    if self.selected_setting + 1 < SETTINGS.len() {
                        self.selected_setting += 1;
                        self.dirty = true;
                    }
                    true
                }
                KeyCode::Enter => {
                    self.add_toast(&format!("{} toggled", SETTINGS[self.selected_setting].label), "info");
                    true
                }
                _ => false,
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            if self.show_confirm {
                let dw = 34u16;
                let dh = 8u16;
                let dx = (area.width - dw) / 2;
                let dy = (area.height - dh) / 2;
                if col < dx || col >= dx + dw || row < dy || row >= dy + dh {
                    self.show_confirm = false;
                    self.add_toast("Action cancelled", "info");
                    self.dirty = true;
                    return true;
                }
                if row == dy + 5 && col >= dx + 4 && col < dx + 9 {
                    self.show_confirm = false;
                    self.add_toast(&format!("{} completed", self.confirm_action), "success");
                    self.dirty = true;
                    return true;
                }
                if row == dy + 5 && col >= dx + dw - 8 && col < dx + dw - 3 {
                    self.show_confirm = false;
                    self.add_toast("Action cancelled", "info");
                    self.dirty = true;
                    return true;
                }
                return true;
            }
            if self.show_help {
                let hw = 44u16.min(area.width.saturating_sub(4));
                let hh = 14u16.min(area.height.saturating_sub(4));
                let hx = (area.width - hw) / 2;
                let hy = (area.height - hh) / 2;
                if col < hx || col >= hx + hw || row < hy || row >= hy + hh {
                    self.show_help = false;
                    self.dirty = true;
                    return true;
                }
                return true;
            }
        }
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
