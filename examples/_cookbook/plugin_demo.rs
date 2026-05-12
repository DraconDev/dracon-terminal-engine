//! Plugin system example — demonstrates dynamic widget registration via PluginRegistry.
//!
//! This example shows how to:
//! - Define a custom widget implementing the `Widget` trait
//! - Register it with `PluginRegistry` by name
//! - Instantiate widgets dynamically by name (simulates loading from a plugin)
//! - Use the registry pattern for extensible, hot-reloadable widgets
//!
//! Run with:
//!   cargo run --example plugin_demo

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::plugin::{PluginRegistry, WidgetFactory};
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;
use std::sync::{Arc, RwLock};

// ─────────────────────────────────────────────────────────────────────────────
// 1. Plugin: StatWidget
// ─────────────────────────────────────────────────────────────────────────────

/// A plugin-style widget that displays a labeled metric value.
/// This would be compiled into a separate .so plugin in production.
struct StatWidget {
    id: WidgetId,
    theme: Theme,
    label: String,
    value: String,
    accent: Color,
}

impl StatWidget {
    fn new(id: WidgetId, theme: Theme, label: &str, value: &str) -> Self {
        Self {
            id,
            theme,
            label: label.to_string(),
            value: value.to_string(),
            accent: theme.primary,
        }
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
        Rect::default()
    }
    fn set_area(&mut self, _area: Rect) {}
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(self.id.0 as u16, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let t = &self.theme;
        let label = &self.label;
        let value = &self.value;

        // Top border
        plane.put_str(0, 0, &format!("╭─ {} ─╮", label.chars().take(area.width as usize - 8).collect::<String>()));

        // Fill label (dim, muted)
        for (i, c) in label.chars().enumerate().take((area.width - 4) as usize) {
            let idx = (2 * area.width + 2 + i) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        // Value (primary, bold)
        for (i, c) in value.chars().enumerate().take((area.width - 4) as usize) {
            let idx = ((area.height / 2) * area.width + 2 + i) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.accent;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Bottom border
        plane.put_str(0, area.height - 1, &format!("╰{}╯", "─".repeat(area.width as usize - 2)));

        plane
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.accent = theme.primary;
    }
    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. App State
// ─────────────────────────────────────────────────────────────────────────────

struct PluginDemoApp {
    theme: Theme,
    registry: Arc<RwLock<PluginRegistry>>,
    // Widget instances created from registry
    stat_widgets: Vec<Box<dyn Widget>>,
    // Hovered widget
    hovered_idx: Option<usize>,
    // Dirty flag
    dirty: bool,
}

impl PluginDemoApp {
    fn new(theme: Theme) -> Self {
        let registry = Arc::new(RwLock::new(PluginRegistry::new()));

        // Register built-in plugins
        {
            let mut reg = registry.write().unwrap();
            reg.register("stat_cpu", |id, theme| Box::new(StatWidget::new(id, theme, "CPU", "12.4%")));
            reg.register("stat_mem", |id, theme| Box::new(StatWidget::new(id, theme, "MEM", "1.2 GiB")));
            reg.register("stat_disk", |id, theme| Box::new(StatWidget::new(id, theme, "DISK", "47%")),
            );
            reg.register("stat_net", |id, theme| Box::new(StatWidget::new(id, theme, "NET", "↓2.1")));
        }

        // Create initial instances from registry
        let stat_widgets = {
            let reg = registry.read().unwrap();
            let wid = WidgetId::new(1);
            vec![
                reg.create("stat_cpu", wid, theme).unwrap(),
                reg.create("stat_mem", WidgetId::new(2), theme).unwrap(),
                reg.create("stat_disk", WidgetId::new(3), theme).unwrap(),
                reg.create("stat_net", WidgetId::new(4), theme).unwrap(),
            ]
        };

        Self {
            theme,
            registry,
            stat_widgets,
            hovered_idx: None,
            dirty: true,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::nord(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::catppuccin_mocha(),
            Theme::gruvbox_dark(),
            Theme::nord(),
        ];
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.dirty = true;
    }
}

impl Widget for PluginDemoApp {
    fn id(&self) -> WidgetId {
        WidgetId::new(0)
    }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect {
        Rect::default()
    }
    fn set_area(&mut self, _area: Rect) {}
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Title bar
        plane.fill_bg(t.surface);
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ' ';
                plane.cells[idx].bg = t.surface;
            }
        }
        let title = "Plugin System Demo";
        let tx = (area.width.saturating_sub(title.len() as u16)) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (tx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Status bar
        let status = "Ctrl+T: theme | 1-4: refresh stat | Esc: quit | Click: spawn plugin";
        for x in 0..area.width {
            let idx = ((area.height - 1) * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].char = ' ';
            }
        }
        for (i, c) in status.chars().enumerate().take(area.width as usize) {
            let idx = ((area.height - 1) * area.width + i) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        // Plugin registry info panel
        let reg = self.registry.read().unwrap();
        let registered: Vec<_> = reg.list();
        let info = format!("Registered plugins: {} — stat_cpu, stat_mem, stat_disk, stat_net",
                           registered.len());
        for (i, c) in info.chars().enumerate().take(area.width as usize - 2) {
            let idx = (1 * area.width + 1 + i) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
            }
        }

        // Stat widget grid (2×2)
        let grid_x = 2;
        let grid_y = 3;
        let widget_w = 18u16.min((area.width - 8) / 2);
        let widget_h = (area.height - 8).max(3).min(5);

        let positions = [
            (grid_x, grid_y),
            (grid_x + widget_w + 2, grid_y),
            (grid_x, grid_y + widget_h + 1),
            (grid_x + widget_w + 2, grid_y + widget_h + 1),
        ];

        for (i, (x, y)) in positions.iter().enumerate() {
            if i < self.stat_widgets.len() {
                let wa = Rect::new(*x, *y, widget_w, widget_h);
                let wp = self.stat_widgets[i].render(wa);

                // Hover highlight
                let is_hovered = self.hovered_idx == Some(i);
                if is_hovered {
                    plane.put_str(*x, *y, "▶");
                    plane.cells[(*y * area.width + *x) as usize].fg = t.primary;
                }

                // Blit widget plane
                for cy in 0..wp.height {
                    for cx in 0..wp.width {
                        let sx = x + cx;
                        let sy = y + cy;
                        if sx < area.width && sy < area.height {
                            let src_idx = (cy * wp.width + cx) as usize;
                            let dst_idx = (sy * area.width + sx) as usize;
                            if src_idx < wp.cells.len() && dst_idx < plane.cells.len() {
                                let src = &wp.cells[src_idx];
                                plane.cells[dst_idx].char = src.char;
                                plane.cells[dst_idx].fg = src.fg;
                                plane.cells[dst_idx].bg = src.bg;
                                plane.cells[dst_idx].style = src.style;
                            }
                        }
                    }
                }

                // Key hint at bottom of each widget
                let hint = format!("[{}]", i + 1);
                for (j, c) in hint.chars().enumerate() {
                    let idx = ((y + widget_h) * area.width + x + j as u16 + widget_w.saturating_sub(3)) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.outline;
                    }
                }
            }
        }

        plane
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        use KeyCode::*;
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Theme cycling
        if let Char('t') = key.code {
            self.cycle_theme();
            for w in &mut self.stat_widgets {
                w.on_theme_change(&self.theme);
            }
            return true;
        }

        // Refresh individual stats (simulates plugin refresh)
        if let Char('1') = key.code {
            self.stat_widgets[0] = self.registry.read().unwrap()
                .create("stat_cpu", WidgetId::new(1), self.theme)
                .unwrap();
            self.dirty = true;
            return true;
        }
        if let Char('2') = key.code {
            self.stat_widgets[1] = self.registry.read().unwrap()
                .create("stat_mem", WidgetId::new(2), self.theme)
                .unwrap();
            self.dirty = true;
            return true;
        }
        if let Char('3') = key.code {
            self.stat_widgets[2] = self.registry.read().unwrap()
                .create("stat_disk", WidgetId::new(3), self.theme)
                .unwrap();
            self.dirty = true;
            return true;
        }
        if let Char('4') = key.code {
            self.stat_widgets[3] = self.registry.read().unwrap()
                .create("stat_net", WidgetId::new(4), self.theme)
                .unwrap();
            self.dirty = true;
            return true;
        }

        if key.code == Esc {
            return true; // Handled by main loop
        }
        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = Rect::new(0, 0, 80, 24);

        let grid_x = 2u16;
        let grid_y = 3u16;
        let widget_w = 18u16;
        let widget_h = 5u16;

        let positions = [
            (grid_x, grid_y),
            (grid_x + widget_w + 2, grid_y),
            (grid_x, grid_y + widget_h + 1),
            (grid_x + widget_w + 2, grid_y + widget_h + 1),
        ];

        match kind {
            MouseEventKind::Moved => {
                let old = self.hovered_idx;
                self.hovered_idx = positions.iter().enumerate()
                    .find(|(_, (x, y))| col >= *x && col < *x + widget_w && row >= *y && row < *y + widget_h)
                    .map(|(i, _)| i);
                if self.hovered_idx != old {
                    self.dirty = true;
                }
                false
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // Spawn a new stat widget instance on click
                let reg = self.registry.read().unwrap();
                let next_id = WidgetId::new((self.stat_widgets.len() + 1) as u32);
                if let Some(w) = reg.create("stat_cpu", next_id, self.theme) {
                    if self.stat_widgets.len() < 8 {
                        self.stat_widgets.push(w);
                        self.dirty = true;
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.dirty = true;
    }

    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Main
// ─────────────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_theme = Theme::from_env_or(Theme::nord());
    let theme = env_theme;

    let app = PluginDemoApp::new(theme);

    App::new()?
        .title("Plugin Demo")
        .add_widget(Box::new(app), Rect::new(0, 0, 80, 24))
        .run();

    Ok(())
}
