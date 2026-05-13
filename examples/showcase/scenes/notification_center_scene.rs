//! Embedded NotificationCenter scene for the showcase.
//!
//! Demonstrates the NotificationCenter widget with toast notifications.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{NotificationCenter, NotificationKind};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

pub struct NotificationCenterScene {
    notifications: NotificationCenter,
    theme: Theme,
    show_help: bool,
    tick_count: usize,
    keybindings: KeybindingSet,
}

impl NotificationCenterScene {
    pub fn new(theme: Theme) -> Self {
        let mut nc = NotificationCenter::new(theme);
        // Add some initial notifications
        nc.info("Welcome", "NotificationCenter demo started");
        nc.success("Success", "Files saved successfully");
        Self {
            notifications: nc,
            theme,
            show_help: false,
            tick_count: 0,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn add_random_notification(&mut self) {
        let kinds = [
            (NotificationKind::Info, "Info", "New message received"),
            (NotificationKind::Success, "Done", "Task completed"),
            (NotificationKind::Warning, "Warning", "Disk space low"),
            (NotificationKind::Error, "Error", "Connection failed"),
        ];
        let (kind, title, msg) = kinds[self.tick_count % kinds.len()];
        self.notifications.notify(title, msg, kind);
    }
}

impl Scene for NotificationCenterScene {
    fn scene_id(&self) -> &str { "notification_center" }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        let title = " NotificationCenter ";
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

        // Instructions
        let instructions = "Press SPACE to add notifications";
        draw_text(&mut plane, 2, 2, instructions, t.fg, t.bg, false);

        let instructions2 = "Click notifications to dismiss";
        draw_text(&mut plane, 2, 3, instructions2, t.fg_muted, t.bg, false);

        // Notification area (top-right)
        let notif_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(4));
        let notif_plane = self.notifications.render(notif_area);
        blit_to(&mut plane, &notif_plane, notif_area.x as usize, notif_area.y as usize);

        // Legend
        let legend_y = area.height.saturating_sub(5);
        draw_text(&mut plane, 2, legend_y, "Notification types:", t.fg_muted, t.bg, false);
        draw_text(&mut plane, 2, legend_y + 1, "  i = Info (blue)", t.info, t.bg, false);
        draw_text(&mut plane, 2, legend_y + 2, "  ✔ = Success (green)", t.success, t.bg, false);
        draw_text(&mut plane, 2, legend_y + 3, "  ! = Warning (yellow)", t.warning, t.bg, false);
        draw_text(&mut plane, 22, legend_y + 1, "  ✖ = Error (red)", t.error, t.bg, false);

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let nav = " SPACE: add | Click: dismiss | B/Esc: back | ?: help ";
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
            KeyCode::Char(' ') => {
                self.add_random_notification();
                self.tick_count += 1;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.notifications.handle_mouse(kind, col, row)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.notifications.on_theme_change(theme);
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
            dest.cells[idx] = *cell;
        }
    }
}

fn draw_help_overlay(plane: &mut Plane, area: Rect, t: Theme) {
    let hw = 44u16.min(area.width.saturating_sub(4));
    let hh = 10u16.min(area.height.saturating_sub(4));
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

    let title = "NotificationCenter Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

    let shortcuts = [
        ("SPACE", "Add notification"),
        ("Click", "Dismiss notification"),
        ("B/Esc", "Back to showcase"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 14, row, desc, t.fg, t.surface_elevated, false);
    }
}
