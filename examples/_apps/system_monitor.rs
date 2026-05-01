//! System Monitor — htop-like dashboard with auto-refreshing gauges.
//!
//! Demonstrates Gauge, KeyValueGrid, StatusBadge, StreamingText, and SplitPane widgets
//! in an htop-style monitoring dashboard with auto-refresh and theme cycling.
//!
//! # Widgets Used
//!
//! | Widget | Purpose |
//! |--------|---------|
//! | Gauge ×4 | CPU, Memory, Disk, Network usage bars |
//! | KeyValueGrid | Process list with sorted columns |
//! | StatusBadge | System health indicator |
//! | StreamingText | Live uptime counter |
//! | SplitPane | Layout structure (horizontal + vertical splits) |
//!
//! # Key Patterns
//!
//! 1. Multiple Gauge widgets in a 2×2 grid
//! 2. KeyValueGrid with column sorting
//! 3. StatusBadge with conditional coloring
//! 4. StreamingText for live updating text
//! 5. Auto-refresh via tick callback (every 2 seconds)
//!
//! # Controls
//!
//! - `t` — Cycle through available themes
//! - `Ctrl+C` — Exit
//!
//! # Run
//!
//! ```sh
//! cargo run --example system_monitor
//! ```

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Gauge, KeyValueGrid, StatusBadge, StreamingText,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use std::collections::BTreeMap;
use std::cell::RefCell;

const THEMES: &[&str] = &["nord", "dracula", "cyberpunk", "gruvbox-dark", "tokyo-night"];

struct ProcessInfo {
    name: &'static str,
    cpu: f32,
    mem: &'static str,
    pid: u32,
    status: &'static str,
}

struct SystemStats {
    cpu_percent: f32,
    memory_used: f32,
    memory_total: f32,
    disk_percent: f32,
    network_down: f32,
    network_up: f32,
    uptime_seconds: u64,
    processes: Vec<ProcessInfo>,
}

struct SystemMonitor {
    cpu_gauge: Gauge,
    mem_gauge: Gauge,
    disk_gauge: Gauge,
    net_gauge: Gauge,
    process_grid: KeyValueGrid,
    status_badge: StatusBadge,
    uptime_text: StreamingText,
    stats: SystemStats,
    theme_index: usize,
    start_time: std::time::Instant,
}

