//! Plugin System Demo — StatWidgetPlugin
//!
//! Demonstrates the dynamic widget loading system using `WidgetFactory`.
//! Run with: `cargo run --example stat_widget_plugin`
//!
//! This plugin exposes a `StatWidget` that displays a labeled metric
//! with a value, trend arrow, and color-coded threshold.

use dracon_terminal_engine::framework::prelude::*;
use std::time::Duration;

// ============================================================================
// PLUGIN: StatWidget — dynamically registered and loaded
// ============================================================================

/// A simple stat/metric display widget provided by a plugin.
pub struct StatWidget {
    label: String,
    value: String,
    trend: Trend,
    threshold: f64,       // color turns warning/error above this
    width: u16,
}

#[derive(Clone, Copy)]
enum Trend {
    Up,
    Down,
    Neutral,
}

impl StatWidget {
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            trend: Trend::Neutral,
            threshold: 80.0,
            width: 20,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_trend(mut self, trend: Trend) -> Self {
        self.trend = trend;
        self
    }

    fn render_trend(&self, plane: &mut Plane, x: u16, y: u16, theme: &Theme) {
        let (ch, color) = match self.trend {
            Trend::Up => ('▲', theme.success),
            Trend::Down => ('▼', theme.error),
            Trend::Neutral => ('◆', theme.dim),
        };
        let idx = (y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx].char = ch;
        plane.cells[idx].fg = color;
    }
}

impl Widget for StatWidget {
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(theme.bg);

        let theme = Theme::default();
        let inner_w = area.width.saturating_sub(2);

        // Label row (top)
        let label_text = truncate(&self.label, inner_w as usize);
        for (i, c) in label_text.chars().enumerate() {
            let idx = ((area.y + 1) as usize * plane.width as usize + (area.x + 1 + i) as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = c;
            plane.cells[idx].fg = theme.fg_muted;
        }

        // Value row (middle, large)
        let value_text = truncate(&self.value, inner_w as usize);
        let value_x = (area.width - value_text.len() as u16) / 2;
        let value_y = area.y + area.height / 2;
        for (i, c) in value_text.chars().enumerate() {
            let idx = (value_y as usize * plane.width as usize + (value_x + i) as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = c;
            plane.cells[idx].fg = theme.primary;
            plane.cells[idx].style = Styles::BOLD;
        }

        // Border
        draw_border(&mut plane, area, &theme);

        // Trend indicator (top-right corner area)
        let trend_x = area.x + area.width - 2;
        let trend_y = area.y + 1;
        self.render_trend(&mut plane, trend_x, trend_y, &theme);

        plane
    }

    fn handle_key(&mut self, _key: KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }
    fn on_theme_change(&mut self, _theme: &Theme) {}
}

fn truncate(s: &str, max: usize) -> String {
    let w = unicode_width(s);
    if w > max {
        let mut r = String::new();
        let mut taken = 0;
        for c in s.chars() {
            let cw = unicode_width_char(c);
            if taken + cw > max - 1 {
                r.push('…');
                break;
            }
            r.push(c);
            taken += cw;
        }
        r
    } else {
        s.to_string()
    }
}

fn unicode_width(s: &str) -> usize {
    s.chars().map(|c| unicode_width_char(c)).sum()
}

fn unicode_width_char(c: char) -> usize {
    if c == '…' { 1 } else { 1 }
}

fn draw_border(plane: &mut Plane, area: Rect, theme: &Theme) {
    // Top border
    for x in area.x..area.x + area.width {
        let idx = (area.y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx].char = '─';
        plane.cells[idx].fg = theme.outline;
    }
    // Bottom border
    for x in area.x..area.x + area.width {
        let idx = ((area.y + area.height - 1) as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx].char = '─';
        plane.cells[idx].fg = theme.outline;
    }
    // Left border
    for y in area.y..area.y + area.height {
        let idx = (y as usize * plane.width as usize + area.x as usize).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx].char = '│';
        plane.cells[idx].fg = theme.outline;
    }
    // Right border
    for y in area.y..area.y + area.height {
        let idx = (y as usize * plane.width as usize + (area.x + area.width - 1) as usize).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx].char = '│';
        plane.cells[idx].fg = theme.outline;
    }
    // Corners
    let corners = [
        (area.x, area.y, '┌'),
        (area.x + area.width - 1, area.y, '┐'),
        (area.x, area.y + area.height - 1, '└'),
        (area.x + area.width - 1, area.y + area.height - 1, '┘'),
    ];
    for (x, y, ch) in corners {
        let idx = (y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
        plane.cells[idx].char = ch;
        plane.cells[idx].fg = theme.outline;
    }
}

// ============================================================================
// APP: PluginLoader — demonstrates WidgetFactory loading
// ============================================================================

struct PluginLoader {
    factory: WidgetFactory,
    loaded_widgets: Vec<Box<dyn Widget>>,
    selected: usize,
    dirty: bool,
    theme: Theme,
}

