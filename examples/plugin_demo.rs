#![allow(missing_docs)]
//! Plugin Demo — Demonstrates dynamic widget loading via PluginRegistry.
//!
//! Shows how to:
//! - Define a custom widget with factory function
//! - Register it with PluginRegistry
//! - Dynamically create widgets by name
//! - Use PluginRegistry in an App
//!
//! Controls:
//!   t          — cycle theme
//!   ?          — toggle help
//!   q          — quit

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::plugin::PluginRegistry;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTOM WIDGET: ClockWidget
// ═══════════════════════════════════════════════════════════════════════════════

struct ClockWidget {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    theme: Theme,
    use_24h: bool,
}

impl ClockWidget {
    fn new(id: WidgetId, theme: Theme) -> Self {
        Self {
            id,
            area: std::cell::Cell::new(Rect::new(0, 0, 20, 3)),
            theme,
            use_24h: true,
        }
    }
}

impl Widget for ClockWidget {
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
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        false
    }
    fn render(&self, _area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, 20, 3);
        plane.fill_bg(t.bg);

        // Draw border
        for col in 0..20 {
            plane.cells[col as usize].char = '─';
            plane.cells[col as usize].fg = t.outline;
            plane.cells[40 + col as usize].char = '─';
            plane.cells[40 + col as usize].fg = t.outline;
        }
        for row in 0..3u16 {
            plane.cells[(row * 20) as usize].char = '│';
            plane.cells[(row * 20) as usize].fg = t.outline;
            plane.cells[(row * 20 + 19) as usize].char = '│';
            plane.cells[(row * 20 + 19) as usize].fg = t.outline;
        }
        // Corners
        plane.cells[0].char = '╭';
        plane.cells[19].char = '╮';
        plane.cells[40].char = '╰';
        plane.cells[59].char = '╯';

        // Title
        let title = "Clock";
        for (i, c) in title.chars().enumerate() {
            plane.cells[21 + i].char = c;
            plane.cells[21 + i].fg = t.primary;
            plane.cells[21 + i].style = Styles::BOLD;
        }

        // Time display
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        let hours = (secs / 3600) % 24;
        let mins = (secs / 60) % 60;
        let s = secs % 60;
        let time_str = if self.use_24h {
            format!("{:02}:{:02}:{:02}", hours, mins, s)
        } else {
            let h12 = if hours.is_multiple_of(12) { 12 } else { hours % 12 };
            let ampm = if hours >= 12 { "PM" } else { "AM" };
            format!("{:>2}:{:02}:{:02} {}", h12, mins, s, ampm)
        };

        for (i, c) in time_str.chars().enumerate() {
            plane.cells[42 + i].char = c;
            plane.cells[42 + i].fg = t.success;
            plane.cells[42 + i].style = Styles::BOLD;
        }

        plane
    }
    fn handle_key(&mut self, _key: KeyEvent) -> bool {
        false
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        if kind == MouseEventKind::Down(MouseButton::Left) {
            self.use_24h = !self.use_24h;
            return true;
        }
        false
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
    }
}

// Factory function for ClockWidget
fn clock_factory(id: WidgetId, theme: Theme) -> Box<dyn Widget> {
    Box::new(ClockWidget::new(id, theme))
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTOM WIDGET: CounterWidget
// ═══════════════════════════════════════════════════════════════════════════════

struct CounterWidget {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    theme: Theme,
    count: i32,
}

impl CounterWidget {
    fn new(id: WidgetId, theme: Theme) -> Self {
        Self {
            id,
            area: std::cell::Cell::new(Rect::new(0, 0, 15, 3)),
            theme,
            count: 0,
        }
    }
}

impl Widget for CounterWidget {
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
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        false
    }
    fn render(&self, _area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, 15, 3);
        plane.fill_bg(t.bg);

        // Border
        for col in 0..15 {
            plane.cells[col as usize].char = '─';
            plane.cells[col as usize].fg = t.outline;
            plane.cells[30 + col as usize].char = '─';
            plane.cells[30 + col as usize].fg = t.outline;
        }
        for row in 0..3u16 {
            plane.cells[(row * 15) as usize].char = '│';
            plane.cells[(row * 15) as usize].fg = t.outline;
            plane.cells[(row * 15 + 14) as usize].char = '│';
            plane.cells[(row * 15 + 14) as usize].fg = t.outline;
        }
        plane.cells[0].char = '╭';
        plane.cells[14].char = '╮';
        plane.cells[30].char = '╰';
        plane.cells[44].char = '╯';

        // Title
        let title = "Counter";
        for (i, c) in title.chars().enumerate() {
            plane.cells[16 + i].char = c;
            plane.cells[16 + i].fg = t.primary;
            plane.cells[16 + i].style = Styles::BOLD;
        }

        // Reset indicator
        plane.cells[25].char = '[';
        plane.cells[25].fg = t.warning;
        plane.cells[26].char = 'R';
        plane.cells[26].fg = t.warning;
        plane.cells[26].style = Styles::BOLD;
        plane.cells[27].char = ']';
        plane.cells[27].fg = t.warning;

        // Count value
        let count_str = format!("{}", self.count);
        let x = 7 - count_str.len() as u16 / 2;
        for (i, c) in count_str.chars().enumerate() {
            plane.cells[(32 + x + i as u16) as usize].char = c;
            plane.cells[(32 + x + i as u16) as usize].fg = t.info;
            plane.cells[(32 + x + i as u16) as usize].style = Styles::BOLD;
            plane.cells[(32 + x + i as u16) as usize].bg = t.surface;
        }

        // +/- controls
        plane.cells[32].char = '-';
        plane.cells[32].fg = t.primary;
        plane.cells[32].style = Styles::BOLD;
        plane.cells[42].char = '+';
        plane.cells[42].fg = t.primary;
        plane.cells[42].style = Styles::BOLD;

        plane
    }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Char('+') | KeyCode::Right => {
                self.count += 1;
                true
            }
            KeyCode::Char('-') | KeyCode::Left => {
                self.count -= 1;
                true
            }
            _ => false,
        }
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if kind != MouseEventKind::Down(MouseButton::Left) {
            return false;
        }
        if row == 1 && (9..=13).contains(&col) {
            self.count = 0;
            return true;
        }
        if row == 2 && (1..=4).contains(&col) {
            self.count -= 1;
            return true;
        }
        if row == 2 && (10..=13).contains(&col) {
            self.count += 1;
            return true;
        }
        false
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
    }
}

