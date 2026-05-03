#![allow(missing_docs)]
//! File Manager — real filesystem browser with Tree, SplitPane, and Breadcrumbs.
//!
//! Reads actual directory contents from the current working directory.
//!
//! Controls:
//!   ↑/↓           — navigate tree
//!   Enter or →    — expand directory / open file
//!   Backspace or ← — go up to parent directory
//!   c             — context menu
//!   r             — refresh directory
//!   ?             — help overlay
//!   q             — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, ContextAction, ContextMenu, SplitPane, StatusBar, StatusSegment, Toast, ToastKind,
    Tree, TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

struct FsNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    children: Vec<FsNode>,
}

impl FsNode {
    fn icon(&self, expanded: bool) -> &'static str {
        if self.is_dir {
            if expanded {
                "▾ "
            } else {
                "▸ "
            }
        } else {
            "  "
        }
    }

    fn file_symbol(&self) -> &'static str {
        if self.is_dir {
            return "▸";
        }
        let name_lower = self.name.to_lowercase();
        if name_lower.ends_with(".rs") {
            ""
        } else if name_lower.ends_with(".toml") {
            "�"
        } else if name_lower.ends_with(".md") {
            ""
        } else if name_lower.ends_with(".json")
            || name_lower.ends_with(".yaml")
            || name_lower.ends_with(".yml")
        {
            "�"
        } else if name_lower.ends_with(".sh") || name_lower.ends_with(".bash") {
            ""
        } else if name_lower.ends_with(".py") {
            ""
        } else if name_lower.ends_with(".js") || name_lower.ends_with(".ts") {
            ""
        } else if name_lower.ends_with(".html") || name_lower.ends_with(".css") {
            ""
        } else if name_lower.ends_with(".gitignore") || name_lower.ends_with(".lock") {
            "﬍"
        } else {
            ""
        }
    }

    fn to_tree_node(&self, expanded: bool) -> TreeNode {
        let label = format!("{}{}", self.icon(expanded), self.name);
        let mut node = TreeNode::new(&label);
        node.expanded = expanded;
        for child in &self.children {
            node.add_child(child.to_tree_node(expanded));
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
                FsNode {
                    name,
                    path,
                    is_dir,
                    size,
                    children: Vec::new(),
                }
            })
            .collect();
        nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        Some(nodes)
    }

    fn build_tree(path: &PathBuf, depth: usize) -> FsNode {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();
        let is_dir = path.is_dir();
        let meta = std::fs::metadata(path).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);

        let children = if is_dir && depth < 3 {
            Self::read_dir(path)
                .unwrap_or_default()
                .into_iter()
                .map(|child| Self::build_tree(&child.path, depth + 1))
                .collect()
        } else {
            Vec::new()
        };

        FsNode {
            name,
            path: path.clone(),
            is_dir,
            size,
            children,
        }
    }

    fn find_by_path(&self, path: &[usize]) -> Option<&FsNode> {
        if path.is_empty() {
            return Some(self);
        }
        let idx = path[0];
        if idx >= self.children.len() {
            return None;
        }
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
    split: SplitPane,
    tree_path: Vec<usize>,
    selected_path: Option<PathBuf>,
    context_menu: Option<ContextMenu>,
    toast: Option<Toast>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
    theme: Theme,
    show_help: bool,
    is_dragging_split: bool,
}

impl FileManager {
    fn new(id: WidgetId, should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root = FsNode::build_tree(&cwd, 0);

        let tree = Tree::new(WidgetId::new(2))
            .with_root(vec![root.to_tree_node(true)])
            .with_theme(theme);

        let segments: Vec<String> = cwd
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        let breadcrumbs = Breadcrumbs::new_with_id(WidgetId::new(3), segments).with_theme(theme);

        let mut fm = Self {
            id,
            root,
            tree,
            breadcrumbs,
            split: SplitPane::new(Orientation::Horizontal)
                .ratio(0.35)
                .with_divider('┃'),
            tree_path: Vec::new(),
            selected_path: None,
            context_menu: None,
            toast: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            should_quit,
            theme,
            show_help: false,
            is_dragging_split: false,
        };
        fm.update_breadcrumbs();
        fm
    }

