#![allow(missing_docs)]
//! Table Widget Example — demonstrates sortable, selectable data table.
//!
//! Shows a table with columns, row selection, and keyboard navigation.
//!
//! Controls: arrows to navigate, Enter to select, q to quit.

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Column, Table};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
#[allow(dead_code)]
struct User {
    name: String,
    role: String,
    status: String,
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

struct TableApp {
    table: Table<User>,
    should_quit: Arc<AtomicBool>,
}

impl TableApp {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let columns = vec![
            Column { header: "Name".to_string(), width: 20 },
            Column { header: "Role".to_string(), width: 15 },
            Column { header: "Status".to_string(), width: 10 },
        ];

        let rows = vec![
            User { name: "Alice".to_string(), role: "Admin".to_string(), status: "Active".to_string() },
            User { name: "Bob".to_string(), role: "User".to_string(), status: "Active".to_string() },
            User { name: "Charlie".to_string(), role: "Editor".to_string(), status: "Away".to_string() },
            User { name: "Diana".to_string(), role: "Admin".to_string(), status: "Active".to_string() },
            User { name: "Eve".to_string(), role: "User".to_string(), status: "Active".to_string() },
            User { name: "Frank".to_string(), role: "Viewer".to_string(), status: "Offline".to_string() },
            User { name: "Grace".to_string(), role: "Editor".to_string(), status: "Active".to_string() },
        ];

        let mut table = Table::new(columns)
            .with_theme(theme)
            .with_rows(rows)
            .on_select(|user| {
                let _ = user;
            });
        table.set_visible_count(10);

        Self { table, should_quit }
    }
}

impl Widget for TableApp {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.table.area() }
    fn set_area(&mut self, area: Rect) { self.table.set_area(area); }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }
    fn render(&self, area: Rect) -> Plane { self.table.render(area) }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Char('q') => { self.should_quit.store(true, Ordering::SeqCst); true }
            _ => self.table.handle_key(key),
        }
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::nord();
    let app = TableApp::new(should_quit, theme);

    App::new()?
        .title("Table Widget Demo")
        .fps(30)
        .theme(theme)
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) { ctx.stop(); }
        })
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();
            ctx.add_plane(app.render(Rect::new(0, 0, w, h)));
        })
}
