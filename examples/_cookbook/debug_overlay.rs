#![allow(missing_docs)]
//! Debug overlay example demonstrating diagnostic tools.
//!
//! ## Features Shown
//!
//! 1. **DebugOverlay**  -  Modal overlay containing all debug tools
//! 2. **Profiler**  -  Real-time FPS, frame time, tick count, memory metrics
//! 3. **WidgetInspector**  -  Shows focused widget ID, type, position, state
//! 4. **EventLogger**  -  Scrollable log of recent keyboard/mouse events
//!
//! ## Key Patterns
//!
//! - F12 toggles debug overlay visibility
//! - Debug tools render at high z-index (170-200) above main content
//! - EventLogger records ALL events even when overlay is hidden
//! - Profiler updates every frame with mock metrics
//! - ESC or x closes the debug overlay
//!
//! ## Usage
//!
//! Press arrow keys, click, or type to see events logged.
//! Press F12 to toggle the debug overlay panel.

use std::cell::{Cell as StdCell, RefCell};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{EventLogger, Profiler, WidgetInspector, WidgetNode};
use ratatui::layout::Rect;

use std::os::fd::AsFd;

struct DebugOverlayPanel {
    id: WidgetId,
    area: StdCell<Rect>,
    profiler: RefCell<Profiler>,
    inspector: RefCell<WidgetInspector>,
    event_logger: RefCell<EventLogger>,
    visible: bool,
    theme: Theme,
    show_help: bool,
    should_quit: Arc<AtomicBool>,
    keybindings: KeybindingSet,
    frame_count: StdCell<u64>,
    start_time: Instant,
    dirty: StdCell<bool>,
}

impl DebugOverlayPanel {
    fn new(id: WidgetId, theme: Theme, should_quit: Arc<AtomicBool>) -> Self {
        let mut inspector = WidgetInspector::new(WidgetId::new(180));
        let hierarchy = vec![
            WidgetNode {
                id: WidgetId::new(1),
                label: "App".to_string(),
                children: vec![
                    WidgetNode {
                        id: WidgetId::new(160),
                        label: "Profiler".to_string(),
                        children: vec![],
                    },
                    WidgetNode {
                        id: WidgetId::new(180),
                        label: "WidgetInspector".to_string(),
                        children: vec![],
                    },
                    WidgetNode {
                        id: WidgetId::new(170),
                        label: "EventLogger".to_string(),
                        children: vec![],
                    },
                ],
            },
        ];
        inspector.set_hierarchy(hierarchy);

        Self {
            id,
            area: StdCell::new(Rect::new(0, 0, 80, 24)),
            profiler: RefCell::new(Profiler::new(WidgetId::new(160))),
            inspector: RefCell::new(inspector),
            event_logger: RefCell::new(EventLogger::new(WidgetId::new(170))),
            visible: false,
            theme,
            show_help: false,
            should_quit,
            keybindings: KeybindingSet::default(),
            frame_count: StdCell::new(0),
            start_time: Instant::now(),
            dirty: StdCell::new(true),
        }
    }

    fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        // Propagate theme to all child widgets
        self.profiler.borrow_mut().on_theme_change(&self.theme);
        self.inspector.borrow_mut().on_theme_change(&self.theme);
        self.event_logger.borrow_mut().on_theme_change(&self.theme);
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let hw = 44u16.min(area.width.saturating_sub(4));
        let hh = 12u16.min(area.height.saturating_sub(4));
        let hx = (area.width.saturating_sub(hw)) / 2;
        let hy = (area.height.saturating_sub(hh)) / 2;

