//! StatusBadge widget — renders a colored status label.
//!
//! Binds to a CLI command that outputs JSON with `status` and optional `label`.
//! Renders as `[OK]`, `[WARN]`, or `[ERROR]` with theme colors.
//!
//! ## TOML definition
//!
//! ```toml
//! [[widget]]
//! id = 1
//! type = "StatusBadge"
//! bind = "dracon-sync status --json"
//! parser = { type = "json_key", key = "status" }
//! refresh = 5
//! ```

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::command::BoundCommand;
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct StatusBadge {
    pub id: WidgetId,
    pub status: String,
    pub label: String,
    pub theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    bound_command: Option<BoundCommand>,
}

impl StatusBadge {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            status: String::from("UNKNOWN"),
            label: String::new(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 12, 1)),
            dirty: true,
            bound_command: None,
        }
    }

    pub fn with_status(mut self, status: &str) -> Self {
        self.status = status.to_string();
        self.dirty = true;
        self
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self.dirty = true;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self.dirty = true;
        self
    }

    pub fn bind_command(mut self, cmd: BoundCommand) -> Self {
        self.bound_command = Some(cmd);
        self
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
        self.dirty = true;
    }

    pub fn label_text(&self) -> &str {
        &self.label
    }

    fn render_badge(&self, text: &str, fg: Color, bg: Color, width: u16) -> Plane {
        let mut plane = Plane::new(0, width, 1);

        let content = format!("[{}]", text);
        let max = plane.width.min(content.len() as u16);

        for (i, c) in content.chars().take(max as usize).enumerate() {
            plane.cells[i] = Cell {
                char: c,
                fg,
                bg,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }

        plane
    }
}

impl Widget for StatusBadge {
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
        let status_upper = self.status.to_uppercase();
        let (fg, bg, label) =
            if status_upper.contains("OK") || status_upper.contains("GREEN") || status_upper == "1"
            {
                (self.theme.success, self.theme.bg, "OK")
            } else if status_upper.contains("WARN")
                || status_upper.contains("WARNING")
                || status_upper.contains("YELLOW")
            {
                (self.theme.warning, self.theme.bg, "WARN")
            } else if status_upper.contains("ERROR")
                || status_upper.contains("FAIL")
                || status_upper.contains("RED")
                || status_upper == "0"
            {
                (self.theme.error, self.theme.bg, "ERROR")
            } else if status_upper.is_empty() {
                (self.theme.fg_muted, self.theme.bg, "EMPTY")
            } else {
                let l: &str = if self.label.is_empty() {
                    status_upper.as_str()
                } else {
                    &self.label
                };
                (self.theme.fg, self.theme.bg, l)
            };

        let mut plane = self.render_badge(label, fg, bg, area.width);
        plane.z_index = 0;
        plane
    }

    fn commands(&self) -> Vec<BoundCommand> {
        self.bound_command.iter().cloned().collect()
    }

    fn apply_command_output(&mut self, output: &crate::framework::command::ParsedOutput) {
        if let crate::framework::command::ParsedOutput::Scalar(s) = output {
            self.set_status(s);
        }
    }
}
