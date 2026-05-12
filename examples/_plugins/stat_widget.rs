// Stat Widget Plugin — Displays system CPU and memory statistics.
//
// This widget is designed to be loaded dynamically as a plugin.
// It displays real-time CPU load percentage and memory usage.

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::plugin::PluginRegistry;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::Cell;
use sysinfo::System;

/// Plugin identifier for registration
pub const STAT_WIDGET_NAME: &str = "stat_widget";

/// Creates a StatWidget factory function for PluginRegistry.
pub fn stat_widget_factory(id: WidgetId, theme: Theme) -> Box<dyn Widget> {
    Box::new(StatWidget::new(id, theme))
}

/// Register this plugin with a registry.
pub fn register(registry: &mut PluginRegistry) {
    let _ = registry.register(STAT_WIDGET_NAME, stat_widget_factory);
}

// ═══════════════════════════════════════════════════════════════════════════════
// STAT WIDGET
// ═══════════════════════════════════════════════════════════════════════════════

/// A widget that displays system statistics (CPU and memory usage).
pub struct StatWidget {
    id: WidgetId,
    area: Cell<Rect>,
    theme: Theme,
    sys: std::cell::RefCell<System>,
}

impl StatWidget {
    /// Creates a new StatWidget with the given ID and theme.
    pub fn new(id: WidgetId, theme: Theme) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            id,
            area: Cell::new(Rect::new(0, 0, 28, 7)),
            theme,
            sys: std::cell::RefCell::new(sys),
        }
    }

    /// Refreshes system stats.
    pub fn refresh(&self) {
        self.sys.borrow_mut().refresh_all();
    }

    /// Gets the current CPU load as a percentage (0.0 - 100.0).
    fn cpu_usage(&self) -> f32 {
        self.sys.borrow().global_cpu_usage()
    }

    /// Gets the used memory in bytes.
    fn used_memory(&self) -> u64 {
        self.sys.borrow().used_memory()
    }

    /// Gets the total memory in bytes.
    fn total_memory(&self) -> u64 {
        self.sys.borrow().total_memory()
    }

    /// Gets memory usage as a percentage (0.0 - 100.0).
    fn memory_usage(&self) -> f32 {
        let total = self.total_memory();
        if total == 0 {
            return 0.0;
        }
        (self.used_memory() as f32 / total as f32) * 100.0
    }
}

impl Widget for StatWidget {
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
        let mut plane = Plane::new(0, 28, 7);
        plane.fill_bg(t.bg);

        // Refresh stats
        self.sys.borrow_mut().refresh_cpu();
        let cpu = self.sys.borrow().global_cpu_usage();
        let mem_pct = self.memory_usage();
        let mem_used = self.used_memory() / (1024 * 1024); // MB
        let mem_total = self.total_memory() / (1024 * 1024); // MB

        // Draw border
        for col in 0..28 {
            plane.cells[col as usize].char = '─';
            plane.cells[col as usize].fg = t.outline;
            plane.cells[140 + col as usize].char = '─';
            plane.cells[140 + col as usize].fg = t.outline;
        }
        for row in 0..7u16 {
            plane.cells[(row * 28) as usize].char = '│';
            plane.cells[(row * 28) as usize].fg = t.outline;
            plane.cells[(row * 28 + 27) as usize].char = '│';
            plane.cells[(row * 28 + 27) as usize].fg = t.outline;
        }
        // Corners
        plane.cells[0].char = '╭';
        plane.cells[27].char = '╮';
        plane.cells[140].char = '╰';
        plane.cells[167].char = '╯';

        // Title
        let title = "System Stats";
        for (i, c) in title.chars().enumerate() {
            plane.cells[29 + i].char = c;
            plane.cells[29 + i].fg = t.primary;
            plane.cells[29 + i].style = Styles::BOLD;
        }

        // CPU section
        let cpu_label = "CPU";
        for (i, c) in cpu_label.chars().enumerate() {
            plane.cells[31 + i].char = c;
            plane.cells[31 + i].fg = t.secondary;
        }

        // CPU bar
        let cpu_bar_width = 14;
        let cpu_filled = ((cpu / 100.0) * cpu_bar_width as f32) as usize;
        for i in 0..cpu_bar_width {
            let idx = 34 + i;
            if i < cpu_filled {
                plane.cells[idx].char = '█';
                plane.cells[idx].fg = if cpu > 80.0 {
                    t.error
                } else if cpu > 50.0 {
                    t.warning
                } else {
                    t.success
                };
            } else {
                plane.cells[idx].char = '░';
                plane.cells[idx].fg = t.fg_subtle;
            }
        }

        // CPU percentage text
        let cpu_str = format!("{:>5.1}%", cpu);
        for (i, c) in cpu_str.chars().enumerate() {
            plane.cells[49 + i].char = c;
            plane.cells[49 + i].fg = t.fg;
            plane.cells[49 + i].style = Styles::BOLD;
        }

        // Memory section
        let mem_label = "MEM";
        for (i, c) in mem_label.chars().enumerate() {
            plane.cells[84 + i].char = c;
            plane.cells[84 + i].fg = t.secondary;
        }

        // Memory bar
        let mem_bar_width = 14;
        let mem_filled = ((mem_pct / 100.0) * mem_bar_width as f32) as usize;
        for i in 0..mem_bar_width {
            let idx = 87 + i;
            if i < mem_filled {
                plane.cells[idx].char = '█';
                plane.cells[idx].fg = if mem_pct > 90.0 {
                    t.error
                } else if mem_pct > 70.0 {
                    t.warning
                } else {
                    t.info
                };
            } else {
                plane.cells[idx].char = '░';
                plane.cells[idx].fg = t.fg_subtle;
            }
        }

        // Memory text
        let mem_str = format!("{:>5.1}%", mem_pct);
        for (i, c) in mem_str.chars().enumerate() {
            plane.cells[102 + i].char = c;
            plane.cells[102 + i].fg = t.fg;
            plane.cells[102 + i].style = Styles::BOLD;
        }

        // Memory details (used / total)
        let mem_detail = format!("{} / {} MB", mem_used, mem_total);
        for (i, c) in mem_detail.chars().enumerate() {
            if 107 + i < plane.cells.len() {
                plane.cells[107 + i].char = c;
                plane.cells[107 + i].fg = t.fg_subtle;
            }
        }

        plane
    }

    fn handle_key(&mut self, _key: KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
    }
}
