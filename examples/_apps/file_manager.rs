#![allow(missing_docs)]
//! File Manager — real filesystem browser with Tree, SplitPane, and Breadcrumbs.
//!
//! Reads actual directory contents from the current working directory.
//!
//! Controls:
//!   ↑/↓ or j/k    — navigate tree
//!   Enter or →    — expand directory / open file
//!   Backspace or ← — go up
//!   c             — context menu
//!   r             — refresh directory
//!   ?             — help overlay
//!   q             — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, ContextMenu, SplitPane, StatusBar, StatusSegment, Toast, ToastKind, Tree, TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct FsNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    children: Vec<FsNode>,
}

impl FsNode {
    fn icon(&self) -> &'static str {
        if self.is_dir { "📁 " } else { "📄 " }
    }

    fn to_tree_node(&self) -> TreeNode {
        let label = format!("{}{}", self.icon(), self.name);
        let mut node = TreeNode::new(&label);
        node.expanded = !self.children.is_empty();
        for child in &self.children {
            node.add_child(child.to_tree_node());
        }
        node
    }

    fn read_dir(path: &PathBuf) -> Option<Vec<FsNode>> {
        let entries = std::fs::read_dir(path).ok()?;
        let mut nodes: Vec<FsNode> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let path = e.path();
                let meta = e.metadata().ok();
                let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                FsNode { name, path, is_dir, size, children: Vec::new() }
            })
            .collect();
        nodes.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        Some(nodes)
    }

    fn build_tree(path: &PathBuf, depth: usize) -> FsNode {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();
        let is_dir = path.is_dir();
        let meta = std::fs::metadata(path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);

        let children = if is_dir && depth < 3 {
            Self::read_dir(path).unwrap_or_default()
                .into_iter()
                .map(|child| Self::build_tree(&child.path, depth + 1))
                .collect()
        } else {
            Vec::new()
        };

        FsNode { name, path: path.clone(), is_dir, size, children }
    }

    fn find_by_path(&self, path: &[usize]) -> Option<&FsNode> {
        if path.is_empty() { return Some(self); }
        let idx = path[0];
        if idx >= self.children.len() { return None; }
        self.children[idx].find_by_path(&path[1..])
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }
}

struct FileManager {
    id: WidgetId,
    root: FsNode,
    tree: Tree,
    breadcrumbs: Breadcrumbs,
    tree_path: Vec<usize>,
    selected_path: Option<PathBuf>,
    context_menu: Option<ContextMenu>,
    toast: Option<Toast>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
    theme: Theme,
    show_help: bool,
    last_click: Instant,
}

impl FileManager {
    fn new(id: WidgetId, should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FsNode::build_tree(&cwd, 0);

        let tree = Tree::new(WidgetId::new(2))
            .with_root(vec![root.to_tree_node()])
            .with_theme(theme);

        let breadcrumbs = Breadcrumbs::new_with_id(
            WidgetId::new(3),
            cwd.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect(),
        );

        Self {
            id,
            root,
            tree,
            breadcrumbs,
            tree_path: Vec::new(),
            selected_path: None,
            context_menu: None,
            toast: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            should_quit,
            theme,
            show_help: false,
            last_click: Instant::now(),
        }
    }

    fn refresh(&mut self) {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        self.root = FsNode::build_tree(&cwd, 0);
        self.tree = Tree::new(WidgetId::new(2))
            .with_root(vec![self.root.to_tree_node()])
            .with_theme(self.theme);
        self.dirty = true;
        self.toast("Directory refreshed", ToastKind::Info);
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        self.toast = Some(Toast::new(WidgetId::new(100), msg)
            .with_kind(kind)
            .with_duration(Duration::from_secs(2))
            .with_theme(self.theme));
        self.dirty = true;
    }

    fn update_breadcrumbs(&mut self) {
        let path = self.selected_path.as_ref()
            .or_else(|| Some(&self.root.path))
            .unwrap();
        let segments: Vec<String> = path.components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        self.breadcrumbs = Breadcrumbs::new_with_id(WidgetId::new(3), segments);
        self.breadcrumbs.on_theme_change(&self.theme);
    }
}