impl PluginLoader {
    fn new(theme: Theme) -> Self {
        let mut factory = WidgetFactory::new();

        // Register the StatWidget plugin
        factory.register("stat", |config| {
            let label = config.get("label").unwrap_or("CPU");
            let value = config.get("value").unwrap_or("42%");
            let threshold: f64 = config.get("threshold").and_then(|s| s.parse().ok()).unwrap_or(80.0);
            Box::new(StatWidget::new(label, value).with_threshold(threshold))
        });

        Self {
            factory,
            loaded_widgets: Vec::new(),
            selected: 0,
            dirty: true,
            theme,
        }
    }

    fn load_plugin(&mut self, name: &str, config: &[(&str, &str)]) {
        if let Some(mut widget) = self.factory.create(name) {
            for (k, v) in config {
                widget.set_config(k, v);
            }
            self.loaded_widgets.push(widget);
            self.dirty = true;
        }
    }
}

impl Widget for PluginLoader {
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        // Header
        let title = "Plugin System Demo";
        let tx = (area.width.saturating_sub(title.len() as u16)) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = ((area.y + 1) as usize * plane.width as usize + (area.x + tx + i) as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = c;
            plane.cells[idx].fg = self.theme.primary;
            plane.cells[idx].style = Styles::BOLD;
        }

        // Loaded widgets area
        let widget_area = Rect::new(area.x + 2, area.y + 3, area.width - 4, area.height - 8);
        let cols = 4.max(1);
        let rows = ((self.loaded_widgets.len() as u16 + cols - 1) / cols).max(1);
        let cell_w = widget_area.width / cols;
        let cell_h = rows.min(widget_area.height / rows).max(3);

        for (i, widget) in self.loaded_widgets.iter().enumerate() {
            let col = (i as u16) % cols;
            let row = (i as u16) / cols;
            let w_rect = Rect::new(
                widget_area.x + col * cell_w,
                widget_area.y + row * cell_h,
                cell_w,
                cell_h,
            );
            let sub_plane = widget.render(w_rect);
            overlay_plane(&mut plane, &sub_plane, w_rect.x, w_rect.y);
        }

        // Status bar
        let status = format!(
            "  Loaded: {} widget{} | ←/→ navigate | 1-4: load plugin | F1: help | Ctrl+Q: quit  ",
            self.loaded_widgets.len(),
            if self.loaded_widgets.len() == 1 { "" } else { "s" }
        );
        for (i, c) in status.chars().enumerate() {
            let idx = ((area.y + area.height - 1) as usize * plane.width as usize + (area.x + i) as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = c;
            plane.cells[idx].fg = self.theme.fg_muted;
            plane.cells[idx].bg = self.theme.surface;
        }

        draw_border(&mut plane, area, &self.theme);
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        use KeyCode::*;
        match key.code {
            Char('1') => { self.load_plugin("stat", &[("label", "CPU"), ("value", "67%"), ("threshold", "80")]); true }
            Char('2') => { self.load_plugin("stat", &[("label", "Memory"), ("value", "4.2 GB")]); true }
            Char('3') => { self.load_plugin("stat", &[("label", "Disk"), ("value", "512 GB")]); true }
            Char('4') => { self.load_plugin("stat", &[("label", "Network"), ("value", "↑ 2.4 MB/s")]); true }
            Char('q') if key.modifiers.contains(Modifiers::CONTROL) => { true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }
    fn on_theme_change(&mut self, theme: &Theme) { self.theme = *theme; self.dirty = true; }
    fn needs_render(&self) -> bool { self.dirty }
    fn set_dirty(&mut self, dirty: bool) { self.dirty = dirty; }
}

fn overlay_plane(target: &mut Plane, source: &Plane, ox: u16, oy: u16) {
    for sy in 0..source.height {
        for sx in 0..source.width {
            let tx = ox + sx;
            let ty = oy + sy;
            if tx >= target.width || ty >= target.height { continue; }
            let src_idx = (sy as usize * source.width as usize + sx as usize).min(source.cells.len().saturating_sub(1));
            let tgt_idx = (ty as usize * target.width as usize + tx as usize).min(target.cells.len().saturating_sub(1));
            let cell = &source.cells[src_idx];
            if !cell.transparent {
                target.cells[tgt_idx] = cell.clone();
            }
        }
    }
}

// Extend Widget trait with set_config
trait WidgetExt {
    fn set_config(&mut self, _key: &str, _value: &str) {}
}

impl WidgetExt for Box<dyn Widget> {
    fn set_config(&mut self, _key: &str, _value: &str) {}
}

fn main() -> Result<()> {
    let env_theme = Theme::from_env_or(Theme::nord());
    let app = App::new()?
        .theme(env_theme)
        .add_widget(
            Box::new(PluginLoader::new(env_theme)),
            Rect::new(0, 0, 80, 24),
        );
    app.run()?;
    Ok(())
}
