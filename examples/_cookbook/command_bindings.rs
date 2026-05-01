//! Command Bindings — Auto-refresh widgets via CLI commands.
//!
//! Demonstrates all 5 command-bound widgets with mock commands that update
//! them on configurable intervals. Shows the core framework value proposition:
//! any widget can be bound to a CLI command with custom parsers.
//!
//! ## Widgets
//!
//! | Widget | Mock Command | Parser | Interval |
//! |--------|-------------|--------|----------|
//! | Gauge (CPU) | simulated | direct | 2s |
//! | KeyValueGrid | simulated | direct | 5s |
//! | StatusBadge | simulated | direct | 10s |
//! | LogViewer | simulated | direct | 3s |
//! | StreamingText | simulated | direct | 1s |
//!
//! ## Controls
//!
//! - `s` — trigger manual refresh of all commands
//! - `p` — pause/resume auto-refresh
//! - `Ctrl+C` — quit

use std::collections::BTreeMap;
use std::time::Duration;

use dracon_terminal_engine::compositor::{Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText};
use ratatui::layout::Rect;

struct CommandBindings {
    id: WidgetId,
    gauge: Gauge,
    kv_grid: KeyValueGrid,
    status: StatusBadge,
    log_viewer: LogViewer,
    streaming: StreamingText,
    theme: Theme,
    area: Rect,
    dirty: bool,
    paused: bool,
    tick: u64,
    cpu_value: f32,
}

impl CommandBindings {
    fn new() -> Self {
        Self {
            id: WidgetId::new(0),
            gauge: Gauge::new("CPU").max(100.0).warn_threshold(70.0).crit_threshold(90.0),
            kv_grid: KeyValueGrid::new().separator(" "),
            status: StatusBadge::new(WidgetId::new(4)).with_status("OK").with_label("Connection"),
            log_viewer: LogViewer::with_id(WidgetId::new(5)).max_lines(200),
            streaming: StreamingText::with_id(WidgetId::new(6)).max_lines(50),
            theme: Theme::nord(),
            area: Rect::new(0, 0, 80, 24),
            dirty: true,
            paused: false,
            tick: 0,
            cpu_value: 50.0,
        }
    }

    fn refresh_all(&mut self) {
        self.cpu_value = 30.0 + (self.tick % 50) as f32;
        self.gauge.set_value(self.cpu_value as f64);
        let mut pairs = BTreeMap::new();
        pairs.insert("Memory".to_string(), format!("{:.1} GB", 8.0 + (self.tick % 10) as f32 * 0.1));
        pairs.insert("Disk".to_string(), format!("{}%", 40 + (self.tick % 20) as u32));
        pairs.insert("Network".to_string(), format!("{} Mbps", 100 + (self.tick % 50) as u32));
        pairs.insert("Uptime".to_string(), format!("{}h", self.tick / 60));
        self.kv_grid.set_pairs(pairs);
        let status_text = if self.tick % 20 < 15 { "OK" } else { "WARNING" };
        self.status.set_status(status_text);
        self.log_viewer.clear();
        self.log_viewer.append_line(&format!("[INFO] Tick {} - System nominal", self.tick));
        self.log_viewer.append_line(&format!("[WARN] Load average: {:.2}", 1.5 + (self.tick % 10) as f32 * 0.1));
        if self.tick % 10 == 0 {
            self.log_viewer.append_line(&format!("[ERROR] Simulated connection issue at tick {}", self.tick));
        }
        self.streaming.clear();
        self.streaming.append(&format!("Last tick: {} @ {:.1}s", self.tick, self.cpu_value));
        self.dirty = true;
    }

    fn tick(&mut self, elapsed_secs: u64) {
        self.tick += 1;
        if self.paused { return; }
        if elapsed_secs % 2 == 0 {
            self.cpu_value = 30.0 + (self.tick % 50) as f32;
            self.gauge.set_value(self.cpu_value as f64);
        }
        if elapsed_secs % 5 == 0 {
            let mut pairs = BTreeMap::new();
            pairs.insert("Memory".to_string(), format!("{:.1} GB", 8.0 + (self.tick % 10) as f32 * 0.1));
            pairs.insert("Disk".to_string(), format!("{}%", 40 + (self.tick % 20) as u32));
            self.kv_grid.set_pairs(pairs);
        }
        if elapsed_secs % 10 == 0 {
            let status_text = if self.tick % 20 < 15 { "OK" } else { "WARNING" };
            self.status.set_status(status_text);
        }
        if elapsed_secs % 3 == 0 {
            self.log_viewer.clear();
            self.log_viewer.append_line(&format!("[INFO] Tick {}", self.tick));
            self.log_viewer.append_line(&format!("[WARN] Load: {:.1}", 1.5 + (self.tick % 10) as f32 * 0.1));
        }
        self.streaming.append(&format!("T:{} ", self.tick));
        self.dirty = true;
    }
}

impl Default for CommandBindings {
    fn default() -> Self { Self::new() }
}