        for ry in hy..hy + hh {
            for rx in hx..hx + hw {
                let idx = (ry * plane.width + rx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.surface_elevated;
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
        for (ch, cx, cy) in corners.iter() {
            let idx = (cy * plane.width + cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = *ch;
                plane.cells[idx].fg = self.theme.outline;
                plane.cells[idx].bg = self.theme.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
        for x in hx + 1..hx + hw - 1 {
            let top = (hy * plane.width + x) as usize;
            let bot = ((hy + hh - 1) * plane.width + x) as usize;
            if top < plane.cells.len() {
                plane.cells[top].char = '─';
                plane.cells[top].fg = self.theme.outline;
                plane.cells[top].bg = self.theme.surface_elevated;
            }
            if bot < plane.cells.len() {
                plane.cells[bot].char = '─';
                plane.cells[bot].fg = self.theme.outline;
                plane.cells[bot].bg = self.theme.surface_elevated;
            }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * plane.width + hx) as usize;
            let right = (y * plane.width + hx + hw - 1) as usize;
            if left < plane.cells.len() {
                plane.cells[left].char = '│';
                plane.cells[left].fg = self.theme.outline;
                plane.cells[left].bg = self.theme.surface_elevated;
            }
            if right < plane.cells.len() {
                plane.cells[right].char = '│';
                plane.cells[right].fg = self.theme.outline;
                plane.cells[right].bg = self.theme.surface_elevated;
            }
        }

        let title = "Debug Overlay Help";
        let tx = hx + (hw.saturating_sub(title.len() as u16)) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((hy + 1) * plane.width + tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let shortcuts = [
            ("F12", "Toggle overlay"),
            (self.keybindings.display(actions::BACK).unwrap_or("esc"), "Close overlay"),
            (self.keybindings.display(actions::THEME).unwrap_or("ctrl+t"), "Cycle theme"),
            (self.keybindings.display(actions::HELP).unwrap_or("f1"), "Toggle help"),
            (self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q"), "Quit"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            for (j, c) in key.chars().enumerate() {
                let idx = (row * plane.width + hx + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.primary;
                    plane.cells[idx].bg = self.theme.surface_elevated;
                }
            }
            for (j, c) in desc.chars().enumerate() {
                let idx = (row * plane.width + hx + 14 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.fg;
                    plane.cells[idx].bg = self.theme.surface_elevated;
                }
            }
        }
    }
}

impl Widget for DebugOverlayPanel {
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
        self.profiler.borrow_mut().set_area(Rect::new(0, 1, 25, 8));
        self.inspector.borrow_mut().set_area(Rect::new(26, 1, 25, 8));
        self.event_logger
            .borrow_mut()
            .set_area(Rect::new(0, 10, area.width, area.height.saturating_sub(11)));
    }
    fn z_index(&self) -> u16 {
        200
    }
    fn needs_render(&self) -> bool {
        self.dirty.get() || self.visible
    }
    fn mark_dirty(&mut self) {
        self.dirty.set(true);
    }
    fn clear_dirty(&mut self) {
        self.dirty.set(false);
    }
    fn focusable(&self) -> bool {
        true
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.profiler.borrow_mut().on_theme_change(theme);
        self.inspector.borrow_mut().on_theme_change(theme);
        self.event_logger.borrow_mut().on_theme_change(theme);
    }

    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme.clone())
    }

    fn render(&self, area: Rect) -> Plane {
        if !self.visible {
            let mut plane = Plane::new(0, area.width, area.height);
            plane.fill_bg(self.theme.bg);
            for cell in plane.cells.iter_mut() {
                cell.transparent = false;
            }

            let lines = [
                "Debug Overlay Demo",
                "",
                "Press F12 to toggle the debug overlay panel.",
                "",
                &format!(
                    "{}: cycle theme  |  {}: help  |  {}: quit",
                    self.keybindings.display(actions::THEME).unwrap_or("ctrl+t"),
                    self.keybindings.display(actions::HELP).unwrap_or("f1"),
                    self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q"),
                ),
            ];
            let start_y = area.height.saturating_sub(lines.len() as u16) / 2;
            for (i, line) in lines.iter().enumerate() {
                let y = start_y + i as u16;
                let x = (area.width.saturating_sub(line.len() as u16)) / 2;
                for (j, c) in line.chars().enumerate() {
                    let idx = (y * plane.width + x + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = if i == 0 {
                            self.theme.primary
                        } else {
                            self.theme.fg
                        };
                        plane.cells[idx].style = if i == 0 {
                            Styles::BOLD
                        } else {
                            Styles::empty()
                        };
                    }
                }
            }

            if self.show_help {
                self.render_help_overlay(&mut plane, area);
            }
            return plane;
        }

        // Update profiler with mock metrics each frame
        let frame = self.frame_count.get() + 1;
        self.frame_count.set(frame);
        let elapsed = self.start_time.elapsed();
        let variable = ((frame as f64 / 60.0).sin() * 5.0 + 10.0) as u64;
        let metrics = vec![
            dracon_terminal_engine::framework::widgets::Metric {
                name: "FPS".to_string(),
                value: Duration::from_millis(16),
                call_count: frame,
            },
            dracon_terminal_engine::framework::widgets::Metric {
                name: "Render".to_string(),
                value: Duration::from_micros(500 + (frame % 200) * 3),
                call_count: frame,
            },
            dracon_terminal_engine::framework::widgets::Metric {
                name: "Input".to_string(),
                value: Duration::from_micros(120 + (frame % 50) * 2),
                call_count: frame,
            },
            dracon_terminal_engine::framework::widgets::Metric {
                name: "Layout".to_string(),
                value: Duration::from_micros(350),
                call_count: frame,
            },
            dracon_terminal_engine::framework::widgets::Metric {
                name: "Composite".to_string(),
                value: Duration::from_micros(480 + (frame % 100)),
                call_count: frame,
            },
            dracon_terminal_engine::framework::widgets::Metric {
                name: "Memory".to_string(),
                value: Duration::from_millis(variable),
                call_count: 1,
            },
            dracon_terminal_engine::framework::widgets::Metric {
                name: "Uptime".to_string(),
                value: elapsed,
                call_count: 1,
            },
        ];
        self.profiler.borrow_mut().set_metrics(metrics);

        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 200;
        plane.fill_bg(self.theme.bg);
        for cell in plane.cells.iter_mut() {
            cell.transparent = false;
        }

        for y in 0..area.height {
            for x in 0..area.width {
                let idx = (y * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    let border = y == 0 || y == 9 || y == area.height - 1;
                    let separator = y == 9 && x == 26;
                    let bg_color = if border {
                        self.theme.surface_elevated
                    } else {
                        self.theme.bg
                    };
                    plane.cells[idx] = Cell {
                        char: if separator {
                            '+'
                        } else if border {
                            '-'
                        } else {
                            ' '
                        },
                        fg: if border {
                            self.theme.primary
                        } else {
                            self.theme.fg
                        },
                        bg: bg_color,
                        style: if border {
                            Styles::BOLD
                        } else {
                            Styles::empty()
                        },
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '-';
            }
            let idx = (9 * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '-';
            }
            let idx = ((area.height - 1) * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '-';
            }
        }

        for y in 0..area.height {
            for idx in [
                (y * plane.width) as usize,
                (y * plane.width + 25) as usize,
                (y * plane.width + area.width.saturating_sub(1)) as usize,
            ].clone()
            .iter()
            {
                if *idx < plane.cells.len() {
                    plane.cells[*idx].char = '|';
                    plane.cells[*idx].fg = self.theme.primary;
                }
            }
        }

        let header = "Debug Overlay";
        for (i, c) in header
            .chars()
            .enumerate()
            .take((area.width as usize).saturating_sub(10))
        {
            let idx = (2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let close = "[x] Close";
        for (i, c) in close.chars().enumerate() {
            let idx = (plane.width.saturating_sub(10) + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
            }
        }

        let profiler_area = Rect::new(1, 2, 23, 6);
        let prof_plane = self.profiler.borrow().render(profiler_area);
        for y in 0..prof_plane.height {
            for x in 0..prof_plane.width {
                let src_idx = (y * prof_plane.width + x) as usize;
                if prof_plane.cells[src_idx].transparent {
                    continue;
                }
                let dst_idx = ((y + 2) * plane.width + x + 1) as usize;
                if src_idx < prof_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = prof_plane.cells[src_idx];
                }
            }
        }

        let inspector_area = Rect::new(27, 2, 23, 6);
        let insp_plane = self.inspector.borrow().render(inspector_area);
        for y in 0..insp_plane.height {
            for x in 0..insp_plane.width {
                let src_idx = (y * insp_plane.width + x) as usize;
                if insp_plane.cells[src_idx].transparent {
                    continue;
                }
                let dst_idx = ((y + 2) * plane.width + x + 27) as usize;
                if src_idx < insp_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = insp_plane.cells[src_idx];
                }
            }
        }

        let logger_area = Rect::new(
            1,
            11,
            area.width.saturating_sub(2),
            area.height.saturating_sub(12),
        );
        let log_plane = self.event_logger.borrow().render(logger_area);
        for y in 0..log_plane.height {
            for x in 0..log_plane.width {
                let src_idx = (y * log_plane.width + x) as usize;
                if log_plane.cells[src_idx].transparent {
                    continue;
                }
                let dst_idx = ((y + 11) * plane.width + x + 1) as usize;
                if src_idx < log_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = log_plane.cells[src_idx];
                }
            }
        }

        // Status bar
        let status_y = plane.height.saturating_sub(1);
        let hint = format!("F12: toggle | {}: theme | {}: help | {}: dismiss | {}: quit",
            self.keybindings.display(actions::THEME).unwrap_or("ctrl+t"),
            self.keybindings.display(actions::HELP).unwrap_or("f1"),
            self.keybindings.display(actions::BACK).unwrap_or("esc"),
            self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q"),
        );
        for (i, c) in hint
            .chars()
            .take((plane.width as usize).saturating_sub(2))
            .enumerate()
        {
            let idx = (status_y * plane.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
            }
        }

        // Render help overlay if active
        if self.show_help {
            self.render_help_overlay(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let ts = format!("{:?}", Instant::now());
        let desc = format!("Key {:?} mods={:?}", key.code, key.modifiers);
        self.event_logger.borrow_mut().log(&ts, &desc);

        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Handle help overlay first
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty.set(true);
            }
            return true;
        }

        match key.code {
            KeyCode::F(12) => {
                self.toggle();
                self.dirty.set(true);
                true
            }
            _ if self.keybindings.matches(actions::QUIT, &key) => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            _ if self.visible && self.keybindings.matches(actions::BACK, &key) => {
                self.toggle();
                self.dirty.set(true);
                true
            }
            _ if self.keybindings.matches(actions::THEME, &key) => {
                self.cycle_theme();
                self.dirty.set(true);
                true
            }
            _ if self.keybindings.matches(actions::HELP, &key) => {
                self.show_help = true;
                self.dirty.set(true);
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let ts = format!("{:?}", Instant::now());
        let desc = format!("Mouse {:?} at ({},{})", kind, col, row);
        self.event_logger.borrow_mut().log(&ts, &desc);

        if self.visible && row == 0 && col >= self.area().width.saturating_sub(9) {
            self.toggle();
            self.dirty.set(true);
            return true;
        }
        false
    }
}

fn main() -> io::Result<()> {
    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let theme = Theme::from_env_or(Theme::dark());
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let keybindings = KeybindingSet::from_config(&resolve_keybindings());

    let mut debug_panel = DebugOverlayPanel::new(WidgetId::new(42), theme.clone(), should_quit);
    debug_panel.keybindings = keybindings;
    // panel.set_area is called by add_widget which sets area from Rect

    let mut app = App::new()?.title("Debug Overlay Demo").fps(60).theme(theme);
    app.add_widget(Box::new(debug_panel), Rect::new(0, 0, w, h));
    app.run(move |ctx| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
}
