//! Tags/chips input widget.
//!
//! A widget for entering and managing tags/chips with autocomplete support.

use std::cell::RefCell;

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;

/// Callback type for tag add events.
pub type TagAddCallback = Box<dyn FnMut(String)>;

/// Callback type for tag remove events.
pub type TagRemoveCallback = Box<dyn FnMut(usize)>;

/// Callback type for input change events.
pub type InputChangeCallback = Box<dyn FnMut(String)>;

/// Callback type for suggestion selection.
pub type SuggestionCallback = Box<dyn FnMut(String)>;

/// A tags/chips input widget with autocomplete.
pub struct TagsInput {
    id: WidgetId,
    tags: Vec<String>,
    input_text: String,
    placeholder: String,
    suggestions: Vec<String>,
    filtered_suggestions: Vec<String>,
    selected_suggestion: Option<usize>,
    theme: Theme,
    width: u16,
    max_tags: Option<usize>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    focused: bool,
    hovered_tag: Option<usize>,
    on_tag_add: Option<TagAddCallback>,
    on_tag_remove: Option<TagRemoveCallback>,
    on_input_change: Option<InputChangeCallback>,
    on_suggestion_select: Option<SuggestionCallback>,
    allow_duplicates: bool,
}

impl TagsInput {
    /// Creates a new TagsInput widget with optional initial tags.
    pub fn new(tags: Vec<String>) -> Self {
        Self {
            id: WidgetId::default_id(),
            tags,
            input_text: String::new(),
            placeholder: "Add tag...".to_string(),
            suggestions: Vec::new(),
            filtered_suggestions: Vec::new(),
            selected_suggestion: None,
            theme: Theme::default(),
            width: 40,
            max_tags: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 3)),
            dirty: true,
            focused: false,
            hovered_tag: None,
            on_tag_add: None,
            on_tag_remove: None,
            on_input_change: None,
            on_suggestion_select: None,
            allow_duplicates: false,
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the placeholder text.
    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self.dirty = true;
        self
    }

    /// Sets the width of the widget.
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self.dirty = true;
        self
    }

    /// Sets the maximum number of tags.
    pub fn with_max_tags(mut self, max: usize) -> Self {
        self.max_tags = Some(max);
        self
    }

    /// Sets whether to allow duplicate tags.
    pub fn allow_duplicates(mut self, allow: bool) -> Self {
        self.allow_duplicates = allow;
        self
    }

    /// Sets the available suggestions for autocomplete.
    pub fn with_suggestions(mut self, suggestions: Vec<&str>) -> Self {
        self.suggestions = suggestions.into_iter().map(|s| s.to_string()).collect();
        self.dirty = true;
        self
    }

    /// Registers a callback invoked when a tag is added.
    pub fn on_tag_add(mut self, f: impl FnMut(String) + 'static) -> Self {
        self.on_tag_add = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when a tag is removed.
    pub fn on_tag_remove(mut self, f: impl FnMut(usize) + 'static) -> Self {
        self.on_tag_remove = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when the input text changes.
    pub fn on_input_change(mut self, f: impl FnMut(String) + 'static) -> Self {
        self.on_input_change = Some(Box::new(f));
        self
    }

    /// Registers a callback invoked when a suggestion is selected.
    pub fn on_suggestion_select(mut self, f: impl FnMut(String) + 'static) -> Self {
        self.on_suggestion_select = Some(Box::new(f));
        self
    }

    /// Sets the initial tags.
    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self.dirty = true;
        self
    }

    /// Adds a tag.
    pub fn add_tag(&mut self, tag: String) {
        // Check for duplicates
        if !self.allow_duplicates && self.tags.iter().any(|t| t.eq_ignore_ascii_case(&tag)) {
            return;
        }

        // Check max tags
        if let Some(max) = self.max_tags {
            if self.tags.len() >= max {
                return;
            }
        }

        let tag = tag.trim().to_string();
        if tag.is_empty() {
            return;
        }

        self.tags.push(tag);
        self.input_text.clear();
        self.filter_suggestions();
        self.dirty = true;
    }

    /// Removes a tag at the given index.
    pub fn remove_tag(&mut self, index: usize) {
        if index < self.tags.len() {
            self.tags.remove(index);
            self.dirty = true;
            if let Some(ref mut cb) = self.on_tag_remove {
                cb(index);
            }
        }
    }

    /// Removes the last tag.
    pub fn remove_last_tag(&mut self) {
        if !self.tags.is_empty() {
            let idx = self.tags.len() - 1;
            self.tags.remove(idx);
            self.dirty = true;
            if let Some(ref mut cb) = self.on_tag_remove {
                cb(idx);
            }
        }
    }

    /// Returns the current tags.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Returns the current input text.
    pub fn input(&self) -> &str {
        &self.input_text
    }

    /// Clears all tags.
    pub fn clear(&mut self) {
        self.tags.clear();
        self.dirty = true;
    }

    fn filter_suggestions(&mut self) {
        if self.input_text.is_empty() {
            self.filtered_suggestions = self.suggestions.clone();
        } else {
            let query = self.input_text.to_lowercase();
            self.filtered_suggestions = self
                .suggestions
                .iter()
                .filter(|s| {
                    !self.tags.iter().any(|t| t.eq_ignore_ascii_case(s))
                        && s.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }
        self.selected_suggestion = self.filtered_suggestions.is_empty().then(|| None);
        if self.filtered_suggestions.len() == 1 {
            self.selected_suggestion = Some(0);
        }
    }

    fn select_suggestion(&mut self, index: usize) {
        if index < self.filtered_suggestions.len() {
            let suggestion = self.filtered_suggestions[index].clone();
            self.add_tag(suggestion);
        }
    }
}

impl crate::framework::widget::Widget for TagsInput {
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

    fn focusable(&self) -> bool {
        true
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
        plane.fill_bg(self.theme.bg);

        // Calculate layout
        let tag_height = 1u16;
        let input_height = 1u16;
        let suggestions_height = self.filtered_suggestions.len().min(5) as u16;
        let total_height = 1 + tag_height + input_height + if !self.filtered_suggestions.is_empty() { suggestions_height + 1 } else { 0 };

        let mut y = 0u16;

        // Render tags
        let mut x = 0u16;
        for (i, tag) in self.tags.iter().enumerate() {
            let tag_str = format!("[{}] ", tag);
            let tag_width = tag_str.width() as u16;

            if x + tag_width >= area.width {
                // Move to next line
                y += 1;
                x = 0;
                if y >= area.height {
                    break;
                }
            }

            let is_hovered = self.hovered_tag == Some(i);

            // Draw tag
            for (j, ch) in tag_str.chars().enumerate() {
                let cx = x + j as u16;
                if cx >= area.width {
                    break;
                }
                let idx = (y * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = if ch == '[' || ch == ']' {
                        self.theme.fg_muted
                    } else {
                        self.theme.fg
                    };
                    plane.cells[idx].bg = if is_hovered {
                        self.theme.hover_bg
                    } else {
                        self.theme.surface_elevated
                    };
                }
            }
            x += tag_width;
        }

        // Move to next line for input
        if x > 0 {
            y += 1;
            x = 0;
        }

        // Render input field
        if y < area.height {
            let input_bg = if self.focused {
                self.theme.primary_active
            } else {
                self.theme.surface_elevated
            };

            // Input border
            let input_x = x;
            for col in 0..(area.width.saturating_sub(x)) {
                let idx = (y * area.width + x + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = input_bg;
                    plane.cells[idx].char = ' ';
                }
            }

            // Left border
            let idx = (y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '[';
                plane.cells[idx].fg = self.theme.outline;
            }

            // Input text or placeholder
            let display_text = if self.input_text.is_empty() {
                self.placeholder.clone()
            } else {
                self.input_text.clone()
            };

            let text_x = x + 1;
            let max_input_width = (area.width.saturating_sub(text_x + 1)).max(1);

            for (i, ch) in display_text.chars().enumerate() {
                if i as u16 >= max_input_width {
                    break;
                }
                let idx = (y * area.width + text_x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = if self.input_text.is_empty() {
                        self.theme.fg_muted
                    } else {
                        self.theme.fg
                    };
                    plane.cells[idx].style = if self.focused && !self.input_text.is_empty() {
                        Styles::empty()
                    } else {
                        Styles::DIM
                    };
                }
            }

            // Cursor
            if self.focused && x + 1 + self.input_text.len() as u16 < area.width {
                let cursor_x = text_x + self.input_text.len() as u16;
                let cursor_idx = (y * area.width + cursor_x) as usize;
                if cursor_idx < plane.cells.len() {
                    plane.cells[cursor_idx].char = '_';
                    plane.cells[cursor_idx].fg = self.theme.primary;
                }
            }

            // Right border
            let right_x = text_x + max_input_width;
            let idx = (y * area.width + right_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ']';
                plane.cells[idx].fg = self.theme.outline;
            }
        }

        // Render suggestions dropdown
        if !self.filtered_suggestions.is_empty() && y + 2 < area.height {
            y += input_height + 1;

            // Suggestions header
            for col in 0..area.width {
                let idx = (y * area.width + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.surface_elevated;
                    plane.cells[idx].char = ' ';
                }
            }

            // Suggestions
            for (i, suggestion) in self.filtered_suggestions.iter().enumerate() {
                if y + 1 + i as u16 >= area.height {
                    break;
                }
                if i >= 5 {
                    break;
                }

                let is_selected = self.selected_suggestion == Some(i);

                // Draw suggestion
                let sug_x = 0u16;
                let prefix = if is_selected { "> " } else { "  " };
                for (j, ch) in prefix.chars().enumerate() {
                    let idx = ((y + 1 + i as u16) * area.width + sug_x + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if is_selected {
                            self.theme.primary
                        } else {
                            self.theme.fg_muted
                        };
                    }
                }

                let text_start = sug_x + 2;
                for (j, ch) in suggestion.chars().enumerate() {
                    if text_start + j as u16 >= area.width {
                        break;
                    }
                    let idx = ((y + 1 + i as u16) * area.width + text_start + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = self.theme.fg;
                        if is_selected {
                            plane.cells[idx].bg = self.theme.hover_bg;
                        }
                    }
                }

                // Highlight matching text
                if !self.input_text.is_empty() && i == self.selected_suggestion.unwrap_or(0) {
                    let query = self.input_text.to_lowercase();
                    if let Some(start) = suggestion.to_lowercase().find(&query) {
                        let start_x = text_start + start as u16;
                        for j in 0..self.input_text.len() as u16 {
                            let idx = ((y + 1 + i as u16) * area.width + start_x + j) as usize;
                            if idx < plane.cells.len() {
                                plane.cells[idx].style = Styles::BOLD;
                            }
                        }
                    }
                }
            }
        }

        // Tags count indicator
        if let Some(max) = self.max_tags {
            let count_text = format!("{}/{} tags", self.tags.len(), max);
            let count_x = area.width.saturating_sub(count_text.len() as u16);
            let count_y = area.height.saturating_sub(1);
            for (i, ch) in count_text.chars().enumerate() {
                let idx = (count_y * area.width + count_x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = if self.tags.len() >= max {
                        self.theme.warning
                    } else {
                        self.theme.fg_muted
                    };
                    plane.cells[idx].style = Styles::DIM;
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
                if let Some(idx) = self.selected_suggestion {
                    self.select_suggestion(idx);
                } else if !self.input_text.is_empty() {
                    self.add_tag(self.input_text.clone());
                }
                true
            }
            KeyCode::Backspace => {
                if self.input_text.is_empty() && !self.tags.is_empty() {
                    self.remove_last_tag();
                } else if !self.input_text.is_empty() {
                    self.input_text.pop();
                    self.filter_suggestions();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Esc => {
                if !self.filtered_suggestions.is_empty() {
                    self.filtered_suggestions.clear();
                    self.selected_suggestion = None;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Up => {
                if !self.filtered_suggestions.is_empty() {
                    if let Some(idx) = self.selected_suggestion {
                        self.selected_suggestion = Some(idx.saturating_sub(1));
                    } else {
                        self.selected_suggestion = Some(self.filtered_suggestions.len() - 1);
                    }
                    self.dirty = true;
                }
                true
            }
            KeyCode::Down => {
                if !self.filtered_suggestions.is_empty() {
                    let max = self.filtered_suggestions.len() - 1;
                    if let Some(idx) = self.selected_suggestion {
                        self.selected_suggestion = Some((idx + 1).min(max));
                    } else {
                        self.selected_suggestion = Some(0);
                    }
                    self.dirty = true;
                }
                true
            }
            KeyCode::Tab => {
                if !self.filtered_suggestions.is_empty() {
                    if let Some(idx) = self.selected_suggestion {
                        self.select_suggestion(idx);
                    } else {
                        self.select_suggestion(0);
                    }
                } else if !self.input_text.is_empty() {
                    self.add_tag(self.input_text.clone());
                }
                true
            }
            KeyCode::Char(c) if !key.modifiers.contains(crate::input::event::KeyModifiers::CONTROL) => {
                self.input_text.push(c);
                self.filter_suggestions();
                if let Some(ref mut cb) = self.on_input_change {
                    cb(self.input_text.clone());
                }
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        let area = self.area.get();

        match kind {
            crate::input::event::MouseEventKind::Down(btn) if btn == crate::input::event::MouseButton::Left => {
                // Check if clicking on a tag's remove button
                // (assuming remove is triggered by clicking on the tag)
                let mut x = 0u16;
                for (i, tag) in self.tags.iter().enumerate() {
                    let tag_str = format!("[{}] ", tag);
                    let tag_width = tag_str.width() as u16;

                    if col >= x && col < x + tag_width {
                        self.hovered_tag = Some(i);
                        // Double-click to remove
                        self.dirty = true;
                        return true;
                    }
                    x += tag_width;

                    if x >= area.width {
                        break;
                    }
                }

                // Check if clicking on input area
                if col >= x && col < area.width {
                    self.focused = true;
                    self.dirty = true;
                }
                true
            }
            crate::input::event::MouseEventKind::Moved => {
                // Check hover on tags
                let mut x = 0u16;
                let mut found = false;
                for (i, tag) in self.tags.iter().enumerate() {
                    let tag_str = format!("[{}] ", tag);
                    let tag_width = tag_str.width() as u16;

                    if col >= x && col < x + tag_width {
                        if self.hovered_tag != Some(i) {
                            self.hovered_tag = Some(i);
                            self.dirty = true;
                        }
                        found = true;
                        break;
                    }
                    x += tag_width;

                    if x >= area.width {
                        break;
                    }
                }

                if !found && self.hovered_tag.is_some() {
                    self.hovered_tag = None;
                    self.dirty = true;
                }
                true
            }
            _ => false,
        }
    }

    fn on_focus(&mut self) {
        self.focused = true;
        self.dirty = true;
    }

    fn on_blur(&mut self) {
        self.focused = false;
        self.filtered_suggestions.clear();
        self.selected_suggestion = None;
        self.dirty = true;
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}