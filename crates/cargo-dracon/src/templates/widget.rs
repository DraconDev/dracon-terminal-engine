//! Widget-based Dracon Terminal Engine Application

use dracon_terminal_engine::framework::prelude::*;
use ratatui::layout::Rect;

/// A simple counter widget.
struct Counter {
    count: u32,
    area: Rect,
    theme: Theme,
}

impl Counter {
    fn new(theme: Theme) -> Self {
        Self {
            count: 0,
            area: Rect::new(0, 0, 40, 10),
            theme,
        }
    }
}

impl Widget for Counter {
    fn id(&self) -> WidgetId {
        WidgetId::new()
    }

    fn set_id(&mut self, id: WidgetId) {
        let _ = id; // Store if needed
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    fn needs_render(&self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {
        // Implementation
    }

    fn clear_dirty(&mut self) {
        // Implementation
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        // Render counter value
        let text = format!("Count: {}", self.count);
        let x = (area.width.saturating_sub(text.len() as u16)) / 2;
        let y = area.height / 2;
        plane.put_str(x, y, &text);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.count += 1;
                true
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                if self.count > 0 {
                    self.count -= 1;
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()?
        .title("Counter Example")
        .fps(30)
        .on_tick(|ctx, _| {
            let counter = Counter::new(ctx.theme().clone());
            let area = Rect::new(5, 7, 40, 10);
            ctx.add_plane(counter.render(area));
        })
        .on_input(|key| {
            if key.code == KeyCode::Char('q') && key.modifiers.is_empty() {
                return true;
            }
            false
        })
        .run();

    Ok(())
}