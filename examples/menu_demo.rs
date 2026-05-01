//! Menu System — MenuBar, ContextMenu, and keyboard shortcuts.
//!
//! Demonstrates:
//! - MenuBar with File/Edit/View/Help dropdown menus
//! - ContextMenu on right-click with item-specific actions
//! - Global keyboard shortcuts (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Q)
//! - Toast feedback for menu actions
//! - List widget with selection and context menu support
//!
//! ## Shortcuts
//! | Key | Action |
//! |-----|--------|
//! | Ctrl+N | New file |
//! | Ctrl+O | Open file |
//! | Ctrl+S | Save file |
//! | Ctrl+Q | Quit application |
//! | ESC | Close menu/modal |

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    ContextAction, ContextMenu, MenuBar, MenuEntry, MenuItem, StatusBar, StatusSegment, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

struct MenuApp {
    id: WidgetId,
    menu_bar: MenuBar,
    list: List<String>,
    status_bar: StatusBar,
    context_menu: Option<ContextMenu>,
    toasts: Vec<Toast>,
    active_menu: Option<usize>,
    ctx_selected: Option<usize>,
}

impl MenuApp {
    fn new(id: WidgetId) -> Self {
        let items = vec!["documents", "images", "projects", "downloads", "music", "videos"];
        let menu_bar = MenuBar::new(WidgetId::new(1)).with_entries(vec![
            MenuEntry::new("File").add_item(MenuItem::new("New (Ctrl+N)"))
                .add_item(MenuItem::new("Open (Ctrl+O)"))
                .add_item(MenuItem::new("Save (Ctrl+S)"))
                .add_item(MenuItem::new("Exit (Ctrl+Q)")),
            MenuEntry::new("Edit").add_item(MenuItem::new("Copy (Ctrl+C)"))
                .add_item(MenuItem::new("Paste (Ctrl+V)"))
                .add_item(MenuItem::new("Select All (Ctrl+A)")),
            MenuEntry::new("View").add_item(MenuItem::new("Toggle Sidebar"))
                .add_item(MenuItem::new("Zoom In"))
                .add_item(MenuItem::new("Zoom Out")),
            MenuEntry::new("Help").add_item(MenuItem::new("About"))
                .add_item(MenuItem::new("Documentation")),
        ]);
        let status_bar = StatusBar::new(WidgetId::new(2))
            .add_segment(StatusSegment::new("Ready").with_fg(Color::Rgb(100, 255, 100)))
            .add_segment(StatusSegment::new("Ctrl+N|O|S: New|Open|Save | Ctrl+Q: Quit").with_fg(Color::Rgb(180, 180, 180)));
        Self { id, menu_bar, list: List::new(items), status_bar, context_menu: None, toasts: Vec::new(), active_menu: None, ctx_selected: None }
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        self.toasts.push(Toast::new(WidgetId::new(100 + self.toasts.len()), msg).with_kind(kind).with_duration(std::time::Duration::from_secs(2)));
    }

    fn do_menu(&mut self, label: &str) {
        let l = label.trim();
        match l {
            s if s.contains("New") => self.toast("New file created", ToastKind::Success),
            s if s.contains("Open") => self.toast("Opened file dialog", ToastKind::Info),
            s if s.contains("Save") => self.toast("Saved!", ToastKind::Success),
            s if s.contains("Exit") => self.toast("Goodbye!", ToastKind::Info),
            s if s.contains("Copy") => self.toast("Copied to clipboard", ToastKind::Info),
            s if s.contains("Paste") => self.toast("Pasted from clipboard", ToastKind::Info),
            s if s.contains("Select All") => self.toast("All items selected", ToastKind::Info),
            s if s.contains("Toggle Sidebar") => self.toast("Sidebar toggled", ToastKind::Info),
            s if s.contains("Zoom") => self.toast("Zoom changed", ToastKind::Info),
            s if s.contains("About") => self.toast("Dracon Terminal Engine v27", ToastKind::Info),
            _ => {}
        }
        self.active_menu = None;
    }

    fn do_ctx(&mut self, idx: usize) {
        let items = vec![("Copy Item", ContextAction::Copy), ("Paste Item", ContextAction::Paste), ("Rename Item", ContextAction::Rename), ("Delete Item", ContextAction::Delete), ("Properties", ContextAction::Open)];
        if idx < items.len() {
            match items[idx].1 {
                ContextAction::Copy => self.toast("Item copied", ToastKind::Info),
                ContextAction::Paste => self.toast("Item pasted", ToastKind::Info),
                ContextAction::Delete => self.toast("Item deleted", ToastKind::Warning),
                ContextAction::Rename => self.toast("Rename mode", ToastKind::Info),
                ContextAction::Open => self.toast("Opening...", ToastKind::Info),
                _ => {}
            }
        }
        self.context_menu = None;
        self.ctx_selected = None;
    }
}

impl Widget for MenuApp {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
    fn set_area(&mut self, _area: Rect) {}
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 0;
        for cell in plane.cells.iter_mut() { cell.bg = Color::Ansi(17); }

        let (hdr, ftr) = (1u16, 1u16);
        let content_h = area.height - hdr - ftr;

        let menu_plane = self.menu_bar.render(Rect::new(0, 0, area.width, hdr));
        for (i, c) in menu_plane.cells.iter().enumerate() { plane.cells[i] = c.clone(); }

