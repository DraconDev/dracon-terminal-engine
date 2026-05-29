#![allow(missing_docs)]
//! Cyberpunk Dashboard — Animated system dashboard with charts, gauges, and sparklines.
//!
//! A live-updating dashboard themed in cyberpunk style with sin-wave data simulation,
//! memory gauge, network sparkline, and alert overlay.

use std::cell::RefCell;
use std::io::Result;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::StatusBar;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct CyberpunkDashboard {
    id: WidgetId,
    area: Rect,
    theme: Theme,
    dirty: bool,
    tick: u64,
    data: Vec<f64>,
    spark_data: Vec<u64>,
    alert_visible: bool,
    show_help: bool,
    paused: bool,
    status_bar: StatusBar,
    keybindings: KeybindingSet,
}

const DATA_LEN: usize = 40;

impl CyberpunkDashboard {
    fn new(theme: Theme) -> Self {
        let status_bar = StatusBar::new(WidgetId::new(60))
            .add_segment(
                dracon_terminal_engine::framework::widgets::StatusSegment::new(
                    "Ctrl+P: pause | Ctrl+T: theme | Space: alert | F1: help | Ctrl+Q: quit",
                ),
            )
            .with_theme(theme.clone());

        Self {
            id: WidgetId::new(1),
            area: Rect::new(0, 0, 80, 24),
            theme,
            dirty: true,
            tick: 0,
            data: vec![0.0; DATA_LEN],
            spark_data: vec![0; DATA_LEN],
            alert_visible: false,
            show_help: false,
            paused: false,
            status_bar,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn update(&mut self) {
        if self.paused {
            return;
        }
        self.tick += 1;
        self.data.remove(0);
        let val = (self.tick as f64 * 0.15).sin().abs() * 100.0;
        self.data.push(val);
        self.spark_data.remove(0);
        self.spark_data.push(val as u64);
        self.dirty = true;
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.status_bar.on_theme_change(&self.theme);
        self.dirty = true;
    }

    fn render_header(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let title = " DRACON CYBERPUNK SYSTEM v1.0 ";
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
            let px = area.width as usize - label.len() - 2;
            for (i, c) in label.chars().enumerate() {
                let idx = px + i;
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
    }

    fn render_chart(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let chart_x = 1u16;
        let chart_y = 2u16;
        let chart_w = (area.width / 2).saturating_sub(2);
        let chart_h = area.height.saturating_sub(5);

        for y in chart_y..chart_y + chart_h {
            for x in chart_x..chart_x + chart_w {
                let idx = (y as usize) * plane.width as usize + x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                }
            }
        }

        let title = "CORE 01";
        for (i, c) in title.chars().enumerate() {
            let idx = (chart_y as usize) * plane.width as usize + chart_x as usize + 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.secondary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let mid_y = chart_y + chart_h / 2;
        for (i, &val) in self.data.iter().enumerate() {
            let x = chart_x + 1 + i as u16;
            if x >= chart_x + chart_w - 1 {
                break;
            }
            let bar_h = (val / 100.0 * (chart_h as f64 - 4.0)).max(1.0) as u16;
            for dy in 0..bar_h {
                let y = mid_y + chart_h / 2 - 2 - dy;
                if y > chart_y && y < chart_y + chart_h - 1 {
                    let idx = (y as usize) * plane.width as usize + x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = if dy == bar_h - 1 { '▔' } else { '█' };
                        plane.cells[idx].fg = t.secondary;
                        plane.cells[idx].bg = t.surface;
                    }
                }
            }
        }
    }

    fn render_gauge(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let gx = area.width / 2 + 1;
        let gy = 2u16;
        let gw = area.width / 2 - 2;
        let _gh = 3u16;

        let title = "MEMORY";
        for (i, c) in title.chars().enumerate() {
            let idx = (gy as usize) * plane.width as usize + gx as usize + 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.success;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let pct = self.tick % 100;
        let filled = (pct as f32 / 100.0 * (gw as f32 - 4.0)) as u16;
        let bar_y = gy + 1;
        for x in 0..gw.saturating_sub(2) {
            let idx = (bar_y as usize) * plane.width as usize + (gx + 1 + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                if x < filled {
                    plane.cells[idx].char = '█';
                    plane.cells[idx].fg = t.success;
                } else {
                    plane.cells[idx].char = '░';
                    plane.cells[idx].fg = t.fg_muted;
                }
            }
        }

        let pct_str = format!("{:>3}%", pct);
        for (i, c) in pct_str.chars().enumerate() {
            let idx = (bar_y as usize) * plane.width as usize + (gx + gw - 5) as usize + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg;
            }
        }
    }

    fn render_sparkline(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let sx = area.width / 2 + 1;
        let sy = 6u16;
        let sw = area.width / 2 - 2;
        let _sh = 4u16;

        let title = "NET I/O";
        for (i, c) in title.chars().enumerate() {
            let idx = (sy as usize) * plane.width as usize + sx as usize + 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.warning;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let max_val = self.spark_data.iter().copied().max().unwrap_or(1).max(1);
        for (i, &val) in self.spark_data.iter().enumerate() {
            let x = sx + 1 + i as u16;
            if x >= sx + sw - 1 {
                break;
            }
            let bar_idx = (val as f32 / max_val as f32 * (bars.len() - 1) as f32).round() as usize;
            let bar_idx = bar_idx.min(bars.len() - 1);
            let idx = ((sy + 1) as usize) * plane.width as usize + x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = bars[bar_idx];
                plane.cells[idx].fg = t.warning;
                plane.cells[idx].bg = t.surface;
            }
        }
    }

    fn render_alert(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let w = 30u16.min(area.width.saturating_sub(4));
        let h = 7u16.min(area.height.saturating_sub(4));
        let hx = (area.width - w) / 2;
        let hy = (area.height - h) / 2;

        for y in hy..hy + h {
            for x in hx..hx + w {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.error;
                    plane.cells[idx].fg = t.fg_on_accent;
                    plane.cells[idx].transparent = false;
                    let is_border = y == hy || y == hy + h - 1 || x == hx || x == hx + w - 1;
                    if is_border {
                        plane.cells[idx].char = '#';
                    } else {
                        plane.cells[idx].char = ' ';
                    }
                }
            }
        }

        let msg = "SYSTEM BREACH DETECTED";
        let sub = "[Space] TO DISMISS";
        let msg_x = hx + (w - msg.len() as u16) / 2;
        let sub_x = hx + (w - sub.len() as u16) / 2;
        for (i, c) in msg.chars().enumerate() {
            let idx = ((hy + 2) * area.width + msg_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].style = Styles::BOLD;
            }
        }
        for (i, c) in sub.chars().enumerate() {
            let idx = ((hy + 4) * area.width + sub_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].style = Styles::empty();
            }
        }
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 42u16.min(area.width.saturating_sub(4));
        let hh = 13u16.min(area.height.saturating_sub(4));
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

        let title = "Cyberpunk Dashboard Help";
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
            ("Space", "Toggle alert"),
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

impl Widget for CyberpunkDashboard {
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
        self.status_bar.on_theme_change(theme);
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        self.render_header(&mut plane, area);
        self.render_chart(&mut plane, area);
        self.render_gauge(&mut plane, area);
        self.render_sparkline(&mut plane, area);

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

        if self.alert_visible {
            self.render_alert(&mut plane, area);
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

        if self.alert_visible {
            if key.code == KeyCode::Char(' ') && key.modifiers.is_empty() {
                self.alert_visible = false;
                self.dirty = true;
                return true;
            }
            return true;
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
        if key.code == KeyCode::Char(' ') && key.modifiers.is_empty() {
            self.alert_visible = true;
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
    target: Rc<RefCell<CyberpunkDashboard>>,
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
    let dash = Rc::new(RefCell::new(CyberpunkDashboard::new(env_theme.clone())));
    let dash_tick = Rc::clone(&dash);
    let dash_router = Rc::clone(&dash);
    let dash_input = Rc::clone(&dash);

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?
        .title("Cyberpunk Dashboard")
        .fps(30)
        .set_theme(env_theme.clone());

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
        d.update();
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
