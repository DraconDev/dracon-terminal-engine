//! Embedded Color Studio scene for the showcase.
//!
//! Full color design tool with:
//!   - Large ColorPicker widget on left
//!   - Generated palette (shades, tints) on right
//!   - CSS output box
//!   - Quick palette presets
//!   - Contrast ratio calculator

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::{Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::color_picker::ColorPicker;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind, MouseButton};
use ratatui::layout::Rect;
use std::cell::RefCell;

const SIDEBAR_WIDTH: u16 = 38;

pub struct ColorPickerScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    picker: RefCell<ColorPicker>,
    selected_color: Color,
    selected_hex: String,
    dirty: bool,
    area: std::cell::Cell<Rect>,
    // Track recent color picks
    recent_colors: RefCell<Vec<Color>>,
}

impl ColorPickerScene {
    pub fn new(theme: Theme) -> Self {
        let initial_color = Color::Rgb(88, 166, 255);
        let picker = ColorPicker::with_color(initial_color).with_theme(theme.clone());

        Self {
            theme: theme.clone(),
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            picker: RefCell::new(picker),
            selected_color: initial_color,
            selected_hex: "#58a6ff".into(),
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            recent_colors: RefCell::new(vec![
                Color::Rgb(136, 192, 208),
                Color::Rgb(208, 135, 112),
                Color::Rgb(163, 190, 140),
            ]),
        }
    }

    fn add_recent(&self, color: Color) {
        let mut recent = self.recent_colors.borrow_mut();
        // Avoid duplicates
        if !recent.contains(&color) {
            recent.insert(0, color);
            if recent.len() > 8 {
                recent.pop();
            }
        }
    }

    fn rgb_to_hex(color: Color) -> Option<String> {
        if let Color::Rgb(r, g, b) = color {
            Some(format!("#{:02x}{:02x}{:02x}", r, g, b))
        } else {
            None
        }
    }

