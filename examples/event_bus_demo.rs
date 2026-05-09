//! Event Bus Demo — Decoupled widget communication.
//!
//! Shows how widgets communicate via events without direct references.
//! Two widgets (counter + logger) share state through the event bus.
//!
//! Controls:
//!   ↑/↓ or +/-  — adjust counter
//!   l            — log a message
//!   c            — clear log
//!   t            — cycle theme
//!   ?            — toggle help
//!   q            — quit

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════════
// APP EVENTS
// ═══════════════════════════════════════════════════════════════════════════════

// App events that widgets can publish/subscribe to via EventBus
#[allow(dead_code)]
#[derive(Clone, Debug)]
enum AppEvent {
    CounterChanged(i32),
    MessageLogged(String),
    LogCleared,
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP STATE
// ═══════════════════════════════════════════════════════════════════════════════

struct EventBusApp {
    counter_value: i32,
    messages: Vec<String>,
    theme: Theme,
    show_help: bool,
    should_quit: Arc<AtomicBool>,
    dirty: bool,
    last_event: Option<String>,
}

impl EventBusApp {
    fn new(should_quit: Arc<AtomicBool>) -> Self {
        Self {
            counter_value: 0,
            messages: vec!["Event bus demo started".into()],
            theme: Theme::nord(),
            show_help: false,
            should_quit,
            dirty: true,
            last_event: None,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::nord(), Theme::cyberpunk(), Theme::dracula(),
            Theme::catppuccin_mocha(), Theme::gruvbox_dark(), Theme::tokyo_night(),
        ];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.dirty = true;
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('t') => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                self.dirty = true;
                true
            }
            KeyCode::Up | KeyCode::Char('+') | KeyCode::Char('=') => {
                self.counter_value += 1;
                self.last_event = Some(format!("CounterChanged({})", self.counter_value));
                self.messages.push(format!("Counter: {}", self.counter_value));
                self.dirty = true;
                true
            }
            KeyCode::Down | KeyCode::Char('-') => {
                self.counter_value -= 1;
                self.last_event = Some(format!("CounterChanged({})", self.counter_value));
                self.messages.push(format!("Counter: {}", self.counter_value));
                self.dirty = true;
                true
            }
            KeyCode::Char('l') => {
                self.messages.push("Manual log entry".into());
                self.last_event = Some("MessageLogged".into());
                self.dirty = true;
                true
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => {
                self.messages.clear();
                self.messages.push("Log cleared".into());
                self.last_event = Some("LogCleared".into());
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);

        // Fill background
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.transparent = false;
        }

        let half_w = area.width / 2;

        // Left panel: Counter
        self.render_counter(&mut plane, Rect::new(1, 1, half_w - 2, area.height - 2), t);

        // Right panel: Event Log
        self.render_log(&mut plane, Rect::new(half_w + 1, 1, half_w - 2, area.height - 2), t);

        // Status bar
        let status = "↑/↓: counter | l: log | c: clear | t: theme | ?: help | Esc: dismiss | q: quit";
        let sx = (area.width as usize - status.len().min(area.width as usize)) / 2;
        let sy = area.height - 1;
        for (i, c) in status.chars().take(area.width as usize).enumerate() {
            let idx = (sy * area.width + sx as u16 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.bg;
            }
        }

        // Help overlay
        if self.show_help {
            self.render_help(&mut plane, area, t);
        }

        plane
    }

