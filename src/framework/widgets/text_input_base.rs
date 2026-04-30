//! Internal text input shared by SearchInput and PasswordInput.

#![allow(missing_docs)]

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct BaseInput {
    pub id: WidgetId,
    pub text: String,
    pub cursor_pos: usize,
    pub theme: Theme,
    pub on_submit: Option<Box<dyn FnMut(&str)>>,
    pub area: std::cell::Cell<Rect>,
    pub placeholder: String,
    pub mask_char: Option<char>,
    pub dirty: bool,
}

impl BaseInput {
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

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.dirty = true;
    }

    pub fn cursor_position(&self) -> Option<(u16, u16)> {
        let area = self.area.get();
        Some((area.x + self.cursor_pos as u16, area.y))
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

        let width = plane.cells.len() / plane.height as usize;

        let display = if self.text.is_empty() {
            self.placeholder.clone()
        } else if let Some(mask) = self.mask_char {
            self.text.chars().map(|_| mask).collect::<String>()
        } else {
            self.text.clone()
        };

        for (i, c) in display.chars().take(width.saturating_sub(2)).enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                let is_cursor = i == self.cursor_pos && !self.text.is_empty();
                plane.cells[idx] = Cell {
                    char: c,
                    fg: if is_cursor { self.theme.bg } else { self.theme.fg },
                    bg: if is_cursor { self.theme.fg } else { self.theme.input_bg },
                    style: Styles::empty(),
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
                if self.cursor_pos > 0 && !self.text.is_empty() {
                    self.text.pop();
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                    self.dirty = true;
                }
                true
            }
            KeyCode::Char(ch) => {
                self.text.push(ch);
                if self.cursor_pos < self.text.len() {
                    self.cursor_pos = self.text.len();
                }
                self.dirty = true;
                true
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_pos < self.text.len() {
                    self.cursor_pos += 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.text.len() {
                    self.text.remove(self.cursor_pos);
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                self.dirty = true;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.text.len();
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    pub fn handle_mouse(&mut self, _kind: crate::input::event::MouseEventKind, col: u16, _row: u16) -> bool {
        if col < self.text.len() as u16 {
            self.cursor_pos = col as usize;
            self.dirty = true;
            true
        } else {
            false
        }
    }
}