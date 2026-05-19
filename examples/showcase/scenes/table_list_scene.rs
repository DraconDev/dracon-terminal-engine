//! Server Dashboard scene — Table + List + sortable columns demonstration.
//!
//! Shows a process table with sortable columns, a filterable category list,
//! and a detail panel. Demonstrates the Table and List framework widgets
//! working together in a realistic app layout.

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Column, List, StatusBar, StatusSegment, Table,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

// ── Process data ───────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Process {
    pid: u32,
    name: String,
    cpu: f32,
    memory: f64,
    status: String,
    category: String,
}

impl Process {
    fn all_processes() -> Vec<Self> {
        vec![
            Self { pid: 1, name: "systemd".into(), cpu: 0.1, memory: 12.4, status: "running".into(), category: "system".into() },
            Self { pid: 245, name: "sshd".into(), cpu: 0.0, memory: 4.2, status: "running".into(), category: "network".into() },
            Self { pid: 312, name: "bash".into(), cpu: 0.2, memory: 8.1, status: "running".into(), category: "shell".into() },
            Self { pid: 415, name: "vim".into(), cpu: 1.2, memory: 24.6, status: "running".into(), category: "editor".into() },
            Self { pid: 502, name: "cargo".into(), cpu: 45.3, memory: 512.0, status: "running".into(), category: "build".into() },
            Self { pid: 603, name: "rust-analyzer".into(), cpu: 12.8, memory: 890.4, status: "running".into(), category: "build".into() },
            Self { pid: 710, name: "firefox".into(), cpu: 8.5, memory: 1024.0, status: "running".into(), category: "browser".into() },
            Self { pid: 822, name: "alacritty".into(), cpu: 2.1, memory: 64.8, status: "running".into(), category: "shell".into() },
            Self { pid: 934, name: "nginx".into(), cpu: 0.5, memory: 18.3, status: "running".into(), category: "network".into() },
            Self { pid: 1045, name: "postgres".into(), cpu: 3.4, memory: 256.0, status: "running".into(), category: "database".into() },
            Self { pid: 1156, name: "redis-server".into(), cpu: 0.8, memory: 48.2, status: "running".into(), category: "database".into() },
            Self { pid: 1267, name: "node".into(), cpu: 15.2, memory: 320.0, status: "running".into(), category: "build".into() },
            Self { pid: 1378, name: "docker".into(), cpu: 6.7, memory: 768.0, status: "running".into(), category: "system".into() },
            Self { pid: 1489, name: "pipewire".into(), cpu: 1.0, memory: 32.0, status: "running".into(), category: "system".into() },
            Self { pid: 1600, name: "waybar".into(), cpu: 0.3, memory: 28.5, status: "running".into(), category: "system".into() },
            Self { pid: 1711, name: "swaybg".into(), cpu: 0.0, memory: 8.0, status: "sleeping".into(), category: "system".into() },
            Self { pid: 1822, name: "mako".into(), cpu: 0.0, memory: 6.4, status: "sleeping".into(), category: "system".into() },
            Self { pid: 1933, name: "clangd".into(), cpu: 8.9, memory: 445.0, status: "running".into(), category: "build".into() },
        ]
    }
}

// ── Scene ───────────────────────────────────────────────────────────────────

pub struct TableListScene {
    theme: Theme,
    keybindings: KeybindingSet,
    processes: Vec<Process>,
    filtered: Vec<usize>,
    sort_column: Option<usize>,
    sort_ascending: bool,
    table: RefCell<Table<usize>>,
    categories: Vec<String>,
    category_list: RefCell<List<String>>,
    selected_category: Option<String>,
    selected_process: Option<usize>,
    show_help: bool,
    status_bar: StatusBar,
    dirty: bool,
}

