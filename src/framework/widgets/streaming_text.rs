//! StreamingText widget — displays text that updates in real-time.
//!
//! Binds to a CLI command that outputs streaming text (like `curl`, `ping`, `tail -f`).
//! Renders as a scrolling text area with auto-scroll, word-wrap, and line limit.
//!
//! ## TOML definition
//!
//! ```toml
//! [[widget]]
//! id = 1
//! type = "StreamingText"
//! bind = "curl -N https://api.example.com/stream"
//! max_lines = 100
//! auto_scroll = true
//! word_wrap = true
//! ```

use std::collections::VecDeque;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::command::{BoundCommand, ParsedOutput};
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

pub struct StreamingText {
    pub id: WidgetId,
    pub content: String,
    pub lines: VecDeque<String>,
    pub max_lines: usize,
    pub auto_scroll: bool,
    pub word_wrap: bool,
    pub theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    bound_command: Option<BoundCommand>,
}

impl StreamingText {
    pub fn new() -> Self {
        Self {
            id: WidgetId::default_id(),
            content: String::new(),
            lines: VecDeque::new(),
            max_lines: 200,
            auto_scroll: true,
            word_wrap: false,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 15)),
            dirty: true,
            bound_command: None,
        }
    }

    pub fn with_id(id: WidgetId) -> Self {
        Self { id, ..Self::new() }
    }

    pub fn max_lines(mut self, max: usize) -> Self {
        self.max_lines = max;
        self.dirty = true;
        self
    }

    pub fn auto_scroll(mut self, auto: bool) -> Self {
        self.auto_scroll = auto;
        self.dirty = true;
        self
    }

    pub fn word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
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

    pub fn append(&mut self, text: &str) {
        for line in text.lines() {
            if self.lines.len() >= self.max_lines {
                self.lines.pop_front();
            }
            self.lines.push_back(line.to_string());
        }
        self.content.push_str(text);
        self.dirty = true;
    }

    pub fn append_output(&mut self, output: ParsedOutput) {
        match output {
            ParsedOutput::Text(t) => self.append(&t),
            ParsedOutput::Scalar(s) => self.append(&s),
            ParsedOutput::Lines(log_lines) => {
                for line in log_lines {
                    if self.lines.len() >= self.max_lines {
                        self.lines.pop_front();
                    }
                    self.lines.push_back(line.text);
                }
            }
            _ => {}
        }
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.lines.clear();
        self.dirty = true;
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Default for StreamingText {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for StreamingText {
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

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);

        if self.lines.is_empty() {
            let msg = "(waiting for input...)";
            let col_start = (area.width as usize).saturating_sub(msg.len()) / 2;
            let row = (area.height / 2) as usize;
            let char_index = row * (area.width as usize) + col_start;
            for (i, c) in msg.chars().enumerate() {
                let idx = char_index + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: self.theme.inactive_fg,
                        bg: self.theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
            return plane;
        }

        let lines_to_show = self.lines.len().min(area.height as usize);
        let start_idx = if self.auto_scroll {
            self.lines.len().saturating_sub(lines_to_show)
        } else {
            0
        };

        for (screen_row, line_idx) in (start_idx..self.lines.len())
            .take(area.height as usize)
            .enumerate()
        {
            if let Some(line) = self.lines.get(line_idx) {
                let display = if self.word_wrap {
                    let chars = line.chars().peekable();
                    let mut col = 0;
                    let mut result = String::new();
                    for c in chars {
                        if col >= area.width as usize {
                            result.push('\n');
                            col = 0;
                        }
                        result.push(c);
                        col += 1;
                    }
                    result
                } else {
                    line.chars().take(area.width as usize).collect()
                };

                for (i, c) in display.chars().enumerate().take(area.width as usize) {
                    let idx = screen_row * area.width as usize + i;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = Cell {
                            char: c,
                            fg: self.theme.fg,
                            bg: self.theme.bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }
        }

        plane
    }

    fn commands(&self) -> Vec<BoundCommand> {
        self.bound_command.iter().cloned().collect()
    }

    fn apply_command_output(&mut self, output: &crate::framework::command::ParsedOutput) {
        self.append_output(output.clone());
    }
}
