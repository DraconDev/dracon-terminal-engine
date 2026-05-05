#![allow(missing_docs)]
//! System Monitor — htop-like dashboard with real system data and sparkline history.
//!
//! Reads actual system metrics from /proc filesystem:
//! - CPU usage from /proc/stat
//! - Memory from /proc/meminfo
//! - Disk I/O from /sys/class/block/*/stat
//! - Network from /proc/net/dev
//! - Process list from /proc/<pid>/stat
//!
//! Controls:
//!   t          — cycle theme (15 themes)
//!   ?          — toggle help
//!   ↑/↓        — navigate process list
//!   q          — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Gauge, StatusBadge};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;
use std::time::Duration;

const THEMES: &[&str] = &[
    "nord", "dracula", "cyberpunk", "gruvbox-dark", "tokyo-night",
    "catppuccin", "solarized-dark", "one-dark", "rose-pine", "kanagawa",
    "everforest", "monokai", "solarized-light", "gruvbox-dark", "light",
];
const HISTORY_SIZE: usize = 60;

// ═══════════════════════════════════════════════════════════════════════════════
// HISTORY TRACKER
// ═══════════════════════════════════════════════════════════════════════════════

struct MetricHistory {
    values: VecDeque<f64>,
}

impl MetricHistory {
    fn new() -> Self {
        Self { values: VecDeque::with_capacity(HISTORY_SIZE) }
    }
    fn push(&mut self, v: f64) {
        if self.values.len() >= HISTORY_SIZE { self.values.pop_front(); }
        self.values.push_back(v);
    }
    fn current(&self) -> f64 { self.values.back().copied().unwrap_or(0.0) }
    fn max(&self) -> f64 { self.values.iter().copied().fold(0.0, f64::max) }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_percent: f32,
    mem_mb: f32,
    state: String,
}

struct SystemData {
    cpu_hist: MetricHistory,
    mem_hist: MetricHistory,
    disk_hist: MetricHistory,
    net_hist: MetricHistory,
    memory_used_mb: f32,
    memory_total_mb: f32,
    uptime_seconds: u64,
    load_avg: (f32, f32, f32),
    processes: Vec<ProcessInfo>,
    last_cpu_total: u64,
    last_cpu_idle: u64,
    last_disk_read: u64,
    last_disk_write: u64,
    last_net_rx: u64,
    last_net_tx: u64,
}

impl SystemData {
    fn new() -> Self {
        Self {
            cpu_hist: MetricHistory::new(),
            mem_hist: MetricHistory::new(),
            disk_hist: MetricHistory::new(),
            net_hist: MetricHistory::new(),
            memory_used_mb: 0.0,
            memory_total_mb: 16384.0,
            uptime_seconds: 0,
            load_avg: (0.0, 0.0, 0.0),
            processes: Vec::new(),
            last_cpu_total: 0,
            last_cpu_idle: 0,
            last_disk_read: 0,
            last_disk_write: 0,
            last_net_rx: 0,
            last_net_tx: 0,
        }
    }

    fn refresh(&mut self) {
        self.read_cpu();
        self.read_memory();
        let (dr, dw) = self.read_disk();
        let (nr, nt) = self.read_network();
        self.disk_hist.push(dr + dw);
        self.net_hist.push(nr + nt);
        self.uptime_seconds = self.read_uptime();
        self.load_avg = self.read_load_avg();
        self.read_processes();
    }

    fn read_cpu(&mut self) {
        let mut pct = 0.0;
        if let Ok(content) = fs::read_to_string("/proc/stat") {
            let parts: Vec<&str> = content.lines().next().unwrap_or_default().split_whitespace().collect();
            if parts.len() >= 5 {
                let user: u64 = parts[1].parse().unwrap_or(0);
                let nice: u64 = parts[2].parse().unwrap_or(0);
                let system: u64 = parts[3].parse().unwrap_or(0);
                let idle: u64 = parts[4].parse().unwrap_or(0);
                let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
                let total = user + nice + system + idle + iowait;
                let dt = total.saturating_sub(self.last_cpu_total);
                let di = (idle + iowait).saturating_sub(self.last_cpu_idle);
                self.last_cpu_total = total;
                self.last_cpu_idle = idle + iowait;
                if dt > 0 { pct = ((dt - di) as f64 / dt as f64) * 100.0; }
            }
        }
        self.cpu_hist.push(pct.clamp(0.0, 100.0));
    }

