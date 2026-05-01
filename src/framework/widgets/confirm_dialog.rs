//! ConfirmDialog widget — a modal confirmation box with Confirm/Cancel.
//!
//! Binds to a CLI command that requires confirmation before execution.
//! Renders centered with a title, message, and two buttons.
//!
//! ## TOML definition
//!
//! ```toml
//! [[widget]]
//! id = 1
//! type = "ConfirmDialog"
//! title = "Run destructive command?"
//! message = "This will delete all data. Continue?"
//! confirm_label = "Delete"
//! cancel_label = "Cancel"
//! bind = "rm -rf /"
//! danger = true
//! ```

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::command::BoundCommand;
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResult {
    Confirmed,
    Cancelled,
}

pub struct ConfirmDialog {
    pub id: WidgetId,
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub cancel_label: String,
    pub result: Option<ConfirmResult>,
    pub danger: bool,
    pub theme: Theme,
    focused: bool,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    bound_command: Option<BoundCommand>,
}

impl ConfirmDialog {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            id: WidgetId::default_id(),
            title: title.to_string(),
            message: message.to_string(),
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            result: None,
            danger: false,
            theme: Theme::default(),
            focused: false,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 7)),
            dirty: true,
            bound_command: None,
        }
    }

    pub fn with_id(id: WidgetId, title: &str, message: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            message: message.to_string(),
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            result: None,
            danger: false,
            theme: Theme::default(),
            focused: false,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 7)),
            dirty: true,
            bound_command: None,
        }
    }

    pub fn confirm_label(mut self, label: &str) -> Self {
        self.confirm_label = label.to_string();
        self.dirty = true;
        self
    }

    pub fn cancel_label(mut self, label: &str) -> Self {
        self.cancel_label = label.to_string();
        self.dirty = true;
        self
    }

    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self.dirty = true;
        self
    }

    pub fn bind_command(mut self, cmd: BoundCommand) -> Self {
        self.bound_command = Some(cmd);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self.dirty = true;
        self
    }

    pub fn confirmed(&self) -> Option<ConfirmResult> {
        self.result
    }

    pub fn clear_result(&mut self) {
        self.result = None;
    }

    fn render_centered_text(
        text: &str,
        width: u16,
        fg: Color,
        bg: Color,
        style: Styles,
    ) -> Vec<Cell> {
        let mut cells = Vec::new();
        let display = if text.len() > width as usize - 4 {
            text.chars().take(width as usize - 6).collect::<String>() + ".."
        } else {
            text.to_string()
        };
        let padding = (width.saturating_sub(display.len() as u16)) / 2;
        for _ in 0..padding {
            cells.push(Cell {
                char: ' ',
                fg,
                bg,
                style,
                transparent: false,
                skip: false,
            });
        }
        for c in display.chars() {
            cells.push(Cell {
                char: c,
                fg,
                bg,
                style,
                transparent: false,
                skip: false,
            });
        }
        while cells.len() < width as usize {
            cells.push(Cell {
                char: ' ',
                fg,
                bg,
                style,
                transparent: false,
                skip: false,
            });
        }
        cells
    }
}

impl Widget for ConfirmDialog {
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

    fn focusable(&self) -> bool {
        true
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);

        let border_fg = if self.danger {
            self.theme.error_fg
        } else {
            self.theme.fg
        };
        let btn_fg = if self.focused {
            self.theme.accent
        } else {
            self.theme.fg
        };

        for x in 0..area.width {
            plane.cells[x as usize] = Cell {
                char: '─',
                fg: border_fg,
                bg: self.theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
            let bottom_idx = ((area.height as usize - 1) * area.width as usize) + x as usize;
            if bottom_idx < plane.cells.len() {
                plane.cells[bottom_idx] = Cell {
                    char: '─',
                    fg: border_fg,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
        for y in 1..area.height.saturating_sub(1) {
            let left_idx = (y as usize) * area.width as usize;
            plane.cells[left_idx] = Cell {
                char: '│',
                fg: border_fg,
                bg: self.theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
            let right_idx = left_idx + area.width as usize - 1;
            if right_idx < plane.cells.len() {
                plane.cells[right_idx] = Cell {
                    char: '│',
                    fg: border_fg,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
        let corners = [
            (0usize, '┌'),
            ((area.width - 1) as usize, '┐'),
            (((area.height - 1) as usize) * (area.width as usize), '└'),
            (
                (((area.height - 1) as usize) * (area.width as usize)) + (area.width as usize) - 1,
                '┘',
            ),
        ];
        for (idx, ch) in corners {
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: ch,
                    fg: border_fg,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        let title_cells = Self::render_centered_text(
            &self.title,
            area.width,
            self.theme.fg,
            self.theme.bg,
            Styles::BOLD,
        );
        for (i, cell) in title_cells
            .into_iter()
            .enumerate()
            .take(area.width as usize)
        {
            plane.cells[area.width as usize + i] = cell;
        }

        let msg_cells = Self::render_centered_text(
            &self.message,
            area.width,
            self.theme.fg,
            self.theme.bg,
            Styles::empty(),
        );
        let msg_row = (area.height / 2) as usize;
        for (i, cell) in msg_cells.into_iter().enumerate().take(area.width as usize) {
            plane.cells[msg_row * area.width as usize + i] = cell;
        }

        let btn_row = (area.height - 2) as usize;
        let total_btn_len = self.confirm_label.len() + self.cancel_label.len() + 5;
        let start_col = (area.width.saturating_sub(total_btn_len as u16)) / 2;

        let confirm_str = format!("[{}]", self.confirm_label);
        for (i, c) in confirm_str.chars().enumerate().take(area.width as usize) {
            let idx = btn_row * area.width as usize + start_col as usize + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: if self.danger && self.focused {
                        self.theme.error_fg
                    } else {
                        btn_fg
                    },
                    bg: self.theme.bg,
                    style: if self.danger {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    },
                    transparent: false,
                    skip: false,
                };
            }
        }

        let cancel_str = format!("[{}]", self.cancel_label);
        let cancel_start = start_col as usize + confirm_str.len() + 3;
        for (i, c) in cancel_str.chars().enumerate().take(area.width as usize) {
            let idx = btn_row * area.width as usize + cancel_start + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: btn_fg,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }

    fn commands(&self) -> Vec<BoundCommand> {
        self.bound_command.iter().cloned().collect()
    }
}
