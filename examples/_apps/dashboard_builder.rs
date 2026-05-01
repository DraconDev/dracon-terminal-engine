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

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widgets::{
    Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText,
};
use dracon_terminal_engine::framework::widgets::split::{Orientation, SplitPane};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use std::cell::RefCell;
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
    let refresh_version = Arc::new(AtomicUsize::new(0));

    let paused_clone = paused.clone();
    let theme_idx_clone = theme_idx.clone();
    let refresh_in_clone = refresh_in.clone();
    let tick_count_clone = tick_count.clone();
    let refresh_version_clone = refresh_version.clone();

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
            if tick % 3 == 0 && !paused_clone.load(Ordering::SeqCst) {
                refresh_version_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

    let (w, h) = (80, 24);
    let header_h = 2u16;
    let footer_h = 1u16;
    let content_h = h.saturating_sub(header_h + footer_h);

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

    let streaming = StreamingText::new("Last Update: ")
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
    let refresh_version_r = refresh_version.clone();

    app.run(move |ctx| {
        let theme_name = THEMES[theme_idx_r.load(Ordering::SeqCst) % THEMES.len()].0;
        let is_paused = paused_r.load(Ordering::SeqCst);
        let next_refresh = refresh_in_r.load(Ordering::SeqCst);

        ctx.hide_cursor().ok();

        let (screen_w, screen_h) = ctx.compositor().size();
        render_header(ctx, screen_w, theme_name, is_paused, next_refresh);
        render_footer(ctx, screen_w, screen_h.saturating_sub(1));
    });

    Ok(())
}

fn render_header(ctx: &mut Ctx, width: u16, theme_name: &str, is_paused: bool, next_refresh: usize) {
    let theme = ctx.theme();
    let title = "Dashboard Builder";
    let status = if is_paused { "[PAUSED]" } else { "[ACTIVE]" };
    let refresh_text = format!("Refresh: {}s", next_refresh);
    let theme_text = format!("Theme: {}", theme_name);

    let title_len = title.len();
    let status_len = status.len();
    let refresh_len = refresh_text.len();
    let theme_len = theme_text.len();

    let available = width as usize;
    let right_section = status_len + 1 + refresh_len + 1 + theme_len;
    let left_end = available.saturating_sub(right_section);

    for (i, c) in title.chars().enumerate().take(left_end) {
        let idx = i;
        if idx < ctx.compositor().size().0 as usize * 2 {
            let cell_idx = idx;
            if let Some(mut plane) = ctx.compositor().planes.first_mut() {
                if cell_idx < plane.cells.len() {
                    plane.cells[cell_idx] = Cell {
                        char: c,
                        fg: theme.accent,
                        bg: theme.bg,
                        style: Styles::BOLD,
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }
    }

    let mut offset = left_end + 1;
    for c in status.chars() {
        if offset < available {
            let cell_idx = offset;
            if let Some(mut plane) = ctx.compositor().planes.first_mut() {
                if cell_idx < plane.cells.len() {
                    plane.cells[cell_idx] = Cell {
                        char: c,
                        fg: if is_paused { theme.warning_fg } else { theme.success_fg },
                        bg: theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }
        offset += 1;
    }
    offset += 1;

    for c in refresh_text.chars() {
        if offset < available {
            let cell_idx = offset;
            if let Some(mut plane) = ctx.compositor().planes.first_mut() {
                if cell_idx < plane.cells.len() {
                    plane.cells[cell_idx] = Cell {
                        char: c,
                        fg: theme.inactive_fg,
                        bg: theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }
        offset += 1;
    }
    offset += 1;

    for c in theme_text.chars() {
        if offset < available {
            let cell_idx = offset;
            if let Some(mut plane) = ctx.compositor().planes.first_mut() {
                if cell_idx < plane.cells.len() {
                    plane.cells[cell_idx] = Cell {
                        char: c,
                        fg: theme.fg,
                        bg: theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }
        offset += 1;
    }

    if let Some(mut plane) = ctx.compositor().planes.first_mut() {
        let separator_idx = (1 * plane.width as usize).min(plane.cells.len().saturating_sub(1));
        plane.cells[separator_idx] = Cell {
            char: '─',
            fg: theme.border,
            bg: theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        for x in 1..width as usize {
            let idx = (1 * plane.width as usize + x).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx] = Cell {
                char: '─',
                fg: theme.border,
                bg: theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
    }
    ctx.mark_dirty(0, 0, width, 2);
}

fn render_footer(ctx: &mut Ctx, width: u16, footer_y: u16) {
    let theme = ctx.theme();
    let controls = "[r] Refresh  [p] Pause  [t] Theme";
    let offset = 1;

    if let Some(mut plane) = ctx.compositor().planes.first_mut() {
        let separator_idx = (footer_y as usize * plane.width as usize).min(plane.cells.len().saturating_sub(1));
        plane.cells[separator_idx] = Cell {
            char: '─',
            fg: theme.border,
            bg: theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        for x in 1..width as usize {
            let idx = (footer_y as usize * plane.width as usize + x).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx] = Cell {
                char: '─',
                fg: theme.border,
                bg: theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }

        for (i, c) in controls.chars().enumerate().take(width as usize - offset) {
            let idx = ((footer_y + 1) as usize * plane.width as usize + offset + i)
                .min(plane.cells.len().saturating_sub(1));
            plane.cells[idx] = Cell {
                char: c,
                fg: theme.inactive_fg,
                bg: theme.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
    }
    ctx.mark_dirty(0, footer_y, width, 2);
}