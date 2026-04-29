//! Framework demo — shows App, List, Breadcrumbs, SplitPane, Hud, SystemMonitor.

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, Hud, List, SplitPane};
use dracon_terminal_engine::framework::hitzone::HitZone;
use dracon_terminal_engine::SystemMonitor;
use ratatui::layout::Rect;

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    App::new()?
        .title("Framework Demo")
        .fps(30)
        .theme(theme)
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();
            let split = SplitPane::new(Orientation::Horizontal).ratio(0.3);
            let (left_rect, right_rect) = split.split(Rect::new(0, 0, w, h));

            let mut list = List::new(vec![
                "System Monitor",
                "File Browser",
                "Network Stats",
                "Process List",
                "Disk Usage",
                "Memory Info",
                "CPU Graph",
                "Settings",
            ]);
            list.set_visible_count((left_rect.height as usize).saturating_sub(2).max(1));
            let list_plane = list.render(left_rect);
            ctx.add_plane(list_plane);

            let (bc_plane, bc_zones) = Breadcrumbs::new(vec!["home".to_string(), "user".to_string(), "projects".to_string(), "app".to_string()])
                .render(right_rect);
            ctx.add_plane(bc_plane);
            for zone in bc_zones {
                ctx.add_plane(Plane::new(zone.id, 1, 1));
            }

            let mut sys = SystemMonitor::new();
            let data = sys.get_data();

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
            ctx.add_plane(gauge_plane);

            ctx.add_plane(info_plane);
        })
}