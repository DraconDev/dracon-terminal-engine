#![allow(missing_docs)]
//! SQLite Browser — Database browser with table list, query editor, and results.
//!
//! Demonstrates Table widget, SearchInput, SplitPane, and StatusBar.
//!
//! Reads from a real SQLite database file (or creates a mock one).
//!
//! Controls:
//!   ↑/↓ or j/k     — navigate tables/results
//!   Enter          — select table / run query
//!   Tab            — switch panel (tables/query/results)
//!   e              — edit query
//!   r              — refresh
//!   q              — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    SearchInput, SplitPane, StatusBar, StatusSegment, Table, Column, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
struct RowData {
    cells: Vec<String>,
}

impl std::fmt::Display for RowData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cells.first().cloned().unwrap_or_default())
    }
}

enum Panel { Tables, Query, Results }

struct SqliteBrowser {
    should_quit: Arc<AtomicBool>,
    theme: Theme,
    area: Rect,

    db_path: String,
    tables: Vec<String>,
    selected_table: usize,

    query: String,
    editing_query: bool,
    search_input: SearchInput,

    results_columns: Vec<Column>,
    results_rows: Vec<RowData>,
    results_table: Option<Table<RowData>>,

    active_panel: Panel,

    status_bar: StatusBar,
    toasts: Vec<Toast>,
    dirty: bool,
}

impl SqliteBrowser {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme, db_path: &str) -> Self {
        let search_input = SearchInput::new(WidgetId::new(3)).with_theme(theme);

        let status_bar = StatusBar::new(WidgetId::new(4))
            .add_segment(StatusSegment::new("SQLite Browser").with_fg(theme.primary))
            .add_segment(StatusSegment::new("Tab: switch | e: edit | r: refresh | q: quit").with_fg(theme.fg_muted));

        let mut app = Self {
            should_quit,
            theme,
            area: Rect::new(0, 0, 80, 24),
            db_path: db_path.to_string(),
            tables: Vec::new(),
            selected_table: 0,
            query: "SELECT * FROM users LIMIT 10".to_string(),
            editing_query: false,
            search_input,
            results_columns: Vec::new(),
            results_rows: Vec::new(),
            results_table: None,
            active_panel: Panel::Tables,
            status_bar,
            toasts: Vec::new(),
            dirty: true,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.tables = self.read_tables();
        if !self.tables.is_empty() && self.query.is_empty() {
            self.query = format!("SELECT * FROM {} LIMIT 10", self.tables[0]);
        }
        self.run_query(&self.query.clone());
        self.dirty = true;
    }

    fn read_tables(&self) -> Vec<String> {
        let output = Command::new("sqlite3")
            .args([&self.db_path, ".tables"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout);
                text.split_whitespace().map(|s| s.to_string()).collect()
            }
            _ => self.create_mock_db(),
        }
    }

    fn create_mock_db(&self) -> Vec<String> {
        // Create a temporary mock database
        let mock_db = format!("/tmp/dracon_mock_{}.db", std::process::id());
        let _ = Command::new("sqlite3")
            .args([&mock_db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, role TEXT);"])
            .output();

        let inserts = [
            "INSERT INTO users VALUES (1, 'Alice', 'alice@example.com', 'admin');",
            "INSERT INTO users VALUES (2, 'Bob', 'bob@example.com', 'user');",
            "INSERT INTO users VALUES (3, 'Charlie', 'charlie@example.com', 'editor');",
            "INSERT INTO users VALUES (4, 'Diana', 'diana@example.com', 'admin');",
            "INSERT INTO users VALUES (5, 'Eve', 'eve@example.com', 'user');",
        ];
        for insert in &inserts {
            let _ = Command::new("sqlite3").args([&mock_db, insert]).output();
        }

        let _ = Command::new("sqlite3")
            .args([&mock_db, "CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY, title TEXT, author TEXT, views INTEGER);"])
            .output();

        let post_inserts = [
            "INSERT INTO posts VALUES (1, 'Hello World', 'Alice', 150);",
            "INSERT INTO posts VALUES (2, 'Rust Tips', 'Bob', 89);",
            "INSERT INTO posts VALUES (3, 'TUI Design', 'Charlie', 234);",
        ];
        for insert in &post_inserts {
            let _ = Command::new("sqlite3").args([&mock_db, insert]).output();
        }

        self.toast("Using mock database (sqlite3 not available)", ToastKind::Warning);
        vec!["users".to_string(), "posts".to_string()]
    }

    fn run_query(&mut self, query: &str) {
        let output = Command::new("sqlite3")
            .args([&self.db_path, query, "-header", "-csv"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout);
                self.parse_results(&text);
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                self.results_columns = vec![Column { header: "Error".to_string(), width: 50 }];
                self.results_rows = vec![RowData { cells: vec![err.to_string()] }];
                self.toast("Query error", ToastKind::Error);
            }
            Err(_) => {
                self.results_columns = vec![Column { header: "Info".to_string(), width: 50 }];
                self.results_rows = vec![RowData { cells: vec!["SQLite not available. Install sqlite3.".to_string()] }];
            }
        }

        let mut table = Table::new(self.results_columns.clone()).with_theme(self.theme);
        table.rows = self.results_rows.iter().map(|r| dracon_terminal_engine::framework::widgets::TableRow { data: r.clone() }).collect();
        table.set_visible_count(20);
        self.results_table = Some(table);
        self.dirty = true;
    }

    fn parse_results(&mut self, csv: &str) {
        let lines: Vec<&str> = csv.lines().collect();
        if lines.is_empty() {
            self.results_columns = Vec::new();
            self.results_rows = Vec::new();
            return;
        }

        // Parse header
        let header: Vec<String> = lines[0].split(',').map(|s| s.trim().to_string()).collect();
        self.results_columns = header.iter().map(|h| Column { header: h.clone(), width: (h.len() + 4).max(10) as u16 }).collect();

        // Parse rows
        self.results_rows = lines.iter().skip(1).filter(|l| !l.is_empty()).map(|line| {
            let cells: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            RowData { cells }
        }).collect();
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        let toast = Toast::new(WidgetId::new(100 + self.toasts.len()), msg)
            .with_kind(kind)
            .with_duration(Duration::from_secs(2))
            .with_theme(self.theme);
        self.toasts.push(toast);
        self.dirty = true;
    }
}