    fn read_memory(&mut self) {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut available_kb = 0u64;
            for line in content.lines() {
                let p: Vec<&str> = line.split_whitespace().collect();
                if p.len() >= 2 {
                    let v: u64 = p[1].parse().unwrap_or(0);
                    match p[0] {
                        "MemTotal:" => total_kb = v,
                        "MemAvailable:" => available_kb = v,
                        _ => {}
                    }
                }
            }
            if total_kb > 0 {
                self.memory_total_mb = total_kb as f32 / 1024.0;
                let used_kb = total_kb.saturating_sub(available_kb);
                self.memory_used_mb = used_kb as f32 / 1024.0;
                let pct = used_kb as f64 / total_kb as f64 * 100.0;
                self.mem_hist.push(pct);
            }
        }
    }

    fn read_disk(&mut self) -> (f64, f64) {
        let mut read_b = 0u64;
        let mut write_b = 0u64;
        if let Ok(entries) = fs::read_dir("/sys/class/block") {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(s) = fs::read_to_string(entry.path().join("stat")) {
                    let p: Vec<&str> = s.split_whitespace().collect();
                    if p.len() >= 6 {
                        read_b += p[2].parse::<u64>().unwrap_or(0) * 512;
                        write_b += p[6].parse::<u64>().unwrap_or(0) * 512;
                    }
                }
            }
        }
        let dr = if self.last_disk_read > 0 { (read_b.saturating_sub(self.last_disk_read) as f64) / 1048576.0 } else { 0.0 };
        let dw = if self.last_disk_write > 0 { (write_b.saturating_sub(self.last_disk_write) as f64) / 1048576.0 } else { 0.0 };
        self.last_disk_read = read_b;
        self.last_disk_write = write_b;
        (dr, dw)
    }

    fn read_network(&mut self) -> (f64, f64) {
        let mut rx_b = 0u64;
        let mut tx_b = 0u64;
        if let Ok(content) = fs::read_to_string("/proc/net/dev") {
            for line in content.lines().skip(2) {
                let p: Vec<&str> = line.split_whitespace().collect();
                if p.len() >= 10 {
                    rx_b += p[1].parse::<u64>().unwrap_or(0);
                    tx_b += p[9].parse::<u64>().unwrap_or(0);
                }
            }
        }
        let nr = if self.last_net_rx > 0 { (rx_b.saturating_sub(self.last_net_rx) as f64) / 1048576.0 } else { 0.0 };
        let nt = if self.last_net_tx > 0 { (tx_b.saturating_sub(self.last_net_tx) as f64) / 1048576.0 } else { 0.0 };
        self.last_net_rx = rx_b;
        self.last_net_tx = tx_b;
        (nr, nt)
    }

    fn read_uptime(&self) -> u64 {
        fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|c| c.split_whitespace().next()?.parse::<f64>().ok())
            .unwrap_or(0.0) as u64
    }

    fn read_load_avg(&self) -> (f32, f32, f32) {
        fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|c| {
                let p: Vec<&str> = c.split_whitespace().take(3).collect();
                if p.len() >= 3 {
                    Some((p[0].parse().unwrap_or(0.0), p[1].parse().unwrap_or(0.0), p[2].parse().unwrap_or(0.0)))
                } else { None }
            })
            .unwrap_or((0.0, 0.0, 0.0))
    }

    fn read_processes(&mut self) {
        self.processes.clear();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                let pid: u32 = match name_str.parse() { Ok(p) => p, _ => continue };
                if let Ok(content) = fs::read_to_string(format!("/proc/{}/stat", pid)) {
                    let paren = content.find('(').unwrap_or(0);
                    let end_paren = content.find(')').unwrap_or(content.len());
                    if end_paren > paren && paren > 0 {
                        let pname = content[paren + 1..end_paren].to_string();
                        let rest: Vec<&str> = content[end_paren + 1..].split_whitespace().collect();
                        if rest.len() >= 12 {
                            let utime: u64 = rest.get(10).and_then(|s| s.parse().ok()).unwrap_or(0);
                            let stime: u64 = rest.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
                            let state = rest.first().copied().unwrap_or("?").to_string();
                            let mem_mb: f32 = fs::read_to_string(format!("/proc/{}/status", pid))
                                .ok()
                                .and_then(|c| c.lines().find(|l| l.starts_with("VmRSS:"))
                                    .and_then(|l| l.split_whitespace().nth(1))
                                    .and_then(|s| s.parse().ok()))
                                .unwrap_or(0.0) / 1024.0;
                            self.processes.push(ProcessInfo {
                                pid, name: pname,
                                cpu_percent: ((utime + stime) as f32 / 100.0).clamp(0.0, 100.0),
                                mem_mb, state,
                            });
                        }
                    }
                }
            }
        }
        self.processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
        self.processes.truncate(20);
    }
}

