//! Command Bindings — Auto-refresh widgets via CLI commands.
//!
//! Demonstrates all 5 command-bound widgets with mock commands that update
//! them on configurable intervals. Shows the core framework value proposition:
//! any widget can be bound to a CLI command with custom parsers.
//!
//! ## Widgets
//!
//! | Widget | Mock Command | Parser | Interval |
//! |--------|-------------|--------|----------|
//! | Gauge (CPU) | `echo "cpu:67"` | regex | 2s |
//! | KeyValueGrid | `echo -e "Memory:8.2GB\n..."` | text/kv | 5s |
//! | StatusBadge | `echo "$((RANDOM % 2 ? 'OK' : 'ERROR'))"` | plain | 10s |
//! | LogViewer | `echo "[INFO]..."` | severity | 3s |
//! | StreamingText | `date +"%H:%M:%S"` | plain | 1s |
//!
//! ## Controls
//!
//! - `s` — trigger manual refresh of all commands
//! - `p` — pause/resume auto-refresh
//! - `Ctrl+C` — quit

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::command::{BoundCommand, CommandRunner, OutputParser, ParsedOutput};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Gauge, KeyValueGrid, LogViewer, StatusBadge, StreamingText,
};
use ratatui::layout::Rect;

struct CommandBindings {
    id: WidgetId,
    gauge: Gauge,
    kv_grid: KeyValueGrid,
    status: StatusBadge,
    log_viewer: LogViewer,
    streaming: StreamingText,
    theme: Theme,
    area: Rect,
    dirty: bool,
    paused: bool,
    next_refresh: Instant,
}

