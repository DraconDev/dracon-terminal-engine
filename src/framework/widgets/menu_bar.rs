//! Menu bar widget for application menus.
//!
//! A horizontal bar with dropdown menus triggered by clicking.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A single menu item with a label and optional action.
pub struct MenuItem {
    /// The label for this menu item.
    pub label: String,
    /// The action callback for this menu item.
    pub action: Option<Box<dyn FnMut()>>,
    /// Whether this menu item is enabled.
    pub enabled: bool,
}

impl MenuItem {
    /// Creates a new menu item with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            action: None,
            enabled: true,
        }
    }

    /// Sets the action callback for this menu item.
    pub fn with_action(mut self, action: impl FnMut() + 'static) -> Self {
        self.action = Some(Box::new(action));
        self
    }

    /// Sets whether this menu item is enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// A menu entry in the menu bar with dropdown items.
pub struct MenuEntry {
    /// The label for this menu entry.
    pub label: String,
    /// The items in this menu entry.
    pub items: Vec<MenuItem>,
}

impl MenuEntry {
    /// Creates a new menu entry with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            items: Vec::new(),
        }
    }

    /// Adds an item to this menu entry.
    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }
}

/// A horizontal menu bar with dropdown submenus.
pub struct MenuBar {
    /// The widget ID for this menu bar.
    id: WidgetId,
    /// The menu entries to display.
    entries: Vec<MenuEntry>,
    /// The currently active menu entry index.
    active_entry: Option<usize>,
    /// The theme for this widget.
    theme: Theme,
    /// The last recorded area width for layout.
    last_area_width: std::cell::Cell<u16>,
    area: std::cell::Cell<Rect>,
}

impl MenuBar {
    /// Creates a new menu bar with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            entries: Vec::new(),
            active_entry: None,
            theme: Theme::default(),
            last_area_width: std::cell::Cell::new(80),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 1)),
        }
    }

    /// Sets the menu entries for this menu bar.
    pub fn with_entries(mut self, entries: Vec<MenuEntry>) -> Self {
        self.entries = entries;
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Closes any open menu dropdown.
    pub fn close(&mut self) {
        self.active_entry = None;
    }
}

impl crate::framework::widget::Widget for MenuBar {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn z_index(&self) -> u16 {
        60
    }

    fn render(&self, area: Rect) -> Plane {
        self.last_area_width.set(area.width);
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 60;

        let width = plane.cells.len() / plane.height as usize;
        let total_entries = self.entries.len();
        let entry_width = width / total_entries.max(1);

        for (i, entry) in self.entries.iter().enumerate() {
            let is_active = self.active_entry == Some(i);
            let prefix = if is_active { "[" } else { " " };
            let suffix = if is_active { "]" } else { " " };
            let display = format!("{}{}{}", prefix, entry.label, suffix);
            let cell_width = display.width().min(entry_width);

            for (j, c) in display.chars().take(cell_width).enumerate() {
                let idx = (0u16 * plane.width + (i * entry_width + j) as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: if is_active { self.theme.bg } else { self.theme.fg },
                        bg: if is_active { self.theme.accent } else { self.theme.bg },
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        plane
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        if row != 0 {
            return false;
        }
        match kind {
            crate::input::event::MouseEventKind::Down(_) => {
                let width = self.last_area_width.get() as usize;
                let total_entries = self.entries.len();
                let entry_width = (width / total_entries.max(1)).max(1);
                let entry_idx = (col as usize / entry_width).min(total_entries.saturating_sub(1));
                if entry_idx < total_entries {
                    if self.active_entry == Some(entry_idx) {
                        self.active_entry = None;
                    } else {
                        self.active_entry = Some(entry_idx);
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}