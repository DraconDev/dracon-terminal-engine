//! Live Feed scene — Split Pane + TabBar + StreamingText + Sparkline.
//!
//! Shows a split-pane layout with draggable divider, tab bar switching content,
//! streaming text log, and sparkline metric charts. A realistic multi-panel
//! dashboard layout.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::framework::widgets::{
    Sparkline, SplitPane, StatusBar, StatusSegment, StreamingText, TabBar,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::time::Instant;

pub struct LiveFeedScene {
    theme: Theme,
    keybindings: KeybindingSet,
    split: RefCell<SplitPane>,
    tab_bar: RefCell<TabBar>,
    stream: RefCell<StreamingText>,
    sparkline_cpu: RefCell<Sparkline>,
    sparkline_mem: RefCell<Sparkline>,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    tick: u64,
    cpu_data: Vec<f64>,
    mem_data: Vec<f64>,
    last_feed: Instant,
    // Time-based auto-update state (mutable-from-render via Cell)
    last_auto_tick: std::cell::Cell<Instant>,
    render_dirty: std::cell::Cell<bool>,
    // Live mode toggle (paused by default, but tab switches and Space force updates)
    live_mode: bool,
    // Burst counter for visual feedback
    burst_count: u32,
    // Peak values for stats panel
    peak_cpu: f64,
    peak_mem: f64,
    dirty: bool,
}

impl LiveFeedScene {
    pub fn new(theme: Theme) -> Self {
        let split = SplitPane::new_with_id(WidgetId::new(500), Orientation::Horizontal).ratio(0.6);

        let tab_bar = TabBar::new_with_id(WidgetId::new(501), vec!["Logs", "CPU", "Memory"])
            .with_theme(theme.clone());

        let stream = StreamingText::with_id(WidgetId::new(502))
            .max_lines(500)
            .auto_scroll(true)
            .word_wrap(true)
            .with_theme(theme.clone());

        let cpu_data: Vec<f64> = (0..40)
            .map(|i| 30.0 + 20.0 * (i as f64 / 40.0).sin())
            .collect();
        let mem_data: Vec<f64> = (0..40)
            .map(|i| 45.0 + 10.0 * (i as f64 / 20.0).cos())
            .collect();

        let sparkline_cpu = Sparkline::new(cpu_data.clone())
            .with_theme(theme.clone())
            .with_height(6)
            .with_min_max(true);

        let sparkline_mem = Sparkline::new(mem_data.clone())
            .with_theme(theme.clone())
            .with_height(6)
            .with_min_max(true);

        let status_bar = StatusBar::new(WidgetId::new(503))
            .add_segment(StatusSegment::new(
                "1/2/3: tabs | Space: add log | F1: help | Esc: back",
            ))
            .with_theme(theme.clone());

        let peak_cpu_init = cpu_data.last().copied().unwrap_or(0.0);
        let peak_mem_init = mem_data.last().copied().unwrap_or(0.0);

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            split: RefCell::new(split),
            tab_bar: RefCell::new(tab_bar),
            stream: RefCell::new(stream),
            sparkline_cpu: RefCell::new(sparkline_cpu),
            sparkline_mem: RefCell::new(sparkline_mem),
            status_bar: RefCell::new(status_bar),
            show_help: false,
            tick: 0,
            cpu_data,
            mem_data,
            last_feed: Instant::now(),
            last_auto_tick: std::cell::Cell::new(Instant::now()),
            render_dirty: std::cell::Cell::new(false),
            live_mode: false,
            burst_count: 0,
            peak_cpu: peak_cpu_init,
            peak_mem: peak_mem_init,
            dirty: true,
        }
    }

    fn add_log_entry(&mut self) {
        let messages = [
            "[INFO] Request processed in 12ms",
            "[INFO] Cache hit for /api/users",
            "[WARN] Slow query detected: 450ms",
            "[INFO] Connection pool: 8/10 active",
            "[ERROR] Timeout connecting to db-replica-2",
            "[INFO] Background job completed: cleanup",
            "[DEBUG] Token refreshed for user 42",
            "[INFO] Response 200 OK for GET /api/health",
            "[WARN] Memory usage at 78%",
            "[INFO] Scheduled task started: backup",
        ];
        let idx = (self.tick as usize) % messages.len();
        self.stream.borrow_mut().append(messages[idx]);
        self.tick += 1;
        self.dirty = true;
    }

    fn update_metrics(&mut self) {
        let t = self.tick as f64;
        let cpu = 30.0 + 20.0 * (t * 0.1).sin() + 5.0 * (t * 0.3).sin();
        let mem = 45.0 + 10.0 * (t * 0.05).cos() + 3.0 * (t * 0.2).sin();

        self.cpu_data.push(cpu);
        self.mem_data.push(mem);
        if self.cpu_data.len() > 60 {
            self.cpu_data.remove(0);
        }
        if self.mem_data.len() > 60 {
            self.mem_data.remove(0);
        }

        self.sparkline_cpu
            .borrow_mut()
            .set_data(self.cpu_data.clone());
        self.sparkline_mem
            .borrow_mut()
            .set_data(self.mem_data.clone());
    }
}

