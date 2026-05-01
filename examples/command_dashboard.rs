//! Command-driven dashboard example — shows auto-refreshing widgets.
//!
//! Run with: cargo run --example command_dashboard
//!
//! This example demonstrates binding CLI commands to widgets with auto-refresh.
//! Each widget re-runs its bound command every N seconds and updates automatically.
//!
//! ## Pattern
//!
//! ```rust,ignore
//! // Create gauge with a bound command that auto-refreshes every 5 seconds
//! let gauge = Gauge::new("CPU")
//!     .bind_command(BoundCommand::new("cat /proc/loadavg").refresh(5));
//! let id = app.add_widget(Box::new(gauge), area);
//!
//! // The app tick loop re-runs the command and calls apply_command_output automatically
//! ```
//!
//! ## Key concepts
//!
//! - `BoundCommand` binds a CLI command + parser + refresh interval
//! - `OutputParser` extracts values from command output (regex, JSON key, etc.)
//! - `App::add_widget` registers the widget for auto-refresh tracking
//! - `apply_command_output` is called on each widget when its command re-runs

use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::{Gauge, KeyValueGrid, StatusBadge};
use ratatui::layout::Rect;

fn main() -> std::io::Result<()> {
    let mut app = App::new()?
        .title("Command Dashboard")
        .fps(30)
        .theme(Theme::nord());

    // Create gauges bound to real commands
    let cpu_gauge = Gauge::new("CPU %").max(100.0).bind_command(
        BoundCommand::new("cat /proc/loadavg")
            .parser(OutputParser::Regex {
                pattern: r"^([0-9.]+)".into(),
                group: Some(0),
            })
            .refresh(2),
    );

    let mem_gauge = Gauge::new("Memory %").max(100.0).bind_command(
        BoundCommand::new("free | grep Mem | awk '{print int($3/$2*100)}'").refresh(5),
    );

    let disk_gauge = Gauge::new("Disk %").max(100.0).bind_command(
        BoundCommand::new("df -h / | tail -1 | awk '{print $5}' | tr -d '%'").refresh(30),
    );

    let kv_grid = KeyValueGrid::new().separator("  ").bind_command(
        BoundCommand::new("uname -snr").refresh(0), // 0 = never auto-refresh (static data)
    );

    let status = StatusBadge::new(WidgetId::default_id())
        .with_status("OK")
        .with_label("System");

    // Layout: 3 gauges across top (30% height), key-value grid + badge below
    app.add_widget(Box::new(cpu_gauge), Rect::new(0, 0, 26, 3));
    app.add_widget(Box::new(mem_gauge), Rect::new(26, 0, 27, 3));
    app.add_widget(Box::new(disk_gauge), Rect::new(53, 0, 27, 3));
    app.add_widget(Box::new(kv_grid), Rect::new(0, 3, 60, 6));
    app.add_widget(Box::new(status), Rect::new(60, 3, 20, 1));

    app.run(|_ctx| {})
}
