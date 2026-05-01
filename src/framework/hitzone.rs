//! Hit zone system for mouse event dispatch.
//!
//! Provides `HitZone` (with callbacks), `HitZoneGroup` (multi-zone dispatcher),
//! `ScopedZone` (simple geometry-only zone), and `ScopedZoneRegistry`
//! for per-frame scoped hit testing.

use crate::input::event::{KeyModifiers, MouseButton, MouseEventKind};
use std::time::{Duration, Instant};

/// Describes the multiplicity of a click sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickKind {
    /// A single click.
    Single,
    /// A double click within the timeout window.
    Double,
    /// A triple click within the timeout window.
    Triple,
}

/// Describes a drag interaction and its phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragState {
    /// Drag started at the given coordinates.
    /// Drag started at the given coordinates.
    Started {
        /// X coordinate.
        x: u16,
        /// Y coordinate.
        y: u16,
    },
    /// Drag moved to the given coordinates.
    /// Drag moved to the given coordinates.
    Moved {
        /// X coordinate.
        x: u16,
        /// Y coordinate.
        y: u16,
    },
    /// Drag ended at the given coordinates.
    /// Drag ended at the given coordinates.
    Ended {
        /// X coordinate.
        x: u16,
        /// Y coordinate.
        y: u16,
    },
}

/// A rectangular interactive zone with typed ID and mouse/drag callbacks.
pub struct HitZone<T> {
    /// Arbitrary ID identifying this zone.
    pub id: T,
    /// Left edge column.
    pub x: u16,
    /// Top edge row.
    pub y: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
    on_click: Option<Box<dyn FnMut(ClickKind)>>,
    on_right_click: Option<Box<dyn FnMut()>>,
    on_drag_start: Option<Box<dyn FnMut(DragState)>>,
    on_drag_move: Option<Box<dyn FnMut(DragState)>>,
    on_drag_end: Option<Box<dyn FnMut(DragState)>>,
    double_click_timeout: Duration,
    last_click_time: Option<Instant>,
    last_click_pos: Option<(u16, u16)>,
    click_count: u8,
    drag_active: bool,
}

impl<T: Clone + 'static> HitZone<T> {
    /// Creates a new `HitZone` with the given id and geometry.
    pub fn new(id: T, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            on_click: None,
            on_right_click: None,
            on_drag_start: None,
            on_drag_move: None,
            on_drag_end: None,
            double_click_timeout: Duration::from_millis(500),
            last_click_time: None,
            last_click_pos: None,
            click_count: 0,
            drag_active: false,
        }
    }

    /// Registers a callback invoked on left-click with the click multiplicity.
    pub fn on_click(mut self, f: impl FnMut(ClickKind) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked on right-click.
    pub fn on_right_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_right_click = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when drag starts within this zone.
    pub fn on_drag_start(mut self, f: impl FnMut(DragState) + 'static) -> Self {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when drag moves within this zone.
    pub fn on_drag_move(mut self, f: impl FnMut(DragState) + 'static) -> Self {
        self.on_drag_move = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when drag ends within this zone.
    pub fn on_drag_end(mut self, f: impl FnMut(DragState) + 'static) -> Self {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    /// Returns `true` if `(col, row)` lies within this zone's rectangle.
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.x && col < self.x.saturating_add(self.width)
            && row >= self.y && row < self.y.saturating_add(self.height)
    }

    /// Routes a mouse event to the appropriate callback.
    /// Automatically detects single/double/triple click.
    pub fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, _modifiers: KeyModifiers) {
        if !self.contains(col, row) {
            return;
        }

        match kind {
            MouseEventKind::Down(btn) => {
                if btn == MouseButton::Right {
                    if let Some(f) = self.on_right_click.as_mut() {
                        f();
                    }
                    return;
                }
                if btn != MouseButton::Left {
                    return;
                }

                let now = Instant::now();
                let is_double = self.last_click_time
                    .zip(self.last_click_pos)
                    .map(|(t, (px, py))| {
                        t.elapsed() < self.double_click_timeout
                            && (px as i32 - col as i32).abs() <= 1
                            && (py as i32 - row as i32).abs() <= 1
                    })
                    .unwrap_or(false);

                self.click_count = if is_double { 2 } else { 1 };
                self.last_click_time = Some(now);
                self.last_click_pos = Some((col, row));

                let kind = match self.click_count {
                    1 => ClickKind::Single,
                    2 => ClickKind::Double,
                    _ => ClickKind::Triple,
                };
                if let Some(f) = self.on_click.as_mut() {
                    f(kind);
                }

                if self.on_drag_start.is_some() {
                    self.drag_active = true;
                    if let Some(f) = self.on_drag_start.as_mut() {
                        f(DragState::Started { x: col, y: row });
                    }
                }
            }
            MouseEventKind::Drag(_)
                if self.drag_active => {
                    if let Some(f) = self.on_drag_move.as_mut() {
                        f(DragState::Moved { x: col, y: row });
                    }
                }
            MouseEventKind::Up(_)
                if self.drag_active => {
                    self.drag_active = false;
                    if let Some(f) = self.on_drag_end.as_mut() {
                        f(DragState::Ended { x: col, y: row });
                    }
                }
            _ => {}
        }
    }

    /// Like `handle_mouse` but returns `Some(self.id.clone())` if the event hit this zone.
    pub fn dispatch_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, modifiers: KeyModifiers) -> Option<T> {
        if !self.contains(col, row) {
            return None;
        }
        self.handle_mouse(kind, col, row, modifiers);
        Some(self.id.clone())
    }
}

