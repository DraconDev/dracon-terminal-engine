//! Embedded Autocomplete scene for the showcase.
//!
//! Demonstrates the Autocomplete widget with search suggestions.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::Autocomplete;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

pub struct AutocompleteScene {
    autocomplete: Autocomplete,
    theme: Theme,
    show_help: bool,
    selected_item: Option<String>,
    keybindings: KeybindingSet,
}

const SUGGESTIONS: [&str; 12] = [
    "rustacean",
    "rust-analyzer",
    "rustdoc",
    "rustfmt",
    "rustc",
    "cargo",
    "clippy",
    "miri",
    "rls",
    "rustlings",
    "rustup",
    "crates.io",
];

impl AutocompleteScene {
    pub fn new(theme: Theme) -> Self {
        let suggestions: Vec<String> = SUGGESTIONS.iter().map(|s| s.to_string()).collect();
        Self {
            autocomplete: Autocomplete::new(WidgetId::new(100), suggestions)
                .with_theme(theme)
                .with_max_visible(6)
                .on_select(|s| { /* selection callback */ }),
            theme,
            show_help: false,
            selected_item: None,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }
}

impl Scene for AutocompleteScene {
    fn scene_id(&self) -> &str { "autocomplete" }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        let title = " Autocomplete ";
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

        // Label
        let label = "Search packages:";
        draw_text(&mut plane, 2, 2, label, t.fg_muted, t.bg, false);

        // Autocomplete widget area
        let input_area = Rect::new(area.x + 2, area.y + 3, 30, 1);
        let dropdown_area = Rect::new(area.x + 2, area.y + 4, 30, 10);
        self.autocomplete.set_area(input_area);
        let ac_plane = self.autocomplete.render(dropdown_area.union(input_area));
        blit_to(&mut plane, &ac_plane, input_area.x as usize, input_area.y as usize);

        // Selected item display
        if let Some(ref item) = self.selected_item {
            let status = format!(" Selected: {} ", item);
            draw_text(&mut plane, 2, 8, &status, t.success, t.bg, true);
        }

        // Info panel
        let info = "Type to filter suggestions";
        draw_text(&mut plane, 2, 10, info, t.fg_muted, t.bg, false);

        let shortcuts = [
            "↑↓: navigate",
            "Enter: select",
            "Tab: complete",
            "Esc: close",
        ];
        for (i, shortcut) in shortcuts.iter().enumerate() {
            draw_text(&mut plane, 2, 12 + i as u16, shortcut, t.fg_muted, t.bg, false);
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
        let nav = " Type to search | ↑↓ nav | Enter select | B/Esc: back | ?: help ";
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

        if self.autocomplete.handle_key(key) {
            if let Some(selected) = self.autocomplete.selected() {
                self.selected_item = Some(selected.to_string());
            }
            return true;
        }
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let input_area = Rect::new(2, 3, 30, 1);
        let rel_col = col.saturating_sub(input_area.x);
        let rel_row = row.saturating_sub(input_area.y);
        self.autocomplete.handle_mouse(kind, rel_col, rel_row)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.autocomplete.on_theme_change(theme);
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

    let title = "Autocomplete Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

    let shortcuts = [
        ("↑↓", "Navigate suggestions"),
        ("Enter", "Select item"),
        ("Tab", "Auto-complete"),
        ("B/Esc", "Back to showcase"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 14, row, desc, t.fg, t.surface_elevated, false);
    }
}