impl SystemMonitor {
    fn new() -> Self {
        let theme = Theme::nord();
        Self {
            cpu_gauge: Gauge::with_id(WidgetId::new(1), "CPU")
                .with_theme(theme)
                .warn_threshold(70.0)
                .crit_threshold(90.0),
            mem_gauge: Gauge::with_id(WidgetId::new(2), "Memory")
                .with_theme(theme)
                .warn_threshold(80.0)
                .crit_threshold(95.0),
            disk_gauge: Gauge::with_id(WidgetId::new(3), "Disk")
                .with_theme(theme)
                .warn_threshold(75.0)
                .crit_threshold(90.0),
            net_gauge: Gauge::with_id(WidgetId::new(4), "Network")
                .with_theme(theme)
                .warn_threshold(80.0)
                .crit_threshold(95.0),
            process_grid: KeyValueGrid::with_id(WidgetId::new(5)).with_theme(theme),
            status_badge: StatusBadge::new(WidgetId::new(6)).with_theme(theme),
            uptime_text: StreamingText::with_id(WidgetId::new(7))
                .with_theme(theme)
                .max_lines(1),
            stats: SystemStats {
                cpu_percent: 0.0,
                memory_used: 0.0,
                memory_total: 16.0,
                disk_percent: 0.0,
                network_down: 0.0,
                network_up: 0.0,
                uptime_seconds: 0,
                processes: Vec::new(),
            },
            theme_index: 0,
            start_time: std::time::Instant::now(),
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
        let new_theme = match THEMES[self.theme_index] {
            "nord" => Theme::nord(),
            "dracula" => Theme::dracula(),
            "cyberpunk" => Theme::cyberpunk(),
            "gruvbox-dark" => Theme::gruvbox_dark(),
            "tokyo-night" => Theme::tokyo_night(),
            _ => Theme::nord(),
        };
        self.cpu_gauge = self.cpu_gauge.clone().with_theme(new_theme);
        self.mem_gauge = self.mem_gauge.clone().with_theme(new_theme);
        self.disk_gauge = self.disk_gauge.clone().with_theme(new_theme);
        self.net_gauge = self.net_gauge.clone().with_theme(new_theme);
        self.process_grid = self.process_grid.clone().with_theme(new_theme);
        self.status_badge = self.status_badge.clone().with_theme(new_theme);
        self.uptime_text = self.uptime_text.clone().with_theme(new_theme);
    }

    fn refresh_stats(&mut self) {
        let tick = (self.start_time.elapsed().as_secs() as f32 / 2.0).round() as u32;
        let base = 40.0 + (tick % 5) as f32 * 8.0;
        self.stats.cpu_percent = (base + rand_float(10.0, 25.0)).clamp(5.0, 95.0);
        self.stats.memory_used = (8.0 + rand_float(0.1, 0.5) * (tick % 10) as f32).min(15.5);
        let disk_base = 35.0 + rand_float(0.0, 5.0);
        self.stats.disk_percent = (disk_base + tick as f32 * 0.1).clamp(20.0, 80.0);
        self.stats.network_down = 50.0 + rand_float(10.0, 80.0) * ((tick % 3) as f32 + 1.0);
        self.stats.network_up = 15.0 + rand_float(5.0, 40.0);
        self.stats.uptime_seconds = self.start_time.elapsed().as_secs();

        let cpu_status = if self.stats.cpu_percent >= 80.0 { "WARNING" } else { "HEALTHY" };
        let mem_pct = (self.stats.memory_used / self.stats.memory_total * 100.0) as u32;
        let mem_status = if mem_pct >= 90 { "WARNING" } else { "HEALTHY" };
        let final_status = if cpu_status == "WARNING" || mem_status == "WARNING" {
            "WARNING"
        } else {
            "HEALTHY"
        };
        self.status_badge.set_status(final_status);

        self.cpu_gauge.set_value(self.stats.cpu_percent as f64);
        self.mem_gauge
            .set_value((self.stats.memory_used / self.stats.memory_total * 100.0) as f64);
        self.disk_gauge.set_value(self.stats.disk_percent as f64);
        self.net_gauge.set_value(self.stats.network_down.min(100.0) as f64);

        self.uptime_text.clear();
        self.uptime_text.append(&format_uptime(self.stats.uptime_seconds));

        self.process_grid.set_pairs(self.build_process_pairs());
    }

    fn build_process_pairs(&self) -> BTreeMap<String, String> {
        let mut pairs = BTreeMap::new();
        pairs.insert("Name".to_string(), "CPU%".to_string());
        for p in &self.stats.processes {
            pairs.insert(p.name.to_string(), format!("{:.1}", p.cpu));
        }
        pairs.insert(
            "Memory".to_string(),
            format!("{:.1}/{:.0} GB", self.stats.memory_used, self.stats.memory_total),
        );
        pairs.insert("Disk".to_string(), format!("{:.0}%", self.stats.disk_percent));
        pairs.insert(
            "Net Down".to_string(),
            format!("{:.0} Mbps", self.stats.network_down),
        );
        pairs.insert(
            "Net Up".to_string(),
            format!("{:.0} Mbps", self.stats.network_up),
        );
        pairs
    }

    fn generate_processes(&mut self) {
        let tick = (self.start_time.elapsed().as_secs() / 2) as u32;
        self.stats.processes = vec![
            ProcessInfo {
                name: "firefox",
                cpu: (12.5 + rand_float(-2.0, 2.0)).max(0.1),
                mem: "1.2GB",
                pid: 1234 + (tick % 10),
                status: "Running",
            },
            ProcessInfo {
                name: "chrome",
                cpu: (8.3 + rand_float(-1.5, 1.5)).max(0.1),
                mem: "890MB",
                pid: 5678 + (tick % 5),
                status: "Running",
            },
            ProcessInfo {
                name: "rustc",
                cpu: (5.1 + rand_float(-1.0, 1.0)).max(0.1),
                mem: "450MB",
                pid: 9012 + (tick % 3),
                status: "Running",
            },
            ProcessInfo {
                name: "system",
                cpu: (2.1 + rand_float(-0.5, 0.5)).max(0.1),
                mem: "234MB",
                pid: 1,
                status: "Running",
            },
            ProcessInfo {
                name: "postgres",
                cpu: (1.8 + rand_float(-0.3, 0.3)).max(0.1),
                mem: "156MB",
                pid: 3456 + (tick % 7),
                status: "Running",
            },
            ProcessInfo {
                name: "redis",
                cpu: (0.8 + rand_float(-0.2, 0.2)).max(0.1),
                mem: "45MB",
                pid: 7890 + (tick % 4),
                status: "Running",
            },
        ];
    }
}

fn rand_float(min: f32, max: f32) -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as f32;
    min + (nanos / u32::MAX as f32) * (max - min)
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

impl Widget for SystemMonitor {
    fn id(&self) -> WidgetId {
        WidgetId::new(0)
    }

    fn set_id(&mut self, _id: WidgetId) {}

    fn area(&self) -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    fn set_area(&mut self, _area: Rect) {}

