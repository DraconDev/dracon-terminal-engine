#![allow(missing_docs)]
//! System Monitor — htop-like dashboard with real system data.
//!
//! Reads actual system metrics from /proc filesystem:
//! - CPU usage from /proc/stat
//! - Memory from /proc/meminfo
//! - Disk I/O from /sys/class/block/*/stat
//! - Network from /proc/net/dev
//! - Process list from /proc/<pid>/stat
//!
//! Falls back to simulated data if /proc unavailable (e.g., non-Linux).

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Gauge, KeyValueGrid, StatusBadge, StreamingText,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::rc::Rc;
use std::time::{Duration, Instant};

const THEMES: &[&str] = &["nord", "dracula", "cyberpunk", "gruvbox-dark", "tokyo-night"];

struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_percent: f32,
}

struct SystemStats {
    cpu_percent: f32,
    memory_used_mb: f32,
    memory_total_mb: f32,
    disk_read_mb: f64,
    disk_write_mb: f64,
    network_rx_mb: f64,
    network_tx_mb: f64,
    uptime_seconds: u64,
    load_avg: (f32, f32, f32),
    processes: Vec<ProcessInfo>,
}

impl SystemStats {
    fn new() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_used_mb: 0.0,
            memory_total_mb: 4096.0,
            disk_read_mb: 0.0,
            disk_write_mb: 0.0,
            network_rx_mb: 0.0,
            network_tx_mb: 0.0,
            uptime_seconds: 0,
            load_avg: (0.0, 0.0, 0.0),
            processes: Vec::new(),
        }
    }
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
    start_time: Instant,
    last_cpu_total: u64,
    last_cpu_idle: u64,
    last_disk_read: u64,
    last_disk_write: u64,
    last_net_rx: u64,
    last_net_tx: u64,
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
            disk_gauge: Gauge::with_id(WidgetId::new(3), "Disk I/O")
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
            stats: SystemStats::new(),
            theme_index: 0,
            start_time: Instant::now(),
            last_cpu_total: 0,
            last_cpu_idle: 0,
            last_disk_read: 0,
            last_disk_write: 0,
            last_net_rx: 0,
            last_net_tx: 0,
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
    }

    fn get_theme(&self) -> Theme {
        match THEMES[self.theme_index] {
            "nord" => Theme::nord(),
            "dracula" => Theme::dracula(),
            "cyberpunk" => Theme::cyberpunk(),
            "gruvbox-dark" => Theme::gruvbox_dark(),
            "tokyo-night" => Theme::tokyo_night(),
            _ => Theme::nord(),
        }
    }

    fn read_cpu_stats(&mut self) -> f32 {
        if let Ok(content) = fs::read_to_string("/proc/stat") {
            let line = content.lines().next().unwrap_or_default();
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let user: u64 = parts[1].parse().unwrap_or(0);
                let nice: u64 = parts[2].parse().unwrap_or(0);
                let system: u64 = parts[3].parse().unwrap_or(0);
                let idle: u64 = parts[4].parse().unwrap_or(0);
                let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
                let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
                let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);

                let total = user + nice + system + idle + iowait + irq + softirq;
                let idle_total = idle + iowait;

                let delta_total = total.saturating_sub(self.last_cpu_total);
                let delta_idle = idle_total.saturating_sub(self.last_cpu_idle);

                self.last_cpu_total = total;
                self.last_cpu_idle = idle_total;

                if delta_total > 0 {
                    return ((delta_total - delta_idle) as f32 / delta_total as f32) * 100.0;
                }
            }
        }
        self.simulate_cpu()
    }

    fn simulate_cpu(&mut self) -> f32 {
        let tick = (self.start_time.elapsed().as_secs() as f32 / 2.0).round() as u32;
        let base = 30.0 + (tick % 5) as f32 * 10.0;
        (base + rand_float(5.0, 15.0)).clamp(5.0, 95.0)
    }

    fn read_memory_stats(&mut self) -> (f32, f32) {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            let mut mem_total_kb: u64 = 0;
            let mut mem_available_kb: u64 = 0;

            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let value: u64 = parts[1].parse().unwrap_or(0);
                    match parts[0] {
                        "MemTotal:" => mem_total_kb = value,
                        "MemAvailable:" => mem_available_kb = value,
                        _ => {}
                    }
                }
            }

            if mem_total_kb > 0 {
                let total_mb = mem_total_kb as f32 / 1024.0;
                let available_mb = mem_available_kb as f32 / 1024.0;
                let used_mb = total_mb - available_mb;
                self.stats.memory_total_mb = total_mb;
                return (used_mb, total_mb);
            }
        }
        self.stats.memory_total_mb = 16384.0;
        (8192.0 + rand_float(0.0, 500.0), self.stats.memory_total_mb)
    }

    fn read_disk_stats(&mut self) -> (f64, f64) {
        let mut read_bytes: u64 = 0;
        let mut write_bytes: u64 = 0;

        if let Ok(entries) = fs::read_dir("/sys/class/block") {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Ok(stat) = fs::read_to_string(path.join("stat")) {
                    let parts: Vec<&str> = stat.split_whitespace().collect();
                    if parts.len() >= 6 {
                        read_bytes += parts[2].parse::<u64>().unwrap_or(0);
                        write_bytes += parts[6].parse::<u64>().unwrap_or(0);
                    }
                }
            }
        }

        if read_bytes > 0 || write_bytes > 0 {
            let delta_read = read_bytes.saturating_sub(self.last_disk_read) / 1024 / 1024;
            let delta_write = write_bytes.saturating_sub(self.last_disk_write) / 1024 / 1024;
            self.last_disk_read = read_bytes;
            self.last_disk_write = write_bytes;
            return (delta_read as f64, delta_write as f64);
        }
        (rand_float(1.0, 10.0) as f64, rand_float(0.5, 5.0) as f64)
    }

    fn read_network_stats(&mut self) -> (f64, f64) {
        let mut rx_bytes: u64 = 0;
        let mut tx_bytes: u64 = 0;

        if let Ok(content) = fs::read_to_string("/proc/net/dev") {
            for line in content.lines().skip(2) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 10 {
                    rx_bytes += parts[1].parse::<u64>().unwrap_or(0);
                    tx_bytes += parts[9].parse::<u64>().unwrap_or(0);
                }
            }
        }

        if rx_bytes > 0 || tx_bytes > 0 {
            let delta_rx = (rx_bytes.saturating_sub(self.last_net_rx) as f64) / 1024.0 / 1024.0;
            let delta_tx = (tx_bytes.saturating_sub(self.last_net_tx) as f64) / 1024.0 / 1024.0;
            self.last_net_rx = rx_bytes;
            self.last_net_tx = tx_bytes;
            return (delta_rx, delta_tx);
        }
        (rand_float(5.0, 50.0) as f64, rand_float(1.0, 20.0) as f64)
    }

    fn read_load_avg(&self) -> (f32, f32, f32) {
        if let Ok(content) = fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = content.split_whitespace().take(3).collect();
            if parts.len() >= 3 {
                let a: f32 = parts[0].parse().unwrap_or(0.0);
                let b: f32 = parts[1].parse().unwrap_or(0.0);
                let c: f32 = parts[2].parse().unwrap_or(0.0);
                return (a, b, c);
            }
        }
        (rand_float(0.1, 2.0), rand_float(0.1, 1.5), rand_float(0.1, 1.0))
    }

    fn read_uptime(&self) -> u64 {
        if let Ok(content) = fs::read_to_string("/proc/uptime") {
            if let Some(first) = content.split_whitespace().next() {
                return first.parse::<f64>().unwrap_or(0.0) as u64;
            }
        }
        self.start_time.elapsed().as_secs()
    }

    fn read_top_processes(&mut self) {
        self.stats.processes.clear();

        if let Ok(entries) = fs::read_dir("/proc") {
            let mut pids: Vec<u32> = Vec::new();
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(pid) = name.parse::<u32>() {
                        pids.push(pid);
                    }
                }
            }

            for pid in pids.into_iter().take(20) {
                if let Ok(content) = fs::read_to_string(format!("/proc/{}/stat", pid)) {
                    let paren_pos = content.find('(').unwrap_or(0);
                    let end_paren = content.find(')').unwrap_or(content.len());

                    if end_paren > paren_pos && paren_pos > 0 {
                        let name = content[paren_pos + 1..end_paren].to_string();
                        let rest: Vec<&str> = content[end_paren + 1..].split_whitespace().collect();

                        if rest.len() >= 3 {
                            let utime: u64 = rest.get(10).and_then(|s| s.parse().ok()).unwrap_or(0);
                            let stime: u64 = rest.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
                            let _state = rest.get(0).unwrap_or(&"?");

                            let cpu_usage = (utime + stime) as f32 / 100.0;

                            let proc_stat = format!("/proc/{}/status", pid);
                            let _mem_kb: f32 = fs::read_to_string(&proc_stat)
                                .ok()
                                .and_then(|c| {
                                    c.lines()
                                        .find(|l| l.starts_with("VmRSS:"))
                                        .and_then(|l| l.split_whitespace().nth(1))
                                        .and_then(|s| s.parse().ok())
                                })
                                .unwrap_or(0.0)
                                / 1024.0;

                            self.stats.processes.push(ProcessInfo {
                                pid,
                                name,
                                cpu_percent: cpu_usage,
                            });
                        }
                    }
                }
            }
        }

        self.stats.processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
        self.stats.processes.truncate(8);
    }

    fn refresh_stats(&mut self) {
        self.stats.cpu_percent = self.read_cpu_stats();
        let (mem_used, mem_total) = self.read_memory_stats();
        self.stats.memory_used_mb = mem_used;
        self.stats.memory_total_mb = mem_total;
        let (disk_r, disk_w) = self.read_disk_stats();
        self.stats.disk_read_mb = disk_r;
        self.stats.disk_write_mb = disk_w;
        let (net_rx, net_tx) = self.read_network_stats();
        self.stats.network_rx_mb = net_rx;
        self.stats.network_tx_mb = net_tx;
        self.stats.uptime_seconds = self.read_uptime();
        self.stats.load_avg = self.read_load_avg();

        self.read_top_processes();

        let cpu_status = if self.stats.cpu_percent >= 80.0 { "HIGH CPU" } else { "Normal" };
        let mem_pct = (self.stats.memory_used_mb / self.stats.memory_total_mb * 100.0) as u32;
        let mem_status = if mem_pct >= 90 { "HIGH MEM" } else { "Normal" };
        let final_status = if cpu_status == "HIGH CPU" || mem_status == "HIGH MEM" {
            "WARNING"
        } else {
            "HEALTHY"
        };
        self.status_badge.set_status(final_status);

        self.cpu_gauge.set_value(self.stats.cpu_percent as f64);
        self.mem_gauge.set_value((self.stats.memory_used_mb / self.stats.memory_total_mb * 100.0) as f64);

        let disk_activity = (self.stats.disk_read_mb + self.stats.disk_write_mb).min(100.0) as f64;
        self.disk_gauge.set_value(disk_activity);

        let net_activity = (self.stats.network_rx_mb + self.stats.network_tx_mb).min(100.0) as f64;
        self.net_gauge.set_value(net_activity);

        self.uptime_text.clear();
        self.uptime_text.append(&format_uptime(self.stats.uptime_seconds));

        self.process_grid.set_pairs(self.build_process_pairs());
    }

    fn build_process_pairs(&self) -> BTreeMap<String, String> {
        let mut pairs = BTreeMap::new();
        pairs.insert("Top Processes".to_string(), "CPU%".to_string());
        for p in &self.stats.processes {
            let name = if p.name.len() > 15 {
                format!("{}...", &p.name[..12])
            } else {
                p.name.clone()
            };
            pairs.insert(format!("{} ({})", name, p.pid), format!("{:.1}%", p.cpu_percent));
        }
        let (la1, la5, la15) = self.stats.load_avg;
        pairs.insert("Load Avg".to_string(), format!("{:.2} {:.2} {:.2}", la1, la5, la15));
        pairs
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
    let secs = seconds % 60;
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, mins, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

struct InputRouter {
    monitor: Rc<RefCell<SystemMonitor>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for InputRouter {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, _id: WidgetId) { }
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { false }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }
    fn render(&self, _area: Rect) -> Plane { Plane::new(0, 0, 0) }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Char('t') => {
                let mut m = self.monitor.borrow_mut();
                m.cycle_theme();
                let t = m.get_theme();
                m.cpu_gauge.on_theme_change(&t);
                m.mem_gauge.on_theme_change(&t);
                m.disk_gauge.on_theme_change(&t);
                m.net_gauge.on_theme_change(&t);
                m.process_grid.on_theme_change(&t);
                m.status_badge.on_theme_change(&t);
                m.uptime_text.on_theme_change(&t);
                true
            }
            KeyCode::Char('q') => { std::process::exit(0); }
            _ => false,
        }
    }
}

