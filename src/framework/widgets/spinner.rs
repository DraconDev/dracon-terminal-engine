//! Spinner widget.
//!
//! An animated loading spinner with configurable frame sequence.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;
use std::time::{Duration, Instant};

/// A spinner widget.
pub struct Spinner {
    id: WidgetId,
    frames: Vec<char>,
    current_frame: usize,
    last_update: Instant,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Spinner {
    /// Creates a new spinner with the given id.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            frames: vec!['|', '/', '-', '\\'],
            current_frame: 0,
            last_update: Instant::now(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 10, 1)),
            dirty: true,
        }
    }

    /// Sets the theme for this spinner.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the frames for this spinner.
    pub fn with_frames(mut self, frames: Vec<char>) -> Self {
        if !frames.is_empty() {
            self.frames = frames;
        }
        self
    }

    /// Advances the spinner to the next frame.
    pub fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(100) {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_update = now;
            self.dirty = true;
        }
    }

    /// Returns the current frame character.
    pub fn current_frame(&self) -> char {
        self.frames[self.current_frame]
    }
}

impl crate::framework::widget::Widget for Spinner {
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
        let height = plane.height as usize;

        let frame = self.frames[self.current_frame];
        let center_x = width / 2;
        let center_y = height / 2;

        let idx = (center_y as u16 * plane.width + center_x as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: frame,
                fg: self.theme.primary,
                bg: self.theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::KeyEventKind;
        if key.kind != KeyEventKind::Press {
            return false;
        }
        false
    }
}
