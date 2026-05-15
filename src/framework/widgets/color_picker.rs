//! ColorPicker widget with HSL sliders and hex input.
//!
//! A color picker widget that allows users to select colors using:
//! - HSL (Hue, Saturation, Lightness) sliders
//! - Hex input field
//! - Live color preview swatch

use crate::compositor::{Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use ratatui::layout::Rect;

/// Callback type for color change events.
pub type ColorChangeCallback = Box<dyn FnMut(Color)>;

/// A color picker widget with HSL sliders and hex input.
pub struct ColorPicker {
    id: WidgetId,
    /// Current hue (0-360)
    hue: f32,
    /// Current saturation (0-100)
    saturation: f32,
    /// Current lightness (0-100)
    lightness: f32,
    /// Current hex string
    hex_value: String,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    hovered_slider: Option<SliderKind>,
    selected_slider: Option<SliderKind>,
    on_color_change: Option<ColorChangeCallback>,
    input_focused: bool,
}

/// The kind of slider in the color picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderKind {
    Hue,
    Saturation,
    Lightness,
}

impl ColorPicker {
    /// Creates a new ColorPicker with default color (red).
    pub fn new() -> Self {
        Self {
            id: WidgetId::next(),
            hue: 0.0,
            saturation: 100.0,
            lightness: 50.0,
            hex_value: "#FF0000".to_string(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 12)),
            dirty: true,
            hovered_slider: None,
            selected_slider: None,
            on_color_change: None,
            input_focused: false,
        }
    }

    /// Creates a ColorPicker with a specific starting color.
    pub fn with_color(color: Color) -> Self {
        let (h, s, l) = color_to_hsl(color);
        let hex = color_to_hex(color);
        Self {
            id: WidgetId::next(),
            hue: h,
            saturation: s,
            lightness: l,
            hex_value: hex,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 12)),
            dirty: true,
            hovered_slider: None,
            selected_slider: None,
            on_color_change: None,
            input_focused: false,
        }
    }

    /// Creates a ColorPicker with a hex string.
    pub fn with_hex(hex: &str) -> Self {
        let color = hex_to_color(hex);
        let (h, s, l) = color_to_hsl(color);
        Self {
            id: WidgetId::next(),
            hue: h,
            saturation: s,
            lightness: l,
            hex_value: hex.to_uppercase(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 12)),
            dirty: true,
            hovered_slider: None,
            selected_slider: None,
            on_color_change: None,
            input_focused: false,
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Registers a callback invoked when the color changes.
    pub fn on_color_change(mut self, f: impl FnMut(Color) + 'static) -> Self {
        self.on_color_change = Some(Box::new(f));
        self
    }

    /// Returns the current color.
    pub fn color(&self) -> Color {
        hsl_to_color(self.hue, self.saturation, self.lightness)
    }

    /// Returns the current hex string.
    pub fn hex(&self) -> &str {
        &self.hex_value
    }

    /// Sets the color from HSL values.
    pub fn set_hsl(&mut self, hue: f32, saturation: f32, lightness: f32) {
        self.hue = hue.clamp(0.0, 360.0);
        self.saturation = saturation.clamp(0.0, 100.0);
        self.lightness = lightness.clamp(0.0, 100.0);
        self.hex_value = color_to_hex(self.color());
        self.dirty = true;
    }

    /// Sets the color from a hex string.
    pub fn set_hex(&mut self, hex: &str) {
        let cleaned = hex.trim_start_matches('#').to_uppercase();
        if is_valid_hex(&cleaned) {
            self.hex_value = format!("#{}", cleaned);
            let color = hex_to_color(&self.hex_value);
            let (h, s, l) = color_to_hsl(color);
            self.hue = h;
            self.saturation = s;
            self.lightness = l;
            self.dirty = true;
        }
    }

    fn update_color(&mut self) {
        let color = self.color();
        self.hex_value = color_to_hex(color);
        if let Some(ref mut cb) = self.on_color_change {
            cb(color);
        }
    }
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::framework::widget::Widget for ColorPicker {
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
        self.dirty = true;
    }

    fn z_index(&self) -> u16 {
        10
    }

    fn focusable(&self) -> bool {
        true
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

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;
        plane.fill_bg(self.theme.bg);

        let current_color = self.color();

        // === Color Preview Swatch ===
        let swatch_width = 8u16.min(area.width.saturating_sub(2));
        let swatch_height = 4u16.min(area.height.saturating_sub(2));

        for y in 0..swatch_height {
            for x in 0..swatch_width {
                let idx = (y * area.width + x + 1) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = current_color;
                    plane.cells[idx].char = ' ';
                }
            }
        }

        // Swatch border
        for x in 0..swatch_width {
            let top_idx = (swatch_height * area.width + x + 1) as usize;
            let bot_idx = ((swatch_height - 1) * area.width + x + 1) as usize;
            if top_idx < plane.cells.len() { plane.cells[top_idx].char = '─'; plane.cells[top_idx].fg = self.theme.outline; }
            if bot_idx < plane.cells.len() { plane.cells[bot_idx].char = '─'; plane.cells[bot_idx].fg = self.theme.outline; }
        }
        for y in 0..swatch_height {
            let left_idx = (y * area.width) as usize;
            let right_idx = (y * area.width + swatch_width + 1) as usize;
            if left_idx < plane.cells.len() { plane.cells[left_idx].char = '│'; plane.cells[left_idx].fg = self.theme.outline; }
            if right_idx < plane.cells.len() { plane.cells[right_idx].char = '│'; plane.cells[right_idx].fg = self.theme.outline; }
        }
        // Corners
        let corners = [
            (0, 0, '┌'),
            (swatch_width + 1, 0, '┐'),
            (0, swatch_height, '└'),
            (swatch_width + 1, swatch_height, '┘'),
        ];
        for (x, y, ch) in corners {
            let idx = (y * area.width + x + 1) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = self.theme.outline; }
        }

        // === Hex Input Display ===
        let hex_x = swatch_width + 4;
        let hex_label = "Hex: ";
        for (i, ch) in hex_label.chars().enumerate() {
            let idx = (area.width + hex_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = self.theme.fg_muted;
            }
        }

        let hex_start = hex_x + hex_label.len() as u16;
        let hex_display = self.hex_value.clone();
        let hex_bg = if self.input_focused {
            self.theme.primary_active
        } else {
            self.theme.surface_elevated
        };

        for (i, ch) in hex_display.chars().enumerate().take(8) {
            let idx = (area.width + hex_start + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = if self.input_focused {
                    self.theme.primary
                } else {
                    self.theme.fg
                };
                plane.cells[idx].bg = hex_bg;
            }
        }

        // === Sliders ===
        let slider_start_y = swatch_height + 2;
        let slider_width = (area.width.saturating_sub(4)).max(20);

        // Hue slider
        self.render_slider((&mut plane, 0, slider_start_y, slider_width, "H", self.hue / 360.0 * 100.0, SliderKind::Hue));

        // Saturation slider
        self.render_slider((&mut plane, 1, slider_start_y + 2, slider_width, "S", self.saturation, SliderKind::Saturation));

        // Lightness slider
        self.render_slider((&mut plane, 2, slider_start_y + 4, slider_width, "L", self.lightness, SliderKind::Lightness));

        // === HSL Values Display ===
        let values_y = slider_start_y + 7;
        let values_text = format!(
            "  H:{:>3.0}°  S:{:>3.0}%  L:{:>3.0}%  ",
            self.hue, self.saturation, self.lightness
        );
        for (i, ch) in values_text.chars().enumerate() {
            let idx = (values_y * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = self.theme.fg_muted;
            }
        }

        // === Instructions ===
        let instr_y = area.height.saturating_sub(1);
        let instructions = "Click/drag sliders to adjust | Type hex to set color";
        for (i, ch) in instructions.chars().enumerate() {
            let idx = (instr_y * area.width + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].style = Styles::DIM;
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Handle hex input when focused
        if self.input_focused {
            match key.code {
                KeyCode::Esc => {
                    self.input_focused = false;
                    self.dirty = true;
                    true
                }
                KeyCode::Enter => {
                    let hex = self.hex_value.clone();
                    self.set_hex(&hex);
                    self.input_focused = false;
                    self.update_color();
                    true
                }
                KeyCode::Char(c) if c.is_ascii_hexdigit() => {
                    if self.hex_value.len() < 7 {
                        self.hex_value.push(c.to_ascii_uppercase());
                        self.dirty = true;
                    }
                    true
                }
                KeyCode::Backspace => {
                    if self.hex_value.len() > 1 {
                        self.hex_value.pop();
                        self.dirty = true;
                    }
                    true
                }
                _ => false,
            }
        } else {
            // Slider navigation
            match key.code {
                KeyCode::Tab => {
                    // Cycle through sliders
                    let sliders = [SliderKind::Hue, SliderKind::Saturation, SliderKind::Lightness];
                    let current = self.selected_slider.unwrap_or(SliderKind::Hue);
                    let idx = sliders.iter().position(|&s| s == current).unwrap_or(0);
                    self.selected_slider = Some(sliders[(idx + 1) % sliders.len()]);
                    self.dirty = true;
                    true
                }
                KeyCode::Left => {
                    if let Some(slider) = self.selected_slider {
                        self.adjust_slider(slider, -5.0);
                    }
                    true
                }
                KeyCode::Right => {
                    if let Some(slider) = self.selected_slider {
                        self.adjust_slider(slider, 5.0);
                    }
                    true
                }
                KeyCode::Up => {
                    if let Some(slider) = self.selected_slider {
                        self.adjust_slider(slider, 1.0);
                    }
                    true
                }
                KeyCode::Down => {
                    if let Some(slider) = self.selected_slider {
                        self.adjust_slider(slider, -1.0);
                    }
                    true
                }
                KeyCode::Enter => {
                    if self.selected_slider.is_none() {
                        self.selected_slider = Some(SliderKind::Hue);
                    }
                    self.input_focused = true;
                    self.dirty = true;
                    true
                }
                _ => false,
            }
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        let area = self.area.get();
        let rel_col = col.saturating_sub(area.x);
        let rel_row = row.saturating_sub(area.y);

        match kind {
            crate::input::event::MouseEventKind::Moved => {
                // Determine which slider is hovered
                let slider_start_y = 6u16;
                let _slider_width = (area.width.saturating_sub(4)).max(20);

                for (i, kind) in [SliderKind::Hue, SliderKind::Saturation, SliderKind::Lightness].iter().enumerate() {
                    let slider_y = slider_start_y + (i as u16) * 2;
                    if rel_row == slider_y {
                        self.hovered_slider = Some(*kind);
                    }
                }

                // Check if over hex input
                let swatch_width = 8u16.min(area.width.saturating_sub(2));
                let hex_x = swatch_width + 4 + 5; // After "Hex: " label
                if rel_row == 1 && rel_col >= hex_x && rel_col < hex_x + 8 {
                    self.input_focused = true;
                    self.dirty = true;
                }
                true
            }
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                let slider_start_y = 6u16;
                let slider_width = (area.width.saturating_sub(4)).max(20);

                for (i, kind) in [SliderKind::Hue, SliderKind::Saturation, SliderKind::Lightness].iter().enumerate() {
                    let slider_y = slider_start_y + (i as u16) * 2;
                    if rel_row == slider_y && rel_col >= 2 && rel_col < 2 + slider_width {
                        self.selected_slider = Some(*kind);
                        self.update_slider_from_position(*kind, rel_col - 2, slider_width);
                        self.dirty = true;
                    }
                }

                // Click on hex input
                let swatch_width = 8u16.min(area.width.saturating_sub(2));
                let hex_x = swatch_width + 4 + 5;
                if rel_row == 1 && rel_col >= hex_x && rel_col < hex_x + 8 {
                    self.input_focused = true;
                    self.dirty = true;
                }
                true
            }
            crate::input::event::MouseEventKind::Drag(_) => {
                if let Some(slider) = self.selected_slider {
                    let slider_start_y = 6u16;
                    let slider_width = (area.width.saturating_sub(4)).max(20);
                    let slider_y = match slider {
                        SliderKind::Hue => slider_start_y,
                        SliderKind::Saturation => slider_start_y + 2,
                        SliderKind::Lightness => slider_start_y + 4,
                    };
                    if rel_row == slider_y && rel_col >= 2 && rel_col < 2 + slider_width {
                        self.update_slider_from_position(slider, rel_col - 2, slider_width);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = theme.clone();
    }
}

impl WidgetState for ColorPicker {
    fn state_id(&self) -> Option<&str> {
        Some("color_picker")
    }

    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "selected_color": self.hex_value,
        })
    }

    fn apply_json(&mut self, json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        if let Some(hex) = json.get("selected_color").and_then(|v| v.as_str()) {
            self.set_hex(hex);
        }
        self.dirty = true;
        Ok(())
    }
}