    fn luminance(color: Color) -> f64 {
        if let Color::Rgb(r, g, b) = color {
            let to_linear = |v: u8| {
                let v = v as f64 / 255.0;
                if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * to_linear(r) + 0.7152 * to_linear(g) + 0.0722 * to_linear(b)
        } else {
            0.0
        }
    }

    fn contrast_ratio(fg: Color, bg: Color) -> f64 {
        let l1 = Self::luminance(fg);
        let l2 = Self::luminance(bg);
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    // Generate shades (darker variations)
    fn generate_shades(color: Color) -> Vec<Color> {
        if let Color::Rgb(r, g, b) = color {
            let factors = [0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
            factors.iter().map(|f| {
                Color::Rgb(
                    (r as f32 * *f as f32).min(255.0) as u8,
                    (g as f32 * *f as f32).min(255.0) as u8,
                    (b as f32 * *f as f32).min(255.0) as u8,
                )
            }).collect()
        } else {
            vec![color]
        }
    }

    fn render_picker_panel(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;

        // Section header
        draw_text_clipped(plane, 1, 0, " Color Picker ", SIDEBAR_WIDTH, t.fg_on_accent, t.primary, true);
        for x in 0..SIDEBAR_WIDTH {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Large picker widget
        let picker_area = Rect::new(1, 2, SIDEBAR_WIDTH.saturating_sub(2), area.height.saturating_sub(4));
        self.picker.borrow_mut().set_area(picker_area);
        let picker_plane = self.picker.borrow().render(picker_area);
        blit_to(plane, &picker_plane, 1, 2);

        // Key hints below picker
        draw_text_clipped(plane, 1, area.height.saturating_sub(2), "↑↓←→: HSL | Tab: slider | Enter: copy hex", area.width, t.fg_muted, t.surface, false);
    }

    fn render_palette_panel(&self, plane: &mut Plane, area: Rect, div_x: u16) {
        let t = &self.theme;
        let panel_x = div_x + 1;
        let panel_w = area.width.saturating_sub(panel_x + 1);

        // ── Preview Swatch ────────────────────────────────────
        draw_text_clipped(plane, panel_x, 0, " Preview ", panel_w, t.fg_on_accent, t.primary, true);
        for x in panel_x..panel_x + panel_w {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Large color swatch
        let swatch_y = 1;
        let swatch_h = 4u16;
        for dy in 0..swatch_h {
            for dx in 0..panel_w.saturating_sub(2) {
                let px = panel_x + 1 + dx;
                let py = swatch_y + dy;
                if px < area.width && py < area.height.saturating_sub(10) {
                    let idx = (py * area.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = self.selected_color;
                        plane.cells[idx].char = ' ';
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // Color info
        let info_y = swatch_y + swatch_h + 1;
        draw_text_clipped(plane, panel_x + 1, info_y, &format!("Hex: {}", self.selected_hex), panel_w, t.primary, t.bg, true);

        if let Color::Rgb(r, g, b) = self.selected_color {
            draw_text_clipped(plane, panel_x + 1, info_y + 1, &format!("RGB: {}, {}, {}", r, g, b), panel_w, t.fg_muted, t.bg, false);
        }

        // ── Contrast Calculator ────────────────────────────────
        let contrast_y = info_y + 3;
        draw_text_clipped(plane, panel_x + 1, contrast_y, "Contrast", panel_w, t.secondary, t.bg, true);

        let bg = t.bg;
        let ratio = Self::contrast_ratio(self.selected_color, bg);
        let (rating, rating_color) = if ratio >= 7.0 {
            ("AAA (best)", t.success)
        } else if ratio >= 4.5 {
            ("AA", t.info)
        } else if ratio >= 3.0 {
            ("AA Large", t.warning)
        } else {
            ("Fail", t.error)
        };
        draw_text_clipped(plane, panel_x + 1, contrast_y + 1, &format!("{:.1}:1", ratio), panel_w, t.fg, t.bg, false);
        draw_text_clipped(plane, panel_x + 1, contrast_y + 2, rating, panel_w, rating_color, t.bg, true);

        // Secondary contrast (with white)
        let ratio_w = Self::contrast_ratio(self.selected_color, Color::Rgb(255, 255, 255));
        let (rating_w, rating_w_color) = if ratio_w >= 7.0 {
            ("AAA", t.success)
        } else if ratio_w >= 4.5 {
            ("AA", t.info)
        } else if ratio_w >= 3.0 {
            ("AA Large", t.warning)
        } else {
            ("Fail", t.error)
        };
        draw_text_clipped(plane, panel_x + 1, contrast_y + 4, "vs white:", panel_w, t.fg_muted, t.bg, false);
        let ratio_w_str = format!("{:.1}:1", ratio_w);
        draw_text_clipped(plane, panel_x + 1, contrast_y + 5, &format!("{} ({})", ratio_w_str, rating_w), panel_w, rating_w_color, t.bg, false);

        // ── Generated Palette (shades) ───────────────────────
        let shades_y = contrast_y + 7;
        if shades_y + 4 < area.height.saturating_sub(6) {
            draw_text_clipped(plane, panel_x + 1, shades_y, "Shades", panel_w, t.secondary, t.bg, true);
            for x in panel_x + 1..panel_x + panel_w.saturating_sub(2) {
                let idx = ((shades_y + 1) * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }

            let shades = Self::generate_shades(self.selected_color);
            let swatch_w = panel_w.saturating_sub(2) / shades.len() as u16;
            for (i, shade) in shades.iter().enumerate() {
                let sx = panel_x + 1 + i as u16 * swatch_w;
                let sy = shades_y + 2;

                // Swatch
                for dy in 0..2u16 {
                    for dx in 0..swatch_w.saturating_sub(1) {
                        let px = sx + dx;
                        let py = sy + dy;
                        if px < panel_x + panel_w && py < area.height.saturating_sub(6) {
                            let idx = (py * area.width + px) as usize;
                            if idx < plane.cells.len() {
                                plane.cells[idx].bg = *shade;
                                plane.cells[idx].char = ' ';
                                plane.cells[idx].transparent = false;
                            }
                        }
                    }
                }

                // Hex label
                if let Some(hex) = Self::rgb_to_hex(*shade) {
                    let label = if hex.len() > 7 { &hex[..7] } else { &hex };
                    draw_text_clipped(plane, sx, sy + 2, label, sx + swatch_w, t.fg_muted, t.bg, false);
                }
            }
        }

        // ── CSS Output ────────────────────────────────────────
        let css_y = shades_y + 5;
        if css_y + 4 < area.height.saturating_sub(6) {
            draw_text_clipped(plane, panel_x + 1, css_y, "CSS", panel_w, t.secondary, t.bg, true);
            for x in panel_x + 1..panel_x + panel_w.saturating_sub(2) {
                let idx = ((css_y + 1) * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }

            let css_hex = format!("# {}", self.selected_hex);
            let css_rgb = if let Color::Rgb(r, g, b) = self.selected_color {
                format!("rgb({}, {}, {})", r, g, b)
            } else {
                String::new()
            };

            for (i, line) in [css_hex.as_str(), css_rgb.as_str()].iter().enumerate() {
                if !line.is_empty() {
                    draw_text_clipped(plane, panel_x + 1, css_y + 2 + i as u16, line, panel_w, t.fg, t.bg, false);
                }
            }
            draw_text_clipped(plane, panel_x + 1, css_y + 4, "[Enter] Copy hex", panel_w, t.fg_muted, t.bg, false);
        }

        // ── Recent Colors ──────────────────────────────────────
        let recent_y = area.height.saturating_sub(7);
        if recent_y > 2 {
            draw_text_clipped(plane, panel_x + 1, recent_y, "Recent", panel_w, t.secondary, t.bg, true);

            let recent = self.recent_colors.borrow();
            let recent_w = panel_w.saturating_sub(2) / recent.len().max(1) as u16;
            for (i, color) in recent.iter().enumerate() {
                let rx = panel_x + 1 + i as u16 * recent_w;
                let ry = recent_y + 1;

                for dx in 0..recent_w.saturating_sub(1) {
                    let px = rx + dx;
                    if px < panel_x + panel_w && ry < area.height.saturating_sub(4) {
                        let idx = (ry * area.width + px) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].bg = *color;
                            plane.cells[idx].char = ' ';
                            plane.cells[idx].transparent = false;
                        }
                    }
                }
            }
        }

        // ── Quick Palette ─────────────────────────────────────
        let palette_y = area.height.saturating_sub(5);
        draw_text_clipped(plane, panel_x + 1, palette_y, "Palette", panel_w, t.secondary, t.bg, true);

        let palette = [
            ("#ff6b6b", Color::Rgb(255, 107, 107)),
            ("#ffd93d", Color::Rgb(255, 217, 61)),
            ("#6bcb77", Color::Rgb(107, 203, 119)),
            ("#4d96ff", Color::Rgb(77, 150, 255)),
            ("#9b59b6", Color::Rgb(155, 89, 182)),
        ];

        for (i, (hex, color)) in palette.iter().enumerate() {
            let px = panel_x + 1 + i as u16 * 8;
            let py = palette_y + 1;
            if px + 6 < panel_x + panel_w && py < area.height.saturating_sub(2) {
                // Color swatch
                for dx in 0..6u16 {
                    let idx = (py * area.width + px + dx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = *color;
                        plane.cells[idx].char = ' ';
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }
    }
}

impl Scene for ColorPickerScene {
    fn scene_id(&self) -> &str { "color_picker" }

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
        draw_text(&mut plane, 2, 0, " Color Studio ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Sidebar divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * plane.width + SIDEBAR_WIDTH) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Left: ColorPicker
        let picker_area = Rect::new(0, 1, SIDEBAR_WIDTH, area.height.saturating_sub(2));
        self.render_picker_panel(&mut plane, picker_area);

        // Right: Palette panel
        let palette_area = Rect::new(SIDEBAR_WIDTH + 1, 1, area.width.saturating_sub(SIDEBAR_WIDTH + 2), area.height.saturating_sub(2));
        self.render_palette_panel(&mut plane, palette_area, SIDEBAR_WIDTH);

        // Footer
        let fy = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (fy * plane.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        let nav = " ↑↓←→: adjust | Enter: copy hex | Click: pick | Esc: back | ?: help ";
        draw_text(&mut plane, 2, fy, nav, t.fg_muted, t.surface, false);

        if self.show_help {
            render_help_overlay(&mut plane, area, t, "Color Studio Help", &[
                ("↑/↓/←/→", "Adjust HSL values"),
                ("Tab", "Cycle through sliders"),
                ("Enter", "Copy hex to clipboard"),
                ("Click", "Pick color directly"),
                ("Palette", "Click preset colors"),
                ("Esc", "Back to showcase"),
            ]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
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

        // Enter to copy hex
        if key.code == KeyCode::Enter {
            // In a real app, this would copy to clipboard
            // For demo, just log it
            self.dirty = true;
            return true;
        }

        // Forward to picker
        let area = self.area.get();
        let picker_area = Rect::new(1, 2, SIDEBAR_WIDTH.saturating_sub(2), area.height.saturating_sub(4));
        {
            let mut picker = self.picker.borrow_mut();
            picker.set_area(picker_area);
            if picker.handle_key(key) {
                self.selected_color = picker.color();
                self.selected_hex = picker.hex().to_string();
                self.add_recent(self.selected_color);
                self.dirty = true;
                return true;
            }
        }

        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();

        // Left panel: ColorPicker
        if col < SIDEBAR_WIDTH && row >= 2 && row < area.height.saturating_sub(2) {
            let picker_area = Rect::new(1, 2, SIDEBAR_WIDTH.saturating_sub(2), area.height.saturating_sub(4));
            let mut picker = self.picker.borrow_mut();
            picker.set_area(picker_area);
            if picker.handle_mouse(kind, col, row) {
                self.selected_color = picker.color();
                self.selected_hex = picker.hex().to_string();
                self.add_recent(self.selected_color);
                self.dirty = true;
                return true;
            }
        }

        // Right panel: Quick palette clicks
        let panel_x = SIDEBAR_WIDTH + 1;
        let palette_y = area.height.saturating_sub(5);
        if row >= palette_y + 1 && row < palette_y + 2 && col >= panel_x && col < area.width - 2 {
            let palette = [
                ("#ff6b6b", Color::Rgb(255, 107, 107)),
                ("#ffd93d", Color::Rgb(255, 217, 61)),
                ("#6bcb77", Color::Rgb(107, 203, 119)),
                ("#4d96ff", Color::Rgb(77, 150, 255)),
                ("#9b59b6", Color::Rgb(155, 89, 182)),
            ];
            if matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
                for (i, (hex, color)) in palette.iter().enumerate() {
                    let px = panel_x + 1 + i as u16 * 8;
                    if col >= px && col < px + 6 {
                        self.picker.borrow_mut().set_hex(hex);
                        self.selected_color = *color;
                        self.selected_hex = hex.to_string();
                        self.add_recent(self.selected_color);
                        self.dirty = true;
                        return true;
                    }
                }
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.picker.borrow_mut().on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}