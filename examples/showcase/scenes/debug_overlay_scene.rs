//! Embedded Debug Overlay scene for the showcase.
//!
//! Demonstrates DebugOverlay, Profiler, and HUD widgets
//! with simulated performance metrics and frame timing.

use crate::scenes::shared_helpers::{blit_to, draw_text};
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
use std::time::Duration;

pub struct DebugOverlayScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    debug_overlay: std::cell::RefCell<DebugOverlay>,
    profiler: std::cell::RefCell<Profiler>,
    // Simulated metrics
    frame_count: Cell<u64>,
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

struct GaugeBarConfig<'a> {
    x: u16,
    y: u16,
    w: u16,
    label: &'a str,
    value: f32,
    color: Color,
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
        let variation = ((frame * 7 + 13) % 17) as i32 - 8; // deterministic noise
        let dip = if frame % 60 < 3 { -15 } else { 0 }; // periodic dips
        let fps = (base_fps as i32 + variation + dip).max(30) as u32;
        self.fps.set(fps);

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

        let fps_color = if fps >= 55 { "ok" } else if fps >= 30 { "warn" } else { "bad" };

        self.debug_overlay.borrow_mut().set_lines(vec![
            format!("Frame: #{frame}"),
            format!("FPS: {fps} [{fps_color}]"),
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
                value: Duration::from_micros((ft * 800.0) as u64),
                call_count: 1,
            },
            Metric {
                name: "layout".to_string(),
                value: Duration::from_micros((ft * 120.0) as u64),
                call_count: 3,
            },
            Metric {
                name: "input".to_string(),
                value: Duration::from_micros((ft * 30.0) as u64),
                call_count: 2,
            },
            Metric {
                name: "compose".to_string(),
                value: Duration::from_micros((ft * 50.0) as u64),
                call_count: 1,
            },
            Metric {
                name: "gc".to_string(),
                value: Duration::from_micros(if self.frame_count.get() % 200 > 190 { (ft * 500.0) as u64 } else { 50 }),
                call_count: if self.frame_count.get() % 200 > 190 { 1 } else { 0 },
            },
        ]);
    }
}

impl DebugOverlayScene {
    fn render_gauge_bar(&self, plane: &mut Plane, cfg: &GaugeBarConfig<'_>) {
        let t = &self.theme;
        let x = cfg.x;
        let y = cfg.y;

        // Label
        draw_text(plane, x, y, cfg.label, t.fg, t.bg, false);

        // Bar background
        let bar_x = x + cfg.label.len() as u16 + 1;
        let bar_w = cfg.w.saturating_sub(cfg.label.len() as u16 + 8);

        for bx in 0..bar_w {
            let idx = (y * plane.width + bar_x + bx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '░';
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].transparent = false;
            }
        }

        // Bar fill
        let filled = (cfg.value / 100.0 * bar_w as f32) as u16;
        for bx in 0..filled.min(bar_w) {
            let idx = (y * plane.width + bar_x + bx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '█';
                plane.cells[idx].fg = cfg.color;
                plane.cells[idx].transparent = false;
            }
        }