impl Widget for SqliteBrowser {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; self.dirty = true; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let status_h = 1u16;
        let content_h = area.height.saturating_sub(status_h);

        // Split: left 25% tables, right 75% query+results
        let split = SplitPane::new(Orientation::Horizontal).ratio(0.25);
        let (left_rect, right_rect) = split.split(Rect::new(0, 0, area.width, content_h));

        // Left panel: tables
        let left_active = matches!(self.active_panel, Panel::Tables);
        let left_bg = if left_active { t.surface_elevated } else { t.surface };
        for y in 0..left_rect.height {
            for x in 0..left_rect.width {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = left_bg;
                }
            }
        }

        draw_text(&mut plane, 2, 0, "Tables", t.primary, left_bg, true);
        for (i, table) in self.tables.iter().enumerate() {
            let row = 2 + i as u16;
            let is_selected = self.selected_table == i && left_active;
            let fg = if is_selected { t.fg_on_accent } else { t.fg };
            let bg = if is_selected { t.primary_active } else { left_bg };
            let prefix = if is_selected { "> " } else { "  " };
            draw_text(&mut plane, 2, row, &format!("{}{}", prefix, table), fg, bg, is_selected);
        }

        // Separator
        for y in 0..content_h {
            let idx = (y * area.width + left_rect.width) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Right panel: query + results
        let query_h = 3u16;
        let results_y = query_h;
        let results_h = right_rect.height.saturating_sub(query_h);

        // Query section
        let query_active = matches!(self.active_panel, Panel::Query);
        let query_bg = if query_active { t.surface_elevated } else { t.surface };
        for y in 0..query_h {
            for x in 0..right_rect.width {
                let idx = (y * area.width + left_rect.width + 1 + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = query_bg;
                }
            }
        }

        draw_text(&mut plane, left_rect.width + 3, 0, "Query", t.primary, query_bg, true);
        let query_text = if self.editing_query {
            format!("{}_", self.search_input.query())
        } else {
            self.query.clone()
        };
        draw_text(&mut plane, left_rect.width + 3, 1, &query_text, t.fg, query_bg, false);

        // Results section
        let results_active = matches!(self.active_panel, Panel::Results);
        let results_bg = if results_active { t.surface_elevated } else { t.surface };
        for y in results_y..content_h {
            for x in 0..right_rect.width {
                let idx = (y * area.width + left_rect.width + 1 + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = results_bg;
                }
            }
        }

        draw_text(&mut plane, left_rect.width + 3, results_y, "Results", t.primary, results_bg, true);

        if let Some(ref table) = self.results_table {
            let table_plane = table.render(Rect::new(left_rect.width + 2, results_y + 1, right_rect.width.saturating_sub(2), results_h.saturating_sub(1)));
            for (i, c) in table_plane.cells.iter().enumerate() {
                if c.transparent { continue; }
                let row = i / table_plane.width as usize;
                let col = i % table_plane.width as usize;
                let dst_x = left_rect.width + 2 + col as u16;
                let dst_y = results_y + 1 + row as u16;
                let idx = (dst_y * area.width + dst_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = c.clone();
                }
            }
        } else {
            draw_text(&mut plane, left_rect.width + 3, results_y + 2, "No results", t.fg_muted, results_bg, false);
        }

        // Status bar
        let status_y = area.height.saturating_sub(1);
        let status_plane = self.status_bar.render(Rect::new(0, status_y, area.width, status_h));
        for (i, c) in status_plane.cells.iter().enumerate() {
            if !c.transparent && i < plane.cells.len() {
                let base = (status_y * area.width) as usize;
                if base + i < plane.cells.len() {
                    plane.cells[base + i] = c.clone();
                }
            }
        }

        // Toasts
        for (i, toast) in self.toasts.iter().enumerate() {
            let toast_y = status_y.saturating_sub(2 + i as u16);
            let toast_plane = toast.render(Rect::new(2, toast_y, area.width.saturating_sub(4), 1));
            for (j, c) in toast_plane.cells.iter().enumerate() {
                if !c.transparent && j < plane.cells.len() {
                    let base = (toast_y * area.width + 2) as usize;
                    if base + j < plane.cells.len() {
                        plane.cells[base + j] = c.clone();
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.editing_query {
            match key.code {
                KeyCode::Esc => {
                    self.editing_query = false;
                    self.query = self.search_input.query().to_string();
                    self.dirty = true;
                    true
                }
                KeyCode::Enter => {
                    self.editing_query = false;
                    self.query = self.search_input.query().to_string();
                    self.run_query(&self.query.clone());
                    true
                }
                _ => {
                    let handled = self.search_input.handle_key(key);
                    if handled { self.dirty = true; }
                    handled
                }
            }
        } else {
            match key.code {
                KeyCode::Char('q') => { self.should_quit.store(true, Ordering::SeqCst); true }
                KeyCode::Char('r') => { self.refresh(); self.toast("Refreshed", ToastKind::Info); true }
                KeyCode::Char('e') => {
                    self.editing_query = true;
                    self.search_input = SearchInput::new(WidgetId::new(3)).with_theme(self.theme);
                    self.active_panel = Panel::Query;
                    self.dirty = true;
                    true
                }
                KeyCode::Tab => {
                    self.active_panel = match self.active_panel {
                        Panel::Tables => Panel::Query,
                        Panel::Query => Panel::Results,
                        Panel::Results => Panel::Tables,
                    };
                    self.dirty = true;
                    true
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    match self.active_panel {
                        Panel::Tables => if self.selected_table + 1 < self.tables.len() { self.selected_table += 1; self.dirty = true; }
                        Panel::Results => if let Some(ref mut table) = self.results_table { table.handle_key(key); self.dirty = true; }
                        _ => {}
                    }
                    true
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    match self.active_panel {
                        Panel::Tables => if self.selected_table > 0 { self.selected_table -= 1; self.dirty = true; }
                        Panel::Results => if let Some(ref mut table) = self.results_table { table.handle_key(key); self.dirty = true; }
                        _ => {}
                    }
                    true
                }
                KeyCode::Enter => {
                    if self.active_panel == Panel::Tables {
                        if let Some(table) = self.tables.get(self.selected_table) {
                            self.query = format!("SELECT * FROM {} LIMIT 10", table);
                            self.run_query(&self.query.clone());
                        }
                    }
                    true
                }
                _ => false,
            }
        }
    }
}

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

fn main() -> std::io::Result<()> {
    println!("SQLite Browser — Database explorer");
    println!("Tab: switch panels | e: edit query | r: refresh | q: quit");
    std::thread::sleep(Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let db_path = std::env::args().nth(1).unwrap_or_else(|| ":memory:".to_string());

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::nord();
    let browser = SqliteBrowser::new(should_quit, theme, &db_path);

    let mut app = App::new()?.title("SQLite Browser").fps(30).theme(theme);
    app.add_widget(Box::new(browser), Rect::new(0, 0, w, h));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) { ctx.stop(); }
    }).run(|_ctx| {});

    println!("\nSQLite Browser exited cleanly");
    Ok(())
}
