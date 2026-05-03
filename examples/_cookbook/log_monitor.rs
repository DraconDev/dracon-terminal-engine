#![allow(missing_docs)]
//! Log Monitor — Real-time log viewer with severity parsing and filtering.
//!
//! Demonstrates LogViewer + auto-scroll + severity filtering + StatusBadge.
//!
//! Controls: `c` clear, `r` resume scroll, click to pause/toggle filters.

use std::cell::RefCell;
use std::io::Result;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::Rng;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{LogViewer, StatusBadge};
use ratatui::layout::Rect;

const LOGS: &[(&str, &str)] = &[
    ("INFO", "Application started"),
    ("INFO", "Connection established to database"),
    ("WARN", "High memory usage: 85%"),
    ("DEBUG", "Processing request from 192.168.1.100"),
    ("ERROR", "Failed to connect to cache server"),
    ("INFO", "Cache invalidated for session"),
    ("WARN", "Slow query: 2.3s"),
    ("ERROR", "Unhandled exception in worker"),
    ("DEBUG", "GC completed in 150ms"),
];

struct LogMonitor {
    id: WidgetId,
    log_viewer: LogViewer,
    status: StatusBadge,
    last_log: Instant,
    total_lines: usize,
    area: Rect,
    dirty: bool,
    auto_scroll: bool,
}

impl LogMonitor {
    fn new() -> Self {
        let log_viewer = LogViewer::with_id(WidgetId::new(2)).max_lines(500);
        let status = StatusBadge::new(WidgetId::new(3));
        Self {
            id: WidgetId::new(1),
            log_viewer,
            status,
            last_log: Instant::now(),
            total_lines: 0,
            area: Rect::new(0, 0, 80, 20),
            dirty: true,
            auto_scroll: true,
        }
    }

    fn push_log(&mut self) {
        let mut rng = rand::thread_rng();
        let (lvl, msg) = LOGS[rng.gen_range(0..LOGS.len())];
        let t = format_time();
        self.log_viewer
            .append_line(&format!("[{}] {} - {}", t, lvl, msg));
        self.last_log = Instant::now();
        self.total_lines += 1;
        self.dirty = true;
    }

    fn refresh_status(&mut self) {
        let s = self.last_log.elapsed().as_secs();
        let status_str = if s < 1 {
            "just now".to_string()
        } else {
            format!("{}s ago", s)
        };
        self.status.set_status(&status_str);
    }

    fn clear(&mut self) {
        self.log_viewer.clear();
        self.total_lines = 0;
        self.dirty = true;
    }

    fn tick(&mut self) {
        self.push_log();
        self.refresh_status();
    }
}

fn format_time() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let (h, m, s) = (
        d.as_secs() / 3600 % 24,
        d.as_secs() / 60 % 60,
        d.as_secs() % 60,
    );
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, d.subsec_millis())
}

impl Widget for LogMonitor {
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
        let la = Rect::new(area.x, area.y + 2, area.width, area.height - 4);
        self.log_viewer.set_area(la);
        self.status
            .set_area(Rect::new(area.x, area.y + area.height - 1, area.width, 1));
        self.dirty = true;
    }
    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut p = Plane::new(0, area.width, area.height);

        // Header
        let title = " Log Monitor ";
        for (i, c) in title.chars().enumerate().take(area.width as usize - 2) {
            let idx = (i + 2);
            p.cells[idx] = Cell {
                char: c,
                fg: self.log_viewer.theme.primary,
                bg: Color::Reset,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }

        // Filters
        let fx = area.width.saturating_sub(36);
        let labels = ["[INFO]", "[WARN]", "[ERROR]", "[DEBUG]"];
        let colors = [
            self.log_viewer.theme.info,
            self.log_viewer.theme.warning,
            self.log_viewer.theme.error,
            self.log_viewer.theme.fg_muted,
        ];
        for (i, (&l, &c)) in labels.iter().zip(colors.iter()).enumerate() {
            for (j, ch) in l.chars().enumerate() {
                let idx = (fx + i as u16 * 7 + j as u16) as usize;
                if idx < p.cells.len() {
                    p.cells[idx] = Cell {
                        char: ch,
                        fg: c,
                        bg: Color::Reset,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        // Separator
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx] = Cell {
                    char: '─',
                    fg: self.log_viewer.theme.outline,
                    bg: Color::Reset,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        // Log viewer
        let la = Rect::new(area.x, area.y + 2, area.width, area.height - 4);
        let lp = self.log_viewer.render(la);
        for (i, c) in lp
            .cells
            .iter()
            .enumerate()
            .take(p.cells.len() - 2 * area.width as usize)
        {
            if c.transparent {
                continue;
            }
            let row = i / area.width as usize;
            let col = i % area.width as usize;
            let target = (row + 2) * area.width as usize + col;
            if target < p.cells.len() {
                p.cells[target] = c.clone();
            }
        }

        // Status bar
        let sp = self
            .status
            .render(Rect::new(area.x, area.y + area.height - 1, area.width, 1));
        let sr = (area.height - 1) as usize * area.width as usize;
        for (i, c) in sp.cells.iter().enumerate().take(area.width as usize) {
            if c.transparent {
                continue;
            }
            let idx = sr + i;
            if idx < p.cells.len() {
                p.cells[idx] = c.clone();
            }
        }
        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Char('c') => {
                self.clear();
                true
            }
            KeyCode::Char('r') => {
                self.auto_scroll = true;
                self.log_viewer.auto_scroll = true;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, row: u16) -> bool {
        if kind == MouseEventKind::Down(MouseButton::Left) {
            if row >= 2 && row < self.area.height - 1 {
                self.auto_scroll = false;
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
        Self::new()
    }
}

/// Thin wrapper that routes keyboard/mouse events to a Rc<RefCell<LogMonitor>>.
/// Registered in the widget system so input dispatch works, but does not render
/// (rendering is handled by the on_tick callback calling ctx.add_plane()).
struct InputRouter {
    target: Rc<RefCell<LogMonitor>>,
    id: WidgetId,
    area: Rect,
}

impl Widget for InputRouter {
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
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        false
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, _area: Rect) -> Plane {
        Plane::new(0, 0, 0)
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        self.target.borrow_mut().handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.target.borrow_mut().handle_mouse(kind, col, row)
    }
}

fn main() -> Result<()> {
    println!("Log Monitor — c=clear, r=resume, click=pause/filters");
    std::thread::sleep(Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let mon = Rc::new(RefCell::new(LogMonitor::new()));
    let mon_for_tick = Rc::clone(&mon);
    let mon_for_input = Rc::clone(&mon);

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app_ctx = App::new()?.title("Log Monitor").fps(30).tick_interval(200);

    let router = InputRouter {
        target: mon_for_input,
        id: WidgetId::new(100),
        area: Rect::new(0, 0, w, h),
    };
    app_ctx.add_widget(Box::new(router), Rect::new(0, 0, w, h));

    app_ctx
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, tick| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
                return;
            }
            let mut mon = mon_for_tick.borrow_mut();
            if tick % 2 == 0 {
                mon.tick();
            }
            let (w, h) = ctx.compositor().size();
            if mon.area.width != w || mon.area.height != h {
                mon.set_area(Rect::new(0, 0, w, h));
            }
            if mon.needs_render() {
                ctx.add_plane(mon.render(mon.area));
                mon.clear_dirty();
            }
        })
        .run(|_| {})
}
