//! Tab bar widget.

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use ratatui::layout::Rect;

/// A horizontal tab bar widget with clickable and keyboard-navigable tabs.
pub struct TabBar {
    tabs: Vec<String>,
    active: usize,
    theme: Theme,
}

impl TabBar {
    /// Creates a new `TabBar` from a list of tab labels.
    pub fn new(tabs: Vec<&str>) -> Self {
        Self {
            tabs: tabs.iter().map(|s| s.to_string()).collect(),
            active: 0,
            theme: Theme::default(),
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

    /// Renders the tab bar into a `Plane` and returns hit zones for each tab.
    ///
    /// Returns `(plane, hit_zones)` where hit zones have `id = tab_index`.
    pub fn render(&self, area: Rect) -> (Plane, Vec<HitZone<usize>>) {
        let mut plane = Plane::new(0, area.width, area.height);
        let mut zones = Vec::new();
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

            let label_len = tab.len().min(tab_width as usize - 2);
            let start_col = if tab_width > 2 { 1 } else { 0 };
            for (j, ch) in tab.chars().take(label_len).enumerate() {
                let idx = (start_col + j) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = style;
                }
            }

            let zone = HitZone::new(i, x, area.y, tab_width, area.height);
            zones.push(zone);
        }

        (plane, zones)
    }

    /// Handles a mouse event. Returns `true` if the event was consumed.
    pub fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, _row: u16) -> bool {
        let tab_count = self.tabs.len().max(1);
        let tab_width = (80u16 / tab_count as u16).max(1);
        let idx = col / tab_width;
        if idx >= tab_count as u16 {
            return false;
        }

        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                self.active = idx as usize;
                true
            }
            _ => false,
        }
    }

    /// Handles a key event for tab navigation. Returns `true` if consumed.
    pub fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Left => {
                if self.active > 0 {
                    self.active -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.active + 1 < self.tabs.len() {
                    self.active += 1;
                }
                true
            }
            _ => false,
        }
    }
}