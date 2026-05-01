//! Slider widget for selecting a value within a range.
//!
//! A horizontal track with a draggable thumb indicator.

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A horizontal slider for selecting a value within a range.
pub struct Slider {
    id: WidgetId,
    value: f32,
    min: f32,
    max: f32,
    theme: Theme,
    on_change: Option<Box<dyn FnMut(f32)>>,
    last_area_width: std::cell::Cell<u16>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Slider {
    /// Creates a new slider with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            value: 0.5,
            min: 0.0,
            max: 1.0,
            theme: Theme::default(),
            on_change: None,
            last_area_width: std::cell::Cell::new(80),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 1)),
            dirty: true,
        }
    }

    /// Sets the minimum and maximum values for the slider.
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self.value = (min + max) / 2.0;
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Registers a callback when the slider value changes.
    pub fn on_change(mut self, f: impl FnMut(f32) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Sets the current value of the slider.
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
        self.dirty = true;
    }

    /// Returns the current value of the slider.
    pub fn value(&self) -> f32 {
        self.value
    }

    #[allow(unused)]
    fn value_to_position(&self) -> usize {
        let ratio = if self.max > self.min {
            (self.value - self.min) / (self.max - self.min)
        } else {
            0.5
        };
        (ratio * 100.0).round() as usize
    }
}

impl crate::framework::widget::Widget for Slider {
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
        self.last_area_width.set(area.width);
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;
        let height = plane.height as usize;

        let track_width = width.saturating_sub(4);
        let thumb_pos =
            ((self.value - self.min) / (self.max - self.min) * track_width as f32).round() as usize;
        let thumb_pos = thumb_pos.min(track_width);

        let fill_char = '-';
        for x in 0..track_width {
            let idx = (height / 2) as u16 * plane.width + (x + 1) as u16;
            let is_filled = x <= thumb_pos;
            if (idx as usize) < plane.cells.len() {
                plane.cells[idx as usize] = Cell {
                    char: fill_char,
                    fg: if is_filled {
                        self.theme.accent
                    } else {
                        self.theme.inactive_fg
                    },
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        let left_bracket = Cell {
            char: '[',
            fg: self.theme.fg,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        let right_bracket = Cell {
            char: ']',
            fg: self.theme.fg,
            bg: Color::Reset,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };

        let left_idx = (height / 2) as u16 * plane.width;
        let right_idx = left_idx + (width - 1) as u16;
        plane.cells[left_idx as usize] = left_bracket;
        plane.cells[right_idx as usize] = right_bracket;

        let thumb_idx = (height / 2) as u16 * plane.width + (1 + thumb_pos as u16);
        if (thumb_idx as usize) < plane.cells.len() {
            plane.cells[thumb_idx as usize] = Cell {
                char: 'O',
                fg: self.theme.bg,
                bg: self.theme.accent,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }

        plane
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        _row: u16,
    ) -> bool {
        match kind {
            crate::input::event::MouseEventKind::Down(_)
            | crate::input::event::MouseEventKind::Drag(_) => {
                let width = self.last_area_width.get();
                let track_width = width.saturating_sub(4);
                let rel_x = col.saturating_sub(1);
                if rel_x <= track_width {
                    let ratio = rel_x as f32 / track_width.max(1) as f32;
                    self.value = self.min + ratio * (self.max - self.min);
                    self.value = self.value.clamp(self.min, self.max);
                    if let Some(ref mut cb) = self.on_change {
                        cb(self.value);
                    }
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