impl Widget for FileManager {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); self.dirty = true; }
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

        let hh = 1u16;
        let fh = 1u16;
        let content_h = area.height.saturating_sub(hh + fh);

        // Breadcrumbs
        let bc_plane = self.breadcrumbs.render(Rect::new(0, 0, area.width, hh));
        for (i, c) in bc_plane.cells.iter().enumerate() {
            if !c.transparent && i < plane.cells.len() {
                plane.cells[i] = c.clone();
            }
        }

        // Split pane
        let split = SplitPane::new(Orientation::Horizontal).ratio(0.35);
        let (tree_rect, detail_rect) = split.split(Rect::new(0, hh, area.width, content_h));

        // Tree
        let tree_plane = self.tree.render(tree_rect);
        for (i, c) in tree_plane.cells.iter().enumerate() {
            if c.transparent { continue; }
            let row = i / tree_plane.width as usize;
            let col = i % tree_plane.width as usize;
            let idx = (hh + row as u16) * area.width + col as u16;
            if (idx as usize) < plane.cells.len() {
                plane.cells[idx as usize] = c.clone();
            }
        }

        // Separator
        for y in hh..hh + content_h {
            let idx = (y * area.width + tree_rect.width) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Detail pane
        if let Some(ref path) = self.selected_path {
            if let Some(node) = self.root.find_by_path(&self.tree_path) {
                let dx = tree_rect.width + 2;
                let dy = hh + 1;

                draw_text(&mut plane, dx, dy, &node.name, t.primary, t.bg, true);
                draw_text(&mut plane, dx, dy + 2, &format!("Type: {}", if node.is_dir { "Directory" } else { "File" }), t.fg, t.bg, false);

                if !node.is_dir {
                    let size_str = format_size(node.size);
                    draw_text(&mut plane, dx, dy + 3, &format!("Size: {}", size_str), t.fg, t.bg, false);
                }

                if let Ok(meta) = std::fs::metadata(&node.path) {
                    if let Ok(modified) = meta.modified() {
                        let time = format_system_time(modified);
                        draw_text(&mut plane, dx, dy + 4, &format!("Modified: {}", time), t.fg_muted, t.bg, false);
                    }
                }
            }
        } else {
            let dx = tree_rect.width + 2;
            let dy = hh + content_h / 2;
            let msg = "Select a file or folder";
            draw_text(&mut plane, dx, dy, msg, t.fg_muted, t.bg, false);
        }

        // Status bar
        let status_y = area.height.saturating_sub(1);
        let status = StatusBar::new(WidgetId::new(4))
            .add_segment(StatusSegment::new(&format!("{} items", self.root.child_count())).with_fg(t.fg_muted))
            .add_segment(StatusSegment::new("? help | r refresh | q quit").with_fg(t.primary));
        let status_plane = status.render(Rect::new(0, status_y, area.width, fh));
        for (i, c) in status_plane.cells.iter().enumerate() {
            if !c.transparent && i < plane.cells.len() {
                let base = (status_y * area.width) as usize;
                if base + i < plane.cells.len() {
                    plane.cells[base + i] = c.clone();
                }
            }
        }

        // Help overlay
        if self.show_help {
            let help_w = 35u16.min(area.width - 4);
            let help_h = 10u16.min(area.height - 4);
            let help_x = (area.width - help_w) / 2;
            let help_y = (area.height - help_h) / 2;

            for y in help_y..help_y + help_h {
                for x in help_x..help_x + help_w {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].fg = t.fg;
                    }
                }
            }

            let lines = [
                "  File Manager Help  ",
                "",
                "↑/↓ j/k  Navigate",
                "Enter    Open/Expand",
                "Bksp     Go up",
                "c        Context menu",
                "r        Refresh",
                "q        Quit",
                "",
                "  Press any key...  ",
            ];
            for (i, line) in lines.iter().enumerate() {
                let y = help_y + 1 + i as u16;
                draw_text(&mut plane, help_x + 2, y, line, t.primary, t.surface_elevated, i == 0 || i == lines.len() - 1);
            }
        }

        // Toast
        if let Some(ref toast) = self.toast {
            if !toast.is_expired() {
                let toast_plane = toast.render(Rect::new(2, status_y.saturating_sub(1), area.width.saturating_sub(4), 1));
                for (i, c) in toast_plane.cells.iter().enumerate() {
                    if !c.transparent && i < plane.cells.len() {
                        let base = ((status_y.saturating_sub(1)) * area.width + 2) as usize;
                        if base + i < plane.cells.len() {
                            plane.cells[base + i] = c.clone();
                        }
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            self.show_help = false;
            self.dirty = true;
            return true;
        }

        if self.context_menu.is_some() {
            if key.code == KeyCode::Esc {
                self.context_menu = None;
                self.dirty = true;
            }
            return true;
        }

        match key.code {
            KeyCode::Char('q') => { self.should_quit.store(true, Ordering::SeqCst); true }
            KeyCode::Char('r') => { self.refresh(); true }
            KeyCode::Char('?') => { self.show_help = true; self.dirty = true; true }
            KeyCode::Char('c') => {
                self.context_menu = Some(ContextMenu::new_with_id(WidgetId::new(50), vec![
                    ("Open", ContextAction::Open),
                    ("Refresh", ContextAction::Copy),
                    ("Properties", ContextAction::Edit),
                ]));
                self.dirty = true;
                true
            }
            KeyCode::Enter | KeyCode::Right => {
                let path = self.tree.get_selected_path().to_vec();
                if let Some(node) = self.root.find_by_path(&path) {
                    self.selected_path = Some(node.path.clone());
                    self.tree_path = path;
                    self.update_breadcrumbs();
                    if node.is_dir {
                        // Toggle expansion
                    }
                }
                self.dirty = true;
                true
            }
            KeyCode::Backspace | KeyCode::Left => {
                self.selected_path = None;
                self.tree_path.clear();
                self.dirty = true;
                true
            }
            _ => {
                if self.tree.handle_key(key) {
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.show_help {
            self.show_help = false;
            self.dirty = true;
            return true;
        }

        let hh = 1u16;
        let area = self.area.get();
        let ch = area.height.saturating_sub(hh + 1);

        if row == 0 { return self.breadcrumbs.handle_mouse(kind, col, row); }

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.35);
        let (tree_rect, _) = split.split(Rect::new(0, hh, area.width, ch));

        if col < tree_rect.width && row >= hh && row < hh + ch {
            return self.tree.handle_mouse(kind, col, row - hh);
        }
        false
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

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_system_time(time: std::time::SystemTime) -> String {
    let dur = time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let day = secs / 86400;
    format!("{}d {:02}:{:02}", day, hours, mins)
}

fn main() -> std::io::Result<()> {
    println!("File Manager — Real filesystem browser");
    println!("? help | r refresh | q quit");
    std::thread::sleep(Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::nord();
    let fm = FileManager::new(WidgetId::new(1), should_quit, theme);

    let mut app = App::new()?.title("File Manager").fps(30).theme(theme);
    app.add_widget(Box::new(fm), Rect::new(0, 0, w, h));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) { ctx.stop(); }
    }).run(|_ctx| {});

    println!("\nFile manager exited cleanly");
    Ok(())
}
