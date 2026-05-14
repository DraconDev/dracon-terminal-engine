//! Embedded Modal Demo scene for the showcase.
//!
//! Demonstrates ConfirmDialog, Modal, and Toast with z-index layering.
//! Press `B`/`Esc` to go back.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{ConfirmDialog, Modal};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

#[allow(dead_code)]
pub struct ModalDemoScene {
    theme: Theme,
    show_help: bool,
    show_confirm: bool,
    show_toast: bool,
    dirty: bool,
    toast_message: String,
    help_modal: Modal<'static>,
    confirm_dialog: ConfirmDialog,
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
}

impl ModalDemoScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            show_help: false,
            show_confirm: false,
            show_toast: false,
            dirty: true,
            toast_message: String::new(),
            help_modal: Modal::new("Keyboard Shortcuts").with_size(42, 12),
            confirm_dialog: ConfirmDialog::new("Confirm", "Delete item?"),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }
}

impl Scene for ModalDemoScene {
    fn scene_id(&self) -> &str { "modal_demo" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = self.theme.clone();
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title
        draw_text(&mut plane, 2, 0, " Modal Demo ", t.primary, t.bg, true);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Main content area
        let content_y = 2u16;
        let content_h = area.height.saturating_sub(8);

        // Background card
        for y in content_y..content_y + content_h {
            for x in 1..area.width.saturating_sub(1) {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                }
            }
        }

        // Card border
        let card_x = 1u16;
        let card_y = content_y;
        let card_w = area.width.saturating_sub(2);
        let card_h = content_h;

