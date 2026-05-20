//! Embedded Debug Overlay scene for the showcase.
//!
//! Demonstrates DebugOverlay, Profiler, and HUD widgets
//! with simulated performance metrics and frame timing.

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::debug_overlay::DebugOverlay;
use dracon_terminal_engine::framework::widgets::profiler::{Metric, Profiler};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::Cell;

const SIDEBAR_W: u16 = 22;
const DIV_X: u16 = SIDEBAR_W + 2;

pub struct DebugOverlayScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    debug_overlay: std::cell::RefCell<DebugOverlay>,
    profiler: std::cell::RefCell<Profiler>,
    // Simulated metrics
    frame_count: Cell<u64>,
    fps_history: Cell<[u32; 20]>, // Ring buffer of last 20 FPS
    fps_index: Cell<usize>,
    fps: Cell<u32>,
    frame_time_ms: Cell<f64>,
    cpu_usage: Cell<f32>,
    mem_usage: Cell<f32>,
    draw_calls: Cell<u32>,
    widgets_rendered: Cell<u32>,
    // State
    paused: Cell<bool>,
    show_overlay: Cell<bool>,
    show_profiler: Cell<bool>,
    show_gauges: Cell<bool>,
    dirty: bool,
    area: Cell<Rect>,
}

impl DebugOverlayScene {
    pub fn new(theme: Theme) -> Self {
        let debug_overlay = std::cell::RefCell::new(DebugOverlay::new(WidgetId::new(1))
            .with_theme(theme.clone()));

        let profiler = std::cell::RefCell::new(Profiler::new(WidgetId::new(2))
            .with_theme(theme.clone()));

        Self {
            theme,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            debug_overlay,
            profiler,
            frame_count: Cell::new(0),
            fps_history: Cell::new([60; 20]),
            fps_index: Cell::new(0),
            fps: Cell::new(60),
            frame_time_ms: Cell::new(16.7),
            cpu_usage: Cell::new(12.0),
            mem_usage: Cell::new(45.0),
            draw_calls: Cell::new(24),
            widgets_rendered: Cell::new(8),
            paused: Cell::new(false),
            show_overlay: Cell::new(true),
            show_profiler: Cell::new(true),
            show_gauges: Cell::new(true),
            dirty: true,
            area: Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn tick_metrics(&self) {
        if self.paused.get() {
            return;
        }

        let frame = self.frame_count.get() + 1;
        self.frame_count.set(frame);

        // Simulate FPS fluctuation (55-65 range with occasional dips)
        let base_fps = 60u32;
        let variation = ((frame * 7 + 13) % 17) as i32 - 8;
        let dip = if frame % 60 < 3 { -15 } else { 0 };
        let fps = (base_fps as i32 + variation + dip).max(30) as u32;
        self.fps.set(fps);

        // Update FPS history ring buffer
        let mut history = self.fps_history.get();
        let idx = self.fps_index.get();
        history[idx] = fps;
        self.fps_history.set(history);
        self.fps_index.set((idx + 1) % 20);

        // Frame time from FPS
        let ft = 1000.0 / fps as f64;
        self.frame_time_ms.set(ft);

        // CPU usage fluctuation
        let cpu = 12.0 + ((frame * 3 + 7) % 20) as f32 * 0.5
                  + if frame % 90 < 10 { 15.0 } else { 0.0 };
        self.cpu_usage.set(cpu.min(100.0));

        // Memory slowly grows then GCs
        let mem_cycle = (frame % 200) as f32 / 200.0;
        let mem = 45.0 + mem_cycle * 30.0;
        self.mem_usage.set(mem);

        // Draw calls and widget count
        self.draw_calls.set(24 + (frame % 7) as u32);
        self.widgets_rendered.set(8 + (frame % 4) as u32);
    }

    fn update_debug_overlay(&self) {
        let fps = self.fps.get();
        let ft = self.frame_time_ms.get();
        let cpu = self.cpu_usage.get();
        let mem = self.mem_usage.get();
        let dc = self.draw_calls.get();
        let wr = self.widgets_rendered.get();
        let frame = self.frame_count.get();

        let fps_label = if fps >= 55 { "ok" } else if fps >= 30 { "warn" } else { "bad" };

        self.debug_overlay.borrow_mut().set_lines(vec![
            format!("Frame: #{frame}"),
            format!("FPS: {fps} [{fps_label}]"),
            format!("Frame time: {ft:.1}ms"),
            format!("CPU: {cpu:.0}%"),
            format!("Memory: {mem:.0}%"),
            format!("Draw calls: {dc}"),
            format!("Widgets: {wr}"),
        ]);
    }

    fn update_profiler(&self) {
        let ft = self.frame_time_ms.get();

        self.profiler.borrow_mut().set_metrics(vec![
            Metric {
                name: "render".to_string(),
                value: std::time::Duration::from_micros((ft * 800.0) as u64),
                call_count: 1,
            },
            Metric {
                name: "layout".to_string(),
                value: std::time::Duration::from_micros((ft * 120.0) as u64),
                call_count: 3,
            },
            Metric {
                name: "input".to_string(),
                value: std::time::Duration::from_micros((ft * 30.0) as u64),
                call_count: 2,
            },
            Metric {
                name: "compose".to_string(),
                value: std::time::Duration::from_micros((ft * 50.0) as u64),
                call_count: 1,
            },
            Metric {
                name: "gc".to_string(),
                value: std::time::Duration::from_micros(if self.frame_count.get() % 200 > 190 { (ft * 500.0) as u64 } else { 50 }),
                call_count: if self.frame_count.get() % 200 > 190 { 1 } else { 0 },
            },
        ]);
    }
}

impl Scene for DebugOverlayScene {
    fn scene_id(&self) -> &str { "debug_overlay" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        self.tick_metrics();
        self.update_debug_overlay();
        self.update_profiler();

        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // ── Header ──────────────────────────────────────────────────────
        draw_text(&mut plane, 2, 0, " Performance Monitor ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);
        if self.paused.get() {
            draw_text(&mut plane, DIV_X.saturating_sub(10), 0, "■ PAUSED", t.warning, t.bg, true);
        }

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
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

        // FPS history sparkline
        let spark_y = 2;
        draw_text(&mut plane, main_x, spark_y, "FPS History", t.primary, t.bg, true);
        self.render_fps_sparkline(&mut plane, main_x, spark_y + 2, main_w.min(50), t);

        // Gauge bars
        if self.show_gauges.get() {
            let gauge_y = spark_y + 6;
            if gauge_y + 6 < area.height.saturating_sub(6) {
                draw_text(&mut plane, main_x, gauge_y, "Resource Usage", t.primary, t.bg, true);
                let gauge_w = main_w.min(50);
                self.render_gauge_bar(&mut plane, main_x, gauge_y + 1, gauge_w, "CPU", self.cpu_usage.get(), t);
                self.render_gauge_bar(&mut plane, main_x, gauge_y + 3, gauge_w, "MEM", self.mem_usage.get(), t);
                self.render_gauge_bar(&mut plane, main_x, gauge_y + 5, gauge_w, "GPU", (self.draw_calls.get() as f32 / 50.0 * 100.0).min(100.0), t);
            }
        }

        // ── Widget panels ───────────────────────────────────────────────
        let panel_y = area.height.saturating_sub(12);
        if self.show_overlay.get() {
            let overlay_area = Rect::new(main_x, panel_y, 26, 9);
            let overlay_plane = self.debug_overlay.borrow().render(overlay_area);
            blit_to(&mut plane, &overlay_plane, main_x as usize, panel_y as usize);
        }

        if self.show_profiler.get() {
            let profiler_x = main_x + 28;
            let profiler_area = Rect::new(profiler_x, panel_y, main_w.saturating_sub(28), 9);
            let profiler_plane = self.profiler.borrow().render(profiler_area);
            blit_to(&mut plane, &profiler_plane, profiler_x as usize, panel_y as usize);
        }

        // ── Footer ─────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " SPACE:pause | 1/2/3:toggle | r:reset | {}:help | {}:back ",
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
            render_help_overlay(&mut plane, area, &self.theme, "Performance Monitor — Help", &[
                ("SPACE", "Pause/resume metrics"),
                ("1", "Toggle debug overlay"),
                ("2", "Toggle profiler"),
                ("3", "Toggle gauges"),
                ("r", "Reset counters"),
                (help_key, "Toggle this help"),
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
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.paused.set(!self.paused.get());
                self.dirty = true;
                true
            }
            KeyCode::Char('1') if key.modifiers.is_empty() => {
                self.show_overlay.set(!self.show_overlay.get());
                self.dirty = true;
                true
            }
            KeyCode::Char('2') if key.modifiers.is_empty() => {
                self.show_profiler.set(!self.show_profiler.get());
                self.dirty = true;
                true
            }
            KeyCode::Char('3') if key.modifiers.is_empty() => {
                self.show_gauges.set(!self.show_gauges.get());
                self.dirty = true;
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.frame_count.set(0);
                self.fps_history.set([60; 20]);
                self.fps_index.set(0);
                self.fps.set(60);
                self.frame_time_ms.set(16.7);
                self.cpu_usage.set(12.0);
                self.mem_usage.set(45.0);
                self.draw_calls.set(24);
                self.widgets_rendered.set(8);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Down(_) => {
                // Click sidebar pause button area
                if col < DIV_X && (3..5).contains(&row) {
                    self.paused.set(!self.paused.get());
                    self.dirty = true;
                    return true;
                }
                // Click toggle areas
                if col >= DIV_X {
                    if (3..5).contains(&row) { // toggles row
                        self.show_overlay.set(!self.show_overlay.get());
                        self.dirty = true;
                        return true;
                    }
                    if (5..7).contains(&row) {
                        self.show_profiler.set(!self.show_profiler.get());
                        self.dirty = true;
                        return true;
                    }
                    if (7..9).contains(&row) {
                        self.show_gauges.set(!self.show_gauges.get());
                        self.dirty = true;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.debug_overlay.borrow_mut().on_theme_change(theme);
        self.profiler.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl DebugOverlayScene {
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let sx = 2u16;

        // Title
        draw_text(plane, sx, 2, "Controls", t.primary, t.bg, true);

        // Pause button
        let btn_y = 3;
        let is_paused = self.paused.get();
        let btn_bg = if is_paused { t.success } else { t.warning };
        let btn_text = if is_paused { "▶ Resume" } else { "■ Pause" };
        for cx in 0..SIDEBAR_W {
            let idx = (btn_y * plane.width + sx + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = btn_bg;
                plane.cells[idx].transparent = false;
            }
        }
        draw_text_clipped(plane, sx + 1, btn_y, btn_text, sx + SIDEBAR_W, t.bg, btn_bg, false);

        // Reset button
        let reset_y = 5;
        let reset_text = "↺ Reset Counters";
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

        // Panel toggles
        let toggle_y = 9;
        draw_text(plane, sx, toggle_y, "Panels", t.secondary, t.bg, true);

        let toggles = [
            ("1 Overlay", self.show_overlay.get()),
            ("2 Profiler", self.show_profiler.get()),
            ("3 Gauges", self.show_gauges.get()),
        ];

        for (i, (label, enabled)) in toggles.iter().enumerate() {
            let ty = toggle_y + 1 + i as u16;
            let icon = if *enabled { "●" } else { "○" };
            let icon_color = if *enabled { t.success } else { t.fg_muted };

            let icon_idx = (ty * plane.width + sx) as usize;
            if icon_idx < plane.cells.len() {
                plane.cells[icon_idx].char = ' ';
            }
            draw_text(plane, sx + 1, ty, icon, icon_color, t.bg, false);
            draw_text_clipped(plane, sx + 4, ty, label, sx + SIDEBAR_W, t.fg, t.bg, false);
        }

        // Live metrics
        let metrics_y = area.height.saturating_sub(10);
        if metrics_y > toggle_y + 5 {
            draw_text(plane, sx, metrics_y, "Live Metrics", t.secondary, t.bg, true);

            let fps = self.fps.get();
            let ft = self.frame_time_ms.get();
            let cpu = self.cpu_usage.get();
            let mem = self.mem_usage.get();
            let dc = self.draw_calls.get();
            let wr = self.widgets_rendered.get();

            let fps_color = if fps >= 55 { t.success } else if fps >= 30 { t.warning } else { t.error };
            let metrics = [
                ("FPS", format!("{}", fps), fps_color),
                ("Frame", format!("{:.1}ms", ft), t.fg),
                ("CPU", format!("{:.0}%", cpu), if cpu > 80.0 { t.error } else if cpu > 50.0 { t.warning } else { t.success }),
                ("MEM", format!("{:.0}%", mem), if mem > 80.0 { t.error } else if mem > 60.0 { t.warning } else { t.info }),
                ("Draw", format!("{}", dc), t.fg),
                ("Widgets", format!("{}", wr), t.fg),
            ];

            for (i, (label, value, color)) in metrics.iter().enumerate() {
                let my = metrics_y + 1 + i as u16;
                if my >= area.height.saturating_sub(4) { break; }
                draw_text(plane, sx, my, label, t.fg_muted, t.bg, false);
                draw_text_clipped(plane, sx + 10, my, value, sx + SIDEBAR_W, *color, t.bg, false);
            }
        }

        // Frame counter
        let frame_y = area.height.saturating_sub(4);
        if frame_y > metrics_y + 8 {
            draw_text(plane, sx, frame_y, "Frame", t.secondary, t.bg, true);
            let frame_text = format!("#{}", self.frame_count.get());
            draw_text_clipped(plane, sx, frame_y + 1, &frame_text, sx + SIDEBAR_W, t.fg, t.bg, false);
        }
    }

    fn render_fps_sparkline(&self, plane: &mut Plane, x: u16, y: u16, w: u16, t: &Theme) {
        let history = self.fps_history.get();
        let idx = self.fps_index.get();
        let chart_w = w.min(40);
        let chart_h = 6u16;

        // Border
        for i in 0..chart_w {
            let top = (y * plane.width + x + i) as usize;
            let bot = ((y + chart_h - 1) * plane.width + x + i) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for j in 0..chart_h {
            let left = ((y + j) * plane.width + x) as usize;
            let right = ((y + j) * plane.width + x + chart_w - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }

        // Find min/max for scaling
        let max_fps = 70u32; // fixed scale
        let min_fps = 20u32;

        // Draw sparkline
        for i in 0..chart_w {
            // Get FPS value at this position (ring buffer order)
            let pos = (idx + i as usize) % 20;
            let fps = history[pos];

            // Scale to chart height
            let ratio = (fps.saturating_sub(min_fps) as f64 / (max_fps - min_fps) as f64).clamp(0.0, 1.0);
            let bar_height = (ratio * (chart_h - 2) as f64) as usize;

            // Draw bar from bottom
            for j in 0..bar_height {
                let by = y + chart_h - 2 - j as u16;
                let idx = (by * plane.width + x + 1 + i) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '█';
                    plane.cells[idx].fg = if fps >= 55 { t.success } else if fps >= 30 { t.warning } else { t.error };
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Scale labels
        draw_text(plane, x + 1, y + chart_h - 1, "20", t.fg_muted, t.bg, false);
        draw_text(plane, x + chart_w - 3, y + chart_h - 1, "70", t.fg_muted, t.bg, false);
    }

    fn render_gauge_bar(&self, plane: &mut Plane, x: u16, y: u16, w: u16, label: &str, value: f32, t: &Theme) {
        let color = if value > 80.0 { t.error } else if value > 50.0 { t.warning } else { t.success };

        // Label
        draw_text(plane, x, y, label, t.fg, t.bg, false);

        // Bar
        let bar_x = x + 5;
        let bar_w = w.saturating_sub(8);
        let filled = ((value / 100.0) * bar_w as f32).min(bar_w as f32) as usize;

        for bx in 0..bar_w {
            let idx = (y * plane.width + bar_x + bx as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if bx < filled { '█' } else { '░' };
                plane.cells[idx].fg = if bx < filled { color } else { t.fg_muted };
                plane.cells[idx].transparent = false;
            }
        }

        // Percentage
        let pct = format!("{:.0}%", value);
        draw_text(plane, bar_x + bar_w + 1, y, &pct, color, t.bg, true);
    }
}