#![allow(missing_docs)]
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{EventLogger, Profiler, WidgetInspector};
use ratatui::layout::Rect;

use std::os::fd::AsFd;

struct DebugOverlayPanel {
    id: WidgetId,
    profiler: Profiler,
    inspector: WidgetInspector,
    event_logger: EventLogger,
    visible: bool,
    theme: Theme,
}

impl DebugOverlayPanel {
    fn new(id: WidgetId, theme: Theme) -> Self {
        Self {
            id,
            profiler: Profiler::new(WidgetId::new(160)),
            inspector: WidgetInspector::new(WidgetId::new(180)),
            event_logger: EventLogger::new(WidgetId::new(170)),
            visible: false,
            theme,
        }
    }

    fn toggle(&mut self) {
        self.visible = !self.visible;
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
        Rect::new(0, 0, 80, 24)
    }
    fn set_area(&mut self, area: Rect) {
        self.profiler.set_area(Rect::new(0, 1, 25, 8));
        self.inspector.set_area(Rect::new(26, 1, 25, 8));
        self.event_logger
            .set_area(Rect::new(0, 10, area.width, area.height.saturating_sub(11)));
    }
    fn z_index(&self) -> u16 {
        200
    }
    fn needs_render(&self) -> bool {
        self.visible
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
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
                            Color::Reset
                        },
                        bg: if border {
                            self.theme.surface_elevated
                        } else {
                            Color::Reset
                        },
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
            ]
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
        let prof_plane = self.profiler.render(profiler_area);
        for y in 0..prof_plane.height {
            for x in 0..prof_plane.width {
                let src_idx = (y * prof_plane.width + x) as usize;
                if prof_plane.cells[src_idx].transparent {
                    continue;
                }
                let dst_idx = ((y + 2) * plane.width + x + 1) as usize;
                if src_idx < prof_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = prof_plane.cells[src_idx].clone();
                }
            }
        }

        let inspector_area = Rect::new(27, 2, 23, 6);
        let insp_plane = self.inspector.render(inspector_area);
        for y in 0..insp_plane.height {
            for x in 0..insp_plane.width {
                let src_idx = (y * insp_plane.width + x) as usize;
                if insp_plane.cells[src_idx].transparent {
                    continue;
                }
                let dst_idx = ((y + 2) * plane.width + x + 27) as usize;
                if src_idx < insp_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = insp_plane.cells[src_idx].clone();
                }
            }
        }

        let logger_area = Rect::new(
            1,
            11,
            area.width.saturating_sub(2),
            area.height.saturating_sub(12),
        );
        let log_plane = self.event_logger.render(logger_area);
        for y in 0..log_plane.height {
            for x in 0..log_plane.width {
                let src_idx = (y * log_plane.width + x) as usize;
                if log_plane.cells[src_idx].transparent {
                    continue;
                }
                let dst_idx = ((y + 11) * plane.width + x + 1) as usize;
                if src_idx < log_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = log_plane.cells[src_idx].clone();
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

    fn handle_mouse(&mut self, _kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.visible && row == 0 && col >= self.area().width.saturating_sub(9) {
            self.toggle();
            return true;
        }
        false
    }
}

fn main() -> io::Result<()> {
    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let theme = Theme::dark();
    let mut debug_panel = DebugOverlayPanel::new(WidgetId::new(42), theme);
    debug_panel.set_area(Rect::new(0, 0, w, h));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?.title("Debug Overlay Demo").fps(60).theme(theme);
    app.add_widget(Box::new(debug_panel), Rect::new(0, 0, w, h));
    app = app
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        });
    app.run(|_ctx| {})
}