        if let Some(idx) = self.active_menu {
            if idx < self.menu_bar.entries.len() {
                let entry = &self.menu_bar.entries[idx];
                let w = 20.max(entry.label.len() as u16 + 4);
                let h = entry.items.len() as u16;
                let mut dp = Plane::new(0, w, h);
                dp.z_index = 70;
                for cell in dp.cells.iter_mut() { cell.bg = Color::Ansi(236); cell.fg = Color::Rgb(200, 200, 200); }
                for (i, item) in entry.items.iter().enumerate() {
                    for (j, ch) in item.label.chars().take(w as usize - 2).enumerate() {
                        let idx = (i as u16 * w + 2 + j as u16) as usize;
                        if idx < dp.cells.len() { dp.cells[idx].char = ch; }
                    }
                }
                for col in 0..w {
                    if col < dp.cells.len() { dp.cells[col as usize].char = '─'; }
                }
                let base = (hdr * area.width) as usize;
                for (i, c) in dp.cells.iter().enumerate() {
                    let idx = base + i;
                    if idx < plane.cells.len() { plane.cells[idx] = c.clone(); }
                }
            }
        }

        let list_rect = Rect::new(2, hdr + 1, area.width - 4, content_h - 2);
        let list_plane = self.list.render(list_rect);
        let base = (hdr * area.width) as usize;
        for (i, c) in list_plane.cells.iter().enumerate() {
            let idx = base + i;
            if idx < plane.cells.len() { plane.cells[idx] = c.clone(); }
        }

        if let Some(ref cm) = self.context_menu {
            let cm_plane = cm.render(area);
            for (i, c) in cm_plane.cells.iter().enumerate() { plane.cells[i] = c.clone(); }
        }

        for toast in &self.toasts {
            if !toast.is_expired() {
                let tp = toast.render(Rect::new(0, 0, area.width, 1));
                for (i, c) in tp.cells.iter().enumerate() { plane.cells[i] = c.clone(); }
                break;
            }
        }

        let status_plane = self.status_bar.render(Rect::new(0, area.height - ftr, area.width, ftr));
        let base = ((area.height - ftr) * area.width) as usize;
        for (i, c) in status_plane.cells.iter().enumerate() {
            let idx = base + i;
            if idx < plane.cells.len() { plane.cells[idx] = c.clone(); }
        }
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.context_menu.is_some() {
            if let KeyCode::Esc = key.code { self.context_menu = None; self.ctx_selected = None; }
            return true;
        }

        if let Some(_) = self.active_menu {
            if let KeyCode::Esc = key.code { self.active_menu = None; return true; }
            if let KeyCode::Enter = key.code {
                if let Some(idx) = self.active_menu {
                    if idx < self.menu_bar.entries.len() {
                        let entry = &self.menu_bar.entries[idx];
                        if !entry.items.is_empty() { self.do_menu(&entry.items[0].label); }
                    }
                }
            }
            self.active_menu = None;
            return true;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('n') => { self.toast("New file created", ToastKind::Success); true }
                KeyCode::Char('o') => { self.toast("Opened file dialog", ToastKind::Info); true }
                KeyCode::Char('s') => { self.toast("Saved!", ToastKind::Success); true }
                KeyCode::Char('q') => { self.toast("Goodbye!", ToastKind::Info); true }
                KeyCode::Char('c') => { self.toast("Copied to clipboard", ToastKind::Info); true }
                KeyCode::Char('v') => { self.toast("Pasted from clipboard", ToastKind::Info); true }
                KeyCode::Char('a') => { self.toast("All items selected", ToastKind::Info); true }
                _ => false,
            }
        } else if let KeyCode::Esc = key.code {
            self.active_menu = None;
            self.context_menu = None;
            true
        } else {
            self.list.handle_key(key)
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let (hdr, ftr) = (1u16, 1u16);
        let content_h = 24u16 - hdr - ftr;

        if self.context_menu.is_some() {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                self.context_menu = None;
                self.ctx_selected = None;
                return true;
            }
        }

        if let MouseEventKind::Down(MouseButton::Right) = kind {
            let list_rect = Rect::new(2, hdr + 1, 76, content_h - 2);
            if col >= list_rect.x && col < list_rect.x + list_rect.width && row >= list_rect.y && row < list_rect.y + list_rect.height {
                self.context_menu = Some(ContextMenu::new_with_id(WidgetId::new(50), vec![
                    ("Copy Item", ContextAction::Copy),
                    ("Paste Item", ContextAction::Paste),
                    ("Rename Item", ContextAction::Rename),
                    ("Delete Item", ContextAction::Delete),
                    ("Properties", ContextAction::Open),
                ]).with_anchor(col, row));
                return true;
            }
        }

        if let Some(ref mut cm) = self.context_menu {
            if cm.handle_mouse(kind, col, row) {
                if let MouseEventKind::Down(MouseButton::Left) = kind {
                    let local_row = row.saturating_sub(cm.anchor_y) as usize;
                    self.ctx_selected = Some(local_row);
                    self.do_ctx(local_row);
                }
                return true;
            }
        }

        if row == 0 && matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
            return self.menu_bar.handle_mouse(kind, col, row);
        }

        let list_rect = Rect::new(2, hdr + 1, 76, content_h - 2);
        if col >= list_rect.x && col < list_rect.x + list_rect.width && row >= list_rect.y && row < list_rect.y + list_rect.height {
            return self.list.handle_mouse(kind, col - list_rect.x, row - list_rect.y);
        }
        false
    }
}

fn main() -> std::io::Result<()> {
    App::new()?.title("Menu System Demo").fps(30).run(|ctx| {
        let (w, h) = ctx.compositor().size();
        let area = Rect::new(0, 0, w, h);
        let mut app = MenuApp::new(WidgetId::new(0));
        app.set_area(area);
        app.toasts.retain(|t| !t.is_expired());
        ctx.add_plane(app.render(area));
    })
}