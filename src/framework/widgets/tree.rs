//! Tree widget for hierarchical data display.
//!
//! Renders a collapsible tree with expand/collapse state per node.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A node in the tree hierarchy with a label and optional children.
pub struct TreeNode {
    /// The display label for this node.
    pub label: String,
    /// Whether this node is expanded (children visible).
    pub expanded: bool,
    /// Child nodes beneath this one.
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// Creates a new tree node with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            expanded: false,
            children: Vec::new(),
        }
    }

    /// Adds a child node to this tree node.
    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }
}

/// A widget that displays hierarchical data as a collapsible tree.
pub struct Tree {
    id: WidgetId,
    root: Vec<TreeNode>,
    selected_path: Vec<usize>,
    theme: Theme,
    on_select: Option<Box<dyn FnMut(&str)>>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Tree {
    /// Creates a new tree widget with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            root: Vec::new(),
            selected_path: Vec::new(),
            theme: Theme::default(),
            on_select: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 20)),
            dirty: true,
        }
    }

    /// Sets the root nodes of the tree.
    pub fn with_root(mut self, root: Vec<TreeNode>) -> Self {
        self.root = root;
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Registers a callback when a node is selected.
    pub fn on_select(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn set_selected_path(&mut self, path: Vec<usize>) {
        self.selected_path = path;
        self.dirty = true;
    }

    pub fn get_selected_path(&self) -> &[usize] {
        &self.selected_path
    }

    fn get_selected_node<'a>(
        &self,
        nodes: &'a [TreeNode],
        path: &[usize],
    ) -> Option<(&'a TreeNode, usize)> {
        if path.is_empty() {
            return None;
        }
        let idx = path[0];
        if idx >= nodes.len() {
            return None;
        }
        if path.len() == 1 {
            return Some((&nodes[idx], idx));
        }
        self.get_selected_node(&nodes[idx].children, &path[1..])
    }

    fn node_at_row(&self, row: u16) -> Option<Vec<usize>> {
        let mut current_row = 0u16;
        fn traverse(
            nodes: &[TreeNode],
            row: u16,
            current_row: &mut u16,
            path: &mut Vec<usize>,
        ) -> Option<Vec<usize>> {
            for (i, node) in nodes.iter().enumerate() {
                if *current_row >= row {
                    path.push(i);
                    return Some(path.clone());
                }
                *current_row += 1;
                if node.expanded {
                    path.push(i);
                    if let Some(result) = traverse(&node.children, row, current_row, path) {
                        return Some(result);
                    }
                    path.pop();
                }
            }
            None
        }
        traverse(&self.root, row, &mut current_row, &mut Vec::new())
    }

    fn toggle_expand_at(&mut self, path: &[usize]) {
        if path.is_empty() {
            return;
        }
        let mut current = &mut self.root;
        for i in 0..path.len() - 1 {
            if path[i] >= current.len() {
                return;
            }
            current = &mut current[path[i]].children;
        }
        let last_idx = *path.last().unwrap();
        if last_idx < current.len() {
            current[last_idx].expanded = !current[last_idx].expanded;
        }
    }
}

impl crate::framework::widget::Widget for Tree {
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

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;
        let mut row = 0usize;

        fn render_node(
            node: &TreeNode,
            prefix: &str,
            plane: &mut Plane,
            theme: &Theme,
            width: usize,
            row: &mut usize,
        ) {
            if *row >= plane.height as usize {
                return;
            }
            let line = format!("{}{}", prefix, if node.expanded { "- " } else { "+ " });
            let _label_len = line.width().min(width);
            for (i, c) in line.chars().take(width).enumerate() {
                let idx = (*row as u16 * plane.width + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: theme.fg,
                        bg: theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
            *row += 1;

            if node.expanded {
                for child in &node.children {
                    let child_prefix = if node.expanded { "  " } else { "" };
                    render_node(child, child_prefix, plane, theme, width, row);
                }
            }
        }

        for node in &self.root {
            render_node(node, "", &mut plane, &self.theme, width, &mut row);
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Enter => {
                if !self.selected_path.is_empty() {
                    let path = self.selected_path.clone();
                    self.toggle_expand_at(&path);
                    self.dirty = true;
                }
                true
            }
            KeyCode::Down => {
                if let Some((node, _)) = self.get_selected_node(&self.root, &self.selected_path) {
                    if node.expanded && !node.children.is_empty() {
                        self.selected_path.push(0);
                        self.dirty = true;
                    }
                }
                true
            }
            KeyCode::Up => {
                if !self.selected_path.is_empty() {
                    self.selected_path.pop();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Right => {
                if !self.selected_path.is_empty() {
                    let path = self.selected_path.clone();
                    if let Some((node, _)) = self.get_selected_node(&self.root, &path) {
                        if !node.expanded && !node.children.is_empty() {
                            self.toggle_expand_at(&path);
                            self.selected_path.push(0);
                            self.dirty = true;
                        }
                    }
                }
                true
            }
            KeyCode::Left => {
                if !self.selected_path.is_empty() {
                    let path = self.selected_path.clone();
                    if let Some((node, _)) = self.get_selected_node(&self.root, &path) {
                        if node.expanded {
                            self.toggle_expand_at(&path);
                        } else {
                            self.selected_path.pop();
                        }
                        self.dirty = true;
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        _col: u16,
        row: u16,
    ) -> bool {
        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                if let Some(path) = self.node_at_row(row) {
                    if let Some((node, _)) = self.get_selected_node(&self.root, &path) {
                        if node.expanded && !node.children.is_empty() {
                            self.selected_path = path;
                            self.selected_path.push(0);
                        } else if !node.children.is_empty() {
                            self.selected_path = path.clone();
                            self.toggle_expand_at(&path);
                            self.selected_path.push(0);
                        } else {
                            self.selected_path = path;
                        }
                        self.dirty = true;
                    }
                }
                true
            }
            _ => false,
        }
    }
}
