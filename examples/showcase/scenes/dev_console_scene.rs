//! Dev Console scene — LogViewer + EventLogger + Label + Divider + WidgetInspector.
//!
//! Shows a log viewer with filtered levels, an event logger tracking UI actions,
//! styled labels, dividers for visual separation, and a widget inspector toggle.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Divider, EventLogger, Label, LogViewer, LogLevel, StatusBar, StatusSegment,
    WidgetInspector, WidgetNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

pub struct DevConsoleScene {
    theme: Theme,
    keybindings: KeybindingSet,
    log_viewer: RefCell<LogViewer>,
    event_logger: RefCell<EventLogger>,
    inspector: RefCell<WidgetInspector>,
    filter_level: LogLevel,
    show_inspector: bool,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    tick: u64,
    dirty: bool,
}

impl DevConsoleScene {
    pub fn new(theme: Theme) -> Self {
        let log_viewer = LogViewer::with_id(WidgetId::new(700))
            .max_lines(500)
            .auto_scroll(true)
            .with_scroll_indicator(true)
            .with_theme(theme.clone());

        let event_logger = EventLogger::new(WidgetId::new(701))
            .with_max_events(50)
            .with_theme(theme.clone());

        let mut inspector = WidgetInspector::new(WidgetId::new(702))
            .with_theme(theme.clone());
        inspector.set_hierarchy(vec![
            WidgetNode::new(WidgetId::new(1), "App"),
            WidgetNode::new(WidgetId::new(2), "  LogViewer"),
            WidgetNode::new(WidgetId::new(3), "  EventLogger"),
            WidgetNode::new(WidgetId::new(4), "  StatusBar"),
            WidgetNode::new(WidgetId::new(5), "  Inspector"),
        ]);

        let status_bar = StatusBar::new(WidgetId::new(703))
            .add_segment(StatusSegment::new(
                "Space: log | 1-5: filter | I: inspector | F1: help | Esc: back",
            ))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            log_viewer: RefCell::new(log_viewer),
            event_logger: RefCell::new(event_logger),
            inspector: RefCell::new(inspector),
            filter_level: LogLevel::Info,
            show_inspector: false,
            status_bar: RefCell::new(status_bar),
            show_help: false,
            tick: 0,
            dirty: true,
        }
    }

    fn add_log(&mut self) {
        let messages: &[(LogLevel, &str)] = &[
            (LogLevel::Debug, "Connection pool initialized with 8 workers"),
            (LogLevel::Info, "Request processed: GET /api/health (12ms)"),
            (LogLevel::Info, "Cache hit: /static/app.js"),
            (LogLevel::Warn, "Slow query detected: SELECT * FROM logs (450ms)"),
            (LogLevel::Error, "Connection refused: db-replica-2:5432"),
            (LogLevel::Debug, "Token refreshed for session abc123"),
            (LogLevel::Info, "Background job completed: cleanup"),
            (LogLevel::Warn, "Memory usage at 78% — approaching limit"),
            (LogLevel::Info, "Scheduled task started: backup"),
            (LogLevel::Error, "Failed to send notification: timeout"),
            (LogLevel::Fatal, "Unrecoverable error: disk full"),
        ];
        let idx = (self.tick as usize) % messages.len();
        let (level, msg) = messages[idx];
        let prefix = Self::level_prefix(level);
        self.log_viewer
            .borrow_mut()
            .append_line(&format!("[{}] {}", prefix, msg));
        self.event_logger.borrow_mut().log("just now", msg);
        self.tick += 1;
        self.dirty = true;
    }

    fn set_filter(&mut self, level: LogLevel) {
        self.filter_level = level;
        let pattern = match level {
            LogLevel::Debug => "",
            LogLevel::Info => "INF|WRN|ERR|FTL",
            LogLevel::Warn => "WRN|ERR|FTL",
            LogLevel::Error => "ERR|FTL",
            LogLevel::Fatal => "FTL",
        };
        let new_lv = std::mem::replace(
            &mut *self.log_viewer.borrow_mut(),
            LogViewer::with_id(WidgetId::new(700))
                .max_lines(500)
                .auto_scroll(true)
                .with_scroll_indicator(true)
                .filter(pattern)
                .with_theme(self.theme.clone()),
        );
        // Preserve existing lines from old viewer
        let old_lines: Vec<_> = new_lv.lines.iter().map(|l| l.raw.clone()).collect();
        for line in &old_lines {
            self.log_viewer.borrow_mut().append_line(line);
        }
        self.dirty = true;
    }

    fn level_prefix(level: LogLevel) -> &'static str {
        match level {
            LogLevel::Debug => "DBG",
            LogLevel::Info => "INF",
            LogLevel::Warn => "WRN",
            LogLevel::Error => "ERR",
            LogLevel::Fatal => "FTL",
        }
    }
}