        // Percentage
        let pct = format!("{:.0}%", cfg.value);
        draw_text(plane, bar_x + bar_w + 1, y, &pct, cfg.color, t.bg, true);
    }

    fn fps_color(&self) -> Color {
        let fps = self.fps.get();
        let t = &self.theme;
        if fps >= 55 { t.success }
        else if fps >= 30 { t.warning }
        else { t.error }
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

        // Header
        draw_text(&mut plane, 2, 0, " Debug Overlay ", t.primary, t.bg, true);
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
        draw_text(&mut plane, 2, 2, "Real-time performance metrics and debug info", t.fg, t.bg, false);
        let paused_text = if self.paused.get() { " [PAUSED]" } else { "" };
        draw_text(&mut plane, 48, 2, paused_text, t.warning, t.bg, true);

        // ── FPS Counter (top-left) ──────────────────────────────────────────
        let fps = self.fps.get();
        let fps_text = format!("{} FPS", fps);
        let fps_color = self.fps_color();
        draw_text(&mut plane, 2, 4, &fps_text, fps_color, t.bg, true);

        // Frame time
        let ft = self.frame_time_ms.get();
        let ft_text = format!("{:.1}ms", ft);
        draw_text(&mut plane, 14, 4, &ft_text, t.fg_muted, t.bg, false);

        // Frame counter
        let frame_text = format!("Frame #{}", self.frame_count.get());
        draw_text(&mut plane, 2, 5, &frame_text, t.fg_muted, t.bg, false);

        // ── Gauge Bars ──────────────────────────────────────────────────────
        if self.show_gauges.get() {
            let gauge_w = area.width.saturating_sub(4);
            self.render_gauge_bar(&mut plane, &GaugeBarConfig {
                x: 2, y: 7, w: gauge_w, label: "CPU",
                value: self.cpu_usage.get(),
                color: if self.cpu_usage.get() > 80.0 { t.error } else if self.cpu_usage.get() > 50.0 { t.warning } else { t.success },
            });
            self.render_gauge_bar(&mut plane, &GaugeBarConfig {
                x: 2, y: 9, w: gauge_w, label: "MEM",
                value: self.mem_usage.get(),
                color: if self.mem_usage.get() > 80.0 { t.error } else if self.mem_usage.get() > 60.0 { t.warning } else { t.info },
            });
            self.render_gauge_bar(&mut plane, &GaugeBarConfig {
                x: 2, y: 11, w: gauge_w, label: "GPU",
                value: (self.draw_calls.get() as f32 / 50.0 * 100.0).min(100.0),
                color: t.secondary,
            });
        }

        // ── DebugOverlay widget ────────────────────────────────────────────
        if self.show_overlay.get() {
            let overlay_area = Rect::new(2, 13, 28, 9);
            let overlay_plane = self.debug_overlay.borrow().render(overlay_area);
            blit_to(&mut plane, &overlay_plane, overlay_area.x as usize, overlay_area.y as usize);
        }

        // ── Profiler widget ────────────────────────────────────────────────
        if self.show_profiler.get() {
            let profiler_x = 32u16;
            let profiler_area = Rect::new(profiler_x, 13, area.width.saturating_sub(profiler_x + 2), 9);
            let profiler_plane = self.profiler.borrow().render(profiler_area);
            blit_to(&mut plane, &profiler_plane, profiler_area.x as usize, profiler_area.y as usize);
        }

        // ── Render Stats Summary ───────────────────────────────────────────
        let stats_y = area.height.saturating_sub(3);
        draw_text(&mut plane, 2, stats_y, "Stats:", t.primary, t.bg, true);
        let dc = self.draw_calls.get();
        let wr = self.widgets_rendered.get();
        let stats = format!("Draw calls: {} | Widgets: {} | Overhead: {:.1}ms",
            dc, wr, self.frame_time_ms.get() * 0.1);
        draw_text(&mut plane, 2, stats_y + 1, &stats, t.fg, t.bg, false);

        // Footer
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " p:pause | 1:overlay | 2:profiler | 3:gauges | {}:help | {}:back ",
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
            KeyCode::Char('p') if key.modifiers.is_empty() => {
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
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Down(_) => {
                // [PAUSED] indicator: row 2, col ~48-58
                if row == 2 && (48..60).contains(&col) {
                    self.paused.set(!self.paused.get());
                    self.dirty = true;
                    return true;
                }
                // Gauge bars area (rows 7-12): click to toggle gauges
                if (7..12).contains(&row) {
                    self.show_gauges.set(!self.show_gauges.get());
                    self.dirty = true;
                    return true;
                }
                // DebugOverlay area (rows 13-21, cols 2-29): toggle overlay
                if (13..22).contains(&row) && (2..30).contains(&col) {
                    self.show_overlay.set(!self.show_overlay.get());
                    self.dirty = true;
                    return true;
                }
                // Profiler area (rows 13-21, cols 32+): toggle profiler
                if (13..22).contains(&row) && col >= 32 {
                    self.show_profiler.set(!self.show_profiler.get());
                    self.dirty = true;
                    return true;
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

    fn needs_render(&self) -> bool { true } // always re-render for live metrics
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl DebugOverlayScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 48u16.min(area.width.saturating_sub(4));
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

        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let lines = [
            ("╭────────────────────────────────────────────────╮", true),
            ("│         Debug Overlay Help                     │", true),
            ("├────────────────────────────────────────────────┤", true),
            ("│  p        Pause/resume metrics                │", false),
            ("│  1        Toggle debug overlay widget          │", false),
            ("│  2        Toggle profiler widget               │", false),
            ("│  3        Toggle gauge bars                    │", false),
            (&format!("│  {:<10} Dismiss / go back                  │", back_key), false),
            ("╰────────────────────────────────────────────────╯", true),
        ];
        for (i, (line, is_border)) in lines.iter().enumerate() {
            let ly = hy + i as u16;
            let lx = (area.width - line.len() as u16) / 2;
            for (j, ch) in line.chars().enumerate() {
                let px = lx + j as u16;
                if px < area.width && ly < area.height {
                    let idx = (ly * area.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if *is_border || "│╭╮├┤╰╯─".contains(ch) { t.outline } else { t.fg };
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }
    }
}
