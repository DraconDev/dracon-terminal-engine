//! Embedded Tree Navigator scene for the showcase.
//!
//! Demonstrates Tree + Breadcrumbs with hierarchical navigation.
//! Rich detail pane with file type icons, size bars, content preview.

use crate::scenes::shared_helpers::{blit_to, draw_text};
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, Tree, TreeNode, StatusBar, StatusSegment};
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct MockFs {
    name: &'static str,
    kind: &'static str, // "dir", "rs", "toml", "md", "json", "lock", "sh", "txt"
    size_kb: f32,       // 0.0 for dirs
    modified: &'static str,
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
        1 + self
            .children
            .as_ref()
            .map(|c| c.iter().map(|ch| ch.total_items()).sum::<usize>())
            .unwrap_or(0)
    }

    fn find(&self, name: &str) -> Option<&MockFs> {
        if self.name == name {
            return Some(self);
        }
        self.children
            .as_ref()
            .and_then(|c| c.iter().find_map(|ch| ch.find(name)))
    }

    fn icon(&self) -> char {
        match self.kind {
            "dir" => '◈',
            "rs" => '⚙',
            "toml" => '▤',
            "md" => '◇',
            "json" => '◆',
            "lock" => '⊞',
            "sh" => '▶',
            "txt" => '◧',
            _ => '○',
        }
    }

    fn type_label(&self) -> &str {
        match self.kind {
            "dir" => "Directory",
            "rs" => "Rust Source",
            "toml" => "TOML Config",
            "md" => "Markdown",
            "json" => "JSON Data",
            "lock" => "Lock File",
            "sh" => "Shell Script",
            "txt" => "Plain Text",
            _ => "File",
        }
    }

    fn icon_color(&self, t: &Theme) -> Color {
        match self.kind {
            "dir" => t.primary,
            "rs" => t.warning,
            "toml" => t.secondary,
            "md" => t.info,
            "json" => t.success,
            "lock" => t.fg_muted,
            "sh" => t.error,
            "txt" => t.fg,
            _ => t.fg_muted,
        }
    }

    fn preview_lines(&self) -> Vec<&str> {
        match self.name {
            "main.rs" => vec![
                "fn main() {",
                "    println!(\"Hello\");",
                "    app::run();",
                "}",
            ],
            "lib.rs" => vec![
                "pub mod engine;",
                "pub mod compositor;",
                "pub mod framework;",
                "pub mod input;",
            ],
            "mod.rs" => vec!["pub mod app;", "pub mod widget;", "pub mod theme;"],
            "engine.rs" => vec![
                "pub struct Engine {",
                "    running: bool,",
                "    theme: Theme,",
                "}",
            ],
            "event.rs" => vec![
                "pub enum Event {",
                "    Key(KeyEvent),",
                "    Mouse(MouseEvent),",
                "}",
            ],
            "theme.rs" => vec![
                "pub struct Theme {",
                "    pub name: &str,",
                "    pub bg: Color,",
                "}",
            ],
            "Cargo.toml" => vec![
                "[package]",
                "name = \"dte\"",
                "version = \"0.1.0\"",
                "edition = \"2021\"",
            ],
            "Cargo.lock" => vec![
                "# This file is auto-generated",
                "# by Cargo.",
                "version = 3",
            ],
            "README.md" => vec![
                "# Dracon Terminal Engine",
                "",
                "GUI-grade terminal apps.",
                "Widgets + compositor.",
            ],
            "LICENSE" => vec!["MIT License", "", "Copyright (c) 2024"],
            "Makefile.toml" => vec!["[tasks.build]", "command = \"cargo\"", "args = [\"build\"]"],
            ".gitignore" => vec!["/target", "**/*.rs.bk", "Cargo.lock"],
            "run.sh" => vec!["#!/bin/bash", "cargo run --example $1"],
            "config.json" => vec!["{", "  \"theme\": \"nord\",", "  \"font_size\": 14", "}"],
            _ => vec!["(no preview available)"],
        }
    }
}

