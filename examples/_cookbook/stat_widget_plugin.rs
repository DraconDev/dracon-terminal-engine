//! Plugin System Demo — StatWidget
//!
//! Demonstrates the dynamic widget loading system using `WidgetFactory`.
//! Run with: `cargo run --example stat_widget_plugin`
//!
//! This plugin exposes a `StatWidget` that displays a labeled metric
//! with a value, trend arrow, and color-coded threshold.

use dracon_terminal_engine::framework::prelude::*;

// ============================================================================
// PLUGIN: StatWidget — dynamically registered and loaded
// ============================================================================

/// A simple stat/metric display widget provided by a plugin.
pub struct StatWidget {
    label: String,
    value: String,
    trend: Trend,
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
        }
    }
}

impl Widget for StatWidget {
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(theme.bg);

        let inner_w = area.width.saturating_sub(2);

        // Label row (top)
        let label_text = truncate(&self.label, inner_w as usize);
        for (i, c) in label_text.chars().enumerate() {
            let x = area.x + 1 + i as u16;
            if x < area.x + area.width - 1 {
                let idx = (area.y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = c;
                plane.cells[idx].fg = theme.fg_muted;
            }
        }

        // Value row (middle, large)
        let value_text = truncate(&self.value, inner_w as usize);
        let value_x = area.x.saturating_add((area.width.saturating_sub(value_text.len() as u16)) / 2);
        let value_y = area.y + area.height / 2;
        for (i, c) in value_text.chars().enumerate() {
            let x = value_x + i as u16;
            if x < area.x + area.width - 1 {
                let idx = (value_y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = c;
                plane.cells[idx].fg = theme.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        // Border corners
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
        // Top/bottom borders
        for x in (area.x + 1)..(area.x + area.width - 1) {
            for y in [area.y, area.y + area.height - 1] {
                let idx = (y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = theme.outline;
            }
        }
        // Left/right borders
        for y in (area.y + 1)..(area.y + area.height - 1) {
            for x in [area.x, area.x + area.width - 1] {
                let idx = (y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = theme.outline;
            }
        }

        // Trend indicator
        let trend_x = area.x + area.width - 2;
        let trend_y = area.y + 1;
        if trend_x > area.x && trend_y < area.y + area.height {
            let (ch, color) = match self.trend {
                Trend::Up => ('▲', theme.success),
                Trend::Down => ('▼', theme.error),
                Trend::Neutral => ('◆', theme.dim),
            };
            let idx = (trend_y as usize * plane.width as usize + trend_x as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = color;
        }

        plane
    }

    fn handle_key(&mut self, _key: KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }
    fn on_theme_change(&mut self, _theme: &Theme) {}
}

fn truncate(s: &str, max: usize) -> String {
    let mut r = String::new();
    let mut taken = 0;
    for c in s.chars() {
        let cw = if c == '…' { 0 } else { 1 };
        if taken + cw >= max {
            r.push('…');
            break;
        }
        r.push(c);
        taken += cw;
    }
    r
}

// ============================================================================
// APP: PluginLoader — demonstrates WidgetFactory loading
// ============================================================================

struct PluginLoader {
    factory: WidgetFactory,
    loaded_widgets: Vec<Box<dyn Widget>>,
    dirty: bool,
    theme: Theme,
}

impl PluginLoader {
    fn new(theme: Theme) -> Self {
        let mut factory = WidgetFactory::new();

        // Register the StatWidget plugin
        factory.register("stat", |_config| {
            Box::new(StatWidget::new("CPU", "67%")) as Box<dyn Widget>
        });

        Self {
            factory,
            loaded_widgets: Vec::new(),
            dirty: true,
            theme,
        }
    }

    fn load_plugin(&mut self) {
        if let Some(widget) = self.factory.create("stat") {
            self.loaded_widgets.push(widget);
            self.dirty = true;
        }
    }
}

impl Widget for PluginLoader {
    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        // Title
        let title = "Plugin System Demo — StatWidget";
        let tx = (area.width.saturating_sub(title.len() as u16)) / 2;
        for (i, c) in title.chars().enumerate() {
            let x = area.x + tx + i as u16;
            let idx = ((area.y + 1) as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = c;
            plane.cells[idx].fg = self.theme.primary;
            plane.cells[idx].style = Styles::BOLD;
        }

        // Loaded widgets area
        let widget_area = Rect::new(area.x + 2, area.y + 3, area.width - 4, area.height - 8);
        let cols = 4.max(1);
        let cell_w = widget_area.width / cols;
        let cell_h = 6u16;

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

        // Empty state hint
        if self.loaded_widgets.is_empty() {
            let hint = "Press 1-4 to load StatWidget plugins";
            let hx = (area.width.saturating_sub(hint.len() as u16)) / 2;
            let hy = area.y + area.height / 2;
            for (i, c) in hint.chars().enumerate() {
                let idx = (hy as usize * plane.width as usize + (area.x + hx + i as u16) as usize).min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.dim;
            }
        }

        // Footer
        let footer = "  1: CPU  2: Memory  3: Disk  4: Network  |  F1: help  |  Ctrl+Q: quit  ";
        for (i, c) in footer.chars().enumerate() {
            let x = area.x + i as u16;
            let idx = ((area.y + area.height - 1) as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = c;
            plane.cells[idx].fg = self.theme.fg_muted;
            plane.cells[idx].bg = self.theme.surface;
        }

        // Border
        draw_rect_border(&mut plane, area, &self.theme);
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        use KeyCode::*;
        match key.code {
            Char('1') => { self.load_plugin(); true }
            Char('2') => { self.load_plugin(); true }
            Char('3') => { self.load_plugin(); true }
            Char('4') => { self.load_plugin(); true }
            Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => { true }
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

fn draw_rect_border(plane: &mut Plane, area: Rect, theme: &Theme) {
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
    for x in (area.x + 1)..(area.x + area.width - 1) {
        for y in [area.y, area.y + area.height - 1] {
            let idx = (y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = theme.outline;
        }
    }
    for y in (area.y + 1)..(area.y + area.height - 1) {
        for x in [area.x, area.x + area.width - 1] {
            let idx = (y as usize * plane.width as usize + x as usize).min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = '│';
            plane.cells[idx].fg = theme.outline;
        }
    }
}

fn main() -> Result<()> {
    let env_theme = Theme::from_env_or(Theme::nord());
    let app = App::new()?
        .theme(env_theme);
    let _ = app.add_widget(Box::new(PluginLoader::new(env_theme)), Rect::new(0, 0, 80, 24));
    app.run(|_| {});
    Ok(())
}
