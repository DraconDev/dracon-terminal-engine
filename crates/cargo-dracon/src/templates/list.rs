//! List-based Dracon Terminal Engine Application

use dracon_terminal_engine::prelude::*;
use ratatui::layout::Rect;

/// Application state with a list widget.
struct MyApp {
    list: List<String>,
    theme: Theme,
}

impl MyApp {
    fn new(theme: Theme) -> Self {
        let items = vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string(),
            "Item 4".to_string(),
            "Item 5".to_string(),
        ];

        let mut list = List::new(items);
        list.with_theme(theme);

        Self { list, theme }
    }
}

impl Widget for MyApp {
    fn id(&self) -> WidgetId {
        WidgetId::new()
    }

    fn set_id(&mut self, id: WidgetId) {
        let _ = id;
    }

    fn area(&self) -> Rect {
        Rect::new(0, 0, 80, 24)
    }

    fn set_area(&mut self, area: Rect) {
        // Pass to list if needed
        let _ = area;
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

        // Render the list widget
        let list_area = Rect::new(5, 3, 70, 18);
        let list_plane = self.list.render(list_area);

        // Blit list onto main plane
        for y in 0..list_plane.height {
            for x in 0..list_plane.width {
                let src_idx = (y * list_plane.width + x) as usize;
                let dst_x = list_area.x + x;
                let dst_y = list_area.y + y;
                let dst_idx = (dst_y * area.width + dst_x) as usize;

                if src_idx < list_plane.cells.len() && dst_idx < plane.cells.len() {
                    plane.cells[dst_idx] = list_plane.cells[src_idx].clone();
                }
            }
        }

        // Render header
        let header = "My List";
        plane.put_str(5, 1, header);

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.list.handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let area = Rect::new(5, 3, 70, 18);
        let rel_col = col.saturating_sub(area.x);
        let rel_row = row.saturating_sub(area.y);
        self.list.handle_mouse(kind, rel_col, rel_row)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.list.on_theme_change(theme);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()?
        .title("List Example")
        .fps(30)
        .on_tick(|ctx, _| {
            let app = MyApp::new(ctx.theme().clone());
            let area = Rect::new(0, 0, 80, 24);
            ctx.add_plane(app.render(area));
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