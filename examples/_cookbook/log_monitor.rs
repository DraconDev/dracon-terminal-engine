#![allow(missing_docs)]
//! Log Monitor — Real-time log viewer with severity filtering.
//!
//! Controls: `c` clear, `r` resume auto-scroll, click filter buttons to toggle.

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
use dracon_terminal_engine::framework::widgets::LogViewer;
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
    last_log: Instant,
    total_lines: usize,
    area: Rect,
    dirty: bool,
    auto_scroll: bool,
    filter_info: bool,
    filter_warn: bool,
    filter_error: bool,
    filter_debug: bool,
    theme: Theme,
}

impl LogMonitor {
    fn new() -> Self {
        Self {
            id: WidgetId::new(1),
            log_viewer: LogViewer::with_id(WidgetId::new(2)).max_lines(500),
            last_log: Instant::now(),
            total_lines: 0,
            area: Rect::new(0, 0, 80, 20),
            dirty: true,
            auto_scroll: true,
            filter_info: true,
            filter_warn: true,
            filter_error: true,
            filter_debug: true,
            theme: Theme::nord(),
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

    fn clear(&mut self) {
        self.log_viewer.clear();
        self.total_lines = 0;
        self.dirty = true;
    }

    fn tick(&mut self) {
        self.push_log();
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
        let la = Rect::new(area.x + 1, area.y + 3, area.width - 2, area.height - 5);
        self.log_viewer.set_area(la);
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
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.log_viewer.on_theme_change(theme);
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut p = Plane::new(0, area.width, area.height);

        for c in p.cells.iter_mut() {
            c.bg = t.bg;
            c.fg = t.fg;
        }

        let w = area.width as usize;
        let h = area.height as usize;

        // ── Rounded border ──
        p.cells[0] = Cell { char: '╭', fg: t.outline, bg: t.bg, style: Styles::empty(), transparent: false, skip: false };
        p.cells[w - 1] = Cell { char: '╮', fg: t.outline, bg: t.bg, style: Styles::empty(), transparent: false, skip: false };
        p.cells[(h - 1) * w] = Cell { char: '╰', fg: t.outline, bg: t.bg, style: Styles::empty(), transparent: false, skip: false };
        p.cells[(h - 1) * w + w - 1] = Cell { char: '╯', fg: t.outline, bg: t.bg, style: Styles::empty(), transparent: false, skip: false };
        for x in 1..w - 1 {
            p.cells[x] = Cell { char: '─', fg: t.outline, bg: t.bg, style: Styles::empty(), transparent: false, skip: false };
            p.cells[(h - 1) * w + x] = Cell { char: '─', fg: t.outline, bg: t.bg, style: Styles::empty(), transparent: false, skip: false };
        }
        for y in 1..h - 1 {
            p.cells[y * w] = Cell { char: '│', fg: t.outline, bg: t.bg, style: Styles::empty(), transparent: false, skip: false };
            p.cells[y * w + w - 1] = Cell { char: '│', fg: t.outline, bg: t.bg, style: Styles::empty(), transparent: false, skip: false };
        }

        // ── Header bar with icon and title ──
        let header = " 󰑊 Log Monitor ";
        for (i, c) in header.chars().enumerate().take(w - 4) {
            p.cells[1 + i] = Cell { char: c, fg: t.fg_on_accent, bg: t.primary, style: Styles::BOLD, transparent: false, skip: false };
        }
        for x in (1 + header.len())..w - 1 {
            p.cells[1 + x] = Cell { char: '─', fg: t.primary, bg: t.primary, style: Styles::empty(), transparent: false, skip: false };
        }

        // ── Filter toggles ──
        let filters = [
            ("INFO", self.filter_info, t.info),
            ("WARN", self.filter_warn, t.warning),
            ("ERROR", self.filter_error, t.error),
            ("DEBUG", self.filter_debug, t.fg_muted),
        ];
        let filter_x = 3;
        for (i, (lvl, active, color)) in filters.iter().enumerate() {
            let fx = filter_x + i as u16 * 10;
            let check = if *active { "✓" } else { " " };
            let label = format!("[{}] {}", check, lvl);
            for (j, c) in label.chars().enumerate() {
                let idx = (2 * w + fx as usize + j) as usize;
                if idx < p.cells.len() {
                    let fg = if *active { *color } else { t.fg_muted };
                    p.cells[idx] = Cell { char: c, fg, bg: t.surface, style: Styles::empty(), transparent: false, skip: false };
                }
            }
        }

        // ── Log content area ──
        let content_top = 3;
        let content_bottom = h - 2;
        for y in content_top..content_bottom {
            for x in 1..w - 1 {
                let idx = (y * w + x) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].bg = t.surface;
                }
            }
        }

        // ── Log viewer ──
        let la = Rect::new(1, content_top as u16, (w - 2) as u16, (content_bottom - content_top) as u16);
        let lp = self.log_viewer.render(la);
        for (i, c) in lp.cells.iter().enumerate() {
            if c.transparent {
                continue;
            }
            let row = i / lp.width as usize;
            let col = i % lp.width as usize;
            let src_x = la.x + col as u16;
            let src_y = la.y + row as u16;
            if src_y < area.height - 1 && src_x < area.width - 1 {
                let target = (src_y as usize * w + src_x as usize) as usize;
                if target < p.cells.len() {
                    p.cells[target] = c.clone();
                }
            }
        }

        // ── Status bar ──
        let s = self.last_log.elapsed().as_secs();
        let last_str = if s < 1 { "now" } else { &format!("{}s ago", s) };
        let auto_str = if self.auto_scroll { "auto" } else { "paused" };
        let status = format!(" 󰔱 {}  |  󰑎 {} lines  |  scroll: {}  |  c=clear r=resume  |  q=quit", last_str, self.total_lines, auto_str);
        for (i, c) in status.chars().enumerate().take(w - 2) {
            let idx = ((h - 1) * w + 1 + i) as usize;
            if idx < p.cells.len() {
                p.cells[idx] = Cell { char: c, fg: t.fg_muted, bg: t.surface_elevated, style: Styles::empty(), transparent: false, skip: false };
            }
        }
        for x in 1..w - 1 {
            let idx = ((h - 1) * w + x) as usize;
            p.cells[idx].bg = t.surface_elevated;
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

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if kind == MouseEventKind::Down(MouseButton::Left) {
            // Click on filter toggles (row 2)
            if row == 2 {
                let filter_x = 3u16;
                for i in 0..4 {
                    let fx = filter_x + i * 10;
                    if col >= fx && col < fx + 7 {
                        match i {
                            0 => self.filter_info = !self.filter_info,
                            1 => self.filter_warn = !self.filter_warn,
                            2 => self.filter_error = !self.filter_error,
                            3 => self.filter_debug = !self.filter_debug,
                            _ => {}
                        }
                        self.dirty = true;
                        return true;
                    }
                }
            }
            // Click on log area to pause
            if row >= 3 && row < self.area.height - 1 {
                self.auto_scroll = !self.auto_scroll;
                self.log_viewer.auto_scroll = self.auto_scroll;
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
    println!("Log Monitor — c=clear, r=resume, click filters to toggle");
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