fn format_uptime(seconds: u64) -> String {
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if d > 0 { format!("{}d {:02}h {:02}m {:02}s", d, h, m, s) }
    else if h > 0 { format!("{:02}h {:02}m {:02}s", h, m, s) }
    else { format!("{:02}m {:02}s", m, s) }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYSTEM MONITOR WIDGET
// ═══════════════════════════════════════════════════════════════════════════════

struct SystemMonitor {
    data: SystemData,
    cpu_gauge: Gauge,
    mem_gauge: Gauge,
    disk_gauge: Gauge,
    net_gauge: Gauge,
    status_badge: StatusBadge,
    theme_index: usize,
    theme: Theme,
    selected_process: Option<usize>,
    process_scroll: usize,
    show_help: bool,
    area: Rect,
}

impl SystemMonitor {
    fn new() -> Self {
        let mut data = SystemData::new();
        data.refresh();
        let theme = Theme::nord();
        Self {
            cpu_gauge: Gauge::with_id(WidgetId::new(1), "CPU %").with_theme(theme).warn_threshold(70.0).crit_threshold(90.0),
            mem_gauge: Gauge::with_id(WidgetId::new(2), "Memory %").with_theme(theme).warn_threshold(80.0).crit_threshold(95.0),
            disk_gauge: Gauge::with_id(WidgetId::new(3), "I/O").with_theme(theme).warn_threshold(75.0).crit_threshold(90.0),
            net_gauge: Gauge::with_id(WidgetId::new(4), "Network").with_theme(theme).warn_threshold(80.0).crit_threshold(95.0),
            status_badge: StatusBadge::new(WidgetId::new(5)).with_theme(theme),
            data,
            theme_index: 0,
            theme,
            selected_process: None,
            process_scroll: 0,
            show_help: false,
            area: Rect::new(0, 0, 80, 24),
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
        self.theme = match THEMES[self.theme_index] {
            "nord" => Theme::nord(), "dracula" => Theme::dracula(),
            "cyberpunk" => Theme::cyberpunk(), "gruvbox-dark" => Theme::gruvbox_dark(),
            "tokyo-night" => Theme::tokyo_night(), "catppuccin" => Theme::catppuccin_mocha(),
            "solarized-dark" => Theme::solarized_dark(), "one-dark" => Theme::one_dark(),
            "rose-pine" => Theme::rose_pine(), "kanagawa" => Theme::kanagawa(),
            "everforest" => Theme::everforest(), "monokai" => Theme::monokai(),
            "solarized-light" => Theme::solarized_light(), "light" => Theme::light(),
            _ => Theme::nord(),
        };
        self.cpu_gauge.on_theme_change(&self.theme);
        self.mem_gauge.on_theme_change(&self.theme);
        self.disk_gauge.on_theme_change(&self.theme);
        self.net_gauge.on_theme_change(&self.theme);
        self.status_badge.on_theme_change(&self.theme);
    }

    fn update_gauges(&mut self) {
        let cpu = self.data.cpu_hist.current();
        let mem = self.data.mem_hist.current();
        self.cpu_gauge.set_value(cpu);
        self.mem_gauge.set_value(mem);

        let cpu_status = if cpu >= 80.0 { "HIGH CPU" } else if cpu >= 50.0 { "MODERATE" } else { "Normal" };
        let mem_pct = mem;
        let mem_status = if mem_pct >= 90.0 { "HIGH MEM" } else { "Normal" };
        let status = if cpu_status == "HIGH CPU" || mem_status == "HIGH MEM" { "WARNING" } else if cpu_status == "MODERATE" { "CAUTION" } else { "HEALTHY" };
        self.status_badge.set_status(status);
    }
}

impl Widget for SystemMonitor {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg; cell.fg = t.fg; cell.transparent = false;
        }

        // ── Header ──
        let header = " System Monitor ";
        let theme_label = format!(" {} ", THEMES[self.theme_index]);
        let uptime = format!(" Up {}", format_uptime(self.data.uptime_seconds));
        draw_text(&mut plane, 2, 0, header, t.primary, t.bg, true);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + uptime.len() as u16 + 3), 0,
            &uptime, t.fg_muted, t.bg, false);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 1), 0,
            &theme_label, t.secondary, t.bg, false);

        // Separator
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = '─'; plane.cells[idx].fg = t.outline; }
        }

        // Gauge labels with icons
        draw_text(&mut plane, 2, 2, " 󰍛 CPU", t.primary, t.surface, true);
        draw_text(&mut plane, half_w + 2, 2, " 󰘚 Memory", t.primary, t.surface, true);
        draw_text(&mut plane, 2, header_y, header_text, t.fg_muted, t.surface, true);

        let max_visible = (list_h as usize).saturating_sub(3);
        for i in 0..max_visible {
            let proc_idx = self.process_scroll + i;
            let row_y = header_y + 1 + i as u16;
            if row_y >= list_y + list_h - 1 { break; }
            if let Some(proc) = self.data.processes.get(proc_idx) {
                let is_selected = self.selected_process == Some(proc_idx);
                let (fg, bg) = if is_selected { (t.fg_on_accent, t.primary_active) } else { (t.fg, t.surface) };
                let name = if proc.name.len() > 14 { &proc.name[..14] } else { &proc.name };
                let line = format!(" {:>5}  {:<3}  {:<14} {:>5.1}%  {:>5.0}MB", proc.pid, proc.state, name, proc.cpu_percent, proc.mem_mb);
                draw_text(&mut plane, 2, row_y, &line, fg, bg, is_selected);
            }
        }

        // ── Detail Panel (right side, when process selected) ──
        if let Some(sel) = self.selected_process {
            if let Some(proc) = self.data.processes.get(sel) {
                let detail_x = area.width / 2;
                let detail_w = area.width.saturating_sub(detail_x + 2);
                let detail_y = badge_y + 1;
                if detail_w > 10 {
                    let mut dy = detail_y;
                    draw_text(&mut plane, detail_x, dy, &format!(" Process: {}", proc.name), t.primary, t.bg, true);
                    dy += 1;
                    draw_text(&mut plane, detail_x, dy, &format!(" PID: {}", proc.pid), t.fg, t.bg, false);
                    dy += 1;
                    draw_text(&mut plane, detail_x, dy, &format!(" CPU: {:.1}%", proc.cpu_percent), t.fg, t.bg, false);
                    dy += 1;
                    draw_text(&mut plane, detail_x, dy, &format!(" MEM: {:.0} MB", proc.mem_mb), t.fg, t.bg, false);
                    dy += 1;
                    draw_text(&mut plane, detail_x, dy, &format!(" State: {}", proc.state), t.fg_muted, t.bg, false);
                }
            }
        }

        // ── Footer ──
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = '─'; plane.cells[idx].fg = t.outline; }
        }
        let footer = " t:theme  ?:help  ↑↓:nav  q:quit ";
        draw_text(&mut plane, 2, footer_y, footer, t.fg_muted, t.bg, false);

        if self.show_help { render_help(&mut plane, area, t); }
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        if self.show_help { self.show_help = false; return true; }
        match key.code {
            KeyCode::Char('q') => std::process::exit(0),
            KeyCode::Char('t') => { self.cycle_theme(); true }
            KeyCode::Char('?') => { self.show_help = true; true }
            KeyCode::Up => {
                let n = self.selected_process.unwrap_or(0);
                if n > 0 { self.selected_process = Some(n - 1); }
                if self.selected_process.unwrap_or(0) < self.process_scroll { self.process_scroll = self.selected_process.unwrap_or(0); }
                true
            }
            KeyCode::Down => {
                let n = self.selected_process.unwrap_or(0);
                if n + 1 < self.data.processes.len() { self.selected_process = Some(n + 1); }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Process list area
                if col < self.area.width / 2 && row >= 9 && row < self.area.height.saturating_sub(2) {
                    let proc_row = (row - 9) as usize;
                    let idx = self.process_scroll + proc_row;
                    if idx < self.data.processes.len() {
                        self.selected_process = Some(idx);
                        return true;
                    }
                }
                // Click right side clears
                if col >= self.area.width / 2 { self.selected_process = None; return true; }
            }
            MouseEventKind::ScrollDown => {
                let max_scroll = self.data.processes.len().saturating_sub(10);
                if self.process_scroll < max_scroll { self.process_scroll += 1; return true; }
            }
            MouseEventKind::ScrollUp if self.process_scroll > 0 => {
                self.process_scroll -= 1;
                return true;
            }
            _ => {}
        }
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RENDERING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

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

