//! Embedded Table List scene for the showcase.
//!
//! Demonstrates the Table and List framework widgets with sortable columns,
//! filterable category list, and a process detail panel.

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::{Column, List, StatusBar, StatusSegment, Table};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

const SIDEBAR_W: u16 = 20;
const DIV_X: u16 = SIDEBAR_W + 2;

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
            Self {
                pid: 1,
                name: "systemd".into(),
                cpu: 0.1,
                memory: 12.4,
                status: "running".into(),
                category: "system".into(),
            },
            Self {
                pid: 245,
                name: "sshd".into(),
                cpu: 0.0,
                memory: 4.2,
                status: "running".into(),
                category: "network".into(),
            },
            Self {
                pid: 312,
                name: "bash".into(),
                cpu: 0.2,
                memory: 8.1,
                status: "running".into(),
                category: "shell".into(),
            },
            Self {
                pid: 415,
                name: "vim".into(),
                cpu: 1.2,
                memory: 24.6,
                status: "running".into(),
                category: "editor".into(),
            },
            Self {
                pid: 502,
                name: "cargo".into(),
                cpu: 45.3,
                memory: 512.0,
                status: "running".into(),
                category: "build".into(),
            },
            Self {
                pid: 603,
                name: "rust-analyzer".into(),
                cpu: 12.8,
                memory: 890.4,
                status: "running".into(),
                category: "build".into(),
            },
            Self {
                pid: 710,
                name: "firefox".into(),
                cpu: 8.5,
                memory: 1024.0,
                status: "running".into(),
                category: "browser".into(),
            },
            Self {
                pid: 822,
                name: "alacritty".into(),
                cpu: 2.1,
                memory: 64.8,
                status: "running".into(),
                category: "shell".into(),
            },
            Self {
                pid: 934,
                name: "nginx".into(),
                cpu: 0.5,
                memory: 18.3,
                status: "running".into(),
                category: "network".into(),
            },
            Self {
                pid: 1045,
                name: "postgres".into(),
                cpu: 3.4,
                memory: 256.0,
                status: "running".into(),
                category: "database".into(),
            },
            Self {
                pid: 1156,
                name: "redis-server".into(),
                cpu: 0.8,
                memory: 48.2,
                status: "running".into(),
                category: "database".into(),
            },
            Self {
                pid: 1267,
                name: "node".into(),
                cpu: 15.2,
                memory: 320.0,
                status: "running".into(),
                category: "build".into(),
            },
            Self {
                pid: 1378,
                name: "docker".into(),
                cpu: 6.7,
                memory: 768.0,
                status: "running".into(),
                category: "system".into(),
            },
            Self {
                pid: 1489,
                name: "pipewire".into(),
                cpu: 1.0,
                memory: 32.0,
                status: "running".into(),
                category: "system".into(),
            },
            Self {
                pid: 1600,
                name: "waybar".into(),
                cpu: 0.3,
                memory: 28.5,
                status: "running".into(),
                category: "system".into(),
            },
            Self {
                pid: 1711,
                name: "swaybg".into(),
                cpu: 0.0,
                memory: 8.0,
                status: "sleeping".into(),
                category: "system".into(),
            },
            Self {
                pid: 1822,
                name: "mako".into(),
                cpu: 0.0,
                memory: 6.4,
                status: "sleeping".into(),
                category: "system".into(),
            },
            Self {
                pid: 1933,
                name: "clangd".into(),
                cpu: 8.9,
                memory: 445.0,
                status: "running".into(),
                category: "build".into(),
            },
        ]
    }
}

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
    status_bar: RefCell<StatusBar>,
    dirty: bool,
}

