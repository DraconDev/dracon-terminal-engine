#![allow(missing_docs)]
//! Command Bindings — Auto-refresh widgets via CLI commands.
//!
//! Demonstrates all 5 command-bound widgets with mock commands that update
//! them on configurable intervals.
//!
//! ## Controls
//!
//! - `s` — trigger manual refresh of all commands
//! - `p` — pause/resume auto-refresh
//! - `t` — cycle theme (Nord, Cyberpunk, Dracula, Monokai)
//! - `?` — toggle help overlay
//! - `q` — quit
//! - `Esc` — close help overlay

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
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
    tick: u64,
    cpu_value: f32,
    show_help: bool,
    keybindings: KeybindingSet,
    should_quit: Arc<AtomicBool>,
}

impl CommandBindings {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        Self {
            id: WidgetId::new(0),
            gauge: Gauge::new("CPU")
                .max(100.0)
                .warn_threshold(70.0)
                .crit_threshold(90.0),
            kv_grid: KeyValueGrid::new().separator(" "),
            status: StatusBadge::new(WidgetId::new(4))
                .with_status("OK")
                .with_label("Connection"),
            log_viewer: LogViewer::with_id(WidgetId::new(5)).max_lines(200),
            streaming: StreamingText::with_id(WidgetId::new(6)).max_lines(50),
            theme,
            area: Rect::new(0, 0, 80, 24),
            dirty: true,
            paused: false,
            tick: 0,
            cpu_value: 50.0,
            show_help: false,
            keybindings: KeybindingSet::default(),
            should_quit,
        }
    }

    fn refresh_all(&mut self) {
        self.cpu_value = 30.0 + (self.tick % 50) as f32;
        self.gauge.set_value(self.cpu_value as f64);
        let mut pairs = BTreeMap::new();
        pairs.insert(
            "Memory".to_string(),
            format!("{:.1} GB", 8.0 + (self.tick % 10) as f32 * 0.1),
        );
        pairs.insert(
            "Disk".to_string(),
            format!("{}%", 40 + (self.tick % 20) as u32),
        );
        pairs.insert(
            "Network".to_string(),
            format!("{} Mbps", 100 + (self.tick % 50) as u32),
        );
        pairs.insert("Uptime".to_string(), format!("{}h", self.tick / 60));
        self.kv_grid.set_pairs(pairs);
        let status_text = if self.tick % 20 < 15 { "OK" } else { "WARNING" };
        self.status.set_status(status_text);
        self.log_viewer.clear();
        self.log_viewer
            .append_line(&format!("[INFO] Tick {} - System nominal", self.tick));
        self.log_viewer.append_line(&format!(
            "[WARN] Load average: {:.2}",
            1.5 + (self.tick % 10) as f32 * 0.1
        ));
        if self.tick.is_multiple_of(10) {
            self.log_viewer.append_line(&format!(
                "[ERROR] Simulated connection issue at tick {}",
                self.tick
            ));
        }
        self.streaming.clear();
        self.streaming.append(&format!(
            "Last tick: {} @ {:.1}s",
            self.tick, self.cpu_value
        ));
        self.dirty = true;
    }

    fn tick(&mut self, elapsed_secs: u64) {
        self.tick += 1;
        if self.paused {
            return;
        }
        if elapsed_secs.is_multiple_of(2) {
            self.cpu_value = 30.0 + (self.tick % 50) as f32;
            self.gauge.set_value(self.cpu_value as f64);
        }
        if elapsed_secs.is_multiple_of(5) {
            let mut pairs = BTreeMap::new();
            pairs.insert(
                "Memory".to_string(),
                format!("{:.1} GB", 8.0 + (self.tick % 10) as f32 * 0.1),
            );
            pairs.insert(
                "Disk".to_string(),
                format!("{}%", 40 + (self.tick % 20) as u32),
            );
            self.kv_grid.set_pairs(pairs);
        }
        if elapsed_secs.is_multiple_of(10) {
            let status_text = if self.tick % 20 < 15 { "OK" } else { "WARNING" };
            self.status.set_status(status_text);
        }
        if elapsed_secs.is_multiple_of(3) {
            self.log_viewer.clear();
            self.log_viewer
                .append_line(&format!("[INFO] Tick {}", self.tick));
            self.log_viewer.append_line(&format!(
                "[WARN] Load: {:.1}",
                1.5 + (self.tick % 10) as f32 * 0.1
            ));
        }
        self.streaming.append(&format!("T:{} ", self.tick));
        self.dirty = true;
    }
}

