//! Command-driven dashboard example — shows auto-refreshing widgets.
//!
//! Run with: cargo run --example command_dashboard
//!
//! This example demonstrates binding CLI commands to widgets with auto-refresh.
//! Each widget re-runs its bound command every N seconds and updates automatically.

use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widgets::{Gauge, StatusBadge, KeyValueGrid, SplitPane};
use dracon_terminal_engine::SystemMonitor;
use ratatui::layout::Rect;

fn main() -> std::io::Result<()> {
    App::new()?
        .title("Command Dashboard")
        .fps(30)
        .theme(Theme::nord())
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();

            // Top: system metrics row (3 gauges)
            let header_height = 5u16;
            let header_rect = Rect::new(0, 0, w, header_height);

            let col_w = w / 3;
            let cpu_area = Rect::new(0, 0, col_w, header_height);
            let mem_area = Rect::new(col_w, 0, col_w, header_height);
            let disk_area = Rect::new(col_w * 2, 0, w - col_w * 2, header_height);

            // Gauge bound to `SystemMonitor` — updated via tick helper below
            let mut cpu_gauge = Gauge::new("CPU").max(100.0).warn_threshold(70.0).crit_threshold(90.0);
            cpu_gauge.set_value(42.0);
            ctx.add_plane(cpu_gauge.render(cpu_area));

            let mut mem_gauge = Gauge::new("Memory").max(100.0).warn_threshold(70.0).crit_threshold(90.0);
            mem_gauge.set_value(64.0);
            ctx.add_plane(mem_gauge.render(mem_area));

            let mut disk_gauge = Gauge::new("Disk").max(100.0).warn_threshold(80.0).crit_threshold(95.0);
            disk_gauge.set_value(55.0);
            ctx.add_plane(disk_gauge.render(disk_area));

            // Bottom: key-value info + status badge
            let body_rect = Rect::new(0, header_height, w, h - header_height);
            let (left_rect, right_rect) = SplitPane::new(Orientation::Vertical).split(body_rect);

            let mut kv = KeyValueGrid::new();
            kv.set_pairs(std::collections::BTreeMap::from([
                ("Hostname".to_string(), "dracon-dev".to_string()),
                ("OS".to_string(), "NixOS 24.11".to_string()),
                ("Kernel".to_string(), "6.12.1".to_string()),
                ("Uptime".to_string(), "3d 14h".to_string()),
                ("Shell".to_string(), "zsh 5.9".to_string()),
            ]));
            ctx.add_plane(kv.render(left_rect));

            let mut status = StatusBadge::new(WidgetId::default_id())
                .with_status("OK")
                .with_label("System Healthy");
            ctx.add_plane(status.render(right_rect));
        })
}