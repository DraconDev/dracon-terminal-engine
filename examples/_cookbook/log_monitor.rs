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

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
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
    all_logs: Vec<String>,
    show_help: bool,
    keybindings: KeybindingSet,
}

impl LogMonitor {
    fn new(theme: Theme) -> Self {
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
            theme,
            all_logs: Vec::new(),
            show_help: false,
            keybindings: KeybindingSet::default(),
        }
    }

    fn push_log(&mut self) {
        let mut rng = rand::thread_rng();
        let (lvl, msg) = LOGS[rng.gen_range(0..LOGS.len())];
        let t = format_time();
        let line = format!("[{}] {} - {}", t, lvl, msg);
        self.all_logs.push(line.clone());
        self.total_lines += 1;

        // Only append to viewer if filter allows this level
        if self.level_visible(lvl) {
            self.log_viewer.append_line(&line);
        }
        self.last_log = Instant::now();
        self.dirty = true;
    }

    fn level_visible(&self, lvl: &str) -> bool {
        match lvl {
            "INFO" => self.filter_info,
            "WARN" => self.filter_warn,
            "ERROR" => self.filter_error,
            "DEBUG" => self.filter_debug,
            _ => true,
        }
    }

    fn apply_filters(&mut self) {
        self.log_viewer.clear();
        for line in &self.all_logs {
            // Extract level from line format: [HH:MM:SS.mmm] LEVEL - msg
            if let Some(level_start) = line.find(']') {
                if let Some(level_end) = line.find(" -") {
                    let level = &line[level_start + 2..level_end];
                    if self.level_visible(level) {
                        self.log_viewer.append_line(line);
                    }
                }
            }
        }
        self.dirty = true;
    }

    fn clear(&mut self) {
        self.log_viewer.clear();
        self.all_logs.clear();
        self.total_lines = 0;
        self.dirty = true;
    }

    fn tick(&mut self) {
        self.push_log();
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.log_viewer.on_theme_change(&self.theme);
        self.dirty = true;
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let t = self.theme.clone();
        let w = 42.min(area.width as usize);
        let h = 14.min(area.height as usize);
        let x = (area.width as usize - w) / 2;
        let y = (area.height as usize - h) / 2;

        // Rounded box background
        for row in 0..h {
            for col in 0..w {
                let idx = (y + row) * plane.width as usize + x + col;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].fg = t.fg;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        // Rounded corners (top row)
        if w >= 2 && h >= 1 {
            plane.cells[y * plane.width as usize + x].char = '╭';
            plane.cells[y * plane.width as usize + x].fg = t.outline;
            plane.cells[y * plane.width as usize + x + w - 1].char = '╮';
            plane.cells[y * plane.width as usize + x + w - 1].fg = t.outline;
        }
        // Rounded corners (bottom row)
        if h >= 2 {
            let bot_y = (y + h - 1) * plane.width as usize;
            plane.cells[bot_y + x].char = '╰';
            plane.cells[bot_y + x].fg = t.outline;
            plane.cells[bot_y + x + w - 1].char = '╯';
            plane.cells[bot_y + x + w - 1].fg = t.outline;
        }
        // Horizontal borders
        if h >= 1 {
            for col in 1..w - 1 {
                plane.cells[y * plane.width as usize + x + col].char = '─';
                plane.cells[y * plane.width as usize + x + col].fg = t.outline;
            }
        }
        if h >= 2 {
            let bot_y = (y + h - 1) * plane.width as usize;
            for col in 1..w - 1 {
                plane.cells[bot_y + x + col].char = '─';
                plane.cells[bot_y + x + col].fg = t.outline;
            }
        }
        // Vertical borders
        for row in 1..h - 1 {
            plane.cells[(y + row) * plane.width as usize + x].char = '│';
            plane.cells[(y + row) * plane.width as usize + x].fg = t.outline;
            plane.cells[(y + row) * plane.width as usize + x + w - 1].char = '│';
            plane.cells[(y + row) * plane.width as usize + x + w - 1].fg = t.outline;
        }

        let lines = [
            "┌─ Log Monitor Help ─────────────┐",
            &format!("│ {:<6} Clear all logs           │", self.keybindings.display(actions::NEW_ITEM).unwrap_or("c")),
            &format!("│ {:<6} Resume auto-scroll       │", self.keybindings.display(actions::REFRESH).unwrap_or("r")),
            &format!("│ {:<6} Cycle theme              │", self.keybindings.display(actions::THEME).unwrap_or("t")),
            &format!("│ {:<6} Toggle this help         │", self.keybindings.display(actions::HELP).unwrap_or("?")),
            &format!("│ {:<6} Dismiss help             │", self.keybindings.display(actions::BACK).unwrap_or("esc")),
            "│ Click  Toggle log level filter │",
            "│ Click  Pause/resume scroll      │",
            &format!("│ {:<6} Quit application         │", self.keybindings.display(actions::QUIT).unwrap_or("q")),
            "└────────────────────────────────┘",
        ];
        let start_y = y + (h - lines.len()) / 2;
        for (i, line) in lines.iter().enumerate() {
            let row = start_y + i;
            for (j, c) in line.chars().enumerate() {
                let idx = row * plane.width as usize + x + 2 + j;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                }
            }
        }
    }
}