impl Default for CommandBindings {
    fn default() -> Self {
        Self::new(Arc::new(AtomicBool::new(false)), Theme::nord())
    }
}

fn blit_plane(src: &Plane, dst: &mut Plane, dst_x: u16, dst_y: u16) {
    for y in 0..src.height {
        for x in 0..src.width {
            let src_idx = (y * src.width + x) as usize;
            if src_idx >= src.cells.len() {
                continue;
            }
            if src.cells[src_idx].transparent {
                continue;
            }
            let dx = dst_x + x;
            let dy = dst_y + y;
            if dx >= dst.width || dy >= dst.height {
                continue;
            }
            let dst_idx = dy as usize * dst.width as usize + dx as usize;
            if dst_idx < dst.cells.len() {
                dst.cells[dst_idx] = src.cells[src_idx];
            }
        }
    }
}

fn render_card(p: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: &Theme) {
    let w = w as usize;
    let h = h as usize;
    let px = x as usize;
    let py = y as usize;

    if w < 2 || h < 2 {
        return;
    }

    // corners
    p.cells[py * p.width as usize + px] = Cell {
        char: '╭',
        fg: t.outline,
        bg: t.surface,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    p.cells[py * p.width as usize + px + w - 1] = Cell {
        char: '╮',
        fg: t.outline,
        bg: t.surface,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    p.cells[(py + h - 1) * p.width as usize + px] = Cell {
        char: '╰',
        fg: t.outline,
        bg: t.surface,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };
    p.cells[(py + h - 1) * p.width as usize + px + w - 1] = Cell {
        char: '╯',
        fg: t.outline,
        bg: t.surface,
        style: Styles::empty(),
        transparent: false,
        skip: false,
    };

    // horizontal edges
    for ix in 1..w - 1 {
        p.cells[py * p.width as usize + px + ix] = Cell {
            char: '─',
            fg: t.outline,
            bg: t.surface,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        p.cells[(py + h - 1) * p.width as usize + px + ix] = Cell {
            char: '─',
            fg: t.outline,
            bg: t.surface,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }

    // vertical edges
    for iy in 1..h - 1 {
        p.cells[(py + iy) * p.width as usize + px] = Cell {
            char: '│',
            fg: t.outline,
            bg: t.surface,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
        p.cells[(py + iy) * p.width as usize + px + w - 1] = Cell {
            char: '│',
            fg: t.outline,
            bg: t.surface,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }

    // fill interior
    for iy in 1..h - 1 {
        for ix in 1..w - 1 {
            p.cells[(py + iy) * p.width as usize + px + ix] = Cell {
                char: ' ',
                fg: t.fg,
                bg: t.surface,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }
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
        self.dirty = true;
    }
    fn z_index(&self) -> u16 {
        10
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
        self.gauge.on_theme_change(theme);
        self.kv_grid.on_theme_change(theme);
        self.status.on_theme_change(theme);
        self.log_viewer.on_theme_change(theme);
        self.streaming.on_theme_change(theme);
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

        // ── Header bar ──
        let header = " 󰔟 Command Bindings ";
        for (i, c) in header.chars().enumerate().take(w - 4) {
            p.cells[w + 1 + i] = Cell {
                char: c,
                fg: t.fg_on_accent,
                bg: t.primary,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        for x in (1 + header.len() + 1)..(w - 1) {
            p.cells[w + x] = Cell {
                char: '─',
                fg: t.primary,
                bg: t.primary,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
        }

        // ── Card: CPU Gauge (top-left) ──
        render_card(&mut p, 2, 3, 24, 8, t);
        let card_title = " 󰓃 CPU ";
        for (i, c) in card_title.chars().enumerate() {
            p.cells[3 * w + 3 + i] = Cell {
                char: c,
                fg: t.primary,
                bg: t.surface,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        let gauge_area = Rect::new(3, 5, 22, 5);
        let gp = self.gauge.render(gauge_area);
        blit_plane(&gp, &mut p, 3, 5);

        // ── Card: Status (top-right) ──
        render_card(&mut p, 28, 3, 24, 8, t);
        let status_title = " 󰀄 Connection ";
        for (i, c) in status_title.chars().enumerate() {
            p.cells[3 * w + 29 + i] = Cell {
                char: c,
                fg: t.primary,
                bg: t.surface,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        let status_area = Rect::new(29, 5, 22, 5);
        let sp = self.status.render(status_area);
        blit_plane(&sp, &mut p, 29, 5);

        // ── Card: Metrics (middle) ──
        render_card(&mut p, 54, 3, 24, 8, t);
        let metrics_title = " 󰕙 System Metrics ";
        for (i, c) in metrics_title.chars().enumerate() {
            p.cells[3 * w + 55 + i] = Cell {
                char: c,
                fg: t.primary,
                bg: t.surface,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        let kv_area = Rect::new(55, 5, 22, 5);
        let kvp = self.kv_grid.render(kv_area);
        blit_plane(&kvp, &mut p, 55, 5);

        // ── Card: Log (bottom-left) ──
        render_card(&mut p, 2, 12, 38, 10, t);
        let log_title = " 󰑊 Activity Log ";
        for (i, c) in log_title.chars().enumerate() {
            p.cells[12 * w + 3 + i] = Cell {
                char: c,
                fg: t.primary,
                bg: t.surface,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        let log_area = Rect::new(3, 14, 36, 7);
        let lp = self.log_viewer.render(log_area);
        blit_plane(&lp, &mut p, 3, 14);

        // ── Card: Streaming (bottom-right) ──
        render_card(&mut p, 42, 12, 36, 10, t);
        let stream_title = " 󰅐 Live Stream ";
        for (i, c) in stream_title.chars().enumerate() {
            p.cells[12 * w + 43 + i] = Cell {
                char: c,
                fg: t.primary,
                bg: t.surface,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }
        let stream_area = Rect::new(43, 14, 34, 7);
        let streamp = self.streaming.render(stream_area);
        blit_plane(&streamp, &mut p, 43, 14);

        // ── Status bar ──
        let auto_str = if self.paused {
            "⏸ PAUSED"
        } else {
            "▶ RUNNING"
        };
        let status = format!(
            "  {}  |  tick: {}  |  {}: theme | {}: help | {}: dismiss | {}: quit",
            auto_str, self.tick,
            self.keybindings.display(actions::THEME).unwrap_or("t"),
            self.keybindings.display(actions::HELP).unwrap_or("?"),
            self.keybindings.display(actions::BACK).unwrap_or("esc"),
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
            let help_w = 36;
            let help_h = 12;
            let help_x = (w as i32 - help_w as i32) / 2;
            let help_y = (h as i32 - help_h as i32) / 2;

            // backdrop
            for sy in 0..help_h {
                for sx in 0..help_w {
                    let px = help_x + sx as i32;
                    let py = help_y + sy as i32;
                    if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                        let idx = py as usize * w + px as usize;
                        p.cells[idx] = Cell {
                            char: ' ',
                            fg: t.fg,
                            bg: t.surface,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }

            // rounded border
            let hx = help_x as usize;
            let hy = help_y as usize;
            p.cells[hy * w + hx] = Cell {
                char: '╭',
                fg: t.outline,
                bg: t.surface,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
            p.cells[hy * w + hx + help_w - 1] = Cell {
                char: '╮',
                fg: t.outline,
                bg: t.surface,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
            p.cells[(hy + help_h - 1) * w + hx] = Cell {
                char: '╰',
                fg: t.outline,
                bg: t.surface,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
            p.cells[(hy + help_h - 1) * w + hx + help_w - 1] = Cell {
                char: '╯',
                fg: t.outline,
                bg: t.surface,
                style: Styles::empty(),
                transparent: false,
                skip: false,
            };
            for ix in 1..help_w - 1 {
                p.cells[hy * w + hx + ix] = Cell {
                    char: '─',
                    fg: t.outline,
                    bg: t.surface,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
                p.cells[(hy + help_h - 1) * w + hx + ix] = Cell {
                    char: '─',
                    fg: t.outline,
                    bg: t.surface,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
            for iy in 1..help_h - 1 {
                p.cells[(hy + iy) * w + hx] = Cell {
                    char: '│',
                    fg: t.outline,
                    bg: t.surface,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
                p.cells[(hy + iy) * w + hx + help_w - 1] = Cell {
                    char: '│',
                    fg: t.outline,
                    bg: t.surface,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }

            // title
            let title = " Command Bindings Help ";
            for (i, c) in title.chars().enumerate() {
                let idx = (hy + 1) * w + hx + 1 + i;
                if idx < p.cells.len() {
                    p.cells[idx] = Cell {
                        char: c,
                        fg: t.primary,
                        bg: t.surface,
                        style: Styles::BOLD,
                        transparent: false,
                        skip: false,
                    };
                }
            }

            // shortcuts
            let shortcuts = [
                ("s", "Refresh all commands"),
                ("p", "Pause/Resume auto-refresh"),
                (self.keybindings.display(actions::THEME).unwrap_or("t"), "Cycle theme"),
                (self.keybindings.display(actions::HELP).unwrap_or("?"), "Toggle this help"),
                (self.keybindings.display(actions::QUIT).unwrap_or("q"), "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let y_off = 3 + i;
                let key_str = format!(" {} ", key);
                for (j, c) in key_str.chars().enumerate() {
                    let idx = (hy + y_off) * w + hx + 2 + j;
                    if idx < p.cells.len() {
                        p.cells[idx] = Cell {
                            char: c,
                            fg: t.primary,
                            bg: t.surface,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (hy + y_off) * w + hx + 2 + key_str.len() + 1 + j;
                    if idx < p.cells.len() {
                        p.cells[idx] = Cell {
                            char: c,
                            fg: t.fg,
                            bg: t.surface,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }

            // hint
            let hint = format!(" Press {} or {} to close ",
                self.keybindings.display(actions::HELP).unwrap_or("?"),
                self.keybindings.display(actions::BACK).unwrap_or("Esc"),
            );
            for (i, c) in hint.chars().enumerate() {
                let idx = (hy + help_h - 2) * w + hx + (help_w - hint.len()) / 2 + i;
                if idx < p.cells.len() {
                    p.cells[idx] = Cell {
                        char: c,
                        fg: t.fg_muted,
                        bg: t.surface,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            let themes = Theme::all();
            let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
            let next = themes[(idx + 1) % themes.len()].clone();
            self.on_theme_change(&next);
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.show_help && self.keybindings.matches(actions::BACK, &key) {
            self.show_help = false;
            self.dirty = true;
            return true;
        }
        match key.code {
            KeyCode::Char('s') if key.modifiers.is_empty() => {
                self.refresh_all();
                true
            }
            KeyCode::Char('p') if key.modifiers.is_empty() => {
                self.paused = !self.paused;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }
}

struct InputRouter {
    view: Rc<RefCell<CommandBindings>>,
}

impl Widget for InputRouter {
    fn id(&self) -> WidgetId { WidgetId::new(9999) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { Rect::new(0, 0, 0, 0) }
    fn set_area(&mut self, _area: Rect) {}
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { false }
    fn render(&self, area: Rect) -> Plane { Plane::new(0, area.width, area.height) }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.view.borrow_mut().handle_key(key)
    }
    fn current_theme(&self) -> Option<Theme> {
        Some(self.view.borrow().theme.clone())
    }
}

fn main() -> std::io::Result<()> {
    println!("Command Bindings — s=refresh all, p=pause, Ctrl+C=quit\nStarting...");
    std::thread::sleep(Duration::from_millis(500));

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());
    let env_theme = Theme::from_env_or(Theme::nord());

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let view = Rc::new(RefCell::new(CommandBindings::new(quit_check.clone(), env_theme.clone())));
    view.borrow_mut().keybindings = keybindings;
    view.borrow_mut().refresh_all();
    let view_for_tick = Rc::clone(&view);

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let mut app = App::new()?
        .title("Command Bindings")
        .fps(20)
        .tick_interval(1000)
        .theme(env_theme.clone());

    // Add InputRouter widget so framework can detect theme changes via current_theme()
    app.add_widget(
        Box::new(InputRouter { view: Rc::clone(&view) }),
        Rect::new(0, 0, w, h),
    );

    app.on_tick(move |ctx, tick| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }
        let mut view = view_for_tick.borrow_mut();
        view.tick(tick);
        view.mark_dirty();
        let (w, h) = ctx.compositor().size();
        if view.area.width != w || view.area.height != h {
            view.set_area(Rect::new(0, 0, w, h));
        }
        if view.needs_render() {
            let area = view.area;
            let plane = view.render(area);
            view.clear_dirty();
            drop(view);
            ctx.add_plane(plane);
        }
    })
    .run(|_| {})
}
