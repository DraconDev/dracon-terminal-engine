//! Embedded Theme Switcher scene for the showcase.
//!
//! Displays a grid of theme swatches with color dots, plus widget preview.
//! Arrow keys or click to select, t to cycle.

use crate::scenes::shared_helpers::{blit_to, draw_text};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, Checkbox, Gauge, StatusBadge};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

struct SwatchConfig<'a> {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    theme: &'a Theme,
    selected: bool,
}

pub struct ThemeSwitcherScene {
    theme_index: usize,
    theme: Theme,
    show_help: bool,
    dirty: bool,
    checkbox: Checkbox,
    button: Button,
    gauge: Gauge,
    badge: StatusBadge,
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
}

impl ThemeSwitcherScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme_index: 0,
            theme,
            show_help: false,
            dirty: true,
            checkbox: Checkbox::new(WidgetId::new(10), "Preview"),
            button: Button::with_id(WidgetId::new(11), "Action"),
            gauge: {
                let mut g = Gauge::with_id(WidgetId::new(12), "Load")
                    .warn_threshold(70.0)
                    .crit_threshold(90.0);
                g.set_value(65.0);
                g
            },
            badge: StatusBadge::new(WidgetId::new(13)).with_status("OK"),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn select_theme(&mut self, index: usize) {
        let themes = Theme::all();
        if index < themes.len() {
            self.theme_index = index;
            self.theme = themes[index].clone();
            self.checkbox.on_theme_change(&self.theme);
            self.button.on_theme_change(&self.theme);
            self.gauge.on_theme_change(&self.theme);
            self.badge.on_theme_change(&self.theme);
            self.dirty = true;
        }
    }

    fn render_swatch(&self, plane: &mut Plane, cfg: SwatchConfig) {
        let SwatchConfig { x, y, w, h, theme, selected } = cfg;
        let border = if selected { self.theme.primary } else { self.theme.outline };

        for row in y..y + h {
            for col in x..x + w {
                let idx = (row * plane.width + col) as usize;
                if idx >= plane.cells.len() { continue; }
                plane.cells[idx].bg = theme.bg;
                plane.cells[idx].transparent = false;
                let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
                if is_border {
                    plane.cells[idx].fg = if selected { self.theme.primary } else { border };
                    plane.cells[idx].char = match (row == y, row == y + h - 1, col == x, col == x + w - 1) {
                        (true, _, true, _) => '╭',
                        (true, _, _, true) => '╮',
                        (_, true, true, _) => '╰',
                        (_, true, _, true) => '╯',
                        _ => '─',
                    };
                }
            }
        }

        // Name (centered in first interior row)
        let name = &theme.name;
        let name_x = x + 1 + (w.saturating_sub(2).saturating_sub(name.len() as u16)) / 2;
        draw_text(plane, name_x, y + 1, name, theme.fg, theme.bg, selected);

        // Color dots (second interior row) showing key theme colors
        if h >= 4 {
            let colors = [theme.bg, theme.fg, theme.primary, theme.secondary, theme.success, theme.warning, theme.error];
            let mut dot_x = x + 2;
            for &color in &colors {
                if dot_x >= x + w - 1 { break; }
                let idx = ((y + 2) * plane.width + dot_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '●';
                    plane.cells[idx].fg = color;
                    plane.cells[idx].bg = theme.bg;
                }
                dot_x += 2;
            }
        }

        // Selection indicator
        if selected {
            let arrow_x = x + w / 2;
            let arrow_y = y + h - 1;
            let idx = (arrow_y * plane.width + arrow_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '▼';
                plane.cells[idx].fg = self.theme.primary;
            }
        }
    }
}

