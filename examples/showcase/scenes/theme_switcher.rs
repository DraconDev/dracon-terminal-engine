//! Embedded Theme Studio scene for the showcase.
//!
//! Interactive theme browser: select a theme from the sidebar, see a full
//! widget preview on the right, plus color palette and contrast info.
//! Like a theme designer tool.

use crate::scenes::shared_helpers::{
    blit_to, draw_focus_ring, draw_text, draw_text_clipped, render_help_overlay,
};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::hitzone::ScopedZoneRegistry;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, Checkbox, Gauge, ProgressBar, Radio, SearchInput, Slider, StatusBadge, Toggle,
};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;
use std::cell::RefCell;

const SIDEBAR_WIDTH: u16 = 22;

pub struct ThemeSwitcherScene {
    theme_index: usize,
    theme: Theme,
    show_help: bool,
    dirty: bool,
    zones: RefCell<ScopedZoneRegistry<usize>>,
    area: std::cell::Cell<Rect>,

    // Preview widgets (re-themed on selection)
    checkbox: Checkbox,
    radio: Radio,
    toggle: Toggle,
    slider: Slider,
    button: Button,
    gauge: Gauge,
    progress: ProgressBar,
    search: SearchInput,
    badge: StatusBadge,

    keybindings: KeybindingSet,
}

