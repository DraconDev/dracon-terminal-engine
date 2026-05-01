//! File Manager UI — demonstrates Tree + Table + Breadcrumbs + StatusBar + SplitPane + ContextMenu.
//!
//! # Layout
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │ 📁 ~/projects/dracon-terminal-engine          [≡ Menu]  │
//! ├─────────────────┬───────────────────────────────────────┤
//! │ EXPLORER        │  📋 details (right)                   │
//! │ ├─ docs/        │  ─────────────────────                │
//! │ ├─ src/         │  Name: Cargo.toml                     │
//! │ │  ├─ main.rs   │  Size: 2.3 KB                         │
//! │ │  └─ lib.rs    │  Modified: 2025-01-15 14:32           │
//! │ ├─ tests/       │  Permissions: rw-r--r--              │
//! │ └─ README.md    │                                       │
//! │                 │  Preview:                             │
//! │                 │  [package]                            │
//! │                 │  name = "dracon-terminal-engine"      │
//! │                 │  version = "27.0.5"                   │
//! ├─────────────────┴───────────────────────────────────────┤
//! │ 8 items | 3 selected | Press ? for shortcuts            │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Mock File System
//! Uses static mock data — no real filesystem access.
//!
//! # Navigation
//! | Key | Action |
//! |-----|--------|
//! | Up/Down | Move selection in tree or table |
//! | Left/Right | Collapse/expand tree node |
//! | Enter | Open selected folder/file |
//! | Backspace | Go up one directory level |
//! | c | Show context menu |
//!
//! # Mouse
//! - Click tree item to select and expand
//! - Click table row to select
//! - Right-click anywhere for context menu

use dracon_terminal_engine::compositor::{Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, ContextAction, ContextMenu, SplitPane, StatusBar, StatusSegment, Toast, ToastKind,
    Tree, TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

#[derive(Clone)]
struct MockFs {
    name: &'static str,
    children: Option<Vec<MockFs>>,
    is_dir: bool,
}

impl MockFs {
    fn icon(&self) -> &'static str {
        if self.is_dir { "📁" } else { "📄" }
    }

    fn to_tree_node(&self) -> TreeNode {
        let label = format!("{}{}", self.icon(), self.name);
        let mut node = TreeNode::new(&label);
        if let Some(ref children) = self.children {
            for child in children {
                node.add_child(child.to_tree_node());
            }
        }
        node
    }

    fn find_by_path(&self, path: &[usize]) -> Option<&MockFs> {
        if path.is_empty() { return Some(self); }
        let children = self.children.as_ref()?;
        let idx = path[0];
        if idx >= children.len() { return None; }
        children[idx].find_by_path(&path[1..])
    }

    fn child_count(&self) -> usize {
        self.children.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}

#[derive(Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
}

enum FocusPanel { Tree, Table, ContextMenu }

