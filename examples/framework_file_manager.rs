//! File manager demo — shows List + Breadcrumbs + SplitPane + ContextMenu.
//!
//! Keyboard: arrows navigate, Enter opens, Backspace goes up, 'c' contextual menu.
//! Mouse: click to select, right-click for context menu, scroll to browse.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::SplitPane;
use ratatui::layout::Rect;
use std::path::PathBuf;

fn read_dir(path: &PathBuf) -> Vec<String> {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let meta = e.metadata().ok();
                    FileEntry {
                        name: e.file_name().to_string_lossy().into_owned(),
                        is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                        size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn main() -> std::io::Result<()> {
    let theme = Theme::dark();

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let crumbs: Vec<String> = current_dir
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    App::new()?
        .title("File Manager")
        .fps(30)
        .theme(theme)
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();
            let split = SplitPane::new(Orientation::Vertical).ratio(0.7);
            let (main_rect, side_rect) = split.split(Rect::new(0, 0, w, h));

            let entries = read_dir(&PathBuf::from(crumbs.join("/")));
            let mut list = List::new(entries);
            list.set_visible_count((main_rect.height as usize).saturating_sub(2).max(1));
            let list_plane = list.render(main_rect);
            ctx.add_plane(list_plane);

            let bc_plane = Breadcrumbs::new(crumbs.clone()).render(main_rect);
            ctx.add_plane(bc_plane);

            let _sel_idx = list.selected_index();
            let mut info_plane = Plane::new(1, side_rect.width, side_rect.height);
            info_plane.z_index = 5;

            let mut y = 1u16;
            let mut print = |plane: &mut Plane, text: &str, fg: Color| {
                for (i, c) in text.chars().take(side_rect.width as usize - 2).enumerate() {
                    let idx = ((y * side_rect.width) + 1 + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].transparent = false;
                    }
                }
                y += 1;
            };

            print(&mut info_plane, "INFORMATION", Color::Rgb(0, 255, 136));
            print(
                &mut info_plane,
                &format!("Items: {}", list.len()),
                Color::Rgb(180, 180, 180),
            );

            if let Some(entry) = list.get_selected() {
                print(
                    &mut info_plane,
                    &format!("Name: {}", entry.name),
                    Color::Rgb(255, 255, 255),
                );
                if entry.is_dir {
                    print(
                        &mut info_plane,
                        "Type: Directory",
                        Color::Rgb(100, 200, 255),
                    );
                } else {
                    let size_str = if entry.size < 1024 {
                        format!("{}B", entry.size)
                    } else if entry.size < 1024 * 1024 {
                        format!("{}KB", entry.size / 1024)
                    } else if entry.size < 1024 * 1024 * 1024 {
                        format!("{}MB", entry.size / 1024 / 1024)
                    } else {
                        format!("{}GB", entry.size / 1024 / 1024 / 1024)
                    };
                    print(
                        &mut info_plane,
                        &format!("Size: {}", size_str),
                        Color::Rgb(200, 150, 100),
                    );
                }
            }

            ctx.add_plane(info_plane);
        })
}
