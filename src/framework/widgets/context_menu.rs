//! Context menu widget.

use crate::compositor::Plane;
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// An action type for context menu items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextAction {
    /// Open the target.
    Open,
    /// Edit the target.
    Edit,
    /// Delete the target.
    Delete,
    /// Rename the target.
    Rename,
    /// Copy the target.
    Copy,
    /// Cut the target.
    Cut,
    /// Paste from clipboard.
    Paste,
    /// Visual separator between items.
    Separator,
}

/// A popup context menu with labeled actions.
pub struct ContextMenu {
    id: WidgetId,
    items: Vec<(String, ContextAction)>,
    theme: Theme,
    width: u16,
    anchor_x: u16,
    anchor_y: u16,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl ContextMenu {
    /// Creates a new `ContextMenu` from a list of label/action pairs.
    pub fn new(items: Vec<(&'static str, ContextAction)>) -> Self {
        Self {
            id: WidgetId::default_id(),
            items: items.into_iter().map(|(s, a)| (s.to_string(), a)).collect(),
            theme: Theme::default(),
            width: 20,
            anchor_x: 0,
            anchor_y: 0,
            area: std::cell::Cell::new(Rect::new(0, 0, 20, 10)),
            dirty: true,
        }
    }

    /// Creates a new `ContextMenu` with the given widget ID and label/action pairs.
    pub fn new_with_id(id: WidgetId, items: Vec<(&'static str, ContextAction)>) -> Self {
        Self {
            id,
            items: items.into_iter().map(|(s, a)| (s.to_string(), a)).collect(),
            theme: Theme::default(),
            width: 20,
            anchor_x: 0,
            anchor_y: 0,
            area: std::cell::Cell::new(Rect::new(0, 0, 20, 10)),
            dirty: true,
        }
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the menu width in cells.
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self.dirty = true;
        self
    }

    /// Sets the anchor position for the menu.
    pub fn with_anchor(mut self, x: u16, y: u16) -> Self {
        self.anchor_x = x;
        self.anchor_y = y;
        self.dirty = true;
        self
    }
}

impl crate::framework::widget::Widget for ContextMenu {
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
        200
    }

    fn render(&self, screen: Rect) -> Plane {
        let height = self.items.len() as u16;
        let mut x = self.anchor_x;
        let mut y = self.anchor_y;

        if x + self.width > screen.width {
            x = screen.width.saturating_sub(self.width);
        }
        if y + height > screen.height {
            y = screen.height.saturating_sub(height);
        }

        let mut plane = Plane::new(0, self.width, height);
        plane.x = x;
        plane.y = y;
        plane.z_index = 200;

        for cell in &mut plane.cells {
            cell.bg = self.theme.bg;
            cell.fg = self.theme.fg;
        }

        for (i, (label, _action)) in self.items.iter().enumerate() {
            let row = i as u16;
            let _zone = HitZone::new(i, x, y + row, self.width, 1);

            let idx = (row * self.width) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ' ';
            }

            for (j, ch) in label.chars().enumerate() {
                if j as u16 >= self.width - 1 {
                    break;
                }
                let idx = (row * self.width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.fg;
                }
            }
        }

        for col in 0..self.width {
            let top_idx = col as usize;
            if top_idx < plane.cells.len() {
                plane.cells[top_idx].char = '─';
            }
            let bot_idx = ((height - 1) * self.width + col) as usize;
            if bot_idx < plane.cells.len() {
                plane.cells[bot_idx].char = '─';
            }
        }
        for r in 1..height.saturating_sub(1) {
            let left_idx = (r * self.width) as usize;
            if left_idx < plane.cells.len() {
                plane.cells[left_idx].char = '│';
            }
            let right_idx = (r * self.width + self.width - 1) as usize;
            if right_idx < plane.cells.len() {
                plane.cells[right_idx].char = '│';
            }
        }

        plane
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        if col < self.anchor_x
            || col >= self.anchor_x + self.width
            || row < self.anchor_y
            || row >= self.anchor_y + self.items.len() as u16
        {
            return false;
        }

        let idx = (row - self.anchor_y) as usize;
        if idx >= self.items.len() {
            return false;
        }

        if let crate::input::event::MouseEventKind::Down(_) = kind {
            return true;
        }
        false
    }
}
