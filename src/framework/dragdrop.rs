//! Drag and drop system for framework apps.
//!
//! Provides `DragSource`, `DropTarget`, `DragGhost`, and `DragManager` for
//! declarative drag-and-drop with visual ghost rendering.

use crate::compositor::{Color, Plane, Styles};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPhase {
    Idle,
    Dragging,
    Dropped,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct DragItem<T> {
    pub data: T,
    pub source_id: usize,
}

#[derive(Debug, Clone)]
pub struct DragGhost {
    pub label: String,
    pub width: u16,
    pub height: u16,
}

impl DragGhost {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let width = label.len() as u16 + 2;
        Self { label, width, height: 1 }
    }

    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn render(&self, x: u16, y: u16) -> Plane {
        let mut plane = Plane::new(9999, self.width, self.height);
        plane.set_z_index(9000);

        let bg = Color::Ansi(236);
        let fg = Color::Ansi(250);

        for i in 0..(self.width * self.height) as usize {
            if i < plane.cells.len() {
                plane.cells[i].bg = bg;
                plane.cells[i].fg = fg;
                plane.cells[i].char = ' ';
            }
        }

        for (i, ch) in self.label.chars().enumerate() {
            let idx = i + 1;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
            }
        }

        plane
    }
}

pub struct DragManager<T> {
    active: Option<DragItem<T>>,
    phase: DragPhase,
    ghost: Option<DragGhost>,
    ghost_x: u16,
    ghost_y: u16,
    targets: Vec<DropTarget<T>>,
}

#[derive(Debug, Clone)]
pub struct DropTarget<T> {
    pub id: T,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl<T: Clone + 'static> DragManager<T> {
    pub fn new() -> Self {
        Self {
            active: None,
            phase: DragPhase::Idle,
            ghost: None,
            ghost_x: 0,
            ghost_y: 0,
            targets: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.active = None;
        self.phase = DragPhase::Idle;
        self.ghost = None;
        self.targets.clear();
    }

    pub fn register_target(&mut self, id: T, x: u16, y: u16, width: u16, height: u16) {
        self.targets.push(DropTarget { id, x, y, width, height });
    }

    pub fn start_drag(&mut self, data: T, source_id: usize, ghost: DragGhost) {
        self.active = Some(DragItem { data, source_id });
        self.phase = DragPhase::Dragging;
        self.ghost = Some(ghost);
    }

    pub fn move_ghost(&mut self, x: u16, y: u16) {
        self.ghost_x = x;
        self.ghost_y = y;
    }

    pub fn end_drag(&mut self) -> Option<T> {
        let target = self.targets.iter().find(|t| {
            self.ghost_x >= t.x && self.ghost_x < t.x.saturating_add(t.width)
                && self.ghost_y >= t.y && self.ghost_y < t.y.saturating_add(t.height)
        }).cloned();

        let result = target.map(|t| t.id);

        if result.is_some() {
            self.phase = DragPhase::Dropped;
        } else {
            self.phase = DragPhase::Cancelled;
        }

        self.active = None;
        self.ghost = None;

        result
    }

    pub fn cancel(&mut self) {
        self.phase = DragPhase::Cancelled;
        self.active = None;
        self.ghost = None;
    }

    pub fn phase(&self) -> DragPhase {
        self.phase
    }

    pub fn is_dragging(&self) -> bool {
        self.phase == DragPhase::Dragging
    }

    pub fn ghost_plane(&self) -> Option<Plane> {
        self.ghost.as_ref().map(|g| g.render(self.ghost_x, self.ghost_y))
    }
}

impl<T: Clone + 'static> Default for DragManager<T> {
    fn default() -> Self {
        Self::new()
    }
}