    fn refresh(&mut self) {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        self.root = FsNode::build_tree(&cwd, 0);
        self.tree = Tree::new(WidgetId::new(2))
            .with_root(vec![self.root.to_tree_node(true)])
            .with_theme(self.theme);
        self.dirty = true;
        self.toast("Directory refreshed", ToastKind::Info);
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        self.toast = Some(
            Toast::new(WidgetId::new(100), msg)
                .with_kind(kind)
                .with_duration(Duration::from_secs(2))
                .with_theme(self.theme),
        );
        self.dirty = true;
    }

    fn update_breadcrumbs(&mut self) {
        let path = self
            .selected_path
            .as_ref()
            .unwrap_or(&self.root.path);
        let segments: Vec<String> = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        self.breadcrumbs =
            Breadcrumbs::new_with_id(WidgetId::new(3), segments).with_theme(self.theme);
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if let Ok(canonical) = path.canonicalize() {
            std::env::set_current_dir(&canonical).ok();
            self.root = FsNode::build_tree(&canonical, 0);
            self.tree = Tree::new(WidgetId::new(2))
                .with_root(vec![self.root.to_tree_node(true)])
                .with_theme(self.theme);
            self.tree_path.clear();
            self.selected_path = None;
            self.update_breadcrumbs();
            self.dirty = true;
        }
    }

    fn go_up(&mut self) {
        if let Some(parent) = self.root.path.parent() {
            let parent_buf = parent.to_path_buf();
            self.navigate_to(parent_buf);
            self.toast("Navigated up", ToastKind::Info);
        }
    }

    fn preview_file(&self, path: &PathBuf, max_lines: usize) -> Vec<String> {
        if let Ok(content) = std::fs::read_to_string(path) {
            content
                .lines()
                .take(max_lines)
                .map(|s| s.to_string())
                .collect()
        } else {
            vec!["<binary file>".to_string()]
        }
    }