fn main() -> std::io::Result<()> {
    println!("System Monitor — Real system metrics from /proc | t: cycle theme | q: quit");
    std::thread::sleep(Duration::from_millis(300));

    let monitor = Rc::new(RefCell::new(SystemMonitor::new()));
    let mon_for_tick = Rc::clone(&monitor);
    let mon_for_input = Rc::clone(&monitor);

    let mut app = App::new()?
        .title("System Monitor")
        .fps(30)
        .tick_interval(2000);

    let router = InputRouter {
        monitor: mon_for_input,
        id: WidgetId::new(100),
        area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
    };
    app.add_widget(Box::new(router), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _| {
        let mut m = mon_for_tick.borrow_mut();
        m.refresh_stats();

        let (w, h) = ctx.compositor().size();

        let gauges_h = 6u16;

        let cpu_plane = m.cpu_gauge.render(Rect::new(0, 0, w / 4, gauges_h));
        let mem_plane = m.mem_gauge.render(Rect::new(w / 4, 0, w / 4, gauges_h));
        let disk_plane = m.disk_gauge.render(Rect::new(w / 2, 0, w / 4, gauges_h));
        let net_plane = m.net_gauge.render(Rect::new(3 * w / 4, 0, w / 4, gauges_h));

        ctx.add_plane(cpu_plane);
        ctx.add_plane(mem_plane);
        ctx.add_plane(disk_plane);
        ctx.add_plane(net_plane);

        let process_rect = Rect::new(0, gauges_h, w / 2, h.saturating_sub(gauges_h + 2));
        let process_plane = m.process_grid.render(process_rect);
        ctx.add_plane(process_plane);

        let status_rect = Rect::new(w / 2, gauges_h, w / 2, 4);
        let status_plane = m.status_badge.render(status_rect);
        ctx.add_plane(status_plane);

        let uptime_rect = Rect::new(w / 2, gauges_h + 4, w / 2, h.saturating_sub(gauges_h + 6));
        let uptime_plane = m.uptime_text.render(uptime_rect);
        ctx.add_plane(uptime_plane);
    }).run(|_| {})
}