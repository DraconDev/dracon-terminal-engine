//! Plugin System Demo — StatWidget
//!
//! Demonstrates the dynamic widget loading system using `PluginRegistry`.
//! Run with: `cargo run --example stat_widget_plugin`
//!
//! This plugin exposes a `StatWidget` that displays a labeled metric
//! with a value and color-coded border.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::plugin::PluginRegistry;
use dracon_terminal_engine::framework::widget::WidgetId;

// ============================================================================
// PLUGIN: StatWidget — dynamically registered and loaded
// ============================================================================

/// A simple stat/metric display widget provided by a plugin.
pub struct StatWidget {
    id: WidgetId,
    label: String,
    value: String,
    trend: Trend,
    area: Rect,
    theme: Theme,
}

#[derive(Clone, Copy)]
pub enum Trend {
    Up,
    #[allow(dead_code)]
    Down,
    Neutral,
}

impl StatWidget {
    pub fn new(label: &str, value: &str, trend: Trend, theme: Theme) -> Self {
        Self {
            id: WidgetId::new(0),
            label: label.to_string(),
            value: value.to_string(),
            trend,
            area: Rect::default(),
            theme,
        }
    }
}

impl Widget for StatWidget {
    fn id(&self) -> WidgetId { self.id }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn needs_render(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        let inner_w = area.width.saturating_sub(2);

        // Label row (top)
        let label_text = truncate(&self.label, inner_w as usize);
        for (i, c) in label_text.chars().enumerate() {
            let x = area.x + 1 + i as u16;
            if x < area.x + area.width - 1 {
                let idx = (area.y as usize * plane.width as usize + x as usize)
                    .min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
            }
        }

        // Value row (middle, large)
        let value_text = truncate(&self.value, inner_w as usize);
        let value_x = area.x.saturating_add((area.width.saturating_sub(value_text.len() as u16)) / 2);
        let value_y = area.y + area.height / 2;
        for (i, c) in value_text.chars().enumerate() {
            let x = value_x + i as u16;
            if x < area.x + area.width - 1 {
                let idx = (value_y as usize * plane.width as usize + x as usize)
                    .min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.primary;
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
            let idx = (y as usize * plane.width as usize + x as usize)
                .min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = self.theme.outline;
        }
        // Top/bottom borders
        for x in (area.x + 1)..(area.x + area.width - 1) {
            for y in [area.y, area.y + area.height - 1] {
                let idx = (y as usize * plane.width as usize + x as usize)
                    .min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = self.theme.outline;
            }
        }
        // Left/right borders
        for y in (area.y + 1)..(area.y + area.height - 1) {
            for x in [area.x, area.x + area.width - 1] {
                let idx = (y as usize * plane.width as usize + x as usize)
                    .min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = self.theme.outline;
            }
        }

        // Trend indicator
        let trend_x = area.x + area.width - 2;
        let trend_y = area.y + 1;
        if trend_x > area.x && trend_y < area.y + area.height {
            let (ch, color) = match self.trend {
                Trend::Up => ('▲', self.theme.success),
                Trend::Down => ('▼', self.theme.error),
                Trend::Neutral => ('◆', self.theme.fg_muted),
            };
            let idx = (trend_y as usize * plane.width as usize + trend_x as usize)
                .min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = color;
        }

        plane
    }

    fn on_theme_change(&mut self, theme: &Theme) { self.theme = *theme; }
    fn handle_key(&mut self, _key: KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }
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
// APP: PluginLoader — demonstrates PluginRegistry loading
// ============================================================================

struct PluginLoader {
    registry: PluginRegistry,
    loaded_widgets: Vec<Box<dyn Widget>>,
    next_id: usize,
    dirty: bool,
    theme: Theme,
}

impl PluginLoader {
    fn new(theme: Theme) -> Self {
        let mut registry = PluginRegistry::new();

        // Register the StatWidget plugin factory
        registry.register("stat", |id, theme| {
            let widgets = [
                ("CPU", "67%", Trend::Neutral),
                ("Memory", "4.2 GB", Trend::Up),
                ("Disk", "512 GB", Trend::Neutral),
                ("Network", "↑ 2.4 MB/s", Trend::Up),
            ];
            let (label, value, trend) = widgets[id.0 % 4];
            Box::new(StatWidget::new(label, value, trend, theme)) as Box<dyn Widget>
        });

        Self {
            registry,
            loaded_widgets: Vec::new(),
            next_id: 0,
            dirty: true,
            theme,
        }
    }

    fn load_plugin(&mut self) {
        let id = WidgetId::new(self.next_id);
        self.next_id += 1;
        if let Some(widget) = self.registry.create("stat", id, self.theme) {
            self.loaded_widgets.push(widget);
            self.dirty = true;
        }
    }
}

impl Widget for PluginLoader {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn area(&self) -> Rect { Rect::default() }
    fn set_area(&mut self, _area: Rect) {}
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        // Title
        let title = "Plugin System Demo — StatWidget";
        let tx = (area.width.saturating_sub(title.len() as u16)) / 2;
        for (i, c) in title.chars().enumerate() {
            let x = area.x + tx + i as u16;
            let idx = ((area.y + 1) as usize * plane.width as usize + x as usize)
                .min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = c;
            plane.cells[idx].fg = self.theme.primary;
            plane.cells[idx].style = Styles::BOLD;
        }

        // Loaded widgets area
        let widget_area = Rect::new(area.x + 2, area.y + 3, area.width - 4, area.height - 8);
        let cols = 4;
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
                let x = area.x + hx + i as u16;
                let idx = (hy as usize * plane.width as usize + x as usize)
                    .min(plane.cells.len().saturating_sub(1));
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
            }
        }