    #[allow(dead_code)]
    fn delete_selected(&mut self) {
        if let Some(ref path) = self.selected_path {
            let msg = if path.is_dir() {
                format!(
                    "Deleted folder: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            } else {
                format!(
                    "Deleted file: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            };
            if std::fs::remove_dir_all(path).is_ok() || std::fs::remove_file(path).is_ok() {
                self.refresh();
                self.toast(&msg, ToastKind::Success);
            } else {
                self.toast("Failed to delete", ToastKind::Error);
            }
        }
    }

    #[allow(dead_code)]
    fn create_file(&mut self, name: &str) {
        let path = self.root.path.join(name);
        if std::fs::File::create(&path).is_ok() {
            self.refresh();
            self.toast(&format!("Created file: {}", name), ToastKind::Success);
        } else {
            self.toast("Failed to create file", ToastKind::Error);
        }
    }

    #[allow(dead_code)]
    fn create_folder(&mut self, name: &str) {
        let path = self.root.path.join(name);
        if std::fs::create_dir(&path).is_ok() {
            self.refresh();
            self.toast(&format!("Created folder: {}", name), ToastKind::Success);
        } else {
            self.toast("Failed to create folder", ToastKind::Error);
        }
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
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
    fn focusable(&self) -> bool {
        true
    }

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
        let (tree_rect, _detail_rect) = self.split.split(Rect::new(0, hh, area.width, content_h));

        // Tree
        let tree_plane = self.tree.render(tree_rect);
        for (i, c) in tree_plane.cells.iter().enumerate() {
            if c.transparent {
                continue;
            }
            let row = i / tree_plane.width as usize;
            let col = i % tree_plane.width as usize;
            let idx = (hh + row as u16) * area.width + col as u16;
            if (idx as usize) < plane.cells.len() {
                plane.cells[idx as usize] = c.clone();
            }
        }

        // Divider
        let divider = self
            .split
            .render_divider(Rect::new(0, hh, area.width, content_h));
        for (i, c) in divider.cells.iter().enumerate() {
            if c.transparent {
                continue;
            }
            let row = i / divider.width as usize;
            let col = i % divider.width as usize;
            let idx = (hh + row as u16 + divider.y) * area.width + (divider.x + col as u16);
            if (idx as usize) < plane.cells.len() {
                plane.cells[idx as usize] = c.clone();
            }
        }

        // Detail pane
        let detail_x = tree_rect.width + 2;
        let detail_w = area.width.saturating_sub(detail_x);
        let detail_h = content_h.saturating_sub(2);

        if detail_w > 0 && detail_h > 0 {
            if let Some(ref _sel_path) = self.selected_path {
                if let Some(node) = self.root.find_by_path(&self.tree_path) {
                    // Detail pane border
                    render_box(
                        &mut plane,
                        detail_x - 1,
                        hh,
                        detail_w + 1,
                        detail_h,
                        t.outline,
                        t.surface_elevated,
                    );

                    let dx = detail_x + 1;
                    let dy = hh + 1;

                    // File icon and name
                    let sym = node.file_symbol();
                    draw_text(&mut plane, dx, dy, sym, t.primary, t.surface_elevated, true);
                    draw_text(
                        &mut plane,
                        dx + 2,
                        dy,
                        &node.name,
                        t.primary,
                        t.surface_elevated,
                        true,
                    );

                    // Metadata
                    let mut meta_y = dy + 2;
                    draw_text(
                        &mut plane,
                        dx,
                        meta_y,
                        &format!("Type: {}", if node.is_dir { "Directory" } else { "File" }),
                        t.fg,
                        t.surface_elevated,
                        false,
                    );
                    meta_y += 1;

                    if !node.is_dir {
                        let size_str = format_size(node.size);
                        draw_text(
                            &mut plane,
                            dx,
                            meta_y,
                            &format!("Size: {}", size_str),
                            t.fg,
                            t.surface_elevated,
                            false,
                        );
                        meta_y += 1;
                    }

                    if let Ok(meta) = std::fs::metadata(&node.path) {
                        if let Ok(modified) = meta.modified() {
                            let time = format_system_time(modified);
                            draw_text(
                                &mut plane,
                                dx,
                                meta_y,
                                &format!("Modified: {}", time),
                                t.fg_muted,
                                t.surface_elevated,
                                false,
                            );
                            meta_y += 1;
                        }
                        let perms = format_permissions(meta.permissions().mode());
                        draw_text(
                            &mut plane,
                            dx,
                            meta_y,
                            &format!("Permissions: {}", perms),
                            t.fg_muted,
                            t.surface_elevated,
                            false,
                        );
                        meta_y += 1;
                    }

                    // File preview for text files
                    if !node.is_dir && detail_h > 10 {
                        let preview_y = meta_y + 2;
                        if preview_y + 3 < hh + detail_h {
                            draw_text(
                                &mut plane,
                                dx,
                                preview_y,
                                "Preview:",
                                t.info,
                                t.surface_elevated,
                                true,
                            );
                            let preview = self
                                .preview_file(&node.path, (hh + detail_h - preview_y - 1) as usize);
                            for (i, line) in preview.iter().enumerate() {
                                let y = preview_y + 1 + i as u16;
                                if y >= hh + detail_h {
                                    break;
                                }
                                let truncated: String =
                                    line.chars().take(detail_w as usize - 4).collect();
                                draw_text(
                                    &mut plane,
                                    dx,
                                    y,
                                    &truncated,
                                    t.fg,
                                    t.surface_elevated,
                                    false,
                                );
                            }
                        }
                    }
                }
            } else {
                // Empty state with icon
                let cx = detail_x + detail_w / 2;
                let cy = hh + content_h / 2;
                let msg = "Select a file or folder";
                let mx = cx.saturating_sub((msg.width() as u16) / 2);
                draw_text(&mut plane, mx, cy, msg, t.fg_muted, t.bg, false);
                let sub = "↑/↓ to navigate  •  Enter to select";
                let sx = cx.saturating_sub((sub.width() as u16) / 2);
                draw_text(&mut plane, sx, cy + 1, sub, t.fg_muted, t.bg, false);
            }
        }

        // Status bar
        let status_y = area.height.saturating_sub(1);
        let status = StatusBar::new(WidgetId::new(4))
            .add_segment(
                StatusSegment::new(&format!("{} items", self.root.child_count()))
                    .with_fg(t.fg_muted),
            )
            .add_segment(
                StatusSegment::new("? help | c context | r refresh | q quit").with_fg(t.primary),
            );
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
            render_help_overlay(&mut plane, area, t);
        }

        // Toast
        if let Some(ref toast) = self.toast {
            if !toast.is_expired() {
                let toast_plane = toast.render(Rect::new(
                    2,
                    status_y.saturating_sub(1),
                    area.width.saturating_sub(4),
                    1,
                ));
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
        if key.kind != KeyEventKind::Press {
            return false;
        }

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
            KeyCode::Char('q') => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('r') => {
                self.refresh();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                self.dirty = true;
                true
            }
            KeyCode::Char('c') => {
                self.context_menu = Some(ContextMenu::new_with_id(
                    WidgetId::new(50),
                    vec![
                        ("New Folder", ContextAction::Open),
                        ("New File", ContextAction::Copy),
                        ("Delete", ContextAction::Cut),
                        ("Refresh", ContextAction::Edit),
                    ],
                ));
                self.dirty = true;
                true
            }
            KeyCode::Enter | KeyCode::Right => {
                let path = self.tree.get_selected_path().to_vec();
                let node_info = self
                    .root
                    .find_by_path(&path)
                    .map(|n| (n.path.clone(), n.is_dir, n.name.clone()));
                if let Some((node_path, is_dir, name)) = node_info {
                    if is_dir {
                        self.navigate_to(node_path);
                        self.toast(&format!("Opened: {}", name), ToastKind::Info);
                    } else {
                        self.selected_path = Some(node_path);
                        self.tree_path = path;
                        self.update_breadcrumbs();
                        self.dirty = true;
                    }
                }
                true
            }
            KeyCode::Backspace | KeyCode::Left => {
                self.go_up();
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

        // Breadcrumb clicks — delegate to Breadcrumbs widget
        if row == 0 {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                // Try to find which segment was clicked using Breadcrumbs' zone logic
                let segs: Vec<String> = self
                    .root
                    .path
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                let mut x: u16 = 0;
                for (i, seg) in segs.iter().enumerate() {
                    let seg_width = (seg.width() as u16 + 2).min(area.width.saturating_sub(x));
                    if seg_width < 3 {
                        break;
                    }
                    // Zone covers the whole segment area [x, x+seg_width)
                    if col >= x && col < x + seg_width {
                        let components: Vec<_> = self.root.path.components().collect();
                        let target_path: PathBuf = components[..=i].iter().collect();
                        self.navigate_to(target_path);
                        return true;
                    }
                    if i > 0 {
                        x += 1;
                    } // separator
                    x += seg_width;
                }
            }
            return true;
        }

        // Split pane drag resize
        let divider_rect = self.split.divider_rect(Rect::new(0, hh, area.width, ch));
        if col >= divider_rect.x
            && col < divider_rect.x + divider_rect.width
            && row >= divider_rect.y
            && row < divider_rect.y + divider_rect.height
        {
            match kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    self.is_dragging_split = true;
                    return true;
                }
                MouseEventKind::Drag(_) if self.is_dragging_split => {
                    if self
                        .split
                        .handle_resize(kind, col, row, Rect::new(0, hh, area.width, ch))
                    {
                        self.dirty = true;
                    }
                    return true;
                }
                MouseEventKind::Up(_) if self.is_dragging_split => {
                    self.is_dragging_split = false;
                    self.dirty = true;
                    return true;
                }
                _ => {}
            }
        }
        if self.is_dragging_split && matches!(kind, MouseEventKind::Up(_)) {
            self.is_dragging_split = false;
            self.dirty = true;
            return true;
        }
        if self.is_dragging_split {
            if let MouseEventKind::Drag(_) = kind {
                if self
                    .split
                    .handle_resize(kind, col, row, Rect::new(0, hh, area.width, ch))
                {
                    self.dirty = true;
                }
                return true;
            }
        }

        // Tree pane click
        let (tree_rect, _) = self.split.split(Rect::new(0, hh, area.width, ch));
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

fn render_box(
    plane: &mut Plane,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    border_color: Color,
    bg_color: Color,
) {
    if w < 2 || h < 2 {
        return;
    }
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * plane.width + col) as usize;
            if idx < plane.cells.len() {
                let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
                plane.cells[idx].bg = bg_color;
                plane.cells[idx].fg = border_color;
                if is_border {
                    if row == y && col == x {
                        plane.cells[idx].char = '╭';
                    } else if row == y && col == x + w - 1 {
                        plane.cells[idx].char = '╮';
                    } else if row == y + h - 1 && col == x {
                        plane.cells[idx].char = '╰';
                    } else if row == y + h - 1 && col == x + w - 1 {
                        plane.cells[idx].char = '╯';
                    } else if row == y || row == y + h - 1 {
                        plane.cells[idx].char = '─';
                    } else {
                        plane.cells[idx].char = '│';
                    }
                } else {
                    plane.cells[idx].char = ' ';
                }
                plane.cells[idx].transparent = false;
            }
        }
    }
}

fn render_help_overlay(plane: &mut Plane, area: Rect, t: Theme) {
    let help_w = 40u16.min(area.width - 4);
    let help_h = 12u16.min(area.height - 4);
    let help_x = (area.width - help_w) / 2;
    let help_y = (area.height - help_h) / 2;

    render_box(
        plane,
        help_x,
        help_y,
        help_w,
        help_h,
        t.outline,
        t.surface_elevated,
    );

    let lines = [
        ("  File Manager Help  ", true),
        ("", false),
        ("↑/↓       Navigate tree", false),
        ("Enter     Open folder / Select file", false),
        ("Bksp      Go to parent directory", false),
        ("c         Context menu", false),
        ("r         Refresh directory", false),
        ("q         Quit", false),
        ("", false),
        ("  Click breadcrumbs to navigate  ", false),
        ("  Drag divider to resize panels  ", false),
    ];
    for (i, (line, bold)) in lines.iter().enumerate() {
        let y = help_y + 1 + i as u16;
        let x = help_x + (help_w.saturating_sub(line.width() as u16)) / 2;
        draw_text(
            plane,
            x,
            y,
            line,
            if *bold { t.primary } else { t.fg },
            t.surface_elevated,
            *bold,
        );
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
    let dur = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let day = secs / 86400;
    format!("{}d {:02}:{:02}", day, hours, mins)
}

fn format_permissions(mode: u32) -> String {
    let perms = [
        if mode & 0o400 != 0 { 'r' } else { '-' },
        if mode & 0o200 != 0 { 'w' } else { '-' },
        if mode & 0o100 != 0 { 'x' } else { '-' },
        if mode & 0o040 != 0 { 'r' } else { '-' },
        if mode & 0o020 != 0 { 'w' } else { '-' },
        if mode & 0o010 != 0 { 'x' } else { '-' },
        if mode & 0o004 != 0 { 'r' } else { '-' },
        if mode & 0o002 != 0 { 'w' } else { '-' },
        if mode & 0o001 != 0 { 'x' } else { '-' },
    ];
    perms.iter().collect()
}

fn main() -> std::io::Result<()> {
    println!("File Manager — Real filesystem browser");
    println!("? help | c context | r refresh | q quit");
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
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(|_ctx| {})?;

    println!("\nFile manager exited cleanly");
    Ok(())
}
