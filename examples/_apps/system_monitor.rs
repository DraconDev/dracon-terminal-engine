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
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Gauge, StatusBadge};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

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
const HISTORY_SIZE: usize = 60;

// ═══════════════════════════════════════════════════════════════════════════════
// HISTORY TRACKER
// ═══════════════════════════════════════════════════════════════════════════════

struct MetricHistory {
    values: VecDeque<f64>,
}

impl MetricHistory {
    fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(HISTORY_SIZE),
        }
    }
    fn push(&mut self, v: f64) {
        if self.values.len() >= HISTORY_SIZE {
            self.values.pop_front();
        }
        self.values.push_back(v);
    }
    fn current(&self) -> f64 {
        self.values.back().copied().unwrap_or(0.0)
    }
    fn max(&self) -> f64 {
        self.values.iter().copied().fold(0.0, f64::max)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

struct ProcessInfo {
    pid: u32,
    ppid: Option<u32>,
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
    hostname: String,
    cpu_cores: usize,
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
            hostname: "localhost".to_string(),
            cpu_cores: 1,
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
        self.hostname = self.read_hostname();
        self.cpu_cores = self.read_cpu_cores();
        self.read_processes();
    }

    fn read_cpu(&mut self) {
        let mut pct = 0.0;
        if let Ok(content) = fs::read_to_string("/proc/stat") {
            let parts: Vec<&str> = content
                .lines()
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .collect();
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
                if dt > 0 {
                    pct = ((dt - di) as f64 / dt as f64) * 100.0;
                }
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
        let dr = if self.last_disk_read > 0 {
            (read_b.saturating_sub(self.last_disk_read) as f64) / 1048576.0
        } else {
            0.0
        };
        let dw = if self.last_disk_write > 0 {
            (write_b.saturating_sub(self.last_disk_write) as f64) / 1048576.0
        } else {
            0.0
        };
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
        let nr = if self.last_net_rx > 0 {
            (rx_b.saturating_sub(self.last_net_rx) as f64) / 1048576.0
        } else {
            0.0
        };
        let nt = if self.last_net_tx > 0 {
            (tx_b.saturating_sub(self.last_net_tx) as f64) / 1048576.0
        } else {
            0.0
        };
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
                    Some((
                        p[0].parse().unwrap_or(0.0),
                        p[1].parse().unwrap_or(0.0),
                        p[2].parse().unwrap_or(0.0),
                    ))
                } else {
                    None
                }
            })
            .unwrap_or((0.0, 0.0, 0.0))
    }

    fn read_processes(&mut self) {
        self.processes.clear();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                let pid: u32 = match name_str.parse() {
                    Ok(p) => p,
                    _ => continue,
                };
                if let Ok(content) = fs::read_to_string(format!("/proc/{}/stat", pid)) {
                    let paren = content.find('(').unwrap_or(0);
                    if paren > 0 {
                        let after_paren = &content[paren + 1..];
                        // Find ") " where the char after is a valid process state
                        let mut end_paren = None;
                        for (i, window) in after_paren.as_bytes().windows(3).enumerate() {
                            if window[0] == b')' && window[1] == b' ' {
                                let state_char = window[2] as char;
                                if "RSDZTWtXxPKIW".contains(state_char) {
                                    end_paren = Some(paren + 1 + i);
                                    break;
                                }
                            }
                        }
                        let end_paren = end_paren.unwrap_or(content.len());
                        let pname = content[paren + 1..end_paren].to_string();
                        let rest: Vec<&str> = content[end_paren + 2..].split_whitespace().collect();
                        if rest.len() >= 12 {
                            let ppid: u32 = rest.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                            let utime: u64 = rest.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
                            let stime: u64 = rest.get(12).and_then(|s| s.parse().ok()).unwrap_or(0);
                            let state = rest.first().copied().unwrap_or("?").to_string();
                            let mem_mb: f32 = fs::read_to_string(format!("/proc/{}/status", pid))
                                .ok()
                                .and_then(|c| {
                                    c.lines()
                                        .find(|l| l.starts_with("VmRSS:"))
                                        .and_then(|l| l.split_whitespace().nth(1))
                                        .and_then(|s| s.parse().ok())
                                })
                                .unwrap_or(0.0)
                                / 1024.0;
                            self.processes.push(ProcessInfo {
                                pid,
                                ppid: if ppid > 0 { Some(ppid) } else { None },
                                name: pname,
                                cpu_percent: ((utime + stime) as f32 / 100.0).clamp(0.0, 100.0),
                                mem_mb,
                                state,
                            });
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
    }

    fn read_hostname(&self) -> String {
        fs::read_to_string("/proc/sys/kernel/hostname")
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "localhost".to_string())
    }

    fn read_cpu_cores(&self) -> usize {
        fs::read_to_string("/proc/cpuinfo")
            .ok()
            .map(|c| c.lines().filter(|l| l.starts_with("processor")).count())
            .unwrap_or(1)
    }
}

fn format_uptime(seconds: u64) -> String {
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if d > 0 {
        format!("{}d {:02}h {:02}m {:02}s", d, h, m, s)
    } else if h > 0 {
        format!("{:02}h {:02}m {:02}s", h, m, s)
    } else {
        format!("{:02}m {:02}s", m, s)
    }
}

/// A node in the process tree view.
struct TreeNode {
    proc_idx: usize,
    /// Pre-computed tree connector prefix (e.g. "│  ├─ ", "   └─ ")
    prefix: String,
}

/// Build a flat list of tree nodes for tree view rendering.
fn build_process_tree(processes: &[ProcessInfo]) -> Vec<TreeNode> {
    // Build parent -> children mapping
    let mut children: std::collections::HashMap<u32, Vec<usize>> = std::collections::HashMap::new();
    for (idx, proc) in processes.iter().enumerate() {
        let parent = proc.ppid.unwrap_or(0);
        children.entry(parent).or_default().push(idx);
    }

    // Find root processes (ppid = 0 or 1, or parent not in our list)
    let mut roots: Vec<usize> = Vec::new();
    for (idx, proc) in processes.iter().enumerate() {
        let ppid = proc.ppid.unwrap_or(0);
        if ppid == 0 || ppid == 1 || !processes.iter().any(|p| p.pid == ppid) {
            roots.push(idx);
        }
    }
    // Sort roots by PID for consistency
    roots.sort_by_key(|&idx| processes[idx].pid);

    // Depth-first traversal with proper tree connectors
    let mut result = Vec::new();
    #[allow(clippy::too_many_arguments)]
    fn dfs(
        idx: usize,
        depth: usize,
        is_last: bool,
        ancestor_last: &[bool],
        processes: &[ProcessInfo],
        children: &std::collections::HashMap<u32, Vec<usize>>,
        result: &mut Vec<TreeNode>,
        visited: &mut std::collections::HashSet<u32>,
    ) {
        let pid = processes[idx].pid;
        if !visited.insert(pid) {
            return; // Avoid cycles
        }

        // Build prefix from ancestor last-child status
        let mut prefix = String::new();
        for &last in ancestor_last {
            prefix.push_str(if last { "   " } else { "│  " });
        }
        if depth > 0 {
            prefix.push_str(if is_last { "└─ " } else { "├─ " });
        }

        result.push(TreeNode {
            proc_idx: idx,
            prefix,
        });

        if let Some(child_indices) = children.get(&pid) {
            let mut sorted_children = child_indices.clone();
            sorted_children.sort_by_key(|&ci| processes[ci].pid);
            for (i, &child_idx) in sorted_children.iter().enumerate() {
                let child_is_last = i == sorted_children.len() - 1;
                let mut new_ancestors = ancestor_last.to_vec();
                new_ancestors.push(is_last);
                dfs(
                    child_idx,
                    depth + 1,
                    child_is_last,
                    &new_ancestors,
                    processes,
                    children,
                    result,
                    visited,
                );
            }
        }
    }

    let mut visited = std::collections::HashSet::new();
    for (i, &root) in roots.iter().enumerate() {
        let is_last = i == roots.len() - 1;
        dfs(root, 0, is_last, &[], processes, &children, &mut result, &mut visited);
    }

    // Add any orphaned processes not visited
    for (idx, proc) in processes.iter().enumerate() {
        if !visited.contains(&proc.pid) {
            result.push(TreeNode {
                proc_idx: idx,
                prefix: String::new(),
            });
        }
    }

    result
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
    hovered_process: Option<usize>,
    process_scroll: usize,
    show_help: bool,
    tree_mode: bool,
    keybindings: KeybindingSet,
    area: Rect,
    should_quit: Arc<AtomicBool>,
}

impl SystemMonitor {
    fn new(should_quit: Arc<AtomicBool>) -> Self {
        let mut data = SystemData::new();
        data.refresh();
        let theme = Theme::nord();
        Self {
            cpu_gauge: Gauge::with_id(WidgetId::new(1), "CPU %")
                .with_theme(theme)
                .warn_threshold(70.0)
                .crit_threshold(90.0),
            mem_gauge: Gauge::with_id(WidgetId::new(2), "Memory %")
                .with_theme(theme)
                .warn_threshold(80.0)
                .crit_threshold(95.0),
            disk_gauge: Gauge::with_id(WidgetId::new(3), "I/O")
                .with_theme(theme)
                .warn_threshold(75.0)
                .crit_threshold(90.0),
            net_gauge: Gauge::with_id(WidgetId::new(4), "Network")
                .with_theme(theme)
                .warn_threshold(80.0)
                .crit_threshold(95.0),
            status_badge: StatusBadge::new(WidgetId::new(5)).with_theme(theme),
            data,
            theme_index: 0,
            theme,
            selected_process: None,
            hovered_process: None,
            process_scroll: 0,
            show_help: false,
            tree_mode: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            area: Rect::new(0, 0, 80, 24),
            should_quit,
        }
    }

    /// How many process rows fit in the visible area.
    fn visible_process_rows(&self) -> usize {
        // Layout: header(3) + gauges(4) + sparklines(2) + badge(1) + spacing(3) + footer(1) = 14
        // Process list area starts at y=13, subtract border + header (3 rows) = area.height - 16
        self.area.height.saturating_sub(16).max(1) as usize
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
        self.cpu_gauge.on_theme_change(&self.theme);
        self.mem_gauge.on_theme_change(&self.theme);
        self.disk_gauge.on_theme_change(&self.theme);
        self.net_gauge.on_theme_change(&self.theme);
        self.status_badge.on_theme_change(&self.theme);
    }

    fn update_gauges(&mut self) {
        let cpu = self.data.cpu_hist.current();
        let mem = self.data.mem_hist.current();
        let disk = self.data.disk_hist.current();
        let net = self.data.net_hist.current();
        self.cpu_gauge.set_value(cpu);
        self.mem_gauge.set_value(mem);
        self.disk_gauge.set_value(disk);
        self.net_gauge.set_value(net);

        let cpu_status = if cpu >= 80.0 {
            "HIGH CPU"
        } else if cpu >= 50.0 {
            "MODERATE"
        } else {
            "Normal"
        };
        let mem_status = if mem >= 90.0 { "HIGH MEM" } else { "Normal" };
        let disk_status = if disk >= 75.0 { "HIGH I/O" } else { "Normal" };
        let net_status = if net >= 80.0 { "HIGH NET" } else { "Normal" };
        let status = if cpu_status == "HIGH CPU"
            || mem_status == "HIGH MEM"
            || disk_status == "HIGH I/O"
            || net_status == "HIGH NET"
        {
            "WARNING"
        } else if cpu_status == "MODERATE" {
            "CAUTION"
        } else {
            "HEALTHY"
        };
        self.status_badge.set_status(status);
    }
}

impl Widget for SystemMonitor {
    fn id(&self) -> WidgetId {
        WidgetId::new(0)
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

        let _w = area.width as usize;

        // ── Header Row ──
        let hostname = &self.data.hostname;
        let cores = self.data.cpu_cores;
        let uptime = format_uptime(self.data.uptime_seconds);
        let (l1, l5, l15) = self.data.load_avg;
        let theme_label = format!(" {} ", THEMES[self.theme_index]);

        // Left: hostname + cores
        let left_info = format!(" 󰣇 {} | {} cores | {} ", hostname, cores, uptime);
        draw_text(&mut plane, 2, 0, &left_info, t.primary, t.bg, true);

        // Right: theme label
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 1),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // ── System Info Bar (row 1) ──
        let mem_total = self.data.memory_total_mb;
        let mem_used = self.data.memory_used_mb;
        let mem_pct = if mem_total > 0.0 {
            (mem_used / mem_total * 100.0) as u16
        } else {
            0
        };
        let info_text = format!(" 󰍛 Load: {:.2} {:.2} {:.2} | 󰘚 Memory: {:.0}/{:.0} MB ({}%) | 󰋊 Disk I/O: {:.1} MB/s | 󰀂 Network: {:.1} MB/s ",
            l1, l5, l15, mem_used, mem_total, mem_pct,
            self.data.disk_hist.current(), self.data.net_hist.current());
        let info_x = (area.width.saturating_sub(info_text.len() as u16)) / 2;
        draw_text(&mut plane, info_x, 1, &info_text, t.fg_muted, t.bg, false);

        // Separator
        for x in 0..area.width {
            let idx = (2 * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── 4 Mini Gauges Row ──
        let gauge_y = 3u16;
        let gauge_h = 4u16;
        let qw = area.width / 4;

        // Gauge labels and values
        let gauges = [
            ("󰍛 CPU", self.data.cpu_hist.current(), t),
            ("󰘚 Memory", self.data.mem_hist.current(), t),
            ("󰋊 Disk I/O", self.data.disk_hist.current(), t),
            ("󰀂 Network", self.data.net_hist.current(), t),
        ];

        for (i, (label, val, _)) in gauges.iter().enumerate() {
            let gx = i as u16 * qw;
            let gcolor = if *val >= 90.0 {
                t.error
            } else if *val >= 70.0 {
                t.warning
            } else {
                t.success
            };

            // Card border
            render_card_border(&mut plane, gx, gauge_y, qw, gauge_h, t);

            // Label
            draw_text(
                &mut plane,
                gx + 2,
                gauge_y,
                label,
                t.primary,
                t.surface,
                true,
            );

            // Value
            let val_text = format!("{:.1}%", val);
            draw_text(
                &mut plane,
                gx + 2,
                gauge_y + 1,
                &val_text,
                gcolor,
                t.surface,
                true,
            );

            // Mini bar
            let bar_w = qw.saturating_sub(4);
            let filled = ((*val / 100.0) * bar_w as f64).round() as u16;
            for bx in 0..bar_w {
                let idx = ((gauge_y + 2) * area.width + gx + 2 + bx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = if bx < filled { '█' } else { '░' };
                    plane.cells[idx].fg = if bx < filled { gcolor } else { t.fg_subtle };
                    plane.cells[idx].bg = t.surface;
                }
            }
        }

        // ── Sparkline Charts Row ──
        let spark_y = gauge_y + gauge_h + 1;
        let spark_h = 2u16;
        let spark_w = qw.saturating_sub(4);

        let sparklines = [
            (0u16, &self.data.cpu_hist, t.success),
            (1, &self.data.mem_hist, t.info),
            (2, &self.data.disk_hist, t.warning),
            (3, &self.data.net_hist, t.secondary),
        ];

        for (i, hist, color) in sparklines.iter() {
            let sx = (*i * qw) + 2;
            render_sparkline(
                &mut plane,
                SparklineConfig {
                    x: sx,
                    y: spark_y,
                    w: spark_w,
                    h: spark_h,
                    color: *color,
                    bg: t.bg,
                },
                hist,
            );
        }

        // ── Status Badge ──
        let badge_y = spark_y + spark_h + 1;
        let sb = self.status_badge.render(Rect::new(0, 0, 16, 1));
        blit_to(&mut plane, &sb, 2, badge_y);

        // Process count
        let proc_count = format!("{} processes", self.data.processes.len());
        draw_text(
            &mut plane,
            20,
            badge_y,
            &proc_count,
            t.fg_muted,
            t.bg,
            false,
        );

        // ── Process List ──
        let list_y = badge_y + 2;
        let list_h = area.height.saturating_sub(list_y + 2);
        render_card_border(&mut plane, 0, list_y, area.width, list_h, t);

        let header_y = list_y + 1;
        let header_text =
            " 󰀽 PID      NAME             CPU%    MEM     STATE  ";
        draw_text(
            &mut plane,
            2,
            header_y,
            header_text,
            t.fg_muted,
            t.surface,
            true,
        );

        if self.data.processes.is_empty() {
            let cx = area.width / 2;
            let cy = header_y + list_h / 2 - 2;
            draw_text(
                &mut plane,
                cx.saturating_sub(10),
                cy,
                " 󰓇 Collecting process data... ",
                t.fg_muted,
                t.surface,
                false,
            );
        } else {
            let max_visible = (list_h as usize).saturating_sub(3);
            let tree_view = if self.tree_mode {
                build_process_tree(&self.data.processes)
            } else {
                (0..self.data.processes.len())
                    .map(|i| TreeNode {
                        proc_idx: i,
                        prefix: String::new(),
                    })
                    .collect()
            };
            let total_items = tree_view.len();
            for i in 0..max_visible {
                let view_idx = self.process_scroll + i;
                let row_y = header_y + 1 + i as u16;
                if row_y >= list_y + list_h - 1 {
                    break;
                }
                if let Some(node) = tree_view.get(view_idx) {
                    if let Some(proc) = self.data.processes.get(node.proc_idx) {
                        let is_selected = self.selected_process == Some(view_idx);
                        let is_hovered = self.hovered_process == Some(view_idx);
                        let (fg, bg) = if is_selected {
                            (t.selection_fg, t.selection_bg)
                        } else if is_hovered {
                            (t.fg, t.hover_bg)
                        } else {
                            (t.fg, t.surface)
                        };
                        let name = if proc.name.len() > 16 {
                            &proc.name[..16]
                        } else {
                            &proc.name
                        };
                        let line = format!(
                            " {}{:>6}  {:<16} {:>6.1}%  {:>6.0}MB  {:<6}",
                            node.prefix, proc.pid, name, proc.cpu_percent, proc.mem_mb, proc.state
                        );
                        draw_text(&mut plane, 2, row_y, &line, fg, bg, is_selected);
                    }
                }
            }

            // Scrollbar indicator
            if total_items > max_visible {
                let sb_x = area.width - 2;
                let content_h = max_visible as u16;
                let thumb_h = (max_visible as f32 / total_items as f32
                    * content_h as f32)
                    .max(1.0) as u16;
                let thumb_y = (self.process_scroll as f32
                    / total_items.saturating_sub(max_visible).max(1) as f32
                    * (content_h - thumb_h) as f32) as u16
                    + header_y
                    + 1;
                for i in 0..thumb_h {
                    let y = thumb_y + i;
                    if y > header_y && y < list_y + list_h - 1 {
                        let idx = (y * area.width + sb_x) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = '▐';
                            plane.cells[idx].fg = t.primary;
                            plane.cells[idx].bg = t.surface;
                            plane.cells[idx].transparent = false;
                        }
                    }
                }
            }
        }

        // ── Detail Panel (right side, when process selected) ──
        let tree_view_for_detail = if self.tree_mode {
            build_process_tree(&self.data.processes)
        } else {
            (0..self.data.processes.len())
                .map(|i| TreeNode {
                    proc_idx: i,
                    prefix: String::new(),
                })
                .collect::<Vec<_>>()
        };
        if let Some(sel_view) = self.selected_process {
            if let Some(node) = tree_view_for_detail.get(sel_view) {
                if let Some(proc) = self.data.processes.get(node.proc_idx) {
                let detail_x = area.width / 2;
                let detail_w = area.width.saturating_sub(detail_x + 2);
                let detail_y = list_y + 1;
                if detail_w > 10 && detail_y + 6 < list_y + list_h {
                    let mut dy = detail_y;
                    draw_text(
                        &mut plane,
                        detail_x,
                        dy,
                        &format!(" Process: {}", proc.name),
                        t.primary,
                        t.surface,
                        true,
                    );
                    dy += 1;
                    draw_text(
                        &mut plane,
                        detail_x,
                        dy,
                        &format!(" PID: {}", proc.pid),
                        t.fg,
                        t.surface,
                        false,
                    );
                    dy += 1;
                    let ppid_str = proc.ppid.map(|p| p.to_string()).unwrap_or_else(|| "None".to_string());
                    draw_text(
                        &mut plane,
                        detail_x,
                        dy,
                        &format!(" PPID: {}", ppid_str),
                        t.fg,
                        t.surface,
                        false,
                    );
                    dy += 1;
                    // Mini CPU gauge
                    let cpu_bar =
                        ((proc.cpu_percent / 100.0).min(1.0) * (detail_w - 10) as f32) as u16;
                    draw_text(&mut plane, detail_x, dy, " CPU: ", t.fg, t.surface, false);
                    for bx in 0..(detail_w - 10) {
                        let idx = (dy * area.width + detail_x + 6 + bx) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = if bx < cpu_bar { '█' } else { '░' };
                            plane.cells[idx].fg =
                                if bx < cpu_bar { t.warning } else { t.fg_subtle };
                            plane.cells[idx].bg = t.surface;
                        }
                    }
                    dy += 1;
                    // Mini MEM gauge
                    let mem_bar = ((proc.mem_mb / self.data.memory_total_mb).min(1.0)
                        * (detail_w - 10) as f32) as u16;
                    draw_text(&mut plane, detail_x, dy, " MEM: ", t.fg, t.surface, false);
                    for bx in 0..(detail_w - 10) {
                        let idx = (dy * area.width + detail_x + 6 + bx) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = if bx < mem_bar { '█' } else { '░' };
                            plane.cells[idx].fg = if bx < mem_bar { t.info } else { t.fg_subtle };
                            plane.cells[idx].bg = t.surface;
                        }
                    }
                    dy += 1;
                    draw_text(
                        &mut plane,
                        detail_x,
                        dy,
                        &format!(" State: {}", proc.state),
                        t.fg_muted,
                        t.surface,
                        false,
                    );
                }
            }
        }
        }

        // ── Footer ──
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let footer = format!(
            "{} | ↑/↓: nav | Click: select",
            self.keybindings.format_hint(&[
                (actions::THEME, "theme"),
                (actions::TREE_MODE, "tree"),
                (actions::HELP, "help"),
                (actions::DISMISS, "dismiss"),
                (actions::QUIT, "quit"),
            ])
        );
        draw_text(&mut plane, 2, footer_y, &footer, t.fg_muted, t.bg, false);

        if self.show_help {
            render_help(&mut plane, area, t);
        }
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        let kb = &self.keybindings;
        if self.show_help {
            if kb.matches(actions::DISMISS, &key) || kb.matches(actions::HELP, &key) {
                self.show_help = false;
                return true;
            }
            return true;
        }
        // Compute view size for navigation bounds
        let view_count = if self.tree_mode {
            build_process_tree(&self.data.processes).len()
        } else {
            self.data.processes.len()
        };
        let max_scroll = view_count.saturating_sub(1);

        if kb.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if kb.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if kb.matches(actions::TREE_MODE, &key) {
            self.tree_mode = !self.tree_mode;
            self.process_scroll = 0;
            self.selected_process = None;
            return true;
        }
        if kb.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            return true;
        }

        match key.code {
            KeyCode::Up => {
                let n = self.selected_process.unwrap_or(0);
                if n > 0 {
                    self.selected_process = Some(n - 1);
                }
                if self.selected_process.unwrap_or(0) < self.process_scroll {
                    self.process_scroll = self.selected_process.unwrap_or(0);
                }
                true
            }
            KeyCode::Down => {
                let n = self.selected_process.unwrap_or(0);
                if n < max_scroll {
                    self.selected_process = Some(n + 1);
                }
                // Auto-scroll if selection goes below visible area
                let max_visible = self.visible_process_rows();
                if self.selected_process.unwrap_or(0) >= self.process_scroll + max_visible {
                    self.process_scroll = self.selected_process.unwrap_or(0) + 1 - max_visible;
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let view_count = if self.tree_mode {
            build_process_tree(&self.data.processes).len()
        } else {
            self.data.processes.len()
        };
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if col < self.area.width / 2 && row >= 9 && row < self.area.height.saturating_sub(2)
                {
                    let proc_row = (row - 9) as usize;
                    let idx = self.process_scroll + proc_row;
                    if idx < view_count {
                        self.selected_process = Some(idx);
                        return true;
                    }
                }
                if col >= self.area.width / 2 {
                    self.selected_process = None;
                    return true;
                }
            }
            MouseEventKind::Moved => {
                if col < self.area.width / 2 && row >= 9 && row < self.area.height.saturating_sub(2)
                {
                    let proc_row = (row - 9) as usize;
                    let idx = self.process_scroll + proc_row;
                    self.hovered_process = if idx < view_count {
                        Some(idx)
                    } else {
                        None
                    };
                } else {
                    self.hovered_process = None;
                }
                return true;
            }
            MouseEventKind::ScrollDown => {
                let max_scroll = view_count.saturating_sub(10);
                if self.process_scroll < max_scroll {
                    self.process_scroll += 1;
                    return true;
                }
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

fn render_card_border(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: Theme) {
    if w < 3 || h < 2 {
        return;
    }
    let (border, bg) = (t.outline, t.surface);
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() {
                continue;
            }
            let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
            plane.cells[idx].bg = bg;
            plane.cells[idx].fg = if is_border { border } else { t.fg };
            plane.cells[idx].char = ' ';
        }
    }

    for col in x..x + w {
        let idx = (y * plane.width + col) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = border;
        }
        let idx2 = ((y + h - 1) * plane.width + col) as usize;
        if idx2 < plane.cells.len() {
            plane.cells[idx2].char = '─';
            plane.cells[idx2].fg = border;
        }
    }
    for row in y..y + h {
        let idx = (row * plane.width + x) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '│';
            plane.cells[idx].fg = border;
        }
        let idx2 = (row * plane.width + x + w - 1) as usize;
        if idx2 < plane.cells.len() {
            plane.cells[idx2].char = '│';
            plane.cells[idx2].fg = border;
        }
    }
    let corners = [
        (y, x, '┌'),
        (y, x + w - 1, '┐'),
        (y + h - 1, x, '└'),
        (y + h - 1, x + w - 1, '┘'),
    ];
    for (r, c, ch) in corners {
        let idx = (r * plane.width + c) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = border;
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
        let bar_h = ((val / max_val) * h as f64).round().clamp(0.0, h as f64) as u16;
        let col = x + i as u16;
        if col >= x + w {
            break;
        }
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
        if cell.char == '\0' || cell.transparent {
            continue;
        }
        let w = src.width as usize;
        let row = i / w;
        let col = i % w;
        let dy = offset_y as usize + row;
        let dx = offset_x as usize + col;
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
    let hw = 46u16.min(area.width.saturating_sub(4));
    let hh = 13u16.min(area.height.saturating_sub(4));
    let hx = (area.width - hw) / 2;
    let hy = (area.height - hh) / 2;

    // Fill background with surface_elevated
    for y in hy..hy + hh {
        for x in hx..hx + hw {
            let idx = (y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].fg = t.fg;
                plane.cells[idx].transparent = false;
            }
        }
    }

    // Draw rounded corners
    let corners = [
        (hy, hx, '╭'),
        (hy, hx + hw - 1, '╮'),
        (hy + hh - 1, hx, '╰'),
        (hy + hh - 1, hx + hw - 1, '╯'),
    ];
    for (r, c, ch) in corners {
        let idx = (r * plane.width + c) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = t.outline;
        }
    }
    // Draw borders
    for col in hx..hx + hw {
        let idx = (hy * plane.width + col) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = t.outline;
        }
        let idx2 = ((hy + hh - 1) * plane.width + col) as usize;
        if idx2 < plane.cells.len() {
            plane.cells[idx2].char = '─';
            plane.cells[idx2].fg = t.outline;
        }
    }
    for row in hy..hy + hh {
        let idx = (row * plane.width + hx) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '│';
            plane.cells[idx].fg = t.outline;
        }
        let idx2 = (row * plane.width + hx + hw - 1) as usize;
        if idx2 < plane.cells.len() {
            plane.cells[idx2].char = '│';
            plane.cells[idx2].fg = t.outline;
        }
    }

    let lines = [
        (" System Monitor Help ", true),
        ("", false),
        ("t          Cycle theme (15 themes)", false),
        ("p          Toggle tree view", false),
        ("?          Toggle this help", false),
        ("Esc        Dismiss help", false),
        ("↑/↓        Navigate process list", false),
        ("Click      Select process", false),
        ("Scroll     Scroll process list", false),
        ("q          Quit", false),
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
// INPUT ROUTER
// ═══════════════════════════════════════════════════════════════════════════════

struct InputRouter {
    monitor: Rc<RefCell<SystemMonitor>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for InputRouter {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect {
        self.area.get()
    }
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        false
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, _area: Rect) -> Plane {
        Plane::new(0, 0, 0)
    }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.monitor.borrow_mut().handle_key(key)
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.monitor.borrow_mut().handle_mouse(kind, col, row)
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.monitor.borrow_mut().on_theme_change(theme);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("System Monitor — Real /proc data | t:theme ?:help q:quit");
    std::thread::sleep(Duration::from_millis(300));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let monitor = Rc::new(RefCell::new(SystemMonitor::new(should_quit)));
    let mon_for_tick = Rc::clone(&monitor);
    let mon_for_input = Rc::clone(&monitor);

    let mut app = App::new()?
        .title("System Monitor")
        .fps(30)
        .tick_interval(2000)
        .theme(Theme::from_env_or(Theme::nord()));

    let router = InputRouter {
        monitor: mon_for_input,
        id: WidgetId::new(100),
        area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
    };
    app.add_widget(Box::new(router), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }
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
