#![allow(missing_docs)]
//! Dashboard Builder — Live system dashboard with sparklines and real-time metrics.
//!
//! A polished system monitoring dashboard featuring:
//! - Real CPU, memory, disk, and network metrics
//! - Sparkline history graphs for each metric
//! - Color-coded status indicators
//! - Draggable panel resize
//! - Theme cycling (20 themes)
//! - Pause/resume updates
//! - Process table with selection
//!
//! Controls:
//!   t     — cycle theme
//!   p     — pause/resume updates
//!   r     — force refresh
//!   ↑/↓   — navigate process list
//!   q     — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::framework::widgets::SplitPane;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::collections::VecDeque;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const HISTORY_SIZE: usize = 60;
const THEMES: &[&str] = &[
    "nord",
    "dracula",
    "cyberpunk",
    "gruvbox-dark",
    "tokyo-night",
    "catppuccin",
    "solarized-dark",
    "one-dark",
    "rose-pine",
    "kanagawa",
    "everforest",
    "monokai",
    "solarized-light",
    "light",
    "dark",
    "warm",
    "cool",
    "forest",
    "sunset",
    "mono",
];

// ═══════════════════════════════════════════════════════════════════════════════
// METRIC HISTORY (Sparkline)
// ═══════════════════════════════════════════════════════════════════════════════

struct MetricHistory {
    values: VecDeque<f64>,
    unit: String,
    warn_threshold: f64,
    crit_threshold: f64,
}

impl MetricHistory {
    fn new(_label: &str, unit: &str, warn: f64, crit: f64) -> Self {
        Self {
            values: VecDeque::with_capacity(HISTORY_SIZE),
            unit: unit.to_string(),
            warn_threshold: warn,
            crit_threshold: crit,
        }
    }

    fn push(&mut self, value: f64) {
        if self.values.len() >= HISTORY_SIZE {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    fn current(&self) -> f64 {
        self.values.back().copied().unwrap_or(0.0)
    }

    fn avg(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<f64>() / self.values.len() as f64
    }

    fn max(&self) -> f64 {
        self.values.iter().copied().fold(0.0, f64::max)
    }

    fn status_color(&self, theme: Theme) -> Color {
        let v = self.current();
        if v >= self.crit_threshold {
            theme.error
        } else if v >= self.warn_threshold {
            theme.warning
        } else {
            theme.success
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYSTEM DATA
// ═══════════════════════════════════════════════════════════════════════════════

struct SystemData {
    cpu: MetricHistory,
    mem: MetricHistory,
    disk_read: MetricHistory,
    disk_write: MetricHistory,
    net_rx: MetricHistory,
    net_tx: MetricHistory,
    last_cpu_total: u64,
    last_cpu_idle: u64,
    last_disk_read: u64,
    last_disk_write: u64,
    last_net_rx: u64,
    last_net_tx: u64,
    mem_total_mb: f64,
    processes: Vec<ProcessInfo>,
    selected_process: usize,
    process_scroll: usize,
}

#[derive(Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_percent: f32,
    mem_mb: f32,
}

impl SystemData {
    fn new() -> Self {
        Self {
            cpu: MetricHistory::new("CPU", "%", 70.0, 90.0),
            mem: MetricHistory::new("Memory", "%", 80.0, 95.0),
            disk_read: MetricHistory::new("Disk Read", "MB/s", 50.0, 100.0),
            disk_write: MetricHistory::new("Disk Write", "MB/s", 50.0, 100.0),
            net_rx: MetricHistory::new("Net RX", "MB/s", 10.0, 50.0),
            net_tx: MetricHistory::new("Net TX", "MB/s", 10.0, 50.0),
            last_cpu_total: 0,
            last_cpu_idle: 0,
            last_disk_read: 0,
            last_disk_write: 0,
            last_net_rx: 0,
            last_net_tx: 0,
            mem_total_mb: 16384.0,
            processes: Vec::new(),
            selected_process: 0,
            process_scroll: 0,
        }
    }

    fn refresh(&mut self) {
        self.read_cpu();
        self.read_memory();
        let (dr, dw) = self.read_disk();
        let (nr, nt) = self.read_network();
        self.disk_read.push(dr);
        self.disk_write.push(dw);
        self.net_rx.push(nr);
        self.net_tx.push(nt);
        self.read_processes();
    }

    fn read_cpu(&mut self) {
        let mut pct = 0.0;
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
                    pct = ((delta_total - delta_idle) as f64 / delta_total as f64) * 100.0;
                }
            }
        }
        self.cpu.push(pct.clamp(0.0, 100.0));
    }

    fn read_memory(&mut self) {
        let mut mem_total_kb: u64 = 0;
        let mut mem_available_kb: u64 = 0;

        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
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
        }

        if mem_total_kb > 0 {
            self.mem_total_mb = mem_total_kb as f64 / 1024.0;
            let available_mb = mem_available_kb as f64 / 1024.0;
            let used_mb = self.mem_total_mb - available_mb;
            let pct = (used_mb / self.mem_total_mb * 100.0).clamp(0.0, 100.0);
            self.mem.push(pct);
        } else {
            self.mem.push(50.0);
        }
    }

