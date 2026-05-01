//! Password input widget with character masking.

use crate::framework::widget::WidgetId;
use crate::framework::widget::Widget;
use ratatui::layout::Rect;

/// A password input widget that masks characters as they're typed.
pub struct PasswordInput {
    id: WidgetId,
    base: super::text_input_base::BaseInput,
}

impl PasswordInput {
    /// Creates a new password input with the given ID.
    pub fn new(id: WidgetId) -> Self {
        let mut base = super::text_input_base::BaseInput::new(id, "Password...");
        base.mask_char = Some('*');
        Self { id, base }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(self, theme: crate::framework::theme::Theme) -> Self {
        let id = self.id;
        let base = self.base.with_theme(theme);
        Self { id, base }
    }

    /// Sets the mask character (default is '*').
    pub fn with_mask_char(self, ch: char) -> Self {
        let id = self.id;
        let mut base = self.base;
        base.mask_char = Some(ch);
        Self { id, base }
    }

    /// Sets the placeholder text shown when empty.
    pub fn with_placeholder(self, text: &str) -> Self {
        let id = self.id;
        let mut base = self.base;
        base.placeholder = text.to_string();
        Self { id, base }
    }

    /// Registers a callback when the user submits the password (Enter key).
    pub fn on_submit(self, f: impl FnMut(&str) + 'static) -> Self {
        let id = self.id;
        let base = self.base.on_submit(f);
        Self { id, base }
    }

    /// Clears the password.
    pub fn clear(&mut self) {
        self.base.clear();
    }

    /// Returns the current password (unmasked).
    pub fn password(&self) -> &str {
        &self.base.text
    }
}

impl Widget for PasswordInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        self.base.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.base.set_area(area);
    }

    fn needs_render(&self) -> bool {
        self.base.dirty
    }

    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.base.dirty = false;
    }

    fn render(&self, area: Rect) -> crate::compositor::Plane {
        self.base.render_input(area)
    }

    fn cursor_position(&self) -> Option<(u16, u16)> {
        self.base.cursor_position()
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        self.base.handle_key(key)
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        self.base.handle_mouse(kind, col, row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_input_new() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id);
        assert_eq!(input.id, id);
        assert_eq!(input.base.placeholder, "Password...");
        assert_eq!(input.base.mask_char, Some('*'));
    }

    #[test]
    fn test_password_input_with_theme() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id).with_theme(crate::framework::theme::Theme::default());
        assert_eq!(input.base.theme.name, "default");
    }

    #[test]
    fn test_password_input_mask_char() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id).with_mask_char('X');
        assert_eq!(input.base.mask_char, Some('X'));
    }

    #[test]
    fn test_password_input_placeholder() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id).with_placeholder("Enter password");
        assert_eq!(input.base.placeholder, "Enter password");
    }

    #[test]
    fn test_password_input_on_submit() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id).on_submit(|text| {
            assert_eq!(text, "secret123");
        });
        assert!(input.base.on_submit.is_some());
    }

    #[test]
    fn test_password_input_clear() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        input.base.text = "secret".to_string();
        input.base.cursor_pos = 6;
        input.clear();
        assert!(input.base.text.is_empty());
        assert_eq!(input.base.cursor_pos, 0);
    }

    #[test]
    fn test_password_input_password() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        input.base.text = "mypassword".to_string();
        assert_eq!(input.password(), "mypassword");
    }

    #[test]
    fn test_password_input_widget_id() {
        let id = WidgetId::new(42);
        let input = PasswordInput::new(id);
        assert_eq!(input.id(), id);
    }

    #[test]
    fn test_password_input_set_id() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        let new_id = WidgetId::new(99);
        input.set_id(new_id);
        assert_eq!(input.id, new_id);
    }

    #[test]
    fn test_password_input_area() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id);
        input.base.area.set(Rect::new(10, 5, 30, 1));
        let area = input.area();
        assert_eq!(area.x, 10);
        assert_eq!(area.y, 5);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 1);
    }

    #[test]
    fn test_password_input_set_area() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        input.set_area(Rect::new(20, 10, 40, 2));
        let area = input.area();
        assert_eq!(area.x, 20);
        assert_eq!(area.y, 10);
        assert_eq!(area.width, 40);
        assert_eq!(area.height, 2);
    }

    #[test]
    fn test_password_input_needs_render() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id);
        assert!(input.needs_render());
    }

    #[test]
    fn test_password_input_mark_dirty() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        input.base.dirty = false;
        input.mark_dirty();
        assert!(input.base.dirty);
    }

    #[test]
    fn test_password_input_clear_dirty() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        input.base.dirty = true;
        input.clear_dirty();
        assert!(!input.base.dirty);
    }

    #[test]
    fn test_password_input_render() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id);
        let plane = input.render(Rect::new(0, 0, 20, 1));
        assert_eq!(plane.width, 20);
    }

    #[test]
    fn test_password_input_cursor_position() {
        let id = WidgetId::new(1);
        let input = PasswordInput::new(id);
        input.base.area.set(Rect::new(5, 3, 20, 1));
        let pos = input.cursor_position();
        assert!(pos.is_some());
        let (x, y) = pos.unwrap();
        assert_eq!(x, 5);
        assert_eq!(y, 3);
    }

    #[test]
    fn test_password_input_handle_key_char() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Char('s'),
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = input.handle_key(key);
        assert!(result);
        assert_eq!(input.base.text, "s");
    }

    #[test]
    fn test_password_input_handle_key_backspace() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        input.base.text = "pass".to_string();
        input.base.cursor_pos = 4;
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Backspace,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = input.handle_key(key);
        assert!(result);
        assert_eq!(input.base.text, "pas");
    }

    #[test]
    fn test_password_input_handle_key_enter_triggers_callback() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        input.base.text = "secret".to_string();
        let key = crate::input::event::KeyEvent {
            kind: crate::input::event::KeyEventKind::Press,
            code: crate::input::event::KeyCode::Enter,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = input.handle_key(key);
        assert!(result);
    }

    #[test]
    fn test_password_input_handle_mouse() {
        let id = WidgetId::new(1);
        let mut input = PasswordInput::new(id);
        input.base.text = "password".to_string();
        input.base.cursor_pos = 0;
        let result = input.handle_mouse(
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left),
            4,
            0,
        );
        assert!(result);
        assert_eq!(input.base.cursor_pos, 4);
    }
}