impl Scene for LiveFeedScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // ── Tab bar (row 0) ────────────────────────────────────────
        self.tab_bar
            .borrow_mut()
            .set_area(Rect::new(0, 0, area.width, 1));
        let tb_plane = self.tab_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &tb_plane, 0, 0);

        // ── Split pane area (rows 1..height-1) ────────────────────
        let split_area = Rect::new(0, 1, area.width, area.height.saturating_sub(2));
        let (left, right) = self.split.borrow().split(split_area);

        // Divider
        let div_plane = self.split.borrow().render_divider(split_area);
        let div_rect = self.split.borrow().divider_rect(split_area);
        blit_to(
            &mut plane,
            &div_plane,
            div_rect.x as usize,
            div_rect.y as usize,
        );

        // ── Active tab content ─────────────────────────────────────
        let active = self.tab_bar.borrow().active();

        match active {
            0 => {
                // Logs tab — streaming text
                self.stream.borrow_mut().set_area(left);
                let stream_plane = self.stream.borrow().render(left);
                blit_to(&mut plane, &stream_plane, left.x as usize, left.y as usize);

                // Right panel: metrics overview
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 1,
                    "Metrics Overview",
                    t.primary,
                    t.bg,
                    true,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 3,
                    &format!("CPU: {:.1}%", self.cpu_data.last().unwrap_or(&0.0)),
                    t.fg,
                    t.bg,
                    false,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 4,
                    &format!("Memory: {:.1}%", self.mem_data.last().unwrap_or(&0.0)),
                    t.fg,
                    t.bg,
                    false,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 5,
                    &format!("Lines: {}", self.tick),
                    t.fg_muted,
                    t.bg,
                    false,
                );

                // Mini sparklines
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 7,
                    "CPU History",
                    t.primary,
                    t.bg,
                    true,
                );
                self.sparkline_cpu.borrow_mut().set_area(Rect::new(
                    right.x + 1,
                    right.y + 8,
                    right.width.saturating_sub(2),
                    6,
                ));
                let cpu_plane = self.sparkline_cpu.borrow().render(Rect::new(
                    0,
                    0,
                    right.width.saturating_sub(2),
                    6,
                ));
                blit_to(
                    &mut plane,
                    &cpu_plane,
                    (right.x + 1) as usize,
                    (right.y + 8) as usize,
                );

                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 15,
                    "Memory History",
                    t.primary,
                    t.bg,
                    true,
                );
                self.sparkline_mem.borrow_mut().set_area(Rect::new(
                    right.x + 1,
                    right.y + 16,
                    right.width.saturating_sub(2),
                    6,
                ));
                let mem_plane = self.sparkline_mem.borrow().render(Rect::new(
                    0,
                    0,
                    right.width.saturating_sub(2),
                    6,
                ));
                blit_to(
                    &mut plane,
                    &mem_plane,
                    (right.x + 1) as usize,
                    (right.y + 16) as usize,
                );
            }
            1 => {
                // CPU tab — big sparkline
                draw_text(
                    &mut plane,
                    left.x + 1,
                    left.y + 1,
                    "CPU Usage Over Time",
                    t.primary,
                    t.bg,
                    true,
                );
                self.sparkline_cpu.borrow_mut().set_area(Rect::new(
                    left.x + 1,
                    left.y + 3,
                    left.width.saturating_sub(2),
                    left.height.saturating_sub(4),
                ));
                let cpu_plane = self.sparkline_cpu.borrow().render(Rect::new(
                    0,
                    0,
                    left.width.saturating_sub(2),
                    left.height.saturating_sub(4),
                ));
                blit_to(
                    &mut plane,
                    &cpu_plane,
                    (left.x + 1) as usize,
                    (left.y + 3) as usize,
                );

                // Stats on right
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 1,
                    "CPU Stats",
                    t.primary,
                    t.bg,
                    true,
                );
                let avg = self.cpu_data.iter().sum::<f64>() / self.cpu_data.len().max(1) as f64;
                let max = self.cpu_data.iter().cloned().fold(0.0_f64, f64::max);
                let min = self.cpu_data.iter().cloned().fold(f64::MAX, f64::min);
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 3,
                    &format!("Current: {:.1}%", self.cpu_data.last().unwrap_or(&0.0)),
                    t.fg,
                    t.bg,
                    false,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 4,
                    &format!("Average: {:.1}%", avg),
                    t.fg,
                    t.bg,
                    false,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 5,
                    &format!("Max: {:.1}%", max),
                    t.warning,
                    t.bg,
                    false,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 6,
                    &format!("Min: {:.1}%", min),
                    t.success,
                    t.bg,
                    false,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 8,
                    "Press Space to update",
                    t.fg_muted,
                    t.bg,
                    false,
                );
            }
            2 => {
                // Memory tab
                draw_text(
                    &mut plane,
                    left.x + 1,
                    left.y + 1,
                    "Memory Usage Over Time",
                    t.primary,
                    t.bg,
                    true,
                );
                self.sparkline_mem.borrow_mut().set_area(Rect::new(
                    left.x + 1,
                    left.y + 3,
                    left.width.saturating_sub(2),
                    left.height.saturating_sub(4),
                ));
                let mem_plane = self.sparkline_mem.borrow().render(Rect::new(
                    0,
                    0,
                    left.width.saturating_sub(2),
                    left.height.saturating_sub(4),
                ));
                blit_to(
                    &mut plane,
                    &mem_plane,
                    (left.x + 1) as usize,
                    (left.y + 3) as usize,
                );

                // Stats on right
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 1,
                    "Memory Stats",
                    t.primary,
                    t.bg,
                    true,
                );
                let avg = self.mem_data.iter().sum::<f64>() / self.mem_data.len().max(1) as f64;
                let max = self.mem_data.iter().cloned().fold(0.0_f64, f64::max);
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 3,
                    &format!("Current: {:.1}%", self.mem_data.last().unwrap_or(&0.0)),
                    t.fg,
                    t.bg,
                    false,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 4,
                    &format!("Average: {:.1}%", avg),
                    t.fg,
                    t.bg,
                    false,
                );
                draw_text(
                    &mut plane,
                    right.x + 1,
                    right.y + 5,
                    &format!("Max: {:.1}%", max),
                    t.warning,
                    t.bg,
                    false,
                );
            }
            _ => {}
        }

        // ── Status bar ─────────────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        // ── Help overlay ───────────────────────────────────────────
        if self.show_help {
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Live Feed — Help",
                &[
                    ("1/2/3", "Switch tab (Logs/CPU/Memory)"),
                    ("Tab", "Cycle tabs"),
                    ("Space", "Add log entry + update metrics"),
                    ("b", "Burst — emit 5 log entries rapidly"),
                    ("L", "Toggle live auto-tick mode"),
                    ("Left/Right", "Resize split pane"),
                    ("Click tab", "Switch tab"),
                    ("Drag divider", "Resize split pane"),
                    ("F1", "Toggle this help"),
                    ("Esc", "Back"),
                ],
            );
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
            KeyCode::Char('1') => {
                self.tab_bar.borrow_mut().set_active(0);
                self.dirty = true;
                true
            }
            KeyCode::Char('2') => {
                self.tab_bar.borrow_mut().set_active(1);
                self.dirty = true;
                true
            }
            KeyCode::Char('3') => {
                self.tab_bar.borrow_mut().set_active(2);
                self.dirty = true;
                true
            }
            KeyCode::Char(' ') => {
                self.add_log_entry();
                self.update_metrics();
                true
            }
            KeyCode::Char('b') if key.modifiers.is_empty() => {
                // Burst mode: emit 5 log entries in rapid succession to
                // demonstrate the streaming log and sparkline tracking.
                for _ in 0..5 {
                    self.add_log_entry();
                    self.update_metrics();
                }
                self.burst_count += 1;
                self.dirty = true;
                true
            }
            KeyCode::Char('L') if key.modifiers.is_empty() => {
                self.live_mode = !self.live_mode;
                self.last_auto_tick.set(Instant::now());
                self.dirty = true;
                true
            }
            KeyCode::Tab => {
                let active = self.tab_bar.borrow().active();
                self.tab_bar.borrow_mut().set_active((active + 1) % 3);
                self.dirty = true;
                true
            }
            KeyCode::Left => {
                let ratio = self.split.borrow().get_ratio();
                let new_ratio = (ratio - 0.05).max(0.2);
                *self.split.borrow_mut() =
                    SplitPane::new_with_id(WidgetId::new(500), Orientation::Horizontal)
                        .ratio(new_ratio);
                self.dirty = true;
                true
            }
            KeyCode::Right => {
                let ratio = self.split.borrow().get_ratio();
                let new_ratio = (ratio + 0.05).min(0.8);
                *self.split.borrow_mut() =
                    SplitPane::new_with_id(WidgetId::new(500), Orientation::Horizontal)
                        .ratio(new_ratio);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Tab bar clicks
        if row == 0 {
            if let MouseEventKind::Down(_) = kind {
                // Approximate tab detection: each tab ~10 chars
                if col < 10 {
                    self.tab_bar.borrow_mut().set_active(0);
                    self.dirty = true;
                    return true;
                }
                if col < 20 {
                    self.tab_bar.borrow_mut().set_active(1);
                    self.dirty = true;
                    return true;
                }
                if col < 30 {
                    self.tab_bar.borrow_mut().set_active(2);
                    self.dirty = true;
                    return true;
                }
            }
        }

        // Split divider drag
        if let MouseEventKind::Drag(_) = kind {
            let split_area = Rect::new(0, 1, 80, 24); // approximate
            if self
                .split
                .borrow_mut()
                .handle_resize(kind, col, row, split_area)
            {
                self.dirty = true;
                return true;
            }
        }
        if let MouseEventKind::Up(_) = kind {
            // Release any drag
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.split.borrow_mut().on_theme_change(theme);
        self.tab_bar.borrow_mut().on_theme_change(theme);
        self.stream.borrow_mut().on_theme_change(theme);
        self.sparkline_cpu.borrow_mut().on_theme_change(theme);
        self.sparkline_mem.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str {
        "live_feed"
    }
    fn needs_render(&self) -> bool {
        if self.render_dirty.get() {
            self.render_dirty.set(false);
            return true;
        }
        true
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
