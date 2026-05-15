//! CellPool usage example.
//!
//! Demonstrates how to use the `CellPool` to reduce allocation pressure
//! when creating many planes per frame.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct PoolDemo {
    cell_pool: CellPool,
    theme: Theme,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
    keybindings: KeybindingSet,
    show_help: bool,
}

impl PoolDemo {
    fn new(theme: Theme, should_quit: Arc<AtomicBool>) -> Self {
        Self {
            cell_pool: CellPool::new(),
            theme,
            dirty: true,
            should_quit,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            show_help: false,
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
        ];

        let start_y = (area.height.saturating_sub(lines.len() as u16 + 2)) / 2;
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

        if area.height > 0 {
            let sb_y = area.height - 1;
            let status = " Ctrl+T: theme | F1: help | Ctrl+Q: quit";
            for (j, c) in status.chars().enumerate() {
                let x = j as u16;
                if x >= area.width {
                    break;
                }
                let idx = (sb_y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.surface;
                    plane.cells[idx].bg = self.theme.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
            for x in status.len() as u16..area.width {
                let idx = (sb_y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = self.theme.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        if self.show_help {
            let t = &self.theme;
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 12u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;

            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }

            let corners = [
                ('╭', hx, hy),
                ('╮', hx + hw - 1, hy),
                ('╰', hx, hy + hh - 1),
                ('╯', hx + hw - 1, hy + hh - 1),
            ];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = t.outline;
                    plane.cells[idx].transparent = false;
                }
            }
            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < plane.cells.len() {
                    plane.cells[top].char = '─';
                    plane.cells[top].fg = t.outline;
                    plane.cells[top].transparent = false;
                }
                if bot < plane.cells.len() {
                    plane.cells[bot].char = '─';
                    plane.cells[bot].fg = t.outline;
                    plane.cells[bot].transparent = false;
                }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < plane.cells.len() {
                    plane.cells[left].char = '│';
                    plane.cells[left].fg = t.outline;
                    plane.cells[left].transparent = false;
                }
                if right < plane.cells.len() {
                    plane.cells[right].char = '│';
                    plane.cells[right].fg = t.outline;
                    plane.cells[right].transparent = false;
                }
            }

            let title = "CellPool Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].style = Styles::BOLD;
                    plane.cells[idx].transparent = false;
                }
            }

            let shortcuts = [
                ("A", "Acquire cell batch"),
                ("R", "Release cells to pool"),
                ("Ctrl+T", "Cycle theme"),
                ("F1", "Toggle this help"),
                ("Ctrl+Q", "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 3 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.primary;
                        plane.cells[idx].transparent = false;
                    }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.show_help && self.keybindings.matches(actions::BACK, &key) {
            self.show_help = false;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            let themes = Theme::all();
            let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
            self.theme = themes[(idx + 1) % themes.len()].clone();
            self.dirty = true;
            return true;
        }
        match key.code {
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                let _cells = self.cell_pool.acquire_cells(80 * 24);
                self.dirty = true;
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.cell_pool.shrink_to_fit();
                self.dirty = true;
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