//! Internal text input shared by SearchInput and PasswordInput.

#![allow(missing_docs)]

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthStr;

/// Callback type for text submission.
pub type SubmitCallback = Box<dyn FnMut(&str)>;

pub struct BaseInput {
    pub id: WidgetId,
    pub text: String,
    pub cursor_pos: usize,
    pub theme: Theme,
    pub on_submit: Option<SubmitCallback>,
    pub area: std::cell::Cell<Rect>,
    pub placeholder: String,
    pub mask_char: Option<char>,
    pub dirty: bool,
    pub focused: bool,
    pub scroll_offset: usize,
}

impl BaseInput {
    /// Convert char-based cursor_pos to a byte index in self.text.
    /// cursor_pos counts characters, but String::insert/remove need byte offsets.
    fn cursor_byte_offset(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Number of characters in the text (not bytes).
    fn char_count(&self) -> usize {
        self.text.chars().count()
    }
    pub fn new(id: WidgetId, placeholder: &str) -> Self {
        Self {
            id,
            text: String::new(),
            cursor_pos: 0,
            theme: Theme::default(),
            on_submit: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 1)),
            placeholder: placeholder.to_string(),
            mask_char: None,
            dirty: true,
            focused: false,
            scroll_offset: 0,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn on_submit(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.scroll_offset = 0;
        self.dirty = true;
    }

    pub fn cursor_position(&self) -> Option<(u16, u16)> {
        let area = self.area.get();
        let before_text: String = self.text.chars().take(self.cursor_pos).collect();
        let visual_col = before_text.width() as u16;
        let scrolled_col = visual_col.saturating_sub(self.scroll_offset as u16);
        Some((area.x + scrolled_col, area.y))
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area.set(area);
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn render_input(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;
        plane.fill_bg(self.theme.input_bg);

        let width = plane.cells.len() / plane.height as usize;

        let display = if self.text.is_empty() {
            self.placeholder.clone()
        } else if let Some(mask) = self.mask_char {
            self.text.chars().map(|_| mask).collect::<String>()
        } else {
            self.text.clone()
        };

        let bg = if self.focused {
            self.theme.focus_bg
        } else {
            self.theme.input_bg
        };

        let focus_style = if self.focused {
            Styles::UNDERLINE
        } else {
            Styles::empty()
        };

        let visible_start = self.scroll_offset;
        let visible_chars: String = display
            .chars()
            .skip(visible_start)
            .take(width.saturating_sub(1))
            .collect();
        let cursor_visual = {
            let before: String = self.text.chars().take(self.cursor_pos).collect();
            before.width()
        };
        let cursor_display_pos = cursor_visual.saturating_sub(visible_start);

        for (i, c) in visible_chars.chars().enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                let is_cursor = i == cursor_display_pos && !self.text.is_empty();
                plane.cells[idx] = Cell {
                    char: c,
                    fg: if is_cursor {
                        self.theme.bg
                    } else {
                        self.theme.fg
                    },
                    bg: if is_cursor { self.theme.fg } else { bg },
                    style: if is_cursor {
                        Styles::empty()
                    } else {
                        focus_style
                    },
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }

    pub fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Enter => {
                if let Some(ref mut cb) = self.on_submit {
                    cb(&self.text);
                }
                true
            }
            KeyCode::Backspace => {
                if !self.text.is_empty() && self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    let byte_idx = self.cursor_byte_offset();
                    let ch = self.text[byte_idx..]
                        .chars()
                        .next()
                        .expect("cursor_byte_offset guarantees valid index for non-empty text at cursor > 0");
                    self.text
                        .replace_range(byte_idx..byte_idx + ch.len_utf8(), "");
                    self.clamp_scroll();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Char(ch)
                if key.modifiers.is_empty()
                    || key.modifiers == crate::input::event::KeyModifiers::SHIFT =>
            {
                let byte_idx = self.cursor_byte_offset();
                self.text.insert(byte_idx, ch);
                self.cursor_pos += 1;
                self.clamp_scroll();
                self.dirty = true;
                true
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.clamp_scroll();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_pos < self.char_count() {
                    self.cursor_pos += 1;
                    self.clamp_scroll();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.char_count() {
                    let byte_idx = self.cursor_byte_offset();
                    let ch = self.text[byte_idx..].chars().next().expect(
                        "cursor_byte_offset guarantees valid index for cursor < char_count",
                    );
                    self.text
                        .replace_range(byte_idx..byte_idx + ch.len_utf8(), "");
                    self.clamp_scroll();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                self.scroll_offset = 0;
                self.dirty = true;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.char_count();
                self.clamp_scroll();
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn clamp_scroll(&mut self) {
        let area = self.area.get();
        let visible_width = area.width.saturating_sub(1) as usize;
        let before_text: String = self.text.chars().take(self.cursor_pos).collect();
        let cursor_visual = before_text.width();
        if cursor_visual < self.scroll_offset {
            self.scroll_offset = cursor_visual;
        } else if cursor_visual >= self.scroll_offset + visible_width {
            self.scroll_offset = cursor_visual.saturating_sub(visible_width) + 1;
        }
    }

    pub fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        _row: u16,
    ) -> bool {
        // Middle-click paste from X11 primary selection (best-effort: X11 works, Wayland spotty)
        if let crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Middle) =
            kind
        {
            if let Some(text) = crate::utils::get_primary_selection_text() {
                self.insert_text(&text);
                self.dirty = true;
                return true;
            }
            return true; // Consume middle-click even if no selection available
        }

        let text_pos = (col as usize + self.scroll_offset).min(self.char_count());
        self.cursor_pos = text_pos;
        self.clamp_scroll();
        self.dirty = true;
        true
    }

    /// Insert text at the current cursor position.
    fn insert_text(&mut self, text: &str) {
        let byte_idx = self.cursor_byte_offset();
        self.text.insert_str(byte_idx, text);
        self.cursor_pos += text.chars().count();
        self.clamp_scroll();
    }
}

impl WidgetState for BaseInput {
    fn state_id(&self) -> Option<&str> {
        Some("text_input")
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "text": self.text,
            "cursor_pos": self.cursor_pos,
            "input_bg": format!("{:?}", self.theme.input_bg),
            "input_fg": format!("{:?}", self.theme.fg),
            "focused": self.focused,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
            self.text = text.to_string();
        }
        if let Some(pos) = json.get("cursor_pos").and_then(|v| v.as_u64()) {
            self.cursor_pos = pos as usize;
        }
        if let Some(focused) = json.get("focused").and_then(|v| v.as_bool()) {
            self.focused = focused;
        }
        self.dirty = true;
        Ok(())
    }
}
