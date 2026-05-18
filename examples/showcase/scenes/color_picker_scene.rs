//! Embedded Color Picker scene for the showcase.
//!
//! Demonstrates the ColorPicker widget with live preview:
//!   - Interactive color picker with hue/saturation/lightness
//!   - Live preview panel showing the selected color
//!   - Hex code display
//!   - Mouse and keyboard interaction

use crate::scenes::shared_helpers::draw_text;
use dracon_terminal_engine::compositor::plane::{Color, Plane};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widgets::color_picker::ColorPicker;
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

pub struct ColorPickerScene {
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
    picker: ColorPicker,
    selected_color: Color,
    selected_hex: String,
    dirty: bool,
    area: std::cell::Cell<Rect>,
}

impl ColorPickerScene {
    pub fn new(theme: Theme) -> Self {
        let initial_color = Color::Rgb(88, 166, 255);
        let picker = ColorPicker::with_color(initial_color)
            .with_theme(theme.clone());

        Self {
            theme: theme.clone(),
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            picker,
            selected_color: initial_color,
            selected_hex: "#58a6ff".into(),
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
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
        let title = " Color Picker ";
        draw_text(&mut plane, 2, 0, title, t.primary, t.bg, true);
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

        // ── Color Picker Widget (left side) ────────────────────────────────
        let picker_area = Rect::new(area.x + 2, area.y + 2, area.width / 2 - 2, area.height.saturating_sub(6));
        let picker_plane = self.picker.render(picker_area);
        // Blit picker plane
        for y in 0..picker_plane.height.min(picker_area.height) {
            for x in 0..picker_plane.width.min(picker_area.width) {
                let src_idx = (y * picker_plane.width + x) as usize;
                let dst_idx = ((picker_area.y + y) * area.width + picker_area.x + x) as usize;
                if src_idx < picker_plane.cells.len() && dst_idx < plane.cells.len() {
                    let src = &picker_plane.cells[src_idx];
                    if !src.transparent {
                        plane.cells[dst_idx] = *src;
                    }
                }
            }
        }

        // ── Preview Panel (right side) ─────────────────────────────────────
        let right_x = area.width / 2 + 2;
        draw_text(&mut plane, right_x, 2, "Preview", t.fg, t.bg, true);

        // Large color swatch
        let swatch_w = (area.width / 2 - 4) as usize;
        let swatch_h = 6usize;
        for y in 0..swatch_h {
            for x in 0..swatch_w {
                let px = right_x + x as u16;
                let py = 4 + y as u16;
                if px < area.width && py < area.height {
                    let idx = (py * area.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = self.selected_color;
                        plane.cells[idx].fg = self.selected_color;
                        plane.cells[idx].char = ' ';
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // Hex code
        let hex_label = format!("Hex: {}", self.selected_hex);
        draw_text(&mut plane, right_x, 4 + swatch_h as u16 + 1, &hex_label, t.fg, t.bg, true);

        // RGB values
        if let Color::Rgb(r, g, b) = self.selected_color {
            let rgb_label = format!("RGB: {}, {}, {}", r, g, b);
            draw_text(&mut plane, right_x, 4 + swatch_h as u16 + 2, &rgb_label, t.fg, t.bg, false);
        }

        // Color name suggestions
        draw_text(&mut plane, right_x, 4 + swatch_h as u16 + 4, "Click picker or use ↑↓←→", t.fg_muted, t.bg, false);
        draw_text(&mut plane, right_x, 4 + swatch_h as u16 + 5, "to change color", t.fg_muted, t.bg, false);

        // Contrast ratio with theme bg
        let contrast_y = 4 + swatch_h as u16 + 7;
        draw_text(&mut plane, right_x, contrast_y, "Contrast", t.secondary, t.bg, true);
        if let Color::Rgb(r, g, b) = self.selected_color {
            // Simple relative luminance
            let lum_fg = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
            let lum_bg = match t.bg {
                Color::Rgb(br, bg_, bb) => 0.299 * br as f64 + 0.587 * bg_ as f64 + 0.114 * bb as f64,
                _ => 0.0,
            };
            let (lighter, darker) = if lum_fg > lum_bg { (lum_fg, lum_bg) } else { (lum_bg, lum_fg) };
            let contrast = (lighter + 0.05) / (darker + 0.05);
            let (rating, color) = if contrast >= 7.0 { ("AAA", t.success) } else if contrast >= 4.5 { ("AA", t.info) } else if contrast >= 3.0 { ("AA Large", t.warning) } else { ("Fail", t.error) };
            let contrast_text = format!("{:.1}:1 ({})", contrast, rating);
            draw_text(&mut plane, right_x, contrast_y + 1, &contrast_text, color, t.bg, true);
        }

        // Recent colors row
        let recent_y = contrast_y + 3;
        draw_text(&mut plane, right_x, recent_y, "Recent", t.secondary, t.bg, true);
        // Show some recently-picked colors (simulated)
        let recent = [
            Color::Rgb(136, 192, 208), Color::Rgb(208, 135, 112),
            Color::Rgb(163, 190, 140), Color::Rgb(235, 203, 139),
            Color::Rgb(180, 142, 173),
        ];
        for (i, color) in recent.iter().enumerate() {
            let rx = right_x + i as u16 * 4;
            let idx = ((recent_y + 1) * plane.width + rx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = *color;
                plane.cells[idx].fg = *color;
                plane.cells[idx].char = ' ';
                plane.cells[idx].transparent = false;
            }
        }

        // ── Palette Strip ──────────────────────────────────────────────────
        let palette_y = 4 + swatch_h as u16 + 8;
        draw_text(&mut plane, 2, palette_y, "Quick Palette:", t.fg, t.bg, true);
        let palette = [
            ("#ff6b6b", Color::Rgb(255, 107, 107)),
            ("#ffd93d", Color::Rgb(255, 217, 61)),
            ("#6bcb77", Color::Rgb(107, 203, 119)),
            ("#4d96ff", Color::Rgb(77, 150, 255)),
            ("#9b59b6", Color::Rgb(155, 89, 182)),
            ("#e91e63", Color::Rgb(233, 30, 99)),
            ("#00bcd4", Color::Rgb(0, 188, 212)),
            ("#ff9800", Color::Rgb(255, 152, 0)),
        ];
        for (i, (hex, color)) in palette.iter().enumerate() {
            let px = 2 + i as u16 * 9;
            let py = palette_y + 2;
            if px + 7 < area.width && py + 1 < area.height {
                // Color swatch (2 cells)
                for dx in 0..7u16 {
                    let idx = (py * area.width + px + dx) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = *color;
                        plane.cells[idx].fg = *color;
                        plane.cells[idx].char = ' ';
                        plane.cells[idx].transparent = false;
                    }
                }
                // Hex label
                for (j, ch) in hex.chars().enumerate() {
                    let idx = ((py + 1) * area.width + px + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = t.fg_muted;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // ── Footer ────────────────────────────────────────────────────────
        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
        let footer = format!(
            " {}:help | {}:back | ↑↓←→:adjust color ",
            help_key, back_key,
        );
        let fy = area.height.saturating_sub(1);
        for (i, c) in footer.chars().enumerate() {
            let idx = (fy * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg_muted;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }

        // Help overlay
        if self.show_help {
            self.render_help(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.keybindings.matches(actions::BACK, &key) {
            if self.show_help {
                self.show_help = false;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            return true;
        }
        if self.show_help {
            return false;
        }

        // Forward to picker
        if self.picker.handle_key(key) {
            self.selected_color = self.picker.color();
            self.selected_hex = self.picker.hex().to_string();
            self.dirty = true;
            return true;
        }

        false
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = self.area.get();
        let picker_area = Rect::new(area.x + 2, area.y + 2, area.width / 2 - 2, area.height.saturating_sub(6));

        // Forward to picker if within its bounds
        if col >= picker_area.x && col < picker_area.x + picker_area.width
            && row >= picker_area.y && row < picker_area.y + picker_area.height
        {
            let local_col = col - picker_area.x;
            let local_row = row - picker_area.y;
            if self.picker.handle_mouse(kind, local_col, local_row) {
                self.selected_color = self.picker.color();
                self.selected_hex = self.picker.hex().to_string();
                self.dirty = true;
                return true;
            }
        }

        // Quick palette clicks
        let palette_y = 20; // approximate
        if row >= palette_y + 2 && row <= palette_y + 3 {
            let palette = [
                ("#ff6b6b", Color::Rgb(255, 107, 107)),
                ("#ffd93d", Color::Rgb(255, 217, 61)),
                ("#6bcb77", Color::Rgb(107, 203, 119)),
                ("#4d96ff", Color::Rgb(77, 150, 255)),
                ("#9b59b6", Color::Rgb(155, 89, 182)),
                ("#e91e63", Color::Rgb(233, 30, 99)),
                ("#00bcd4", Color::Rgb(0, 188, 212)),
                ("#ff9800", Color::Rgb(255, 152, 0)),
            ];
            if matches!(kind, MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left)) {
                for (i, (hex, color)) in palette.iter().enumerate() {
                    let px = 2 + i as u16 * 9;
                    if col >= px && col < px + 7 {
                        self.picker.set_hex(hex);
                        self.selected_color = *color;
                        self.selected_hex = hex.to_string();
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
        self.picker.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { self.dirty }

    fn mark_dirty(&mut self) { self.dirty = true; }

    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl ColorPickerScene {
    fn render_help(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 38u16.min(area.width.saturating_sub(4));
        let hh = 9u16.min(area.height.saturating_sub(4));
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

        let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
        let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");

        let lines = [
            ("╭──────────────────────────────────╮", true),
            ("│     Color Picker Help            │", true),
            ("├──────────────────────────────────┤", true),
            ("│  ↑/↓/←/→   Adjust HSL values    │", false),
            ("│  Click      Pick color directly  │", false),
            ("│  Palette    Click preset colors  │", false),
            (&format!("│  {:<10} Toggle this help       │", help_key), false),
            (&format!("│  {:<10} Dismiss / go back     │", back_key), false),
            ("╰──────────────────────────────────╯", true),
        ];

        for (i, (line, is_border)) in lines.iter().enumerate() {
            let ly = hy + i as u16;
            let lx = (area.width - line.len() as u16) / 2;
            for (j, ch) in line.chars().enumerate() {
                let px = lx + j as u16;
                if px < area.width && ly < area.height {
                    let idx = (ly * area.width + px) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if *is_border || "│╭╮├┤╰╯─".contains(ch) { t.outline } else { t.fg };
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }
    }
}
