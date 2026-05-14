//! Tree widget for hierarchical data display.
//!
//! Renders a collapsible tree with expand/collapse state per node.

use std::cell::RefCell;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::dragdrop::DragManager;
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use crate::framework::widgets::context_menu::ContextMenu;
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
type SelectCallback = Box<dyn FnMut(&str)>;

pub struct Tree {
    id: WidgetId,
    root: Vec<TreeNode>,
    selected_path: Vec<usize>,
    hovered_path: Option<Vec<usize>>,
    theme: Theme,
    dirty: bool,
    scroll_offset: usize,
    visible_count: u16,
    on_select: Option<SelectCallback>,
    area: std::cell::Cell<Rect>,
    // Drag and drop
    drag_manager: RefCell<DragManager<String>>,
    // Context menu
    context_menu: RefCell<Option<ContextMenu>>,
}

impl Tree {
    /// Creates a new tree widget with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            root: Vec::new(),
            selected_path: Vec::new(),
            hovered_path: None,
            theme: Theme::default(),
            dirty: true,
            scroll_offset: 0,
            visible_count: 10,
            on_select: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            drag_manager: RefCell::new(DragManager::new()),
            context_menu: RefCell::new(None),
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

    /// Sets a context menu to show on right-click.
    pub fn with_context_menu(mut self, menu: ContextMenu) -> Self {
        self.context_menu = RefCell::new(Some(menu));
        self
    }

    /// Returns the drag manager for this tree.
    pub fn drag_manager(&self) -> &RefCell<DragManager<String>> {
        &self.drag_manager
    }

    /// Returns the current selection path as a dot-separated string.
    fn path_to_string(path: &[usize]) -> String {
        path.iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(".")
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

    fn count_visible_nodes(&self) -> usize {
        fn count(nodes: &[TreeNode]) -> usize {
            let mut total = 0;
            for node in nodes.iter() {
                total += 1;
                if node.expanded {
                    total += count(&node.children);
                }
            }
            total
        }
        count(&self.root)
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
        self.visible_count = area.height;
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
        plane.fill_bg(self.theme.bg);

        let width = plane.cells.len() / plane.height as usize;
        let hovered = &self.hovered_path;
        let skip = self.scroll_offset;
        let max_rows = self.visible_count as usize;

        #[allow(clippy::too_many_arguments)]
        fn render_node(
            node: &TreeNode,
            prefix: &str,
            plane: &mut Plane,
            theme: &Theme,
            width: usize,
            row: &mut usize,
            current_path: &mut Vec<usize>,
            hovered: &Option<Vec<usize>>,
            skip: usize,
            max_rows: usize,
        ) {
            if *row >= max_rows {
                return;
            }
            let is_hovered = hovered.as_ref().is_some_and(|h| h == current_path);
            let bg = if is_hovered { theme.hover_bg } else { theme.bg };
            let fg = theme.fg;

            let skip_counter = *row < skip;
            if !skip_counter {
                let line = format!(
                    "{}{}{}",
                    prefix,
                    if node.expanded { "- " } else { "+ " },
                    node.label
                );
                for (i, c) in line.chars().take(width).enumerate() {
                    let idx = (*row - skip) as u16 * plane.width + i as u16;
                    if (idx as usize) < plane.cells.len() {
                        plane.cells[idx as usize] = Cell {
                            char: c,
                            fg,
                            bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }
            *row += 1;

            if node.expanded {
                for (i, child) in node.children.iter().enumerate() {
                    current_path.push(i);
                    let child_prefix = if node.expanded { "  " } else { "" };
                    render_node(
                        child,
                        child_prefix,
                        plane,
                        theme,
                        width,
                        row,
                        current_path,
                        hovered,
                        skip,
                        max_rows,
                    );
                    current_path.pop();
                }
            }
        }

        let mut actual_row = 0usize;
        for (i, node) in self.root.iter().enumerate() {
            let mut path = vec![i];
            render_node(
                node,
                "",
                &mut plane,
                &self.theme,
                width,
                &mut actual_row,
                &mut path,
                hovered,
                skip,
                max_rows,
            );
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
        col: u16,
        row: u16,
    ) -> bool {
        // Check if context menu is visible
        if let Some(ref mut menu) = *self.context_menu.borrow_mut() {
            if menu.is_visible()
                && menu.handle_mouse(kind, col, row) {
                    return true;
                }
        }

        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                let adjusted_row = (row as usize).saturating_add(self.scroll_offset);
                let adjusted_row = adjusted_row.min(u16::MAX as usize) as u16;
                if let Some(path) = self.node_at_row(adjusted_row) {
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
            crate::input::event::MouseEventKind::Moved => {
                let adjusted_row = (row as usize).saturating_add(self.scroll_offset);
                let adjusted_row = adjusted_row.min(u16::MAX as usize) as u16;
                if let Some(path) = self.node_at_row(adjusted_row) {
                    if self.hovered_path.as_ref() != Some(&path) {
                        self.hovered_path = Some(path);
                        self.dirty = true;
                        return true;
                    }
                } else if self.hovered_path.is_some() {
                    self.hovered_path = None;
                    self.dirty = true;
                    return true;
                }
                false
            }
            // Right-click: Show context menu
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Right) => {
                let adjusted_row = (row as usize).saturating_add(self.scroll_offset);
                let adjusted_row = adjusted_row.min(u16::MAX as usize) as u16;
                if let Some(_path) = self.node_at_row(adjusted_row) {
                    if let Some(menu) = &mut *self.context_menu.borrow_mut() {
                        menu.show();
                        let area = self.area.get();
                        menu.set_anchor(area.x + col, area.y + row);
                        self.dirty = true;
                    }
                }
                true
            }
            crate::input::event::MouseEventKind::Drag(_) => {
                if self.drag_manager.borrow().is_dragging() {
                    let area = self.area.get();
                    self.drag_manager.borrow_mut().move_ghost(area.x + col, area.y + row);
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                let total = self.count_visible_nodes();
                let max_offset = total.saturating_sub(self.visible_count as usize);
                if self.scroll_offset < max_offset {
                    self.scroll_offset += 1;
                    self.dirty = true;
                    return true;
                }
                false
            }
            crate::input::event::MouseEventKind::ScrollUp => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    self.dirty = true;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
    }
}

impl WidgetState for Tree {
    fn state_id(&self) -> Option<&str> {
        Some("tree")
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "selected_path": Self::path_to_string(&self.selected_path),
            "scroll_offset": self.scroll_offset,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(path_str) = json.get("selected_path").and_then(|v| v.as_str()) {
            let path: Vec<usize> = path_str
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();
            self.selected_path = path;
        }
        if let Some(offset) = json.get("scroll_offset").and_then(|v| v.as_u64()) {
            self.scroll_offset = offset as usize;
        }
        self.dirty = true;
        Ok(())
    }
}