impl Scene for DevConsoleScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // ── Title with Label widget ────────────────────────────────
        let title = Label::new("Dev Console")
            .with_style(Styles::BOLD)
            .with_theme(t.clone());
        let title_plane = title.render(Rect::new(0, 0, 14, 1));
        blit_to(&mut plane, &title_plane, 1, 0);

        draw_text(
            &mut plane,
            16,
            0,
            "— LogViewer + EventLogger + Label",
            t.fg_muted,
            t.bg,
            false,
        );

        // ── Filter bar (row 1) ─────────────────────────────────────
        let filter_names = ["ALL", "DBG", "INFO", "WARN", "ERR"];
        let filter_levels = [
            LogLevel::Debug,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];
        let mut fx = 1u16;
        for (i, name) in filter_names.iter().enumerate() {
            let is_active = filter_levels[i] == self.filter_level;
            let fg = if is_active { t.primary } else { t.fg_muted };
            let bg = if is_active { t.selection_bg } else { t.bg };
            let label = format!("[{}]", name);
            for (j, ch) in label.chars().enumerate() {
                let idx = (1 * area.width as usize) + (fx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                    if is_active {
                        plane.cells[idx].style = Styles::BOLD;
                    }
                }
            }
            fx += label.len() as u16 + 1;
        }

        // ── Divider (row 2) ────────────────────────────────────────
        let divider = Divider::new()
            .with_label("Logs")
            .with_theme(t.clone());
        let div_plane = divider.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div_plane, 0, 2);

        // ── Log viewer (rows 3..height-8) ──────────────────────────
        let lv_h = area.height.saturating_sub(10);
        let lv_area = Rect::new(0, 3, area.width, lv_h);
        self.log_viewer.borrow_mut().set_area(lv_area);
        let lv_plane = self.log_viewer.borrow().render(lv_area);
        blit_to(&mut plane, &lv_plane, 0, 3);

        // ── Divider (before event log) ──────────────────────────────
        let div2 = Divider::new()
            .with_label("Events")
            .with_theme(t.clone());
        let div2_y = 3 + lv_h;
        let div2_plane = div2.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div2_plane, 0, div2_y as usize);

        // ── Event logger (rows below divider) ──────────────────────
        let el_y = div2_y + 1;
        let el_h = area.height.saturating_sub(div2_y + 3);
        if self.show_inspector {
            let half_w = area.width / 2;
            let el_area = Rect::new(0, el_y, half_w, el_h);
            self.event_logger.borrow_mut().set_area(el_area);
            let el_plane = self.event_logger.borrow().render(el_area);
            blit_to(&mut plane, &el_plane, 0, el_y as usize);

            let ins_area = Rect::new(
                half_w + 1,
                el_y,
                area.width.saturating_sub(half_w + 1),
                el_h,
            );
            let ins_plane = self.inspector.borrow().render(ins_area);
            blit_to(
                &mut plane,
                &ins_plane,
                (half_w + 1) as usize,
                el_y as usize,
            );
        } else {
            let el_area = Rect::new(0, el_y, area.width, el_h);
            self.event_logger.borrow_mut().set_area(el_area);
            let el_plane = self.event_logger.borrow().render(el_area);
            blit_to(&mut plane, &el_plane, 0, el_y as usize);
        }

        // ── Status bar ─────────────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            render_help_overlay(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key)
                || self.keybindings.matches(actions::BACK, &key)
            {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        match key.code {
            KeyCode::Char(' ') => {
                self.add_log();
                true
            }
            KeyCode::Char('1') if key.modifiers.is_empty() => {
                self.set_filter(LogLevel::Debug);
                true
            }
            KeyCode::Char('2') if key.modifiers.is_empty() => {
                self.set_filter(LogLevel::Info);
                true
            }
            KeyCode::Char('3') if key.modifiers.is_empty() => {
                self.set_filter(LogLevel::Warn);
                true
            }
            KeyCode::Char('4') if key.modifiers.is_empty() => {
                self.set_filter(LogLevel::Error);
                true
            }
            KeyCode::Char('5') if key.modifiers.is_empty() => {
                self.set_filter(LogLevel::Fatal);
                true
            }
            KeyCode::Char('i') if key.modifiers.is_empty() => {
                self.show_inspector = !self.show_inspector;
                self.dirty = true;
                true
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => {
                self.log_viewer.borrow_mut().clear();
                self.event_logger.borrow_mut().clear();
                self.dirty = true;
                true
            }
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End => {
                self.log_viewer.borrow_mut().handle_key(key);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Filter bar clicks (row 1)
        if row == 1 {
            if let MouseEventKind::Down(_) = kind {
                let filter_starts = [1u16, 6, 13, 19, 25];
                let filter_levels = [
                    LogLevel::Debug,
                    LogLevel::Debug,
                    LogLevel::Info,
                    LogLevel::Warn,
                    LogLevel::Error,
                ];
                for (i, &start) in filter_starts.iter().enumerate() {
                    if col >= start && col < start + 5 {
                        self.set_filter(filter_levels[i]);
                        return true;
                    }
                }
            }
        }

        // Log viewer scroll
        if row >= 3 {
            self.log_viewer
                .borrow_mut()
                .handle_mouse(kind, col, row.saturating_sub(3));
            self.dirty = true;
            return true;
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.log_viewer.borrow_mut().on_theme_change(theme);
        self.event_logger.borrow_mut().on_theme_change(theme);
        self.inspector.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str {
        "dev_console"
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

fn render_help_overlay(plane: &mut Plane, area: Rect, t: &Theme) {
    let hw = 46u16.min(area.width.saturating_sub(4));
    let hh = 16u16.min(area.height.saturating_sub(4));
    let hx = (area.width.saturating_sub(hw)) / 2;
    let hy = (area.height.saturating_sub(hh)) / 2;

    for y in hy..hy + hh {
        for x in hx..hx + hw {
            let idx = (y as usize) * area.width as usize + x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }
    let corners = [
        ('╭', hx, hy),
        ('╮', hx + hw - 1, hy),
        ('╰', hx, hy + hh - 1),
        ('╯', hx + hw - 1, hy + hh - 1),
    ];
    for (ch, cx, cy) in corners {
        let idx = (cy as usize) * area.width as usize + cx as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = t.outline;
        }
    }
    for x in hx + 1..hx + hw - 1 {
        let ti = (hy as usize) * area.width as usize + x as usize;
        let bi = ((hy + hh - 1) as usize) * area.width as usize + x as usize;
        if ti < plane.cells.len() {
            plane.cells[ti].char = '─';
            plane.cells[ti].fg = t.outline;
        }
        if bi < plane.cells.len() {
            plane.cells[bi].char = '─';
            plane.cells[bi].fg = t.outline;
        }
    }
    for y in hy + 1..hy + hh - 1 {
        let li = (y as usize) * area.width as usize + hx as usize;
        let ri = (y as usize) * area.width as usize + (hx + hw - 1) as usize;
        if li < plane.cells.len() {
            plane.cells[li].char = '│';
            plane.cells[li].fg = t.outline;
        }
        if ri < plane.cells.len() {
            plane.cells[ri].char = '│';
            plane.cells[ri].fg = t.outline;
        }
    }
    let title = "Dev Console — Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    for (i, c) in title.chars().enumerate() {
        let idx = ((hy + 1) as usize) * area.width as usize + (tx + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = t.primary;
            plane.cells[idx].style = Styles::BOLD;
        }
    }
    let shortcuts = [
        ("Space", "Add log entry"),
        ("1-5", "Filter: ALL/DBG/INFO/WARN/ERR"),
        ("I", "Toggle widget inspector"),
        ("C", "Clear logs + events"),
        ("Up/Dn", "Scroll log viewer"),
        ("PgUp/Dn", "Page scroll"),
        ("Click filter", "Set filter level"),
        ("F1", "Toggle this help"),
        ("Esc", "Back to showcase"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let y = hy + 3 + i as u16;
        for (j, c) in key.chars().enumerate() {
            let idx = (y as usize) * area.width as usize + (hx + 2 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
            }
        }
        for (j, c) in desc.chars().enumerate() {
            let idx = (y as usize) * area.width as usize + (hx + 14 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg;
            }
        }
    }
}
