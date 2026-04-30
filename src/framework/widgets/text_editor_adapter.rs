//! TextEditor adapter for the framework widget system.
//!
//! Wraps the standalone `TextEditor` widget (which implements ratatui's `Widget`)
//! into the framework's `Widget` trait so it can be used with `App::add_widget()`.

use crate::compositor::Plane;
use crate::framework::widget::WidgetId;
use crate::input::event::{Event, KeyEvent, MouseEvent, MouseEventKind};
use crate::widgets::editor::TextEditor;
use ratatui::layout::Rect;

/// Adapter that wraps a [`TextEditor`] to implement the framework's
/// [`Widget`](crate::framework::widget::Widget) trait.
///
/// This allows the standalone text editor (which implements ratatui's `Widget`)
/// to be used inside the framework's `App` with focus management, event routing,
/// and compositor integration.
pub struct TextEditorAdapter {
    id: WidgetId,
    editor: TextEditor,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl TextEditorAdapter {
    /// Creates a new adapter wrapping the given [`TextEditor`].
    pub fn new(id: WidgetId, editor: TextEditor) -> Self {
        Self {
            id,
            editor,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
        }
    }

    /// Returns a reference to the underlying [`TextEditor`].
    pub fn editor(&self) -> &TextEditor {
        &self.editor
    }

    /// Returns a mutable reference to the underlying [`TextEditor`].
    pub fn editor_mut(&mut self) -> &mut TextEditor {
        &mut self.editor
    }

    /// Sets the screen area allocated to this widget.
    pub fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }
}

impl crate::framework::widget::Widget for TextEditorAdapter {
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

    fn z_index(&self) -> u16 {
        10
    }

    fn focusable(&self) -> bool {
        true
    }

    fn cursor_position(&self) -> Option<(u16, u16)> {
        let area = self.area.get();
        let visual_x = self.editor.get_visual_x(self.editor.cursor_row, self.editor.cursor_col);
        let screen_row = self.editor.cursor_row.saturating_sub(self.editor.scroll_row) as u16;
        let screen_col = visual_x.saturating_sub(self.editor.scroll_col) as u16;
        // Clamp to visible area to avoid reporting coordinates outside the widget
        let clamped_col = screen_col.min(area.width.saturating_sub(1));
        let clamped_row = screen_row.min(area.height.saturating_sub(1));
        Some((area.x + clamped_col, area.y + clamped_row))
    }

    fn render(&self, area: Rect) -> Plane {
        use crate::compositor::engine::map_color;
        use crate::compositor::Cell;
        use crate::compositor::Styles;
        use ratatui::buffer::Buffer;
        use ratatui::prelude::Widget;

        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let mut buf = Buffer::empty(area);
        (&self.editor).render(area, &mut buf);

        for (i, cell) in buf.content().iter().enumerate() {
            let x = (i % area.width as usize) as u16;
            let y = (i / area.width as usize) as u16;
            if x < area.width && y < area.height {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: cell.symbol().chars().next().unwrap_or(' '),
                        fg: map_color(cell.fg),
                        bg: map_color(cell.bg),
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        plane
    }

    fn on_focus(&mut self) {}

    fn on_blur(&mut self) {}

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let area = self.area.get();
        let result = self.editor.handle_event(&Event::Key(key), area);
        if result {
            self.dirty = true;
        }
        result
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        let mouse = MouseEvent {
            kind,
            column: area.x + col,
            row: area.y + row,
            modifiers: crate::input::event::KeyModifiers::empty(),
        };
        let result = self.editor.handle_mouse_event(mouse, area);
        if result {
            self.dirty = true;
        }
        result
    }
}
