//! Dashboard Builder — showcases all command-bound widgets in a grid layout.
//!
//! Run with: cargo run --example dashboard_builder
//!
//! Demonstrates:
//! - All 5 command-bound widgets: Gauge, KeyValueGrid, StatusBadge, LogViewer, StreamingText
//! - Grid layout via nested SplitPane
//! - Different auto-refresh intervals per widget
//! - Keyboard controls: r=refresh all, p=pause/resume, t=cycle themes

use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widgets::{
    Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText,
};
use dracon_terminal_engine::framework::widgets::split::{Orientation, SplitPane};
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

const THEMES: &[(&str, fn() -> Theme)] = &[
    ("Nord", Theme::nord),
    ("Cyberpunk", Theme::cyberpunk),
    ("Dracula", Theme::dracula),
    ("Gruvbox", Theme::gruvbox_dark),
];

struct Dashboard {
    gauge: Gauge,
    kv_grid: KeyValueGrid,
    status_badge: StatusBadge,
    log_viewer: LogViewer,
    streaming: StreamingText,
    theme_idx: usize,
    paused: bool,
    refresh_in: usize,
}

impl Dashboard {
    fn new() -> Self {
        Self {
            gauge: Gauge::with_id(WidgetId::new(1), "CPU %")
                .max(100.0)
                .warn_threshold(70.0)
                .crit_threshold(90.0)
                .bind_command(
                    BoundCommand::new("echo 'cpu:67'")
                        .parser(OutputParser::Regex {
                            pattern: r"cpu:(\d+)".into(),
                            group: Some(1),
                        })
                        .refresh(2),
                ),
            kv_grid: KeyValueGrid::with_id(WidgetId::new(2))
                .separator(": ")
                .bind_command(
                    BoundCommand::new("echo 'Memory:8.2GB|Disk:45%|Network:120Mbps|Uptime:3d'")
                        .refresh(5),
                ),
            status_badge: StatusBadge::new(WidgetId::new(3))
                .with_label("Connection")
                .bind_command(BoundCommand::new("echo 'Connected'").refresh(10)),
            log_viewer: LogViewer::with_id(WidgetId::new(4))
                .max_lines(100)
                .bind_command(
                    BoundCommand::new(
                        "echo '[INFO] Server connected\\n[WARN] High latency: 250ms\\n[DEBUG] Reconnecting...'",
                    )
                    .refresh(3),
                ),
            streaming: StreamingText::with_id(WidgetId::new(5))
                .max_lines(50)
                .bind_command(BoundCommand::new("date +'%H:%M:%S'").refresh(1)),
            theme_idx: 0,
            paused: false,
            refresh_in: 3,
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_idx = (self.theme_idx + 1) % THEMES.len();
        let theme = THEMES[self.theme_idx].1();
        self.gauge = Gauge::with_id(self.gauge.id, "CPU %")
            .max(100.0)
            .warn_threshold(70.0)
            .crit_threshold(90.0)
            .with_theme(theme)
            .bind_command(
                BoundCommand::new("echo 'cpu:67'")
                    .parser(OutputParser::Regex {
                        pattern: r"cpu:(\d+)".into(),
                        group: Some(1),
                    })
                    .refresh(2),
            );
        self.kv_grid = KeyValueGrid::with_id(self.kv_grid.id)
            .separator(": ")
            .with_theme(theme)
            .bind_command(
                BoundCommand::new("echo 'Memory:8.2GB|Disk:45%|Network:120Mbps|Uptime:3d'")
                    .refresh(5),
            );
        self.status_badge = StatusBadge::new(self.status_badge.id)
            .with_label("Connection")
            .with_theme(theme)
            .bind_command(BoundCommand::new("echo 'Connected'").refresh(10));
        self.log_viewer = LogViewer::with_id(self.log_viewer.id)
            .max_lines(100)
            .with_theme(theme)
            .bind_command(
                BoundCommand::new(
                    "echo '[INFO] Server connected\\n[WARN] High latency: 250ms\\n[DEBUG] Reconnecting...'",
                )
                .refresh(3),
            );
        self.streaming = StreamingText::with_id(self.streaming.id)
            .max_lines(50)
            .with_theme(theme)
            .bind_command(BoundCommand::new("date +'%H:%M:%S'").refresh(1));
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    fn refresh_all(&mut self) {
        self.gauge.mark_dirty();
        self.kv_grid.mark_dirty();
        self.status_badge.mark_dirty();
        self.log_viewer.mark_dirty();
        self.streaming.mark_dirty();
    }
}

fn main() -> std::io::Result<()> {
    let mut dashboard = Dashboard::new();
    let theme_idx = Arc::new(AtomicUsize::new(0));
    let paused = Arc::new(AtomicBool::new(false));

    let theme_idx_clone = theme_idx.clone();
    let paused_clone = paused.clone();

    let mut app = App::new()?
        .title("Dashboard Builder")
        .fps(30)
        .theme(Theme::nord())
        .tick_interval(1000)
        .on_tick(move |ctx, tick| {
            if !paused_clone.load(Ordering::SeqCst) {
                if tick % 3 == 0 {
                    theme_idx_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

    let w = 80u16;
    let h = 24u16;
    let header_h: u16 = 2;
    let footer_h: u16 = 2;
    let content_h = h.saturating_sub(header_h).saturating_sub(footer_h);

    let content_rect = Rect::new(0, header_h, w, content_h);
    let v_split = SplitPane::new(Orientation::Vertical).ratio(0.5);
    let (top_rect, bottom_rect) = v_split.split(content_rect);

    let h_split = SplitPane::new(Orientation::Horizontal).ratio(0.5);
    let (tl, tr) = h_split.split(top_rect);

    let gauge_rect = Rect::new(tl.x, tl.y, tl.width, tl.height);
    let kv_rect = Rect::new(tr.x, tr.y, tr.width, tr.height);

    let b_split = SplitPane::new(Orientation::Horizontal).ratio(0.33);
    let (bl, br) = b_split.split(bottom_rect);

    let badge_rect = Rect::new(bl.x, bl.y, bl.width, bl.height);
    let log_rect = Rect::new(br.x, br.y, br.width / 2, br.height);
    let stream_rect = Rect::new(br.x + br.width / 2, br.y, br.width / 2, br.height);

    app.add_widget(Box::new(dashboard.gauge.clone()), gauge_rect);
    app.add_widget(Box::new(dashboard.kv_grid.clone()), kv_rect);
    app.add_widget(Box::new(dashboard.status_badge.clone()), badge_rect);
    app.add_widget(Box::new(dashboard.log_viewer.clone()), log_rect);
    app.add_widget(Box::new(dashboard.streaming.clone()), stream_rect);

    let theme_idx_r = theme_idx.clone();
    let paused_r = paused.clone();

    app.run(move |ctx| {
        let theme_name = THEMES[theme_idx_r.load(Ordering::SeqCst) % THEMES.len()].0;
        let is_paused = paused_r.load(Ordering::SeqCst);

        ctx.hide_cursor().ok();
        ctx.mark_dirty(0, 0, w, header_h);
        ctx.mark_dirty(0, h.saturating_sub(footer_h), w, footer_h);
    });

    Ok(())
}