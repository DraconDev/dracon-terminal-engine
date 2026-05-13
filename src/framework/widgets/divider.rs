//! Divider widget - horizontal or vertical separator.
//!
//! A simple widget that renders a divider line with optional label.

use crate::compositor::Plane;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// Direction of the divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerDirection {
    /// Horizontal divider (renders ────)
    Horizontal,
    /// Vertical divider (renders │││)
    Vertical,
}

/// Style of the divider line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerStyle {
    /// Solid line (─ or │)
    Solid,
    /// Dashed line (- - or | |)
    Dashed,
    /// Double line (= or ║)
    Double,
    /// Bold line (━ or ┃)
    Bold,
}

/// A simple divider/separator widget.
pub struct Divider {
    id: WidgetId,
    direction: DividerDirection,
    style: DividerStyle,
    label: Option<String>,
    label_position: LabelPosition,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

/// Position of the label relative to the divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelPosition {
    Left,
    Center,
    Right,
}

impl Divider {
    /// Creates a new horizontal divider.
    pub fn new() -> Self {
        Self {
            id: WidgetId::default_id(),
            direction: DividerDirection::Horizontal,
            style: DividerStyle::Solid,
            label: None,
            label_position: LabelPosition::Center,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 1)),
            dirty: true,
        }
    }

    /// Creates a vertical divider.
    pub fn vertical() -> Self {
        Self {
            id: WidgetId::default_id(),
            direction: DividerDirection::Vertical,
            style: DividerStyle::Solid,
            label: None,
            label_position: LabelPosition::Center,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 1, 20)),
            dirty: true,
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the direction (horizontal or vertical).
    pub fn direction(mut self, dir: DividerDirection) -> Self {
        self.direction = dir;
        self.dirty = true;
        self
    }

    /// Sets the line style.
    pub fn style(mut self, style: DividerStyle) -> Self {
        self.style = style;
        self.dirty = true;
        self
    }

    /// Adds a label to the divider.
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self.dirty = true;
        self
    }

    /// Sets the position of the label.
    pub fn label_position(mut self, pos: LabelPosition) -> Self {
        self.label_position = pos;
        self.dirty = true;
        self
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: Option<&str>) {
        self.label = label.map(|s| s.to_string());
        self.dirty = true;
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::framework::widget::Widget for Divider {
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
        5
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
        plane.z_index = 5;
        plane.fill_bg(self.theme.bg);

        match self.direction {
            DividerDirection::Horizontal => {
                self.render_horizontal(&mut plane, area);
            }
            DividerDirection::Vertical => {
                self.render_vertical(&mut plane, area);
            }
        }

        plane
    }

    fn handle_key(&mut self, _key: crate::input::event::KeyEvent) -> bool {
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

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}

impl Divider {
    fn render_horizontal(&self, plane: &mut Plane, area: Rect) {
        let (line_char, space_char) = match self.style {
            DividerStyle::Solid => ('─', ' '),
            DividerStyle::Dashed => ('-', ' '),
            DividerStyle::Double => ('═', ' '),
            DividerStyle::Bold => ('━', ' '),
        };

        let has_label = self.label.is_some();
        let label_text = self.label.as_deref().unwrap_or("");
        let label_len = label_text.len() as u16;

        // Calculate line segments
        let available_width = area.width;
        let label_area = if has_label { label_len + 2 } else { 0 };
        let line_width = available_width.saturating_sub(label_area);

        // Split line into left/right around label
        let left_width = match self.label_position {
            LabelPosition::Left => 0,
            LabelPosition::Center => line_width / 2,
            LabelPosition::Right => line_width.saturating_sub(1),
        };
        let right_width = line_width.saturating_sub(left_width);

        let mut x = 0u16;

        // Draw left line
        for _ in 0..left_width {
            if x < area.width {
                let idx = x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = line_char;
                    plane.cells[idx].fg = self.theme.outline;
                }
                x += 1;
            }
        }

        // Draw label with surrounding spaces
        if has_label {
            // Opening space
            if x < area.width {
                let idx = x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = space_char;
                    plane.cells[idx].fg = self.theme.outline;
                }
                x += 1;
            }

            // Label text
            for ch in label_text.chars() {
                if x < area.width {
                    let idx = x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = self.theme.fg_muted;
                        plane.cells[idx].style = crate::compositor::Styles::DIM;
                    }
                    x += 1;
                }
            }

            // Closing space
            if x < area.width {
                let idx = x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = space_char;
                    plane.cells[idx].fg = self.theme.outline;
                }
                x += 1;
            }
        }

        // Draw right line
        for _ in 0..right_width {
            if x < area.width {
                let idx = x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = line_char;
                    plane.cells[idx].fg = self.theme.outline;
                }
                x += 1;
            }
        }
    }

    fn render_vertical(&self, plane: &mut Plane, area: Rect) {
        let line_char = match self.style {
            DividerStyle::Solid => '│',
            DividerStyle::Dashed => '|',
            DividerStyle::Double => '║',
            DividerStyle::Bold => '┃',
        };

        let has_label = self.label.is_some();
        let label_text = self.label.as_deref().unwrap_or("");

        // Draw vertical line
        for y in 0..area.height {
            let idx = (y * area.width) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = line_char;
                plane.cells[idx].fg = self.theme.outline;
            }
        }

        // Draw label centered vertically
        if has_label && area.height >= 3 {
            let label_y = area.height / 2;

            // Draw label vertically (each char on a separate row)
            for (char_index, ch) in label_text.chars().enumerate() {
                let y = label_y.saturating_sub(char_index as u16);
                if y < area.height {
                    let idx = (y * area.width + 1) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = self.theme.fg_muted;
                        plane.cells[idx].style = crate::compositor::Styles::DIM;
                    }
                }
            }
        }
    }
}