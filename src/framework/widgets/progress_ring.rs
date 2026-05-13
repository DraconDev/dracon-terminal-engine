//! Progress ring widget.
//!
//! A circular progress indicator using Unicode block characters.

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;

/// Callback type for when the value changes.
pub type ProgressChangeCallback = Box<dyn FnMut(f64)>;

/// A circular progress indicator widget.
pub struct ProgressRing {
    id: WidgetId,
    progress: f64, // 0.0 to 1.0
    size: u16,
    color: Color,
    bg_color: Color,
    show_percentage: bool,
    label: Option<String>,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    on_change: Option<ProgressChangeCallback>,
}

impl ProgressRing {
    /// Creates a new ProgressRing with the given initial progress.
    pub fn new(progress: f64) -> Self {
        Self {
            id: WidgetId::default_id(),
            progress: progress.clamp(0.0, 1.0),
            size: 5,
            color: Color::Ansi(12), // cyan
            bg_color: Color::Ansi(8), // dark gray
            show_percentage: true,
            label: None,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 9, 5)),
            dirty: true,
            on_change: None,
        }
    }

    /// Creates a ProgressRing with default 50% progress.
    pub fn default() -> Self {
        Self::new(0.5)
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the progress value (0.0 to 1.0).
    pub fn with_progress(mut self, progress: f64) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Sets the ring size in cells.
    pub fn with_size(mut self, size: u16) -> Self {
        self.size = size.clamp(3, 15);
        self
    }

    /// Sets the progress color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the background ring color.
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Sets whether to show the percentage text.
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Sets a label to display below the percentage.
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Registers a callback invoked when the progress changes.
    pub fn on_change(mut self, f: impl FnMut(f64) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Sets the progress value (0.0 to 1.0).
    pub fn set_progress(&mut self, progress: f64) {
        let new_progress = progress.clamp(0.0, 1.0);
        if (self.progress - new_progress).abs() > f64::EPSILON {
            self.progress = new_progress;
            self.dirty = true;
            if let Some(ref mut cb) = self.on_change {
                cb(self.progress);
            }
        }
    }

    /// Returns the current progress value.
    pub fn progress(&self) -> f64 {
        self.progress
    }

    /// Increments the progress by a given amount.
    pub fn increment(&mut self, amount: f64) {
        self.set_progress(self.progress + amount);
    }

    /// Decrements the progress by a given amount.
    pub fn decrement(&mut self, amount: f64) {
        self.set_progress(self.progress - amount);
    }
}

impl Default for ProgressRing {
    fn default() -> Self {
        Self::new(0.5)
    }
}

impl crate::framework::widget::Widget for ProgressRing {
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

        // Calculate ring dimensions
        let ring_width = self.size.min(area.width.saturating_sub(2));
        let ring_height = ring_width * 2; // Ring is taller than wide

        let center_x = area.width / 2;
        let center_y = (area.height - ring_height) / 2 + ring_height / 2;

        // Draw the ring using block characters
        let radius = ring_width as f64 / 2.0;
        let inner_radius = radius - 1.0;

        // Ring characters for different progress positions
        // The ring is drawn using quarter characters
        let segments = 24; // Granularity of the ring
        let progress_segments = (self.progress * segments as f64) as i32;

        // Calculate position for each character in the ring
        let chars_per_row = ring_width as i32;
        let rows = ring_height as i32;

        let mut filled_cells: Vec<(i32, i32)> = Vec::new();
        let mut empty_cells: Vec<(i32, i32)> = Vec::new();

        for y in 0..rows {
            for x in 0..chars_per_row {
                let dx = x as f64 - radius + 0.5;
                let dy = y as f64 - radius + 0.5;
                let dist = (dx * dx + dy * dy).sqrt();

                if (dist - radius).abs() < 0.5 {
                    let angle = (-dy).atan2(dx) + std::f64::consts::FRAC_PI_2;
                    let normalized_angle = if angle < 0.0 {
                        angle + 2.0 * std::f64::consts::PI
                    } else {
                        angle
                    };
                    let segment = ((normalized_angle / (2.0 * std::f64::consts::PI)) * segments as f64) as i32;
                    let segment = segment.min(segments - 1);

                    if segment < progress_segments {
                        filled_cells.push((x, y));
                    } else {
                        empty_cells.push((x, y));
                    }
                }
            }
        }

        // Draw empty ring first
        for &(x, y) in &empty_cells {
            let px = center_x - ring_width / 2 + x as u16;
            let py = center_y - ring_height / 2 + y as u16;

            if px < area.width && py < area.height {
                let idx = (py * area.width + px) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '○';
                    plane.cells[idx].fg = self.bg_color;
                    plane.cells[idx].bg = self.theme.bg;
                }
            }
        }

        // Draw filled ring
        for &(x, y) in &filled_cells {
            let px = center_x - ring_width / 2 + x as u16;
            let py = center_y - ring_height / 2 + y as u16;

            if px < area.width && py < area.height {
                let idx = (py * area.width + px) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '●';
                    plane.cells[idx].fg = self.color;
                    plane.cells[idx].bg = self.theme.bg;
                }
            }
        }

        // Draw percentage text in center
        if self.show_percentage {
            let pct = (self.progress * 100.0).round() as i32;
            let pct_text = format!("{}%", pct);
            let text_len = pct_text.len() as u16;
            let text_x = (area.width.saturating_sub(text_len)) / 2;
            let text_y = center_y.saturating_sub(1);

            for (i, ch) in pct_text.chars().enumerate() {
                let idx = (text_y * area.width + text_x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.color;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }
        }

        // Draw label below the ring
        if let Some(ref label) = self.label {
            let label_y = center_y + ring_height / 2 + 1;
            if label_y < area.height {
                let label_len = label.len() as u16;
                let label_x = (area.width.saturating_sub(label_len)) / 2;

                for (i, ch) in label.chars().enumerate() {
                    let idx = (label_y * area.width + label_x + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = self.theme.fg_muted;
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }

        match key.code {
            KeyCode::Left | KeyCode::Down => {
                self.decrement(0.05);
                true
            }
            KeyCode::Right | KeyCode::Up => {
                self.increment(0.05);
                true
            }
            KeyCode::Home => {
                self.set_progress(0.0);
                true
            }
            KeyCode::End => {
                self.set_progress(1.0);
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        match kind {
            crate::input::event::MouseEventKind::Down(btn) if btn == crate::input::event::MouseButton::Left => {
                let area = self.area.get();
                let rel_x = col.saturating_sub(area.x) as i32;
                let rel_y = row.saturating_sub(area.y) as i32;

                let center_x = area.width as i32 / 2;
                let center_y = area.height as i32 / 2;

                let dx = rel_x - center_x;
                let dy = rel_y - center_y;

                // Calculate angle from center
                let angle = (-dy as f64).atan2(dx as f64);
                let normalized = (angle + std::f64::consts::FRAC_PI_2) / (2.0 * std::f64::consts::PI);
                let progress = if normalized < 0.0 {
                    normalized + 1.0
                } else {
                    normalized
                };

                self.set_progress(progress);
                true
            }
            crate::input::event::MouseEventKind::Drag(_) => {
                let area = self.area.get();
                let rel_x = col.saturating_sub(area.x) as i32;
                let rel_y = row.saturating_sub(area.y) as i32;

                let center_x = area.width as i32 / 2;
                let center_y = area.height as i32 / 2;

                let dx = rel_x - center_x;
                let dy = rel_y - center_y;

                let angle = (-dy as f64).atan2(dx as f64);
                let normalized = (angle + std::f64::consts::FRAC_PI_2) / (2.0 * std::f64::consts::PI);
                let progress = if normalized < 0.0 {
                    normalized + 1.0
                } else {
                    normalized
                };

                self.set_progress(progress);
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}