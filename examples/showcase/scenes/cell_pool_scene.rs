//! Embedded CellPool scene for the showcase.
//!
//! Demonstrates the Cell allocation recycling pool with a visual
//! memory map, allocation wave chart, and real-time stats.

use crate::scenes::shared_helpers::{draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::compositor::pool::CellPool;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widgets::{StatusBar, StatusSegment};
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::Cell;
use std::cell::RefCell;

const SIDEBAR_W: u16 = 22;
const DIV_X: u16 = SIDEBAR_W + 2;

pub struct CellPoolScene {
    pool: CellPool,
    theme: Theme,
    show_help: bool,
    acquired: usize,
    released: usize,
    total_cells: usize,
    tick_count: usize,
    auto_running: Cell<bool>,
    wave_history: Vec<(usize, usize)>, // (acquired, released) per tick
    alloc_history: Vec<usize>,         // size history
    keybindings: KeybindingSet,
    dirty: bool,
    status_bar: RefCell<StatusBar>,
}

impl CellPoolScene {
    pub fn new(theme: Theme) -> Self {
        let status_bar = StatusBar::new(WidgetId::new(2004))
            .add_segment(StatusSegment::new(
                "SPACE:alloc | a:auto | +/-:speed | r:reset | F1:help | Esc:back",
            ))
            .with_theme(theme.clone());
        Self {
            pool: CellPool::new(),
            theme,
            show_help: false,
            acquired: 0,
            released: 0,
            total_cells: 0,
            tick_count: 0,
            auto_running: Cell::new(false),
            wave_history: Vec::new(),
            alloc_history: Vec::new(),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            dirty: true,
            status_bar: RefCell::new(status_bar),
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
            let n = pw as usize * ph as usize;
            self.released += n;
            release_plane_cells(&mut self.pool, pw, ph, cells);
            n
        } else {
            0
        };

        self.total_cells = self.pool.total_cells();
        self.wave_history.push((acq, rel));
        if self.wave_history.len() > 80 {
            self.wave_history.remove(0);
        }
        self.alloc_history.push(acq);
        if self.alloc_history.len() > 80 {
            self.alloc_history.remove(0);
        }
        self.tick_count += 1;
    }

    fn reset(&mut self) {
        self.pool = CellPool::new();
        self.acquired = 0;
        self.released = 0;
        self.total_cells = 0;
        self.tick_count = 0;
        self.auto_running.set(false);
        self.wave_history.clear();
        self.alloc_history.clear();
        self.dirty = true;
    }
}

impl Scene for CellPoolScene {
    fn scene_id(&self) -> &str {
        "cell_pool"
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // ── Header ──────────────────────────────────────────────────────
        draw_text(
            &mut plane,
            2,
            0,
            " Memory Visualizer ",
            t.primary,
            t.bg,
            true,
        );
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );
        if self.auto_running.get() {
            draw_text(
                &mut plane,
                DIV_X.saturating_sub(6),
                0,
                "▶ AUTO",
                t.primary,
                t.bg,
                true,
            );
        }

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Left sidebar ───────────────────────────────────────────────
        self.render_sidebar(&mut plane, area, t);

        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * plane.width + DIV_X) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Main area ───────────────────────────────────────────────────
        let main_x = DIV_X + 2;
        let main_w = area.width.saturating_sub(main_x + 2);

        // Section: Allocation Wave Chart
        let chart_y = 2;
        let chart_h = 12u16.min(area.height.saturating_sub(10));
        self.render_wave_chart(&mut plane, main_x, chart_y, main_w, chart_h);

        // Section: Pool Memory Grid
        let grid_y = chart_y + chart_h + 2;
        if grid_y + 10 < area.height.saturating_sub(4) {
            self.render_pool_grid(
                &mut plane,
                main_x,
                grid_y,
                main_w,
                area.height.saturating_sub(grid_y + 4),
                t,
            );
        }

        // ── Footer ─────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " SPACE:alloc | a:auto | +/-:speed | r:reset | {}:help | {}:back ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                &self.theme,
                "Memory Visualizer — Help",
                &[
                    ("SPACE", "Single allocation"),
                    ("a", "Toggle auto indicator"),
                    ("r", "Reset all stats"),
                    ("Click", "Allocate"),
                    (help_key, "Toggle this help"),
                    (back_key, "Back"),
                ],
            );
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_area = Rect::new(0, sb_y, area.width, 1);
        self.status_bar.borrow_mut().set_area(sb_area);
        let sb_plane = self.status_bar.borrow().render(sb_area);
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
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
                self.auto_running.set(!self.auto_running.get());
                self.dirty = true;
                true
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.auto_running.set(!self.auto_running.get());
                self.dirty = true;
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.reset();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        match kind {
            MouseEventKind::Down(_) => {
                self.simulate_allocation();
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.status_bar.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl CellPoolScene {
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let sx = 2u16;

        // Controls section
        draw_text(plane, sx, 2, "Controls", t.primary, t.bg, true);

        // Auto toggle button
        let btn_y = 3;
        let is_running = self.auto_running.get();
        let btn_bg = if is_running { t.warning } else { t.success };
        let btn_text = if is_running { "■ Stop" } else { "▶ Auto" };
        for cx in 0..SIDEBAR_W {
            let idx = (btn_y * plane.width + sx + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = btn_bg;
                plane.cells[idx].transparent = false;
            }
        }
        draw_text_clipped(
            plane,
            sx + 1,
            btn_y,
            btn_text,
            sx + SIDEBAR_W,
            t.bg,
            btn_bg,
            false,
        );

        // Reset button
        let reset_y = 5;
        let reset_text = "↺ Reset All";
        for cx in 0..SIDEBAR_W {
            let idx = (reset_y * plane.width + sx + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        draw_text(plane, sx, reset_y, reset_text, t.fg, t.surface, false);

        // Divider
        let div1_y = 7;
        for dx in 0..SIDEBAR_W {
            let idx = (div1_y * plane.width + sx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Stats section
        let stats_y = 9;
        draw_text(plane, sx, stats_y, "Statistics", t.secondary, t.bg, true);

        let active = self.acquired.saturating_sub(self.released);
        let stats = [
            ("Acquired", self.acquired.to_string()),
            ("Released", self.released.to_string()),
            ("Active", active.to_string()),
            ("Pooled", self.total_cells.to_string()),
            ("Ticks", self.tick_count.to_string()),
        ];

        for (i, (label, _value)) in stats.iter().enumerate() {
            let sy = stats_y + 1 + i as u16;
            if sy >= area.height.saturating_sub(4) {
                break;
            }
            draw_text(plane, sx, sy, label, t.fg_muted, t.bg, false);
        }

        // Gauge: Reuse Rate
        let reuse_y = stats_y + 7;
        if reuse_y + 2 < area.height.saturating_sub(4) {
            draw_text(plane, sx, reuse_y, "Reuse Rate", t.secondary, t.bg, true);
            let reuse_rate = if self.acquired > 0 {
                (self.released as f64 / self.acquired as f64 * 100.0).min(100.0)
            } else {
                0.0
            };

            let bar_y = reuse_y + 1;
            let bar_w = SIDEBAR_W - 2;
            let filled = ((reuse_rate / 100.0) * bar_w as f64) as usize;
            for dx in 0..bar_w {
                let idx = (bar_y * plane.width + sx + 1 + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = if (dx as usize) < filled { '█' } else { '░' };
                    plane.cells[idx].fg = if (dx as usize) < filled {
                        t.success
                    } else {
                        t.fg_muted
                    };
                    plane.cells[idx].transparent = false;
                }
            }
            let pct_text = format!("{:3.0}%", reuse_rate);
            draw_text(plane, sx + 1, bar_y + 1, &pct_text, t.success, t.bg, false);
        }

        // Efficiency meter
        let eff_y = reuse_y + 3;
        if eff_y + 2 < area.height.saturating_sub(4) {
            draw_text(plane, sx, eff_y, "Pool Efficiency", t.secondary, t.bg, true);
            let pool_pct = if self.total_cells > 0 && self.acquired > 0 {
                (self.total_cells as f64 / self.acquired.max(1) as f64 * 100.0).min(100.0)
            } else {
                0.0
            };

            let bar_y = eff_y + 1;
            let bar_w = SIDEBAR_W - 2;
            let filled = ((pool_pct / 100.0) * bar_w as f64) as usize;
            for dx in 0..bar_w {
                let idx = (bar_y * plane.width + sx + 1 + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = if (dx as usize) < filled { '█' } else { '░' };
                    plane.cells[idx].fg = if (dx as usize) < filled {
                        t.primary
                    } else {
                        t.fg_muted
                    };
                    plane.cells[idx].transparent = false;
                }
            }
            let pct_text = format!("{:3.0}%", pool_pct);
            draw_text(plane, sx + 1, bar_y + 1, &pct_text, t.primary, t.bg, false);
        }

        // Legend
        let leg_y = area.height.saturating_sub(6);
        if leg_y > eff_y + 4 {
            draw_text(plane, sx, leg_y, "Legend", t.secondary, t.bg, true);
            let legends = [
                ("█", "Active", t.success),
                ("▓", "Pooled", t.info),
                ("░", "Free", t.fg_muted),
            ];
            for (i, (ch, label, color)) in legends.iter().enumerate() {
                let ly = leg_y + 1 + i as u16;
                let icon_idx = (ly * plane.width + sx) as usize;
                if icon_idx < plane.cells.len() {
                    plane.cells[icon_idx].char = ' ';
                }
                draw_text(plane, sx + 1, ly, ch, *color, t.bg, false);
                draw_text_clipped(plane, sx + 4, ly, label, sx + SIDEBAR_W, t.fg, t.bg, false);
            }
        }
    }

    fn render_wave_chart(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16) {
        let t = &self.theme;
        if self.wave_history.is_empty() || w < 4 || h < 3 {
            return;
        }

        // Border
        for i in 0..w {
            let top = (y * plane.width + x + i) as usize;
            let bot = ((y + h - 1) * plane.width + x + i) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
            }
        }
        for j in 0..h {
            let left = ((y + j) * plane.width + x) as usize;
            let right = ((y + j) * plane.width + x + w - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = t.outline;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = t.outline;
            }
        }
        // Corners
        for (ch, cx, cy) in [
            ('╭', x, y),
            ('╮', x + w - 1, y),
            ('╰', x, y + h - 1),
            ('╯', x + w - 1, y + h - 1),
        ] {
            let idx = (cy * plane.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = t.outline;
            }
        }

        // Find max for scaling
        let max_val = self
            .wave_history
            .iter()
            .map(|(a, r)| (*a).max(*r))
            .max()
            .unwrap_or(1)
            .max(1);

        // Draw bars for each tick (acquired = green, released = dim)
        let chart_w = (w as usize).saturating_sub(2);
        let chart_h = (h as usize).saturating_sub(2);
        let data: Vec<(usize, usize)> = self
            .wave_history
            .iter()
            .rev()
            .take(chart_w)
            .cloned()
            .collect();

        for (i, (acq, rel)) in data.iter().enumerate() {
            let bar_x = x + 1 + (chart_w.saturating_sub(data.len()) + i) as u16;
            let acq_h =
                (*acq as f64 / max_val as f64 * chart_h as f64).min(chart_h as f64) as usize;
            let rel_h =
                (*rel as f64 / max_val as f64 * chart_h as f64).min(chart_h as f64) as usize;

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
            // Released bars (stacked, show as dim if overlaps)
            for j in 0..rel_h {
                let by = y + h - 2 - j as u16;
                let idx = (by * plane.width + bar_x) as usize;
                if idx < plane.cells.len() && plane.cells[idx].char != '▓' {
                    plane.cells[idx].char = '▒';
                    plane.cells[idx].fg = t.fg_muted;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Title
        draw_text_clipped(
            plane,
            x + 2,
            y + 1,
            "Allocation Waves",
            x + w - 2,
            t.fg_muted,
            t.bg,
            false,
        );

        // Legend
        let legend_x = x + w.saturating_sub(18);
        draw_text_clipped(
            plane,
            legend_x,
            y + 1,
            "▓ Acq  ▒ Rel",
            x + w - 2,
            t.fg_muted,
            t.bg,
            false,
        );
    }

    fn render_pool_grid(&self, plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: &Theme) {
        draw_text(plane, x, y, "Pool Memory Grid", t.secondary, t.bg, true);
        for dx in 0..w {
            let idx = ((y + 1) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Visual grid showing pool state
        let grid_x = x + 1;
        let grid_y = y + 2;
        let grid_w = w.saturating_sub(2);
        let grid_h = h.saturating_sub(3);

        let total_slots = grid_w as usize * grid_h as usize;
        let active = self.acquired.saturating_sub(self.released);
        let pooled = self.total_cells;

        let active_slots = if self.acquired > 0 {
            (active as f64 / self.acquired.max(1) as f64 * total_slots as f64)
                .min(total_slots as f64) as usize
        } else {
            0
        };
        let pooled_slots = if self.acquired > 0 {
            (pooled as f64 / self.acquired.max(1) as f64 * total_slots as f64)
                .min((total_slots - active_slots) as f64) as usize
        } else {
            0
        };

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
    }
}
