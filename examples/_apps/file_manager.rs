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

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, ContextAction, ContextMenu, SplitPane, StatusBar, StatusSegment, Table, TableRow,
    Toast, ToastKind, Tree, TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

static MOCK_FS: FileNode = FileNode {
    name: "root",
    is_dir: true,
    children: Some(&[
        FileNode {
            name: "src",
            is_dir: true,
            children: Some(&[
                FileNode {
                    name: "main.rs",
                    is_dir: false,
                    size: "1.2 KB",
                    modified: "2025-01-10",
                },
                FileNode {
                    name: "lib.rs",
                    is_dir: false,
                    size: "3.4 KB",
                    modified: "2025-01-12",
                },
            ]),
        },
        FileNode {
            name: "docs",
            is_dir: true,
            children: Some(&[
                FileNode {
                    name: "README.md",
                    is_dir: false,
                    size: "4.1 KB",
                    modified: "2025-01-08",
                },
                FileNode {
                    name: "CHANGELOG.md",
                    is_dir: false,
                    size: "8.7 KB",
                    modified: "2025-01-15",
                },
            ]),
        },
        FileNode {
            name: "tests",
            is_dir: true,
            children: Some(&[FileNode {
                name: "test_main.rs",
                is_dir: false,
                size: "0.8 KB",
                modified: "2025-01-05",
            }]),
        },
        FileNode {
            name: "Cargo.toml",
            is_dir: false,
            size: "2.3 KB",
            modified: "2025-01-15",
        },
        FileNode {
            name: "README.md",
            is_dir: false,
            size: "4.1 KB",
            modified: "2025-01-08",
        },
        FileNode {
            name: ".gitignore",
            is_dir: false,
            size: "0.1 KB",
            modified: "2025-01-01",
        },
    ]),
};

struct FileNode {
    name: &'static str,
    is_dir: bool,
    children: Option<&'static [FileNode]>,
    size: &'static str,
    modified: &'static str,
}

impl FileNode {
    fn icon(&self) -> &'static str {
        if self.is_dir {
            "📁"
        } else {
            "📄"
        }
    }

    fn to_tree_node(&self, depth: usize) -> TreeNode {
        let label = format!("{}{}", self.icon(), self.name);
        let mut node = TreeNode::new(&label);
        node.expanded = depth == 0;
        if let Some(children) = self.children {
            for child in children {
                node.add_child(child.to_tree_node(depth + 1));
            }
        }
        node
    }

    fn find_by_path(&self, path: &[usize]) -> Option<&FileNode> {
        if path.is_empty() {
            return Some(self);
        }
        let children = self.children?;
        let idx = path[0];
        if idx >= children.len() {
            return None;
        }
        children[idx].find_by_path(&path[1..])
    }

    fn children_as_entries(&self) -> Vec<FileEntry> {
        self.children
            .map(|arr| arr.iter().map(FileEntry::from).collect())
            .unwrap_or_default()
    }
}

#[derive(Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: String,
    modified: String,
}

impl From<&FileNode> for FileEntry {
    fn from(node: &FileNode) -> Self {
        Self {
            name: node.name.to_string(),
            is_dir: node.is_dir,
            size: node.size.to_string(),
            modified: node.modified.to_string(),
        }
    }
}

impl std::fmt::Display for FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.size)
    }
}

struct FileManager {
    id: WidgetId,
    fs: &'static FileNode,
    tree: Tree,
    table: Table<FileEntry>,
    breadcrumbs: Breadcrumbs,
    tree_path: Vec<usize>,
    selected_entry: Option<FileEntry>,
    context_menu: Option<ContextMenu>,
    toast: Option<Toast>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl FileManager {
    fn new(id: WidgetId) -> Self {
        let root_node = MOCK_FS.to_tree_node(0);
        let tree = Tree::new(WidgetId::new(1)).with_root(vec![root_node]);

        let columns = vec![
            Column { header: "Name".to_string(), width: 20 },
            Column { header: "Size".to_string(), width: 10 },
            Column { header: "Modified".to_string(), width: 12 },
        ];
        let mut table = Table::new_with_id(WidgetId::new(2), columns);
        table.set_visible_count(15);

        let segments = vec!["~".to_string(), "projects".to_string(), "dracon-terminal-engine".to_string()];
        let breadcrumbs = Breadcrumbs::new_with_id(WidgetId::new(3), segments);

        Self {
            id,
            fs: &MOCK_FS,
            tree,
            table,
            breadcrumbs,
            tree_path: Vec::new(),
            selected_entry: None,
            context_menu: None,
            toast: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
        }
    }

    fn current_node(&self) -> &FileNode {
        self.fs.find_by_path(&self.tree_path).unwrap_or(self.fs)
    }

    fn update_table(&mut self) {
        let entries = self.current_node().children_as_entries();
        self.table = Table::new_with_id(WidgetId::new(2), vec![
            Column { header: "Name".to_string(), width: 20 },
            Column { header: "Size".to_string(), width: 10 },
            Column { header: "Modified".to_string(), width: 12 },
        ]);
        self.table.set_visible_count(15);
        for entry in entries {
            self.table.rows.push(TableRow { data: entry });
        }
        self.table.dirty = true;
    }

    fn show_toast(&mut self, message: &str, kind: ToastKind) {
        self.toast = Some(Toast::new(WidgetId::new(100), message).with_kind(kind));
        self.dirty = true;
    }

    fn navigate_to_path(&mut self, path: Vec<usize>) {
        self.tree_path = path.clone();
        self.tree.set_selected_path(path);
        self.update_table();
        self.update_breadcrumbs();
        self.dirty = true;
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
            self.update_table();
            self.update_breadcrumbs();
            self.dirty = true;
        }
    }

