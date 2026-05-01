//! Menu System — MenuBar, ContextMenu, and keyboard shortcuts.
//!
//! Demonstrates:
//! - Custom MenuBar with File/Edit/View/Help dropdown menus
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
    ContextAction, ContextMenu, StatusBar, StatusSegment, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

struct MenuLabel(&'static str);

struct MenuApp {
    id: WidgetId,
    list: List<String>,
    status_bar: StatusBar,
    context_menu: Option<ContextMenu>,
    toasts: Vec<Toast>,
    menu_bar: Vec<MenuLabel>,
    active_menu: Option<usize>,
    selected_idx: Option<usize>,
}

impl MenuApp {
    fn new(id: WidgetId) -> Self {
        let items = vec!["documents".to_string(), "images".to_string(), "projects".to_string(), "downloads".to_string(), "music".to_string(), "videos".to_string()];
        let menu_bar = vec![MenuLabel("File"), MenuLabel("Edit"), MenuLabel("View"), MenuLabel("Help")];
        let status_bar = StatusBar::new(WidgetId::new(2))
            .add_segment(StatusSegment::new("Ready").with_fg(Color::Rgb(100, 255, 100)))
            .add_segment(StatusSegment::new("Ctrl+N|O|S: New|Open|Save | Ctrl+Q: Quit").with_fg(Color::Rgb(180, 180, 180)));
        Self { id, list: List::new(items), status_bar, context_menu: None, toasts: Vec::new(), menu_bar, active_menu: None, selected_idx: None }
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        self.toasts.push(Toast::new(WidgetId::new(100 + self.toasts.len()), msg).with_kind(kind).with_duration(std::time::Duration::from_secs(2)));
    }

    fn menu_item_count(&self, menu_idx: usize) -> usize {
        match menu_idx {
            0 => 4,
            1 => 3,
            2 => 3,
            3 => 2,
            _ => 0,
        }
    }

    fn get_menu_item(&self, menu_idx: usize, item_idx: usize) -> &'static str {
        match menu_idx {
            0 => match item_idx { 0 => "New (Ctrl+N)", 1 => "Open (Ctrl+O)", 2 => "Save (Ctrl+S)", 3 => "Exit (Ctrl+Q)", _ => "" },
            1 => match item_idx { 0 => "Copy (Ctrl+C)", 1 => "Paste (Ctrl+V)", 2 => "Select All (Ctrl+A)", _ => "" },
            2 => match item_idx { 0 => "Toggle Sidebar", 1 => "Zoom In", 2 => "Zoom Out", _ => "" },
            3 => match item_idx { 0 => "About", 1 => "Documentation", _ => "" },
            _ => "",
        }
    }

    fn do_menu(&mut self, menu_idx: usize, item_idx: usize) {
        let label = self.get_menu_item(menu_idx, item_idx);
        match label {
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
            s if s.contains("Documentation") => self.toast("Opening docs...", ToastKind::Info),
            _ => {}
        }
        self.active_menu = None;
    }

    fn do_ctx(&mut self, idx: usize) {
        match idx {
            0 => self.toast("Item copied", ToastKind::Info),
            1 => self.toast("Item pasted", ToastKind::Info),
            2 => self.toast("Rename mode", ToastKind::Info),
            3 => self.toast("Item deleted", ToastKind::Warning),
            4 => self.toast("Opening...", ToastKind::Info),
            _ => {}
        }
        self.context_menu = None;
        self.selected_idx = None;
    }

    fn render_menu_bar(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, 1);
        plane.z_index = 60;
        for cell in plane.cells.iter_mut() { cell.bg = Color::Ansi(236); cell.fg = Color::Rgb(200, 200, 200); }

        let total = self.menu_bar.len();
        let entry_w = (area.width as usize / total.max(1)) as u16;

        for (i, m) in self.menu_bar.iter().enumerate() {
            let is_active = self.active_menu == Some(i);
            let prefix = if is_active { "[" } else { " " };
            let suffix = if is_active { "]" } else { " " };
            let display = format!("{}{}{}", prefix, m.0, suffix);
            for (j, ch) in display.chars().enumerate() {
                let idx = (i as u16 * entry_w + j as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = ch; }
            }
        }
        plane
    }

    fn render_dropdown(&self, menu_idx: usize, _area: Rect) -> Plane {
        let item_count = self.menu_item_count(menu_idx) as u16;
        let label = self.menu_bar.get(menu_idx).map(|m| m.0).unwrap_or("");
        let w = 20.max(label.len() as u16 + 4);
        let h = item_count;

        let mut plane = Plane::new(0, w, h);
        plane.z_index = 70;
        for cell in plane.cells.iter_mut() { cell.bg = Color::Ansi(236); cell.fg = Color::Rgb(200, 200, 200); }

        for i in 0..self.menu_item_count(menu_idx) {
            let item_label = self.get_menu_item(menu_idx, i);
            for (j, ch) in item_label.chars().take(w as usize - 2).enumerate() {
                let idx = (i as u16 * w + 2 + j as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = ch; }
            }
        }

        for col in 0..w {
            if (col as usize) < plane.cells.len() { plane.cells[col as usize].char = '─'; }
        }
        plane
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
        let content_h = area.height.saturating_sub(hdr + ftr);

        let menu_plane = self.render_menu_bar(Rect::new(0, 0, area.width, hdr));
        for (i, c) in menu_plane.cells.iter().enumerate() { plane.cells[i] = c.clone(); }

        if let Some(idx) = self.active_menu {
            let dp = self.render_dropdown(idx, area);
            let base = (hdr * area.width) as usize;
            for (i, c) in dp.cells.iter().enumerate() {
                let idx = base + i;
                if idx < plane.cells.len() { plane.cells[idx] = c.clone(); }
            }
        }

        let list_rect = Rect::new(2, hdr + 1, area.width - 4, content_h.saturating_sub(2));
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
            if let KeyCode::Esc = key.code { self.context_menu = None; self.selected_idx = None; }
            return true;
        }

        if let Some(menu_idx) = self.active_menu {
            match key.code {
                KeyCode::Esc => { self.active_menu = None; true }
                KeyCode::Enter => {
                    if let Some(item_idx) = self.selected_idx {
                        self.do_menu(menu_idx, item_idx);
                    }
                    true
                }
                KeyCode::Down | KeyCode::Up => {
                    let item_count = self.menu_item_count(menu_idx);
                    if item_count == 0 { return true; }
                    let current = self.selected_idx.unwrap_or(0);
                    let next = match key.code {
                        KeyCode::Down => (current + 1).min(item_count - 1),
                        KeyCode::Up => current.saturating_sub(1),
                        _ => current,
                    };
                    self.selected_idx = Some(next);
                    true
                }
                _ => { self.active_menu = None; false }
            }
        } else if key.modifiers.contains(KeyModifiers::CONTROL) {
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
        let content_h = 24u16.saturating_sub(hdr + ftr);

        if self.context_menu.is_some() {
            if matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
                self.context_menu = None;
                self.selected_idx = None;
                return true;
            }
        }

        if row == 0 {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                let total = self.menu_bar.len();
                let entry_w = (80usize / total.max(1)) as u16;
                let menu_idx = (col / entry_w) as usize;
                if menu_idx < total {
                    if self.active_menu == Some(menu_idx) {
                        self.active_menu = None;
                    } else {
                        self.active_menu = Some(menu_idx);
                        self.selected_idx = Some(0);
                    }
                }
                return true;
            }
        }

        if let MouseEventKind::Down(MouseButton::Right) = kind {
            let list_rect = Rect::new(2, hdr + 1, 76, content_h.saturating_sub(2));
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
                    let local_row = row.saturating_sub(cm.area().y) as usize;
                    self.selected_idx = Some(local_row);
                    self.do_ctx(local_row);
                }
                return true;
            }
        }

        let list_rect = Rect::new(2, hdr + 1, 76, content_h.saturating_sub(2));
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