/// Slider render params: (plane, idx, y, width, label, value, kind)
type SliderParams<'a> = (&'a mut Plane, u16, u16, u16, &'a str, f32, SliderKind);

impl ColorPicker {
    fn render_slider(&self, params: SliderParams) {
        let (plane, _idx, y, width, label, value, kind) = params;
        let area = self.area.get();
        let x = 2u16;

        // Label
        for (i, ch) in label.chars().enumerate() {
            let cell_idx = (y * area.width + x + i as u16) as usize;
            if cell_idx < plane.cells.len() {
                plane.cells[cell_idx].char = ch;
                plane.cells[cell_idx].fg = self.theme.fg_muted;
            }
        }

        // Slider track
        let track_x = x + 3;
        let track_width = width.saturating_sub(6);

        for i in 0..track_width {
            let cell_idx = (y * area.width + track_x + i) as usize;
            if cell_idx < plane.cells.len() {
                // Color the track based on slider type
                let progress = i as f32 / track_width as f32;
                let cell_color = match kind {
                    SliderKind::Hue => hsl_to_color(progress * 360.0, 100.0, 50.0),
                    SliderKind::Saturation => hsl_to_color(self.hue, progress * 100.0, self.lightness),
                    SliderKind::Lightness => {
                        let l = progress * 100.0;
                        Color::Rgb((l * 2.55) as u8, (l * 2.55) as u8, (l * 2.55) as u8)
                    }
                };
                plane.cells[cell_idx].char = '█';
                plane.cells[cell_idx].fg = cell_color;
            }
        }

        // Slider thumb
        let thumb_pos = ((value / 100.0) * track_width as f32) as u16;
        let thumb_idx = (y * area.width + track_x + thumb_pos) as usize;
        if thumb_idx < plane.cells.len() {
            plane.cells[thumb_idx].char = '◆';
            plane.cells[thumb_idx].fg = self.theme.primary;
            plane.cells[thumb_idx].style = Styles::BOLD;
        }

        // Value display
        let value_text = format!("{:>3.0}", value);
        let value_x = track_x + track_width + 2;
        for (i, ch) in value_text.chars().enumerate() {
            let cell_idx = (y * area.width + value_x + i as u16) as usize;
            if cell_idx < plane.cells.len() {
                plane.cells[cell_idx].char = ch;
                plane.cells[cell_idx].fg = if self.selected_slider == Some(kind) {
                    self.theme.primary
                } else {
                    self.theme.fg
                };
            }
        }

        // Highlight if hovered or selected
        let is_active = self.hovered_slider == Some(kind) || self.selected_slider == Some(kind);
        if is_active {
            for i in 0..track_width {
                let cell_idx = (y * area.width + track_x + i) as usize;
                if cell_idx < plane.cells.len() {
                    let bg_idx = (y * area.width + track_x + i) as usize;
                    if bg_idx < plane.cells.len() {
                        plane.cells[bg_idx].bg = self.theme.hover_bg;
                    }
                }
            }
        }
    }

