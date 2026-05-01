//! Log Monitor — Real-time log viewer with severity parsing and filtering.
//!
//! Demonstrates:
//! - LogViewer with severity-colored log lines
//! - Auto-scroll with pause on user interaction
//! - Log level filtering (INFO/WARN/ERROR/DEBUG)
//! - StatusBadge showing update frequency and line count
//!
//! ## Controls
//!
//! - `c` — Clear all logs
//! - `r` — Resume auto-scroll
//! - Click on LogViewer — Pause auto-scroll
//! - Filter buttons toggle visibility of each severity

use std::io::Result;
use std::time::{Duration, Instant};

use rand::Rng;

use dracon_terminal_engine::framework::app::App;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::log_viewer::{LogLevel, LogViewer};
use dracon_terminal_engine::framework::widgets::status_badge::StatusBadge;
use ratatui::layout::Rect;

const LOG_MESSAGES: &[(&str, &str)] = &[
    ("INFO", "Application started successfully"),
    ("INFO", "Connection established to database"),
    ("WARN", "High memory usage detected: 85%"),
    ("DEBUG", "Processing request from client 192.168.1.100"),
    ("ERROR", "Failed to connect to cache server"),
    ("INFO", "Cache invalidated for user session"),
    ("WARN", "Slow query detected: 2.3s"),
    ("ERROR", "Unhandled exception in worker thread"),
    ("DEBUG", "Garbage collection completed in 150ms"),
    ("INFO", "User authentication successful: admin"),
    ("DEBUG", "TCP keepalive probe sent"),
    ("WARN", "Disk space low: 10% remaining"),
    ("ERROR", "Connection timeout to 10.0.0.5:8080"),
    ("INFO", "Background task completed in 45ms"),
    ("DEBUG", "Cache hit for key: user_session_abc"),
];

struct LogMonitor {
    id: WidgetId,
    log_viewer: LogViewer,
    status_badge: StatusBadge,
    last_log_time: Instant,
    line_count: usize,
    filter_debug: bool,
    filter_info: bool,
    filter_warn: bool,
    filter_error: bool,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl LogMonitor {
    fn new(id: WidgetId) -> Self {
        let mut log_viewer = LogViewer::with_id(WidgetId::new(id.value + 1));
        log_viewer = log_viewer.max_lines(500).auto_scroll(true);

        let mut status_badge = StatusBadge::new(WidgetId::new(id.value + 2));
        status_badge = status_badge.with_label("lines").with_status("0s");

        Self {
            id,
            log_viewer,
            status_badge,
            last_log_time: Instant::now(),
            line_count: 0,
            filter_debug: true,
            filter_info: true,
            filter_warn: true,
            filter_error: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 20)),
            dirty: true,
        }
    }

    fn generate_log(&mut self) {
        let mut rng = rand::thread_rng();
        let (level, msg) = LOG_MESSAGES[rng.gen_range(0..LOG_MESSAGES.len())];

        let now = chrono_lite_timestamp();
        let raw = format!("[{}] {} - {}", now, level, msg);

        self.log_viewer.append_line(&raw);
        self.last_log_time = Instant::now();
        self.line_count += 1;
        self.dirty = true;
    }

    fn update_status(&mut self) {
        let elapsed = self.last_log_time.elapsed().as_secs();
        let status = if elapsed < 1 {
            "just now".to_string()
        } else {
            format!("{}s ago", elapsed)
        };
        self.status_badge.set_status(&status);
        self.status_badge.set_label(&format!("{} lines", self.line_count));
    }

    fn clear_logs(&mut self) {
        self.log_viewer.clear();
        self.line_count = 0;
        self.dirty = true;
    }
}

fn chrono_lite_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;
    let ms = now.subsec_millis();
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, ms)
}

