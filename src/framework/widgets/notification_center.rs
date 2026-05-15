//! Notification center widget.
//!
//! A stacked notification queue that appears in the top-right corner.
//! Notifications auto-dismiss after a configurable duration and can be
//! clicked to dismiss immediately.

use crate::compositor::{Color, Plane, Styles};
use crate::framework::hitzone::ScopedZoneRegistry;
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use crate::input::event::MouseEventKind;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::time::{Duration, Instant};

/// Severity level for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    /// An informational notification.
    Info,
    /// A success notification.
    Success,
    /// A warning notification.
    Warning,
    /// An error notification.
    Error,
}

/// A single notification entry.
pub struct Notification {
    pub id: usize,
    pub title: String,
    pub message: String,
    pub kind: NotificationKind,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Notification {
    /// Returns true if this notification has expired.
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > self.duration
    }

    fn accent_color(&self, theme: &Theme) -> Color {
        match self.kind {
            NotificationKind::Info => theme.info,
            NotificationKind::Success => theme.success,
            NotificationKind::Warning => theme.warning,
            NotificationKind::Error => theme.error,
        }
    }

    fn icon(&self) -> char {
        match self.kind {
            NotificationKind::Info => 'i',
            NotificationKind::Success => '✔',
            NotificationKind::Warning => '!',
            NotificationKind::Error => '✖',
        }
    }
}

/// A notification center that displays a stack of transient notifications.
pub struct NotificationCenter {
    id: WidgetId,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    notifications: RefCell<Vec<Notification>>,
    next_id: RefCell<usize>,
    max_width: u16,
    zones: RefCell<ScopedZoneRegistry<usize>>,
}

impl NotificationCenter {
    /// Creates a new notification center.
    pub fn new(theme: Theme) -> Self {
        Self {
            id: WidgetId::default(),
            theme,
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 1)),
            dirty: true,
            notifications: RefCell::new(Vec::new()),
            next_id: RefCell::new(1),
            max_width: 40,
            zones: RefCell::new(ScopedZoneRegistry::new()),
        }
    }

    /// Sets the maximum width of notification cards.
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Queues a new notification.
    pub fn notify(&mut self, title: &str, message: &str, kind: NotificationKind) {
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;
        self.notifications.borrow_mut().push(Notification {
            id,
            title: title.to_string(),
            message: message.to_string(),
            kind,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
        });
        self.dirty = true;
    }

    /// Queues an info notification.
    pub fn info(&mut self, title: &str, message: &str) {
        self.notify(title, message, NotificationKind::Info);
    }

    /// Queues a success notification.
    pub fn success(&mut self, title: &str, message: &str) {
        self.notify(title, message, NotificationKind::Success);
    }

    /// Queues a warning notification.
    pub fn warn(&mut self, title: &str, message: &str) {
        self.notify(title, message, NotificationKind::Warning);
    }

    /// Queues an error notification.
    pub fn error(&mut self, title: &str, message: &str) {
        self.notify(title, message, NotificationKind::Error);
    }

    /// Removes expired notifications and returns true if any were removed.
    fn prune_expired(&self) -> bool {
        let before = self.notifications.borrow().len();
        self.notifications.borrow_mut().retain(|n| !n.is_expired());
        let after = self.notifications.borrow().len();
        before != after
    }

    /// Dismisses a notification by its ID.
    fn dismiss(&mut self, id: usize) {
        self.notifications.borrow_mut().retain(|n| n.id != id);
        self.dirty = true;
    }

    /// Returns the number of active notifications.
    pub fn len(&self) -> usize {
        self.notifications.borrow().len()
    }

    /// Returns true if there are no active notifications.
    pub fn is_empty(&self) -> bool {
        self.notifications.borrow().is_empty()
    }

    fn card_height(&self, notif: &Notification) -> u16 {
        // title line + message lines
        let msg_lines = 1 + (notif.message.len() as u16).saturating_sub(1) / (self.max_width - 4);
        2 + msg_lines.min(3) // border + title + clamped message
    }
}

impl crate::framework::widget::Widget for NotificationCenter {
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
        9500
    }

    fn focusable(&self) -> bool {
        false
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.prune_expired()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(9500, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let notifs = self.notifications.borrow();
        if notifs.is_empty() {
            return plane;
        }

        self.zones.borrow_mut().clear();

        let card_w = self.max_width.min(area.width.saturating_sub(2));
        let mut y = 1u16;

        for notif in notifs.iter() {
            let card_h = self.card_height(notif);
            if y + card_h >= area.height {
                break;
            }
            let x = area.width.saturating_sub(card_w + 1);

            // Register hit zone for click-to-dismiss
            self.zones
                .borrow_mut()
                .register(notif.id, x, y, card_w, card_h);

            let accent = notif.accent_color(&self.theme);

            // Draw card background
            for cy in y..y + card_h {
                for cx in x..x + card_w {
                    let idx = (cy * area.width + cx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = self.theme.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Top border accent
            for cx in x..x + card_w {
                let idx = (y * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = accent;
                    plane.cells[idx].fg = self.theme.fg_on_accent;
                    plane.cells[idx].transparent = false;
                }
            }

            // Icon + title on top border
            let title_text = format!(" {}  {}", notif.icon(), notif.title);
            for (i, c) in title_text.chars().enumerate() {
                let cx = x + 1 + i as u16;
                if cx < x + card_w {
                    let idx = (y * area.width + cx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = self.theme.fg_on_accent;
                        plane.cells[idx].bg = accent;
                        plane.cells[idx].style = Styles::BOLD;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Message body
            let msg_y = y + 1;
            let max_msg_chars = (card_w - 4) as usize;
            let msg_text = if notif.message.len() > max_msg_chars {
                format!("{}…", &notif.message[..max_msg_chars.saturating_sub(1)])
            } else {
                notif.message.clone()
            };
            for (i, c) in msg_text.chars().enumerate() {
                let cx = x + 2 + i as u16;
                let cy = msg_y;
                if cx < x + card_w && cy < y + card_h {
                    let idx = (cy * area.width + cx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = self.theme.fg;
                        plane.cells[idx].bg = self.theme.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            // Bottom border line (subtle outline)
            let bottom_y = y + card_h - 1;
            for cx in x..x + card_w {
                let idx = (bottom_y * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = self.theme.outline;
                    plane.cells[idx].bg = self.theme.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }

            // Rounded corners
            let corners = [('╭', x, y), ('╮', x + card_w - 1, y), ('╰', x, bottom_y), ('╯', x + card_w - 1, bottom_y)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = self.theme.outline;
                    plane.cells[idx].bg = if *cy == y { accent } else { self.theme.surface_elevated };
                    plane.cells[idx].transparent = false;
                }
            }

            y += card_h + 1; // gap between cards
        }

        plane
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if let MouseEventKind::Down(_) = kind {
            let hit = self.zones.borrow().dispatch(col, row);
            if let Some(id) = hit {
                self.dismiss(id);
                return true;
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }
}

impl WidgetState for NotificationCenter {
    fn state_id(&self) -> Option<&str> { None }
    fn to_json(&self) -> serde_json::Value { serde_json::json!({}) }
    fn apply_json(&mut self, _json: &serde_json::Value) -> Result<(), crate::error::DraconError> { Ok(()) }
}
