use crate::input::event::{KeyModifiers, MouseButton, MouseEventKind};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickKind {
    Single,
    Double,
    Triple,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragState {
    Started { x: u16, y: u16 },
    Moved { x: u16, y: u16 },
    Ended { x: u16, y: u16 },
}

pub struct HitZone<T> {
    pub id: T,
    pub x: u16,
    pub y: u16,
    pub width: u16,
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

    pub fn on_click(mut self, f: impl FnMut(ClickKind) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn on_right_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_right_click = Some(Box::new(f));
        self
    }

    pub fn on_drag_start(mut self, f: impl FnMut(DragState) + 'static) -> Self {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    pub fn on_drag_move(mut self, f: impl FnMut(DragState) + 'static) -> Self {
        self.on_drag_move = Some(Box::new(f));
        self
    }

    pub fn on_drag_end(mut self, f: impl FnMut(DragState) + 'static) -> Self {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.x && col < self.x.saturating_add(self.width)
            && row >= self.y && row < self.y.saturating_add(self.height)
    }

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
            MouseEventKind::Drag(_) => {
                if self.drag_active {
                    if let Some(f) = self.on_drag_move.as_mut() {
                        f(DragState::Moved { x: col, y: row });
                    }
                }
            }
            MouseEventKind::Up(_) => {
                if self.drag_active {
                    self.drag_active = false;
                    if let Some(f) = self.on_drag_end.as_mut() {
                        f(DragState::Ended { x: col, y: row });
                    }
                }
            }
            _ => {}
        }
    }

    pub fn dispatch_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, modifiers: KeyModifiers) -> Option<T> {
        if !self.contains(col, row) {
            return None;
        }
        self.handle_mouse(kind, col, row, modifiers);
        Some(self.id.clone())
    }
}

pub struct HitZoneGroup<T> {
    zones: Vec<HitZone<T>>,
}

impl<T: Clone + 'static> HitZoneGroup<T> {
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    pub fn zone(mut self, zone: HitZone<T>) -> Self {
        self.zones.push(zone);
        self
    }

    pub fn add_row(&mut self, id: T, y: u16, width: u16, f: impl FnMut(ClickKind) + 'static) {
        let zone = HitZone::new(id, 0, y, width, 1).on_click(f);
        self.zones.push(zone);
    }

    pub fn zones(&self) -> &[HitZone<T>] {
        &self.zones
    }

    pub fn zones_mut(&mut self) -> &mut Vec<HitZone<T>> {
        &mut self.zones
    }

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

impl<T: Clone + 'static> Default for HitZoneGroup<T> {
    fn default() -> Self {
        Self::new()
    }
}