    fn adjust_slider(&mut self, kind: SliderKind, delta: f32) {
        match kind {
            SliderKind::Hue => {
                self.hue = (self.hue + delta).rem_euclid(360.0);
            }
            SliderKind::Saturation => {
                self.saturation = (self.saturation + delta).clamp(0.0, 100.0);
            }
            SliderKind::Lightness => {
                self.lightness = (self.lightness + delta).clamp(0.0, 100.0);
            }
        }
        self.update_color();
        self.dirty = true;
    }

    fn update_slider_from_position(&mut self, kind: SliderKind, pos: u16, track_width: u16) {
        let value = (pos as f32 / track_width as f32 * 100.0).clamp(0.0, 100.0);
        match kind {
            SliderKind::Hue => {
                self.hue = value / 100.0 * 360.0;
            }
            SliderKind::Saturation => {
                self.saturation = value;
            }
            SliderKind::Lightness => {
                self.lightness = value;
            }
        }
        self.update_color();
    }
}

// ============================================================================
// Color Conversion Utilities
// ============================================================================

fn color_to_hsl(color: Color) -> (f32, f32, f32) {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
        Color::Ansi(n) => {
            let (r, g, b) = ansi_to_rgb(n);
            (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        _ => (0.0, 0.0, 0.0),
    };

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        return (0.0, 0.0, l * 100.0);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h * 360.0, s * 100.0, l * 100.0)
}

