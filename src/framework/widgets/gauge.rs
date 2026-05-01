//! Gauge widget — displays a value as a fill bar with percentage.
//!
//! Binds to a CLI command that outputs a numeric value (0-100 or custom max).
//! Renders as a labeled bar with fill percentage.
//!
//! ## TOML definition
//!
//! ```toml
//! [[widget]]
//! id = 1
//! type = "Gauge"
//! label = "CPU"
//! max = 100
//! warn = 70
//! crit = 90
//! bind = "dracon-sync cpu --percent"
//! refresh = 2
//! ```

use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::command::BoundCommand;
use crate::framework::theme::Theme;
use crate::framework::widget::{Widget, WidgetId};
use ratatui::layout::Rect;

pub struct Gauge {
    id: WidgetId,
    label: String,
    value: f64,
    max: f64,
    warn_threshold: f64,
    crit_threshold: f64,
    theme: Theme,
    area: std::cell::Cell<Rect>,
    dirty: bool,
    bound_command: Option<BoundCommand>,
}

impl Gauge {
    pub fn new(label: &str) -> Self {
        Self {
            id: WidgetId::default_id(),
            label: label.to_string(),
            value: 0.0,
            max: 100.0,
            warn_threshold: 70.0,
            crit_threshold: 90.0,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 3)),
            dirty: true,
            bound_command: None,
        }
    }

    pub fn with_id(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_string(),
            value: 0.0,
            max: 100.0,
            warn_threshold: 70.0,
            crit_threshold: 90.0,
            theme: Theme::default(),
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 3)),
            dirty: true,
            bound_command: None,
        }
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self.dirty = true;
        self
    }

    pub fn warn_threshold(mut self, warn: f64) -> Self {
        self.warn_threshold = warn;
        self.dirty = true;
        self
    }

    pub fn crit_threshold(mut self, crit: f64) -> Self {
        self.crit_threshold = crit;
        self.dirty = true;
        self
    }

    pub fn bind_command(mut self, cmd: BoundCommand) -> Self {
        self.bound_command = Some(cmd);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self.dirty = true;
        self
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value.min(self.max);
        self.dirty = true;
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn percentage(&self) -> f64 {
        if self.max == 0.0 {
            0.0
        } else {
            (self.value / self.max) * 100.0
        }
    }

    fn fill_color(&self) -> Color {
        let pct = self.percentage();
        if pct >= self.crit_threshold {
            self.theme.error_fg
        } else if pct >= self.warn_threshold {
            self.theme.warning_fg
        } else {
            self.theme.success_fg
        }
    }

    fn render_bar(&self, width: u16) -> Vec<Cell> {
        let fill_width = ((self.percentage() / 100.0) * (width - 2) as f64).round() as usize;
        let fg = self.fill_color();

        let mut cells = Vec::with_capacity(width as usize);
        cells.push(Cell {
            char: '[',
            fg: self.theme.fg,
            bg: self.theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        });

        for i in 0..(width - 2) as usize {
            if i < fill_width {
                cells.push(Cell {
                    char: '█',
                    fg,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                });
            } else {
                cells.push(Cell {
                    char: '░',
                    fg: self.theme.inactive_fg,
                    bg: self.theme.bg,
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                });
            }
        }

        cells.push(Cell {
            char: ']',
            fg: self.theme.fg,
            bg: self.theme.bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        });
        cells
    }
}

