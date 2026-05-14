//! CellPool usage example.
//!
//! Demonstrates how to use the `CellPool` to reduce allocation pressure
//! when creating many planes per frame.

use dracon_terminal_engine::framework::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct PoolDemo {
    cell_pool: CellPool,
    theme: Theme,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
}

impl PoolDemo {
    fn new(theme: Theme, should_quit: Arc<AtomicBool>) -> Self {
        Self {
            cell_pool: CellPool::new(),
            theme,
            dirty: true,
            should_quit,
        }
    }
}

impl Widget for PoolDemo {
    fn id(&self) -> WidgetId {
        WidgetId::new(1)
    }

    fn area(&self) -> Rect {
        Rect::default()
    }

    fn set_area(&mut self, _area: Rect) {}

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);

        // Show pool stats
        let total = self.cell_pool.total_cells();
        let lines = [
            "CellPool Usage Demo",
            "",
            "This demo shows cell pool statistics.",
            "Cells are recycled across frames to reduce",
            "allocation pressure.",
            "",
            &format!("Pooled cells: {}", total),
            "",
            "Press R to release cells back to pool.",
            "Press A to acquire a new batch of cells.",
            "",
            "Ctrl+Q to quit",
        ];

        let start_y = (area.height.saturating_sub(lines.len() as u16)) / 2;
        for (i, line) in lines.iter().enumerate() {
            for (j, c) in line.chars().enumerate() {
                let idx = ((start_y + i as u16) * area.width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = if i == 0 {
                        self.theme.primary
                    } else {
                        self.theme.fg
                    };
                    plane.cells[idx].style = if i == 0 {
                        Styles::BOLD
                    } else {
                        Styles::empty()
                    };
                    plane.cells[idx].transparent = false;
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        use std::sync::atomic::Ordering;
        match key.code {
            KeyCode::Char('a') => {
                // Acquire a batch of cells (simulating widget render)
                let _cells = self.cell_pool.acquire_cells(80 * 24);
                self.dirty = true;
                true
            }
            KeyCode::Char('r') => {
                // Release cells back (simulating end of frame)
                // In real usage, cells would be placed back in the pool
                // via release_plane_cells() after render
                self.cell_pool.shrink_to_fit();
                self.dirty = true;
                true
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.dirty = true;
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::from_env_or(Theme::nord());
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_clone = Arc::clone(&should_quit);
    let demo = PoolDemo::new(theme.clone(), should_quit);

    let mut app = App::new()?;
    app.add_widget(Box::new(demo), Rect::new(0, 0, 80, 24));

    app.title("CellPool Demo")
        .fps(30)
        .theme(theme)
        .on_tick(move |ctx, _| {
            if quit_clone.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
        .run(|_ctx| {})
}