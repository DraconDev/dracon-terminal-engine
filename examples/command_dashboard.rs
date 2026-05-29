#![allow(missing_docs)]
//! Command Dashboard — Auto-refreshing system gauges with BoundCommand.
//!
//! Demonstrates binding CLI commands to widgets with auto-refresh,
//! theme cycling, pause/resume, and help overlay.

use std::cell::RefCell;
use std::io::Result;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::command::{BoundCommand, OutputParser};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Gauge, KeyValueGrid, StatusBadge, StatusBar};
use dracon_terminal_engine::input::event::{KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct CommandDashboard {
    id: WidgetId,
    area: Rect,
    theme: Theme,
    dirty: bool,
    cpu_gauge: Gauge,
    mem_gauge: Gauge,
    disk_gauge: Gauge,
    kv_grid: KeyValueGrid,
    status: StatusBadge,
    status_bar: StatusBar,
    show_help: bool,
    paused: bool,
    keybindings: KeybindingSet,
}

impl CommandDashboard {
    fn new(theme: Theme) -> Self {
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
        let kv_grid = KeyValueGrid::new()
            .separator("  ")
            .bind_command(BoundCommand::new("uname -snr").refresh(0));
        let status = StatusBadge::new(WidgetId::new(50))
            .with_status("OK")
            .with_label("System");
        let status_bar = StatusBar::new(WidgetId::new(60))
            .add_segment(
                dracon_terminal_engine::framework::widgets::StatusSegment::new(
                    "Ctrl+P: pause | Ctrl+T: theme | F1: help | Esc: dismiss | Ctrl+Q: quit",
                ),
            )
            .with_theme(theme.clone());

        Self {
            id: WidgetId::new(1),
            area: Rect::new(0, 0, 80, 24),
            theme,
            dirty: true,
            cpu_gauge,
            mem_gauge,
            disk_gauge,
            kv_grid,
            status,
            status_bar,
            show_help: false,
            paused: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.cpu_gauge.on_theme_change(&self.theme);
        self.mem_gauge.on_theme_change(&self.theme);
        self.disk_gauge.on_theme_change(&self.theme);
        self.kv_grid.on_theme_change(&self.theme);
        self.status.on_theme_change(&self.theme);
        self.status_bar.on_theme_change(&self.theme);
        self.dirty = true;
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 42u16.min(area.width.saturating_sub(4));
        let hh = 12u16.min(area.height.saturating_sub(4));
        let hx = (area.width - hw) / 2;
        let hy = (area.height - hh) / 2;

        for y in hy..hy + hh {
            for x in hx..hx + hw {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let corners = [
            ('╭', hx, hy),
            ('╮', hx + hw - 1, hy),
            ('╰', hx, hy + hh - 1),
            ('╯', hx + hw - 1, hy + hh - 1),
        ];
        for (ch, cx, cy) in &corners {
            let idx = (*cy * area.width + *cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = *ch;
                plane.cells[idx].fg = t.outline;
            }
        }
        for x in hx + 1..hx + hw - 1 {
            let top = (hy * area.width + x) as usize;
            let bot = ((hy + hh - 1) * area.width + x) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = t.outline;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = t.outline;
            }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * area.width + hx) as usize;
            let right = (y * area.width + hx + hw - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = t.outline;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = t.outline;
            }
        }

        let title = "Command Dashboard Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let shortcuts = [
            ("Ctrl+P", "Pause / resume"),
            ("Ctrl+T", "Cycle theme"),
            ("F1", "Toggle help"),
            ("Esc", "Dismiss help"),
            ("Ctrl+Q", "Quit"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            for (j, c) in key.chars().enumerate() {
                let idx = (row * area.width + hx + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                }
            }
            for (j, c) in desc.chars().enumerate() {
                let idx = (row * area.width + hx + 14 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                }
            }
        }
    }
}

impl Widget for CommandDashboard {
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
    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
    fn focusable(&self) -> bool {
        true
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.cpu_gauge.on_theme_change(theme);
        self.mem_gauge.on_theme_change(theme);
        self.disk_gauge.on_theme_change(theme);
        self.kv_grid.on_theme_change(theme);
        self.status.on_theme_change(theme);
        self.status_bar.on_theme_change(theme);
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(t.bg);

        let title = " Command Dashboard ";
        for (i, c) in title.chars().enumerate() {
            let idx = 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: t.fg_on_accent,
                    bg: t.primary,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        if self.paused {
            let label = " PAUSED ";
            for (i, c) in label.chars().enumerate() {
                let idx = area.width as usize - 10 + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: t.fg_on_accent,
                        bg: t.warning,
                        style: Styles::BOLD,
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        let gauge_h = 3u16;
        let gauge_w = (area.width / 3).saturating_sub(1);
        let areas = [
            Rect::new(0, 1, gauge_w, gauge_h),
            Rect::new(gauge_w + 1, 1, gauge_w, gauge_h),
            Rect::new((gauge_w + 1) * 2, 1, gauge_w, gauge_h),
        ];
        let gauges: [&dyn Widget; 3] = [&self.cpu_gauge, &self.mem_gauge, &self.disk_gauge];
        for (i, gauge) in gauges.iter().enumerate() {
            let g_plane = gauge.render(areas[i]);
            for (ci, c) in g_plane.cells.iter().enumerate() {
                if c.transparent || c.char == '\0' {
                    continue;
                }
                let row = ci / g_plane.width as usize;
                let col = ci % g_plane.width as usize;
                let dy = areas[i].y as usize + row;
                let dx = areas[i].x as usize + col;
                if dy < area.height as usize && dx < area.width as usize {
                    let idx = dy * area.width as usize + dx;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = *c;
                    }
                }
            }
        }

        let kv_area = Rect::new(0, gauge_h + 1, area.width.saturating_sub(20), 6);
        let kv_plane = self.kv_grid.render(kv_area);
        for (ci, c) in kv_plane.cells.iter().enumerate() {
            if c.transparent || c.char == '\0' {
                continue;
            }
            let row = ci / kv_plane.width as usize;
            let col = ci % kv_plane.width as usize;
            let dy = kv_area.y as usize + row;
            let dx = kv_area.x as usize + col;
            if dy < area.height as usize && dx < area.width as usize {
                let idx = dy * area.width as usize + dx;
                if idx < plane.cells.len() {
                    plane.cells[idx] = *c;
                }
            }
        }

        let sb_area = Rect::new(area.width.saturating_sub(20), gauge_h + 1, 20, 1);
        let sb_plane = self.status.render(sb_area);
        for (ci, c) in sb_plane.cells.iter().enumerate() {
            if c.transparent || c.char == '\0' {
                continue;
            }
            let col = ci % sb_plane.width as usize;
            let dy = sb_area.y as usize;
            let dx = sb_area.x as usize + col;
            if dy < area.height as usize && dx < area.width as usize {
                let idx = dy * area.width as usize + dx;
                if idx < plane.cells.len() {
                    plane.cells[idx] = *c;
                }
            }
        }

        let bar_area = Rect::new(0, area.height.saturating_sub(1), area.width, 1);
        let bar_plane = self.status_bar.render(bar_area);
        for (ci, c) in bar_plane.cells.iter().enumerate() {
            if c.transparent || c.char == '\0' {
                continue;
            }
            let col = ci % bar_plane.width as usize;
            if col < area.width as usize {
                let idx = (area.height as usize - 1) * area.width as usize + col;
                if idx < plane.cells.len() {
                    plane.cells[idx] = *c;
                }
            }
        }

        if self.show_help {
            self.render_help_overlay(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if self.keybindings.matches(actions::PAUSE, &key) {
            self.paused = !self.paused;
            self.dirty = true;
            return true;
        }

        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        matches!(kind, MouseEventKind::ScrollDown | MouseEventKind::ScrollUp)
    }
}

struct InputRouter {
    target: Rc<RefCell<CommandDashboard>>,
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
    std::thread::sleep(std::time::Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let env_theme = Theme::from_env_or(Theme::nord());
    let dash = Rc::new(RefCell::new(CommandDashboard::new(env_theme.clone())));
    let dash_tick = Rc::clone(&dash);
    let dash_router = Rc::clone(&dash);
    let dash_input = Rc::clone(&dash);

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?
        .title("Command Dashboard")
        .fps(30)
        .theme(env_theme.clone());

    let kb = dash_input.borrow().keybindings.clone();

    let router = InputRouter {
        target: dash_router,
        id: WidgetId::new(100),
        area: Rect::new(0, 0, w, h),
    };
    app.add_widget(Box::new(router), Rect::new(0, 0, w, h));

    app.on_input(move |key| {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        let mut d = dash_input.borrow_mut();
        if kb.matches(actions::QUIT, &key) {
            should_quit.store(true, Ordering::SeqCst);
            true
        } else {
            d.handle_key(key)
        }
    })
    .on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }
        let mut d = dash_tick.borrow_mut();
        let (w, h) = ctx.compositor().size();
        if d.area.width != w || d.area.height != h {
            d.set_area(Rect::new(0, 0, w, h));
        }
        if d.needs_render() {
            ctx.add_plane(d.render(d.area));
            d.clear_dirty();
        }
    })
    .run(|_| {})
}