/// A group of `HitZone`s that dispatches to the first matching zone.
pub struct HitZoneGroup<T> {
    zones: Vec<HitZone<T>>,
}

impl<T: Clone + 'static> HitZoneGroup<T> {
    /// Creates an empty group.
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    /// Adds a pre-built `HitZone` to the group.
    pub fn zone(mut self, zone: HitZone<T>) -> Self {
        self.zones.push(zone);
        self
    }

    /// Convenience: adds a full-width single-row zone with a click callback.
    pub fn add_row(&mut self, id: T, y: u16, width: u16, f: impl FnMut(ClickKind) + 'static) {
        let zone = HitZone::new(id, 0, y, width, 1).on_click(f);
        self.zones.push(zone);
    }

    /// Returns a slice of all zones.
    pub fn zones(&self) -> &[HitZone<T>] {
        &self.zones
    }

    /// Returns a mutable reference to the zone vector.
    pub fn zones_mut(&mut self) -> &mut Vec<HitZone<T>> {
        &mut self.zones
    }

    /// Iterates zones in order and dispatches to the first containing the event point.
    /// Returns `Some(id)` of the hit zone, or `None`.
    pub fn dispatch_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, modifiers: KeyModifiers) -> Option<T> {
        for zone in self.zones.iter_mut() {
            if zone.contains(col, row) {
                zone.handle_mouse(kind, col, row, modifiers);
                return Some(zone.id.clone());
            }
        }
        None
    }
}

/// A lightweight rectangular zone with only geometry and an ID — no callbacks.
pub struct ScopedZone<T> {
    /// Arbitrary ID identifying this zone.
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

impl<T> ScopedZone<T> {
    /// Creates a new `ScopedZone` with the given id and geometry.
    pub fn new(id: T, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { id, x, y, width, height }
    }

    /// Returns `true` if `(col, row)` lies within this zone's rectangle.
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.x && col < self.x.saturating_add(self.width)
            && row >= self.y && row < self.y.saturating_add(self.height)
    }
}

/// A registry of `ScopedZone`s for bulk per-frame dispatch.
#[derive(Default)]
pub struct ScopedZoneRegistry<T> {
    zones: Vec<ScopedZone<T>>,
}

impl<T: Clone + 'static> ScopedZoneRegistry<T> {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    /// Removes all zones (call at the start of each frame).
    pub fn clear(&mut self) {
        self.zones.clear();
    }

    /// Adds a pre-built `ScopedZone`.
    pub fn add(&mut self, zone: ScopedZone<T>) {
        self.zones.push(zone);
    }

    /// Convenience: constructs and adds a `ScopedZone` from raw coordinates.
    pub fn register(&mut self, id: T, x: u16, y: u16, width: u16, height: u16) {
        self.zones.push(ScopedZone::new(id, x, y, width, height));
    }

    /// Returns `Some(id)` of the first zone containing `(col, row)`, or `None`.
    pub fn dispatch(&self, col: u16, row: u16) -> Option<T> {
        for zone in &self.zones {
            if zone.contains(col, row) {
                return Some(zone.id.clone());
            }
        }
        None
    }

    /// Returns a slice of all registered zones.
    pub fn zones(&self) -> &[ScopedZone<T>] {
        &self.zones
    }
}

impl<T: Clone + 'static> Default for HitZoneGroup<T> {
    fn default() -> Self {
        Self::new()
    }
}