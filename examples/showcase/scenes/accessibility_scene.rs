//! Embedded Accessibility scene for the showcase.
//!
//! Demonstrates screen reader support via OSC 99 announcements.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{Button, Checkbox};
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

pub struct AccessibilityScene {
    theme: Theme,
    show_help: bool,
    enabled: bool,
    announcements: Vec<String>,
    checkbox: Checkbox,
    submit_btn: Button,
    keybindings: KeybindingSet,
}

impl AccessibilityScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            show_help: false,
            enabled: true,
            announcements: Vec::new(),
            checkbox: Checkbox::new(WidgetId::new(200), "Enable notifications"),
            submit_btn: Button::with_id(WidgetId::new(201), "Submit"),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn announce(&mut self, text: &str) {
        if self.enabled {
            self.announcements.push(text.to_string());
            if self.announcements.len() > 5 {
                self.announcements.remove(0);
            }
        }
    }
}

impl Scene for AccessibilityScene {
    fn scene_id(&self) -> &str { "accessibility" }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        let title = " Accessibility (OSC 99) ";
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);

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

        // Status indicator
        let status = if self.enabled { "● Enabled" } else { "○ Disabled" };
        let status_color = if self.enabled { t.success } else { t.error };
        draw_text(&mut plane, 2, 2, status, status_color, t.bg, true);

        // Toggle hint
        draw_text(&mut plane, 15, 2, "(press t to toggle)", t.fg_muted, t.bg, false);

        // OSC 99 info panel
        let info_y = 4;
        draw_text(&mut plane, 2, info_y, "OSC 99 Announcements:", t.primary, t.bg, true);

        let info_items = [
            ("Role:", "push button"),
            ("Label:", "Submit"),
            ("Shortcut:", "Ctrl+Enter"),
            ("Level:", "assertive"),
        ];
        for (i, (label, value)) in info_items.iter().enumerate() {
            let y = info_y + 1 + i as u16;
            draw_text(&mut plane, 2, y, label, t.fg_muted, t.bg, false);
            draw_text(&mut plane, 12, y, value, t.fg, t.bg, false);
        }

        // Sample widgets
        let widget_y = info_y + 6;
        draw_text(&mut plane, 2, widget_y, "Sample widgets:", t.fg_muted, t.bg, false);

        // Checkbox
        let cb_area = Rect::new(2, widget_y + 1, 20, 1);
        let cb_plane = self.checkbox.render(cb_area);
        blit_to(&mut plane, &cb_plane, cb_area.x as usize, cb_area.y as usize);

        // Button
        let btn_area = Rect::new(2, widget_y + 3, 12, 1);
        let btn_plane = self.submit_btn.render(btn_area);
        blit_to(&mut plane, &btn_plane, btn_area.x as usize, btn_area.y as usize);

        // Recent announcements
        let ann_y = widget_y + 6;
        draw_text(&mut plane, 2, ann_y, "Recent announcements:", t.primary, t.bg, true);
        for (i, ann) in self.announcements.iter().enumerate() {
            let y = ann_y + 1 + i as u16;
            if y < area.height.saturating_sub(3) {
                let truncated = if ann.len() > 40 {
                    format!("{}...", &ann[..37])
                } else {
                    ann.clone()
                };
                draw_text(&mut plane, 2, y, &format!("  • {}", truncated), t.fg, t.bg, false);
            }
        }

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let nav = " t: toggle | Enter: activate | B/Esc: back | ?: help ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.bg, false);

        if self.show_help {
            draw_help_overlay(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
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

        match key.code {
            KeyCode::Char('t') => {
                self.enabled = !self.enabled;
                if self.enabled {
                    self.announce("Accessibility announcements enabled".to_string());
                }
                true
            }
            _ => {
                if self.checkbox.handle_key(key.clone()) {
                    self.announce(format!("Checkbox: {}", if self.checkbox.is_checked() { "checked" } else { "unchecked" }));
                    true
                } else {
                    self.submit_btn.handle_key(key)
                }
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let cb_area = Rect::new(2, 14, 20, 1);
        let btn_area = Rect::new(2, 16, 12, 1);

        if col >= cb_area.x && col < cb_area.x + cb_area.width && row == cb_area.y {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                self.checkbox.handle_key(KeyEvent::new(KeyEventKind::Press, KeyCode::Enter));
                return true;
            }
        }
        if col >= btn_area.x && col < btn_area.x + btn_area.width && row == btn_area.y {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                self.announce("Form submitted successfully".to_string());
                return true;
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.checkbox.on_theme_change(theme);
        self.submit_btn.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch, fg, bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false, skip: false,
            };
        }
    }
}

fn blit_to(dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent { continue; }
        let row = i / src.width as usize;
        let col = i % src.width as usize;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize { continue; }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = cell.clone();
        }
    }
}

fn draw_help_overlay(plane: &mut Plane, area: Rect, t: Theme) {
    let hw = 50u16.min(area.width.saturating_sub(4));
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

    let shortcuts = [
        ("t", "Toggle announcements"),
        ("Enter", "Activate widget"),
        ("Click", "Interact with widget"),
        ("B/Esc", "Back to showcase"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 14, row, desc, t.fg, t.surface_elevated, false);
    }
}