    fn open_selected(&mut self) {
        if let Some((node, _)) = self.get_selected_tree_node() {
            if node.is_dir {
                if !self.tree_path.is_empty() || node.name != "root" {
                    if let Some(idx) = self.tree.selected_path().last().copied() {
                        self.tree_path.push(idx);
                        self.update_table();
                        self.update_breadcrumbs();
                        self.dirty = true;
                    }
                } else if let Some(first_dir_idx) = node.children.map(|c| c.iter().position(|n| n.is_dir)) {
                    self.tree_path.push(first_dir_idx);
                    self.update_table();
                    self.update_breadcrumbs();
                    self.dirty = true;
                }
            } else {
                self.show_toast(&format!("Opening {}...", node.name), ToastKind::Info);
            }
        }
    }

    fn get_selected_tree_node(&self) -> Option<(&'static FileNode, Vec<usize>)> {
        let path = self.tree.get_selected_path();
        if path.is_empty() {
            return None;
        }
        self.fs.find_by_path(path).map(|node| (node, path.to_vec()))
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
        self.dirty = true;
    }
}

impl Widget for FileManager {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        0
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.tree.needs_render() || self.table.needs_render() || self.toast.as_ref().map(|t| t.needs_render()).unwrap_or(false)
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
        self.tree.clear_dirty();
        self.table.clear_dirty();
    }

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
            if i < plane.cells.len() {
                plane.cells[i] = cell.clone();
            }
        }

        let tree_plane = self.tree.render(tree_rect);
        for (i, cell) in tree_plane.cells.iter().enumerate() {
            let base = (header_height * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        for (i, cell) in divider_plane.cells.iter().enumerate() {
            let base = (header_height * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        let detail_plane = self.render_details(detail_rect);
        for (i, cell) in detail_plane.cells.iter().enumerate() {
            let base = (header_height * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        let item_count = self.current_node().children.map(|c| c.len()).unwrap_or(0);
        let selected_text = if self.selected_entry.is_some() { "1 selected" } else { "0 selected" };
        let status_text = format!("{} items | {} | Press ? for shortcuts", item_count, selected_text);
        let status_bar = StatusBar::new(WidgetId::new(10))
            .add_segment(StatusSegment::new(&status_text).with_fg(Color::Rgb(180, 180, 180)).with_bg(Color::Ansi(236)));
        let status_plane = status_bar.render(footer_rect);
        for (i, cell) in status_plane.cells.iter().enumerate() {
            let base = ((area.height - footer_height) * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        if let Some(ref menu) = self.context_menu {
            let menu_plane = menu.render(area);
            for (i, cell) in menu_plane.cells.iter().enumerate() {
                let y = menu_plane.y as usize;
                let x = menu_plane.x as usize;
                let row = i / menu_plane.width as usize;
                let col = i % menu_plane.width as usize;
                let target_idx = ((y + row) * area.width as usize) + x + col;
                if target_idx < plane.cells.len() {
                    plane.cells[target_idx] = cell.clone();
                }
            }
        }

        if let Some(ref toast) = self.toast {
            let toast_plane = toast.render(Rect::new(area.width.saturating_sub(40), area.height - 2, 40, 1));
            for (i, cell) in toast_plane.cells.iter().enumerate() {
                let base = ((area.height - 2) * area.width) as usize;
                let idx = base + (area.width as usize - 40) + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = cell.clone();
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if let Some(ref mut menu) = self.context_menu {
            match key.code {
                KeyCode::Escape => {
                    self.context_menu = None;
                    self.dirty = true;
                    return true;
                }
                KeyCode::Enter => {
                    if let Some((_, action)) = menu.items.first() {
                        self.handle_context_action(action.clone());
                    }
                    return true;
                }
                _ => return false,
            }
        }

        if let Some(ref mut toast) = self.toast {
            if toast.is_expired() {
                self.toast = None;
                self.dirty = true;
            }
            return false;
        }

        match key.code {
            KeyCode::Backspace => {
                self.go_up();
                true
            }
            KeyCode::Enter => {
                self.open_selected();
                true
            }
            KeyCode::Char('c') => {
                if let Some((x, y)) = self.tree.get_selected_path().last().map(|_| (30u16, 10u16)) {
                    self.show_context_menu(x, y);
                }
                true
            }
            KeyCode::Escape => {
                self.selected_entry = None;
                self.dirty = true;
                true
            }
            _ => {
                if self.tree.handle_key(key.clone()) {
                    self.tree_path = self.tree.get_selected_path().to_vec();
                    self.update_breadcrumbs();
                    self.dirty = true;
                    return true;
                }
                if self.table.handle_key(key) {
                    if let Some(entry) = self.table.get_selected() {
                        self.selected_entry = Some(entry.clone());
                    }
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
                if let MouseEventKind::Down(Button::Left) = kind {
                    let menu_row = (row - menu.anchor_y) as usize;
                    if menu_row < menu.items.len() {
                        if let Some((_, action)) = menu.items.get(menu_row) {
                            if *action != ContextAction::Separator {
                                self.handle_context_action(action.clone());
                            }
                        }
                    } else {
                        self.context_menu = None;
                    }
                }
                return true;
            }
            if let MouseEventKind::Down(Button::Right) = kind {
                self.context_menu = None;
                self.show_context_menu(col, row);
                return true;
            }
        }

        if let Some(ref mut toast) = self.toast {
            if toast.is_expired() {
                self.toast = None;
                self.dirty = true;
            }
        }

        let header_height = 1u16;
        let footer_height = 1u16;
        let content_height = 24u16.saturating_sub(header_height + footer_height);

        if row == 0 {
            return self.breadcrumbs.handle_mouse(kind, col, row);
        }

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.30);
        let (tree_rect, detail_rect) = split.split(Rect::new(0, header_height, 80, content_height));

        if col <= tree_rect.width && row > header_height && row < header_height + tree_rect.height {
            if self.tree.handle_mouse(kind, col, row - header_height) {
                self.tree_path = self.tree.get_selected_path().to_vec();
                self.update_table();
                self.update_breadcrumbs();
                self.dirty = true;
                return true;
            }
        }

        if col > tree_rect.width && row > header_height && row < header_height + detail_rect.height {
            if self.table.handle_mouse(kind, col - tree_rect.width - 1, row - header_height) {
                if let Some(entry) = self.table.get_selected() {
                    self.selected_entry = Some(entry.clone());
                }
                self.dirty = true;
                return true;
            }
        }

        if let MouseEventKind::Down(Button::Right) = kind {
            self.show_context_menu(col, row);
            return true;
        }

        false
    }
}

impl FileManager {
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
        y += 1;

        if let Some(ref entry) = self.selected_entry {
            print_line(&mut plane, y, &format!("Name: {}", entry.name), Color::Rgb(255, 255, 255), false);
            y += 1;
            print_line(&mut plane, y, &format!("Size: {}", entry.size), Color::Rgb(200, 150, 100), false);
            y += 1;
            print_line(&mut plane, y, &format!("Modified: {}", entry.modified), Color::Rgb(180, 180, 180), false);
            y += 1;
            print_line(&mut plane, y, "Permissions: rw-r--r--", Color::Rgb(180, 180, 180), false);
            y += 2;

            if entry.name.ends_with(".toml") || entry.name.ends_with(".md") || entry.name.ends_with(".rs") {
                print_line(&mut plane, y, "Preview:", Color::Rgb(0, 255, 136), true);
                y += 1;

                let preview_lines = if entry.name == "Cargo.toml" {
                    vec![
                        "[package]",
                        &format!("name = \"{}\"", "dracon-terminal-engine"),
                        "version = \"27.0.5\"",
                    ]
                } else {
                    vec![
                        &format!("# {}", entry.name),
                        "",
                        "Content preview...",
                    ]
                };

                for line in preview_lines {
                    print_line(&mut plane, y, line, Color::Rgb(150, 150, 150), false);
                    y += 1;
                    if y >= area.height - 1 {
                        break;
                    }
                }
            }
        } else {
            print_line(&mut plane, y, "Select a file to view details", Color::Rgb(100, 100, 100), false);
            y += 2;
            let hint = "Use tree to navigate.";
            print_line(&mut plane, y, hint, Color::Rgb(80, 80, 80), false);
        }

        plane
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
            fm.update_table();
            let plane = fm.render(area);
            ctx.add_plane(plane);
        })
}