        // Footer
        let footer = "  1: CPU  2: Memory  3: Disk  4: Network  |  F1: help  |  Ctrl+Q: quit  ";
        for (i, c) in footer.chars().enumerate() {
            let x = area.x + i as u16;
            let idx = ((area.y + area.height - 1) as usize * plane.width as usize + x as usize)
                .min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = c;
            plane.cells[idx].fg = self.theme.fg_muted;
            plane.cells[idx].bg = self.theme.surface;
        }

        // Border
        draw_rect_border(&mut plane, area, &self.theme);
        plane
    }

    fn on_theme_change(&mut self, theme: &Theme) { self.theme = *theme; self.dirty = true; }
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
}

fn overlay_plane(target: &mut Plane, source: &Plane, ox: u16, oy: u16) {
    for sy in 0..source.height {
        for sx in 0..source.width {
            let tx = ox + sx;
            let ty = oy + sy;
            if tx >= target.width || ty >= target.height { continue; }
            let src_idx = (sy as usize * source.width as usize + sx as usize)
                .min(source.cells.len().saturating_sub(1));
            let tgt_idx = (ty as usize * target.width as usize + tx as usize)
                .min(target.cells.len().saturating_sub(1));
            let cell = &source.cells[src_idx];
            if !cell.transparent {
                target.cells[tgt_idx] = *cell;
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
        let idx = (y as usize * plane.width as usize + x as usize)
            .min(plane.cells.len().saturating_sub(1));
        plane.cells[idx].char = ch;
        plane.cells[idx].fg = theme.outline;
    }
    for x in (area.x + 1)..(area.x + area.width - 1) {
        for y in [area.y, area.y + area.height - 1] {
            let idx = (y as usize * plane.width as usize + x as usize)
                .min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = '─';
            plane.cells[idx].fg = theme.outline;
        }
    }
    for y in (area.y + 1)..(area.y + area.height - 1) {
        for x in [area.x, area.x + area.width - 1] {
            let idx = (y as usize * plane.width as usize + x as usize)
                .min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = '│';
            plane.cells[idx].fg = theme.outline;
        }
    }
}

fn main() -> std::io::Result<()> {
    let env_theme = Theme::from_env_or(Theme::nord());
    let mut app = App::new()?
        .theme(env_theme);
    let _ = app.add_widget(Box::new(PluginLoader::new(env_theme)), Rect::new(0, 0, 80, 24));
    app.run(|_| {})?;
    Ok(())
}
