//! Autocomplete widget with search input and dropdown suggestions.
//!
//! Wraps a text input and shows a dropdown of suggestions that match the
//! current query. Supports keyboard navigation (↑/↓), mouse selection,
//! scroll wheel paging, and tab-completion.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::hitzone::ScopedZoneRegistry;
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId, WidgetState};
use crate::framework::widgets::list_common::SelectCallback;
use crate::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

/// An autocomplete widget that wraps a text input and shows a dropdown
/// of matching suggestions.
pub struct Autocomplete {
    id: WidgetId,
    base: super::text_input_base::BaseInput,
    all_suggestions: Vec<String>,
    filtered: Vec<String>,
    selected: Option<usize>,
    offset: usize,
    max_visible: usize,
    dropdown_open: bool,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    zones: std::cell::RefCell<ScopedZoneRegistry<usize>>,
    on_select: Option<SelectCallback>,
}

impl Autocomplete {
    /// Creates a new autocomplete widget with the given ID and suggestions.
    pub fn new(id: WidgetId, suggestions: Vec<String>) -> Self {
        let filtered = suggestions.clone();
        Self {
            id,
            base: super::text_input_base::BaseInput::new(id, "Type to search..."),
            all_suggestions: suggestions,
            filtered,
            selected: None,
            offset: 0,
            max_visible: 8,
            dropdown_open: false,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 1)),
            dirty: true,
            zones: std::cell::RefCell::new(ScopedZoneRegistry::new()),
            on_select: None,
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.base.theme = theme.clone();
        self.theme = theme;
        self
    }

    /// Sets the maximum number of visible suggestions.
    pub fn with_max_visible(mut self, n: usize) -> Self {
        self.max_visible = n.max(1);
        self
    }

    /// Registers a callback when a suggestion is selected.
    pub fn on_select(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Returns the current input query.
    pub fn query(&self) -> &str {
        &self.base.text
    }

    /// Returns the currently highlighted suggestion, if any.
    pub fn selected(&self) -> Option<&str> {
        self.selected.map(|idx| self.filtered[idx].as_str())
    }

    /// Clears the input and resets the dropdown.
    pub fn clear(&mut self) {
        self.base.clear();
        self.update_filter();
    }

    /// Opens the dropdown with the current filter. Shows all suggestions if query is empty.
    pub fn open_dropdown(&mut self) {
        self.update_filter();
    }

    /// Returns whether the dropdown is currently open.
    pub fn is_dropdown_open(&self) -> bool {
        self.dropdown_open
    }

    fn update_filter(&mut self) {
        let query = self.base.text.to_lowercase();
        self.filtered = self
            .all_suggestions
            .iter()
            .filter(|s| s.to_lowercase().contains(&query))
            .cloned()
            .collect();

        if self.filtered.is_empty() {
            self.dropdown_open = false;
            self.selected = None;
        } else {
            self.dropdown_open = true;
            self.selected = Some(0);
        }
        self.offset = 0;
        self.dirty = true;
    }

    fn select_suggestion(&mut self, idx: usize) {
        if idx < self.filtered.len() {
            let text = self.filtered[idx].clone();
            self.base.text = text.clone();
            self.base.cursor_pos = text.len();
            self.dropdown_open = false;
            self.selected = None;
            self.dirty = true;
            if let Some(ref mut cb) = self.on_select {
                cb(&text);
            }
        }
    }
}