impl Widget for LogMonitor {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
        self.log_viewer.set_area(Rect::new(area.x, area.y + 2, area.width, area.height - 4));
        self.status_badge.set_area(Rect::new(area.x, area.y + area.height - 1, area.width, 1));
        self.dirty = true;
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.log_viewer.needs_render() || self.status_badge.needs_render()
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
        self.log_viewer.clear_dirty();
        self.status_badge.clear_dirty();
    }

    fn render(&self, area: Rect) -> dracon_terminal_engine::compositor::Plane {
        use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};

        let mut plane = Plane::new(0, area.width, area.height);

        let header_row = 0;
        let header_text = " Log Monitor ";
        let filters_text = "[INFO] [WARN] [ERROR] [DEBUG]";
        let clear_text = "[Clear]";

        for x in 0..area.width {
            let idx = header_row as usize * area.width as usize + x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: ' ',
                    fg: Color::Reset,
                    bg: Color::Reset,
                    style: Styles::empty(),
                    transparent: true,
                    skip: false,
                };
            }
        }

        for (i, c) in header_text.chars().enumerate() {
            let x = 2 + i as u16;
            let idx = header_row as usize * area.width as usize + x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: Color::Cyan,
                    bg: Color::Reset,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        let filter_x = area.width.saturating_sub(
            filters_text.len() as u16 + clear_text.len() as u16 + 4
        );
        let filter_colors = [
            (Color::Green, self.filter_info),
            (Color::Yellow, self.filter_warn),
            (Color::Red, self.filter_error),
            (Color::Gray, self.filter_debug),
        ];
        let filter_labels = ["[INFO]", "[WARN]", "[ERROR]", "[DEBUG]"];

        let mut fx = filter_x;
        for (i, (color, active)) in filter_colors.iter().enumerate() {
            let label = filter_labels[i];
            let fg = if *active { *color } else { Color::DarkGray };
            for (j, c) in label.chars().enumerate() {
                let idx = header_row as usize * area.width as usize + (fx + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg: Color::Reset,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
            fx += label.len() as u16 + 1;
        }

        let clear_x = area.width.saturating_sub(clear_text.len() as u16 + 2);
        for (i, c) in clear_text.chars().enumerate() {
            let idx = header_row as usize * area.width as usize + (clear_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: Color::Cyan,
                    bg: Color::Reset,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        let separator_row = 1;
        for x in 0..area.width {
            let idx = separator_row as usize * area.width as usize + x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: '─',
                    fg: Color::DarkGray,
                    bg: Color::Reset,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        let log_area = Rect::new(area.x, area.y + 2, area.width, area.height - 4);
        let log_plane = self.log_viewer.render(log_area);
        for (i, cell) in log_plane.cells.iter().enumerate() {
            let row = i / area.width as usize;
            let col = i % area.width as usize;
            let target_idx = (row + 2) * area.width as usize + col;
            if target_idx < plane.cells.len() && row < log_plane.height as usize {
                plane.cells[target_idx] = cell.clone();
            }
        }

        let status_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        let status_plane = self.status_badge.render(status_area);
        let status_row = (area.height - 1) as usize;
        for (i, cell) in status_plane.cells.iter().enumerate() {
            let col = i % area.width as usize;
            let target_idx = status_row * area.width as usize + col;
            if target_idx < plane.cells.len() {
                plane.cells[target_idx] = cell.clone();
            }
        }

        plane
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};

        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Char('c') => {
                self.clear_logs();
                true
            }
            KeyCode::Char('r') => {
                self.log_viewer.auto_scroll = true;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: dracon_terminal_engine::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        use dracon_terminal_engine::input::event::MouseEventKind;

        if kind == MouseEventKind::Down(ratatui::mouse::MouseButton::Left) {
            let rel_y = row;
            let rel_x = col;

            if rel_y == 0 {
                let filter_start = self.area.get().width.saturating_sub(40);
                if rel_x >= filter_start {
                    let filter_index = ((rel_x - filter_start) / 7) as usize;
                    match filter_index {
                        0 => {
                            self.filter_info = !self.filter_info;
                            self.dirty = true;
                        }
                        1 => {
                            self.filter_warn = !self.filter_warn;
                            self.dirty = true;
                        }
                        2 => {
                            self.filter_error = !self.filter_error;
                            self.dirty = true;
                        }
                        3 => {
                            self.filter_debug = !self.filter_debug;
                            self.dirty = true;
                        }
                        _ => {}
                    }
                }
            } else if rel_y >= 2 && rel_y < self.area.get().height - 1 {
                self.log_viewer.auto_scroll = false;
                self.dirty = true;
            }

            true
        } else {
            false
        }
    }
}

impl Default for LogMonitor {
    fn default() -> Self {
        Self::new(WidgetId::new(1))
    }
}

fn main() -> Result<()> {
    println!("Log Monitor");
    println!("============");
    println!("Controls:");
    println!("  c - Clear logs");
    println!("  r - Resume auto-scroll");
    println!("  Click - Pause auto-scroll / toggle filters");
    println!();

    std::thread::sleep(Duration::from_millis(300));

    let mut app = App::new()?.title("Log Monitor").fps(30);

    let theme = Theme::dark();
    app.set_theme(theme);

    let mut monitor = LogMonitor::new(WidgetId::new(1));

    app.tick_interval(200)
        .on_tick(move |ctx, tick| {
            if tick % 2 == 0 {
                monitor.generate_log();
            }
            monitor.update_status();
            monitor.mark_dirty();
        })
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();

            if monitor.area.get().width != w || monitor.area.get().height != h {
                monitor.set_area(Rect::new(0, 0, w, h));
            }

            if monitor.needs_render() {
                let plane = monitor.render(monitor.area());
                ctx.add_plane(plane);
                monitor.clear_dirty();
            }
        })
}