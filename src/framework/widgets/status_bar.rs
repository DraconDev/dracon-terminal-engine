//! Status bar widget for displaying contextual information.
//!
//! A horizontal bar typically at the bottom of the screen showing status items.

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A segment of the status bar with optional content.
pub struct StatusSegment {
    /// The text for this segment.
    pub text: String,
    /// The foreground color for this segment.
    pub fg: Color,
    /// The background color for this segment.
    pub bg: Color,
}

impl StatusSegment {
    /// Creates a new segment with the given text.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }

    /// Sets the foreground color for this segment.
    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    /// Sets the background color for this segment.
    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }
}

/// A horizontal status bar displaying segments of information.
pub struct StatusBar {
    /// The widget ID for this status bar.
    id: WidgetId,
    /// The segments to display.
    segments: Vec<StatusSegment>,
    /// The theme for this widget.
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl StatusBar {
    /// Creates a new status bar with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            segments: Vec::new(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 1)),
            dirty: true,
        }
    }

    /// Adds a segment to the status bar.
    pub fn add_segment(mut self, segment: StatusSegment) -> Self {
        self.segments.push(segment);
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl crate::framework::widget::Widget for StatusBar {
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
        50
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
        plane.z_index = 50;

        let width = plane.cells.len() / plane.height as usize;
        let _height = plane.height as usize;

        if self.segments.is_empty() {
            let default_text = "Ready";
            for (i, c) in default_text.chars().take(width).enumerate() {
                let idx = i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: self.theme.fg,
                        bg: self.theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
            return plane;
        }

        let total_segments = self.segments.len();
        let segment_width = width / total_segments.max(1);

        for (seg_idx, segment) in self.segments.iter().enumerate() {
            let start_x = seg_idx * segment_width;
            for (i, c) in segment
                .text
                .chars()
                .take(segment_width.saturating_sub(1))
                .enumerate()
            {
                let idx = start_x + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: if segment.fg == Color::Reset {
                            self.theme.fg
                        } else {
                            segment.fg
                        },
                        bg: if segment.bg == Color::Reset {
                            self.theme.bg
                        } else {
                            segment.bg
                        },
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        plane
    }
}
