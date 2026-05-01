//! Widget inspector for displaying widget hierarchy.
//!
//! Shows the tree structure of active widgets with their IDs and states.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A node in the widget hierarchy for inspection.
pub struct WidgetNode {
    /// The widget ID for this node.
    pub id: WidgetId,
    /// The label for this node.
    pub label: String,
    /// The child nodes of this widget.
    pub children: Vec<WidgetNode>,
}

impl WidgetNode {
    /// Creates a new widget node with the given ID and label.
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_string(),
            children: Vec::new(),
        }
    }
}

/// A widget that displays the live widget hierarchy for inspection.
pub struct WidgetInspector {
    /// The widget ID for this inspector.
    id: WidgetId,
    /// The root nodes of the widget hierarchy.
    root: Vec<WidgetNode>,
    /// The theme for this widget.
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl WidgetInspector {
    /// Creates a new widget inspector with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            root: Vec::new(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 60, 20)),
            dirty: true,
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the widget hierarchy to display.
    pub fn set_hierarchy(&mut self, nodes: Vec<WidgetNode>) {
        self.root = nodes;
        self.dirty = true;
    }
}

impl crate::framework::widget::Widget for WidgetInspector {
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

    fn z_index(&self) -> u16 {
        180
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 180;

        let width = plane.cells.len() / plane.height as usize;
        let mut row = 0usize;

        fn render_node(
            node: &WidgetNode,
            indent: usize,
            plane: &mut Plane,
            theme: &Theme,
            width: usize,
            row: &mut usize,
        ) {
            if *row >= plane.height as usize {
                return;
            }
            let prefix = "  ".repeat(indent);
            let line = format!("{}{} ({:?})", prefix, node.label, node.id);
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

            for child in &node.children {
                render_node(child, indent + 1, plane, theme, width, row);
            }
        }

        for node in &self.root {
            render_node(node, 0, &mut plane, &self.theme, width, &mut row);
        }

        plane
    }
}