impl ThemeSwitcherScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme_index: 0,
            theme,
            show_help: false,
            dirty: true,
            zones: RefCell::new(ScopedZoneRegistry::new()),
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),

            checkbox: Checkbox::new(WidgetId::new(10), "Enable Feature"),
            radio: Radio::new(WidgetId::new(11), "Selected"),
            toggle: Toggle::new(WidgetId::new(12), "Dark Mode"),
            slider: Slider::new(WidgetId::new(13)).with_range(0.0, 100.0),
            button: Button::with_id(WidgetId::new(14), " Apply "),
            gauge: {
                let mut g = Gauge::with_id(WidgetId::new(15), "CPU")
                    .warn_threshold(70.0)
                    .crit_threshold(90.0);
                g.set_value(65.0);
                g
            },
            progress: ProgressBar::new(WidgetId::new(16)),
            search: SearchInput::new(WidgetId::new(17)),
            badge: StatusBadge::new(WidgetId::new(18)).with_status("Running"),

            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn themes(&self) -> &'static [Theme] {
        Theme::all()
    }

    fn apply_theme(&mut self, index: usize) {
        let themes = self.themes();
        if index < themes.len() {
            self.theme_index = index;
            self.theme = themes[index].clone();
            let t = &self.theme;
            self.checkbox.on_theme_change(t);
            self.radio.on_theme_change(t);
            self.toggle.on_theme_change(t);
            self.slider.on_theme_change(t);
            self.button.on_theme_change(t);
            self.gauge.on_theme_change(t);
            self.progress.on_theme_change(t);
            self.search.on_theme_change(t);
            self.badge.on_theme_change(t);
            self.dirty = true;
        }
    }

    fn render_sidebar(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let max_x = SIDEBAR_WIDTH;
        let themes = self.themes();

        // Sidebar header
        draw_text_clipped(
            plane,
            1,
            0,
            " Themes ",
            max_x,
            t.fg_on_accent,
            t.primary,
            true,
        );

        // Theme list
        for (i, theme) in themes.iter().enumerate() {
            let row = i as u16 + 1;
            if row >= area.height.saturating_sub(2) {
                break;
            }

            let is_selected = i == self.theme_index;
            let bg = if is_selected { t.primary } else { t.surface };

            // Fill row
            for x in 0..SIDEBAR_WIDTH {
                let idx = (row * plane.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ' ';
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }

            // Selection indicator
            if is_selected {
                let idx = (row * plane.width) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '▸';
                    plane.cells[idx].fg = t.fg_on_accent;
                    plane.cells[idx].transparent = false;
                }
            }

            // Theme name
            let fg = if is_selected { t.fg_on_accent } else { t.fg };
            draw_text_clipped(
                plane,
                1,
                row,
                &theme.name,
                max_x.saturating_sub(2),
                fg,
                bg,
                false,
            );

            // Mini color swatch (5 dots)
            let swatch_x = SIDEBAR_WIDTH.saturating_sub(7);
            let colors = [
                theme.primary,
                theme.secondary,
                theme.success,
                theme.warning,
                theme.error,
            ];
            for (j, &color) in colors.iter().enumerate() {
                let sx = swatch_x + j as u16 * 2;
                let idx = (row * plane.width + sx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '●';
                    plane.cells[idx].fg = color;
                    plane.cells[idx].bg = bg;
                }
            }

            // Hit zone
            self.zones
                .borrow_mut()
                .register(i, 0, row, SIDEBAR_WIDTH, 1);
        }

        // Count footer
        let count_y = area.height.saturating_sub(1);
        let count = format!(" {} themes ", themes.len());
        draw_text_clipped(
            plane, 1, count_y, &count, max_x, t.fg_muted, t.surface, false,
        );
    }

    fn render_preview(&self, plane: &mut Plane, area: Rect, _div_x: u16) {
        let t = &self.theme;
        let panel_x = area.x;
        let panel_w = area.width;

        // ── Section 1: Widget Preview ──────────────────────────
        draw_text(plane, panel_x, 0, "Widget Preview", t.primary, t.bg, true);

        // Preview card background
        let card_x = panel_x;
        let card_y = 1;
        let card_w = panel_w;
        let card_h = 8;

        for y in card_y..card_y + card_h {
            for x in card_x..card_x + card_w {
                let idx = y as usize * plane.width as usize + x as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].transparent = false;
                }
            }
        }
        draw_focus_ring(plane, card_x, card_y, card_w, card_h, t.outline);

        // Row 1: Checkbox, Radio, Toggle
        let row1_y = card_y + 1;
        let col_w = card_w / 4;

        let cb_area = Rect::new(card_x + 2, row1_y, col_w, 1);
        blit_to(
            plane,
            &self.checkbox.render(cb_area),
            cb_area.x as usize,
            cb_area.y as usize,
        );

        let radio_area = Rect::new(card_x + 2 + col_w, row1_y, col_w, 1);
        blit_to(
            plane,
            &self.radio.render(radio_area),
            radio_area.x as usize,
            radio_area.y as usize,
        );

        let toggle_area = Rect::new(card_x + 2 + col_w * 2, row1_y, col_w, 1);
        blit_to(
            plane,
            &self.toggle.render(toggle_area),
            toggle_area.x as usize,
            toggle_area.y as usize,
        );

        // Button
        let btn_area = Rect::new(card_x + 2 + col_w * 3, row1_y, col_w.saturating_sub(1), 1);
        blit_to(
            plane,
            &self.button.render(btn_area),
            btn_area.x as usize,
            btn_area.y as usize,
        );

        // Row 2: Slider
        let row2_y = card_y + 3;
        let slider_area = Rect::new(card_x + 2, row2_y, card_w.saturating_sub(4), 1);
        blit_to(
            plane,
            &self.slider.render(slider_area),
            slider_area.x as usize,
            slider_area.y as usize,
        );

        // Row 3: SearchInput
        let row3_y = card_y + 5;
        let search_area = Rect::new(card_x + 2, row3_y, card_w.saturating_sub(4), 1);
        blit_to(
            plane,
            &self.search.render(search_area),
            search_area.x as usize,
            search_area.y as usize,
        );

        // Row 4: Progress + Badge
        let row4_y = card_y + 7;
        let prog_area = Rect::new(card_x + 2, row4_y, card_w.saturating_sub(8), 1);
        blit_to(
            plane,
            &self.progress.render(prog_area),
            prog_area.x as usize,
            prog_area.y as usize,
        );

        let badge_area = Rect::new(card_x + 2 + card_w.saturating_sub(7), row4_y, 6, 1);
        blit_to(
            plane,
            &self.badge.render(badge_area),
            badge_area.x as usize,
            badge_area.y as usize,
        );

        // ── Section 2: Color Palette ────────────────────────────
        let palette_y = card_y + card_h + 2;
        if palette_y < area.height.saturating_sub(8) {
            draw_text(
                plane,
                panel_x,
                palette_y,
                "Color Palette",
                t.primary,
                t.bg,
                true,
            );

            // Palette grid: 4 columns of color swatches
            let colors = [
                ("bg", t.bg),
                ("fg", t.fg),
                ("primary", t.primary),
                ("secondary", t.secondary),
                ("success", t.success),
                ("warning", t.warning),
                ("error", t.error),
                ("outline", t.outline),
                ("muted", t.fg_muted),
                ("selection", t.selection_bg),
                ("hover", t.hover_bg),
                ("bg_elev", t.surface_elevated),
            ];

            let swatch_w = 3u16;
            let swatch_h = 2u16;
            let cols = 4u16;
            let gap_x = panel_w / cols;

            for (i, (name, color)) in colors.iter().enumerate() {
                let col = i as u16 % cols;
                let row = i as u16 / cols;
                let sx = panel_x + col * gap_x;
                let sy = palette_y + 1 + row * (swatch_h + 1);

                if sy + swatch_h >= area.height.saturating_sub(4) {
                    break;
                }

                // Swatch background (colored)
                for dy in 0..swatch_h {
                    for dx in 0..swatch_w {
                        let cx = sx + dx;
                        let cy = sy + dy;
                        let idx = cy as usize * plane.width as usize + cx as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = ' ';
                            plane.cells[idx].bg = *color;
                            plane.cells[idx].transparent = false;
                        }
                    }
                }

                // Border
                let border_color = if *name == "bg" { t.outline } else { *color };
                for x in sx..sx + swatch_w {
                    let top_idx = sy as usize * plane.width as usize + x as usize;
                    let bot_idx = (sy + swatch_h - 1) as usize * plane.width as usize + x as usize;
                    if top_idx < plane.cells.len() {
                        plane.cells[top_idx].char = '─';
                        plane.cells[top_idx].fg = border_color;
                    }
                    if bot_idx < plane.cells.len() {
                        plane.cells[bot_idx].char = '─';
                        plane.cells[bot_idx].fg = border_color;
                    }
                }
                for y in sy..sy + swatch_h {
                    let left_idx = y as usize * plane.width as usize + sx as usize;
                    let right_idx =
                        y as usize * plane.width as usize + (sx + swatch_w - 1) as usize;
                    if left_idx < plane.cells.len() {
                        plane.cells[left_idx].char = '│';
                        plane.cells[left_idx].fg = border_color;
                    }
                    if right_idx < plane.cells.len() {
                        plane.cells[right_idx].char = '│';
                        plane.cells[right_idx].fg = border_color;
                    }
                }

                // Label
                draw_text(plane, sx, sy + swatch_h, name, t.fg_muted, t.bg, false);
            }
        }

        // ── Section 3: Contrast Info ──────────────────────────
        let contrast_y = area.height.saturating_sub(6);
        if contrast_y > 2 {
            draw_text(
                plane,
                panel_x,
                contrast_y,
                "Contrast Ratios",
                t.primary,
                t.bg,
                true,
            );

            // Fake contrast ratios (just for visual)
            let ratios = [
                ("fg/bg", "12.5:1", "AAA"),
                ("primary/bg", "7.2:1", "AAA"),
                ("error/bg", "4.8:1", "AA"),
                ("warning/bg", "3.5:1", "AA Large"),
            ];

            for (i, (pair, ratio, level)) in ratios.iter().enumerate() {
                let ry = contrast_y + 1 + i as u16;
                if ry >= area.height.saturating_sub(2) {
                    break;
                }
                let level_color = match *level {
                    "AAA" => t.success,
                    "AA" => t.primary,
                    "AA Large" => t.warning,
                    _ => t.fg_muted,
                };
                draw_text_clipped(
                    plane,
                    panel_x,
                    ry,
                    pair,
                    panel_x + 8,
                    t.fg_muted,
                    t.bg,
                    false,
                );
                draw_text_clipped(
                    plane,
                    panel_x + 8,
                    ry,
                    ratio,
                    panel_x + 18,
                    t.fg,
                    t.bg,
                    false,
                );
                draw_text_clipped(
                    plane,
                    panel_x + 18,
                    ry,
                    level,
                    panel_x + panel_w,
                    level_color,
                    t.bg,
                    true,
                );
            }
        }
    }
}