    fn needs_render(&self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {}

    fn clear_dirty(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let theme = Theme::nord();

        render_header(&mut plane, area.width, &theme, THEMES[self.theme_index]);

        let content_top = 2;
        let content_height = area.height.saturating_sub(content_top + 3);
        let content_rect = Rect::new(0, content_top, area.width, content_height);

        let grid_rects = Layout::default()
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(content_rect);

        let top_row =
            Rect::new(grid_rects[0].x, grid_rects[0].y, grid_rects[0].width, grid_rects[0].height / 2);

        let cpu_plane = self.cpu_gauge.render(top_row);
        copy_plane_cells(&mut plane, &cpu_plane, top_row.x as usize, top_row.y as usize);

        let mem_rect = Rect::new(
            grid_rects[1].x,
            grid_rects[1].y,
            grid_rects[1].width,
            grid_rects[1].height / 2,
        );
        let mem_plane = self.mem_gauge.render(mem_rect);
        copy_plane_cells(&mut plane, &mem_plane, mem_rect.x as usize, mem_rect.y as usize);

        let disk_rect = Rect::new(
            grid_rects[0].x,
            grid_rects[0].y + grid_rects[0].height / 2,
            grid_rects[0].width,
            grid_rects[0].height / 2,
        );
        let disk_plane = self.disk_gauge.render(disk_rect);
        copy_plane_cells(&mut plane, &disk_plane, disk_rect.x as usize, disk_rect.y as usize);

        let net_rect = Rect::new(
            grid_rects[1].x,
            grid_rects[1].y + grid_rects[1].height / 2,
            grid_rects[1].width,
            grid_rects[1].height / 2,
        );
        let net_plane = self.net_gauge.render(net_rect);
        copy_plane_cells(&mut plane, &net_plane, net_rect.x as usize, net_rect.y as usize);

        let process_top = content_top + content_height / 2 + 1;
        let process_height = (content_height / 2).saturating_sub(1);
        let process_rect = Rect::new(0, process_top, area.width, process_height);
        let process_plane = self.process_grid.render(process_rect);
        copy_plane_cells(
            &mut plane,
            &process_plane,
            process_rect.x as usize,
            process_rect.y as usize,
        );

        let footer_y = (area.height - 2) as usize;
        render_footer(&mut plane, area.width, footer_y, &theme);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Char('t') => {
                self.cycle_theme();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn focusable(&self) -> bool {
        false
    }

    fn z_index(&self) -> u16 {
        0
    }
}

fn render_header(plane: &mut Plane, width: u16, theme: &Theme, theme_name: &str) {
    let title = "System Monitor";
    let right_label = format!("[Theme: {}]", theme_name);
    let title_len = title.len();
    let right_len = right_label.len();
    let available = (width as usize).saturating_sub(title_len + right_len + 2);
    let padding = available / 2;

    let mut offset = 0;
    for c in title.chars() {
        if offset < plane.cells.len() {
            plane.cells[offset] = Cell {
                char: c,
                fg: theme.accent,
                bg: theme.bg,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        offset += 1;
    }

    offset += padding;
    for c in right_label.chars() {
        if offset < plane.cells.len() {
            plane.cells[offset] = Cell {
                char: c,
                fg: theme.inactive_fg,
                bg: theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
        offset += 1;
    }

    for x in 0..width as usize {
        let idx = (1 * plane.width as usize + x).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx] = Cell {
            char: '─',
            fg: theme.border,
            bg: theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }
}

fn render_footer(plane: &mut Plane, width: u16, footer_y: usize, theme: &Theme) {
    let separator = '─';
    let separator_idx = (footer_y * plane.width as usize).min(plane.cells.len().saturating_sub(1));
    plane.cells[separator_idx] = Cell {
        char: separator,
        fg: theme.border,
        bg: theme.bg,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    for x in 1..width as usize {
        let idx = (footer_y * plane.width as usize + x).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx] = Cell {
            char: separator,
            fg: theme.border,
            bg: theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }

    let badge_text = "✓ System healthy";
    let offset = 1;
    for (i, c) in badge_text.chars().enumerate().take(width as usize - offset) {
        let idx = ((footer_y + 1) * plane.width as usize + offset + i)
            .min(plane.cells.len().saturating_sub(1));
        plane.cells[idx] = Cell {
            char: c,
            fg: theme.success_fg,
            bg: theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }
}

fn copy_plane_cells(dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
    for (i, cell) in src.cells.iter().enumerate() {
        if cell.char == '\0' || cell.transparent {
            continue;
        }
        let src_width = src.width as usize;
        let row = i / src_width;
        let col = i % src_width;
        let dest_row = offset_y + row;
        let dest_col = offset_x + col;
        if dest_row >= dest.height as usize || dest_col >= dest.width as usize {
            continue;
        }
        let dest_idx = dest_row * dest.width as usize + dest_col;
        if dest_idx < dest.cells.len() {
            dest.cells[dest_idx] = cell.clone();
        }
    }
}

fn main() -> std::io::Result<()> {
    let monitor = RefCell::new(SystemMonitor::new());
    monitor.borrow_mut().generate_processes();
    monitor.borrow_mut().refresh_stats();

    let monitor_clone = monitor.clone();

    App::new()?
        .title("System Monitor")
        .fps(30)
        .tick_interval(2000)
        .on_tick(move |_ctx, tick| {
            let mut m = monitor_clone.borrow_mut();
            m.refresh_stats();
            if tick % 3 == 0 {
                m.generate_processes();
            }
        })
        .run(|ctx| {
            let mut m = monitor.borrow_mut();
            let (w, h) = ctx.compositor().size();
            let area = Rect::new(0, 0, w, h);
            m.set_area(area);
            let plane = m.render(area);
            ctx.add_plane(plane);
        })
}