fn counter_factory(id: WidgetId, theme: Theme) -> Box<dyn Widget> {
    Box::new(CounterWidget::new(id, theme))
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP STATE
// ═══════════════════════════════════════════════════════════════════════════════

struct PluginDemoState {
    registry: PluginRegistry,
    clock: Box<dyn Widget>,
    counter: Box<dyn Widget>,
    show_help: bool,
    theme: Theme,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
    keybindings: KeybindingSet,
}

impl PluginDemoState {
    fn new(should_quit: Arc<AtomicBool>, keybindings: KeybindingSet) -> Self {
        let mut registry = PluginRegistry::new();

        // Register custom widgets
        registry.register("clock", clock_factory);
        registry.register("counter", counter_factory);

        // Create instances via registry
        let clock = registry
            .create("clock", WidgetId::new(1), Theme::default())
            .unwrap();
        let counter = registry
            .create("counter", WidgetId::new(2), Theme::default())
            .unwrap();

        Self {
            registry,
            clock,
            counter,
            show_help: false,
            theme: Theme::nord(),
            dirty: true,
            should_quit,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::nord(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::gruvbox_dark(),
            Theme::tokyo_night(),
        ];
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.clock.on_theme_change(&self.theme);
        self.counter.on_theme_change(&self.theme);
        self.dirty = true;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INPUT ROUTER
// ═══════════════════════════════════════════════════════════════════════════════

struct InputRouter {
    state: Rc<RefCell<PluginDemoState>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for InputRouter {
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
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let mut state = self.state.borrow_mut();
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if state.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    state.show_help = false;
                    state.dirty = true;
                    return true;
                }
                _ => return true,
            }
        }

        match key.code {
            KeyCode::Char('q') => {
                state.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('t') => {
                state.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                state.show_help = !state.show_help;
                state.dirty = true;
                true
            }
            _ => state.counter.handle_key(key),
        }
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let mut state = self.state.borrow_mut();

        let clock_area = state.clock.area();
        if col >= clock_area.x && col < clock_area.x + clock_area.width
            && row >= clock_area.y && row < clock_area.y + clock_area.height
        {
            let rel_col = col - clock_area.x;
            let rel_row = row - clock_area.y;
            if state.clock.handle_mouse(kind, rel_col, rel_row) {
                state.dirty = true;
                return true;
            }
        }

        let counter_area = state.counter.area();
        if col >= counter_area.x && col < counter_area.x + counter_area.width
            && row >= counter_area.y && row < counter_area.y + counter_area.height
        {
            let rel_col = col - counter_area.x;
            let rel_row = row - counter_area.y;
            if state.counter.handle_mouse(kind, rel_col, rel_row) {
                state.dirty = true;
                return true;
            }
        }

        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELP OVERLAY
// ═══════════════════════════════════════════════════════════════════════════════

fn render_help(plane: &mut Plane, area: Rect, t: &Theme) {
    let hw = 36u16.min(area.width.saturating_sub(4));
    let hh = 13u16.min(area.height.saturating_sub(4));
    let hx = (area.width - hw) / 2;
    let hy = (area.height - hh) / 2;

    // Background
    for y in hy..hy + hh {
        for x in hx..hx + hw {
            let idx = (y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }

    // Border
    let corners = [
        ('╭', hx, hy),
        ('╮', hx + hw - 1, hy),
        ('╰', hx, hy + hh - 1),
        ('╯', hx + hw - 1, hy + hh - 1),
    ];
    for (ch, cx, cy) in corners.iter() {
        let idx = (cy * area.width + cx) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = *ch;
            plane.cells[idx].fg = t.outline;
        }
    }
    for x in hx + 1..hx + hw - 1 {
        let top_idx = (hy * area.width + x) as usize;
        let bot_idx = ((hy + hh - 1) * area.width + x) as usize;
        if top_idx < plane.cells.len() {
            plane.cells[top_idx].char = '─';
            plane.cells[top_idx].fg = t.outline;
        }
        if bot_idx < plane.cells.len() {
            plane.cells[bot_idx].char = '─';
            plane.cells[bot_idx].fg = t.outline;
        }
    }
    for y in hy + 1..hy + hh - 1 {
        let left_idx = (y * area.width + hx) as usize;
        let right_idx = (y * area.width + hx + hw - 1) as usize;
        if left_idx < plane.cells.len() {
            plane.cells[left_idx].char = '│';
            plane.cells[left_idx].fg = t.outline;
        }
        if right_idx < plane.cells.len() {
            plane.cells[right_idx].char = '│';
            plane.cells[right_idx].fg = t.outline;
        }
    }

    // Title
    let title = "Plugin Demo Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    for (i, c) in title.chars().enumerate() {
        let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].char = c;
            plane.cells[idx].fg = t.primary;
            plane.cells[idx].style = Styles::BOLD;
        }
    }

    // Shortcuts
    let shortcuts = [
        ("+/-", "Adjust counter"),
        ("←/→", "Adjust counter"),
        ("Click", "Toggle clock format"),
        ("Click +/-", "Adjust counter"),
        ("Click [R]", "Reset counter"),
        ("t", "Cycle theme"),
        ("?", "Toggle help"),
        ("Esc", "Dismiss help"),
        ("q", "Quit"),
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

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> io::Result<()> {
    println!("Plugin Demo — Clock and Counter widgets loaded via PluginRegistry");
    println!("+/- or ←/→ to adjust counter | t: theme | ?: help | Esc: dismiss | q: quit");
    std::thread::sleep(Duration::from_millis(300));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let state = Rc::new(RefCell::new(PluginDemoState::new(should_quit)));
    let state_for_tick = Rc::clone(&state);
    let state_for_input = Rc::clone(&state);

    let mut app = App::new()?.title("Plugin Demo").fps(30).theme(Theme::nord());

    let router = InputRouter {
        state: state_for_input,
        id: WidgetId::new(100),
        area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
    };
    app.add_widget(Box::new(router), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }
        let mut state = state_for_tick.borrow_mut();
        let (w, h) = ctx.compositor().size();

        if state.dirty {
            let mut plane = Plane::new(0, w, h);
            plane.fill_bg(state.theme.bg);

            // Render header
            let title = "Plugin Registry Demo";
            for (i, c) in title.chars().enumerate() {
                if i < plane.cells.len() {
                    plane.cells[i].char = c;
                    plane.cells[i].fg = state.theme.fg_on_accent;
                    plane.cells[i].bg = state.theme.primary;
                    plane.cells[i].style = Styles::BOLD;
                }
            }

            // Show registered widget names
            let registered = state.registry.list();
            let reg_text = format!("Registered: {}", registered.join(", "));
            for (i, c) in reg_text.chars().enumerate() {
                let idx = (w as usize + i).min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = c;
                plane.cells[idx].fg = state.theme.secondary;
            }

            // Render widgets
            let clock_area = Rect::new(2, 3, 20, 3);
            let counter_area = Rect::new(25, 3, 15, 3);

            state.clock.set_area(clock_area);
            state.counter.set_area(counter_area);

            let clock_plane = state.clock.render(clock_area);
            let counter_plane = state.counter.render(counter_area);

            ctx.add_plane(clock_plane);
            ctx.add_plane(counter_plane);

            // Status bar
            let status_base = ((h - 1) * w) as usize;
            let hint = "t: theme | ?: help | Esc: dismiss | q: quit";
            let hint_x = (w as usize).saturating_sub(hint.len() + 2);
            for (i, c) in hint.chars().enumerate() {
                let idx = status_base + hint_x + i;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = state.theme.fg_subtle;
                    plane.cells[idx].bg = state.theme.surface;
                }
            }

            ctx.add_plane(plane);
            state.dirty = false;
        }

        // Help overlay
        if state.show_help {
            let mut plane = Plane::new(0, w, h);
            plane.fill_bg(state.theme.bg);
            render_help(&mut plane, Rect::new(0, 0, w, h), &state.theme);
            ctx.add_plane(plane);
        }
    })
    .run(|_| {})?;

    println!("\nPlugin demo exited cleanly");
    Ok(())
}