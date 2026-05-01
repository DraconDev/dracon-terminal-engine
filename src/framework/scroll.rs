//! Scroll state and container widgets.
//!
//! Provides [`ScrollState`] for tracking scroll position, content height,
//! and viewport height. [`ScrollContainer`] wraps content and renders a
//! scrollbar with customizable colors.

use crate::compositor::{Cell, Color, Plane, Styles};
use ratatui::layout::Rect;

/// Holds scroll position, content height, and viewport height for virtual scrolling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollState {
    /// Number of rows scrolled off the top.
    pub offset: usize,
    /// Total number of rows in the content.
    pub content_height: usize,
    /// Number of rows visible in the viewport.
    pub viewport_height: usize,
}

impl ScrollState {
    /// Returns the maximum valid offset: `content_height - viewport_height`.
    pub fn max_offset(&self) -> usize {
        self.content_height.saturating_sub(self.viewport_height)
    }

    /// Returns the page size (one viewport minus one row, minimum 1).
    pub fn page_size(&self) -> usize {
        self.viewport_height.saturating_sub(1).max(1)
    }

    /// Scrolls up by `n` rows, clamped to 0.
    pub fn scroll_up(&mut self, n: usize) {
        self.offset = self.offset.saturating_sub(n);
    }

    /// Scrolls down by `n` rows, clamped to `max_offset`.
    pub fn scroll_down(&mut self, n: usize) {
        self.offset = (self.offset + n).min(self.max_offset());
    }

    /// Sets the scroll offset to a specific value, clamped to `max_offset`.
    pub fn scroll_to(&mut self, offset: usize) {
        self.offset = offset.min(self.max_offset());
    }

    /// Scrolls to the top (offset = 0).
    pub fn scroll_to_top(&mut self) {
        self.offset = 0;
    }

    /// Scrolls to the bottom (offset = max_offset).
    pub fn scroll_to_bottom(&mut self) {
        self.offset = self.max_offset();
    }

    /// Scrolls up by one page.
    pub fn scroll_page_up(&mut self) {
        self.scroll_up(self.page_size());
    }

    /// Scrolls down by one page.
    pub fn scroll_page_down(&mut self) {
        self.scroll_down(self.page_size());
    }
}

/// A scrollable container with an optional scrollbar.
///
/// Wraps `ScrollState` and provides keyboard/mouse handling and scrollbar rendering.
#[derive(Debug, Clone)]
pub struct ScrollContainer {
    state: ScrollState,
    scrollbar_visible: bool,
    scrollbar_width: u16,
    scrollbar_track: Color,
    scrollbar_thumb: Color,
}

impl ScrollContainer {
    /// Creates a new `ScrollContainer` with a visible scrollbar.
    pub fn new() -> Self {
        Self {
            state: ScrollState::default(),
            scrollbar_visible: true,
            scrollbar_width: 1,
            scrollbar_track: Color::Rgb(30, 30, 40),
            scrollbar_thumb: Color::Rgb(80, 80, 100),
        }
    }

    /// Sets the total content height in rows.
    pub fn with_content_height(mut self, height: usize) -> Self {
        self.state.content_height = height;
        self
    }

    /// Sets the viewport height in rows.
    pub fn with_viewport_height(mut self, height: usize) -> Self {
        self.state.viewport_height = height;
        self
    }

    /// Sets whether the scrollbar is rendered.
    pub fn with_scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar_visible = visible;
        self
    }

    /// Returns an immutable reference to the scroll state.
    pub fn state(&self) -> &ScrollState {
        &self.state
    }

    /// Returns a mutable reference to the scroll state.
    pub fn state_mut(&mut self) -> &mut ScrollState {
        &mut self.state
    }

    /// Handles arrow keys, Page Up/Down, Home/End for scrolling.
    /// Returns `true` if the key was consumed.
    pub fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Up => {
                self.state.scroll_up(1);
                true
            }
            KeyCode::Down => {
                self.state.scroll_down(1);
                true
            }
            KeyCode::PageUp => {
                self.state.scroll_page_up();
                true
            }
            KeyCode::PageDown => {
                self.state.scroll_page_down();
                true
            }
            KeyCode::Home => {
                self.state.scroll_to_top();
                true
            }
            KeyCode::End => {
                self.state.scroll_to_bottom();
                true
            }
            _ => false,
        }
    }

    /// Handles scroll wheel mouse events. Returns `true` if consumed.
    pub fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        _col: u16,
        _row: u16,
    ) -> bool {
        use crate::input::event::MouseEventKind;
        let visible = self.state.content_height > self.state.viewport_height;
        if !visible {
            return false;
        }

        match kind {
            MouseEventKind::ScrollDown => {
                self.state.scroll_down(3);
                true
            }
            MouseEventKind::ScrollUp => {
                self.state.scroll_up(3);
                true
            }
            _ => false,
        }
    }

    /// Renders the scrollbar as a `Plane` with a thumb and track.
    pub fn render_scrollbar(&self, area: Rect) -> Plane {
        let total = self.state.content_height;
        let visible = self.state.viewport_height;
        let offset = self.state.offset;

        let mut plane = Plane::new(0, self.scrollbar_width, area.height);

        if total <= visible || !self.scrollbar_visible {
            return plane;
        }

        let thumb_len = ((visible as f32 / total as f32) * area.height as f32).ceil() as usize;
        let thumb_len = thumb_len.max(1);
        let max_offset = total.saturating_sub(visible);
        let thumb_pos = if max_offset == 0 {
            0
        } else {
            (offset * (area.height as usize - thumb_len)).checked_div(max_offset).unwrap()
        };

        let thumb_char = '█';
        let track_char = '░';

        for y in 0..area.height as usize {
            let char = if y >= thumb_pos && y < thumb_pos + thumb_len {
                thumb_char
            } else {
                track_char
            };
            let idx = y * self.scrollbar_width as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char,
                    fg: if y >= thumb_pos && y < thumb_pos + thumb_len {
                        self.scrollbar_thumb
                    } else {
                        self.scrollbar_track
                    },
                    bg: Color::Reset,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }
}

impl Default for ScrollContainer {
    fn default() -> Self {
        Self::new()
    }
}
