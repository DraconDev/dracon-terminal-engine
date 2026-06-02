//! Embedded Notification Hub scene for the showcase.
//!
//! Full notification management interface with:
//!   - Split layout: notification feed left, detail panel right
//!   - Filter pills for priority filtering
//!   - Click to select and view notification details
//!   - Auto-generation timer
//!   - Action buttons (clear all, start/stop auto)

use crate::scenes::shared_helpers::{draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::{NotificationKind, StatusBar, StatusSegment};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;
use std::cell::RefCell;

#[derive(Clone, Copy, PartialEq)]
enum FilterMode {
    All,
    Info,
    Success,
    Warning,
    Error,
}

impl FilterMode {
    fn label(self) -> &'static str {
        match self {
            FilterMode::All => "All",
            FilterMode::Info => "Info",
            FilterMode::Success => "Success",
            FilterMode::Warning => "Warning",
            FilterMode::Error => "Error",
        }
    }
    fn next(self) -> Self {
        match self {
            FilterMode::All => FilterMode::Info,
            FilterMode::Info => FilterMode::Success,
            FilterMode::Success => FilterMode::Warning,
            FilterMode::Warning => FilterMode::Error,
            FilterMode::Error => FilterMode::All,
        }
    }
}

// Notification data for tracking
struct NotifEntry {
    title: String,
    message: String,
    kind: NotificationKind,
    timestamp: String,
}

const SIDEBAR_WIDTH: u16 = 38;

pub struct NotificationCenterScene {
    theme: Theme,
    show_help: bool,
    tick_count: usize,
    auto_running: bool,
    filter: FilterMode,
    total_added: usize,
    total_dismissed: usize,
    keybindings: KeybindingSet,
    dirty: bool,
    area: std::cell::Cell<Rect>,

    // Notification tracking for feed
    notifications: RefCell<Vec<NotifEntry>>,
    selected_idx: Option<usize>,
    focused_side: usize, // 0 = feed, 1 = detail

    status_bar: RefCell<StatusBar>,
}

impl NotificationCenterScene {
    pub fn new(theme: Theme) -> Self {
        let status_bar = StatusBar::new(WidgetId::new(2009))
            .add_segment(StatusSegment::new(
                "SPACE:add | A:auto | C:clear | F:filter | F1:help | Esc:back",
            ))
            .with_theme(theme.clone());
        Self {
            theme,
            show_help: false,
            tick_count: 0,
            auto_running: false,
            filter: FilterMode::All,
            total_added: 2,
            total_dismissed: 0,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            notifications: RefCell::new(vec![
                NotifEntry {
                    title: "Welcome".into(),
                    message: "NotificationCenter demo".into(),
                    kind: NotificationKind::Info,
                    timestamp: "now".into(),
                },
                NotifEntry {
                    title: "Ready".into(),
                    message: "All systems operational".into(),
                    kind: NotificationKind::Success,
                    timestamp: "now".into(),
                },
            ]),
            selected_idx: None,
            focused_side: 0,
            status_bar: RefCell::new(status_bar),
        }
    }

    fn add_filtered_notification(&mut self) {
        let kinds = [
            (
                NotificationKind::Info,
                "Info",
                vec![
                    "New message received",
                    "Update available",
                    "Sync complete",
                    "User online",
                ],
            ),
            (
                NotificationKind::Success,
                "Done",
                vec![
                    "File saved",
                    "Build succeeded",
                    "Upload complete",
                    "Tests passed",
                ],
            ),
            (
                NotificationKind::Warning,
                "Warning",
                vec![
                    "Disk space low",
                    "Rate limit approaching",
                    "Certificate expiring",
                    "Memory high",
                ],
            ),
            (
                NotificationKind::Error,
                "Error",
                vec![
                    "Connection failed",
                    "Permission denied",
                    "Timeout exceeded",
                    "Disk full",
                ],
            ),
        ];

        let (kind, title_prefix, msgs) = &kinds[self.tick_count % kinds.len()];

        match self.filter {
            FilterMode::All => {}
            FilterMode::Info if *kind != NotificationKind::Info => {
                self.tick_count += 1;
                return;
            }
            FilterMode::Success if *kind != NotificationKind::Success => {
                self.tick_count += 1;
                return;
            }
            FilterMode::Warning if *kind != NotificationKind::Warning => {
                self.tick_count += 1;
                return;
            }
            FilterMode::Error if *kind != NotificationKind::Error => {
                self.tick_count += 1;
                return;
            }
            _ => {}
        }

        let msg = msgs[self.tick_count / kinds.len() % msgs.len()];
        let title = title_prefix.to_string();
        let message = msg.to_string();

        // Track for feed
        self.notifications.borrow_mut().push(NotifEntry {
            title,
            message,
            kind: *kind,
            timestamp: "just now".into(),
        });

        self.total_added += 1;
        self.tick_count += 1;
    }

