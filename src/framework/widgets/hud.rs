//! Heads-up display (HUD) widget.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A heads-up display overlay positioned at the top-left corner.
///
/// Renders text and progress gauges as planes with a fixed z-index.
pub struct Hud {
    id: WidgetId,
    z_index: u16,
    visible: bool,
    width: u16,
    height: u16,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    theme: Theme,
}

impl Hud {
    /// Creates a new `Hud` with the given z-index.
    pub fn new(z_index: u16) -> Self {
        Self {
            id: WidgetId::default_id(),
            z_index,
            visible: true,
            width: 30,
            height: 10,
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 10)),
            dirty: true,
            theme: Theme::default(),
        }
    }

    /// Creates a new `Hud` with the given widget ID and z-index.
    pub fn new_with_id(id: WidgetId, z_index: u16) -> Self {
        Self {
            id,
            z_index,
            visible: true,
            width: 30,
            height: 10,
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 10)),
            dirty: true,
            theme: Theme::default(),
        }
    }

    /// Sets the width and height of the HUD plane.
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Returns the fixed position of the HUD (always `(0, 0)`).
    pub fn position(&self) -> (u16, u16) {
        (0, 0)
    }

    /// Returns whether the HUD is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Shows the HUD.
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hides the HUD.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Renders a text string at offset `(x, y)` with the given foreground and background colors.
    pub fn render_text(&self, x: u16, y: u16, text: &str, fg: Color, bg: Color) -> Plane {
        let mut plane = Plane::new(0, self.width, self.height);
        plane.z_index = self.z_index as i32;

        let text_len = text
            .width()
            .min((self.width as usize).saturating_sub(x as usize));
        let start_idx = (y * self.width + x) as usize;

        for (i, ch) in text.chars().take(text_len).enumerate() {
            let idx = start_idx + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: ch,
                    fg,
                    bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }

    /// Renders a labeled horizontal progress gauge at offset `(x, y)`.
    ///
    /// `value` is compared against `max` to determine the filled portion.
    /// The gauge uses '█' for filled cells and '░' for empty cells.
    pub fn render_gauge(
        &self,
        x: u16,
        y: u16,
        label: &str,
        value: f32,
        max: f32,
        width: u16,
    ) -> Plane {
        let mut plane = Plane::new(0, self.width, self.height);
        plane.z_index = self.z_index as i32;

        let label_len = label.width().min(width as usize);
        let start_idx = (y * self.width + x) as usize;

        for (i, ch) in label.chars().take(label_len).enumerate() {
            let idx = start_idx + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = Color::Reset;
            }
        }

        let bar_start = start_idx + label_len + 1;
        let filled = if max > 0.0 {
            ((value / max) * (width as f32 - label_len as f32 - 2.0).max(0.0)) as usize
        } else {
            0
        };

        for i in 0..(width as usize - label_len - 2) {
            let idx = bar_start + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = if i < filled { '█' } else { '░' };
                plane.cells[idx].fg = if i < filled {
                    self.theme.success
                } else {
                    self.theme.outline
                };
                plane.cells[idx].bg = Color::Reset;
            }
        }

        plane
    }
}

impl crate::framework::widget::Widget for Hud {
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

    fn z_index(&self) -> u16 {
        self.z_index
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

    fn render(&self, _area: Rect) -> Plane {
        let mut plane = Plane::new(0, self.width, self.height);
        plane.z_index = self.z_index as i32;

        if !self.visible {
            for cell in &mut plane.cells {
                cell.transparent = true;
            }
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

    fn handle_mouse(
        &mut self,
        _kind: crate::input::event::MouseEventKind,
        _col: u16,
        _row: u16,
    ) -> bool {
        false
    }
}