fn build_fs() -> MockFs {
    MockFs {
        name: "project",
        kind: "dir",
        size_kb: 0.0,
        modified: "2024-12-01",
        children: Some(vec![
            MockFs {
                name: "src",
                kind: "dir",
                size_kb: 0.0,
                modified: "2024-12-10",
                children: Some(vec![
                    MockFs {
                        name: "main.rs",
                        kind: "rs",
                        size_kb: 2.4,
                        modified: "2024-12-10",
                        children: None,
                    },
                    MockFs {
                        name: "lib.rs",
                        kind: "rs",
                        size_kb: 1.8,
                        modified: "2024-12-09",
                        children: None,
                    },
                    MockFs {
                        name: "engine",
                        kind: "dir",
                        size_kb: 0.0,
                        modified: "2024-12-08",
                        children: Some(vec![
                            MockFs {
                                name: "mod.rs",
                                kind: "rs",
                                size_kb: 0.3,
                                modified: "2024-12-08",
                                children: None,
                            },
                            MockFs {
                                name: "theme.rs",
                                kind: "rs",
                                size_kb: 4.2,
                                modified: "2024-12-07",
                                children: None,
                            },
                            MockFs {
                                name: "event.rs",
                                kind: "rs",
                                size_kb: 1.1,
                                modified: "2024-12-06",
                                children: None,
                            },
                        ]),
                    },
                    MockFs {
                        name: "framework",
                        kind: "dir",
                        size_kb: 0.0,
                        modified: "2024-12-10",
                        children: Some(vec![
                            MockFs {
                                name: "mod.rs",
                                kind: "rs",
                                size_kb: 0.5,
                                modified: "2024-12-10",
                                children: None,
                            },
                            MockFs {
                                name: "engine.rs",
                                kind: "rs",
                                size_kb: 3.7,
                                modified: "2024-12-09",
                                children: None,
                            },
                        ]),
                    },
                ]),
            },
            MockFs {
                name: "tests",
                kind: "dir",
                size_kb: 0.0,
                modified: "2024-12-05",
                children: Some(vec![MockFs {
                    name: "integration.rs",
                    kind: "rs",
                    size_kb: 1.2,
                    modified: "2024-12-05",
                    children: None,
                }]),
            },
            MockFs {
                name: "examples",
                kind: "dir",
                size_kb: 0.0,
                modified: "2024-12-10",
                children: Some(vec![MockFs {
                    name: "demo.rs",
                    kind: "rs",
                    size_kb: 5.6,
                    modified: "2024-12-10",
                    children: None,
                }]),
            },
            MockFs {
                name: "Cargo.toml",
                kind: "toml",
                size_kb: 0.5,
                modified: "2024-12-01",
                children: None,
            },
            MockFs {
                name: "Cargo.lock",
                kind: "lock",
                size_kb: 12.3,
                modified: "2024-12-10",
                children: None,
            },
            MockFs {
                name: "README.md",
                kind: "md",
                size_kb: 3.1,
                modified: "2024-11-28",
                children: None,
            },
            MockFs {
                name: "LICENSE",
                kind: "txt",
                size_kb: 1.1,
                modified: "2024-11-01",
                children: None,
            },
            MockFs {
                name: ".gitignore",
                kind: "txt",
                size_kb: 0.1,
                modified: "2024-11-01",
                children: None,
            },
            MockFs {
                name: "Makefile.toml",
                kind: "toml",
                size_kb: 0.8,
                modified: "2024-11-15",
                children: None,
            },
            MockFs {
                name: "config.json",
                kind: "json",
                size_kb: 0.2,
                modified: "2024-12-01",
                children: None,
            },
            MockFs {
                name: "run.sh",
                kind: "sh",
                size_kb: 0.1,
                modified: "2024-11-20",
                children: None,
            },
        ]),
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
    dirty: bool,
    status_bar: std::cell::RefCell<StatusBar>,
}

impl TreeNavigatorScene {
    pub fn new(theme: Theme) -> Self {
        let fs = build_fs();
        let root_node = fs.to_tree_node();
        let tree = Tree::new(WidgetId::new(10))
            .with_root(vec![root_node])
            .with_theme(theme.clone());
        let segments = vec![
            "home".to_string(),
            "user".to_string(),
            "projects".to_string(),
        ];
        let breadcrumbs = Breadcrumbs::new(segments).with_theme(theme.clone());

        Self {
            tree,
            breadcrumbs,
            fs,
            theme,
            show_help: false,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            dirty: true,
            status_bar: std::cell::RefCell::new(
                StatusBar::new(WidgetId::new(2017))
                    .add_segment(StatusSegment::new(
                        "↑↓:navigate | Enter:expand | <:collapse | F1:help | Esc:back",
                    ))
                    .with_theme(theme.clone()),
            ),
        }
    }
}

impl Scene for TreeNavigatorScene {
    fn scene_id(&self) -> &str {
        "tree_navigator"
    }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title
        draw_text(&mut plane, 2, 0, " Tree Navigator ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Breadcrumbs
        let bc_area = Rect::new(0, 1, area.width, 2);
        let bc_plane = self.breadcrumbs.render(bc_area);
        blit_to(&mut plane, &bc_plane, 0, 1);

        // Tree (left ~45%)
        let split_x = area.width * 45 / 100;
        let tree_area = Rect::new(0, 4, split_x, area.height.saturating_sub(8));
        let tree_plane = self.tree.render(tree_area);
        blit_to(&mut plane, &tree_plane, 0, 4);

        // Vertical divider
        for y in 4..area.height.saturating_sub(4) {
            let idx = (y * area.width + split_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Detail pane (right)
        let detail_x = split_x + 2;
        let detail_w = area.width.saturating_sub(detail_x + 2);
        let selected_label = self.tree.selected_label().map(|s| s.to_string());
        let selected_fs = selected_label
            .as_deref()
            .and_then(|name| self.fs.find(name));
        render_detail(
            &mut plane,
            detail_x,
            4,
            detail_w,
            area.height.saturating_sub(8),
            t,
            selected_fs,
        );

        // File type legend
        let legend_y = area.height.saturating_sub(4);
        render_legend(&mut plane, 2, legend_y, area.width.saturating_sub(4), t);

        // Status bar
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        let count = self.fs.total_items();
        let status_text = format!(
            " {} items | ↑↓ nav | Enter: expand | ?: help | B: back ",
            count
        );
        draw_text(
            &mut plane,
            2,
            footer_y,
            &status_text,
            t.fg_muted,
            t.surface,
            false,
        );

        if self.show_help {
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            crate::scenes::shared_helpers::render_help_overlay(
                &mut plane,
                area,
                t,
                "Tree Navigator Help",
                &[
                    ("↑/↓", "Navigate tree"),
                    ("Enter/>", "Expand folder"),
                    ("<", "Collapse folder"),
                    ("Click", "Select item"),
                    (back_key, "Back"),
                    ("?", "Toggle help"),
                ],
            );
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_area = ratatui::layout::Rect::new(0, sb_y, area.width, 1);
        self.status_bar.borrow_mut().set_area(sb_area);
        let sb_plane = self.status_bar.borrow().render(sb_area);
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }
        if self.tree.handle_key(key) {
            self.dirty = true;
            return true;
        }
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        let tree_area = Rect::new(0, 4, area.width * 45 / 100, area.height.saturating_sub(8));

        // Tree widget area
        if col >= tree_area.x
            && col < tree_area.x + tree_area.width
            && row >= tree_area.y
            && row < tree_area.y + tree_area.height
        {
            let rel_col = col - tree_area.x;
            let rel_row = row - tree_area.y;
            if self.tree.handle_mouse(kind, rel_col, rel_row) {
                self.dirty = true;
                return true;
            }
        }

        // Breadcrumb clicks (row 1)
        if let MouseEventKind::Down(_) = kind {
            if row == 1 {
                // Clicking breadcrumb navigates up to parent
                let path = self.tree.get_selected_path();
                if path.len() > 1 {
                    let parent_path = path[..path.len() - 1].to_vec();
                    self.tree.set_selected_path(parent_path);
                    self.dirty = true;
                    return true;
                }
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.tree.on_theme_change(theme);
        self.breadcrumbs.on_theme_change(theme);
        self.dirty = true;
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
}

fn render_detail(
    plane: &mut Plane,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    t: &Theme,
    selected: Option<&MockFs>,
) {
    // Background
    for dy in 0..h {
        for dx in 0..w {
            let idx = ((y + dy) * plane.width + x + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
    }

    draw_text(
        plane,
        x + 1,
        y + 1,
        "File Details",
        t.primary,
        t.surface,
        true,
    );
    for dx in 0..w {
        let idx = ((y + 2) * plane.width + x + dx) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = t.outline;
        }
    }

    if let Some(fs) = selected {
        // Icon + name
        let icon_color = fs.icon_color(t);
        let icon_idx = ((y + 3) * plane.width + x + 1) as usize;
        if icon_idx < plane.cells.len() {
            plane.cells[icon_idx].char = fs.icon();
            plane.cells[icon_idx].fg = icon_color;
        }
        draw_text(plane, x + 3, y + 3, fs.name, t.fg, t.surface, true);

        // Type
        draw_text(plane, x + 1, y + 5, "Type:", t.fg_muted, t.surface, false);
        draw_text(
            plane,
            x + 8,
            y + 5,
            fs.type_label(),
            icon_color,
            t.surface,
            false,
        );

        // Size (with bar for files)
        draw_text(plane, x + 1, y + 6, "Size:", t.fg_muted, t.surface, false);
        if fs.kind == "dir" {
            draw_text(
                plane,
                x + 8,
                y + 6,
                "Directory",
                t.fg_muted,
                t.surface,
                false,
            );
        } else {
            let size_str = format!("{:.1} KB", fs.size_kb);
            draw_text(plane, x + 8, y + 6, &size_str, t.fg, t.surface, false);
            // Size bar (max 15KB scale)
            let bar_w = w.saturating_sub(10).min(20);
            let filled = (fs.size_kb / 15.0 * bar_w as f32).min(bar_w as f32) as usize;
            for dx in 0..bar_w as usize {
                let idx = ((y + 7) * plane.width + x + 8 + dx as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = if dx < filled { '█' } else { '░' };
                    plane.cells[idx].fg = if dx < filled { t.primary } else { t.fg_muted };
                }
            }
        }

        // Modified
        draw_text(
            plane,
            x + 1,
            y + 8,
            "Modified:",
            t.fg_muted,
            t.surface,
            false,
        );
        draw_text(plane, x + 12, y + 8, fs.modified, t.fg, t.surface, false);

        // Children count for dirs
        if fs.kind == "dir" {
            if let Some(ref children) = fs.children {
                draw_text(
                    plane,
                    x + 1,
                    y + 9,
                    "Children:",
                    t.fg_muted,
                    t.surface,
                    false,
                );
                let count_str = format!("{} items", children.len());
                draw_text(plane, x + 12, y + 9, &count_str, t.fg, t.surface, false);
            }
        }

        // Content preview
        let preview_y = y + 10;
        if preview_y + 6 < y + h {
            draw_text(
                plane,
                x + 1,
                preview_y,
                "Preview",
                t.secondary,
                t.surface,
                true,
            );
            for dx in 0..w {
                let idx = ((preview_y + 1) * plane.width + x + dx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }

            // Preview background
            for dy in 0..4.min((y + h - preview_y - 2) as usize) {
                for dx in 0..w {
                    let idx = ((preview_y + 2 + dy as u16) * plane.width + x + dx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.bg;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            let lines = fs.preview_lines();
            for (i, line) in lines.iter().take(4).enumerate() {
                let ly = preview_y + 2 + i as u16;
                if ly >= y + h {
                    break;
                }
                // Line number
                draw_text(
                    plane,
                    x + 1,
                    ly,
                    &format!("{:>2}", i + 1),
                    t.fg_muted,
                    t.bg,
                    false,
                );
                // Content
                draw_text(plane, x + 4, ly, line, t.fg, t.bg, false);
            }
        }
    } else {
        let hint = [
            "Select an item from",
            "the tree to see its",
            "details and preview.",
        ];
        for (i, line) in hint.iter().enumerate() {
            let ly = y + 4 + i as u16;
            if ly < y + h {
                draw_text(plane, x + 1, ly, line, t.fg_muted, t.surface, false);
            }
        }
    }
}

fn render_legend(plane: &mut Plane, x: u16, y: u16, w: u16, t: &Theme) {
    draw_text(plane, x, y, "File Types:", t.secondary, t.bg, true);
    let types = [
        ('◈', "dir", t.primary),
        ('⚙', "rs", t.warning),
        ('▤', "toml", t.secondary),
        ('◇', "md", t.info),
        ('◆', "json", t.success),
        ('⊞', "lock", t.fg_muted),
        ('▶', "sh", t.error),
    ];
    let mut lx = x + 12;
    for (icon, label, color) in types {
        if lx + label.len() as u16 + 4 > x + w {
            break;
        }
        let idx = (y * plane.width + lx) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = icon;
            plane.cells[idx].fg = color;
        }
        draw_text(plane, lx + 2, y, label, t.fg_muted, t.bg, false);
        lx += label.len() as u16 + 4;
    }
}
