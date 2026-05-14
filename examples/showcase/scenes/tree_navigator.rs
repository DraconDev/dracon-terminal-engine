//! Embedded Tree Navigator scene for the showcase.
//!
//! Demonstrates Tree + Breadcrumbs with hierarchical navigation.
//! Press `B`/`Esc` to go back.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, Tree, TreeNode};
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct MockFs {
    name: &'static str,
    children: Option<Vec<MockFs>>,
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

    fn total_items(&self) -> usize {
        1 + self.children.as_ref()
            .map(|c| c.iter().map(|ch| ch.total_items()).sum::<usize>())
            .unwrap_or(0)
    }
}

pub struct TreeNavigatorScene {
    tree: Tree,
    breadcrumbs: Breadcrumbs,
    fs: MockFs,
    theme: Theme,
    show_help: bool,
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
}

impl TreeNavigatorScene {
    pub fn new(theme: Theme) -> Self {
        let fs = MockFs {
            name: "root",
            children: Some(vec![
                MockFs {
                    name: "src",
                    children: Some(vec![
                        MockFs { name: "main.rs", children: None },
                        MockFs { name: "lib.rs", children: None },
                    ]),
                },
                MockFs {
                    name: "tests",
                    children: Some(vec![
                        MockFs { name: "test_main.rs", children: None },
                    ]),
                },
                MockFs { name: "README.md", children: None },
                MockFs { name: "Cargo.toml", children: None },
            ]),
        };

        let root_node = fs.to_tree_node();
        let tree = Tree::new(WidgetId::new(10)).with_root(vec![root_node]);
        let segments = vec!["home".to_string(), "user".to_string(), "projects".to_string()];
        let breadcrumbs = Breadcrumbs::new(segments);

        Self {
            tree,
            breadcrumbs,
            fs,
            theme,
            show_help: false,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

}

impl Scene for TreeNavigatorScene {
    fn scene_id(&self) -> &str { "tree_navigator" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = self.theme.clone();
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title
        let title = " Tree Navigator ";
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Breadcrumbs
        let bc_height = 2u16;
        let bc_area = Rect::new(0, 1, area.width, bc_height);
        let mut bc_plane = self.breadcrumbs.render(bc_area);
        blit_to(&mut plane, &mut bc_plane, 0, 1);

        // Tree (left half)
        let split_x = area.width / 2;
        let tree_area = Rect::new(0, 4, split_x, area.height.saturating_sub(8));
        let mut tree_plane = self.tree.render(tree_area);
        blit_to(&mut plane, &mut tree_plane, 0, 4);

        // Divider
        for y in 4..area.height.saturating_sub(4) {
            let idx = (y * area.width + split_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Detail pane (right half)
        let detail_x = split_x + 1;
        let detail_w = area.width.saturating_sub(detail_x);
        let detail_area = Rect::new(detail_x, 4, detail_w, area.height.saturating_sub(8));
        render_detail(&mut plane, detail_area, t);

        // Status bar
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let count = self.fs.total_items();
        let status_text = format!("{} items total | ↑↓ nav | Enter: expand | B/Esc: back | ?: help | q: quit", count);
        draw_text(&mut plane, 2, footer_y, &status_text, t.fg_muted, t.bg, false);

        if self.show_help {
            draw_help(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }
        if self.tree.handle_key(key) {
            return true;
        }
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let tree_area = Rect::new(0, 4, self.area.get().width / 2, self.area.get().height.saturating_sub(8));
        if col >= tree_area.x && col < tree_area.x + tree_area.width &&
           row >= tree_area.y && row < tree_area.y + tree_area.height {
            let rel_col = col - tree_area.x;
            let rel_row = row - tree_area.y;
            return self.tree.handle_mouse(kind, rel_col, rel_row);
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.tree.on_theme_change(theme);
        self.breadcrumbs.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch, fg, bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false, skip: false,
            };
        }
    }
}

fn blit_to(dest: &mut Plane, src: &mut Plane, offset_x: usize, offset_y: usize) {
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent { continue; }
        let row = i / src.width as usize;
        let col = i % src.width as usize;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize { continue; }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = *cell;
        }
    }
}

fn render_detail(plane: &mut Plane, area: Rect, t: Theme) {
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            let idx = (y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
            }
        }
    }

    let title = "Details";
    draw_text(plane, area.x + 1, area.y + 1, title, t.primary, t.surface, true);

    let lines = [
        "Name: main.rs",
        "Type: Source file",
        "Size: 1.2 KB",
        "",
        "Select an item from",
        "the tree to see its",
        "details here.",
    ];

    for (i, line) in lines.iter().enumerate() {
        let y = area.y + 3 + i as u16;
        if y < area.y + area.height.saturating_sub(1) {
            draw_text(plane, area.x + 1, y, line, t.fg_muted, t.surface, false);
        }
    }
}

fn draw_help(plane: &mut Plane, area: Rect, t: Theme) {
    let hw = 42u16.min(area.width.saturating_sub(4));
    let hh = 10u16.min(area.height.saturating_sub(4));
    let hx = (area.width - hw) / 2;
    let hy = (area.height - hh) / 2;

    for y in hy..hy + hh {
        for x in hx..hx + hw {
            let idx = (y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }

    for x in hx + 1..hx + hw - 1 {
        let top = (hy * area.width + x) as usize;
        let bot = ((hy + hh - 1) * area.width + x) as usize;
        if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
        if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
    }
    for y in hy + 1..hy + hh - 1 {
        let left = (y * area.width + hx) as usize;
        let right = (y * area.width + hx + hw - 1) as usize;
        if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
        if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
    }
    // Rounded corners
    let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
    for (ch, cx, cy) in corners {
        let idx = (cy * area.width + cx) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
    }

    let title = "Tree Navigator Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

    let shortcuts = [
        ("↑/↓", "Navigate tree"),
        ("Enter/→", "Expand folder"),
        ("←", "Collapse folder"),
        ("B/Esc", "Back to showcase"),
        ("?", "Toggle help"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 16, row, desc, t.fg, t.surface_elevated, false);
    }
}