impl Widget for Autocomplete {
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
        self.base.set_area(Rect::new(area.x, area.y, area.width, 1));
        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        20
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.base.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.base.mark_dirty();
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
        self.base.clear_dirty();
    }

    fn render(&self, area: Rect) -> Plane {
        let dropdown_height = if self.dropdown_open {
            self.filtered.len().min(self.max_visible) as u16
        } else {
            0
        };
        let total_height = 1 + dropdown_height;

        let mut plane = Plane::new(0, area.width, total_height);
        plane.z_index = 20;
        plane.fill_bg(self.theme.bg);

        // Render input field into row 0
        let input_plane = self.base.render_input(Rect::new(0, 0, area.width, 1));
        for i in 0..input_plane.cells.len() {
            if i < plane.cells.len() {
                plane.cells[i] = input_plane.cells[i];
            }
        }

        if self.dropdown_open {
            let visible_count = self.filtered.len().min(self.max_visible);
            let start = self.offset;
            let end = (start + visible_count).min(self.filtered.len());

            self.zones.borrow_mut().clear();

            for (vis_idx, abs_idx) in (start..end).enumerate() {
                let row = 1 + vis_idx as u16;
                let is_selected = self.selected == Some(abs_idx);

                // Register hit zone for this suggestion row
                self.zones
                    .borrow_mut()
                    .register(abs_idx, 0, row, area.width, 1);

                let bg = if is_selected {
                    self.theme.primary
                } else {
                    self.theme.surface_elevated
                };
                let fg = if is_selected {
                    self.theme.fg_on_accent
                } else {
                    self.theme.fg
                };
                let style = if is_selected {
                    Styles::BOLD
                } else {
                    Styles::empty()
                };

                // Fill row background
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

                // Draw suggestion text
                let text = &self.filtered[abs_idx];
                for (j, ch) in text.chars().take(area.width as usize).enumerate() {
                    let idx = (row * area.width + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].style = style;
                    }
                }
            }
        }

        plane
    }

    fn cursor_position(&self) -> Option<(u16, u16)> {
        self.base.cursor_position()
    }

    fn on_focus(&mut self) {
        self.base.focused = true;
        self.base.dirty = true;
        self.dirty = true;
    }

    fn on_blur(&mut self) {
        self.base.focused = false;
        self.base.dirty = true;
        self.dirty = true;
        self.dropdown_open = false;
        self.selected = None;
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Up => {
                if self.dropdown_open {
                    if let Some(sel) = self.selected {
                        if sel > 0 {
                            self.selected = Some(sel - 1);
                            if sel - 1 < self.offset {
                                self.offset = sel - 1;
                            }
                        }
                    } else if !self.filtered.is_empty() {
                        let last = self.filtered.len() - 1;
                        self.selected = Some(last);
                        self.offset = last.saturating_sub(self.max_visible - 1);
                    }
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            KeyCode::Down => {
                if self.dropdown_open {
                    if let Some(sel) = self.selected {
                        if sel + 1 < self.filtered.len() {
                            self.selected = Some(sel + 1);
                            if sel + 1 >= self.offset + self.max_visible {
                                self.offset = sel + 1 - self.max_visible + 1;
                            }
                        }
                    } else if !self.filtered.is_empty() {
                        self.selected = Some(0);
                        self.offset = 0;
                    }
                    self.dirty = true;
                    true
                } else if !self.filtered.is_empty() {
                    self.dropdown_open = true;
                    self.selected = Some(0);
                    self.offset = 0;
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            KeyCode::Enter => {
                if self.dropdown_open {
                    if let Some(sel) = self.selected {
                        self.select_suggestion(sel);
                    }
                    true
                } else {
                    self.base.handle_key(key)
                }
            }
            KeyCode::Esc => {
                if self.dropdown_open {
                    self.dropdown_open = false;
                    self.selected = None;
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            KeyCode::Tab => {
                if self.dropdown_open && !self.filtered.is_empty() {
                    self.select_suggestion(0);
                    true
                } else {
                    false
                }
            }
            _ => {
                let consumed = self.base.handle_key(key);
                if consumed {
                    self.update_filter();
                }
                consumed
            }
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        match kind {
            MouseEventKind::Moved => {
                if row == 0 {
                    false
                } else if self.dropdown_open {
                    let zone_hit = self.zones.borrow().dispatch(col, row);
                    if let Some(abs_idx) = zone_hit {
                        if self.selected != Some(abs_idx) {
                            self.selected = Some(abs_idx);
                            self.dirty = true;
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if row == 0 {
                    self.base.handle_mouse(kind, col, row)
                } else if self.dropdown_open {
                    let zone_hit = self.zones.borrow().dispatch(col, row);
                    if let Some(abs_idx) = zone_hit {
                        self.select_suggestion(abs_idx);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            MouseEventKind::ScrollDown if self.dropdown_open => {
                let max_offset = self.filtered.len().saturating_sub(self.max_visible);
                if self.offset < max_offset {
                    self.offset += 1;
                    self.dirty = true;
                }
                true
            }
            MouseEventKind::ScrollUp if self.dropdown_open => {
                self.offset = self.offset.saturating_sub(1);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.base.theme = theme.clone();
    }
}

impl WidgetState for Autocomplete {
    fn state_id(&self) -> Option<&str> {
        Some("autocomplete")
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "query": self.base.text,
            "visible": self.dropdown_open,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(query) = json.get("query").and_then(|v| v.as_str()) {
            self.base.text = query.to_string();
            self.base.cursor_pos = query.len();
        }
        if let Some(visible) = json.get("visible").and_then(|v| v.as_bool()) {
            self.dropdown_open = visible;
        }
        self.dirty = true;
        Ok(())
    }
}
