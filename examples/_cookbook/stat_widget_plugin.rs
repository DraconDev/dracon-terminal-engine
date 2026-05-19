//! Plugin System Demo  -  StatWidget
//!
//! Demonstrates the dynamic widget loading system using `PluginRegistry`.
//! Run with: `cargo run --example stat_widget_plugin`
//!
//! This plugin exposes a `StatWidget` that displays a labeled metric
//! with a value and color-coded border.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::plugin::PluginRegistry;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ============================================================================
// PLUGIN: StatWidget  -  dynamically registered and loaded
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
    Down,  // Used by downstream consumers
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
                Trend::Up => ('^', self.theme.success),
                Trend::Down => ('v', self.theme.error),
                Trend::Neutral => ('#', self.theme.fg_muted),
            };
            let idx = (trend_y as usize * plane.width as usize + trend_x as usize)
                .min(plane.cells.len().saturating_sub(1));
            plane.cells[idx].char = ch;
            plane.cells[idx].fg = color;
        }

        plane
    }

    fn on_theme_change(&mut self, theme: &Theme) { self.theme = theme.clone(); }
    fn handle_key(&mut self, _key: KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }
}

fn truncate(s: &str, max: usize) -> String {
    let mut r = String::new();
    let mut taken = 0;
    for c in s.chars() {
        let cw = 1;
        if taken + cw >= max {
            r.push_str("...");
            break;
        }
        r.push(c);
        taken += cw;
    }
    r
}

// ============================================================================
// APP: PluginLoader  -  demonstrates PluginRegistry loading
// ============================================================================

struct PluginLoader {
    registry: PluginRegistry,
    loaded_widgets: Vec<Box<dyn Widget>>,
    next_id: usize,
    show_help: bool,
    dirty: bool,
    theme: Theme,
    should_quit: Arc<AtomicBool>,
    keybindings: KeybindingSet,
}

impl PluginLoader {
    fn new(theme: Theme, should_quit: Arc<AtomicBool>) -> Self {
        let mut registry = PluginRegistry::new();

        // Register the StatWidget plugin factory
        registry.register("stat", |id, theme| {
            let widgets = [
                ("CPU", "67%", Trend::Neutral),
                ("Memory", "4.2 GB", Trend::Up),
                ("Disk", "512 GB", Trend::Neutral),
                ("Network", "^ 2.4 MB/s", Trend::Up),
            ];
            let (label, value, trend) = widgets[id.0 % 4];
            Box::new(StatWidget::new(label, value, trend, theme)) as Box<dyn Widget>
        });

        Self {
            registry,
            loaded_widgets: Vec::new(),
            next_id: 0,
            show_help: false,
            dirty: true,
            theme,
            should_quit,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn load_plugin(&mut self) {
        let id = WidgetId::new(self.next_id);
        self.next_id += 1;
        if let Some(widget) = self.registry.create("stat", id, self.theme.clone()) {
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
        let title = "Plugin System Demo  -  StatWidget";
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
        let footer = format!("  1: CPU  2: Memory  3: Disk  4: Network  |  {}: theme  |  {}: help  |  {}: quit  ",
            self.keybindings.display(actions::THEME).unwrap_or("ctrl+t"),
            self.keybindings.display(actions::HELP).unwrap_or("f1"),
            self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q"),
        );
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

        if self.show_help {
            let t = &self.theme;
            let hw = 42u16.min(area.width.saturating_sub(4));
            let hh = 11u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;

            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
                if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
                if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
            }
            let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
            }

            let help_title = "StatWidget Plugin Help";
            let tx = hx + (hw - help_title.len() as u16) / 2;
            for (i, c) in help_title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            let shortcuts = [
                ("1-4", "Load stat plugin"),
                (self.keybindings.display(actions::THEME).unwrap_or("ctrl+t"), "Cycle theme"),
                (self.keybindings.display(actions::HELP).unwrap_or("f1"), "Toggle help"),
                (self.keybindings.display(actions::BACK).unwrap_or("esc"), "Dismiss help"),
                (self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q"), "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
                }
            }
        }

        plane
    }

    fn on_theme_change(&mut self, theme: &Theme) { self.theme = theme.clone(); self.dirty = true; }
    fn current_theme(&self) -> Option<Theme> { Some(self.theme.clone()) }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            let themes = Theme::all();
            let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
            self.theme = themes[(idx + 1) % themes.len()].clone();
            for w in &mut self.loaded_widgets { w.on_theme_change(&self.theme); }
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }

        use KeyCode::*;
        match key.code {
            Char('1') | Char('2') | Char('3') | Char('4') => { self.load_plugin(); true }
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
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app = App::new()?
        .theme(env_theme.clone());
    let _ = app.add_widget(Box::new(PluginLoader::new(env_theme.clone(), should_quit)), Rect::new(0, 0, 80, 24));
    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(|_| {})?;
    Ok(())
}
