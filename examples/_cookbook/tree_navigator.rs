//! Tree Navigator — hierarchical file browsing with Tree + Breadcrumbs + SplitPane.
//!
//! Demonstrates:
//! - Tree widget with expand/collapse navigation
//! - Breadcrumbs updating with current path
//! - SplitPane with tree on left, detail on right
//! - StatusBar showing item count
//! - Mock filesystem data (no real filesystem access)
//!
//! ## Navigation
//!
//! | Key | Action |
//! |-----|--------|
//! | Up/Down | Move selection in tree |
//! | Right/Enter | Expand folder / enter child |
//! | Left | Collapse folder / go to parent |
//! | Backspace | Go up one level |
//!
//! Mouse: Click to select, click folder to expand

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, SplitPane, StatusBar, Tree, TreeNode};
use ratatui::layout::Rect;

static MOCK_FS: MockNode = MockNode {
    name: "root",
    is_dir: true,
    children: Some(vec![
        MockNode {
            name: "src",
            is_dir: true,
            children: Some(vec![
                MockNode { name: "main.rs", is_dir: false, children: None },
                MockNode { name: "lib.rs", is_dir: false, children: None },
            ]),
        },
        MockNode {
            name: "tests",
            is_dir: true,
            children: Some(vec![
                MockNode { name: "test_main.rs", is_dir: false, children: None },
            ]),
        },
        MockNode { name: "README.md", is_dir: false, children: None },
        MockNode { name: "Cargo.toml", is_dir: false, children: None },
    ]),
};

struct MockNode {
    name: &'static str,
    children: Option<Vec<MockNode>>,
    is_dir: bool,
}

impl MockNode {
    fn to_tree_node(&self) -> TreeNode {
        let mut node = TreeNode::new(self.name);
        if let Some(ref children) = self.children {
            for child in children {
                node.add_child(child.to_tree_node());
            }
        }
        node
    }

    fn find_by_path(&self, path: &[usize]) -> Option<&MockNode> {
        if path.is_empty() {
            return Some(self);
        }
        let children = self.children.as_ref()?;
        let idx = path[0];
        if idx >= children.len() {
            return None;
        }
        children[idx].find_by_path(&path[1..])
    }

    fn child_count(&self) -> usize {
        self.children.as_ref().map(|c| c.len()).unwrap_or(0)
    }

    fn all_items_recursive(&self) -> Vec<&MockNode> {
        let mut items = vec![self];
        if let Some(ref children) = self.children {
            for child in children {
                items.extend(child.all_items_recursive());
            }
        }
        items
    }
}

struct TreeNavState {
    tree: Tree,
    breadcrumbs: Breadcrumbs,
    status_bar: StatusBar,
    current_path: Vec<usize>,
    path_segments: Vec<String>,
}

impl TreeNavState {
    fn new() -> Self {
        let root_node = MOCK_FS.to_tree_node();
        let tree = Tree::new(WidgetId::new(1)).with_root(vec![root_node]);

        Self {
            tree,
            breadcrumbs: Breadcrumbs::new(vec!["home".to_string(), "user".to_string(), "projects".to_string()]),
            status_bar: StatusBar::new(WidgetId::new(3)),
            current_path: Vec::new(),
            path_segments: vec!["home".to_string(), "user".to_string(), "projects".to_string()],
        }
    }

    fn update_path(&mut self) {
        let mut segments = vec!["home".to_string(), "user".to_string(), "projects".to_string()];
        for &idx in &self.current_path {
            if let Some(node) = MOCK_FS.find_by_path(&self.current_path[..]) {
                segments.push(node.name.to_string());
            }
            let _ = idx;
        }
        self.path_segments = segments;
        self.breadcrumbs = Breadcrumbs::new(self.path_segments.clone());
    }

    fn selected_node(&self) -> Option<&MockNode> {
        MOCK_FS.find_by_path(&self.current_path)
    }

    fn parent_path(&self) -> Vec<usize> {
        if self.current_path.is_empty() {
            Vec::new()
        } else {
            self.current_path[..self.current_path.len() - 1].to_vec()
        }
    }

    fn item_count(&self) -> usize {
        if let Some(node) = self.selected_node() {
            node.child_count()
        } else {
            MOCK_FS.child_count()
        }
    }

    fn total_items(&self) -> usize {
        MOCK_FS.all_items_recursive().len()
    }
}

impl Widget for TreeNavState {
    fn id(&self) -> WidgetId {
        WidgetId::new(0)
    }

    fn set_id(&mut self, id: WidgetId) {}

    fn area(&self) -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    fn set_area(&mut self, area: Rect) {}

    fn z_index(&self) -> u16 {
        0
    }

