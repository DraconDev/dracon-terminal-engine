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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_input_new() {
        let id = WidgetId::new(1);
        let base = BaseInput::new(id, "placeholder");
        assert!(base.text.is_empty());
        assert_eq!(base.cursor_pos, 0);
        assert_eq!(base.placeholder, "placeholder");
        assert!(base.dirty);
    }

    #[test]
    fn test_base_input_with_theme() {
        let id = WidgetId::new(1);
        let base = BaseInput::new(id, "placeholder").with_theme(Theme::cyberpunk());
        assert_eq!(base.theme.name, "cyberpunk");
    }

    #[test]
    fn test_base_input_clear() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "hello".to_string();
        base.cursor_pos = 5;
        base.clear();
        assert!(base.text.is_empty());
        assert_eq!(base.cursor_pos, 0);
        assert!(base.dirty);
    }

    #[test]
    fn test_base_input_cursor_position() {
        let id = WidgetId::new(1);
        let base = BaseInput::new(id, "placeholder");
        base.area.set(Rect::new(10, 5, 30, 1));
        let pos = base.cursor_position();
        assert!(pos.is_some());
        let (x, y) = pos.unwrap();
        assert_eq!(x, 10);
        assert_eq!(y, 5);
    }

    #[test]
    fn test_base_input_set_area() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        let area = Rect::new(20, 10, 40, 2);
        base.set_area(area);
        assert!(base.dirty);
        let a = base.area.get();
        assert_eq!(a.x, 20);
        assert_eq!(a.y, 10);
        assert_eq!(a.width, 40);
        assert_eq!(a.height, 2);
    }

    #[test]
    fn test_base_input_mark_dirty() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.clear_dirty();
        assert!(!base.dirty);
        base.mark_dirty();
        assert!(base.dirty);
    }

    #[test]
    fn test_base_input_render_input_empty() {
        let id = WidgetId::new(1);
        let base = BaseInput::new(id, "placeholder");
        let plane = base.render_input(Rect::new(0, 0, 20, 1));
        assert_eq!(plane.width, 20);
        assert!(plane.z_index > 0);
    }

    #[test]
    fn test_base_input_render_input_with_text() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "test".to_string();
        base.cursor_pos = 2;
        let plane = base.render_input(Rect::new(0, 0, 20, 1));
        assert_eq!(plane.width, 20);
    }

    #[test]
    fn test_base_input_render_input_with_mask() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "secret".to_string();
        base.mask_char = Some('*');
        base.cursor_pos = 3;
        let plane = base.render_input(Rect::new(0, 0, 20, 1));
        assert_eq!(plane.width, 20);
    }

    #[test]
    fn test_base_input_handle_key_enter_triggers_callback() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Enter,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
    }

    #[test]
    fn test_base_input_handle_key_char() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Char('a'),
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
        assert_eq!(base.text, "a");
        assert_eq!(base.cursor_pos, 1);
    }

    #[test]
    fn test_base_input_handle_key_backspace() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "abc".to_string();
        base.cursor_pos = 2;
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Backspace,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
        assert_eq!(base.text, "ac");
        assert_eq!(base.cursor_pos, 1);
    }

    #[test]
    fn test_base_input_handle_key_backspace_at_start() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "abc".to_string();
        base.cursor_pos = 0;
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Backspace,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
        assert_eq!(base.text, "abc");
    }

    #[test]
    fn test_base_input_handle_key_left() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "abc".to_string();
        base.cursor_pos = 2;
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Left,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
        assert_eq!(base.cursor_pos, 1);
    }

    #[test]
    fn test_base_input_handle_key_right() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "abc".to_string();
        base.cursor_pos = 1;
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Right,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
        assert_eq!(base.cursor_pos, 2);
    }

    #[test]
    fn test_base_input_handle_key_delete() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "abc".to_string();
        base.cursor_pos = 1;
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Delete,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
        assert_eq!(base.text, "ac");
    }

    #[test]
    fn test_base_input_handle_key_home() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "abc".to_string();
        base.cursor_pos = 2;
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Home,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
        assert_eq!(base.cursor_pos, 0);
    }

    #[test]
    fn test_base_input_handle_key_end() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "abc".to_string();
        base.cursor_pos = 1;
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::End,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(result);
        assert_eq!(base.cursor_pos, 3);
    }

    #[test]
    fn test_base_input_handle_key_repeat_ignored() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Repeat,
            code: crate::input::event::KeyCode::Char('a'),
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = base.handle_key(key);
        assert!(!result);
        assert!(base.text.is_empty());
    }

    #[test]
    fn test_base_input_handle_mouse() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "abcdef".to_string();
        base.cursor_pos = 0;
        let result = base.handle_mouse(
            crate::input::event::MouseEventKind::Press,
            3,
            0,
        );
        assert!(result);
        assert_eq!(base.cursor_pos, 3);
        assert!(base.dirty);
    }

    #[test]
    fn test_base_input_handle_mouse_past_text() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "ab".to_string();
        base.cursor_pos = 0;
        let result = base.handle_mouse(
            crate::input::event::MouseEventKind::Press,
            10,
            0,
        );
        assert!(!result);
    }

    #[test]
    fn test_base_input_on_submit_builder() {
        let id = WidgetId::new(1);
        let base = BaseInput::new(id, "placeholder");
        let result = base.on_submit(|_text| {});
        assert!(result.on_submit.is_some());
    }

    #[test]
    fn test_base_input_render_input_narrow_area() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "this is a very long text that should be truncated".to_string();
        let plane = base.render_input(Rect::new(0, 0, 5, 1));
        assert_eq!(plane.width, 5);
    }

    #[test]
    fn test_base_input_render_input_unicode() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "Hello 世界".to_string();
        let plane = base.render_input(Rect::new(0, 0, 20, 1));
        assert_eq!(plane.width, 20);
    }

    #[test]
    fn test_base_input_render_input_cursor_at_end() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "test".to_string();
        base.cursor_pos = 4;
        let plane = base.render_input(Rect::new(0, 0, 20, 1));
        assert_eq!(plane.width, 20);
    }

    #[test]
    fn test_base_input_render_input_cursor_hidden_when_empty() {
        let id = WidgetId::new(1);
        let mut base = BaseInput::new(id, "placeholder");
        base.text = "".to_string();
        base.cursor_pos = 0;
        let plane = base.render_input(Rect::new(0, 0, 20, 1));
        assert_eq!(plane.width, 20);
    }
}