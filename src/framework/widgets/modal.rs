//! Modal dialog widget.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Plane, Styles};
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// Result returned when the user clicks a button in a modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalResult {
    /// User confirmed (e.g. OK button).
    Confirm,
    /// User cancelled.
    Cancel,
    /// A custom button with the given identifier.
    Custom(u8),
}

/// A centered modal dialog with a title, optional buttons, and a border.
pub struct Modal<'a> {
    id: WidgetId,
    title: &'a str,
    width: u16,
    height: u16,
    theme: Theme,
    buttons: Vec<(&'a str, ModalResult)>,
    focused_btn: usize,
    result: Option<ModalResult>,
    on_confirm: Option<Box<dyn FnMut()>>,
    on_cancel: Option<Box<dyn FnMut()>>,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl<'a> Modal<'a> {
    /// Creates a new `Modal` with the given title and default OK/Cancel buttons.
    pub fn new(title: &'a str) -> Self {
        Self {
            id: WidgetId::default_id(),
            title,
            width: 40,
            height: 5,
            theme: Theme::default(),
            buttons: vec![("OK", ModalResult::Confirm), ("Cancel", ModalResult::Cancel)],
            focused_btn: 0,
            result: None,
            on_confirm: None,
            on_cancel: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 5)),
            dirty: true,
        }
    }

    /// Creates a new `Modal` with the given widget ID and title.
    pub fn new_with_id(id: WidgetId, title: &'a str) -> Self {
        Self {
            id,
            title,
            width: 40,
            height: 5,
            theme: Theme::default(),
            buttons: vec![("OK", ModalResult::Confirm), ("Cancel", ModalResult::Cancel)],
            focused_btn: 0,
            result: None,
            on_confirm: None,
            on_cancel: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 5)),
            dirty: true,
        }
    }

    /// Sets the width and height of the modal.
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the button label/result pairs.
    pub fn with_buttons(mut self, buttons: Vec<(&'a str, ModalResult)>) -> Self {
        self.buttons = buttons;
        self
    }

    /// Sets the callback for when OK is confirmed.
    pub fn on_confirm(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_confirm = Some(Box::new(f));
        self
    }

    /// Sets the callback for when Cancel is pressed.
    pub fn on_cancel(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_cancel = Some(Box::new(f));
        self
    }

    /// Returns the result of the modal after it's been dismissed.
    pub fn get_result(&self) -> Option<ModalResult> {
        self.result
    }

    /// Clears the result, allowing the modal to be reused.
    pub fn clear_result(&mut self) {
        self.result = None;
    }
}

impl<'a> crate::framework::widget::Widget for Modal<'a> {
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
        100
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
        let x = (area.width.saturating_sub(self.width)) / 2;
        let y = (area.height.saturating_sub(self.height)) / 2;

        let mut plane = Plane::new(0, self.width, self.height);
        plane.x = x;
        plane.y = y;
        plane.z_index = 100;

        for cell in &mut plane.cells {
            cell.bg = self.theme.bg;
            cell.fg = self.theme.fg;
        }

