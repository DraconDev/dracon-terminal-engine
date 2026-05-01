//! Profiler widget for displaying performance metrics.
//!
//! Shows timing and resource usage data for UI operations.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;
use std::time::Duration;

/// A performance metric entry.
pub struct Metric {
    /// The name of this metric.
    pub name: String,
    /// The measured duration value.
    pub value: Duration,
    /// The number of times this metric was recorded.
    pub call_count: u64,
}

/// A widget that displays performance profiling data.
pub struct Profiler {
    /// The widget ID for this profiler.
    id: WidgetId,
    /// The performance metrics to display.
    metrics: Vec<Metric>,
    /// The theme for this widget.
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
}

impl Profiler {
    /// Creates a new profiler widget with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            metrics: Vec::new(),
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 60, 15)),
            dirty: true,
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the metrics to display.
    pub fn set_metrics(&mut self, metrics: Vec<Metric>) {
        self.metrics = metrics;
        self.dirty = true;
    }

    /// Records a metric entry.
    pub fn record(&mut self, name: &str, value: Duration, call_count: u64) {
        self.metrics.push(Metric {
            name: name.to_string(),
            value,
            call_count,
        });
        self.dirty = true;
    }
}

impl crate::framework::widget::Widget for Profiler {
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

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn z_index(&self) -> u16 {
        160
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 160;

        let width = plane.cells.len() / plane.height as usize;

        let total_time: Duration = self.metrics.iter().map(|m| m.value).sum();

        for (i, metric) in self
            .metrics
            .iter()
            .take(area.height as usize - 1)
            .enumerate()
        {
            let percentage = if total_time.as_nanos() > 0 {
                (metric.value.as_nanos() as f64 / total_time.as_nanos() as f64 * 100.0).round()
                    as u32
            } else {
                0
            };
            let line = format!(
                "{}: {:?} ({}%, {} calls)",
                metric.name, metric.value, percentage, metric.call_count
            );
            for (j, c) in line.chars().take(width).enumerate() {
                let idx = (i as u16 * plane.width + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: self.theme.fg,
                        bg: self.theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        let total_line = format!("Total: {:?}", total_time);
        let row = (area.height as usize).saturating_sub(1);
        for (j, c) in total_line.chars().take(width).enumerate() {
            let idx = (row as u16 * plane.width + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: self.theme.accent,
                    bg: self.theme.bg,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }
}
