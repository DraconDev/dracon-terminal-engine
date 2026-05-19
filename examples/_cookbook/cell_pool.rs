//! CellPool usage example.
//!
//! Demonstrates how to use the `CellPool` to reduce allocation pressure
//! when creating many planes per frame. Includes a visual utilization
//! gauge, mouse-clickable Acquire/Release buttons, and tick-driven
//! auto-allocation to show the gauge animating.

use dracon_terminal_engine::compositor::pool::{acquire_plane_cells, release_plane_cells};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const ZONE_ACQUIRE: usize = 1;
const ZONE_RELEASE: usize = 2;

struct PoolDemo {
    cell_pool: CellPool,
    theme: Theme,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
    keybindings: KeybindingSet,
    show_help: bool,
    zones: RefCell<ScopedZoneRegistry<usize>>,
    acquired: usize,
    released: usize,
    tick_count: u64,
    held_cells: Vec<(u16, u16, Vec<Cell>)>,
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
            zones: RefCell::new(ScopedZoneRegistry::new()),
            acquired: 0,
            released: 0,
            tick_count: 0,
            held_cells: Vec::new(),
        }
    }

    fn acquire_batch(&mut self) {
        let sizes = [(40, 10), (80, 24), (60, 15), (30, 8)];
        let (w, h) = sizes[self.held_cells.len() % sizes.len()];
        let cells = acquire_plane_cells(&mut self.cell_pool, w, h);
        let count = cells.len();
        self.held_cells.push((w, h, cells));
        self.acquired += count;
        self.dirty = true;
    }

    fn release_batch(&mut self) {
        if let Some((w, h, cells)) = self.held_cells.pop() {
            let count = cells.len();
            release_plane_cells(&mut self.cell_pool, w, h, cells);
            self.released += count;
            self.dirty = true;
        }
    }

    fn auto_tick(&mut self) {
        if !self.tick_count.is_multiple_of(5) {
            return;
        }
        if self.held_cells.is_empty() {
            self.acquire_batch();
        } else if self.held_cells.len() >= 8 {
            self.release_batch();
        } else {
            let roll = self.tick_count % 3;
            if roll == 0 {
                self.release_batch();
            } else {
                self.acquire_batch();
            }
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

        self.zones.borrow_mut().clear();

        let t = &self.theme;
        let total_pooled = self.cell_pool.total_cells();
        let active = self.acquired.saturating_sub(self.released);
        let capacity = total_pooled + active;
        let utilization = if capacity > 0 {
            active as f32 / capacity as f32
        } else {
            0.0
        };

        let title = "CellPool Usage Demo";
        let start_y = 2u16;
        for (j, c) in title.chars().enumerate() {
            let idx = (start_y * area.width + 2 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].transparent = false;
            }
        }

        let desc_lines = [
            "Cells are recycled across frames to reduce allocation pressure.",
            "The gauge below shows active (acquired) vs pooled (available) cells.",
        ];
        for (i, line) in desc_lines.iter().enumerate() {
            for (j, c) in line.chars().enumerate() {
                let idx = ((start_y + 2 + i as u16) * area.width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg_muted;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let stats_y = start_y + 5;
        let stats = [
            ("Pooled cells:", format!("{}", total_pooled)),
            ("Active cells:", format!("{}", active)),
            ("Total acquired:", format!("{}", self.acquired)),
            ("Total released:", format!("{}", self.released)),
            ("Held batches:", format!("{}", self.held_cells.len())),
        ];
        for (i, (label, value)) in stats.iter().enumerate() {
            let y = stats_y + i as u16;
            for (j, c) in label.chars().enumerate() {
                let idx = (y * area.width + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg_muted;
                    plane.cells[idx].transparent = false;
                }
            }
            for (j, c) in value.chars().enumerate() {
                let idx = (y * area.width + 20 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.fg;
                    plane.cells[idx].style = Styles::BOLD;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let gauge_y = stats_y + stats.len() as u16 + 2;
        let gauge_label = "Pool Utilization";
        for (j, c) in gauge_label.chars().enumerate() {
            let idx = (gauge_y * area.width + 2 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].transparent = false;
            }
        }

        let bar_y = gauge_y + 1;
        let bar_x = 2u16;
        let bar_width = area.width.saturating_sub(4).min(60);
        let filled_len = if capacity > 0 {
            (utilization * bar_width as f32) as u16
        } else {
            0
        };

        if bar_width > 0 {
            let idx = (bar_y * area.width + bar_x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '[';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
            for i in 0..bar_width {
                let idx = (bar_y * area.width + bar_x + 1 + i) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].transparent = false;
                    if usize::from(i) < usize::from(filled_len) {
                        plane.cells[idx].char = '█';
                        plane.cells[idx].fg = t.primary;
                        plane.cells[idx].bg = t.primary;
                    } else {
                        plane.cells[idx].char = '░';
                        plane.cells[idx].fg = t.surface;
                    }
                }
            }
            let idx = (bar_y * area.width + bar_x + 1 + bar_width) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ']';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        let pct_str = format!("{:.0}%", utilization * 100.0);
        for (j, c) in pct_str.chars().enumerate() {
            let idx = (bar_y * area.width + bar_x + 2 + bar_width + 2 + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.fg;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].transparent = false;
            }
        }

        let btn_y = bar_y + 3;
        let btn_h = 1u16;
        let btn_w = 12u16;

        let acquire_x = 2u16;
        let release_x = acquire_x + btn_w + 2;

        let acquire_label = " [ Acquire ] ";
        for (j, c) in acquire_label.chars().enumerate() {
            let idx = (btn_y * area.width + acquire_x + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        self.zones.borrow_mut().register(ZONE_ACQUIRE, acquire_x, btn_y, btn_w + 2, btn_h);

        let release_label = " [ Release ] ";
        for (j, c) in release_label.chars().enumerate() {
            let idx = (btn_y * area.width + release_x + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = if self.held_cells.is_empty() { t.fg_muted } else { t.secondary };
                plane.cells[idx].bg = t.surface;
                plane.cells[idx].transparent = false;
            }
        }
        self.zones.borrow_mut().register(ZONE_RELEASE, release_x, btn_y, btn_w + 2, btn_h);

        if area.height > 0 {
            let sb_y = area.height - 1;
            let hint = self.keybindings.format_hint(&[
                (actions::THEME, "theme"),
                (actions::HELP, "help"),
                (actions::QUIT, "quit"),
            ]);
            let status = format!(" a: acquire | r: release | {} ", hint);
            for (j, c) in status.chars().enumerate() {
                let x = j as u16;
                if x >= area.width {
                    break;
                }
                let idx = (sb_y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.surface;
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
            for x in status.len() as u16..area.width {
                let idx = (sb_y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        if self.show_help {
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 14u16.min(area.height.saturating_sub(4));
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

            let quit_key = self.keybindings.display(actions::QUIT).unwrap_or("ctrl+q");
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let theme_key = self.keybindings.display(actions::THEME).unwrap_or("f2");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");

            let shortcuts = [
                ("a", "Acquire cell batch"),
                ("r", "Release cells to pool"),
                ("Click", "Acquire / Release buttons"),
                (quit_key, "Quit"),
                (help_key, "Toggle this help"),
                (back_key, "Dismiss this help"),
                (theme_key, "Cycle theme"),
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
                    let idx = (row * area.width + hx + 16 + j as u16) as usize;
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
                self.acquire_batch();
                true
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.release_batch();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            let zone_id = self.zones.borrow().dispatch(col, row);
            match zone_id {
                Some(ZONE_ACQUIRE) => {
                    self.acquire_batch();
                    return true;
                }
                Some(ZONE_RELEASE) => {
                    self.release_batch();
                    return true;
                }
                _ => {}
            }
        }
        false
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

    let demo = Arc::new(std::sync::Mutex::new(PoolDemo::new(theme.clone(), should_quit)));

    let mut app = App::new()?;
    let demo_tick = Arc::clone(&demo);
    app.add_widget(Box::new(PoolDemoWrapper {
        demo: Arc::clone(&demo),
    }), Rect::new(0, 0, 80, 24));

    app.title("CellPool Demo")
        .fps(30)
        .theme(theme)
        .on_tick(move |ctx, tick| {
            if quit_clone.load(Ordering::SeqCst) {
                ctx.stop();
            }
            let mut d = demo_tick.lock().expect("cell_pool mutex poisoned");
            d.tick_count = tick;
            d.auto_tick();
        })
        .run(|_ctx| {})
}

struct PoolDemoWrapper {
    demo: Arc<std::sync::Mutex<PoolDemo>>,
}

impl Widget for PoolDemoWrapper {
    fn id(&self) -> WidgetId { WidgetId::new(1) }
    fn area(&self) -> Rect { Rect::default() }
    fn set_area(&mut self, _area: Rect) {}
    fn needs_render(&self) -> bool {
        self.demo.lock().expect("cell_pool mutex poisoned").dirty
    }
    fn render(&self, area: Rect) -> Plane {
        self.demo.lock().expect("cell_pool mutex poisoned").render(area)
    }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.demo.lock().expect("cell_pool mutex poisoned").handle_key(key)
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.demo.lock().expect("cell_pool mutex poisoned").handle_mouse(kind, col, row)
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.demo.lock().expect("cell_pool mutex poisoned").on_theme_change(theme)
    }
}
