#![allow(missing_docs)]
//! File Manager UI — demonstrates Tree + Table + Breadcrumbs + StatusBar + SplitPane + ContextMenu.
//!
//! Layout:
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
//! └─ README.md      │                                       │
//! ├─────────────────┴───────────────────────────────────────┤
//! │ 8 items | 1 selected | Press ? for shortcuts            │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! Navigation: Up/Down=select, Enter=open, Backspace=up, c=context menu, Right-click=context menu

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, ContextAction, ContextMenu, SplitPane, StatusBar, StatusSegment, Toast, ToastKind, Tree, TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)] struct MockFs { name: &'static str, children: Option<Vec<MockFs>>, is_dir: bool }
impl MockFs {
    fn icon(&self) -> &'static str { if self.is_dir { "📁" } else { "📄" } }
    fn to_tree_node(&self) -> TreeNode {
        let label = format!("{}{}", self.icon(), self.name);
        let mut node = TreeNode::new(&label);
        if let Some(ref c) = self.children { for ch in c { node.add_child(ch.to_tree_node()); } }
        node
    }
    fn find_by_path(&self, path: &[usize]) -> Option<&MockFs> {
        if path.is_empty() { return Some(self); }
        let c = self.children.as_ref()?; let idx = path[0];
        if idx >= c.len() { return None; } c[idx].find_by_path(&path[1..])
    }
    fn child_count(&self) -> usize { self.children.as_ref().map(|c| c.len()).unwrap_or(0) }
}

#[derive(Clone)] struct FileEntry { name: String, _is_dir: bool }
struct FileManager {
    id: WidgetId, fs: MockFs, tree: Tree, breadcrumbs: Breadcrumbs,
    tree_path: Vec<usize>, selected: Option<FileEntry>,
    context_menu: Option<ContextMenu>, toast: Option<Toast>, area: std::cell::Cell<Rect>, dirty: bool,
    should_quit: Arc<AtomicBool>, theme: Theme,
}

impl FileManager {
    fn new(id: WidgetId, should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let fs = MockFs { name: "root", is_dir: true, children: Some(vec![
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
        ])};

        let tree = Tree::new(WidgetId::new(1)).with_root(vec![fs.to_tree_node()]);
        let breadcrumbs = Breadcrumbs::new_with_id(WidgetId::new(3),
            vec!["~".into(), "projects".into(), "dracon-terminal-engine".into()]);

        Self { id, fs, tree, breadcrumbs, tree_path: Vec::new(), selected: None,
               context_menu: None, toast: None, area: std::cell::Cell::new(Rect::new(0,0,80,24)), dirty: true, should_quit, theme }
    }

    fn current_node(&self) -> &MockFs { self.fs.find_by_path(&self.tree_path).unwrap_or(&self.fs) }

    fn go_up(&mut self) {
        if !self.tree_path.is_empty() {
            self.tree_path.pop();
            self.tree.set_selected_path(self.tree_path.clone());
            self.selected = None;
            self.update_breadcrumbs();
        }
    }

    fn open_selection(&mut self) {
        if let Some(path) = self.tree.get_selected_path().last().copied() {
            if let Some(node) = self.current_node().children.as_ref().and_then(|c| c.get(path)) {
                if node.is_dir {
                    self.tree_path.push(path);
                    self.tree.set_selected_path(self.tree_path.clone());
                    self.selected = None;
                    self.update_breadcrumbs();
                } else { self.show_toast(&format!("Opening {}...", node.name), ToastKind::Info); }
            }
        }
    }

    fn show_toast(&mut self, msg: &str, kind: ToastKind) {
        self.toast = Some(Toast::new(WidgetId::new(100), msg).with_kind(kind)); self.dirty = true;
    }

    fn show_context_menu(&mut self, x: u16, y: u16) {
        self.context_menu = Some(ContextMenu::new_with_id(WidgetId::new(50), vec![
            ("Open", ContextAction::Open), ("Copy", ContextAction::Copy), ("Paste", ContextAction::Paste),
            ("Rename", ContextAction::Rename), ("Delete", ContextAction::Delete),
            ("Separator", ContextAction::Separator), ("Properties", ContextAction::Edit),
        ]).with_width(18).with_anchor(x, y));
    }

    fn update_breadcrumbs(&mut self) {
        let mut crumbs = vec!["~".to_string()];
        let mut node: &MockFs = &self.fs;
        for &idx in &self.tree_path {
            if let Some(ref c) = node.children {
                if idx < c.len() { node = &c[idx]; crumbs.push(node.name.to_string()); }
            }
        }
        self.breadcrumbs = Breadcrumbs::new_with_id(WidgetId::new(3), crumbs);
    }

    fn meta_for(&self, name: &str) -> (String, String) {
        match name {
            "main.rs" => ("1.2 KB".into(), "2025-01-10".into()),
            "lib.rs" => ("3.4 KB".into(), "2025-01-12".into()),
            "README.md" => ("4.1 KB".into(), "2025-01-08".into()),
            "CHANGELOG.md" => ("8.7 KB".into(), "2025-01-15".into()),
            "test_main.rs" => ("0.8 KB".into(), "2025-01-05".into()),
            "Cargo.toml" => ("2.3 KB".into(), "2025-01-15".into()),
            ".gitignore" => ("0.1 KB".into(), "2025-01-01".into()),
            _ => ("—".into(), "—".into()),
        }
    }

    fn render_details(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(1, area.width, area.height); plane.z_index = 5;
        for c in plane.cells.iter_mut() { c.bg = t.surface; }
        let pl = |p: &mut Plane, y: u16, txt: &str, fg: Color, b: bool| {
            for (i, ch) in txt.chars().take(area.width as usize - 2).enumerate() {
                let idx = (y * p.width + 1 + i as u16) as usize;
                if idx < p.cells.len() { p.cells[idx].char = ch; p.cells[idx].fg = fg; p.cells[idx].style = if b { Styles::BOLD } else { Styles::empty() }; }
            }
        };
        let mut y = 1u16;
        pl(&mut plane, y, "DETAILS", t.primary, true); y += 1;
        pl(&mut plane, y, &"─".repeat(28), t.outline_variant, false); y += 2;

        if let Some(ref e) = self.selected {
            let (sz, md) = self.meta_for(&e.name);
            pl(&mut plane, y, &format!("Name: {}", e.name), t.fg_on_accent, false); y += 1;
            pl(&mut plane, y, &format!("Size: {}", sz), t.warning, false); y += 1;
            pl(&mut plane, y, &format!("Modified: {}", md), t.fg_muted, false); y += 1;
            pl(&mut plane, y, "Permissions: rw-r--r--", t.fg_muted, false); y += 2;
            if e.name.ends_with(".toml") || e.name.ends_with(".md") || e.name.ends_with(".rs") {
                pl(&mut plane, y, "Preview:", t.primary, true); y += 1;
                let p1 = if e.name == "Cargo.toml" { "[package]" } else { "# preview" };
                let p2 = if e.name == "Cargo.toml" { "name = \"dracon-terminal-engine\"" } else { "content..." };
                let p3 = if e.name == "Cargo.toml" { "version = \"27.0.5\"" } else { "" };
                pl(&mut plane, y, p1, t.fg_subtle, false); y += 1;
                pl(&mut plane, y, p2, t.fg_subtle, false); y += 1;
                pl(&mut plane, y, p3, t.fg_subtle, false);
            }
        } else { pl(&mut plane, y, "Select a file to view details", t.fg_muted, false); }
        plane
    }

    fn render_table(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height); plane.z_index = 10;
        let children = match self.current_node().children { Some(ref c) => c, None => return plane };
        for c in plane.cells.iter_mut() { c.bg = t.surface; }
        for (i, ch) in "Name                  Size      Modified".chars().take(area.width as usize).enumerate() {
            if i < plane.cells.len() { plane.cells[i].char = ch; plane.cells[i].fg = t.primary; plane.cells[i].style = Styles::BOLD; }
        }
        let mut y = 1u16;
        for child in children.iter() {
            if y >= area.height { break; }
            let (sz, md) = self.meta_for(child.name);
            let line = format!("{}{:<20} {:<10} {:<12}", child.icon(), child.name, sz, md);
            let fg = if child.is_dir { t.info } else { t.fg };
            for (j, c) in line.chars().take(area.width as usize).enumerate() {
                let idx = (y * plane.width + j as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = fg; }
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
    fn on_theme_change(&mut self, theme: &Theme) { self.theme = *theme; }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height); plane.z_index = 0;
        for c in plane.cells.iter_mut() { c.bg = t.bg; }

        let hh = 1u16; let fh = 1u16; let ch = area.height.saturating_sub(hh + fh);
        let header_rect = Rect::new(0, 0, area.width, hh);
        let content_rect = Rect::new(0, hh, area.width, ch);
        let footer_rect = Rect::new(0, area.height - fh, area.width, fh);

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.30);
        let (tree_rect, detail_rect) = split.split(content_rect);
        let div_plane = split.render_divider(content_rect);

        let bc_plane = self.breadcrumbs.render(header_rect);
        for (i, c) in bc_plane.cells.iter().enumerate() { if !c.transparent && i < plane.cells.len() { plane.cells[i] = c.clone(); } }

        let tree_plane = self.tree.render(tree_rect);
        for (i, c) in tree_plane.cells.iter().enumerate() { let idx = (hh * area.width) as usize + i; if !c.transparent && idx < plane.cells.len() { plane.cells[idx] = c.clone(); } }
        for (i, c) in div_plane.cells.iter().enumerate() { let idx = (hh * area.width) as usize + i; if !c.transparent && idx < plane.cells.len() { plane.cells[idx] = c.clone(); } }

        let table_plane = self.render_table(detail_rect);
        for (i, c) in table_plane.cells.iter().enumerate() { let idx = (hh * area.width) as usize + i; if !c.transparent && idx < plane.cells.len() { plane.cells[idx] = c.clone(); } }

        let det_plane = self.render_details(Rect::new(detail_rect.x + detail_rect.width/2, detail_rect.y + ch/2, detail_rect.width/2, detail_rect.height/2));
        for (i, c) in det_plane.cells.iter().enumerate() {
            let det_x = i % det_plane.width as usize;
            let det_y = i / det_plane.width as usize;
            let plane_x = detail_rect.x as usize + detail_rect.width as usize / 2 + det_x;
            let plane_y = detail_rect.y as usize + ch as usize / 2 + det_y;
            let base = plane_y * area.width as usize + plane_x;
            if !c.transparent && base < plane.cells.len() { plane.cells[base] = c.clone(); }
        }

        let cnt = self.current_node().child_count();
        let sel_txt = if self.selected.is_some() { "1 selected" } else { "0 selected" };
        let status = StatusBar::new(WidgetId::new(10)).add_segment(StatusSegment::new(&format!("{} items | {} | Press ? for shortcuts", cnt, sel_txt)).with_fg(t.fg_muted).with_bg(t.surface_elevated));
        let st_plane = status.render(footer_rect);
        for (i, c) in st_plane.cells.iter().enumerate() { let idx = ((area.height - fh) * area.width) as usize + i; if !c.transparent && idx < plane.cells.len() { plane.cells[idx] = c.clone(); } }

        if let Some(ref m) = self.context_menu {
            let m_plane = m.render(area);
            for (i, c) in m_plane.cells.iter().enumerate() {
                let y = m_plane.y as usize; let x = m_plane.x as usize;
                let row = i / m_plane.width as usize; let col = i % m_plane.width as usize;
                let idx = ((y + row) * area.width as usize) + x + col;
                if !c.transparent && idx < plane.cells.len() { plane.cells[idx] = c.clone(); }
            }
        }
        if let Some(ref t) = self.toast {
            let t_plane = t.render(Rect::new(area.width.saturating_sub(40), area.height - 2, 40, 1));
            for (i, c) in t_plane.cells.iter().enumerate() { let idx = ((area.height - 2) * area.width) as usize + (area.width as usize - 40) + i; if !c.transparent && idx < plane.cells.len() { plane.cells[idx] = c.clone(); } }
        }
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        if self.context_menu.is_some() { if key.code == KeyCode::Esc || key.code == KeyCode::Enter { self.context_menu = None; self.dirty = true; } return true; }
        if let Some(ref mut t) = self.toast { if t.is_expired() { self.toast = None; self.dirty = true; } return false; }
        match key.code {
            KeyCode::Char('q') => { self.should_quit.store(true, Ordering::SeqCst); true }
            KeyCode::Backspace => { self.go_up(); true }
            KeyCode::Enter => { self.open_selection(); true }
            KeyCode::Char('c') => { self.show_context_menu(30, 10); true }
            _ => {
                if self.tree.handle_key(key) {
                    self.tree_path = self.tree.get_selected_path().to_vec();
                    let mut name = String::new();
                    let mut is_dir = false;
                    if let Some(node) = self.fs.find_by_path(&self.tree_path) {
                        name = node.name.into();
                        is_dir = node.is_dir;
                    }
                    self.selected = Some(FileEntry { name, _is_dir: is_dir });
                    self.dirty = true;
                    true
                } else { false }
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if let Some(ref mut m) = self.context_menu { if m.handle_mouse(kind.clone(), col, row) { if let MouseEventKind::Down(_) = kind { self.context_menu = None; } return true; } }
        if let Some(ref mut t) = self.toast { if t.is_expired() { self.toast = None; self.dirty = true; } }
        let hh = 1u16; let fh = 1u16;
        let area = self.area.get();
        let ch = area.height.saturating_sub(hh + fh);
        if row == 0 { return self.breadcrumbs.handle_mouse(kind, col, row); }
        let split = SplitPane::new(Orientation::Horizontal).ratio(0.30);
        let (tree_rect, detail_rect) = split.split(Rect::new(0, hh, area.width, ch));
        if col < tree_rect.width && row >= hh && row < hh + ch { return self.tree.handle_mouse(kind, col, row - hh); }
        false
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::cyberpunk();
    let mut app = App::new()?.title("File Manager").fps(30);
    app.set_theme(theme);

    let fm = FileManager::new(WidgetId::new(1), should_quit, theme);
    app.add_widget(Box::new(fm), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) { ctx.stop(); }
    }).run(|_ctx| {});

    println!("\nFile manager exited cleanly");
    Ok(())
}