impl TableListScene {
    pub fn new(theme: Theme) -> Self {
        let processes = Process::all_processes();
        let filtered: Vec<usize> = (0..processes.len()).collect();

        let mut cat_set: Vec<String> = processes.iter().map(|p| p.category.clone()).collect();
        cat_set.sort();
        cat_set.dedup();
        cat_set.insert(0, "all".to_string());

        let t = &theme;
        let table = RefCell::new(
            Table::new_with_id(
                WidgetId::new(300),
                vec![
                    Column {
                        header: "PID".into(),
                        width: 7,
                    },
                    Column {
                        header: "Name".into(),
                        width: 16,
                    },
                    Column {
                        header: "CPU%".into(),
                        width: 7,
                    },
                    Column {
                        header: "Mem MB".into(),
                        width: 9,
                    },
                    Column {
                        header: "Status".into(),
                        width: 9,
                    },
                ],
            )
            .with_theme(t.clone())
            .with_rows(filtered.clone())
            .with_cell_text_fn(|_idx: &usize, _col: usize| -> String { String::new() }),
        );

        let category_list = RefCell::new(
            List::new(cat_set.clone())
                .with_theme(t.clone())
                .with_width(SIDEBAR_W.saturating_sub(2)),
        );

        let status_bar = RefCell::new(
            StatusBar::new(WidgetId::new(62))
                .add_segment(StatusSegment::new(
                    "Up/Dn: nav | Enter: sort | 1-5: sort col | Tab: filter | F1: help | Esc: back",
                ))
                .with_theme(t.clone()),
        );

        let mut scene = Self {
            theme: theme.clone(),
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
        let procs = self.processes.clone();
        let filt = self.filtered.clone();
        let sort_col = self.sort_column;
        let sort_asc = self.sort_ascending;

        let cell_fn = move |idx: &usize, col: usize| -> String {
            if *idx >= filt.len() {
                return String::new();
            }
            let pidx = filt[*idx];
            if pidx >= procs.len() {
                return String::new();
            }
            let p = &procs[pidx];
            match col {
                0 => format!("{}", p.pid),
                1 => p.name.clone(),
                2 => format!("{:.1}", p.cpu),
                3 => format!("{:.1}", p.memory),
                4 => p.status.clone(),
                _ => String::new(),
            }
        };

        let mut table = Table::new_with_id(
            WidgetId::new(300),
            vec![
                Column {
                    header: "PID".into(),
                    width: 7,
                },
                Column {
                    header: "Name".into(),
                    width: 16,
                },
                Column {
                    header: "CPU%".into(),
                    width: 7,
                },
                Column {
                    header: "Mem MB".into(),
                    width: 9,
                },
                Column {
                    header: "Status".into(),
                    width: 9,
                },
            ],
        )
        .with_theme(self.theme.clone())
        .with_rows((0..self.filtered.len()).collect::<Vec<usize>>())
        .with_cell_text_fn(cell_fn);

        if let Some(col) = sort_col {
            table.set_sort(Some(col), sort_asc);
        }

        *self.table.borrow_mut() = table;
        self.dirty = true;
    }

    fn apply_filter(&mut self) {
        self.filtered = if self.selected_category.as_deref() == Some("all") {
            (0..self.processes.len()).collect()
        } else {
            self.processes
                .iter()
                .enumerate()
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
            if self.sort_ascending {
                va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
            }
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
        }
    }

    fn total_cpu(&self) -> f32 {
        self.processes.iter().map(|p| p.cpu).sum()
    }

    fn total_memory(&self) -> f64 {
        self.processes.iter().map(|p| p.memory).sum()
    }

    fn filtered_cpu(&self) -> f32 {
        self.filtered
            .iter()
            .filter_map(|&i| self.processes.get(i))
            .map(|p| p.cpu)
            .sum()
    }

    fn filtered_memory(&self) -> f64 {
        self.filtered
            .iter()
            .filter_map(|&i| self.processes.get(i))
            .map(|p| p.memory)
            .sum()
    }
}

impl Scene for TableListScene {
    fn on_enter(&mut self) {
        self.show_help = false;
        self.dirty = true;
    }

    fn on_exit(&mut self) {
        self.show_help = false;
    }

