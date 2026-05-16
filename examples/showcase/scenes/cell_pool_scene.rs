//! Embedded CellPool scene for the showcase.
//!
//! Demonstrates the Cell allocation recycling pool for performance optimization.

use crate::scenes::shared_helpers::{draw_text, blit_to};
use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::compositor::pool::CellPool;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;

pub struct CellPoolScene {
    pool: CellPool,
    theme: Theme,
    show_help: bool,
    acquired: usize,
    released: usize,
    total_cells: usize,
    tick_count: usize,
    keybindings: KeybindingSet,
}

impl CellPoolScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            pool: CellPool::new(),
            theme,
            show_help: false,
            acquired: 0,
            released: 0,
            total_cells: 0,
            tick_count: 0,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn simulate_allocation(&mut self) {
        use dracon_terminal_engine::compositor::pool::acquire_plane_cells;
        use dracon_terminal_engine::compositor::pool::release_plane_cells;

        // Simulate allocating cells for various plane sizes
        let sizes = [(40, 10), (80, 24), (60, 15), (100, 30)];
        let (w, h) = sizes[self.tick_count % sizes.len()];

        // Acquire cells
        let cells = acquire_plane_cells(&mut self.pool, w, h);
        self.acquired += cells.len();

        // Release cells after a few ticks
        if self.tick_count >= 3 {
            let prev_idx = (self.tick_count - 3) % sizes.len();
            let (pw, ph) = sizes[prev_idx];
            self.released += cells.len();
            release_plane_cells(&mut self.pool, pw, ph, cells);
        }

        self.total_cells = self.pool.total_cells();
        self.tick_count += 1;
    }
}

impl Scene for CellPoolScene {
    fn scene_id(&self) -> &str { "cell_pool" }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        let title = " CellPool ";
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

        // Description
        let desc = "Object pool for Cell allocation. Reduces allocation pressure";
        let desc2 = "by recycling cell vectors across frames.";
        draw_text(&mut plane, 2, 2, desc, t.fg_muted, t.bg, false);
        draw_text(&mut plane, 2, 3, desc2, t.fg_muted, t.bg, false);

        // Stats
        let stats_y = 5;
        draw_text(&mut plane, 2, stats_y, "Pool Statistics:", t.primary, t.bg, true);

        let stats = [
            ("Total acquired:", format!("{}", self.acquired)),
            ("Total released:", format!("{}", self.released)),
            ("Active cells:", format!("{}", self.acquired.saturating_sub(self.released))),
            ("Pooled cells:", format!("{}", self.total_cells)),
        ];

        for (i, (label, value)) in stats.iter().enumerate() {
            let y = stats_y + 1 + i as u16;
            draw_text(&mut plane, 2, y, label, t.fg_muted, t.bg, false);
            draw_text(&mut plane, 20, y, value, t.fg, t.bg, true);
        }

        // Reuse rate
        let reuse_rate = if self.acquired > 0 {
            (self.released as f64 / self.acquired as f64 * 100.0).min(100.0)
        } else {
            0.0
        };
        let rate_str = format!("Reuse rate: {:.1}%", reuse_rate);
        draw_text(&mut plane, 2, stats_y + 6, &rate_str, t.success, t.bg, true);

        // Visual bar
        let bar_y = stats_y + 7;
        let bar_width = 40;
        let filled = ((reuse_rate / 100.0) * bar_width as f64) as usize;
        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(bar_width - filled));
        draw_text(&mut plane, 2, bar_y, &bar, t.info, t.bg, false);

        // Controls hint
        let controls_y = stats_y + 10;
        draw_text(&mut plane, 2, controls_y, "Controls:", t.primary, t.bg, true);
        draw_text(&mut plane, 2, controls_y + 1, "  SPACE: simulate allocation", t.fg_muted, t.bg, false);
        draw_text(&mut plane, 2, controls_y + 2, "  r: reset pool", t.fg_muted, t.bg, false);

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let nav = " SPACE: alloc | r: reset | B/Esc: back | ?: help ";
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
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.simulate_allocation();
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.pool = CellPool::new();
                self.acquired = 0;
                self.released = 0;
                self.total_cells = 0;
                self.tick_count = 0;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: dracon_terminal_engine::input::event::MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn draw_help_overlay(plane: &mut Plane, area: Rect, t: &Theme) {
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

    let title = "CellPool Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

    let shortcuts = [
        ("SPACE", "Simulate allocation"),
        ("r", "Reset pool"),
        ("B/Esc", "Back to showcase"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 14, row, desc, t.fg, t.surface_elevated, false);
    }
}