fn hsl_to_color(h: f32, s: f32, l: f32) -> Color {
    let s = s / 100.0;
    let l = l / 100.0;

    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return Color::Rgb(v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h = h / 360.0;

    fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
        let t = if t < 0.0 { t + 1.0 } else { t };
        let t = if t > 1.0 { t - 1.0 } else { t };
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    let r = (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0) as u8;
    let g = (hue_to_rgb(p, q, h) * 255.0) as u8;
    let b = (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0) as u8;

    Color::Rgb(r, g, b)
}

fn color_to_hex(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        }
        _ => "#000000".to_string(),
    }
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Rgb(0, 0, 0);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    Color::Rgb(r, g, b)
}

fn is_valid_hex(hex: &str) -> bool {
    if hex.len() != 6 {
        return false;
    }
    hex.chars().all(|c| c.is_ascii_hexdigit())
}

fn ansi_to_rgb(n: u8) -> (u8, u8, u8) {
    // Standard ANSI colors (simplified)
    match n % 16 {
        0 => (0, 0, 0),         // Black
        1 => (128, 0, 0),       // Red
        2 => (0, 128, 0),       // Green
        3 => (128, 128, 0),     // Yellow
        4 => (0, 0, 128),       // Blue
        5 => (128, 0, 128),     // Magenta
        6 => (0, 128, 128),     // Cyan
        7 => (192, 192, 192),   // White
        8 => (128, 128, 128),   // Bright Black
        9 => (255, 0, 0),       // Bright Red
        10 => (0, 255, 0),      // Bright Green
        11 => (255, 255, 0),    // Bright Yellow
        12 => (0, 0, 255),      // Bright Blue
        13 => (255, 0, 255),    // Bright Magenta
        14 => (0, 255, 255),    // Bright Cyan
        15 => (255, 255, 255),  // Bright White
        _ => (128, 128, 128),
    }
}