        for x in card_x..card_x + card_w {
            let top = (card_y * area.width + x) as usize;
            let bot = ((card_y + card_h - 1) * area.width + x) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for y in card_y..card_y + card_h {
            let left = (y * area.width + card_x) as usize;
            let right = (y * area.width + card_x + card_w - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }

        // Corners
        let corners = [
            (card_x, card_y, '╭'),
            (card_x + card_w - 1, card_y, '╮'),
            (card_x, card_y + card_h - 1, '╰'),
            (card_x + card_w - 1, card_y + card_h - 1, '╯'),
        ];
        for (x, y, c) in corners {
            let idx = (y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.outline;
            }
        }

        // Content text
        let lines = [
            "This demo shows modal dialogs:",
            "",
            "  • Help overlay (z=100)",
            "  • Confirm dialog (z=110)",
            "  • Toast notifications",
            "",
            "Press '?' to toggle help overlay.",
            "Press 'c' to show confirm dialog.",
        ];
        for (i, line) in lines.iter().enumerate() {
            let y = content_y + 2 + i as u16;
            if y < content_y + card_h.saturating_sub(2) {
                draw_text(&mut plane, card_x + 2, y, line, t.fg, t.surface, false);
            }
        }

        // Toast
        if self.show_toast {
            let toast_text = &self.toast_message;
            let toast_w = toast_text.len() as u16 + 4;
            let toast_x = (area.width.saturating_sub(toast_w)) / 2;
            let toast_y = area.height.saturating_sub(4);

            for x in toast_x..toast_x + toast_w {
                let idx = (toast_y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.success_bg;
                    plane.cells[idx].fg = t.success;
                    plane.cells[idx].char = if x == toast_x || x == toast_x + toast_w - 1 { '│' } else { ' ' };
                }
            }
            draw_text(&mut plane, toast_x + 2, toast_y, toast_text, t.success, t.success_bg, true);
        }

        // Help overlay (rendered above content at z=100)
        if self.show_help {
            let hw = 42u16.min(area.width.saturating_sub(4));
            let hh = 12u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;

            // Backdrop
            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Border
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
            // Rounded corners
            let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
            for (ch, cx, cy) in corners {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
            }

            let title = "Help Overlay";
            let tx = hx + (hw - title.len() as u16) / 2;
            draw_text(&mut plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

            let shortcuts = [
                ("?", "Toggle help"),
                ("c", "Show confirm"),
                ("B/Esc", "Back to showcase"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                draw_text(&mut plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
                draw_text(&mut plane, hx + 16, row, desc, t.fg, t.surface_elevated, false);
            }
        }

        // Confirm dialog (rendered above help at z=110)
        if self.show_confirm {
            let dw = 30u16;
            let dh = 8u16;
            let dx = (area.width - dw) / 2;
            let dy = (area.height - dh) / 2;

            // Backdrop
            for y in dy..dy + dh {
                for x in dx..dx + dw {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Border
            for x in dx..dx + dw {
                let top = (dy * area.width + x) as usize;
                let bot = ((dy + dh - 1) * area.width + x) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
            }
            for y in dy..dy + dh {
                let left = (y * area.width + dx) as usize;
                let right = (y * area.width + dx + dw - 1) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
            }

            let title = "Confirm";
            let tx = dx + (dw - title.len() as u16) / 2;
            draw_text(&mut plane, tx, dy + 1, title, t.primary, t.surface_elevated, true);

            let msg = "Delete item?";
            draw_text(&mut plane, dx + 2, dy + 3, msg, t.fg, t.surface_elevated, false);

            // Yes button
            let yes_x = dx + 4;
            let _yes_w = 6u16;
            for (i, c) in "  Yes  ".chars().enumerate() {
                let idx = ((dy + 5) * area.width + yes_x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].bg = t.primary;
                    plane.cells[idx].fg = t.fg_on_accent;
                }
            }

            // No button
            let no_x = dx + dw.saturating_sub(10);
            for (i, c) in "  No   ".chars().enumerate() {
                let idx = ((dy + 5) * area.width + no_x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].fg = t.fg;
                }
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
        let nav = " ?: help | c: confirm | B/Esc: back | q: quit ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.bg, false);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        // Handle confirm dialog buttons
        if self.show_confirm {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.show_confirm = false;
                    self.show_toast = true;
                    self.toast_message = "Confirmed!".to_string();
                    self.dirty = true;
                    true
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.show_confirm = false;
                    self.dirty = true;
                    true
                }
                _ => true,
            }
        } else if self.show_toast {
            self.show_toast = false;
            self.dirty = true;
            true
        } else if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            true
        } else {
            if self.keybindings.matches(actions::HELP, &key) {
                self.show_help = true;
                self.dirty = true;
                return true;
            }
            if self.keybindings.matches(actions::BACK, &key) {
                return false;
            }
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers.is_empty() => { self.show_confirm = true; self.dirty = true; true }
                _ => false,
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            if self.show_confirm {
                let dw = 30u16;
                let dh = 8u16;
                let dx = (area.width - dw) / 2;
                let dy = (area.height - dh) / 2;
                // Click outside dialog -> dismiss
                if col < dx || col >= dx + dw || row < dy || row >= dy + dh {
                    self.show_confirm = false;
                    self.dirty = true;
                    return true;
                }
                // Yes button
                let yes_x = dx + 4;
                if row == dy + 5 && col >= yes_x && col < yes_x + 7 {
                    self.show_confirm = false;
                    self.show_toast = true;
                    self.toast_message = "Confirmed!".to_string();
                    self.dirty = true;
                    return true;
                }
                // No button
                let no_x = dx + dw.saturating_sub(10);
                if row == dy + 5 && col >= no_x && col < no_x + 7 {
                    self.show_confirm = false;
                    self.dirty = true;
                    return true;
                }
                return true;
            }
            if self.show_help {
                let hw = 42u16.min(area.width.saturating_sub(4));
                let hh = 12u16.min(area.height.saturating_sub(4));
                let hx = (area.width - hw) / 2;
                let hy = (area.height - hh) / 2;
                if col < hx || col >= hx + hw || row < hy || row >= hy + hh {
                    self.show_help = false;
                    self.dirty = true;
                    return true;
                }
                return true;
            }
            // Main content triggers
            let content_y = 2u16;
            let card_x = 1u16;
            let card_w = area.width.saturating_sub(2);
            let help_line_y = content_y + 8; // "Press '?' to toggle help overlay."
            let confirm_line_y = content_y + 9; // "Press 'c' to show confirm dialog."
            if row == help_line_y && col >= card_x + 2 && col < card_x + card_w - 2 {
                self.show_help = true;
                self.dirty = true;
                return true;
            }
            if row == confirm_line_y && col >= card_x + 2 && col < card_x + card_w - 2 {
                self.show_confirm = true;
                self.dirty = true;
                return true;
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.help_modal.on_theme_change(theme);
        self.confirm_dialog.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}



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