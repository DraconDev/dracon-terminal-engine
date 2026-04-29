//! Selectable list widget with keyboard and mouse navigation.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::scroll::ScrollState;
use crate::framework::theme::Theme;
use ratatui::layout::Rect;

/// A generic selectable list widget.
///
/// Renders items with selection highlighting and provides keyboard/mouse navigation.
pub struct List<T> {
    items: Vec<T>,
    selected: usize,
    offset: usize,
    visible_count: usize,
    theme: Theme,
    on_select: Option<Box<dyn FnMut(&T)>>,
    item_height: u16,
    width: u16,
}

impl<T: Clone + ToString> List<T> {
    /// Creates a new `List` with the given items and default theme.
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            selected: 0,
            offset: 0,
            visible_count: 10,
            theme: Theme::default(),
            on_select: None,
            item_height: 1,
            width: 40,
        }
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the height of each item row in cells.
    pub fn with_item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    /// Sets the width of the list in cells.
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Registers a callback invoked when an item is selected (Enter or click).
    pub fn on_select(mut self, f: impl FnMut(&T) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Returns the index of the currently selected item.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Returns a reference to the currently selected item, or `None`.
    pub fn get_selected(&self) -> Option<&T> {
        self.items.get(self.selected)
    }

    /// Returns the total number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `(start, end)` indices of the currently visible items.
    pub fn viewport(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.visible_count).min(self.items.len());
        (start, end)
    }

    /// Scrolls to and selects the item at `index`.
    pub fn scroll_to(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        self.selected = index;
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + self.visible_count {
            self.offset = self.selected.saturating_sub(self.visible_count) + 1;
        }
    }

    /// Returns a `ScrollState` reflecting the current scroll position.
    pub fn scroll_state(&self) -> ScrollState {
        ScrollState {
            offset: self.offset,
            content_height: self.items.len(),
            viewport_height: self.visible_count,
        }
    }

    /// Sets how many items are visible at once.
    pub fn set_visible_count(&mut self, count: usize) {
        self.visible_count = count;
    }

    /// Renders the list into a `Plane` within the given `area`.
    pub fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let visible_items: Vec<_> = self.items.iter()
            .skip(self.offset)
            .take(self.visible_count)
            .collect();

        for (i, item) in visible_items.iter().enumerate() {
            let row = i as u16;
            let is_selected = self.offset + i == self.selected;
            let bg = if is_selected { self.theme.selection_bg } else { self.theme.bg };
            let fg = if is_selected { self.theme.selection_fg } else { self.theme.fg };
            let style = if is_selected { Styles::BOLD } else { Styles::empty() };

            for col in 0..area.width {
                let idx = (row * area.width + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: ' ',
                        fg,
                        bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }

            let text = item.to_string();
            let label_len = text.len().min((area.width as usize).saturating_sub(2));
            for (j, ch) in text.chars().take(label_len).enumerate() {
                let idx = (row * area.width + 1 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].style = style;
                }
            }
        }

        plane
    }

    /// Handles a mouse event. Returns `true` if consumed.
    pub fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        if col >= self.width || row >= self.visible_count as u16 {
            return false;
        }
        let idx = self.offset + row as usize;
        if idx >= self.items.len() {
            return false;
        }
        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                self.selected = idx;
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.items[idx]);
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                self.offset = (self.offset + 1).min(self.items.len().saturating_sub(self.visible_count));
                true
            }
            crate::input::event::MouseEventKind::ScrollUp => {
                self.offset = self.offset.saturating_sub(1);
                true
            }
            _ => false,
        }
    }

    /// Handles a key event. Returns `true` if consumed.
    pub fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Down => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                    if self.selected >= self.offset + self.visible_count {
                        self.offset = self.selected.saturating_sub(self.visible_count) + 1;
                    }
                }
                true
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.offset {
                        self.offset = self.selected;
                    }
                }
                true
            }
            KeyCode::Home => {
                self.selected = 0;
                self.offset = 0;
                true
            }
            KeyCode::End => {
                self.selected = self.items.len().saturating_sub(1);
                self.offset = self.items.len().saturating_sub(self.visible_count);
                true
            }
            KeyCode::PageDown => {
                self.selected = (self.selected + self.visible_count).min(self.items.len().saturating_sub(1));
                if self.selected >= self.offset + self.visible_count {
                    self.offset = self.selected.saturating_sub(self.visible_count) + 1;
                }
                true
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(self.visible_count);
                self.offset = self.selected;
                true
            }
            KeyCode::Enter => {
                if let Some(f) = self.on_select.as_mut() {
                    f(&self.items[self.selected]);
                }
                true
            }
            _ => false,
        }
    }
}