    fn scene_id(&self) -> &str {
        "table_list"
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Header
        draw_text(&mut plane, 2, 0, " Data Explorer ", t.primary, t.bg, true);
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

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Left sidebar
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

        // Main area - Table + Detail
        let main_x = DIV_X + 2;
        let main_w = area.width.saturating_sub(main_x + 2);

        // Table header row
        let table_y = 2;
        let col_widths = [7u16, 16, 7, 9, 9];
        let mut cx = main_x;
        let headers = ["PID", "Name", "CPU%", "Mem MB", "Status"];
        for (i, (w, h)) in col_widths.iter().zip(headers.iter()).enumerate() {
            let is_sorted = self.sort_column == Some(i);
            let fg = if is_sorted { t.primary } else { t.fg };
            let style = is_sorted;
            draw_text(&mut plane, cx, table_y, h, fg, t.bg, style);
            cx += w;
        }

        // Table content
        let table_h = area.height.saturating_sub(8);
        let table_area = Rect::new(main_x, table_y + 1, main_w, table_h);
        self.table.borrow_mut().set_area(table_area);
        let table_plane = self.table.borrow().render(Rect::new(0, 0, main_w, table_h));
        blit_to(
            &mut plane,
            &table_plane,
            main_x as usize,
            (table_y + 1) as usize,
        );

        // Detail panel
        let detail_y = area.height.saturating_sub(5);
        if let Some(idx) = self.selected_process {
            if idx < self.filtered.len() {
                let pidx = self.filtered[idx];
                if pidx < self.processes.len() {
                    let p = &self.processes[pidx];
                    let detail_text = format!("Selected: {} (PID {})", p.name, p.pid);
                    draw_text(
                        &mut plane,
                        main_x,
                        detail_y,
                        &detail_text,
                        t.primary,
                        t.bg,
                        true,
                    );
                    let stats_text = format!(
                        "CPU: {:.1}%  Memory: {:.1} MB  Status: {}",
                        p.cpu, p.memory, p.status
                    );
                    draw_text_clipped(
                        &mut plane,
                        main_x,
                        detail_y + 1,
                        &stats_text,
                        main_x + main_w,
                        t.fg,
                        t.bg,
                        false,
                    );
                    let cat_text = format!("Category: {}", p.category);
                    draw_text(
                        &mut plane,
                        main_x,
                        detail_y + 2,
                        &cat_text,
                        t.fg_muted,
                        t.bg,
                        false,
                    );
                }
            }
        }

        // Sort indicator
        if let Some(col) = self.sort_column {
            let arrow = if self.sort_ascending { "▲" } else { "▼" };
            let col_names = ["PID", "Name", "CPU%", "Mem", "Status"];
            if col < col_names.len() {
                let label = format!("Sort: {} {}", col_names[col], arrow);
                draw_text(
                    &mut plane,
                    main_x + main_w.saturating_sub(label.len() as u16 + 2),
                    table_y,
                    &label,
                    t.warning,
                    t.bg,
                    false,
                );
            }
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Data Explorer — Help",
                &[
                    ("↑/↓", "Navigate table"),
                    ("Enter", "Toggle sort direction"),
                    ("1-5", "Sort by column"),
                    ("Tab", "Cycle category filter"),
                    ("Click header", "Sort that column"),
                    ("Click row", "Select process"),
                    (help_key, "Toggle this help"),
                    (back_key, "Back"),
                ],
            );
        }

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
            KeyCode::Tab => {
                let cur = self.category_list.borrow().selected_index();
                let next = (cur + 1) % self.categories.len();
                self.select_category(next);
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                if let Some(col) = self.sort_column {
                    self.toggle_sort(col);
                } else {
                    self.toggle_sort(2);
                }
                true
            }
            KeyCode::Up => {
                self.table.borrow_mut().handle_key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: key.modifiers,
                    kind: KeyEventKind::Press,
                });
                self.selected_process = self
                    .table
                    .borrow()
                    .selected_indices()
                    .iter()
                    .next()
                    .copied();
                self.dirty = true;
                true
            }
            KeyCode::Down => {
                self.table.borrow_mut().handle_key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: key.modifiers,
                    kind: KeyEventKind::Press,
                });
                self.selected_process = self
                    .table
                    .borrow()
                    .selected_indices()
                    .iter()
                    .next()
                    .copied();
                self.dirty = true;
                true
            }
            KeyCode::Char('1') => {
                self.toggle_sort(0);
                true
            }
            KeyCode::Char('2') => {
                self.toggle_sort(1);
                true
            }
            KeyCode::Char('3') => {
                self.toggle_sort(2);
                true
            }
            KeyCode::Char('4') => {
                self.toggle_sort(3);
                true
            }
            KeyCode::Char('5') => {
                self.toggle_sort(4);
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let cat_w: u16 = SIDEBAR_W;

        // Category list clicks
        if col < cat_w && row >= 3 {
            let rel_row = (row - 3) as usize;
            if rel_row < self.categories.len() {
                if let MouseEventKind::Down(_) = kind {
                    self.select_category(rel_row);
                    self.dirty = true;
                    return true;
                }
            }
        }

        // Table area clicks
        let main_x = DIV_X + 2;
        if col > cat_w && row >= 3 {
            let rel_col = col.saturating_sub(main_x);
            let rel_row = row.saturating_sub(3);
            if let MouseEventKind::Down(_) = kind {
                // Header click → sort
                if rel_row == 0 {
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
                    self.table.borrow_mut().handle_mouse(kind, rel_col, rel_row);
                    self.selected_process = self
                        .table
                        .borrow()
                        .selected_indices()
                        .iter()
                        .next()
                        .copied();
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
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
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

impl TableListScene {
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let sx = 2u16;

        // Title
        draw_text(plane, sx, 2, "Categories", t.primary, t.bg, true);

        // Category list
        let cat_area = Rect::new(0, 3, SIDEBAR_W, area.height.saturating_sub(8));
        self.category_list.borrow_mut().set_area(cat_area);
        let cat_plane = self.category_list.borrow().render(Rect::new(
            0,
            0,
            SIDEBAR_W,
            area.height.saturating_sub(8),
        ));
        blit_to(plane, &cat_plane, 0, 3);

        // Divider
        let div_y = area.height.saturating_sub(6);
        for dx in 0..SIDEBAR_W {
            let idx = (div_y * plane.width + sx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Stats section
        let stats_y = div_y + 2;
        draw_text(plane, sx, stats_y, "Stats", t.secondary, t.bg, true);

        let total_cpu = self.total_cpu();
        let total_mem = self.total_memory();
        let _ = self.filtered_cpu();
        let _ = self.filtered_memory();
        let cat_count = self.categories.len().saturating_sub(1);
        let proc_count = self.filtered.len();

        let stats = [
            ("Total CPU", format!("{:.1}%", total_cpu)),
            ("Total Mem", format!("{:.0}MB", total_mem)),
            ("Visible", format!("{} proc", proc_count)),
            ("Categories", format!("{}", cat_count)),
        ];

        for (i, (label, value)) in stats.iter().enumerate() {
            let sy = stats_y + 1 + i as u16;
            if sy >= area.height.saturating_sub(4) {
                break;
            }
            draw_text(plane, sx, sy, label, t.fg_muted, t.bg, false);
            draw_text_clipped(plane, sx + 10, sy, value, sx + SIDEBAR_W, t.fg, t.bg, false);
        }

        // Filter indicator
        let filter_y = area.height.saturating_sub(4);
        if filter_y > stats_y + 6 {
            draw_text(plane, sx, filter_y, "Filter", t.secondary, t.bg, true);
            let filter_text = self
                .selected_category
                .clone()
                .unwrap_or_else(|| "all".into());
            let filter_color = if filter_text == "all" {
                t.fg_muted
            } else {
                t.primary
            };
            draw_text_clipped(
                plane,
                sx,
                filter_y + 1,
                &filter_text,
                sx + SIDEBAR_W,
                filter_color,
                t.bg,
                false,
            );
        }
    }
}