    fn needs_render(&self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {}

    fn clear_dirty(&mut self) {}

    fn focusable(&self) -> bool {
        true
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 0;

        for cell in plane.cells.iter_mut() {
            cell.bg = Color::Ansi(17);
        }

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.35);
        let (tree_rect, detail_rect) = split.split(Rect::new(0, 0, area.width, area.height - 1));

        let bc_plane = self.breadcrumbs.render(Rect::new(0, 0, area.width, 1));
        for (i, cell) in bc_plane.cells.iter().enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        let tree_plane = self.tree.render(tree_rect);
        let mut tree_plane = tree_plane;
        tree_plane.set_z_index(10);
        for (i, cell) in tree_plane.cells.iter().enumerate() {
            let idx = (1 * area.width) as usize + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        let detail_plane = render_detail_panel(self, detail_rect);
        for (i, cell) in detail_plane.cells.iter().enumerate() {
            let base_y = (tree_rect.height + 1) as usize;
            let idx = (base_y * area.width as usize) + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        let status_rect = Rect::new(0, area.height - 1, area.width, 1);
        let status_plane = self.status_bar.render(status_rect);
        for (i, cell) in status_plane.cells.iter().enumerate() {
            let idx = ((area.height - 1) * area.width) as usize + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Backspace => {
                if !self.current_path.is_empty() {
                    self.current_path.pop();
                    self.update_path();
                    return true;
                }
            }
            _ => {}
        }

        if self.tree.handle_key(key.clone()) {
            let sel = self.tree.selected_path.clone();
            self.current_path = sel;
            self.update_path();
            return true;
        }

        false
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        if row == 0 {
            return self.breadcrumbs.handle_mouse(kind, col, row);
        }

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.35);
        let (tree_rect, _) = split.split(Rect::new(0, 0, 80, 23));

        if col < tree_rect.width && row > 0 && row < tree_rect.height {
            if self.tree.handle_mouse(kind, col, row - 1) {
                let sel = self.tree.selected_path.clone();
                self.current_path = sel;
                self.update_path();
                return true;
            }
        }

        false
    }
}

fn render_detail_panel(state: &TreeNavState, area: Rect) -> Plane {
    let mut plane = Plane::new(1, area.width, area.height);
    plane.z_index = 5;

    for cell in plane.cells.iter_mut() {
        cell.bg = Color::Ansi(17);
    }

    let mut y = 1u16;

    let print = |plane: &mut Plane, y: u16, text: &str, fg: Color| {
        for (i, c) in text.chars().take(area.width as usize - 2).enumerate() {
            let idx = (y * plane.width + 1 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = fg;
            }
        }
    };

    print(plane, y, "DETAILS", Color::Rgb(0, 255, 136));
    y += 2;

    if let Some(node) = state.selected_node() {
        let icon = if node.is_dir { "[DIR]" } else { "[FILE]" };
        print(plane, y, &format!("{} {}", icon, node.name), Color::Rgb(255, 255, 255));
        y += 1;

        if node.is_dir {
            if let Some(ref children) = node.children {
                print(plane, y, &format!("{} items", children.len()), Color::Rgb(150, 150, 150));
                y += 2;

                print(plane, y, "Contents:", Color::Rgb(100, 180, 255));
                y += 1;

                for child in children {
                    let child_icon = if child.is_dir { "[DIR]" } else { "[FILE]" };
                    let child_name = format!("  {} {}", child_icon, child.name);
                    let fg = if child.is_dir {
                        Color::Rgb(100, 200, 255)
                    } else {
                        Color::Rgb(200, 200, 200)
                    };
                    print(plane, y, &child_name, fg);
                    y += 1;
                    if y >= area.height - 1 {
                        break;
                    }
                }
            }
        } else {
            print(plane, y, "Type: Source file", Color::Rgb(150, 150, 150));
            y += 1;
            print(plane, y, "Size: ~1KB (mock)", Color::Rgb(150, 150, 150));
        }
    } else {
        print(plane, y, "No selection", Color::Rgb(150, 150, 150));
    }

    let total = state.total_items();
    let count = state.item_count();
    let status_text = format!("{} items | Total: {} | ? for help", count, total);
    for (i, c) in status_text.chars().enumerate() {
        let idx = ((area.height - 1) * plane.width + 1 + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = Color::Rgb(180, 180, 180);
            plane.cells[idx].bg = Color::Ansi(236);
        }
    }

    plane
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    App::new()?
        .title("Tree Navigator")
        .fps(30)
        .theme(theme)
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();
            let area = Rect::new(0, 0, w, h);

            let mut state = TreeNavState::new();
            state.status_bar = StatusBar::new(WidgetId::new(3))
                .add_segment(StatusSegment::new("3 items").with_fg(Color::Rgb(180, 180, 180)).with_bg(Color::Ansi(236)));

            let plane = state.render(area);
            ctx.add_plane(plane);

            ctx.handle_events_for(&mut state);
        })
}