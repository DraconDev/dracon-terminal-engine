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
//! let gauge = Gauge::new("CPU")
//!     .bind_command(BoundCommand::new("cat /proc/loadavg").refresh(5));
//! app.add_widget(Box::new(gauge), area);
//! ```
//!
//! The framework handles spawning the command, parsing output, and updating the widget.

use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widgets::{Gauge, KeyValueGrid, SplitPane};
use ratatui::layout::Rect;

fn main() -> std::io::Result<()> {
    App::new()?
        .title("Command Dashboard")
        .fps(30)
        .theme(Theme::nord())
        .on_tick(|ctx, _tick| {
            let (w, h) = ctx.compositor().size();
            let header_height = 4u16;
            let col_w = w / 3;

            let cpu_area = Rect::new(0, 0, col_w, header_height);
            let mem_area = Rect::new(col_w, 0, col_w, header_height);
            let disk_area = Rect::new(col_w * 2, 0, w - col_w * 2, header_height);

            let cpu_gauge = Gauge::new("CPU %").max(100.0);
            ctx.add_plane(cpu_gauge.render(cpu_area));

            let mem_gauge = Gauge::new("Memory %").max(100.0);
            ctx.add_plane(mem_gauge.render(mem_area));

            let disk_gauge = Gauge::new("Disk %").max(100.0);
            ctx.add_plane(disk_gauge.render(disk_area));
        })
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();

            // --- Gauge bound to a real command ---
            // Re-runs `cat /proc/loadavg` every 2 seconds, parses scalar (e.g. "0.52")
            let cpu_gauge = Gauge::new("CPU")
                .bind_command(
                    BoundCommand::new("cat /proc/loadavg")
                        .parser(OutputParser::Regex {
                            pattern: r"^([0-9.]+)".into(),
                            group: Some(0),
                        })
                        .refresh(2),
                );
            ctx.add_plane(cpu_gauge.render(Rect::new(0, 0, 30, 3)));

            // --- Gauge with fixed value (no command) ---
            let mut mem_gauge = Gauge::new("Memory").max(100.0);
            mem_gauge.set_value(64.0);
            ctx.add_plane(mem_gauge.render(Rect::new(30, 0, 30, 3)));

            // --- KeyValueGrid showing system info from a shell command ---
            // Parses `uname -a` output into key:value lines
            let kv_grid = KeyValueGrid::new()
                .bind_command(
                    BoundCommand::new("echo 'OS: NixOS\nKernel: 6.12.1\nShell: zsh 5.9'")
                        .refresh(0), // 0 = never auto-refresh
                );
            ctx.add_plane(kv_grid.render(Rect::new(0, 3, 60, 5)));

            // --- Status badge ---
            let status_badge = StatusBadge::new(WidgetId::default_id())
                .with_status("OK")
                .with_label("System");
            ctx.add_plane(status_badge.render(Rect::new(0, 8, 15, 1)));
        })
}