fn format_time() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
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
        self.theme = theme.clone();
        self.log_viewer.on_theme_change(theme);
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let t = self.theme.clone();
        let mut p = Plane::new(0, area.width, area.height);

        for c in p.cells.iter_mut() {
            c.bg = t.bg;
            c.fg = t.fg;
        }

        let w = area.width as usize;
        let h = area.height as usize;

        // ── Rounded border ──
        p.cells[0] = Cell {
            char: '╭',
            fg: t.outline,
            bg: t.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        p.cells[w - 1] = Cell {
            char: '╮',
            fg: t.outline,
            bg: t.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        p.cells[(h - 1) * w] = Cell {
            char: '╰',
            fg: t.outline,
            bg: t.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        p.cells[(h - 1) * w + w - 1] = Cell {
            char: '╯',
            fg: t.outline,
            bg: t.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        for x in 1..w - 1 {
            p.cells[x] = Cell {
                char: '─',
                fg: t.outline,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
            p.cells[(h - 1) * w + x] = Cell {
                char: '─',
                fg: t.outline,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
        for y in 1..h - 1 {
            p.cells[y * w] = Cell {
                char: '│',
                fg: t.outline,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
            p.cells[y * w + w - 1] = Cell {
                char: '│',
                fg: t.outline,
                bg: t.bg,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }

        // ── Header bar with icon and title ──
        let header = " 󰑊 Log Monitor ";
        for (i, c) in header.chars().enumerate().take(w - 4) {
            p.cells[1 + i] = Cell {
                char: c,
                fg: t.fg_on_accent,
                bg: t.primary,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        for x in (1 + header.len())..w - 1 {
            p.cells[1 + x] = Cell {
                char: '─',
                fg: t.primary,
                bg: t.primary,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
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
                let idx = 2 * w + fx as usize + j;
                if idx < p.cells.len() {
                    let fg = if *active { *color } else { t.fg_muted };
                    p.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg: t.surface,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        // ── Log content area ──
        let content_top = 3;
        let content_bottom = h - 2;
        for y in content_top..content_bottom {
            for x in 1..w - 1 {
                let idx = y * w + x;
                if idx < p.cells.len() {
                    p.cells[idx].bg = t.surface;
                }
            }
        }

        // ── Log viewer ──
        let la = Rect::new(
            1,
            content_top as u16,
            (w - 2) as u16,
            (content_bottom - content_top) as u16,
        );
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
                let target = src_y as usize * w + src_x as usize;
                if target < p.cells.len() {
                    p.cells[target] = *c;
                }
            }
        }

        // ── Status bar ──
        let s = self.last_log.elapsed().as_secs();
        let last_str = if s < 1 { "now" } else { &format!("{}s ago", s) };
        let auto_str = if self.auto_scroll { "auto" } else { "paused" };
        let status = format!(" 󰔱 {}  |  󰑎 {} lines  |  scroll: {}  |  {}: theme | {}: help | {}: dismiss | {}: clear | {}: resume | {}: quit", last_str, self.total_lines, auto_str,
            self.keybindings.display(actions::THEME).unwrap_or("t"),
            self.keybindings.display(actions::HELP).unwrap_or("?"),
            self.keybindings.display(actions::BACK).unwrap_or("esc"),
            self.keybindings.display(actions::NEW_ITEM).unwrap_or("c"),
            self.keybindings.display(actions::REFRESH).unwrap_or("r"),
            self.keybindings.display(actions::QUIT).unwrap_or("q"),
        );
        for (i, c) in status.chars().enumerate().take(w - 2) {
            let idx = (h - 1) * w + 1 + i;
            if idx < p.cells.len() {
                p.cells[idx] = Cell {
                    char: c,
                    fg: t.fg_muted,
                    bg: t.surface_elevated,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }
        for x in 1..w - 1 {
            let idx = (h - 1) * w + x;
            p.cells[idx].bg = t.surface_elevated;
        }

        // ── Help overlay ──
        if self.show_help {
            self.render_help_overlay(&mut p, area);
        }

        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Char('c') if key.modifiers.is_empty() => {
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
                        self.apply_filters();
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
        Self::new(Theme::from_env_or(Theme::nord()))
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

    fn on_theme_change(&mut self, theme: &Theme) {
        self.target.borrow_mut().on_theme_change(theme);
    }
    fn current_theme(&self) -> Option<Theme> {
        Some(self.target.borrow().theme.clone())
    }
}

fn main() -> Result<()> {
    println!("Log Monitor — c=clear, r=resume, click filters to toggle");
    std::thread::sleep(Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());
    let kb_input = keybindings.clone();
    let env_theme = Theme::from_env_or(Theme::nord());

    let mon = Rc::new(RefCell::new(LogMonitor::new(env_theme.clone())));
    mon.borrow_mut().keybindings = keybindings;
    let mon_for_tick = Rc::clone(&mon);
    let mon_for_input_router = Rc::clone(&mon);
    let mon_for_input_closure = Rc::clone(&mon);

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app_ctx = App::new()?.title("Log Monitor").fps(30).tick_interval(200).theme(env_theme.clone());

    let router = InputRouter {
        target: mon_for_input_router,
        id: WidgetId::new(100),
        area: Rect::new(0, 0, w, h),
    };
    app_ctx.add_widget(Box::new(router), Rect::new(0, 0, w, h));

    app_ctx
        .on_input(move |key| {
            if key.kind != KeyEventKind::Press {
                return false;
            }
            let mut mon = mon_for_input_closure.borrow_mut();
            if kb_input.matches(actions::QUIT, &key) {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else if kb_input.matches(actions::THEME, &key) {
                mon.cycle_theme();
                true
            } else if kb_input.matches(actions::HELP, &key) {
                mon.show_help = !mon.show_help;
                mon.dirty = true;
                true
            } else if mon.show_help && kb_input.matches(actions::BACK, &key) {
                mon.show_help = false;
                mon.dirty = true;
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
