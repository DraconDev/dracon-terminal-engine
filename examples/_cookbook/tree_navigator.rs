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

use std::cell::RefCell;
use std::rc::Rc;

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, SplitPane, StatusBar, StatusSegment, Tree, TreeNode};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

struct MockFs {
    name: &'static str,
    children: Option<Vec<MockFs>>,
    is_dir: bool,
}

impl MockFs {
    fn to_tree_node(&self) -> TreeNode {
        let mut node = TreeNode::new(self.name);
        if let Some(ref children) = self.children {
            for child in children {
                node.add_child(child.to_tree_node());
            }
        }
        node
    }

    fn find_by_path(&self, path: &[usize]) -> Option<&MockFs> {
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

    fn total_items(&self) -> usize {
        1 + self.children.as_ref().map(|c| c.iter().map(|ch| ch.total_items()).sum::<usize>()).unwrap_or(0)
    }
}

struct TreeNav {
    id: WidgetId,
    tree: Tree,
    breadcrumbs: Breadcrumbs,
    fs: MockFs,
    current_path: Vec<usize>,
}

impl TreeNav {
    fn new() -> Self {
        let fs = MockFs {
            name: "root",
            is_dir: true,
            children: Some(vec![
                MockFs { name: "src", is_dir: true, children: Some(vec![
                    MockFs { name: "main.rs", is_dir: false, children: None },
                    MockFs { name: "lib.rs", is_dir: false, children: None },
                ])},
                MockFs { name: "tests", is_dir: true, children: Some(vec![
                    MockFs { name: "test_main.rs", is_dir: false, children: None },
                ])},
                MockFs { name: "README.md", is_dir: false, children: None },
                MockFs { name: "Cargo.toml", is_dir: false, children: None },
            ]),
        };

        let root_node = fs.to_tree_node();
        let tree = Tree::new(WidgetId::new(1)).with_root(vec![root_node]);

        let segments = vec!["home".to_string(), "user".to_string(), "projects".to_string()];
        let breadcrumbs = Breadcrumbs::new(segments);

        Self {
            id: WidgetId::new(0),
            tree,
            breadcrumbs,
            fs,
            current_path: Vec::new(),
        }
    }

    fn item_count(&self) -> usize {
        if self.current_path.is_empty() {
            self.fs.child_count()
        } else if let Some(node) = self.fs.find_by_path(&self.current_path) {
            node.child_count()
        } else {
            self.fs.child_count()
        }
    }
}

impl Widget for TreeNav {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    fn set_area(&mut self, _area: Rect) {}

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

        let header_height = 1u16;
        let footer_height = 1u16;
        let content_height = area.height.saturating_sub(header_height + footer_height);

        let header_rect = Rect::new(0, 0, area.width, header_height);
        let content_rect = Rect::new(0, header_height, area.width, content_height);
        let footer_rect = Rect::new(0, area.height - footer_height, area.width, footer_height);

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.35);
        let (tree_rect, detail_rect) = split.split(content_rect);

        let bc_plane = self.breadcrumbs.render(header_rect);
        for (i, cell) in bc_plane.cells.iter().enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
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

