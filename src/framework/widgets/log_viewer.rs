//! LogViewer widget — displays scrolling log lines from a CLI command.
//!
//! Binds to a CLI command that outputs line-by-line log data.
//! Renders as a virtualized scrolling list with auto-scroll to bottom.
//!
//! ## TOML definition
//!
//! ```toml
//! [[widget]]
//! id = 1
//! type = "LogViewer"
//! bind = "journalctl -f"
//! max_lines = 1000
//! auto_scroll = true
//! filter = "error"
//! ```

use std::collections::VecDeque;

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::command::{BoundCommand, ParsedOutput};
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct LogLine {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

pub struct LogViewer {
    id: WidgetId,
    lines: VecDeque<LogLine>,
    max_lines: usize,
    auto_scroll: bool,
    filter: Option<String>,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    bound_command: Option<BoundCommand>,
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            id: WidgetId::default_id(),
            lines: VecDeque::new(),
            max_lines: 500,
            auto_scroll: true,
            filter: None,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 20)),
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

    pub fn filter(mut self, pattern: &str) -> Self {
        self.filter = Some(pattern.to_string());
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

    pub fn append_line(&mut self, line: &str) {
        let parsed = self.parse_line(line);
        if self.filter.is_some() && !self.matches_filter(&parsed) {
            return;
        }
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(parsed);
        self.dirty = true;
    }

    pub fn append_output(&mut self, output: ParsedOutput) {
        match output {
            ParsedOutput::Lines(log_lines) => {
                for line in log_lines {
                    let raw = line.text.clone();
                    if self.filter.is_none() || raw.contains(self.filter.as_ref().unwrap()) {
                        if self.lines.len() >= self.max_lines {
                            self.lines.pop_front();
                        }
                        self.lines.push_back(LogLine {
                            timestamp: String::new(),
                            level: self.detect_level(&line.severity),
                            message: line.text.clone(),
                            raw,
                        });
                    }
                }
            }
            ParsedOutput::Text(text) => {
                for line in text.lines() {
                    self.append_line(line);
                }
            }
            _ => {}
        }
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.dirty = true;
    }

    fn parse_line(&self, raw: &str) -> LogLine {
        let (timestamp, rest) = raw.split_once(' ').unwrap_or(("", raw));
        let level = self.detect_level(raw);
        let message = rest.to_string();
        LogLine {
            timestamp: timestamp.to_string(),
            level,
            message,
            raw: raw.to_string(),
        }
    }

    fn detect_level(&self, text: &str) -> LogLevel {
        let upper = text.to_uppercase();
        if upper.starts_with("FATAL") || upper.starts_with("CRITICAL") {
            LogLevel::Fatal
        } else if upper.starts_with("ERROR") || upper.starts_with("ERR") {
            LogLevel::Error
        } else if upper.starts_with("WARN") || upper.starts_with("WARNING") {
            LogLevel::Warn
        } else if upper.starts_with("DEBUG") || upper.starts_with("DBG") {
            LogLevel::Debug
        } else {
            LogLevel::Info
        }
    }

    fn matches_filter(&self, line: &LogLine) -> bool {
        if let Some(ref f) = self.filter {
            line.raw.to_lowercase().contains(&f.to_lowercase())
        } else {
            true
        }
    }

    fn matches_filter_by_raw(&self, raw: &str) -> bool {
        if let Some(ref f) = self.filter {
            raw.contains(f)
        } else {
            true
        }
    }

    fn level_color(&self, level: LogLevel) -> Color {
        match level {
            LogLevel::Fatal => self.theme.error_fg,
            LogLevel::Error => self.theme.error_fg,
            LogLevel::Warn => self.theme.warning_fg,
            LogLevel::Info => self.theme.fg,
            LogLevel::Debug => self.theme.inactive_fg,
        }
    }

    fn level_prefix(&self, level: LogLevel) -> &'static str {
        match level {
            LogLevel::Debug => "[D]",
            LogLevel::Info => "[I]",
            LogLevel::Warn => "[W]",
            LogLevel::Error => "[E]",
            LogLevel::Fatal => "[F]",
        }
    }
}

impl Default for LogViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for LogViewer {
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
            let msg = "(no log output)";
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
                let level_color = self.level_color(line.level);
                let prefix = self.level_prefix(line.level);

                let mut col = 0;