fn render_card_border(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: Theme) {
    if w < 3 || h < 2 { return; }
    let (border, _bg) = (t.outline, t.surface);
    for row in y..y+h {
        for col in x..x+w {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() { continue; }
            plane.cells[idx].char = ' '; plane.cells[idx].fg = t.fg;
        }
    }

    for col in x..x+w {
        let idx = (y * plane.width + col) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = '─'; plane.cells[idx].fg = border; }
        let idx2 = ((y+h-1) * plane.width + col) as usize;
        if idx2 < plane.cells.len() { plane.cells[idx2].char = '─'; plane.cells[idx2].fg = border; }
    }
    for row in y..y+h {
        let idx = (row * plane.width + x) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = '│'; plane.cells[idx].fg = border; }
        let idx2 = (row * plane.width + x+w-1) as usize;
        if idx2 < plane.cells.len() { plane.cells[idx2].char = '│'; plane.cells[idx2].fg = border; }
    }
    let corners = [(y,x,'┌'), (y,x+w-1,'┐'), (y+h-1,x,'└'), (y+h-1,x+w-1,'┘')];
    for (r,c,ch) in corners {
        let idx = (r * plane.width + c) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = border; }
    }
}

struct SparklineConfig {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    color: Color,
    bg: Color,
}

fn render_sparkline(plane: &mut Plane, cfg: SparklineConfig, metric: &MetricHistory) {
    let SparklineConfig { x, y, w, h, color, bg } = cfg;
    if metric.values.is_empty() || w == 0 || h == 0 { return; }
    let max_val = metric.max().max(1.0);
    let values: Vec<f64> = metric.values.iter().copied().collect();
    let start = values.len().saturating_sub(w as usize);
    let to_show = &values[start..];
    for (i, &val) in to_show.iter().enumerate() {
        let bar_h = ((val / max_val) * h as f64).round().clamp(0.0, h as f64) as u16;
        let col = x + i as u16;
        if col >= x + w { break; }
        for row in 0..h {
            let row_y = y + h - 1 - row;
            let idx = (row_y * plane.width + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if row < bar_h { '█' } else { ' ' };
                plane.cells[idx].fg = if row < bar_h { color } else { bg };
                plane.cells[idx].bg = bg;
            }
        }
    }
}

