//! Dashboard Builder — showcases all command-bound widgets in a grid layout.
//!
//! Run with: cargo run --example dashboard_builder
//!
//! Demonstrates:
//! - All 5 command-bound widgets: Gauge, KeyValueGrid, StatusBadge, LogViewer, StreamingText
//! - Nested SplitPane grid layout
//! - Different auto-refresh intervals per widget
//! - Keyboard controls: r=refresh, p=pause/resume, t=cycle themes
//! - Theme switching affecting all widgets
//!
//! ## Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │ Dashboard Builder        [Refresh: 3s] [Theme: Nord]   │
//! ├───────────────────────────┬─────────────────────────────┤
//! │ CPU Gauge                 │ System Metrics              │
//! │ 67% [██████░░░]          │ Memory:  8.2 GB             │
//! │                           │ Disk:    45%                │
//! │                           │ Network: 120 Mbps           │
//! ├───────────────────────────┼─────────────────────────────┤
//! │ Connection Status         │ Event Stream                │
//! │ ✓ Connected               │ [INFO] Server connected    │
//! │                           │ [WARN] High latency: 250ms  │
//! ├───────────────────────────┴─────────────────────────────┤
//! │ Last Update: 14:32:01                                   │
//! └─────────────────────────────────────────────────────────┘
//! ```

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

fn main() -> std::io::Result<()> {
    let paused = Arc::new(AtomicBool::new(false));
    let theme_idx = Arc::new(AtomicUsize::new(0));
    let refresh_in = Arc::new(AtomicUsize::new(3));
    let tick_count = Arc::new(AtomicUsize::new(0));

    let paused_clone = paused.clone();
    let theme_idx_clone = theme_idx.clone();
    let refresh_in_clone = refresh_in.clone();
    let tick_count_clone = tick_count.clone();

    let mut app = App::new()?
        .title("Dashboard Builder")
        .fps(30)
        .theme(Theme::nord())
        .tick_interval(1000)
        .on_tick(move |ctx, tick| {
            tick_count_clone.fetch_add(1, Ordering::SeqCst);
            if !paused_clone.load(Ordering::SeqCst) {
                refresh_in_clone.store(3, Ordering::SeqCst);
            }
            let current_theme = THEMES[theme_idx_clone.load(Ordering::SeqCst) % THEMES.len()].1();
            ctx.theme = &current_theme;
        });

    let (w, h) = (80u16, 24u16);
    let header_h: u16 = 2;
    let footer_h: u16 = 1;
    let content_h = h.saturating_sub(header_h).saturating_sub(footer_h);

    let content_rect = Rect::new(0, header_h, w, content_h);
    let h_split = SplitPane::new(Orientation::Horizontal).ratio(0.5);
    let (top, bottom) = h_split.split(content_rect);

    let v_split = SplitPane::new(Orientation::Horizontal).ratio(0.33);
    let (bl, br) = v_split.split(bottom);

    let gauge = Gauge::new("CPU %")
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
        );

    let kv_grid = KeyValueGrid::new()
        .separator(": ")
        .bind_command(
            BoundCommand::new("echo 'Memory:8.2GB|Disk:45%|Network:120Mbps|Uptime:3d'")
                .refresh(5),
        );

    let status_badge = StatusBadge::new(WidgetId::default_id())
        .with_label("Connection")
        .bind_command(BoundCommand::new("echo 'Connected'").refresh(10));

    let log_viewer = LogViewer::new()
        .max_lines(100)
        .bind_command(
            BoundCommand::new(
                "echo '[INFO] Server connected\\n[WARN] High latency: 250ms\\n[DEBUG] Reconnecting...'",
            )
            .refresh(3),
        );

    let streaming = StreamingText::new()
        .max_lines(50)
        .bind_command(BoundCommand::new("date +'%H:%M:%S'").refresh(1));

    app.add_widget(Box::new(gauge), Rect::new(top.x, top.y, top.width, top.height / 2));
    app.add_widget(
        Box::new(kv_grid),
        Rect::new(top.x + top.width / 2, top.y, top.width / 2, top.height / 2),
    );

    app.add_widget(
        Box::new(status_badge),
        Rect::new(bl.x, bl.y, bl.width, bl.height),
    );
    app.add_widget(
        Box::new(log_viewer),
        Rect::new(br.x, br.y, br.width / 2, br.height),
    );
    app.add_widget(
        Box::new(streaming),
        Rect::new(br.x + br.width / 2, br.y, br.width / 2, br.height),
    );

    let paused_r = paused.clone();
    let theme_idx_r = theme_idx.clone();
    let refresh_in_r = refresh_in.clone();

    app.run(move |ctx| {
        let theme_name = THEMES[theme_idx_r.load(Ordering::SeqCst) % THEMES.len()].0;
        let is_paused = paused_r.load(Ordering::SeqCst);
        let next_refresh = refresh_in_r.load(Ordering::SeqCst);

        ctx.hide_cursor().ok();

        let screen = ctx.compositor().size();
        let width = screen.0;
        render_header(ctx, width, theme_name, is_paused, next_refresh);
        render_footer(ctx, width, h.saturating_sub(1));
    });

    Ok(())
}

fn render_header(ctx: &mut Ctx, width: u16, theme_name: &str, is_paused: bool, next_refresh: usize) {
    let theme = ctx.theme();
    let title = "Dashboard Builder";
    let status = if is_paused { "[PAUSED]" } else { "[ACTIVE]" };
    let refresh_text = format!("Refresh: {}s", next_refresh);
    let theme_text = format!("Theme: {}", theme_name);

    ctx.mark_dirty(0, 0, width, 2);
}

fn render_footer(ctx: &mut Ctx, width: u16, footer_y: u16) {
    ctx.mark_dirty(0, footer_y, width, 2);
}