impl Scene for ThemeSwitcherScene {
    fn scene_id(&self) -> &str { "theme_switcher" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title
        draw_text(&mut plane, 2, 0, " Theme Switcher ", t.primary, t.bg, true);
        let hint = format!(" {} / {} ", self.theme_index + 1, Theme::all().len());
        draw_text(&mut plane, area.width.saturating_sub(hint.len() as u16 + 2), 0,
                  &hint, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Layout: swatch grid (top ~60%) | preview + details (bottom ~35%)
        let grid_end_y = area.height * 60 / 100;

        // Theme grid (4 columns)
        let cols: u16 = 4;
        let swatch_w = (area.width.saturating_sub(2)) / cols;
        let swatch_h = 4u16; // taller to fit color dots
        let start_y = 2u16;

        for (i, theme) in Theme::all().iter().enumerate() {
            let col = i as u16 % cols;
            let row = i as u16 / cols;
            let sx = 1 + col * swatch_w;
            let sy = start_y + row * swatch_h;
            if sy + swatch_h >= grid_end_y { break; }

            let selected = i == self.theme_index;
            self.render_swatch(&mut plane, SwatchConfig { x: sx, y: sy, w: swatch_w, h: swatch_h, theme, selected });
        }

        // Divider before preview
        for x in 0..area.width {
            let idx = (grid_end_y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Preview section
        let preview_y = grid_end_y + 1;
        draw_text(&mut plane, 2, preview_y, "Widget Preview", t.primary, t.bg, true);

        // Left preview: checkbox + button + badge
        let preview_w = area.width.saturating_sub(4) / 3;
        let px = 2u16;

        let cb_plane = self.checkbox.render(Rect::new(px, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &cb_plane, px as usize, (preview_y + 2) as usize);

        let btn_plane = self.button.render(Rect::new(px + preview_w + 1, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &btn_plane, (px + preview_w + 1) as usize, (preview_y + 2) as usize);

        let badge_plane = self.badge.render(Rect::new(px + preview_w * 2 + 2, preview_y + 2, preview_w, 1));
        blit_to(&mut plane, &badge_plane, (px + preview_w * 2 + 2) as usize, (preview_y + 2) as usize);

        // Gauge
        let gauge_plane = self.gauge.render(Rect::new(px, preview_y + 4, area.width.saturating_sub(4), 2));
        blit_to(&mut plane, &gauge_plane, px as usize, (preview_y + 4) as usize);

        // Theme details row
        if preview_y + 7 < area.height.saturating_sub(2) {
            let details_y = preview_y + 7;
            draw_text(&mut plane, 2, details_y, "Selected: ", t.fg_muted, t.bg, false);
            draw_text(&mut plane, 12, details_y, &t.name, t.primary, t.bg, true);

            // Show current theme's color swatch inline
            let dot_colors = [
                ("bg", t.bg), ("fg", t.fg), ("primary", t.primary),
                ("secondary", t.secondary), ("success", t.success),
            ];
            let mut dot_x = 12u16 + t.name.len() as u16 + 3;
            for (label, color) in &dot_colors {
                if dot_x + label.len() as u16 + 3 > area.width.saturating_sub(2) { break; }
                let idx = (details_y * plane.width + dot_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '■';
                    plane.cells[idx].fg = *color;
                }
                draw_text(&mut plane, dot_x + 1, details_y, label, t.fg_muted, t.bg, false);
                dot_x += label.len() as u16 + 3;
            }
        }

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        let nav = " ←→↑↓:navigate | Enter:apply | ?:help | B:back ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.surface, false);

        if self.show_help {
            crate::scenes::shared_helpers::render_help_overlay(
                &mut plane, area, t,
                "Theme Switcher Help",
                &[
                    ("←/→/↑/↓", "Navigate themes"),
                    ("Enter", "Apply theme"),
                    ("Click", "Select swatch"),
                    ("?", "Toggle help"),
                    ("B/Esc", "Back to showcase"),
                ],
            );
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        let themes = Theme::all();
        let cols: usize = 4;
        match key.code {
            KeyCode::Right => {
                let next = (self.theme_index + 1).min(themes.len() - 1);
                self.select_theme(next);
                true
            }
            KeyCode::Left => {
                let prev = self.theme_index.saturating_sub(1);
                self.select_theme(prev);
                true
            }
            KeyCode::Down => {
                let next = self.theme_index + cols;
                if next < themes.len() {
                    self.select_theme(next);
                }
                true
            }
            KeyCode::Up => {
                let prev = self.theme_index.saturating_sub(cols);
                self.select_theme(prev);
                true
            }
            KeyCode::Enter => {
                // Apply selected theme (already applied on select)
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.show_help { return true; }
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            let area = self.area.get();
            let cols: u16 = 4;
            let swatch_w = (area.width.saturating_sub(2)) / cols;
            let swatch_h = 4u16;
            let start_y = 2u16;
            let grid_end_y = area.height * 60 / 100;

            for (i, _theme) in Theme::all().iter().enumerate() {
                let c = i as u16 % cols;
                let r = i as u16 / cols;
                let sx = 1 + c * swatch_w;
                let sy = start_y + r * swatch_h;
                if sy + swatch_h >= grid_end_y { break; }
                if col >= sx && col < sx + swatch_w && row >= sy && row < sy + swatch_h {
                    self.select_theme(i);
                    return true;
                }
            }
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.checkbox.on_theme_change(theme);
        self.button.on_theme_change(theme);
        self.gauge.on_theme_change(theme);
        self.badge.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

