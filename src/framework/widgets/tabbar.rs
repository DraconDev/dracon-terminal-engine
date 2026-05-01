//! Tab bar widget.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A horizontal tab bar widget with clickable and keyboard-navigable tabs.
pub struct TabBar {
    id: WidgetId,
    tabs: Vec<String>,
    active: usize,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl TabBar {
    /// Creates a new `TabBar` from a list of tab labels.
    pub fn new(tabs: Vec<&str>) -> Self {
        Self {
            id: WidgetId::default_id(),
            tabs: tabs.iter().map(|s| s.to_string()).collect(),
            active: 0,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 3)),
            dirty: true,
        }
    }

    /// Creates a new `TabBar` with the given widget ID and tab labels.
    pub fn new_with_id(id: WidgetId, tabs: Vec<&str>) -> Self {
        Self {
            id,
            tabs: tabs.iter().map(|s| s.to_string()).collect(),
            active: 0,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 3)),
            dirty: true,
        }
    }

    /// Sets the rendering theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Returns the index of the currently active tab.
    pub fn active(&self) -> usize {
        self.active
    }

    /// Sets the active tab index, clamped to the valid range.
    pub fn set_active(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active = idx;
        }
    }
}

impl crate::framework::widget::Widget for TabBar {
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
        10
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
        let tab_count = self.tabs.len().max(1);
        let tab_width = (area.width / tab_count as u16).max(1);

        for (i, tab) in self.tabs.iter().enumerate() {
            let x = (i as u16) * tab_width;
            let is_active = i == self.active;

            let bg = if is_active { self.theme.active_bg } else { self.theme.bg };
            let fg = if is_active { self.theme.accent } else { self.theme.inactive_fg };
            let style = if is_active { Styles::BOLD | Styles::UNDERLINE } else { Styles::empty() };

            for col in 0..tab_width {
                let idx = col as usize;
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

            let label_len = tab.width().min((tab_width as usize).saturating_sub(2));
            let start_col = if tab_width > 2 { 1 } else { 0 };
            for (j, ch) in tab.chars().take(label_len).enumerate() {
                let idx = (start_col + j);
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = style;
                }
            }

            let _zone = HitZone::new(i, x, area.y, tab_width, area.height);
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Left => {
                if self.active > 0 {
                    self.active -= 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Right => {
                if self.active + 1 < self.tabs.len() {
                    self.active += 1;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, _row: u16) -> bool {
        let tab_count = self.tabs.len().max(1);
        let tab_width = (self.area.get().width / tab_count as u16).max(1);
        let idx = col / tab_width;
        if idx >= tab_count as u16 {
            return false;
        }

        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                self.active = idx as usize;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }
}