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
    id: WidgetId,
    content: String,
    lines: VecDeque<String>,
    max_lines: usize,
    auto_scroll: bool,
    word_wrap: bool,
    theme: Theme,
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
        Self {
            id,
            ..Self::new()
        }
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
                    plane.cells[idx] = Cell { char: c, fg: self.theme.inactive_fg, bg: self.theme.bg, style: Styles::empty(), transparent: false, skip: false };
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

        for (screen_row, line_idx) in (start_idx..self.lines.len()).take(area.height as usize).enumerate() {
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
                        plane.cells[idx] = Cell { char: c, fg: self.theme.fg, bg: self.theme.bg, style: Styles::empty(), transparent: false, skip: false };
                    }
                }
            }
        }

        plane
    }

    fn commands(&self) -> Vec<BoundCommand> {
        self.bound_command.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_text_new() {
        let st = StreamingText::new();
        assert!(st.lines.is_empty());
        assert_eq!(st.max_lines, 200);
        assert!(st.auto_scroll);
        assert!(!st.word_wrap);
    }

    #[test]
    fn test_streaming_text_with_id() {
        let st = StreamingText::with_id(WidgetId::new(4));
        assert_eq!(st.id, WidgetId::new(4));
    }

    #[test]
    fn test_streaming_text_max_lines() {
        let st = StreamingText::new().max_lines(50);
        assert_eq!(st.max_lines, 50);
    }

    #[test]
    fn test_streaming_text_auto_scroll() {
        let st = StreamingText::new().auto_scroll(false);
        assert!(!st.auto_scroll);
    }

    #[test]
    fn test_streaming_text_word_wrap() {
        let st = StreamingText::new().word_wrap(true);
        assert!(st.word_wrap);
    }

    #[test]
    fn test_streaming_text_bind_command() {
        let cmd = BoundCommand::new("curl -N https://stream").label("stream");
        let st = StreamingText::new().bind_command(cmd);
        assert_eq!(st.commands().len(), 1);
    }

    #[test]
    fn test_streaming_text_append() {
        let mut st = StreamingText::new();
        st.append("Hello\nWorld");
        assert_eq!(st.lines.len(), 2);
    }

    #[test]
    fn test_streaming_text_append_max_lines() {
        let mut st = StreamingText::new().max_lines(3);
        for i in 0..5 {
            st.append(&format!("line {}\n", i));
        }
        assert_eq!(st.lines.len(), 3);
    }

    #[test]
    fn test_streaming_text_append_output_text() {
        let mut st = StreamingText::new();
        st.append_output(ParsedOutput::Text("foo\nbar".to_string()));
        assert_eq!(st.lines.len(), 2);
    }

    #[test]
    fn test_streaming_text_append_output_scalar() {
        let mut st = StreamingText::new();
        st.append_output(ParsedOutput::Scalar("single value".to_string()));
        assert_eq!(st.lines.len(), 1);
    }

    #[test]
    fn test_streaming_text_clear() {
        let mut st = StreamingText::new();
        st.append("test");
        st.clear();
        assert!(st.lines.is_empty());
        assert!(st.content.is_empty());
    }

    #[test]
    fn test_streaming_text_content() {
        let mut st = StreamingText::new();
        st.append("hello");
        assert_eq!(st.content(), "hello");
    }

    #[test]
    fn test_streaming_text_render() {
        let mut st = StreamingText::new();
        st.append("Test line");
        let plane = st.render(Rect::new(0, 0, 40, 10));
        assert_eq!(plane.cells[0].char, 'T');
    }

    #[test]
    fn test_streaming_text_render_empty() {
        let st = StreamingText::new();
        let plane = st.render(Rect::new(0, 0, 30, 5));
        assert!(plane.cells.iter().any(|c| c.char == '('));
    }

    #[test]
    fn test_streaming_text_render_word_wrap() {
        let mut st = StreamingText::new().word_wrap(true);
        st.append("This is a very long line that should wrap");
        let plane = st.render(Rect::new(0, 0, 20, 10));
        assert_eq!(plane.cells[0].char, 'T');
    }

    #[test]
    fn test_streaming_text_dirty_lifecycle() {
        let mut st = StreamingText::new();
        assert!(st.needs_render());
        st.clear_dirty();
        assert!(!st.needs_render());
        st.append("new content");
        assert!(st.needs_render());
    }

    #[test]
    fn test_streaming_text_with_theme() {
        let theme = Theme::solarized_dark();
        let st = StreamingText::new().with_theme(theme);
        assert_eq!(st.theme.name, "solarized-dark");
    }

    #[test]
    fn test_streaming_text_multiline_append() {
        let mut st = StreamingText::new();
        st.append("line1\nline2\nline3");
        assert_eq!(st.lines.len(), 3);
    }

    #[test]
    fn test_streaming_text_auto_scroll_shows_latest() {
        let mut st = StreamingText::new().max_lines(3);
        for i in 0..5 {
            st.append(&format!("line {}\n", i));
        }
        assert_eq!(st.lines[0], "line 2");
        assert_eq!(st.lines[2], "line 4");
    }
}