//! Toast notification widget.
//!
//! A transient notification that appears briefly and auto-dismisses.

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use ratatui::layout::Rect;
use std::time::{Duration, Instant};

/// Severity level for toast notifications.
pub enum ToastKind {
    /// An informational toast.
    Info,
    /// A success toast.
    Success,
    /// A warning toast.
    Warning,
    /// An error toast.
    Error,
}

/// A transient notification toast.
pub struct Toast {
    /// The widget ID for this toast.
    id: WidgetId,
    /// The message text for this toast.
    message: String,
    /// The severity kind for this toast.
    kind: ToastKind,
    /// The creation timestamp for this toast.
    created_at: Instant,
    /// The display duration for this toast.
    duration: Duration,
    /// The theme for this widget.
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Toast {
    /// Creates a new toast with the given ID and message.
    pub fn new(id: WidgetId, message: &str) -> Self {
        Self {
            id,
            message: message.to_string(),
            kind: ToastKind::Info,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 1)),
            dirty: true,
        }
    }

    /// Sets the kind/severity of the toast.
    pub fn with_kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the display duration for this toast.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Returns the toast message text.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns true if this toast has expired and should be removed.
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > self.duration
    }

    fn fg_color(&self) -> Color {
        match self.kind {
            ToastKind::Info => self.theme.fg,
            ToastKind::Success => self.theme.success,
            ToastKind::Warning => self.theme.warning,
            ToastKind::Error => self.theme.error,
        }
    }
}

impl crate::framework::widget::Widget for Toast {
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
        90
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
        plane.z_index = 90;
        plane.fill_bg(self.theme.bg);

        let prefix = match self.kind {
            ToastKind::Info => "[i]",
            ToastKind::Success => "[OK]",
            ToastKind::Warning => "[!]",
            ToastKind::Error => "[X]",
        };

        let full_text = format!("{} {}", prefix, self.message);
        let width = plane.cells.len() / plane.height as usize;

        for (i, c) in full_text.chars().take(width).enumerate() {
            let idx = i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: self.theme.bg,
                    bg: self.fg_color(),
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
    }
}

impl WidgetState for Toast {
    fn state_id(&self) -> Option<&str> {
        Some("toast")
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        let kind_str = match self.kind {
            ToastKind::Info => "info",
            ToastKind::Success => "success",
            ToastKind::Warning => "warning",
            ToastKind::Error => "error",
        };
        json!({
            "message": self.message,
            "kind": kind_str,
            "visible": !self.is_expired(),
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
            self.message = message.to_string();
        }
        if let Some(kind) = json.get("kind").and_then(|v| v.as_str()) {
            self.kind = match kind {
                "success" => ToastKind::Success,
                "warning" => ToastKind::Warning,
                "error" => ToastKind::Error,
                _ => ToastKind::Info,
            };
        }
        self.dirty = true;
        Ok(())
    }
}
