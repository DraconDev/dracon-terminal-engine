#![allow(missing_docs)]
//! Framework demo — shows App, List, Breadcrumbs, SplitPane, Hud, SystemMonitor.

use std::os::fd::AsFd;
use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, Hud, List, Orientation, SplitPane};
use dracon_terminal_engine::SystemMonitor;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

struct FrameworkDemo {
    id: WidgetId,
    breadcrumbs: Breadcrumbs,
    sys: SystemMonitor,
    area: Rect,
}

impl FrameworkDemo {
    fn new(id: WidgetId) -> Self {
        let breadcrumbs = Breadcrumbs::new(vec![
            "home".to_string(),
            "user".to_string(),
            "projects".to_string(),
            "app".to_string(),
        ]);
        Self {
            id,
            breadcrumbs,
            sys: SystemMonitor::new(),
            area: Rect::new(0, 0, 80, 24),
        }
    }
}

impl Widget for FrameworkDemo {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn z_index(&self) -> u16 { 10 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let theme = Theme::cyberpunk();
        let split = SplitPane::new(Orientation::Horizontal).ratio(0.3);
        let (left_rect, right_rect) = split.split(area);

        let mut list = self.list.clone();
        list.set_visible_count((left_rect.height as usize).saturating_sub(2).max(1));
        let list_plane = list.render(left_rect);

        let bc_plane = self.breadcrumbs.render(right_rect);

        let data = self.sys.get_data();

        let mut info_plane = Plane::new(0, right_rect.width, right_rect.height.saturating_sub(2));
        info_plane.z_index = 5;

        let mut y = 2u16;
        let mut print_line = |plane: &mut Plane, text: &str, fg: Color| {
            for (i, c) in text.chars().take(right_rect.width as usize - 2).enumerate() {
                let idx = (y * right_rect.width + 1 + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].transparent = false;
                }
            }
            y += 1;
        };

        print_line(&mut info_plane, &format!("CPU: {:.1}%", data.cpu_usage), Color::Rgb(0, 200, 120));
        print_line(&mut info_plane, &format!("Memory: {:.1} / {:.1} GB", data.mem_usage, data.total_mem), Color::Rgb(100, 180, 255));
        print_line(&mut info_plane, &format!("Swap: {:.1} / {:.1} GB", data.swap_usage, data.total_swap), Color::Rgb(180, 180, 200));
        print_line(&mut info_plane, &format!("Uptime: {}s", data.uptime), Color::Rgb(150, 150, 150));
        print_line(&mut info_plane, "", Color::Reset);

        if let Some(disk) = data.disks.first() {
            let pct = if disk.total_space > 0.0 {
                (disk.used_space / disk.total_space * 100.0) as u16
            } else {
                0
            };
            print_line(&mut info_plane, &format!("Disk: {} ({}%)", disk.name, pct), Color::Rgb(255, 180, 100));
        }

        let hud = Hud::new(100).with_size(30, 5);
        let gauge_plane = hud.render_gauge(0, 0, "CPU", data.cpu_usage, 100.0, 20);

        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 10;
        for i in 0..p.cells.len() {
            p.cells[i].bg = theme.bg;
            p.cells[i].transparent = false;
        }

        for y in 0..area.height {
            for x in 0..area.width {
                let src_idx = (y * area.width + x) as usize;
                if src_idx < list_plane.cells.len() {
                    let dest_idx = (y * area.width + x) as usize;
                    if dest_idx < p.cells.len() {
                        p.cells[dest_idx] = list_plane.cells[src_idx].clone();
                    }
                }
            }
        }

        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Down => { self.list.next(); true }
            KeyCode::Up => { self.list.prev(); true }
            _ => false,
        }
    }
}

fn main() -> std::io::Result<()> {
    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let mut app = App::new()?.title("Framework Demo").fps(30).theme(Theme::cyberpunk());
    app.add_widget(Box::new(FrameworkDemo::new(WidgetId::new(0))), Rect::new(0, 0, w, h));
    app.on_tick(|_ctx, _| {}).run(|_ctx| {})
}