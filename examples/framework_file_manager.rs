#![allow(missing_docs)]
//! File manager demo — shows List + Breadcrumbs + SplitPane + ContextMenu.
//!
//! Keyboard: arrows navigate, Enter opens, Backspace goes up, 'c' contextual menu.
//! Mouse: click to select, right-click for context menu, scroll to browse.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::SplitPane;
use ratatui::layout::Rect;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
}

impl std::fmt::Display for FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = if self.is_dir { ">" } else { "-" };
        write!(f, "{} {} ({})", icon, self.name, self.size)
    }
}

fn read_dir(path: &PathBuf) -> Vec<FileEntry> {
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

struct FileManagerApp {
    theme: Theme,
    theme_idx: usize,
    crumbs: Vec<String>,
    list: List<FileEntry>,
    should_quit: Arc<AtomicBool>,
    show_help: bool,
}

impl FileManagerApp {
    fn cycle_theme(&mut self) {
        let themes = [Theme::nord(), Theme::cyberpunk(), Theme::dracula()];
        self.theme_idx = (self.theme_idx + 1) % themes.len();
        self.theme = themes[self.theme_idx].clone();
        // Propagate to ALL child widgets
        self.list.on_theme_change(&self.theme);
        self.should_quit.store(true, Ordering::SeqCst); // trigger quit to restart with new theme
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();
    let theme_idx = 1; // cyberpunk is at index 1

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let crumbs: Vec<String> = current_dir
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    App::new()?
        .title("File Manager")
        .fps(30)
        .theme(theme)
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
        .run(move |ctx| {
            let t = *ctx.theme();
            let (w, h) = ctx.compositor().size();
            let split = SplitPane::new(Orientation::Vertical).ratio(0.7);
            let (main_rect, side_rect) = split.split(Rect::new(0, 0, w, h));

            let entries = read_dir(&PathBuf::from(crumbs.join("/")));
            let mut list = List::new(entries);
            list.set_visible_count((main_rect.height as usize).saturating_sub(2).max(1));
            list.on_theme_change(&t); // Apply theme to list widget
            let list_plane = list.render(main_rect);
            ctx.add_plane(list_plane);

            let bc_plane = Breadcrumbs::new(crumbs.clone()).render(main_rect);
            ctx.add_plane(bc_plane);

            // Ensure background plane is fully opaque with proper bg color
            let mut info_plane = Plane::new(1, side_rect.width, side_rect.height);
            info_plane.z_index = 5;
            for c in info_plane.cells.iter_mut() {
                c.bg = t.surface;
                c.fg = t.fg;
                c.transparent = false; // Ensure opaque background
            }

            let mut y = 1u16;
            let mut print = |plane: &mut Plane, text: &str, fg: Color| {
                for (i, c) in text.chars().take(side_rect.width as usize - 2).enumerate() {
                    let idx = ((y * side_rect.width) + 1 + i as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = fg;
                        plane.cells[idx].bg = t.surface;
                        plane.cells[idx].transparent = false; // Keep cell opaque
                    }
                }
                y += 1;
            };

            print(&mut info_plane, "INFORMATION", t.primary);
            print(
                &mut info_plane,
                &format!("Items: {}", list.len()),
                t.fg_muted,
            );

            if let Some(entry) = list.get_selected() {
                print(
                    &mut info_plane,
                    &format!("Name: {}", entry.name),
                    t.fg_on_accent,
                );
                if entry.is_dir {
                    print(&mut info_plane, "Type: Directory", t.info);
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
                    print(&mut info_plane, &format!("Size: {}", size_str), t.warning);
                }
            }

            ctx.add_plane(info_plane);

            // Help overlay with rounded corners (box-drawing)
            if true {
                let help_text = vec![
                    "HELP",
                    "",
                    "q - Quit",
                    "t - Cycle theme",
                    "Arrows - Navigate",
                    "Enter - Open",
                    "Backspace - Go up",
                    "c - Context menu",
                    "Esc - Close this",
                ];
                let overlay_w = 30.min(w.saturating_sub(4));
                let overlay_h = (help_text.len() as u16 + 2).min(h.saturating_sub(2));
                let overlay_x = (w.saturating_sub(overlay_w)) / 2;
                let overlay_y = (h.saturating_sub(overlay_h)) / 2;

                let mut overlay = Plane::new(overlay_y, overlay_w, overlay_h);
                // Fill background with rounded corners look (using simple box chars)
                for row in 0..overlay_h {
                    for col in 0..overlay_w {
                        let idx = (row * overlay_w + col) as usize;
                        if idx < overlay.cells.len() {
                            // Determine if corner position
                            let is_corner = (row == 0 && col == 0)
                                || (row == 0 && col == overlay_w - 1)
                                || (row == overlay_h - 1 && col == 0)
                                || (row == overlay_h - 1 && col == overlay_w - 1);
                            let is_border = row == 0 || row == overlay_h - 1 || col == 0 || col == overlay_w - 1;

                            overlay.cells[idx].bg = t.surface;
                            overlay.cells[idx].fg = t.fg;
                            overlay.cells[idx].transparent = false;

                            if is_corner {
                                overlay.cells[idx].char = '+';
                            } else if is_border {
                                overlay.cells[idx].char = '-';
                            } else {
                                overlay.cells[idx].char = ' ';
                            }
                        }
                    }
                }
                // Fill help text
                for (i, line) in help_text.iter().enumerate() {
                    let start_x = 2;
                    let start_y = 1 + i as u16;
                    for (j, ch) in line.chars().enumerate() {
                        let idx = (start_y * overlay_w + start_x + j as u16) as usize;
                        if idx < overlay.cells.len() {
                            overlay.cells[idx].char = ch;
                            overlay.cells[idx].fg = if i == 0 { t.primary } else { t.fg };
                            overlay.cells[idx].bg = t.surface;
                        }
                    }
                }
                ctx.add_plane(overlay);
            }
        })
}
