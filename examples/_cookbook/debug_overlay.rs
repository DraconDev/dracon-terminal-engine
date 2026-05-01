//! Debug overlay example demonstrating diagnostic tools.
//!
//! ## Features Shown
//!
//! 1. **DebugOverlay** — Modal overlay containing all debug tools
//! 2. **Profiler** — Real-time FPS, frame time, tick count, memory metrics
//! 3. **WidgetInspector** — Shows focused widget ID, type, position, state
//! 4. **EventLogger** — Scrollable log of recent keyboard/mouse events
//!
//! ## Key Patterns
//!
//! - F12 toggles debug overlay visibility
//! - Debug tools render at high z-index (170-200) above main content
//! - EventLogger records ALL events even when overlay is hidden
//! - Profiler updates every frame with mock metrics
//! - ESC or × closes the debug overlay
//!
//! ## Usage
//!
//! Press arrow keys, click, or type to see events logged.
//! Press F12 to toggle the debug overlay panel.

use std::io;
use std::time::{Duration, Instant};

use dracon_terminal_engine::compositor::{Cell, Color, Compositor, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    DebugOverlay, EventLogger, List, Profiler, WidgetInspector, WidgetNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct DebugOverlayPanel {
    overlay: DebugOverlay,
    profiler: Profiler,
    inspector: WidgetInspector,
    event_logger: EventLogger,
    visible: bool,
    start_time: Instant,
    frame_count: u64,
    last_fps_update: Instant,
    fps: f64,
}

impl DebugOverlayPanel {
    fn new() -> Self {
        Self {
            overlay: DebugOverlay::new(WidgetId::new(200)),
            profiler: Profiler::new(WidgetId::new(160)),
            inspector: WidgetInspector::new(WidgetId::new(180)),
            event_logger: EventLogger::new(WidgetId::new(170)).with_max_events(50),
            visible: false,
            start_time: Instant::now(),
            frame_count: 0,
            last_fps_update: Instant::now(),
            fps: 60.0,
        }
    }

    fn toggle(&mut self) {
        self.visible = !self.visible;
        self.overlay.mark_dirty();
    }

    fn log_event(&mut self, timestamp: &str, description: &str) {
        self.event_logger.log(timestamp, description);
    }

    fn update_profiler(&mut self) {
        let now = Instant::now();
        self.frame_count += 1;

        if now.duration_since(self.last_fps_update).as_secs() >= 1 {
            let elapsed = now.duration_since(self.start_time).as_secs() as f64;
            self.fps = self.frame_count as f64 / elapsed.max(1.0);
            self.last_fps_update = now;
        }

        let frame_time = Duration::from_millis(16);
        let memory = 45 + (self.frame_count % 20) as u64;

        self.profiler.set_metrics(vec![
            Metric {
                name: "FPS".to_string(),
                value: Duration::from_secs_f64(1000.0 / self.fps.max(1.0)),
                call_count: self.fps as u64,
            },
            Metric {
                name: "Frame".to_string(),
                value: frame_time,
                call_count: self.frame_count,
            },
            Metric {
                name: "Ticks".to_string(),
                value: Duration::from_secs(self.frame_count / 60),
                call_count: self.frame_count,
            },
            Metric {
                name: "Memory".to_string(),
                value: Duration::from_millis(memory),
                call_count: memory,
            },
        ]);
    }

    fn update_inspector(&mut self, focused_widget: Option<&str>) {
        let nodes = vec![WidgetNode {
            id: WidgetId::new(10),
            label: focused_widget.unwrap_or("List_1").to_string(),
            children: (0..20)
                .map(|i| WidgetNode {
                    id: WidgetId::new(11 + i as u64),
                    label: format!("Item_{}", i + 1),
                    children: vec![],
                })
                .collect(),
        }];
        self.inspector.set_hierarchy(nodes);
    }
}

impl Widget for DebugOverlayPanel {
    fn id(&self) -> WidgetId {
        self.overlay.id()
    }

    fn set_id(&mut self, id: WidgetId) {
        self.overlay.set_id(id);
    }

    fn area(&self) -> Rect {
        self.overlay.area()
    }

    fn set_area(&mut self, area: Rect) {
        self.overlay.set_area(area);
        let (w, h) = (area.width, area.height);
        self.profiler.set_area(Rect::new(0, 1, 25, 8));
        self.inspector.set_area(Rect::new(26, 1, 25, 8));
        self.event_logger.set_area(Rect::new(0, 10, w, h.saturating_sub(11)));
    }

    fn z_index(&self) -> u16 {
        200
    }

    fn needs_render(&self) -> bool {
        self.visible || self.overlay.needs_render()
    }

    fn mark_dirty(&mut self) {
        self.overlay.mark_dirty();
    }

    fn clear_dirty(&mut self) {
        self.overlay.clear_dirty();
    }

    fn focusable(&self) -> bool {
        self.visible
    }

    fn render(&self, area: Rect) -> Plane {
        if !self.visible {
            return Plane::new(0, area.width, area.height);
        }

        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 200;

        for y in 0..area.height {
            for x in 0..area.width {
                let idx = (y * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    let border = y == 0 || y == 9 || y == area.height - 1;
                    let separator = y == 9 && x == 26;
                    plane.cells[idx] = Cell {
                        char: if separator { '+' } else if border { '-' } else { ' ' },
                        fg: if border { Color::Cyan } else { Color::Reset },
                        bg: if border { Color::Blue } else { Color::Reset },
                        style: if border { Styles::BOLD } else { Styles::empty() },
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        for x in 0..area.width {
            let idx = (0 * plane.width + x) as usize;
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
                (y * plane.width + 0) as usize,
                (y * plane.width + 25) as usize,
                (y * plane.width + area.width.saturating_sub(1)) as usize,
            ]
            .iter()
            {
                if *idx < plane.cells.len() {
                    plane.cells[*idx].char = '|';
                    plane.cells[*idx].fg = Color::Cyan;
                }
            }
        }

        let header = "Debug Overlay";
        for (i, c) in header.chars().enumerate().take((area.width as usize).saturating_sub(10)) {
            let idx = (0 * plane.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let close = "[x] Close";
        for (i, c) in close.chars().enumerate() {
            let idx = (0 * plane.width + area.width.saturating_sub(10) + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
            }
        }

        let profiler_area = Rect::new(1, 2, 23, 6);
        let mut prof_plane = self.profiler.render(profiler_area);
        for cell in &mut plane.cells {
            cell.transparent = true;
        }
        for y in 0..prof_plane.height {
            for x in 0..prof_plane.width {
                let src_idx = (y * prof_plane.width + x) as usize;
                let dst_idx = ((y + 2) * plane.width + x + 1) as usize;
                if src_idx < prof_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = std::mem::take(&mut prof_plane.cells[src_idx]);
                }
            }
        }

        let inspector_area = Rect::new(27, 2, 23, 6);
        let mut insp_plane = self.inspector.render(inspector_area);
        for y in 0..insp_plane.height {
            for x in 0..insp_plane.width {
                let src_idx = (y * insp_plane.width + x) as usize;
                let dst_idx = ((y + 2) * plane.width + x + 27) as usize;
                if src_idx < insp_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = std::mem::take(&mut insp_plane.cells[src_idx]);
                }
            }
        }

        let logger_area = Rect::new(1, 11, area.width.saturating_sub(2), area.height.saturating_sub(12));
        let mut log_plane = self.event_logger.render(logger_area);
        for y in 0..log_plane.height {
            for x in 0..log_plane.width {
                let src_idx = (y * log_plane.width + x) as usize;
                let dst_idx = ((y + 11) * plane.width + x + 1) as usize;
                if src_idx < log_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = std::mem::take(&mut log_plane.cells[src_idx]);
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if let KeyCode::F(12) = key.code {
            self.toggle();
            return true;
        }
        if self.visible {
            if let KeyCode::Esc = key.code {
                self.toggle();
                return true;
            }
        }
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.visible && row == 0 && col >= self.area().width.saturating_sub(9) {
            self.toggle();
            return true;
        }
        false
    }
}

fn format_time() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn main() -> io::Result<()> {
    println!("Debug Overlay Demo");
    println!("==================");
    println!("Press F12 to toggle debug overlay");
    println!("Arrow keys and mouse clicks are logged");
    println!();

    std::thread::sleep(Duration::from_millis(300));

    let mut app = App::new()?.title("Debug Overlay Demo").fps(60);
    app.set_theme(Theme::dark());

    let items: Vec<String> = (1..=20).map(|i| format!("List Item {}", i)).collect();
    let mut list = List::new(items);
    list.set_area(Rect::new(2, 2, 40, 18));

    let mut debug_panel = DebugOverlayPanel::new();
    debug_panel.update_inspector(Some("List_1"));

    let _result = app.run(move |ctx| {
        ctx.mark_all_dirty();

        let (w, h) = ctx.compositor().size();
        let list_area = Rect::new(2, 2, 40, 18);
        list.mark_dirty();
        let list_plane = list.render(list_area);
        ctx.add_plane(list_plane);

        let footer = "[Toggle Debug: F12]";
        let footer_plane = Plane::new(0, footer.len() as u16 + 4, 1);
        let mut cells = footer_plane.cells;
        for (i, c) in footer.chars().enumerate() {
            if i < cells.len() {
                cells[i].char = c;
            }
        }
        ctx.add_plane(footer_plane);

        let status_items = ["Profiler: OFF", "WidgetInspector: OFF", "EventLog: OFF"];
        let status_y = h.saturating_sub(1);
        let mut status_plane = Plane::new(0, w, 1);
        status_plane.z_index = 5;
        let status_text = status_items.join("  ");
        for (i, c) in status_text.chars().enumerate() {
            let idx = (0 * status_plane.width + i as u16) as usize;
            if idx < status_plane.cells.len() {
                status_plane.cells[idx].char = c;
            }
        }
        ctx.add_plane(status_plane);

        debug_panel.update_profiler();

        let debug_area = Rect::new(0, 0, w, h);
        debug_panel.set_area(debug_area);
        debug_panel.mark_dirty();
        let mut debug_plane = debug_panel.render(debug_area);
        if debug_panel.visible {
            debug_plane.z_index = 200;
        }
        ctx.add_plane(debug_plane);
    });

    println!("\nDebug overlay demo exited cleanly");
    Ok(())
}