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
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, SplitPane, StatusBar, StatusSegment, Tree, TreeNode};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

struct MockFs {
    name: &'static str,
    children: Option<Vec<MockFs>>,
    is_dir: bool,
}

impl MockFs {
    fn new_dir(name: &'static str, children: Vec<MockFs>) -> Self {
        Self { name, children: Some(children), is_dir: true }
    }

    fn new_file(name: &'static str) -> Self {
        Self { name, children: None, is_dir: false }
    }

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

static MOCK_FS: MockFs = MockFs::new_dir("root", vec![
    MockFs::new_dir("src", vec![
        MockFs::new_file("main.rs"),
        MockFs::new_file("lib.rs"),
    ]),
    MockFs::new_dir("tests", vec![
        MockFs::new_file("test_main.rs"),
    ]),
    MockFs::new_file("README.md"),
    MockFs::new_file("Cargo.toml"),
]);

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    let root_node = MOCK_FS.to_tree_node();
    let mut tree = Tree::new(WidgetId::new(1)).with_root(vec![root_node]);

    App::new()?
        .title("Tree Navigator")
        .fps(30)
        .theme(theme)
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();

            let split = SplitPane::new(Orientation::Horizontal).ratio(0.35);
            let header_height = 1u16;
            let footer_height = 1u16;
            let content_height = h.saturating_sub(header_height + footer_height);

            let header_rect = Rect::new(0, 0, w, header_height);
            let content_rect = Rect::new(0, header_height, w, content_height);
            let footer_rect = Rect::new(0, h - footer_height, w, footer_height);

            let (tree_rect, detail_rect) = split.split(content_rect);

            let mut path_segments = vec!["home".to_string(), "user".to_string(), "projects".to_string()];
            let breadcrumbs = Breadcrumbs::new(path_segments.clone());
            let bc_plane = breadcrumbs.render(header_rect);
            ctx.add_plane(bc_plane);

            let tree_plane = tree.render(tree_rect);
            ctx.add_plane(tree_plane);

            let detail_plane = render_detail(&tree, &MOCK_FS, detail_rect);
            ctx.add_plane(detail_plane);

            let tree_path = tree.selected_path.clone();
            let item_count = if tree_path.is_empty() {
                MOCK_FS.child_count()
            } else if let Some(node) = MOCK_FS.find_by_path(&tree_path) {
                node.child_count()
            } else {
                MOCK_FS.child_count()
            };

            let status_text = format!("{} items | Total: {} | arrows: navigate, Enter: expand, Backspace: up", item_count, MOCK_FS.total_items());
            let status_bar = StatusBar::new(WidgetId::new(2))
                .add_segment(StatusSegment::new(&status_text).with_fg(Color::Rgb(180, 180, 180)).with_bg(Color::Ansi(236)));
            let status_plane = status_bar.render(footer_rect);
            ctx.add_plane(status_plane);
        })
}

fn render_detail(tree: &Tree, fs: &MockFs, area: Rect) -> Plane {
    let mut plane = Plane::new(1, area.width, area.height);
    plane.z_index = 5;

    for cell in plane.cells.iter_mut() {
        cell.bg = Color::Ansi(17);
    }

    let path = tree.selected_path.clone();

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

    if let Some(node) = fs.find_by_path(&path) {
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