impl CommandBindings {
    fn new() -> Self {
        let gauge = Gauge::new("CPU")
            .max(100.0)
            .warn_threshold(70.0)
            .crit_threshold(90.0)
            .bind_command(
                BoundCommand::new(r#"echo "cpu:67""#)
                    .parser(OutputParser::Regex {
                        pattern: r"cpu:(\d+)".into(),
                        group: Some(1),
                    })
                    .refresh(2)
                    .label("cpu gauge"),
            );

        let kv_grid = KeyValueGrid::new()
            .separator(" ")
            .bind_command(
                BoundCommand::new(r#"echo -e "Memory:8.2GB\nDisk:45%\nNetwork:120Mbps\nUptime:3d15h""#)
                    .refresh(5)
                    .label("system info"),
            );

        let status = StatusBadge::new(WidgetId::new(4))
            .with_label("Connection")
            .bind_command(
                BoundCommand::new(
                    r#"bash -c 'if [ $(date +%S) -gt 30 ]; then echo "ERROR"; else echo "OK"; fi'"#
                )
                .refresh(10)
                .label("connection status"),
            );

        let log_viewer = LogViewer::with_id(WidgetId::new(5))
            .max_lines(200)
            .auto_scroll(true)
            .bind_command(
                BoundCommand::new(
                    r#"echo -e "[INFO] Connected to server\n[WARN] High load detected\n[ERROR] Connection timeout 3s""#,
                )
                .parser(OutputParser::SeverityLine {
                    patterns: [
                        ("ERROR".into(), "red".into()),
                        ("WARN".into(), "yellow".into()),
                        ("INFO".into(), "default".into()),
                    ]
                    .into_iter()
                    .collect(),
                })
                .refresh(3)
                .label("event log"),
            );

        let streaming = StreamingText::with_id(WidgetId::new(6))
            .max_lines(50)
            .auto_scroll(true)
            .bind_command(
                BoundCommand::new("date +\"Last tick: %H:%M:%S\"")
                    .refresh(1)
                    .label("timestamp"),
            );

        Self {
            id: WidgetId::default_id(),
            gauge,
            kv_grid,
            status,
            log_viewer,
            streaming,
            theme: Theme::nord(),
            area: Rect::new(0, 0, 80, 24),
            dirty: true,
            paused: false,
            next_refresh: Instant::now(),
        }
    }

    fn run_gauge_command(&mut self) {
        let cmd = BoundCommand::new(r#"echo "cpu:67""#)
            .parser(OutputParser::Regex {
                pattern: r"cpu:(\d+)".into(),
                group: Some(1),
            });
        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        self.gauge.apply_command_output(&output);
    }

    fn run_kv_command(&mut self) {
        let cmd = BoundCommand::new(r#"echo -e "Memory:8.2GB\nDisk:45%\nNetwork:120Mbps\nUptime:3d15h""#);
        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        self.kv_grid.apply_command_output(&output);
    }

    fn run_status_command(&mut self) {
        let cmd = BoundCommand::new(
            r#"bash -c 'if [ $(date +%S) -gt 30 ]; then echo "ERROR"; else echo "OK"; fi'"#,
        );
        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        self.status.apply_command_output(&output);
    }

    fn run_log_command(&mut self) {
        let cmd = BoundCommand::new(
            r#"echo -e "[INFO] Connected to server\n[WARN] High load detected\n[ERROR] Connection timeout 3s""#,
        )
        .parser(OutputParser::SeverityLine {
            patterns: [
                ("ERROR".into(), "red".into()),
                ("WARN".into(), "yellow".into()),
                ("INFO".into(), "default".into()),
            ]
            .into_iter()
            .collect(),
        });
        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        self.log_viewer.apply_command_output(&output);
    }

    fn run_streaming_command(&mut self) {
        let cmd = BoundCommand::new("date +\"Last tick: %H:%M:%S\"");
        let runner = CommandRunner::new(&cmd.command);
        let (stdout, stderr, exit_code) = runner.run_sync();
        let output = cmd.parse_output(&stdout, &stderr, exit_code);
        self.streaming.apply_command_output(&output);
    }

    fn refresh_all(&mut self) {
        self.run_gauge_command();
        self.run_kv_command();
        self.run_status_command();
        self.run_log_command();
        self.run_streaming_command();
        self.next_refresh = Instant::now();
        self.dirty = true;
    }

    fn tick(&mut self, elapsed_secs: u64) {
        if self.paused {
            return;
        }
        if elapsed_secs % 2 == 0 {
            self.run_gauge_command();
        }
        if elapsed_secs % 5 == 0 {
            self.run_kv_command();
        }
        if elapsed_secs % 10 == 0 {
            self.run_status_command();
        }
        if elapsed_secs % 3 == 0 {
            self.run_log_command();
        }
        self.run_streaming_command();
        self.dirty = true;
    }
}

impl Default for CommandBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for CommandBindings {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
        let w = area.width;
        let h = area.height;

        let row1_h = 4u16;
        let row2_h = 4u16;
        let row3_h = h - 10;

        self.gauge.set_area(Rect::new(0, 2, 25, row1_h));
        self.status.set_area(Rect::new(0, 2 + row1_h, 25, row2_h));

        let kv_x = 26;
        let kv_w = w.saturating_sub(kv_x);
        self.kv_grid.set_area(Rect::new(kv_x, 2, kv_w, row1_h));

        let log_x = 26;
        let log_w = w.saturating_sub(log_x);
        self.log_viewer
            .set_area(Rect::new(log_x, 2 + row1_h, log_w, row2_h));

        self.streaming.set_area(Rect::new(0, h - 3, w, 3));

        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        10
    }

    fn needs_render(&self) -> bool {
        self.dirty
            || self.gauge.needs_render()
            || self.kv_grid.needs_render()
            || self.status.needs_render()
            || self.log_viewer.needs_render()
            || self.streaming.needs_render()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
        self.gauge.clear_dirty();
        self.kv_grid.clear_dirty();
        self.status.clear_dirty();
        self.log_viewer.clear_dirty();
        self.streaming.clear_dirty();
    }

    fn render(&self, area: Rect) -> Plane {
        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 10;

        for idx in 0..p.cells.len() {
            p.cells[idx].bg = self.theme.bg;
            p.cells[idx].fg = self.theme.fg;
        }

        let title = " Command Bindings — Auto-refresh widgets ";
        let title_color = Color::Cyan;
        let title_width = title.len() as u16;
        let title_x = (area.width.saturating_sub(title_width)) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (i as u16 + title_x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = title_color;
                p.cells[idx].style = Styles::BOLD;
            }
        }

        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = '─';
                p.cells[idx].fg = Color::Ansi(240);
            }
        }

        let gauge_plane = self.gauge.render(self.gauge.area());
        let status_plane = self.status.render(self.status.area());
        let kv_plane = self.kv_grid.render(self.kv_grid.area());
        let log_plane = self.log_viewer.render(self.log_viewer.area());
        let stream_plane = self.streaming.render(self.streaming.area());

        for (src_idx, cell) in gauge_plane.cells.iter().enumerate() {
            let gy = src_idx / gauge_plane.width as usize;
            let gx = src_idx % gauge_plane.width as usize;
            let target = gy * area.width as usize + gx;
            if target < p.cells.len() {
                p.cells[target] = cell.clone();
            }
        }

        let kv_area = self.kv_grid.area();
        for (src_idx, cell) in kv_plane.cells.iter().enumerate() {
            let ry = src_idx / kv_area.width as usize;
            let rx = src_idx % kv_area.width as usize;
            let target = (ry + kv_area.y as usize - 2) * area.width as usize
                + kv_area.x as usize + rx;
            if target < p.cells.len() {
                p.cells[target] = cell.clone();
            }
        }

        let status_area = self.status.area();
        for (src_idx, cell) in status_plane.cells.iter().enumerate() {
            let sy = src_idx / status_area.width as usize;
            let sx = src_idx % status_area.width as usize;
            let target = (status_area.y as usize - 2 + sy) * area.width as usize + sx;
            if target < p.cells.len() {
                p.cells[target] = cell.clone();
            }
        }

        let log_area = self.log_viewer.area();
        for (src_idx, cell) in log_plane.cells.iter().enumerate() {
            let ly = src_idx / log_area.width as usize;
            let lx = src_idx % log_area.width as usize;
            let target = (log_area.y as usize - 2 + ly) * area.width as usize
                + log_area.x as usize + lx;
            if target < p.cells.len() {
                p.cells[target] = cell.clone();
            }
        }

        let stream_area = self.streaming.area();
        for (src_idx, cell) in stream_plane.cells.iter().enumerate() {
            let sy = src_idx / stream_area.width as usize;
            let sx = src_idx % stream_area.width as usize;
            let target = (stream_area.y as usize + sy) * area.width as usize + sx;
            if target < p.cells.len() {
                p.cells[target] = cell.clone();
            }
        }

        let status_line_y = (area.height - 1) as usize;
        let paused_str = if self.paused { "PAUSED" } else { "ON" };
        let status_text = format!(
            " Auto-refresh: {} | Next: {}s | s=refresh p=pause ",
            paused_str,
            3
        );
        for (i, c) in status_text.chars().enumerate() {
            let idx = status_line_y * area.width as usize + i;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = Color::Ansi(240);
            }
        }

        p
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Char('s') => {
                self.refresh_all();
                true
            }
            KeyCode::Char('p') => {
                self.paused = !self.paused;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }
}

fn main() -> std::io::Result<()> {
    println!(
        "Command Bindings — s=refresh all, p=pause, Ctrl+C=quit\nStarting..."
    );
    std::thread::sleep(Duration::from_millis(500));

    let mut app = App::new()?
        .title("Command Bindings")
        .fps(20)
        .tick_interval(1000);

    let mut view = CommandBindings::new();

    app.on_tick(move |ctx, tick| {
        let elapsed = tick;
        view.tick(elapsed);
        view.mark_dirty();

        if view.area.get().width != ctx.compositor().size().0
            || view.area.get().height != ctx.compositor().size().1
        {
            let (w, h) = ctx.compositor().size();
            view.set_area(Rect::new(0, 0, w, h));
        }
    })
    .run(move |ctx| {
        if view.needs_render() {
            ctx.add_plane(view.render(view.area()));
            view.clear_dirty();
        }
    })
}