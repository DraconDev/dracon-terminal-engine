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
    pub id: WidgetId,
    pub label: String,
    pub value: f64,
    pub max: f64,
    pub warn_threshold: f64,
    pub crit_threshold: f64,
    pub theme: Theme,
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

    pub fn fill_color(&self) -> Color {
        let pct = self.percentage();
        if pct >= self.crit_threshold {
            self.theme.error
        } else if pct >= self.warn_threshold {
            self.theme.warning
        } else {
            self.theme.success
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
                    fg: self.theme.fg_muted,
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

    fn on_theme_change(&mut self, theme: &crate::framework::theme::Theme) {
        self.theme = *theme;
    }
}