    fn kind_icon(kind: NotificationKind) -> &'static str {
        match kind {
            NotificationKind::Info => "ℹ",
            NotificationKind::Success => "✓",
            NotificationKind::Warning => "⚠",
            NotificationKind::Error => "✗",
        }
    }

    fn kind_color(kind: NotificationKind, t: &Theme) -> Color {
        match kind {
            NotificationKind::Info => t.info,
            NotificationKind::Success => t.success,
            NotificationKind::Warning => t.warning,
            NotificationKind::Error => t.error,
        }
    }

    fn render_feed(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;

        // Feed header
        draw_text_clipped(
            plane,
            1,
            0,
            " Notifications ",
            SIDEBAR_WIDTH,
            t.fg_on_accent,
            t.primary,
            true,
        );

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Notification entries
        let entries = self.notifications.borrow();
        let notif_h = area.height.saturating_sub(1);
        for (i, entry) in entries.iter().enumerate() {
            let row = i as u16 + 1;
            if row >= notif_h {
                break;
            }

            let is_selected = self.selected_idx == Some(i);
            let is_hovered = !is_selected && i == 0; // simple hover for first

            let bg = if is_selected {
                t.primary
            } else if is_hovered {
                t.hover_bg
            } else {
                t.surface
            };
            let fg = if is_selected { t.fg_on_accent } else { t.fg };
            let icon_color = Self::kind_color(entry.kind, t);

            // Row background
            for x in 0..area.width {
                let idx = (row * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ' ';
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Icon
            let icon = Self::kind_icon(entry.kind);
            let idx = (row * plane.width + 1) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = icon.chars().next().unwrap_or(' ');
                plane.cells[idx].fg = icon_color;
                plane.cells[idx].bg = bg;
            }

            // Title
            let title_x = 4u16;
            draw_text_clipped(
                plane,
                title_x,
                row,
                &entry.title,
                SIDEBAR_WIDTH.saturating_sub(4),
                fg,
                bg,
                is_selected,
            );

            // Timestamp
            let ts_x = SIDEBAR_WIDTH.saturating_sub(entry.timestamp.len() as u16 + 1);
            draw_text_clipped(
                plane,
                ts_x,
                row,
                &entry.timestamp,
                SIDEBAR_WIDTH,
                t.fg_muted,
                bg,
                false,
            );
        }

        // Empty state
        if entries.is_empty() {
            let empty_y = 3;
            draw_text_clipped(
                plane,
                1,
                empty_y,
                "(no notifications)",
                SIDEBAR_WIDTH,
                t.fg_muted,
                t.surface,
                false,
            );
            draw_text_clipped(
                plane,
                1,
                empty_y + 1,
                "Press SPACE to add one",
                SIDEBAR_WIDTH,
                t.fg_muted,
                t.surface,
                false,
            );
        }
    }

    fn render_detail(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let panel_x = SIDEBAR_WIDTH + 1;

        // Detail header
        draw_text_clipped(
            plane,
            panel_x + 1,
            0,
            " Detail ",
            area.width,
            t.fg_on_accent,
            t.primary,
            true,
        );

        // Divider
        for x in panel_x..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Selected notification
        if let Some(idx) = self.selected_idx {
            let entries = self.notifications.borrow();
            if let Some(entry) = entries.get(idx) {
                let kind_color = Self::kind_color(entry.kind, t);

                // Kind + timestamp row
                let icon = Self::kind_icon(entry.kind);
                draw_text(plane, panel_x + 2, 2, icon, kind_color, t.surface, true);
                draw_text_clipped(
                    plane,
                    panel_x + 5,
                    2,
                    entry.title.as_str(),
                    area.width,
                    kind_color,
                    t.surface,
                    true,
                );
                draw_text_clipped(
                    plane,
                    panel_x + 2,
                    3,
                    &format!("Kind: {:?}", entry.kind),
                    area.width,
                    t.fg_muted,
                    t.surface,
                    false,
                );
                draw_text_clipped(
                    plane,
                    panel_x + 2,
                    4,
                    &format!("Time: {}", entry.timestamp),
                    area.width,
                    t.fg_muted,
                    t.surface,
                    false,
                );

                // Divider
                for x in panel_x + 2..area.width.saturating_sub(2) {
                    let idx = (5 * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '─';
                        plane.cells[idx].fg = t.outline;
                    }
                }

                // Message
                draw_text_clipped(
                    plane,
                    panel_x + 2,
                    6,
                    "Message:",
                    area.width,
                    t.secondary,
                    t.surface,
                    true,
                );
                let msg_chars: Vec<char> = entry.message.chars().collect();
                for (i, chunk) in msg_chars.chunks(40).take(8).enumerate() {
                    let line_str: String = chunk.iter().collect();
                    draw_text_clipped(
                        plane,
                        panel_x + 2,
                        7 + i as u16,
                        &line_str,
                        area.width,
                        t.fg,
                        t.surface,
                        false,
                    );
                }

                // Actions
                let action_y = area.height.saturating_sub(4);
                draw_text_clipped(
                    plane,
                    panel_x + 2,
                    action_y,
                    "[D] Dismiss",
                    area.width,
                    t.error,
                    t.surface,
                    false,
                );
                draw_text_clipped(
                    plane,
                    panel_x + 2,
                    action_y + 1,
                    "[C] Clear all",
                    area.width,
                    t.warning,
                    t.surface,
                    false,
                );

                return;
            }
        }

        // No selection state
        let empty_y = 4;
        draw_text_clipped(
            plane,
            panel_x + 2,
            empty_y,
            "No notification selected",
            area.width,
            t.fg_muted,
            t.surface,
            false,
        );
        draw_text_clipped(
            plane,
            panel_x + 2,
            empty_y + 1,
            "Click one from the feed",
            area.width,
            t.fg_muted,
            t.surface,
            false,
        );
        draw_text_clipped(
            plane,
            panel_x + 2,
            empty_y + 2,
            "or press SPACE to add",
            area.width,
            t.fg_muted,
            t.surface,
            false,
        );
    }

    fn render_top_bar(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;

        // Stats
        let active = self.notifications.borrow().len();
        let stats = format!(
            "Active: {} | Added: {} | Dismissed: {} | {}",
            active,
            self.total_added,
            self.total_dismissed,
            if self.auto_running {
                "▶ AUTO"
            } else {
                "○ STOPPED"
            },
        );
        draw_text_clipped(plane, 1, 1, &stats, area.width, t.fg_muted, t.bg, false);

        // Filter pills
        let modes = [
            FilterMode::All,
            FilterMode::Info,
            FilterMode::Success,
            FilterMode::Warning,
            FilterMode::Error,
        ];
        let colors = [t.fg, t.info, t.success, t.warning, t.error];
        let mut cx = area.width.saturating_sub(50);
        for (i, mode) in modes.iter().enumerate() {
            let label = mode.label();
            let is_active = *mode == self.filter;
            let bg = if is_active { colors[i] } else { t.surface };
            let fg = if is_active {
                Color::Rgb(255, 255, 255)
            } else {
                t.fg_muted
            };

            let pill = format!(" {} ", label);
            for (j, ch) in pill.chars().enumerate() {
                let idx = (plane.width + cx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = if is_active {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                    plane.cells[idx].transparent = false;
                }
            }
            cx += pill.len() as u16 + 1;
        }
    }
}

impl Scene for NotificationCenterScene {

    fn on_enter(&mut self) {
        self.show_help = false;
        self.dirty = true;
    }

    fn on_exit(&mut self) {
        self.show_help = false;
    }


    fn scene_id(&self) -> &str {
        "notification_center"
    }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(
            &mut plane,
            2,
            0,
            " Notification Hub ",
            t.primary,
            t.bg,
            true,
        );
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Top bar (stats + filters)
        self.render_top_bar(&mut plane, area);

        // Sidebar divider
        for y in 2..area.height.saturating_sub(1) {
            let idx = (y * plane.width + SIDEBAR_WIDTH) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Left: Notification feed
        let feed_area = Rect::new(0, 2, SIDEBAR_WIDTH, area.height.saturating_sub(3));
        self.render_feed(&mut plane, feed_area);

        // Right: Detail panel
        let detail_area = Rect::new(
            SIDEBAR_WIDTH + 1,
            2,
            area.width.saturating_sub(SIDEBAR_WIDTH + 2),
            area.height.saturating_sub(3),
        );
        self.render_detail(&mut plane, detail_area);

        // Bottom: Action buttons
        let btn_y = area.height.saturating_sub(3);
        for x in 1..area.width.saturating_sub(1) {
            let idx = (btn_y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        let clear_label = "[C] Clear All";
        let auto_label = if self.auto_running {
            "[A] Stop Auto"
        } else {
            "[A] Start Auto"
        };
        let add_label = "[SPACE] Add";
        draw_text(&mut plane, 2, btn_y, clear_label, t.error, t.surface, false);
        draw_text(
            &mut plane,
            20,
            btn_y,
            auto_label,
            if self.auto_running {
                t.warning
            } else {
                t.primary
            },
            t.surface,
            false,
        );
        draw_text(&mut plane, 40, btn_y, add_label, t.fg, t.surface, false);

        // Footer
        let fy = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (fy * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        let nav = " SPACE:add | A:auto | C:clear | F:filter | D:dismiss | ?:help | Esc:back ";
        draw_text(&mut plane, 2, fy, nav, t.fg_muted, t.surface, false);

        if self.show_help {
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Notification Hub — Help",
                &[
                    ("SPACE", "Add notification"),
                    ("A", "Toggle auto-generation"),
                    ("C", "Clear all notifications"),
                    ("F", "Cycle filter priority"),
                    ("D", "Dismiss selected"),
                    ("↑/↓", "Navigate feed"),
                    ("Click", "Select notification"),
                    (back_key, "Back"),
                ],
            );
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_area = Rect::new(0, sb_y, area.width, 1);
        self.status_bar.borrow_mut().set_area(sb_area);
        let sb_plane = self.status_bar.borrow().render(sb_area);
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.add_filtered_notification();
                true
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.auto_running = !self.auto_running;
                true
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => {
                let cleared = self.notifications.borrow().len();
                self.notifications.borrow_mut().clear();
                self.selected_idx = None;
                self.total_dismissed += cleared;
                true
            }
            KeyCode::Char('f') if key.modifiers.is_empty() => {
                self.filter = self.filter.next();
                true
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                if let Some(idx) = self.selected_idx {
                    self.notifications.borrow_mut().remove(idx);
                    self.total_dismissed += 1;
                    self.selected_idx = if self.notifications.borrow().is_empty() {
                        None
                    } else {
                        Some(idx.min(self.notifications.borrow().len().saturating_sub(1)))
                    };
                }
                true
            }
            KeyCode::Up if key.modifiers.is_empty() => {
                if self.selected_idx.is_none() && !self.notifications.borrow().is_empty() {
                    self.selected_idx = Some(0);
                } else if let Some(idx) = self.selected_idx {
                    if idx > 0 {
                        self.selected_idx = Some(idx - 1);
                    }
                }
                true
            }
            KeyCode::Down if key.modifiers.is_empty() => {
                let len = self.notifications.borrow().len();
                if let Some(idx) = self.selected_idx {
                    if idx + 1 < len {
                        self.selected_idx = Some(idx + 1);
                    }
                } else if len > 0 {
                    self.selected_idx = Some(0);
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        // Filter pills
        if row == 1 {
            let modes = [
                FilterMode::All,
                FilterMode::Info,
                FilterMode::Success,
                FilterMode::Warning,
                FilterMode::Error,
            ];
            let mut cx = area.width.saturating_sub(50);
            for mode in &modes {
                let label = mode.label();
                let pill_len = label.len() as u16 + 3;
                if col >= cx
                    && col < cx + pill_len
                    && matches!(kind, MouseEventKind::Down(MouseButton::Left))
                {
                    self.filter = *mode;
                    return true;
                }
                cx += pill_len + 1;
            }
        }

        // Feed clicks (left side)
        if col < SIDEBAR_WIDTH && row >= 2 && row < area.height.saturating_sub(3) {
            let idx = (row - 2) as usize;
            let entries = self.notifications.borrow();
            if idx < entries.len() && matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
                self.selected_idx = Some(idx);
                self.focused_side = 0;
            }
            return true;
        }

        // Action buttons
        if row == area.height.saturating_sub(3)
            && matches!(kind, MouseEventKind::Down(MouseButton::Left))
        {
            if (2..15).contains(&col) {
                // Clear all
                let cleared = self.notifications.borrow().len();
                self.notifications.borrow_mut().clear();
                self.selected_idx = None;
                self.total_dismissed += cleared;
                return true;
            }
            if (20..33).contains(&col) {
                // Toggle auto
                self.auto_running = !self.auto_running;
                return true;
            }
            if (40..52).contains(&col) {
                // Add
                self.add_filtered_notification();
                return true;
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.status_bar.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
