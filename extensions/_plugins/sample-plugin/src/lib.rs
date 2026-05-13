//! Sample plugin demonstrating the Dracon plugin system.

use dracon_terminal_engine::framework::prelude::*;

/// A sample widget plugin.
#[derive(Clone)]
pub struct SampleWidget {
    theme: Theme,
    area: Rect,
    dirty: bool,
}

impl SampleWidget {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            area: Rect::new(0, 0, 40, 10),
            dirty: true,
        }
    }
}

impl Widget for SampleWidget {
    fn id(&self) -> WidgetId {
        WidgetId::new()
    }

    fn set_id(&mut self, id: WidgetId) {
        // Store if needed
        let _ = id;
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
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
        plane.fill_bg(self.theme.bg);

        // Render a simple border
        for x in 0..area.width {
            let top_idx = (0 * area.width + x) as usize;
            let bot_idx = ((area.height - 1) * area.width + x) as usize;
            if top_idx < plane.cells.len() {
                plane.cells[top_idx].char = '─';
                plane.cells[top_idx].fg = self.theme.outline;
            }
            if bot_idx < plane.cells.len() {
                plane.cells[bot_idx].char = '─';
                plane.cells[bot_idx].fg = self.theme.outline;
            }
        }

        // Render title
        let title = "Sample Plugin";
        let title_x = (area.width.saturating_sub(title.len() as u16)) / 2;
        for (i, c) in title.chars().enumerate() {
            let idx = (1 * area.width + title_x + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        plane
    }

    fn handle_key(&mut self, _key: KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.dirty = true;
    }
}

/// Register this plugin with the given registry.
pub fn register(registry: &mut PluginRegistry) {
    registry.register_factory("sample", |theme| {
        Box::new(SampleWidget::new(theme))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_widget_creation() {
        let widget = SampleWidget::new(Theme::nord());
        assert!(widget.needs_render());
        assert_eq!(widget.area.width, 40);
        assert_eq!(widget.area.height, 10);
    }

    #[test]
    fn test_sample_widget_render() {
        let widget = SampleWidget::new(Theme::nord());
        let plane = widget.render(Rect::new(0, 0, 40, 10));
        assert_eq!(plane.width, 40);
        assert_eq!(plane.height, 10);
    }

    #[test]
    fn test_sample_widget_theme_change() {
        let mut widget = SampleWidget::new(Theme::nord());
        widget.on_theme_change(&Theme::dracula());
        assert!(widget.needs_render());
    }
}