struct FileManager {
    id: WidgetId,
    fs: MockFs,
    tree: Tree,
    breadcrumbs: Breadcrumbs,
    tree_path: Vec<usize>,
    selected_entry: Option<FileEntry>,
    context_menu: Option<ContextMenu>,
    toast: Option<Toast>,
    focus: FocusPanel,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl FileManager {
    fn new(id: WidgetId) -> Self {
        let fs = MockFs {
            name: "root",
            is_dir: true,
            children: Some(vec![
                MockFs { name: "src", is_dir: true, children: Some(vec![
                    MockFs { name: "main.rs", is_dir: false, children: None },
                    MockFs { name: "lib.rs", is_dir: false, children: None },
                ])},
                MockFs { name: "docs", is_dir: true, children: Some(vec![
                    MockFs { name: "README.md", is_dir: false, children: None },
                    MockFs { name: "CHANGELOG.md", is_dir: false, children: None },
                ])},
                MockFs { name: "tests", is_dir: true, children: Some(vec![
                    MockFs { name: "test_main.rs", is_dir: false, children: None },
                ])},
                MockFs { name: "Cargo.toml", is_dir: false, children: None },
                MockFs { name: "README.md", is_dir: false, children: None },
                MockFs { name: ".gitignore", is_dir: false, children: None },
            ]),
        };

        let root_node = fs.to_tree_node();
        let tree = Tree::new(WidgetId::new(1)).with_root(vec![root_node]);
        let breadcrumbs = Breadcrumbs::new_with_id(WidgetId::new(3), vec!["~".to_string(), "projects".to_string(), "dracon-terminal-engine".to_string()]);

        Self {
            id,
            fs,
            tree,
            breadcrumbs,
            tree_path: Vec::new(),
            selected_entry: None,
            context_menu: None,
            toast: None,
            focus: FocusPanel::Tree,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
        }
    }

    fn current_node(&self) -> &MockFs {
        self.fs.find_by_path(&self.tree_path).unwrap_or(&self.fs)
    }

    fn update_breadcrumbs(&mut self) {
        let mut segments = vec!["~".to_string(), "projects".to_string(), "dracon-terminal-engine".to_string()];
        for &idx in &self.tree_path {
            if let Some(node) = self.fs.find_by_path(&self.tree_path[..=idx]) {
                segments.push(node.name.to_string());
            }
        }
        self.breadcrumbs = Breadcrumbs::new_with_id(WidgetId::new(3), segments);
        self.dirty = true;
    }

    fn go_up(&mut self) {
        if !self.tree_path.is_empty() {
            self.tree_path.pop();
            self.tree.set_selected_path(self.tree_path.clone());
            self.selected_entry = None;
            self.update_breadcrumbs();
        }
    }

    fn open_tree_selection(&mut self) {
        if let Some(path) = self.tree.get_selected_path().last().copied() {
            if let Some(node) = self.current_node().children.as_ref().and_then(|c| c.get(path)) {
                if node.is_dir {
                    self.tree_path.push(path);
                    self.tree.set_selected_path(self.tree_path.clone());
                    self.selected_entry = None;
                    self.update_breadcrumbs();
                } else {
                    self.show_toast(&format!("Opening {}...", node.name), ToastKind::Info);
                }
            }
        }
    }

    fn show_toast(&mut self, message: &str, kind: ToastKind) {
        self.toast = Some(Toast::new(WidgetId::new(100), message).with_kind(kind));
        self.dirty = true;
    }

    fn show_context_menu(&mut self, x: u16, y: u16) {
        self.context_menu = Some(
            ContextMenu::new_with_id(WidgetId::new(50), vec![
                ("Open", ContextAction::Open),
                ("Copy", ContextAction::Copy),
                ("Paste", ContextAction::Paste),
                ("Rename", ContextAction::Rename),
                ("Delete", ContextAction::Delete),
                ("Separator", ContextAction::Separator),
                ("Properties", ContextAction::Edit),
            ])
            .with_width(18)
            .with_anchor(x, y),
        );
        self.focus = FocusPanel::ContextMenu;
        self.dirty = true;
    }

    fn handle_context_action(&mut self, action: ContextAction) {
        let name = self.selected_entry.as_ref().map(|e| e.name.as_str()).unwrap_or("item");
        match action {
            ContextAction::Open => self.show_toast(&format!("Opening {}...", name), ToastKind::Info),
            ContextAction::Copy => self.show_toast(&format!("Copied {} to clipboard", name), ToastKind::Success),
            ContextAction::Paste => self.show_toast("Pasted from clipboard", ToastKind::Success),
            ContextAction::Rename => self.show_toast(&format!("Renaming {}...", name), ToastKind::Info),
            ContextAction::Delete => self.show_toast(&format!("Deleted {}", name), ToastKind::Warning),
            ContextAction::Edit => self.show_toast(&format!("Properties of {}", name), ToastKind::Info),
            _ => {}
        }
        self.context_menu = None;
        self.focus = FocusPanel::Table;
        self.dirty = true;
    }

    fn get_selected_file_entry(&self) -> Option<FileEntry> {
        let path = self.tree.get_selected_path();
        if path.is_empty() { return None; }
        self.fs.find_by_path(path).map(|node| FileEntry {
            name: node.name.to_string(),
            is_dir: node.is_dir,
        })
    }

    fn size_and_modified_for(&self, name: &str) -> (String, String) {
        match name {
            "main.rs" => ("1.2 KB".to_string(), "2025-01-10".to_string()),
            "lib.rs" => ("3.4 KB".to_string(), "2025-01-12".to_string()),
            "README.md" => ("4.1 KB".to_string(), "2025-01-08".to_string()),
            "CHANGELOG.md" => ("8.7 KB".to_string(), "2025-01-15".to_string()),
            "test_main.rs" => ("0.8 KB".to_string(), "2025-01-05".to_string()),
            "Cargo.toml" => ("2.3 KB".to_string(), "2025-01-15".to_string()),
            ".gitignore" => ("0.1 KB".to_string(), "2025-01-01".to_string()),
            _ => ("—".to_string(), "—".to_string()),
        }
    }

    fn render_details(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(1, area.width, area.height);
        plane.z_index = 5;

        for cell in plane.cells.iter_mut() {
            cell.bg = Color::Ansi(17);
        }

        let print_line = |plane: &mut Plane, y: u16, text: &str, fg: Color, bold: bool| {
            let style = if bold { Styles::BOLD } else { Styles::empty() };
            for (i, c) in text.chars().take(area.width as usize - 2).enumerate() {
                let idx = (y * plane.width + 1 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].style = style;
                }
            }
        };

        let mut y = 1u16;
        print_line(&mut plane, y, "DETAILS", Color::Rgb(0, 255, 136), true);
        y += 1;
        print_line(&mut plane, y, "────────────────────────────", Color::Ansi(100), false);
        y += 2;

        if let Some(ref entry) = self.selected_entry {
            let (size, modified) = self.size_and_modified_for(&entry.name);
            print_line(&mut plane, y, &format!("Name: {}", entry.name), Color::Rgb(255, 255, 255), false);
            y += 1;
            print_line(&mut plane, y, &format!("Size: {}", size), Color::Rgb(200, 150, 100), false);
            y += 1;
            print_line(&mut plane, y, &format!("Modified: {}", modified), Color::Rgb(180, 180, 180), false);
            y += 1;
            print_line(&mut plane, y, "Permissions: rw-r--r--", Color::Rgb(180, 180, 180), false);
            y += 2;

            if entry.name.ends_with(".toml") || entry.name.ends_with(".md") || entry.name.ends_with(".rs") {
                print_line(&mut plane, y, "Preview:", Color::Rgb(0, 255, 136), true);
                y += 1;

                let preview1 = if entry.name == "Cargo.toml" { "[package]" } else { "# preview" };
                let preview2 = if entry.name == "Cargo.toml" { "name = \"dracon-terminal-engine\"" } else { "content..." };
                let preview3 = if entry.name == "Cargo.toml" { "version = \"27.0.5\"" } else { "" };

                print_line(&mut plane, y, preview1, Color::Rgb(150, 150, 150), false); y += 1;
                print_line(&mut plane, y, preview2, Color::Rgb(150, 150, 150), false); y += 1;
                print_line(&mut plane, y, preview3, Color::Rgb(150, 150, 150), false);
            }
        } else {
            print_line(&mut plane, y, "Select a file to view details", Color::Rgb(100, 100, 100), false);
            y += 2;
            print_line(&mut plane, y, "Use tree to navigate.", Color::Rgb(80, 80, 80), false);
        }

        plane
    }

