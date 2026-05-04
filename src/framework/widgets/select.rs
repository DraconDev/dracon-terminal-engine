//! Select widget for choosing from a dropdown list.
//!
//! A compact widget showing the currently selected item with a dropdown.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A dropdown select widget for choosing from a list of options.
type ChangeCallback = Box<dyn FnMut(&str)>;

pub struct Select {
    id: WidgetId,
    options: Vec<String>,
    selected: usize,
    expanded: bool,
    theme: Theme,
    on_change: Option<ChangeCallback>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Select {
    /// Creates a new select dropdown with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            options: Vec::new(),
            selected: 0,
            expanded: false,
            theme: Theme::default(),
            on_change: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 20, 1)),
            dirty: true,
        }
    }

    /// Sets the list of options in the dropdown.
    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Registers a callback when the selection changes.
    pub fn on_change(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Returns the index of the currently selected option.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Returns the label of the currently selected option, if any.
    pub fn selected_label(&self) -> Option<&str> {
        self.options.get(self.selected).map(|s| s.as_str())
    }
}

impl crate::framework::widget::Widget for Select {
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

        let display = if let Some(label) = self.selected_label() {
            format!("{} {}", label, if self.expanded { "^" } else { "v" })
        } else {
            "(select)".to_string()
        };

        let _cell_width = display.width().min(width);
        let fg = if self.expanded {
            self.theme.primary
        } else {
            self.theme.fg
        };

        for (i, c) in display.chars().take(width).enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        if self.expanded {
            for (i, option) in self.options.iter().enumerate() {
                if i + 1 >= plane.height as usize {
                    break;
                }
                let is_selected = i == self.selected;
                let prefix = if is_selected { ">" } else { " " };
                let line = format!("{}{}", prefix, option);

                for (j, c) in line.chars().take(width).enumerate() {
                    let idx = ((i + 1) as u16 * plane.width + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = Cell {
                            char: c,
                            fg: if is_selected {
                                self.theme.primary
                            } else {
                                self.theme.fg
                            },
                            bg: if is_selected {
                                self.theme.selection_bg
                            } else {
                                self.theme.bg
                            },
                            style: if is_selected {
                                Styles::BOLD
                            } else {
                                Styles::empty()
                            },
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Enter => {
                self.expanded = !self.expanded;
                self.dirty = true;
                true
            }
            KeyCode::Down if self.expanded => {
                if self.selected < self.options.len().saturating_sub(1) {
                    self.selected += 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Up if self.expanded => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        _col: u16,
        row: u16,
    ) -> bool {
        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                if row == 0 {
                    self.expanded = !self.expanded;
                    self.dirty = true;
                    true
                } else if self.expanded {
                    let item_idx = (row - 1) as usize;
                    if item_idx < self.options.len() {
                        self.selected = item_idx;
                        self.expanded = false;
                        if let Some(ref mut cb) = self.on_change {
                            cb(&self.options[self.selected]);
                        }
                        self.dirty = true;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}
