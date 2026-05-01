//! Drag and drop system for framework apps.
//!
//! Provides `DragItem`, `DragGhost`, `DropTarget`, and `DragManager` for
//! declarative drag-and-drop with visual ghost rendering.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Color, Plane};

/// The current phase of a drag operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPhase {
    /// No drag in progress.
    Idle,
    /// A drag is actively being performed.
    Dragging,
    /// The drag ended over a valid drop target.
    Dropped,
    /// The drag ended outside any target or was cancelled.
    Cancelled,
}

/// A drag payload with associated data and the source it came from.
#[derive(Debug, Clone)]
pub struct DragItem<T> {
    /// The arbitrary data being dragged.
    pub data: T,
    /// Identifier of the source zone or item.
    pub source_id: usize,
}

/// A visual ghost rendered during a drag to indicate the item being dragged.
#[derive(Debug, Clone)]
pub struct DragGhost {
    /// Display label shown in the ghost.
    pub label: String,
    /// Width of the ghost in cells.
    pub width: u16,
    /// Height of the ghost in cells.
    pub height: u16,
}

impl DragGhost {
    /// Creates a `DragGhost` from a label string, auto-calculating width from the label.
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let width = label.width() as u16 + 2;
        Self {
            label,
            width,
            height: 1,
        }
    }

    /// Overrides the width and height of the ghost.
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Renders the ghost into a `Plane` positioned at `(x, y)` with z-index 9000.
    pub fn render(&self, x: u16, y: u16) -> Plane {
        let mut plane = Plane::new(9999, self.width, self.height);
        plane.set_z_index(9000);
        plane.set_absolute_position(x, y);

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

/// Manages active drag state, ghost position, and registered drop targets.
pub struct DragManager<T> {
    active: Option<DragItem<T>>,
    phase: DragPhase,
    ghost: Option<DragGhost>,
    ghost_x: u16,
    ghost_y: u16,
    targets: Vec<DropTarget<T>>,
}

/// A rectangular drop target with an associated ID.
#[derive(Debug, Clone)]
pub struct DropTarget<T> {
    /// Arbitrary ID identifying this target.
    pub id: T,
    /// Left edge column.
    pub x: u16,
    /// Top edge row.
    pub y: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl<T: Clone + 'static> DragManager<T> {
    /// Creates an idle `DragManager` with no active drag.
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

    /// Resets drag state: clears active item, phase, ghost, and all targets.
    pub fn clear(&mut self) {
        self.active = None;
        self.phase = DragPhase::Idle;
        self.ghost = None;
        self.targets.clear();
    }

    /// Registers a drop target rectangle with the given geometry and ID.
    pub fn register_target(&mut self, id: T, x: u16, y: u16, width: u16, height: u16) {
        self.targets.push(DropTarget {
            id,
            x,
            y,
            width,
            height,
        });
    }

    /// Begins a drag operation with the given data, source id, and ghost.
    pub fn start_drag(&mut self, data: T, source_id: usize, ghost: DragGhost) {
        self.active = Some(DragItem { data, source_id });
        self.phase = DragPhase::Dragging;
        self.ghost = Some(ghost);
    }

    /// Updates the ghost cursor position to `(x, y)`.
    pub fn move_ghost(&mut self, x: u16, y: u16) {
        self.ghost_x = x;
        self.ghost_y = y;
    }

    /// Ends the drag operation. Checks if the ghost overlaps any registered target.
    /// Returns `Some(target_id)` if dropped on a target, `None` otherwise.
    pub fn end_drag(&mut self) -> Option<T> {
        let target = self
            .targets
            .iter()
            .find(|t| {
                self.ghost_x >= t.x
                    && self.ghost_x < t.x.saturating_add(t.width)
                    && self.ghost_y >= t.y
                    && self.ghost_y < t.y.saturating_add(t.height)
            })
            .cloned();

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

    /// Cancels the active drag, setting phase to `Cancelled`.
    pub fn cancel(&mut self) {
        self.phase = DragPhase::Cancelled;
        self.active = None;
        self.ghost = None;
    }

    /// Returns the current `DragPhase`.
    pub fn phase(&self) -> DragPhase {
        self.phase
    }

    /// Returns `true` if phase is `Dragging`.
    pub fn is_dragging(&self) -> bool {
        self.phase == DragPhase::Dragging
    }

    /// Renders and returns the ghost `Plane` if a ghost exists, otherwise `None`.
    pub fn ghost_plane(&self) -> Option<Plane> {
        self.ghost
            .as_ref()
            .map(|g| g.render(self.ghost_x, self.ghost_y))
    }
}

impl<T: Clone + 'static> Default for DragManager<T> {
    fn default() -> Self {
        Self::new()
    }
}
