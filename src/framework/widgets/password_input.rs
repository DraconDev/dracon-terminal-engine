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