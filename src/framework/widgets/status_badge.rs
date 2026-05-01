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
    id: WidgetId,
    status: String,
    label: String,
    theme: Theme,
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
        let (fg, bg, label) = if status_upper.contains("OK") || status_upper.contains("GREEN") || status_upper == "1" {
            (self.theme.success_fg, self.theme.bg, "OK")
        } else if status_upper.contains("WARN") || status_upper.contains("WARNING") || status_upper.contains("YELLOW") {
            (self.theme.warning_fg, self.theme.bg, "WARN")
        } else if status_upper.contains("ERROR") || status_upper.contains("FAIL") || status_upper.contains("RED") || status_upper == "0" {
            (self.theme.error_fg, self.theme.bg, "ERROR")
        } else if status_upper.is_empty() {
            (self.theme.inactive_fg, self.theme.bg, "EMPTY")
        } else {
            let l: &str = if self.label.is_empty() { status_upper.as_str() } else { &self.label };
            (self.theme.fg, self.theme.bg, l)
        };

        let mut plane = self.render_badge(label, fg, bg, area.width);
        plane.z_index = 0;
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
    fn test_status_badge_new() {
        let badge = StatusBadge::new(WidgetId::new(1));
        assert_eq!(badge.id(), WidgetId::new(1));
        assert_eq!(badge.status, "UNKNOWN");
    }

    #[test]
    fn test_status_badge_with_status() {
        let badge = StatusBadge::new(WidgetId::new(1)).with_status("OK");
        assert_eq!(badge.status, "OK");
    }

    #[test]
    fn test_status_badge_with_label() {
        let badge = StatusBadge::new(WidgetId::new(1)).with_label("Disk OK");
        assert_eq!(badge.label, "Disk OK");
    }

    #[test]
    fn test_status_badge_render_ok() {
        let badge = StatusBadge::new(WidgetId::new(1)).with_status("OK");
        let plane = badge.render(Rect::new(0, 0, 6, 1));
        assert_eq!(plane.cells[0].char, '[');
        assert_eq!(plane.cells[1].char, 'O');
        assert_eq!(plane.cells[2].char, 'K');
        assert_eq!(plane.cells[3].char, ']');
    }

    #[test]
    fn test_status_badge_render_error() {
        let badge = StatusBadge::new(WidgetId::new(1)).with_status("ERROR");
        let plane = badge.render(Rect::new(0, 0, 10, 1));
        assert_eq!(plane.cells[0].char, '[');
        assert_eq!(plane.cells[1].char, 'E');
        assert_eq!(plane.cells[3].char, 'R');
    }

    #[test]
    fn test_status_badge_render_warn() {
        let badge = StatusBadge::new(WidgetId::new(1)).with_status("WARNING");
        let plane = badge.render(Rect::new(0, 0, 10, 1));
        assert_eq!(plane.cells[0].char, '[');
        assert_eq!(plane.cells[1].char, 'W');
    }

    #[test]
    fn test_status_badge_numeric_ok() {
        let badge = StatusBadge::new(WidgetId::new(1)).with_status("1");
        let plane = badge.render(Rect::new(0, 0, 6, 1));
        assert_eq!(plane.cells[1].char, 'O');
    }

    #[test]
    fn test_status_badge_numeric_zero() {
        let badge = StatusBadge::new(WidgetId::new(1)).with_status("0");
        let plane = badge.render(Rect::new(0, 0, 10, 1));
        assert_eq!(plane.cells[1].char, 'E');
    }

    #[test]
    fn test_status_badge_dirty_lifecycle() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        assert!(badge.needs_render());
        badge.clear_dirty();
        assert!(!badge.needs_render());
        badge.set_status("OK");
        assert!(badge.needs_render());
    }

    #[test]
    fn test_status_badge_commands() {
        let cmd = BoundCommand::new("test-cmd --json").label("test");
        let badge = StatusBadge::new(WidgetId::new(1)).bind_command(cmd);
        let cmds = badge.commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].command, "test-cmd --json");
    }

    #[test]
    fn test_status_badge_empty_status() {
        let badge = StatusBadge::new(WidgetId::new(1)).with_status("");
        let plane = badge.render(Rect::new(0, 0, 10, 1));
        assert_eq!(plane.cells[1].char, 'E');
        assert_eq!(plane.cells[2].char, 'M');
        assert_eq!(plane.cells[3].char, 'P');
        assert_eq!(plane.cells[4].char, 'T');
    }

    #[test]
    fn test_status_badge_focusable_returns_true_by_default() {
        let badge = StatusBadge::new(WidgetId::new(1));
        assert!(badge.focusable());
    }

    #[test]
    fn test_status_badge_z_index() {
        let badge = StatusBadge::new(WidgetId::new(1));
        assert_eq!(badge.z_index(), 0);
    }

    #[test]
    fn test_status_badge_with_theme() {
        let theme = Theme::cyberpunk();
        let badge = StatusBadge::new(WidgetId::new(1)).with_theme(theme);
        assert_eq!(badge.theme.name, "cyberpunk");
    }

    #[test]
    fn test_status_badge_set_area() {
        let mut badge = StatusBadge::new(WidgetId::new(1));
        badge.set_area(Rect::new(5, 5, 20, 2));
        let area = badge.area();
        assert_eq!(area.x, 5);
        assert_eq!(area.y, 5);
        assert_eq!(area.width, 20);
        assert_eq!(area.height, 2);
    }
}