        let border_char: char = '─';
        for col in 0..self.width {
            let idx = col as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = border_char; }
            let idx = ((self.height - 1) * self.width + col) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = '─'; }
        }
        for row in 1..self.height.saturating_sub(1) {
            let idx = (row * self.width) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = '│'; }
            let idx = (row * self.width + self.width - 1) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = '│'; }
        }

        let title_len = self.title.width().min((self.width as usize).saturating_sub(4));
        let title_start = (self.width as usize - title_len) / 2;
        for (i, ch) in self.title.chars().take(title_len).enumerate() {
            let idx = (1 + title_start + i) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].fg = self.theme.accent;
            }
        }

        let btn_width: u16 = 8;
        let total_btn_width = btn_width * self.buttons.len() as u16 + (self.buttons.len() as u16 - 1);
        let btn_start = (self.width.saturating_sub(total_btn_width)) / 2;
        let btn_y = self.height - 2;

        for (i, (label, _result)) in self.buttons.iter().enumerate() {
            let bx = btn_start + (i as u16) * (btn_width + 1);

            let is_focused = i == self.focused_btn;
            let bg = if is_focused { self.theme.active_bg } else { self.theme.bg };
            let fg = self.theme.fg;
            let style = if is_focused { Styles::BOLD | Styles::REVERSE } else { Styles::empty() };
            for col in 0..btn_width {
                let col_idx = btn_y as usize * self.width as usize + bx as usize + col as usize;
                if col_idx < plane.cells.len() {
                    plane.cells[col_idx].bg = bg;
                    plane.cells[col_idx].fg = fg;
                    plane.cells[col_idx].style = style;
                    plane.cells[col_idx].char = ' ';
                }
            }

            let label_len = label.width().min((btn_width as usize).saturating_sub(2));
            let label_start = (btn_width as usize - label_len) / 2;
            for (j, ch) in label.chars().take(label_len).enumerate() {
                let label_idx = (btn_y as usize) * (self.width as usize) + (bx as usize) + (label_start as usize) + j;
                if label_idx < plane.cells.len() {
                    plane.cells[label_idx].char = ch;
                    plane.cells[label_idx].style = if is_focused { Styles::BOLD } else { Styles::empty() };
                }
            }

            let _zone = HitZone::new(*_result, bx, btn_y, btn_width, 1);
        }

        plane
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        let screen = self.area.get();
        let x = (screen.width.saturating_sub(self.width)) / 2;
        let y = (screen.height.saturating_sub(self.height)) / 2;

        if col < x || col >= x + self.width || row < y || row >= y + self.height {
            return false;
        }

        let local_col = col - x;
        let local_row = row - y;

        let btn_width: u16 = 8;
        let total_btn_width = btn_width * self.buttons.len() as u16 + (self.buttons.len() as u16 - 1);
        let btn_start = (self.width.saturating_sub(total_btn_width)) / 2;
        let btn_y = self.height - 2;

        for (i, (_, result)) in self.buttons.iter().enumerate() {
            let bx = btn_start + (i as u16) * (btn_width + 1);
            let in_btn = local_col >= bx && local_col < bx + btn_width && local_row == btn_y;

            if in_btn {
                if let crate::input::event::MouseEventKind::Down(_) = kind {
                    self.focused_btn = i;
                    self.result = Some(*result);
                    match result {
                        ModalResult::Confirm => {
                            if let Some(ref mut cb) = self.on_confirm {
                                cb();
                            }
                        }
                        ModalResult::Cancel => {
                            if let Some(ref mut cb) = self.on_cancel {
                                cb();
                            }
                        }
                        ModalResult::Custom(_) => {}
                    }
                    self.dirty = true;
                    return true;
                }
            }
        }

        false
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Tab => {
                self.focused_btn = (self.focused_btn + 1) % self.buttons.len();
                self.dirty = true;
                true
            }
            KeyCode::BackTab => {
                if self.focused_btn == 0 {
                    self.focused_btn = self.buttons.len().saturating_sub(1);
                } else {
                    self.focused_btn -= 1;
                }
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                if let Some((_, result)) = self.buttons.get(self.focused_btn) {
                    self.result = Some(*result);
                    match result {
                        ModalResult::Confirm => {
                            if let Some(ref mut cb) = self.on_confirm {
                                cb();
                            }
                        }
                        ModalResult::Cancel => {
                            if let Some(ref mut cb) = self.on_cancel {
                                cb();
                            }
                        }
                        ModalResult::Custom(_) => {}
                    }
                }
                true
            }
            KeyCode::Esc => {
                self.result = Some(ModalResult::Cancel);
                if let Some(ref mut cb) = self.on_cancel {
                    cb();
                }
                true
            }
            _ => false,
        }
    }
}