impl TableListScene {
    pub fn new(theme: Theme) -> Self {
        let processes = Process::all_processes();
        let filtered: Vec<usize> = (0..processes.len()).collect();

        // Categories
        let mut cat_set: Vec<String> = processes.iter().map(|p| p.category.clone()).collect();
        cat_set.sort();
        cat_set.dedup();
        cat_set.insert(0, "all".to_string());

        let t = &theme;
        let table = RefCell::new(
            Table::new_with_id(WidgetId::new(300), vec![
                Column { header: "PID".into(), width: 7 },
                Column { header: "Name".into(), width: 16 },
                Column { header: "CPU%".into(), width: 7 },
                Column { header: "Mem MB".into(), width: 9 },
                Column { header: "Status".into(), width: 9 },
            ])
            .with_theme(t.clone())
            .with_rows(filtered.clone())
            .with_cell_text_fn(|idx: &usize, col: usize| -> String {
                // Placeholder — will be replaced in rebuild_table
                let _ = (idx, col);
                String::new()
            })
        );

        let category_list = RefCell::new(
            List::new(cat_set.clone())
                .with_theme(t.clone())
                .with_width(14)
        );

        let status_bar = StatusBar::new(WidgetId::new(62))
            .add_segment(StatusSegment::new("Up/Dn: nav | Enter: sort | Tab: categories | F1: help | Esc: back"))
            .with_theme(t.clone());

        let mut scene = Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            processes,
            filtered,
            sort_column: None,
            sort_ascending: true,
            table,
            categories: cat_set,
            category_list,
            selected_category: Some("all".to_string()),
            selected_process: None,
            show_help: false,
            status_bar,
            dirty: true,
        };
        scene.rebuild_table();
        scene
    }

    fn rebuild_table(&mut self) {
        let processes = &self.processes;
        let filtered = &self.filtered;

        let cell_fn = {
            let procs = processes.clone();
            let filt = filtered.clone();
            let _sort_col = self.sort_column;
            let _sort_asc = self.sort_ascending;
            move |idx: &usize, col: usize| -> String {
                if *idx >= filt.len() { return String::new(); }
                let pidx = filt[*idx];
                if pidx >= procs.len() { return String::new(); }
                let p = &procs[pidx];
                match col {
                    0 => format!("{}", p.pid),
                    1 => p.name.clone(),
                    2 => format!("{:.1}", p.cpu),
                    3 => format!("{:.1}", p.memory),
                    4 => p.status.clone(),
                    _ => String::new(),
                }
            }
        };

        let mut table = Table::new_with_id(WidgetId::new(300), vec![
            Column { header: "PID".into(), width: 7 },
            Column { header: "Name".into(), width: 16 },
            Column { header: "CPU%".into(), width: 7 },
            Column { header: "Mem MB".into(), width: 9 },
            Column { header: "Status".into(), width: 9 },
        ])
        .with_theme(self.theme.clone())
        .with_rows((0..self.filtered.len()).collect::<Vec<usize>>())
        .with_cell_text_fn(cell_fn);

        if let Some(col) = self.sort_column {
            table.set_sort(Some(col), self.sort_ascending);
        }

        *self.table.borrow_mut() = table;
        self.dirty = true;
    }

    fn apply_filter(&mut self) {
        self.filtered = if self.selected_category.as_deref() == Some("all") {
            (0..self.processes.len()).collect()
        } else {
            self.processes.iter().enumerate()
                .filter(|(_, p)| Some(&p.category) == self.selected_category.as_ref())
                .map(|(i, _)| i)
                .collect()
        };
        self.apply_sort();
        self.rebuild_table();
    }

    fn apply_sort(&mut self) {
        let sort_key: Box<dyn Fn(&Process) -> f64> = match self.sort_column {
            Some(0) => Box::new(|p: &Process| p.pid as f64),
            Some(1) => Box::new(|p: &Process| p.name.chars().next().unwrap_or('\0') as u8 as f64),
            Some(2) => Box::new(|p: &Process| p.cpu as f64),
            Some(3) => Box::new(|p: &Process| p.memory),
            Some(4) => Box::new(|p: &Process| if p.status == "running" { 0.0 } else { 1.0 }),
            _ => Box::new(|_: &Process| 0.0_f64),
        };

        let procs = &self.processes;
        self.filtered.sort_by(|a, b| {
            let va = sort_key(&procs[*a]);
            let vb = sort_key(&procs[*b]);
            if self.sort_ascending { va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal) }
            else { vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal) }
        });
    }

    fn toggle_sort(&mut self, col: usize) {
        if self.sort_column == Some(col) {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = Some(col);
            self.sort_ascending = true;
        }
        self.apply_sort();
        self.rebuild_table();
    }

    fn select_category(&mut self, idx: usize) {
        if idx < self.categories.len() {
            self.selected_category = Some(self.categories[idx].clone());
            self.apply_filter();
            self.dirty = true;
        }
    }

    fn total_cpu(&self) -> f32 {
        self.processes.iter().map(|p| p.cpu).sum()
    }

    fn total_memory(&self) -> f64 {
        self.processes.iter().map(|p| p.memory).sum()
    }
}