impl Widget for CommandBindings {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; self.dirty = true; }
    fn z_index(&self) -> u16 { 10 }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }

    fn render(&self, area: Rect) -> Plane {
        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 10;

        for idx in 0..p.cells.len() { p.cells[idx].bg = self.theme.bg; p.cells[idx].fg = self.theme.fg; }

        let title = " Command Bindings — Auto-refresh ";
        let title_color = Color::Rgb(0, 255, 200);
        let title_width = title.len() as u16;
        let title_x = (area.width.saturating_sub(title_width)) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (i as u16 + title_x) as usize;
            if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = title_color; p.cells[idx].style = Styles::BOLD; }
        }

        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < p.cells.len() { p.cells[idx].char = '─'; p.cells[idx].fg = Color::Rgb(100, 100, 100); }
        }

        // Gauge
        let gauge_area = Rect::new(0, 2, 25, 4);
        let gauge_plane = self.gauge.render(gauge_area);
        for y in 0..gauge_plane.height {
            for x in 0..gauge_plane.width {
                let src_idx = (y * gauge_plane.width + x) as usize;
                let dst_idx = ((y + 2) * area.width + x) as usize;
                if src_idx < gauge_plane.cells.len() && dst_idx < p.cells.len() {
                    p.cells[dst_idx] = gauge_plane.cells[src_idx].clone();
                }
            }
        }

        // StatusBadge
        let status_area = Rect::new(0, 6, 25, 3);
        let status_plane = self.status.render(status_area);
        for y in 0..status_plane.height {
            for x in 0..status_plane.width {
                let src_idx = (y * status_plane.width + x) as usize;
                let dst_idx = ((y + 6) * area.width + x) as usize;
                if src_idx < status_plane.cells.len() && dst_idx < p.cells.len() {
                    p.cells[dst_idx] = status_plane.cells[src_idx].clone();
                }
            }
        }

        // KeyValueGrid
        let kv_area = Rect::new(26, 2, area.width - 26, 4);
        let kv_plane = self.kv_grid.render(kv_area);
        for y in 0..kv_plane.height {
            for x in 0..kv_plane.width {
                let src_idx = (y * kv_plane.width + x) as usize;
                let dst_idx = ((y + 2) * area.width + x + 26) as usize;
                if src_idx < kv_plane.cells.len() && dst_idx < p.cells.len() {
                    p.cells[dst_idx] = kv_plane.cells[src_idx].clone();
                }
            }
        }

        // LogViewer
        let log_area = Rect::new(26, 6, area.width - 26, 6);
        let log_plane = self.log_viewer.render(log_area);
        for y in 0..log_plane.height {
            for x in 0..log_plane.width {
                let src_idx = (y * log_plane.width + x) as usize;
                let dst_idx = ((y + 6) * area.width + x + 26) as usize;
                if src_idx < log_plane.cells.len() && dst_idx < p.cells.len() {
                    p.cells[dst_idx] = log_plane.cells[src_idx].clone();
                }
            }
        }

        // StreamingText
        let stream_area = Rect::new(0, area.height - 3, area.width, 3);
        let stream_plane = self.streaming.render(stream_area);
        for y in 0..stream_plane.height {
            for x in 0..stream_plane.width {
                let src_idx = (y * stream_plane.width + x) as usize;
                let dst_idx = ((y + area.height - 3) * area.width + x) as usize;
                if src_idx < stream_plane.cells.len() && dst_idx < p.cells.len() {
                    p.cells[dst_idx] = stream_plane.cells[src_idx].clone();
                }
            }
        }

        let status_line_y = (area.height - 1) as usize;
        let paused_str = if self.paused { "PAUSED" } else { "ON" };
        let status_text = format!(" Auto-refresh: {} | s=refresh p=pause ", paused_str);
        for (i, c) in status_text.chars().enumerate() {
            let idx = status_line_y * area.width as usize + i;
            if idx < p.cells.len() { p.cells[idx].char = c; p.cells[idx].fg = Color::Rgb(100, 100, 100); }
        }

        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Char('s') => { self.refresh_all(); true }
            KeyCode::Char('p') => { self.paused = !self.paused; self.dirty = true; true }
            _ => false,
        }
    }
}

fn main() -> std::io::Result<()> {
    println!("Command Bindings — s=refresh all, p=pause, Ctrl+C=quit\nStarting...");
    std::thread::sleep(Duration::from_millis(500));

    let mut view = CommandBindings::new();
    view.refresh_all();

    App::new()?
        .title("Command Bindings")
        .fps(20)
        .tick_interval(1000)
        .on_tick(move |ctx, tick| {
            view.tick(tick);
            view.mark_dirty();
            let (w, h) = ctx.compositor().size();
            if view.area.width != w || view.area.height != h {
                view.set_area(Rect::new(0, 0, w, h));
            }
            if view.needs_render() {
                ctx.add_plane(view.render(view.area));
                view.clear_dirty();
            }
        })
        .run(|_| {})
}