        let detail_plane = self.render_detail(detail_rect);
        for (i, cell) in detail_plane.cells.iter().enumerate() {
            let base = (header_height * area.width) as usize;
            let idx = base + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = cell.clone();
            }
        }

        let status_text = format!("{} items | Total: {} | arrows: navigate, Enter: expand, Backspace: up",
            self.item_count(), self.fs.total_items());
        let status_bar = StatusBar::new(WidgetId::new(2))
            .add_segment(StatusSegment::new(&status_text).with_fg(Color::Rgb(180, 180, 180)).with_bg(Color::Ansi(236)));
        let status_plane = status_bar.render(footer_rect);
        for (i, cell) in status_plane.cells.iter().enumerate() {
            let base = ((area.height - footer_height) * area.width) as usize;
            let idx = base + i;
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
                    self.tree.set_selected_path(self.current_path.clone());
                    self.update_breadcrumbs();
                    return true;
                }
            }
            _ => {}
        }

        if self.tree.handle_key(key) {
            self.current_path = self.tree.get_selected_path().to_vec();
            self.update_breadcrumbs();
            true
        } else {
            false
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        let header_height = 1u16;
        let footer_height = 1u16;
        let content_height = 24u16.saturating_sub(header_height + footer_height);

        if row == 0 {
            return self.breadcrumbs.handle_mouse(kind, col, row);
        }

        let split = SplitPane::new(Orientation::Horizontal).ratio(0.35);
        let (tree_rect, _) = split.split(Rect::new(0, header_height, 80, content_height));

        if col < tree_rect.width && row > header_height && row < header_height + tree_rect.height {
            if self.tree.handle_mouse(kind, col, row - header_height) {
                self.current_path = self.tree.get_selected_path().to_vec();
                self.update_breadcrumbs();
                return true;
            }
        }

        false
    }
}

impl TreeNav {
    fn update_breadcrumbs(&mut self) {
        let mut segments = vec!["home".to_string(), "user".to_string(), "projects".to_string()];
        for &idx in &self.current_path {
            if let Some(node) = self.fs.find_by_path(&self.current_path[..=idx]) {
                segments.push(node.name.to_string());
            }
        }
        self.breadcrumbs = Breadcrumbs::new(segments);
    }

    fn render_detail(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(1, area.width, area.height);
        plane.z_index = 5;

        for cell in plane.cells.iter_mut() {
            cell.bg = Color::Ansi(17);
        }

        let print_line = |plane: &mut Plane, y: u16, text: &str, fg: Color| {
            for (i, c) in text.chars().take(area.width as usize - 2).enumerate() {
                let idx = (y * plane.width + 1 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                }
            }
        };

        let mut y = 1u16;
        print_line(&mut plane, y, "DETAILS", Color::Rgb(0, 255, 136));
        y += 2;

        if let Some(node) = self.fs.find_by_path(&self.current_path) {
            let icon = if node.is_dir { "[DIR]" } else { "[FILE]" };
            print_line(&mut plane, y, &format!("{} {}", icon, node.name), Color::Rgb(255, 255, 255));
            y += 1;

            if node.is_dir {
                if let Some(ref children) = node.children {
                    print_line(&mut plane, y, &format!("{} items", children.len()), Color::Rgb(150, 150, 150));
                    y += 2;
                    print_line(&mut plane, y, "Contents:", Color::Rgb(100, 180, 255));
                    y += 1;
                    for child in children {
                        let child_icon = if child.is_dir { "[DIR]" } else { "[FILE]" };
                        let child_name = format!("  {} {}", child_icon, child.name);
                        let fg = if child.is_dir {
                            Color::Rgb(100, 200, 255)
                        } else {
                            Color::Rgb(200, 200, 200)
                        };
                        print_line(&mut plane, y, &child_name, fg);
                        y += 1;
                        if y >= area.height - 1 {
                            break;
                        }
                    }
                }
            } else {
                print_line(&mut plane, y, "Type: Source file", Color::Rgb(150, 150, 150));
                y += 1;
                print_line(&mut plane, y, "Size: ~1KB (mock)", Color::Rgb(150, 150, 150));
            }
        } else {
            print_line(&mut plane, y, "No selection", Color::Rgb(150, 150, 150));
        }

        plane
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    App::new()?
        .title("Tree Navigator")
        .fps(30)
        .theme(theme)
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();
            let area = Rect::new(0, 0, w, h);

            let nav = Rc::new(RefCell::new(TreeNav::new()));
            let nav_clone = nav.clone();

            let plane = nav.borrow().render(area);
            ctx.add_plane(plane);

            ctx.on_key(move |key| {
                let mut nav = nav_clone.borrow_mut();
                nav.handle_key(key);
            });
        })
}