impl Scene for TableListScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // ── Title bar (row 0) ────────────────────────────────────────
        draw_text(&mut plane, 1, 0, "Server Dashboard", t.primary, t.bg, false);
        let stats = format!("CPU: {:.1}%  Mem: {:.0}MB  Procs: {}", self.total_cpu(), self.total_memory(), self.processes.len());
        draw_text(&mut plane, area.width.saturating_sub(stats.len() as u16 + 1), 0, &stats, t.fg_muted, t.bg, false);

        // ── Category list (left 14 columns, rows 1..height-2) ────────
        let cat_w: u16 = 14;
        let cat_h = area.height.saturating_sub(2);
        self.category_list.borrow_mut().set_area(Rect::new(0, 1, cat_w, cat_h));
        let cat_plane = self.category_list.borrow().render(Rect::new(0, 0, cat_w, cat_h));
        blit_to(&mut plane, &cat_plane, 0, 1);

        // ── Divider ──────────────────────────────────────────────────
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y as usize) * area.width as usize + cat_w as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── Process table (right area) ────────────────────────────────
        let table_x = cat_w + 1;
        let table_w = area.width.saturating_sub(cat_w + 1);
        let table_h = area.height.saturating_sub(2);
        self.table.borrow_mut().set_area(Rect::new(table_x, 1, table_w, table_h));
        let table_plane = self.table.borrow().render(Rect::new(0, 0, table_w, table_h));
        blit_to(&mut plane, &table_plane, table_x as usize, 1);

        // ── Detail panel (bottom-right, 3 rows) ─────────────────────
        if let Some(idx) = self.selected_process {
            if idx < self.filtered.len() {
                let pidx = self.filtered[idx];
                if pidx < self.processes.len() {
                    let p = &self.processes[pidx];
                    let dy = area.height.saturating_sub(4);
                    draw_text(&mut plane, table_x + 1, dy, &format!("Selected: {} (PID {})", p.name, p.pid), t.primary, t.bg, false);
                    draw_text(&mut plane, table_x + 1, dy + 1, &format!("CPU: {:.1}%  Memory: {:.1} MB  Status: {}", p.cpu, p.memory, p.status), t.fg, t.bg, false);
                    draw_text(&mut plane, table_x + 1, dy + 2, &format!("Category: {}", p.category), t.fg_muted, t.bg, false);
                }
            }
        }

        // ── Sort indicator ───────────────────────────────────────────
        if let Some(col) = self.sort_column {
            let arrow = if self.sort_ascending { "▲" } else { "▼" };
            let col_names = ["PID", "Name", "CPU%", "Mem", "Status"];
            if col < col_names.len() {
                let label = format!("Sort: {} {}", col_names[col], arrow);
                draw_text(&mut plane, area.width.saturating_sub(label.len() as u16 + 2), 0, &label, t.warning, t.bg, false);
            }
        }

        // ── Status bar ───────────────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self.status_bar.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        // ── Help overlay ─────────────────────────────────────────────
        if self.show_help {
            render_help_overlay(&mut plane, area, t, "Server Dashboard — Help", &[("Up/Dn", "Navigate process table"), ("Enter", "Sort by column / toggle direction"), ("1-5", "Sort by PID/Name/CPU/Mem/Status"), ("Tab", "Cycle category filter"), ("Click header", "Sort that column"), ("Click category", "Filter by category"), ("Click row", "Select process"), ("F1", "Toggle this help"), ("Esc", "Back")]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key) || self.keybindings.matches(actions::BACK, &key) {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Tab => {
                // Toggle focus between category list and table
                // For simplicity, cycle category
                let cur = self.category_list.borrow().selected_index();
                let next = (cur + 1) % self.categories.len();
                self.select_category(next);
                true
            }
            KeyCode::Enter => {
                // Sort by first column or re-sort current
                if let Some(col) = self.sort_column {
                    self.toggle_sort(col);
                } else {
                    self.toggle_sort(2); // Default: sort by CPU
                }
                true
            }
            KeyCode::Up => {
                self.table.borrow_mut().handle_key(KeyEvent {
                    code: KeyCode::Up, modifiers: key.modifiers, kind: KeyEventKind::Press,
                });
                self.selected_process = Some(self.table.borrow().selected_indices().iter().next().copied().unwrap_or(0));
                self.dirty = true;
                true
            }
            KeyCode::Down => {
                self.table.borrow_mut().handle_key(KeyEvent {
                    code: KeyCode::Down, modifiers: key.modifiers, kind: KeyEventKind::Press,
                });
                self.selected_process = Some(self.table.borrow().selected_indices().iter().next().copied().unwrap_or(0));
                self.dirty = true;
                true
            }
            KeyCode::Char('1') => { self.toggle_sort(0); true }
            KeyCode::Char('2') => { self.toggle_sort(1); true }
            KeyCode::Char('3') => { self.toggle_sort(2); true }
            KeyCode::Char('4') => { self.toggle_sort(3); true }
            KeyCode::Char('5') => { self.toggle_sort(4); true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let cat_w: u16 = 14;

        // Category list clicks
        if col < cat_w && row >= 1 {
            let rel_row = (row - 1) as usize;
            if rel_row < self.categories.len() {
                if let MouseEventKind::Down(_) = kind {
                    self.select_category(rel_row);
                    return true;
                }
            }
        }

        // Table area clicks — forward to table
        if col > cat_w && row >= 1 {
            let rel_col = col.saturating_sub(cat_w + 1);
            let rel_row = row.saturating_sub(1);
            if let MouseEventKind::Down(_) = kind {
                // Header click → sort
                if rel_row == 0 {
                    // Approximate column detection
                    let col_widths = [7u16, 16, 7, 9, 9];
                    let mut cum: u16 = 0;
                    for (i, w) in col_widths.iter().enumerate() {
                        cum += w;
                        if rel_col < cum {
                            self.toggle_sort(i);
                            return true;
                        }
                    }
                } else {
                    // Row click → select
                    let table = &mut self.table.borrow_mut();
                    table.handle_mouse(kind, rel_col, rel_row);
                    self.selected_process = table.selected_indices().iter().next().copied();
                    self.dirty = true;
                    return true;
                }
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.table.borrow_mut().on_theme_change(theme);
        self.category_list.borrow_mut().on_theme_change(theme);
        self.status_bar.on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str { "table_list" }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

