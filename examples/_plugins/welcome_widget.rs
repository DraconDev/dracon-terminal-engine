//! Welcome Widget Plugin — Displays a welcome message with Dracon branding.
//!
//! This widget is designed to be loaded dynamically as a plugin.
//! It shows a stylized welcome banner with version info.

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::plugin::PluginRegistry;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::input::event::{KeyEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::Cell;

/// Plugin identifier for registration
pub const WELCOME_WIDGET_NAME: &str = "welcome_widget";

/// Creates a WelcomeWidget factory function for PluginRegistry.
pub fn welcome_widget_factory(id: WidgetId, theme: Theme) -> Box<dyn Widget> {
    Box::new(WelcomeWidget::new(id, theme))
}

/// Register this plugin with a registry.
pub fn register(registry: &mut PluginRegistry) {
    let _ = registry.register(WELCOME_WIDGET_NAME, welcome_widget_factory);
}

// ═══════════════════════════════════════════════════════════════════════════════
// WELCOME WIDGET
// ═══════════════════════════════════════════════════════════════════════════════

/// A widget that displays a welcome banner with Dracon branding.
#[derive(Default)]
pub struct WelcomeWidget {
    id: WidgetId,
    area: Cell<Rect>,
    theme: Theme,
    version: String,
}

impl WelcomeWidget {
    /// Creates a new WelcomeWidget with the given ID and theme.
    pub fn new(id: WidgetId, theme: Theme) -> Self {
        Self {
            id,
            area: Cell::new(Rect::new(0, 0, 40, 9)),
            theme,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Creates a new WelcomeWidget with a custom version string.
    pub fn with_version(id: WidgetId, theme: Theme, version: &str) -> Self {
        Self {
            id,
            area: Cell::new(Rect::new(0, 0, 40, 9)),
            theme,
            version: version.to_string(),
        }
    }
}

impl Widget for WelcomeWidget {
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
        let mut plane = Plane::new(0, 40, 9);
        plane.fill_bg(t.bg);

        // ASCII art banner
        let banner = [
            "╔══════════════════════════════════════╗",
            "║     ___                  _           ║",
            "║    /   |  ____  ____ _ (_)____      ║",
            "║   / /| | / __ \\/ __ `/ / / ___/      ║",
            "║  / ___ |/ / / / /_/ / / / /          ║",
            "║ /_/  |_/_/ /_/\\__,_/ /_/_/           ║",
            "║   Terminal Engine                     ║",
            &format!("║   v{}                           ║", self.version),
            "╚══════════════════════════════════════╝",
        ];

        for (row, line) in banner.iter().enumerate() {
            let start_idx = row * 40;
            for (col, c) in line.chars().enumerate() {
                if start_idx + col < plane.cells.len() {
                    plane.cells[start_idx + col].char = c;

                    // Color the dragon face differently
                    if row == 1 && (col >= 4 && col <= 25) {
                        plane.cells[start_idx + col].fg = t.primary;
                        plane.cells[start_idx + col].style = Styles::BOLD;
                    } else if row >= 2 && row <= 5 && col > 0 && col < 39 {
                        plane.cells[start_idx + col].fg = t.secondary;
                    } else if row == 6 {
                        // "Terminal Engine" text
                        plane.cells[start_idx + col].fg = t.primary;
                        plane.cells[start_idx + col].style = Styles::BOLD;
                    } else if row == 7 {
                        // Version number
                        plane.cells[start_idx + col].fg = t.info;
                    } else {
                        // Border
                        plane.cells[start_idx + col].fg = t.outline;
                    }
                }
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
