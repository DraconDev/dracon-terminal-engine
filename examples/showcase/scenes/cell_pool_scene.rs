//! Embedded CellPool scene for the showcase.
//!
//! Demonstrates the Cell allocation recycling pool with visual gauges,
//! allocation waves, and real-time stats.

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::compositor::pool::CellPool;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

pub struct CellPoolScene {
    pool: CellPool,
    theme: Theme,
    show_help: bool,
    acquired: usize,
    released: usize,
    total_cells: usize,
    tick_count: usize,
    auto_running: bool,
    wave_history: Vec<(usize, usize)>, // (acquired, released) per tick
    keybindings: KeybindingSet,
    dirty: bool,
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
            auto_running: false,
            wave_history: Vec::new(),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            dirty: true,
        }
    }

    fn simulate_allocation(&mut self) {
        use dracon_terminal_engine::compositor::pool::{acquire_plane_cells, release_plane_cells};

        let sizes = [(40, 10), (80, 24), (60, 15), (100, 30), (50, 20), (70, 12)];
        let (w, h) = sizes[self.tick_count % sizes.len()];

        let cells = acquire_plane_cells(&mut self.pool, w, h);
        let acq = cells.len();
        self.acquired += acq;

        let rel = if self.tick_count >= 3 {
            let prev_idx = (self.tick_count - 3) % sizes.len();
            let (pw, ph) = sizes[prev_idx];
            let n: usize = (pw as usize) * (ph as usize);
            self.released += n;
            release_plane_cells(&mut self.pool, pw, ph, cells);
            n
        } else {
            0
        };

        self.total_cells = self.pool.total_cells();
        self.wave_history.push((acq, rel as usize));
        if self.wave_history.len() > 50 {
            self.wave_history.remove(0);
        }
        self.tick_count += 1;
    }

    #[allow(clippy::too_many_arguments)]
    fn render_gauge(&self, plane: &mut Plane, x: u16, y: u16, w: u16, pct: f64, label: &str, color: Color, bg: Color) {
        draw_text(plane, x, y, label, self.theme.fg_muted, bg, false);
        let bar_x = x + label.len() as u16 + 1;
        let bar_w = w.saturating_sub(label.len() as u16 + 8);
        let filled = ((pct / 100.0) * bar_w as f64).min(bar_w as f64) as usize;

        for i in 0..bar_w as usize {
            let idx = (y * plane.width + bar_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if i < filled { '█' } else { '░' };
                plane.cells[idx].fg = if i < filled { color } else { self.theme.fg_muted };
                plane.cells[idx].bg = bg;
                plane.cells[idx].transparent = false;
            }
        }
        let pct_text = format!("{:5.1}%", pct);
        draw_text(plane, bar_x + bar_w + 1, y, &pct_text, color, bg, true);
    }

    fn render_wave_chart(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16) {
        let t = &self.theme;
        if self.wave_history.is_empty() || w < 4 || h < 3 { return; }

        // Border
        for i in 0..w {
            let top = (y * plane.width + x + i) as usize;
            let bot = ((y + h - 1) * plane.width + x + i) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for j in 0..h {
            let left = ((y + j) * plane.width + x) as usize;
            let right = ((y + j) * plane.width + x + w - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }
        // Corners
        for (ch, cx, cy) in [('╭', x, y), ('╮', x + w - 1, y), ('╰', x, y + h - 1), ('╯', x + w - 1, y + h - 1)] {
            let idx = (cy * plane.width + cx) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
        }

        // Find max for scaling
        let max_val = self.wave_history.iter().map(|(a, r)| (*a).max(*r)).max().unwrap_or(1).max(1);

        // Draw bars for each tick (acquired = green, released = blue)
        let chart_w = (w as usize).saturating_sub(2);
        let chart_h = (h as usize).saturating_sub(2);
        let data: Vec<(usize, usize)> = self.wave_history.iter().rev().take(chart_w).cloned().collect();

        for (i, (acq, rel)) in data.iter().enumerate() {
            let bar_x = x + 1 + (chart_w.saturating_sub(data.len()) + i) as u16;
            let acq_h = (*acq as f64 / max_val as f64 * chart_h as f64).min(chart_h as f64) as usize;
            let rel_h = (*rel as f64 / max_val as f64 * chart_h as f64).min(chart_h as f64) as usize;

            // Acquired bars (green, from bottom)
            for j in 0..acq_h {
                let by = y + h - 2 - j as u16;
                let idx = (by * plane.width + bar_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '▓';
                    plane.cells[idx].fg = t.success;
                    plane.cells[idx].transparent = false;
                }
            }
            // Released bars (blue, stacked on top if same column — show as dim)
            for j in 0..rel_h {
                let by = y + h - 2 - j as u16;
                let idx = (by * plane.width + bar_x) as usize;
                if idx < plane.cells.len() && plane.cells[idx].char != '▓' {
                    plane.cells[idx].char = '▒';
                    plane.cells[idx].fg = t.info;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Labels
        draw_text(plane, x + 2, y + 1, "Allocation Waves", t.fg_muted, t.bg, false);
    }

    fn render_pool_grid(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16) {
        let t = &self.theme;

        draw_text(plane, x, y, "Pool Grid", t.secondary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Visual grid showing pool state: each cell is a tiny block
        // Active = green, Pooled = blue, Free = dim
        let grid_x = x + 1;
        let grid_y = y + 2;
        let grid_w = w.saturating_sub(2);
        let grid_h = h.saturating_sub(3);

        let total_slots = grid_w as usize * grid_h as usize;
        let active = self.acquired.saturating_sub(self.released);
        let pooled = self.total_cells;

        // Compute proportions
        let active_slots = if self.acquired > 0 {
            (active as f64 / self.acquired.max(1) as f64 * total_slots as f64).min(total_slots as f64) as usize
        } else { 0 };
        let pooled_slots = if self.acquired > 0 {
            (pooled as f64 / self.acquired.max(1) as f64 * total_slots as f64).min((total_slots - active_slots) as f64) as usize
        } else { 0 };

        let mut drawn = 0;
        for gy in 0..grid_h {
            for gx in 0..grid_w {
                let idx = ((grid_y + gy) * plane.width + grid_x + gx) as usize;
                if idx < plane.cells.len() {
                    let (ch, fg) = if drawn < active_slots {
                        ('█', t.success)
                    } else if drawn < active_slots + pooled_slots {
                        ('▓', t.info)
                    } else if drawn < total_slots / 3 {
                        ('░', t.fg_muted)
                    } else {
                        (' ', t.fg_muted)
                    };
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                }
                drawn += 1;
            }
        }

        // Legend
        let legend_y = grid_y + grid_h + 1;
        if legend_y < y + h {
            let legends = [
                ('█', t.success, "Active"),
                ('▓', t.info, "Pooled"),
                ('░', t.fg_muted, "Free"),
            ];
            let mut lx = x;
            for (ch, color, label) in legends {
                let idx = (legend_y * plane.width + lx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = color;
                }
                draw_text(plane, lx + 1, legend_y, label, t.fg_muted, t.bg, false);
                lx += label.len() as u16 + 4;
            }
        }
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
        draw_text(&mut plane, 2, 0, " CellPool ", t.primary, t.bg, true);
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
        draw_text(&mut plane, 2, 2, "Object pool for Cell recycling across frames", t.fg_muted, t.bg, false);

        // ── Visual Gauges ──────────────────────────────────────────────────
        let active = self.acquired.saturating_sub(self.released);
        let reuse_rate = if self.acquired > 0 { (self.released as f64 / self.acquired as f64 * 100.0).min(100.0) } else { 0.0 };
        let active_pct = if self.acquired > 0 { (active as f64 / self.acquired as f64 * 100.0).min(100.0) } else { 0.0 };

        let gauge_w = area.width.saturating_sub(4);
        self.render_gauge(&mut plane, 2, 4, gauge_w, reuse_rate, "Reuse:", t.success, t.bg);
        self.render_gauge(&mut plane, 2, 5, gauge_w, active_pct, "Active:", t.info, t.bg);

        // Pool utilization (how much pooled memory is sitting idle = good)
        let pool_pct = if self.total_cells > 0 && self.acquired > 0 {
            (self.total_cells as f64 / self.acquired.max(1) as f64 * 100.0).min(100.0)
        } else { 0.0 };
        self.render_gauge(&mut plane, 2, 6, gauge_w, pool_pct, "Pooled:", t.secondary, t.bg);

        // ── Numeric Stats ─────────────────────────────────────────────────
        let stats_y = 8;
        draw_text(&mut plane, 2, stats_y, "Stats:", t.primary, t.bg, true);
        let stats = [
            format!("Acquired: {}", self.acquired),
            format!("Released: {}", self.released),
            format!("Active: {}", active),
            format!("Pooled: {}", self.total_cells),
            format!("Ticks: {}", self.tick_count),
        ];
        for (i, s) in stats.iter().enumerate() {
            let col = 2 + (i / 3) * 25;
            let row = stats_y + 1 + (i % 3) as u16;
            draw_text(&mut plane, col as u16, row, s, t.fg, t.bg, false);
        }

        // ── Allocation Wave Chart (left) ────────────────────────────────────
        let chart_w = (area.width * 60 / 100).saturating_sub(4);
        let chart_y = stats_y + 5;
        let chart_h = area.height.saturating_sub(chart_y + 3);
        if chart_h > 6 {
            self.render_wave_chart(&mut plane, 2, chart_y, chart_w, chart_h);
        }

        // ── Pool Grid (right) ────────────────────────────────────────────────
        let grid_x = chart_w + 4;
        let grid_w = area.width.saturating_sub(grid_x + 2);
        if chart_h > 8 && grid_w > 10 {
            self.render_pool_grid(&mut plane, grid_x, chart_y, grid_w, chart_h);
        }

        // Vertical divider
        for y in chart_y..area.height.saturating_sub(2) {
            let idx = (y * plane.width + chart_w + 2) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Legend for chart
        if chart_h > 6 {
            let legend_y = chart_y + chart_h;
            draw_text(&mut plane, 2, legend_y, "▓ Acquired  ▒ Released", t.fg_muted, t.bg, false);
        }

        // ── Auto indicator ────────────────────────────────────────────────
        if self.auto_running {
            draw_text(&mut plane, area.width.saturating_sub(12), 2, "▶ AUTO", t.primary, t.bg, true);
        }

        // ── Footer ────────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("?");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " SPACE:alloc | a:auto | r:reset | {}:help | {}:back ",
            help_key, back_key,
        );
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
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.simulate_allocation();
                self.dirty = true;
                true
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.auto_running = !self.auto_running;
                self.dirty = true;
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.pool = CellPool::new();
                self.acquired = 0;
                self.released = 0;
                self.total_cells = 0;
                self.tick_count = 0;
                self.auto_running = false;
                self.wave_history.clear();
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        match kind {
            MouseEventKind::Down(_) => {
                // Any click: simulate allocation (same as Space key)
                self.simulate_allocation();
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
    }

    fn needs_render(&self) -> bool { true } // always re-render for auto mode
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl CellPoolScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 40u16.min(area.width.saturating_sub(4));
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

        let help_key = self.keybindings.display(actions::HELP).unwrap_or("?");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let title = "CellPool Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

        let shortcuts = [
            ("SPACE", "Simulate allocation"),
            ("a", "Toggle auto-simulation"),
            ("r", "Reset pool & stats"),
            (back_key, "Back to showcase"),
            (help_key, "Toggle this help"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
            draw_text(plane, hx + 12, row, desc, t.fg, t.surface_elevated, false);
        }
    }
}
