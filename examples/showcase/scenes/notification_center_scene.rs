//! Embedded NotificationCenter scene for the showcase.
//!
//! Demonstrates the NotificationCenter widget with:
//!   - Auto-generation timer
//!   - Clear-all button
//!   - Priority filtering
//!   - Unread badge counter

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::{NotificationCenter, NotificationKind};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

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

pub struct NotificationCenterScene {
    notifications: NotificationCenter,
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
}

impl NotificationCenterScene {
    pub fn new(theme: Theme) -> Self {
        let mut nc = NotificationCenter::new(theme.clone());
        nc.info("Welcome", "NotificationCenter demo");
        nc.success("Ready", "All systems operational");
        Self {
            notifications: nc,
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
        }
    }

    fn add_filtered_notification(&mut self) {
        let kinds = [
            (NotificationKind::Info, "Info", vec!["New message received", "Update available", "Sync complete", "User online"]),
            (NotificationKind::Success, "Done", vec!["File saved", "Build succeeded", "Upload complete", "Tests passed"]),
            (NotificationKind::Warning, "Warning", vec!["Disk space low", "Rate limit approaching", "Certificate expiring", "Memory high"]),
            (NotificationKind::Error, "Error", vec!["Connection failed", "Permission denied", "Timeout exceeded", "Disk full"]),
        ];

        let (kind, title_prefix, msgs) = &kinds[self.tick_count % kinds.len()];

        // Respect filter
        match self.filter {
            FilterMode::All => {},
            FilterMode::Info if *kind != NotificationKind::Info => { self.tick_count += 1; return; },
            FilterMode::Success if *kind != NotificationKind::Success => { self.tick_count += 1; return; },
            FilterMode::Warning if *kind != NotificationKind::Warning => { self.tick_count += 1; return; },
            FilterMode::Error if *kind != NotificationKind::Error => { self.tick_count += 1; return; },
            _ => {},
        }

        let msg = msgs[self.tick_count / kinds.len() % msgs.len()];
        self.notifications.notify(title_prefix, msg, *kind);
        self.total_added += 1;
        self.tick_count += 1;
    }

    fn render_filter_tabs(&self, plane: &mut Plane, x: u16, y: u16) {
        let t = &self.theme;
        let modes = [FilterMode::All, FilterMode::Info, FilterMode::Success, FilterMode::Warning, FilterMode::Error];
        let colors = [t.fg, t.info, t.success, t.warning, t.error];
        let mut cx = x;
        for (i, mode) in modes.iter().enumerate() {
            let label = mode.label();
            let is_active = *mode == self.filter;
            let bg = if is_active { colors[i] } else { t.surface };
            let fg = if is_active { Color::Rgb(255, 255, 255) } else { t.fg_muted };

            // Draw tab
            let tab = format!(" {} ", label);
            for (j, ch) in tab.chars().enumerate() {
                let idx = (y * plane.width + cx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = if is_active { Styles::BOLD } else { Styles::empty() };
                    plane.cells[idx].transparent = false;
                }
            }
            cx += tab.len() as u16;
        }
    }
}

impl Scene for NotificationCenterScene {
    fn scene_id(&self) -> &str { "notification_center" }

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
        draw_text(&mut plane, 2, 0, " NotificationCenter ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // ── Filter Tabs ───────────────────────────────────────────────────
        draw_text(&mut plane, 2, 2, "Filter:", t.fg_muted, t.bg, false);
        self.render_filter_tabs(&mut plane, 10, 2);

        // ── Stats Bar ─────────────────────────────────────────────────────
        let stats_y = 3;
        let active_count = self.notifications.len();
        let stats = format!(
            "Active:{} Added:{} Dismissed:{} {}",
            active_count, self.total_added, self.total_dismissed,
            if self.auto_running { "▶AUTO" } else { "" },
        );
        draw_text(&mut plane, 2, stats_y, &stats, t.fg_muted, t.bg, false);

        // ── Action Buttons ────────────────────────────────────────────────
        let btn_y = stats_y + 1;
        // Clear-all button
        let clear_label = " [C] Clear All ";
        for (j, ch) in clear_label.chars().enumerate() {
            let idx = (btn_y * plane.width + 2 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = t.error;
                plane.cells[idx].transparent = false;
            }
        }
        // Auto toggle button
        let auto_label = if self.auto_running { " [A] Stop Auto " } else { " [A] Start Auto " };
        for (j, ch) in auto_label.chars().enumerate() {
            let idx = (btn_y * plane.width + 18 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = if self.auto_running { t.warning } else { t.primary };
                plane.cells[idx].transparent = false;
            }
        }

        // ── Notification Area ────────────────────────────────────────────
        let notif_area = Rect::new(area.x, area.y + 6, area.width, area.height.saturating_sub(8));
        let notif_plane = self.notifications.render(notif_area);
        blit_to(&mut plane, &notif_plane, notif_area.x as usize, notif_area.y as usize);

        // ── Footer ────────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("?");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " SPACE:add | A:auto | C:clear | F:filter | {}:help | {}:back ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        if self.show_help {
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(&mut plane, area, &self.theme, "Notification Center — Help", &[
                ("SPACE", "Add notification"),
                ("A", "Toggle auto-generation"),
                ("C", "Clear all notifications"),
                ("F", "Cycle filter priority"),
                ("Click tab", "Filter by priority"),
                ("Click notif", "Dismiss notification"),
                (back_key, "Back to showcase"),
            ]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.add_filtered_notification();
                self.dirty = true;
                true
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.auto_running = !self.auto_running;
                self.dirty = true;
                true
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => {
                let cleared = self.notifications.len();
                self.notifications.clear_all();
                self.total_dismissed += cleared;
                self.dirty = true;
                true
            }
            KeyCode::Char('f') if key.modifiers.is_empty() => {
                self.filter = self.filter.next();
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let _area = self.area.get();

        // Clear-all button click
        if row == 4 && (2..16).contains(&col)
            && matches!(kind, MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)) {
                let cleared = self.notifications.len();
                self.notifications.clear_all();
                self.total_dismissed += cleared;
                self.dirty = true;
                return true;
            }

        // Auto toggle button click
        if row == 4 && (18..32).contains(&col)
            && matches!(kind, MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)) {
                self.auto_running = !self.auto_running;
                self.dirty = true;
                return true;
            }

        // Filter tab clicks
        if row == 2 {
            let modes = [FilterMode::All, FilterMode::Info, FilterMode::Success, FilterMode::Warning, FilterMode::Error];
            let mut cx = 10u16;
            for mode in &modes {
                let label = mode.label();
                let tab_w = label.len() as u16 + 2;
                if col >= cx && col < cx + tab_w
                    && matches!(kind, MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)) {
                        self.filter = *mode;
                        self.dirty = true;
                        return true;
                    }
                cx += tab_w;
            }
        }

        // Notification area clicks
        if row >= 6
            && self.notifications.handle_mouse(kind, col, row) {
                self.dirty = true;
                return true;
            }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.notifications.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}


