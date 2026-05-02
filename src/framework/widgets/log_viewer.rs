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
    pub id: WidgetId,
    pub lines: VecDeque<LogLine>,
    pub max_lines: usize,
    pub auto_scroll: bool,
    pub filter: Option<String>,
    pub theme: Theme,
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

    pub fn level_color(&self, level: LogLevel) -> Color {
        match level {
            LogLevel::Fatal => self.theme.error,
            LogLevel::Error => self.theme.error,
            LogLevel::Warn => self.theme.warning,
            LogLevel::Info => self.theme.fg,
            LogLevel::Debug => self.theme.fg_muted,
        }
    }

    pub fn level_prefix(&self, level: LogLevel) -> &'static str {
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
                        fg: self.theme.fg_muted,
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
                                fg: self.theme.fg_muted,
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

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}