    fn render_counter(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        // Border
        for x in area.x..area.x + area.width {
            let top = (area.y * plane.width + x) as usize;
            let bot = ((area.y + area.height - 1) * plane.width + x) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for y in area.y..area.y + area.height {
            let left = (y * plane.width + area.x) as usize;
            let right = (y * plane.width + area.x + area.width - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }

        // Title
        let title = " Counter ";
        let tx = area.x + (area.width - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (area.y * plane.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Value
        let val_str = format!("{}", self.counter_value);
        let vx = area.x + (area.width - val_str.len() as u16) / 2;
        let vy = area.y + area.height / 2;
        for (i, c) in val_str.chars().enumerate() {
            let idx = (vy * plane.width + vx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].bg = t.surface;
            }
        }

        // Last event
        if let Some(ref evt) = self.last_event {
            let evy = vy + 2;
            let ev_text = format!("Event: {}", evt);
            let evx = area.x + 2;
            for (i, c) in ev_text.chars().take((area.width - 4) as usize).enumerate() {
                let idx = (evy * plane.width + evx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.info;
                }
            }
        }
    }

    fn render_log(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        // Border
        for x in area.x..area.x + area.width {
            let top = (area.y * plane.width + x) as usize;
            let bot = ((area.y + area.height - 1) * plane.width + x) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for y in area.y..area.y + area.height {
            let left = (y * plane.width + area.x) as usize;
            let right = (y * plane.width + area.x + area.width - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }

        // Title
        let title = " Event Log ";
        let tx = area.x + (area.width - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (area.y * plane.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Messages
        let visible = (area.height - 2) as usize;
        let start = self.messages.len().saturating_sub(visible);
        for (i, msg) in self.messages.iter().skip(start).take(visible).enumerate() {
            let y = area.y + 1 + i as u16;
            let truncated = if msg.len() > (area.width - 4) as usize {
                format!("{}..", &msg[..(area.width - 6) as usize])
            } else {
                msg.clone()
            };
            for (j, c) in truncated.chars().enumerate() {
                let idx = (y * plane.width + area.x + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                }
            }
        }
    }

    fn render_help(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let hw = 50u16.min(area.width.saturating_sub(4));
        let hh = 14u16.min(area.height.saturating_sub(4));
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
        let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
        for (ch, cx, cy) in corners.iter() {
            let idx = (cy * area.width + cx) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
        }

        // Title
        let title = "Event Bus Demo Help";
        let tx = hx + (hw - title.len() as u16) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (hy as usize + 1) * area.width as usize + tx as usize + i;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Shortcuts
        let shortcuts = [
            ("↑/↓ or +/-", "Adjust counter"),
            ("l", "Log a message"),
            ("c", "Clear log"),
            ("t", "Cycle theme"),
            ("?", "Toggle help"),
            ("Esc", "Dismiss help"),
            ("q", "Quit"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            for (j, c) in key.chars().enumerate() {
                let idx = (row * area.width + hx + 2 + j as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
            }
            for (j, c) in desc.chars().enumerate() {
                let idx = (row * area.width + hx + 16 + j as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INPUT ROUTER
// ═══════════════════════════════════════════════════════════════════════════════

struct InputRouter {
    app: Rc<RefCell<EventBusApp>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for InputRouter {
    fn id(&self) -> WidgetId { self.id }
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); }
    fn needs_render(&self) -> bool { false }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn z_index(&self) -> u16 { 0 }
    fn render(&self, _area: Rect) -> Plane { Plane::new(0, 0, 0) }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        self.app.borrow_mut().handle_key(key)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Event Bus Demo | ↑/↓: counter | l: log | c: clear | t: theme | ?: help | Esc: dismiss | q: quit");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let app = Rc::new(RefCell::new(EventBusApp::new(should_quit)));
    let app_for_router = Rc::clone(&app);

    let mut app_ctx = App::new()?
        .title("Event Bus Demo")
        .fps(30);

    let router = InputRouter {
        app: app_for_router,
        id: WidgetId::new(100),
        area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
    };
    app_ctx.add_widget(Box::new(router), Rect::new(0, 0, 80, 24));

    let _ = app_ctx
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
                return;
            }

            let mut app = app.borrow_mut();
            if app.dirty {
                let (w, h) = ctx.compositor().size();
                let plane = app.render(Rect::new(0, 0, w, h));
                ctx.add_plane(plane);
                app.dirty = false;
            }
        })
        .run(|_| {});

    Ok(())
}
