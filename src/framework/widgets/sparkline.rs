//! Sparkline chart widget.
//!
//! A compact line chart widget that displays data as a simple line graph.
//! Auto-scales to the data range and supports custom colors.

use crate::compositor::{Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::{WidgetId, WidgetState};
use ratatui::layout::Rect;

/// Callback type for point click events.
pub type PointClickCallback = Box<dyn FnMut(usize, f64)>;

/// A sparkline chart widget - compact line chart for data visualization.
pub struct Sparkline {
    id: WidgetId,
    data: Vec<f64>,
    color: Color,
    fill_color: Option<Color>,
    height: u16,
    min_value: f64,
    max_value: f64,
    show_dots: bool,
    show_min_max: bool,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    hovered_point: Option<usize>,
    on_point_click: Option<PointClickCallback>,
    /// Enable exponential right-to-left color gradient.
    /// Recent data points (right) are bright, older points (left) fade subtly.
    gradient_enabled: bool,
    /// Minimum opacity for the oldest data point (0.0 = invisible, 1.0 = full).
    fade_opacity: f64,
    /// Exponential factor controlling fade curve.
    /// Higher values make recent points stand out more sharply.
    exponential_factor: f64,
}

impl Sparkline {
    /// Creates a new Sparkline with the given data.
    pub fn new(data: Vec<f64>) -> Self {
        let (min, max) = Self::compute_range(&data);
        Self {
            id: WidgetId::next(),
            data,
            color: Color::Ansi(12), // cyan
            fill_color: None,
            height: 3,
            min_value: min,
            max_value: max,
            show_dots: false,
            show_min_max: false,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 3)),
            dirty: true,
            hovered_point: None,
            on_point_click: None,
            gradient_enabled: true,
            fade_opacity: 0.15,
            exponential_factor: 2.0,
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the line color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the fill color (area below the line).
    pub fn with_fill_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// Sets the height of the sparkline in cells.
    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height.clamp(1, 10);
        self
    }

    /// Shows dots at each data point.
    pub fn with_dots(mut self, show: bool) -> Self {
        self.show_dots = show;
        self
    }

    /// Shows min/max labels.
    pub fn with_min_max(mut self, show: bool) -> Self {
        self.show_min_max = show;
        self
    }

    /// Enables or disables the exponential right-to-left color gradient.
    /// When enabled, recent data points (right) are bright and older points (left) fade subtly.
    pub fn with_gradient(mut self, enabled: bool) -> Self {
        self.gradient_enabled = enabled;
        self
    }

    /// Sets the minimum opacity for the oldest data point.
    /// Range: 0.0 (invisible) to 1.0 (full color). Default: 0.15.
    pub fn with_fade_opacity(mut self, opacity: f64) -> Self {
        self.fade_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Sets the exponential factor for the gradient curve.
    /// Higher values make recent points stand out more sharply. Default: 2.0.
    pub fn with_exponential_factor(mut self, factor: f64) -> Self {
        self.exponential_factor = factor.max(0.5);
        self
    }

    /// Sets the data and recomputes the range.
    pub fn with_data(mut self, data: Vec<f64>) -> Self {
        let (min, max) = Self::compute_range(&data);
        self.data = data;
        self.min_value = min;
        self.max_value = max;
        self.dirty = true;
        self
    }

    /// Registers a callback invoked when a point is clicked.
    pub fn on_point_click(mut self, f: impl FnMut(usize, f64) + 'static) -> Self {
        self.on_point_click = Some(Box::new(f));
        self
    }

    /// Sets the data and recomputes range.
    pub fn set_data(&mut self, data: Vec<f64>) {
        self.data = data;
        let (min, max) = Self::compute_range(&self.data);
        self.min_value = min;
        self.max_value = max;
        self.dirty = true;
    }

    /// Sets a single data point value.
    pub fn set_value(&mut self, index: usize, value: f64) {
        if index < self.data.len() {
            self.data[index] = value;
            let (min, max) = Self::compute_range(&self.data);
            self.min_value = min;
            self.max_value = max;
            self.dirty = true;
        }
    }

    /// Pushes a new data point.
    pub fn push(&mut self, value: f64) {
        self.data.push(value);
        let (min, max) = Self::compute_range(&self.data);
        self.min_value = min;
        self.max_value = max;
        self.dirty = true;
    }

    /// Clears all data.
    pub fn clear(&mut self) {
        self.data.clear();
        self.min_value = 0.0;
        self.max_value = 0.0;
        self.dirty = true;
    }

    fn compute_range(data: &[f64]) -> (f64, f64) {
        if data.is_empty() {
            return (0.0, 1.0);
        }

        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Add padding to range
        let range = if (max - min).abs() < f64::EPSILON {
            1.0
        } else {
            max - min
        };

        (min - range * 0.1, max + range * 0.1)
    }

    fn value_to_y(&self, value: f64, height: u16) -> u16 {
        let range = self.max_value - self.min_value;
        if range.abs() < f64::EPSILON {
            return height / 2;
        }
        let normalized = (value - self.min_value) / range;
        let y = height.saturating_sub(1) - (normalized * (height - 1) as f64).round() as u16;
        y.min(height.saturating_sub(1))
    }

    /// Computes gradient opacity for a point at the given index.
    /// Returns a value between `fade_opacity` and 1.0 using an exponential curve.
    fn gradient_opacity(&self, index: usize, num_points: usize) -> f64 {
        if !self.gradient_enabled || num_points <= 1 {
            return 1.0;
        }
        // Position: 0.0 = oldest (left), 1.0 = newest (right)
        let position = index as f64 / (num_points - 1) as f64;
        // Exponential curve: recent points are bright, older fade
        let curve = position.powf(self.exponential_factor);
        self.fade_opacity + (1.0 - self.fade_opacity) * curve
    }
}

impl crate::framework::widget::Widget for Sparkline {
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

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        // Intentionally do not clear dirty - sparklines may need frequent updates
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;
        plane.fill_bg(self.theme.bg);

        if self.data.is_empty() {
            // Show empty state
            let text = "No data";
            let x = (area.width.saturating_sub(text.len() as u16)) / 2;
            for (i, ch) in text.chars().enumerate() {
                let idx = ((area.height / 2) * area.width + x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.fg_muted;
                }
            }
            return plane;
        }

        let height = self.height.min(area.height);

        // Draw grid lines (subtle)
        for y in 0..height {
            for x in 0..area.width {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.bg;
                }
            }
        }

        // Draw horizontal grid lines
        for i in 0..height {
            for x in 0..area.width {
                let idx = (i * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.surface_elevated;
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = self.theme.outline;
                    plane.cells[idx].style = Styles::DIM;
                }
            }
        }

        // Calculate points
        let num_points = self.data.len();
        let width = area.width.saturating_sub(2); // Leave padding
        if width == 0 || height == 0 {
            return plane;
        }

        let points: Vec<(u16, u16)> = (0..num_points)
            .map(|i| {
                let x = if num_points > 1 {
                    (i as f64 / (num_points - 1) as f64 * (width - 1) as f64).round() as u16 + 1
                } else {
                    1
                };
                let y = self.value_to_y(self.data[i], height);
                (x, y)
            })
            .collect();

        // Draw fill area if configured
        if let Some(fill) = self.fill_color {
            for i in 0..points.len().saturating_sub(1) {
                let (x1, y1) = points[i];
                let (x2, y2) = points[i + 1];

                let start_x = x1.min(x2);
                let end_x = x1.max(x2);

                for x in start_x..=end_x {
                    let t = if end_x == start_x {
                        0.5
                    } else {
                        (x - start_x) as f64 / (end_x - start_x) as f64
                    };
                    let y_fill = ((1.0 - t) * y1 as f64 + t * y2 as f64).round() as u16;

                    for y in y_fill..height {
                        let idx = (y * area.width + x) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].bg = fill;
                            plane.cells[idx].transparent = false;
                        }
                    }
                }
            }
        }

        // Draw line segments
        for i in 0..points.len().saturating_sub(1) {
            let (x1, y1) = points[i];
            let (x2, y2) = points[i + 1];

            // Draw line using Bresenham's algorithm
            let dx = (x2 as i32 - x1 as i32).abs();
            let dy = (y2 as i32 - y1 as i32).abs();
            let sx = if x1 < x2 { 1 } else { -1 };
            let sy = if y1 < y2 { 1 } else { -1 };
            let mut err = dx - dy;

            let mut x = x1 as i32;
            let mut y = y1 as i32;

            loop {
                if y >= 0 && y < height as i32 && x >= 0 && x < area.width as i32 {
                    let idx = (y as u16 * area.width + x as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '●';
                        plane.cells[idx].fg = self.color;
                        plane.cells[idx].bg = self.theme.bg;
                    }
                }

                if x == x2 as i32 && y == y2 as i32 {
                    break;
                }

                let e2 = 2 * err;
                if e2 > -dy {
                    err -= dy;
                    x += sx;
                }
                if e2 < dx {
                    err += dx;
                    y += sy;
                }
            }
        }

        // Draw dots at data points if configured
        if self.show_dots {
            for (i, &(x, y)) in points.iter().enumerate() {
                let is_hovered = self.hovered_point == Some(i);
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '◉';
                    plane.cells[idx].fg = self.color;
                    plane.cells[idx].style = if is_hovered {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                }
            }
        }

        // Draw min/max values if configured
        if self.show_min_max && num_points > 0 {
            let min_text = format!("{:.1}", self.min_value);
            let max_text = format!("{:.1}", self.max_value);

            // Min on left
            for (i, ch) in min_text.chars().enumerate().take(5) {
                let idx = ((height - 1) * area.width + 1 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.success;
                }
            }

            // Max on right
            for (i, ch) in max_text.chars().enumerate().take(5) {
                let x = area.width.saturating_sub(6 + i as u16);
                let idx = (area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.error;
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, _key: crate::input::event::KeyEvent) -> bool {
        false
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        let area = self.area.get();

        match kind {
            crate::input::event::MouseEventKind::Moved => {
                if self.data.is_empty() || area.width < 3 {
                    return false;
                }

                let rel_col = col.saturating_sub(area.x);
                let rel_row = row.saturating_sub(area.y);

                if rel_row >= self.height {
                    return false;
                }

                // Determine which point is hovered
                let num_points = self.data.len();
                let width = area.width.saturating_sub(2);
                let point_width = if num_points > 1 {
                    width as f64 / (num_points - 1) as f64
                } else {
                    width as f64
                };

                let point_idx = if num_points == 1 {
                    0
                } else {
                    (((rel_col.saturating_sub(1) as f64) / point_width).round() as usize)
                        .min(num_points - 1)
                };

                if self.hovered_point != Some(point_idx) {
                    self.hovered_point = Some(point_idx);
                    self.dirty = true;
                }
                true
            }
            crate::input::event::MouseEventKind::Down(_) => {
                if let Some(idx) = self.hovered_point {
                    if let Some(ref mut cb) = self.on_point_click {
                        cb(idx, self.data[idx]);
                    }
                    self.dirty = true;
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

impl WidgetState for Sparkline {
    fn state_id(&self) -> Option<&str> {
        None
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn apply_json(&mut self, _json: &serde_json::Value) -> Result<(), crate::error::DraconError> {
        Ok(())
    }
}