fn blit_to(dest: &mut Plane, src: &Plane, offset_x: u16, offset_y: u16) {
    for (i, cell) in src.cells.iter().enumerate() {
        if cell.char == '\0' || cell.transparent { continue; }
        let w = src.width as usize;
        let row = i / w;
        let col = i % w;
        let dy = offset_y as usize + row;
        let dx = offset_x as usize + col;
        if dy >= dest.height as usize || dx >= dest.width as usize { continue; }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() { dest.cells[idx] = cell.clone(); }
    }
}

fn render_help(plane: &mut Plane, area: Rect, t: Theme) {
    let hw = 46u16.min(area.width.saturating_sub(4));
    let hh = 12u16.min(area.height.saturating_sub(4));
    let hx = (area.width - hw) / 2;
    let hy = (area.height - hh) / 2;
    render_card_border(plane, hx, hy, hw, hh, t);
    for cell in plane.cells.iter_mut() {
        if cell.bg == t.surface { cell.bg = t.surface_elevated; }
    }
    let lines = [
        (" System Monitor Help ", true),
        ("", false),
        ("t          Cycle theme (15 themes)", false),
        ("?          Toggle this help", false),
        ("↑/↓        Navigate process list", false),
        ("Click      Select process", false),
        ("Scroll     Scroll process list", false),
        ("q          Quit", false),
    ];
    for (i, (line, bold)) in lines.iter().enumerate() {
        let y = hy + 1 + i as u16;
        let x = hx + (hw.saturating_sub(line.len() as u16)) / 2;
        draw_text(plane, x, y, line, if *bold { t.primary } else { t.fg }, t.surface_elevated, *bold);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INPUT ROUTER
// ═══════════════════════════════════════════════════════════════════════════════

struct InputRouter {
    monitor: Rc<RefCell<SystemMonitor>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for InputRouter {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { false }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }
    fn render(&self, _area: Rect) -> Plane { Plane::new(0, 0, 0) }
    fn handle_key(&mut self, key: KeyEvent) -> bool { self.monitor.borrow_mut().handle_key(key) }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.monitor.borrow_mut().handle_mouse(kind, col, row)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("System Monitor — Real /proc data | t:theme ?:help q:quit");
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
        m.data.refresh();
        m.update_gauges();
        let (w, h) = ctx.compositor().size();
        let plane = m.render(Rect::new(0, 0, w, h));
        ctx.add_plane(plane);
        ctx.mark_dirty(0, 0, w, h);
    })
    .run(|_| {})
}