impl Scene for ThemeSwitcherScene {
    fn scene_id(&self) -> &str {
        "theme_switcher"
    }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Header
        draw_text(&mut plane, 2, 0, " Theme Studio ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", t.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Header divider
        for x in 0..area.width {
            let idx = x as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Left sidebar
        self.zones.borrow_mut().clear();
        self.render_sidebar(&mut plane, area);

        // Sidebar divider
        let div_x = SIDEBAR_WIDTH;
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * area.width + div_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Right panel
        self.render_preview(&mut plane, area, div_x);

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        let nav = " ↑/↓: select theme | B/Esc: back | ?: help ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.surface, false);

        if self.show_help {
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Theme Studio Help",
                &[
                    ("↑/↓", "Navigate theme list"),
                    ("Click", "Select theme"),
                    ("T", "Cycle themes"),
                    ("Esc", "Back to showcase"),
                    ("?", "Toggle this help"),
                ],
            );
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                return true;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        let themes = self.themes();
        match key.code {
            KeyCode::Up if key.modifiers.is_empty() => {
                let prev = self.theme_index.saturating_sub(1);
                self.apply_theme(prev);
                true
            }
            KeyCode::Down if key.modifiers.is_empty() => {
                let next = (self.theme_index + 1).min(themes.len() - 1);
                self.apply_theme(next);
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.show_help {
            return true;
        }

        // Check sidebar zones
        let zone_idx = self.zones.borrow().dispatch(col, row);
        if let Some(idx) = zone_idx {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                self.apply_theme(idx);
            }
            return true;
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.checkbox.on_theme_change(theme);
        self.radio.on_theme_change(theme);
        self.toggle.on_theme_change(theme);
        self.slider.on_theme_change(theme);
        self.button.on_theme_change(theme);
        self.gauge.on_theme_change(theme);
        self.progress.on_theme_change(theme);
        self.search.on_theme_change(theme);
        self.badge.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