                for c in prefix.chars() {
                    if col < area.width as usize {
                        plane.cells[screen_row * area.width as usize + col] = Cell {
                            char: c,
                            fg: level_color,
                            bg: self.theme.bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                        col += 1;
                    }
                }

                if col < area.width as usize && !line.timestamp.is_empty() {
                    for c in line.timestamp.chars().take(8) {
                        if col < area.width as usize {
                            plane.cells[screen_row * area.width as usize + col] = Cell {
                                char: c,
                                fg: self.theme.inactive_fg,
                                bg: self.theme.bg,
                                style: Styles::empty(),
                                transparent: false,
                                skip: false,
                            };
                            col += 1;
                        }
                    }
                }

                if col < area.width as usize {
                    plane.cells[screen_row * area.width as usize + col] = Cell {
                        char: ' ',
                        fg: self.theme.fg,
                        bg: self.theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                    col += 1;
                }

                for c in line
                    .message
                    .chars()
                    .take((area.width as usize).saturating_sub(col))
                {
                    if col < area.width as usize {
                        plane.cells[screen_row * area.width as usize + col] = Cell {
                            char: c,
                            fg: self.theme.fg,
                            bg: self.theme.bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                        col += 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_viewer_new() {
        let lv = LogViewer::new();
        assert!(lv.lines.is_empty());
        assert_eq!(lv.max_lines, 500);
        assert!(lv.auto_scroll);
    }

    #[test]
    fn test_log_viewer_with_id() {
        let lv = LogViewer::with_id(WidgetId::new(2));
        assert_eq!(lv.id, WidgetId::new(2));
    }

    #[test]
    fn test_log_viewer_max_lines() {
        let lv = LogViewer::new().max_lines(100);
        assert_eq!(lv.max_lines, 100);
    }

    #[test]
    fn test_log_viewer_auto_scroll() {
        let lv = LogViewer::new().auto_scroll(false);
        assert!(!lv.auto_scroll);
    }

    #[test]
    fn test_log_viewer_filter() {
        let lv = LogViewer::new().filter("error");
        assert_eq!(lv.filter, Some("error".to_string()));
    }

    #[test]
    fn test_log_viewer_bind_command() {
        let cmd = BoundCommand::new("journalctl -f").label("logs");
        let lv = LogViewer::new().bind_command(cmd);
        assert_eq!(lv.commands().len(), 1);
    }

    #[test]
    fn test_log_viewer_append_line() {
        let mut lv = LogViewer::new();
        lv.append_line("2024-01-01 INFO Starting up");
        assert_eq!(lv.lines.len(), 1);
        assert_eq!(lv.lines[0].message, "INFO Starting up");
    }

    #[test]
    fn test_log_viewer_append_line_max_lines() {
        let mut lv = LogViewer::new().max_lines(3);
        for i in 0..5 {
            lv.append_line(&format!("line {}", i));
        }
        assert_eq!(lv.lines.len(), 3);
        assert_eq!(lv.lines[0].raw, "line 2");
        assert_eq!(lv.lines[2].raw, "line 4");
    }

    #[test]
    fn test_log_viewer_parse_level_error() {
        let mut lv = LogViewer::new();
        lv.append_line("ERROR something bad happened");
        assert_eq!(lv.lines[0].level, LogLevel::Error);
    }

    #[test]
    fn test_log_viewer_parse_level_warn() {
        let mut lv = LogViewer::new();
        lv.append_line("WARNING deprecated feature");
        assert_eq!(lv.lines[0].level, LogLevel::Warn);
    }

    #[test]
    fn test_log_viewer_parse_level_debug() {
        let mut lv = LogViewer::new();
        lv.append_line("DEBUG connection established");
        assert_eq!(lv.lines[0].level, LogLevel::Debug);
    }

    #[test]
    fn test_log_viewer_parse_level_fatal() {
        let mut lv = LogViewer::new();
        lv.append_line("FATAL system crash");
        assert_eq!(lv.lines[0].level, LogLevel::Fatal);
    }

    #[test]
    fn test_log_viewer_filter_reject() {
        let mut lv = LogViewer::new().filter("error");
        lv.append_line("INFO this is info");
        assert!(lv.lines.is_empty());
    }

    #[test]
    fn test_log_viewer_filter_accept() {
        let mut lv = LogViewer::new().filter("error");
        lv.append_line("ERROR something failed");
        assert_eq!(lv.lines.len(), 1);
    }

    #[test]
    fn test_log_viewer_clear() {
        let mut lv = LogViewer::new();
        lv.append_line("test line");
        lv.clear();
        assert!(lv.lines.is_empty());
    }

    #[test]
    fn test_log_viewer_render() {
        let mut lv = LogViewer::new();
        lv.append_line("INFO test message");
        let plane = lv.render(Rect::new(0, 0, 40, 10));
        assert_eq!(plane.cells[0].char, '[');
    }

    #[test]
    fn test_log_viewer_render_empty() {
        let lv = LogViewer::new();
        let plane = lv.render(Rect::new(0, 0, 30, 5));
        assert!(plane.cells.iter().any(|c| c.char == '('));
    }

    #[test]
    fn test_log_viewer_dirty_lifecycle() {
        let mut lv = LogViewer::new();
        assert!(lv.needs_render());
        lv.clear_dirty();
        assert!(!lv.needs_render());
        lv.append_line("new line");
        assert!(lv.needs_render());
    }

    #[test]
    fn test_log_viewer_with_theme() {
        let theme = Theme::nord();
        let lv = LogViewer::new().with_theme(theme);
        assert_eq!(lv.theme.name, "nord");
    }

    #[test]
    fn test_log_viewer_level_color() {
        let lv = LogViewer::new();
        assert_eq!(lv.level_color(LogLevel::Error), lv.theme.error_fg);
        assert_eq!(lv.level_color(LogLevel::Warn), lv.theme.warning_fg);
    }

    #[test]
    fn test_log_viewer_level_prefix() {
        let lv = LogViewer::new();
        assert_eq!(lv.level_prefix(LogLevel::Error), "[E]");
        assert_eq!(lv.level_prefix(LogLevel::Info), "[I]");
    }

    #[test]
    fn test_log_viewer_apply_command_output_text() {
        use crate::framework::command::ParsedOutput;
        let mut lv = LogViewer::new();
        lv.apply_command_output(&ParsedOutput::Text(
            "ERROR test error\nINFO test info".to_string(),
        ));
        assert_eq!(lv.lines.len(), 2);
    }

    #[test]
    fn test_log_viewer_apply_command_output_lines() {
        use crate::framework::command::{LoggedLine, ParsedOutput};
        let mut lv = LogViewer::new();
        lv.apply_command_output(&ParsedOutput::Lines(vec![
            LoggedLine::new("FATAL crash", "fatal"),
            LoggedLine::new("ERROR failure", "error"),
        ]));
        assert_eq!(lv.lines.len(), 2);
    }
}