    fn render_table(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let current = self.current_node();
        let children = match current.children {
            Some(ref c) => c,
            None => return plane,
        };

        for cell in plane.cells.iter_mut() {
            cell.bg = Color::Ansi(17);
        }

        for (i, c) in "Name                  Size      Modified".chars().take(area.width as usize).enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = Color::Rgb(0, 255, 136);
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let mut y = 1u16;
        for child in children.iter() {
            if y >= area.height { break; }
            let (size, modified) = self.size_and_modified_for(child.name);
            let line = format!("{}{:<20} {:<10} {:<12}", child.icon(), child.name, size, modified);
            let fg = if child.is_dir { Color::Rgb(100, 200, 255) } else { Color::Rgb(200, 200, 200) };

            for (j, c) in line.chars().take(area.width as usize).enumerate() {
                let idx = (y * plane.width + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                }
            }
            y += 1;
        }

        plane
    }
}

impl Widget for FileManager {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); self.dirty = true; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { self.dirty || self.tree.needs_render() }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; self.tree.clear_dirty(); }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 0;

        for cell in plane.cells.iter_mut() {
            cell.bg = Color::Ansi(17);
        }

        let header_height = 1u16;
        let footer_height = 1u16;
        let content_height = area.height.saturating_sub(header_height + footer_height);

        let header_rect = Rect::new(0, 0, area.width, header_height);
        let content_rect = Rect::new(0, header_height, area.width, content_height);
        let footer_rect = Rect::new(0, area.height - footer_height, area.width, footer_height);

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.30);
        let (tree_rect, detail_rect) = split.split(content_rect);
        let divider_plane = split.render_divider(content_rect);

        let bc_plane = self.breadcrumbs.render(header_rect);
        for (i, cell) in bc_plane.cells.iter().enumerate() {
            if i < plane.cells.len() { plane.cells[i] = cell.clone(); }
        }

        let tree_plane = self.tree.render(tree_rect);
        for (i, cell) in tree_plane.cells.iter().enumerate() {
            let base = (header_height * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() { plane.cells[idx] = cell.clone(); }
        }

        for (i, cell) in divider_plane.cells.iter().enumerate() {
            let base = (header_height * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() { plane.cells[idx] = cell.clone(); }
        }

        let table_plane = self.render_table(detail_rect);
        for (i, cell) in table_plane.cells.iter().enumerate() {
            let base = (header_height * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() { plane.cells[idx] = cell.clone(); }
        }

        let details_plane = self.render_details(Rect::new(
            detail_rect.x + detail_rect.width / 2,
            detail_rect.y + content_height / 2,
            detail_rect.width / 2,
            detail_rect.height / 2,
        ));
        for (i, cell) in details_plane.cells.iter().enumerate() {
            let base = ((header_height + content_height / 2) * area.width) as usize;
            let offset = ((detail_rect.width / 2) * area.width) as usize;
            let idx = base + offset + i;
            if idx < plane.cells.len() { plane.cells[idx] = cell.clone(); }
        }

        let item_count = self.current_node().child_count();
        let selected_text = if self.selected_entry.is_some() { "1 selected" } else { "0 selected" };
        let status_text = format!("{} items | {} | Press ? for shortcuts", item_count, selected_text);
        let status_bar = StatusBar::new(WidgetId::new(10))
            .add_segment(StatusSegment::new(&status_text).with_fg(Color::Rgb(180, 180, 180)).with_bg(Color::Ansi(236)));
        let status_plane = status_bar.render(footer_rect);
        for (i, cell) in status_plane.cells.iter().enumerate() {
            let base = ((area.height - footer_height) * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() { plane.cells[idx] = cell.clone(); }
        }

        if let Some(ref menu) = self.context_menu {
            let menu_plane = menu.render(area);
            for (i, cell) in menu_plane.cells.iter().enumerate() {
                let y = menu_plane.y as usize;
                let x = menu_plane.x as usize;
                let row = i / menu_plane.width as usize;
                let col = i % menu_plane.width as usize;
                let target_idx = ((y + row) * area.width as usize) + x + col;
                if target_idx < plane.cells.len() { plane.cells[target_idx] = cell.clone(); }
            }
        }

        if let Some(ref toast) = self.toast {
            let toast_plane = toast.render(Rect::new(area.width.saturating_sub(40), area.height - 2, 40, 1));
            for (i, cell) in toast_plane.cells.iter().enumerate() {
                let base = ((area.height - 2) * area.width) as usize;
                let idx = base + (area.width as usize - 40) + i;
                if idx < plane.cells.len() { plane.cells[idx] = cell.clone(); }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if let Some(ref mut _menu) = self.context_menu {
            if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                self.context_menu = None;
                self.focus = FocusPanel::Table;
                self.dirty = true;
                return true;
            }
            return false;
        }

        if let Some(ref mut toast) = self.toast {
            if toast.is_expired() { self.toast = None; self.dirty = true; }
            return false;
        }

        match key.code {
            KeyCode::Backspace => { self.go_up(); true }
            KeyCode::Enter => { self.open_tree_selection(); true }
            KeyCode::Char('c') => { self.show_context_menu(30, 10); true }
            _ => {
                if self.tree.handle_key(key) {
                    self.tree_path = self.tree.get_selected_path().to_vec();
                    self.selected_entry = self.get_selected_file_entry();
                    self.dirty = true;
                    return true;
                }
                false
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if let Some(ref mut menu) = self.context_menu {
            if menu.handle_mouse(kind.clone(), col, row) {
                if let MouseEventKind::Down(_) = kind { self.context_menu = None; self.focus = FocusPanel::Table; }
                return true;
            }
            if let MouseEventKind::Down(_) = kind { self.context_menu = None; self.show_context_menu(col, row); return true; }
        }

        if let Some(ref mut toast) = self.toast {
            if toast.is_expired() { self.toast = None; self.dirty = true; }
        }

        let header_height = 1u16;
        let footer_height = 1u16;
        let content_height = 24u16.saturating_sub(header_height + footer_height);

        if row == 0 { return self.breadcrumbs.handle_mouse(kind, col, row); }

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.30);
        let (tree_rect, detail_rect) = split.split(Rect::new(0, header_height, 80, content_height));

        if col <= tree_rect.width && row > header_height && row < header_height + tree_rect.height {
            if self.tree.handle_mouse(kind, col, row - header_height) {
                self.tree_path = self.tree.get_selected_path().to_vec();
                self.selected_entry = self.get_selected_file_entry();
                self.dirty = true;
                return true;
            }
        }

        if let MouseEventKind::Down(_) = kind {
            if col > tree_rect.width && row > header_height && row < header_height + detail_rect.height {
                let rel_row = (row - header_height).saturating_sub(1) as usize;
                let children_opt = self.current_node().children.clone();
                if let Some(children) = children_opt {
                    if rel_row < children.len() {
                        let entry = FileEntry { name: children[rel_row].name.to_string(), is_dir: children[rel_row].is_dir };
                        let show_toast = !children[rel_row].is_dir;
                        drop(children_opt);
                        self.selected_entry = Some(entry);
                        if show_toast { self.show_toast(&format!("Opening {}...", self.selected_entry.as_ref().unwrap().name), ToastKind::Info); }
                        self.dirty = true;
                        return true;
                    }
                }
            }
            self.show_context_menu(col, row);
            return true;
        }

        false
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::dark();

    App::new()?
        .title("File Manager")
        .fps(30)
        .theme(theme)
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();
            let area = Rect::new(0, 0, w, h);
            let mut fm = FileManager::new(WidgetId::new(0));
            fm.set_area(area);
            let plane = fm.render(area);
            ctx.add_plane(plane);
        })
}