impl Widget for Gauge {
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

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);

        let label_text = format!("{}: {:.1}%", self.label, self.percentage());
        for (i, c) in label_text.chars().take(area.width as usize).enumerate() {
            plane.cells[i] = Cell {
                char: c,
                fg: self.theme.fg,
                bg: self.theme.bg,
                style: Styles::BOLD,
                transparent: false,
                skip: false,
            };
        }

        let bar_row = 1usize;
        let bar_cells = self.render_bar(area.width);
        for (i, cell) in bar_cells.into_iter().enumerate().take(area.width as usize) {
            plane.cells[bar_row * area.width as usize + i] = cell;
        }

        plane
    }

    fn commands(&self) -> Vec<BoundCommand> {
        self.bound_command.iter().cloned().collect()
    }

    fn apply_command_output(&mut self, output: &crate::framework::command::ParsedOutput) {
        if let crate::framework::command::ParsedOutput::Scalar(s) = output {
            if let Ok(v) = s.parse::<f64>() {
                self.set_value(v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_new() {
        let g = Gauge::new("CPU");
        assert_eq!(g.label, "CPU");
        assert_eq!(g.value, 0.0);
        assert_eq!(g.max, 100.0);
    }

    #[test]
    fn test_gauge_with_id() {
        let g = Gauge::with_id(WidgetId::new(7), "RAM");
        assert_eq!(g.id, WidgetId::new(7));
        assert_eq!(g.label, "RAM");
    }

    #[test]
    fn test_gauge_max() {
        let g = Gauge::new("Disk").max(1000.0);
        assert_eq!(g.max, 1000.0);
    }

    #[test]
    fn test_gauge_warn_threshold() {
        let g = Gauge::new("CPU").warn_threshold(60.0);
        assert_eq!(g.warn_threshold, 60.0);
    }

    #[test]
    fn test_gauge_crit_threshold() {
        let g = Gauge::new("CPU").crit_threshold(95.0);
        assert_eq!(g.crit_threshold, 95.0);
    }

    #[test]
    fn test_gauge_bind_command() {
        let cmd = BoundCommand::new("cpu --percent").label("cpu");
        let g = Gauge::new("CPU").bind_command(cmd);
        assert_eq!(g.commands().len(), 1);
    }

    #[test]
    fn test_gauge_set_value() {
        let mut g = Gauge::new("CPU");
        g.set_value(50.0);
        assert_eq!(g.value, 50.0);
    }

    #[test]
    fn test_gauge_set_value_clamped() {
        let mut g = Gauge::new("CPU");
        g.set_value(150.0);
        assert_eq!(g.value, 100.0);
    }

    #[test]
    fn test_gauge_percentage() {
        let mut g = Gauge::new("CPU");
        g.set_value(75.0);
        assert!((g.percentage() - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_gauge_percentage_zero_max() {
        let mut g = Gauge::new("CPU").max(0.0);
        g.set_value(50.0);
        assert_eq!(g.percentage(), 0.0);
    }

    #[test]
    fn test_gauge_fill_color_normal() {
        let mut g = Gauge::new("CPU");
        g.set_value(50.0);
        assert_eq!(g.fill_color(), g.theme.success_fg);
    }

    #[test]
    fn test_gauge_fill_color_warning() {
        let mut g = Gauge::new("CPU");
        g.set_value(75.0);
        assert_eq!(g.fill_color(), g.theme.warning_fg);
    }

    #[test]
    fn test_gauge_fill_color_critical() {
        let mut g = Gauge::new("CPU");
        g.set_value(95.0);
        assert_eq!(g.fill_color(), g.theme.error_fg);
    }

    #[test]
    fn test_gauge_render() {
        let mut g = Gauge::new("CPU");
        g.set_value(50.0);
        let plane = g.render(Rect::new(0, 0, 20, 3));
        assert_eq!(plane.cells[0].char, 'C');
    }

    #[test]
    fn test_gauge_render_bar_chars() {
        let mut g = Gauge::new("CPU");
        g.set_value(50.0);
        let plane = g.render(Rect::new(0, 0, 20, 3));
        let bar_cell = &plane.cells[21];
        assert_eq!(bar_cell.char, '█');
    }

    #[test]
    fn test_gauge_dirty_lifecycle() {
        let mut g = Gauge::new("CPU");
        assert!(g.needs_render());
        g.clear_dirty();
        assert!(!g.needs_render());
        g.set_value(25.0);
        assert!(g.needs_render());
    }

    #[test]
    fn test_gauge_with_theme() {
        let theme = Theme::nord();
        let g = Gauge::new("CPU").with_theme(theme);
        assert_eq!(g.theme.name, "nord");
    }

    #[test]
    fn test_gauge_value() {
        let mut g = Gauge::new("RAM");
        g.set_value(42.5);
        assert!((g.value() - 42.5).abs() < 0.001);
    }

    #[test]
    fn test_gauge_apply_command_output_scalar() {
        use crate::framework::command::ParsedOutput;
        let mut g = Gauge::new("CPU");
        g.apply_command_output(&ParsedOutput::Scalar("75.5".to_string()));
        assert!((g.value() - 75.5).abs() < 0.001);
    }

    #[test]
    fn test_gauge_apply_command_output_ignores_non_scalar() {
        use crate::framework::command::ParsedOutput;
        let mut g = Gauge::new("CPU");
        g.set_value(50.0);
        g.apply_command_output(&ParsedOutput::None);
        assert!((g.value() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_gauge_apply_command_output_parses_invalid_as_zero() {
        use crate::framework::command::ParsedOutput;
        let mut g = Gauge::new("CPU");
        g.apply_command_output(&ParsedOutput::Scalar("not-a-number".to_string()));
        assert_eq!(g.value(), 0.0);
    }
}