    fn read_disk(&mut self) -> (f64, f64) {
        let mut read_bytes: u64 = 0;
        let mut write_bytes: u64 = 0;

        if let Ok(entries) = fs::read_dir("/sys/class/block") {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(stat) = fs::read_to_string(entry.path().join("stat")) {
                    let parts: Vec<&str> = stat.split_whitespace().collect();
                    if parts.len() >= 6 {
                        read_bytes += parts[2].parse::<u64>().unwrap_or(0) * 512;
                        write_bytes += parts[6].parse::<u64>().unwrap_or(0) * 512;
                    }
                }
            }
        }

        let dr = if self.last_disk_read > 0 {
            (read_bytes.saturating_sub(self.last_disk_read) as f64) / 1024.0 / 1024.0
        } else {
            0.0
        };
        let dw = if self.last_disk_write > 0 {
            (write_bytes.saturating_sub(self.last_disk_write) as f64) / 1024.0 / 1024.0
        } else {
            0.0
        };
        self.last_disk_read = read_bytes;
        self.last_disk_write = write_bytes;
        (dr, dw)
    }

    fn read_network(&mut self) -> (f64, f64) {
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

        let nr = if self.last_net_rx > 0 {
            (rx_bytes.saturating_sub(self.last_net_rx) as f64) / 1024.0 / 1024.0
        } else {
            0.0
        };
        let nt = if self.last_net_tx > 0 {
            (tx_bytes.saturating_sub(self.last_net_tx) as f64) / 1024.0 / 1024.0
        } else {
            0.0
        };
        self.last_net_rx = rx_bytes;
        self.last_net_tx = tx_bytes;
        (nr, nt)
    }

    fn read_processes(&mut self) {
        self.processes.clear();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(pid) = name.parse::<u32>() {
                        if let Ok(content) = fs::read_to_string(format!("/proc/{}/stat", pid)) {
                            let paren = content.find('(').unwrap_or(0);
                            let end_paren = content.find(')').unwrap_or(content.len());
                            if end_paren > paren && paren > 0 {
                                let pname = content[paren + 1..end_paren].to_string();
                                let rest: Vec<&str> =
                                    content[end_paren + 1..].split_whitespace().collect();
                                if rest.len() >= 12 {
                                    let utime: u64 =
                                        rest.get(10).and_then(|s| s.parse().ok()).unwrap_or(0);
                                    let stime: u64 =
                                        rest.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
                                    let mem_kb: f32 =
                                        fs::read_to_string(format!("/proc/{}/status", pid))
                                            .ok()
                                            .and_then(|c| {
                                                c.lines()
                                                    .find(|l| l.starts_with("VmRSS:"))
                                                    .and_then(|l| l.split_whitespace().nth(1))
                                                    .and_then(|s| s.parse().ok())
                                            })
                                            .unwrap_or(0.0);
                                    self.processes.push(ProcessInfo {
                                        pid,
                                        name: pname,
                                        cpu_percent: ((utime + stime) as f32 / 100.0)
                                            .clamp(0.0, 100.0),
                                        mem_mb: mem_kb / 1024.0,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        self.processes
            .sort_by(|a, b| {
                b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        self.processes.truncate(20);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DASHBOARD WIDGET
// ═══════════════════════════════════════════════════════════════════════════════

struct Dashboard {
    data: SystemData,
    theme: Theme,
    theme_index: usize,
    paused: bool,
    area: Rect,
    split: SplitPane,
    should_quit: Arc<AtomicBool>,
    show_help: bool,
    last_update: Instant,
    keybindings: KeybindingSet,
}

impl Dashboard {
    fn new(should_quit: Arc<AtomicBool>) -> Self {
        let mut data = SystemData::new();
        data.refresh();
        Self {
            data,
            theme: Theme::nord(),
            theme_index: 0,
            paused: false,
            area: Rect::new(0, 0, 80, 24),
            split: SplitPane::new(Orientation::Vertical).ratio(0.6),
            should_quit,
            show_help: false,
            last_update: Instant::now(),
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
        self.theme = match THEMES[self.theme_index] {
            "nord" => Theme::nord(),
            "dracula" => Theme::dracula(),
            "cyberpunk" => Theme::cyberpunk(),
            "gruvbox-dark" => Theme::gruvbox_dark(),
            "tokyo-night" => Theme::tokyo_night(),
            "catppuccin" => Theme::catppuccin_mocha(),
            "solarized-dark" => Theme::solarized_dark(),
            "one-dark" => Theme::one_dark(),
            "rose-pine" => Theme::rose_pine(),
            "kanagawa" => Theme::kanagawa(),
            "everforest" => Theme::everforest(),
            "monokai" => Theme::monokai(),
            "solarized-light" => Theme::solarized_light(),
            "light" => Theme::light(),
            "dark" => Theme::dark(),
            "warm" => Theme::warm(),
            "cool" => Theme::cool(),
            "forest" => Theme::forest(),
            "sunset" => Theme::sunset(),
            "mono" => Theme::mono(),
            _ => Theme::nord(),
        };
        self.split.on_theme_change(&self.theme);
    }
}

impl Widget for Dashboard {
    fn id(&self) -> WidgetId {
        WidgetId::new(1)
    }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let header_h = 1u16;
        let footer_h = 1u16;
        let content_h = area.height.saturating_sub(header_h + footer_h);

        // Header
        let title = " System Dashboard ";
        let theme_label = format!(" {} ", THEMES[self.theme_index]);
        let status = if self.paused {
            " ⏸ PAUSED "
        } else {
            " ● LIVE "
        };
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);
        let sx = area
            .width
            .saturating_sub(status.len() as u16 + theme_label.len() as u16 + 4);
        draw_text(&mut plane, sx, 0, &theme_label, t.secondary, t.bg, false);
        draw_text(
            &mut plane,
            area.width.saturating_sub(status.len() as u16 + 1),
            0,
            status,
            if self.paused { t.warning } else { t.success },
            t.bg,
            true,
        );

        // Content split: left = metrics, right = processes
        let (left_rect, right_rect) = self
            .split
            .split(Rect::new(0, header_h, area.width, content_h));

        // Left panel: metric cards
        self.render_metrics_card(&mut plane, left_rect, t);

        // Right panel: process list
        self.render_process_panel(&mut plane, right_rect, t);

        // Divider
        let div = self
            .split
            .render_divider(Rect::new(0, header_h, area.width, content_h));
        blit_plane(&mut plane, &div, div.x as usize, div.y as usize);

        // Footer
        let footer_y = area.height.saturating_sub(1);
        let footer_text = " t: theme | ?: help | Esc: dismiss | p: pause | r: refresh | ↑↓: nav | q: quit ";
        draw_text(
            &mut plane,
            2,
            footer_y,
            footer_text,
            t.fg_muted,
            t.bg,
            false,
        );

        // Help overlay
        if self.show_help {
            render_help(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.show_help = false;
                return true;
            }
            return true;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('t') => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('p') => {
                self.paused = !self.paused;
                true
            }
            KeyCode::Char('r') => {
                self.data.refresh();
                self.last_update = Instant::now();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                true
            }
            KeyCode::Up => {
                if self.data.selected_process > 0 {
                    self.data.selected_process -= 1;
                }
                if self.data.selected_process < self.data.process_scroll {
                    self.data.process_scroll = self.data.selected_process;
                }
                true
            }
            KeyCode::Down => {
                if self.data.selected_process + 1 < self.data.processes.len() {
                    self.data.selected_process += 1;
                }
                let visible = self.area.height.saturating_sub(4) as usize;
                if self.data.selected_process >= self.data.process_scroll + visible {
                    self.data.process_scroll =
                        self.data.selected_process.saturating_sub(visible - 1);
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let header_h = 1u16;
        let content_h = self.area.height.saturating_sub(2);
        let div_rect = self
            .split
            .divider_rect(Rect::new(0, header_h, self.area.width, content_h));

        match kind {
            MouseEventKind::Down(MouseButton::Left)
                if col >= div_rect.x
                    && col < div_rect.x + div_rect.width
                    && row >= div_rect.y
                    && row < div_rect.y + div_rect.height =>
            {
                return true;
            }
            MouseEventKind::Drag(_)
                if self.split.handle_resize(
                    kind,
                    col,
                    row,
                    Rect::new(0, header_h, self.area.width, content_h),
                ) =>
            {
                return true;
            }
            MouseEventKind::ScrollDown if col > self.area.width / 2 => {
                let visible = content_h.saturating_sub(2) as usize;
                let max_scroll = self.data.processes.len().saturating_sub(visible);
                self.data.process_scroll = (self.data.process_scroll + 1).min(max_scroll);
                return true;
            }
            MouseEventKind::ScrollUp if col > self.area.width / 2 => {
                self.data.process_scroll = self.data.process_scroll.saturating_sub(1);
                return true;
            }
            _ => {}
        }
        false
    }
}

impl Dashboard {
    fn render_metrics_card(&self, plane: &mut Plane, area: Rect, t: Theme) {
        // 2x2 grid of metric cards
        let card_w = area.width / 2;
        let card_h = area.height / 3;

        let cards = [
            (&self.data.cpu, " CPU ", "󰍛"),
            (&self.data.mem, " Memory ", "󰘚"),
            (&self.data.disk_read, " Disk ", "󰋊"),
            (&self.data.net_rx, " Network ", "󰀂"),
        ];

        for (i, (metric, label, icon)) in cards.iter().enumerate() {
            let cx = i as u16 % 2;
            let cy = i as u16 / 2;
            let x = area.x + cx * card_w;
            let y = area.y + cy * card_h;
            let w = if cx == 0 { card_w } else { area.width - card_w };
            let h = if cy == 0 {
                card_h
            } else {
                area.height - card_h * 2
            };

            render_card_border(plane, x, y, w, h, t.outline, t.surface);
            let status_color = metric.status_color(t);

            // Title with icon
            let title = format!("{} {}", icon, label);
            draw_text(plane, x + 2, y + 1, &title, t.primary, t.surface, true);

            // Current value
            let val_text = format!("{:.1}{}", metric.current(), metric.unit);
            draw_text(
                plane,
                x + 2,
                y + 2,
                &val_text,
                status_color,
                t.surface,
                true,
            );

            // Sparkline
            let spark_w = (w as usize).saturating_sub(4) as u16;
            let spark_y = y + h.saturating_sub(3);
            render_sparkline(
                plane,
                SparklineConfig {
                    x: x + 2,
                    y: spark_y,
                    w: spark_w,
                    h: 2,
                    color: status_color,
                    bg: t.surface,
                },
                metric,
            );

            // Mini stats
            let stats = format!("avg {:.1} | max {:.1}", metric.avg(), metric.max());
            draw_text(plane, x + 2, y + 3, &stats, t.fg_muted, t.surface, false);
        }
    }

    fn render_process_panel(&self, plane: &mut Plane, area: Rect, t: Theme) {
        render_card_border(
            plane,
            area.x,
            area.y,
            area.width,
            area.height,
            t.outline,
            t.surface,
        );
        draw_text(
            plane,
            area.x + 2,
            area.y + 1,
            " 󰀽 Processes ",
            t.primary,
            t.surface,
            true,
        );

        let header_y = area.y + 2;
        let header = " PID    NAME             CPU%   MEM ";
        draw_text(
            plane,
            area.x + 2,
            header_y,
            header,
            t.fg_muted,
            t.surface,
            true,
        );

        let list_y = header_y + 1;
        let visible = (area.height.saturating_sub(list_y - area.y + 1)) as usize;

        for i in 0..visible {
            let proc_idx = self.data.process_scroll + i;
            let row_y = list_y + i as u16;
            if row_y >= area.y + area.height - 1 {
                break;
            }

            if let Some(proc) = self.data.processes.get(proc_idx) {
                let is_selected = self.data.selected_process == proc_idx;
                let (fg, bg) = if is_selected {
                    (t.selection_fg, t.selection_bg)
                } else {
                    (t.fg, t.surface)
                };

                let name = if proc.name.len() > 14 {
                    &proc.name[..14]
                } else {
                    &proc.name
                };
                let line = format!(
                    " {:>5} {:<14} {:>5.1}% {:>5.1}",
                    proc.pid, name, proc.cpu_percent, proc.mem_mb
                );
                draw_text(plane, area.x + 2, row_y, &line, fg, bg, is_selected);
            }
        }
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
                char: ch,
                fg,
                bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };
        }
    }
}

fn render_card_border(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, border: Color, bg: Color) {
    if w < 3 || h < 3 {
        return;
    }
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() {
                continue;
            }
            plane.cells[idx].bg = bg;
            let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
            if is_border {
                plane.cells[idx].fg = border;
                plane.cells[idx].char = if row == y && col == x {
                    '╭'
                } else if row == y && col == x + w - 1 {
                    '╮'
                } else if row == y + h - 1 && col == x {
                    '╰'
                } else if row == y + h - 1 && col == x + w - 1 {
                    '╯'
                } else if row == y || row == y + h - 1 {
                    '─'
                } else {
                    '│'
                };
            } else {
                plane.cells[idx].char = ' ';
            }
        }
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
    let SparklineConfig {
        x,
        y,
        w,
        h,
        color,
        bg,
    } = cfg;
    if metric.values.is_empty() || w == 0 || h == 0 {
        return;
    }
    let max_val = metric.max().max(1.0);
    let values: Vec<f64> = metric.values.iter().copied().collect();
    let start = values.len().saturating_sub(w as usize);
    let to_show = &values[start..];

    for (i, &val) in to_show.iter().enumerate() {
        let bar_h = ((val / max_val) * h as f64).round() as u16;
        let col = x + i as u16;
        if col >= x + w {
            break;
        }

        for row in 0..h {
            let row_y = y + h - 1 - row;
            let idx = (row_y * plane.width + col) as usize;
            if idx < plane.cells.len() {
                let is_bar = row < bar_h;
                plane.cells[idx].char = if is_bar { '█' } else { ' ' };
                plane.cells[idx].fg = if is_bar { color } else { bg };
                plane.cells[idx].bg = bg;
            }
        }
    }
}

fn blit_plane(dest: &mut Plane, src: &Plane, offset_x: usize, offset_y: usize) {
    for (i, cell) in src.cells.iter().enumerate() {
        if cell.char == '\0' || cell.transparent {
            continue;
        }
        let src_w = src.width as usize;
        let row = i / src_w;
        let col = i % src_w;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize {
            continue;
        }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = cell.clone();
        }
    }
}

fn render_help(plane: &mut Plane, area: Rect, t: Theme) {
    let hw = 50u16.min(area.width.saturating_sub(4));
    let hh = 12u16.min(area.height.saturating_sub(4));
    let hx = (area.width - hw) / 2;
    let hy = (area.height - hh) / 2;

    render_card_border(plane, hx, hy, hw, hh, t.outline, t.surface_elevated);
    let lines = [
        (" Dashboard Controls ", true),
        ("", false),
        ("t          Cycle theme", false),
        ("p          Pause/resume updates", false),
        ("r          Force refresh", false),
        ("↑/↓        Navigate process list", false),
        ("?          Toggle this help", false),
        ("Esc        Dismiss help", false),
        ("q          Quit", false),
        ("", false),
        (" Mouse: scroll process list  ", false),
    ];
    for (i, (line, bold)) in lines.iter().enumerate() {
        let y = hy + 1 + i as u16;
        let x = hx + (hw.saturating_sub(line.len() as u16)) / 2;
        draw_text(
            plane,
            x,
            y,
            line,
            if *bold { t.primary } else { t.fg },
            t.surface_elevated,
            *bold,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Dashboard Builder — Live system metrics | t:theme p:pause r:refresh q:quit");
    std::thread::sleep(Duration::from_millis(300));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?
        .title("Dashboard Builder")
        .fps(30)
        .theme(Theme::nord())
        .tick_interval(1000);

    let dashboard = Dashboard::new(should_quit);
    app.add_widget(Box::new(dashboard), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(|ctx| {
        let (w, h) = ctx.compositor().size();
        ctx.mark_dirty(0, 0, w, h);
    })
}
