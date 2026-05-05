#![allow(missing_docs)]
//! File manager demo — shows List + Breadcrumbs + SplitPane.
//!
//! Keyboard: q=quit, t=cycle theme, arrows=Navigate, Enter=Open.
//! Mouse: click to select, scroll to browse.

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
    show_help: bool,
}

impl FileManagerApp {
    fn new(crumbs: Vec<String>, entries: Vec<FileEntry>, theme: Theme, theme_idx: usize) -> Self {
        let mut list = List::new(entries);
        list.on_theme_change(&theme);
        Self {
            theme,
            theme_idx,
            crumbs,
            list,
            show_help: false,
        }
    }

    /// Cycle through themes and propagate to ALL child widgets
    fn cycle_theme(&mut self) {
        let themes = [Theme::nord(), Theme::cyberpunk(), Theme::dracula()];
        self.theme_idx = (self.theme_idx + 1) % themes.len();
        self.theme = themes[self.theme_idx].clone();
        // Propagate to ALL child widgets
        self.list.on_theme_change(&self.theme);
    }
}

impl Widget for FileManagerApp {
    fn render(&mut self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.transparent = false;

        // Split layout
        let split = SplitPane::new(Orientation::Vertical).ratio(0.7);
        let (main_rect, side_rect) = split.split(area);

        // Main: File list
        self.list.set_visible_count((main_rect.height as usize).saturating_sub(2).max(1));
        let list_plane = self.list.render(main_rect);
        // Copy list cells onto main plane
        for (i, cell) in list_plane.cells.iter().enumerate() {
            let x = (i as u16) % list_plane.width;
            let y = (i as u16) / list_plane.width;
            if y < main_rect.height && x < main_rect.width {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = cell.clone();
                }
            }
        }

        // Breadcrumbs at top of main area
        let bc_plane = Breadcrumbs::new(self.crumbs.clone()).render(main_rect);
        for (i, cell) in bc_plane.cells.iter().enumerate() {
            let x = (i as u16) % bc_plane.width;
            let y = (i as u16) / bc_plane.width;
            if y < main_rect.height && x < main_rect.width {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = cell.clone();
                }
            }
        }

        // Side panel: Info area with opaque background
        let mut info_plane = Plane::new(1, side_rect.width, side_rect.height);
        for c in info_plane.cells.iter_mut() {
            c.bg = t.surface;
            c.fg = t.fg;
            c.transparent = false;
        }

        let mut y = 1u16;
        let mut print = |p: &mut Plane, text: &str, fg: Color| {
            for (i, c) in text.chars().take(side_rect.width as usize - 2).enumerate() {
                let idx = ((y * side_rect.width) + 1 + i as u16) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].char = c;
                    p.cells[idx].fg = fg;
                    p.cells[idx].bg = t.surface;
                    p.cells[idx].transparent = false;
                }
            }
            y += 1;
        };

        print(&mut info_plane, "INFORMATION", t.primary);
        print(&mut info_plane, &format!("Items: {}", self.list.len()), t.fg_muted);

        if let Some(entry) = self.list.get_selected() {
            print(&mut info_plane, &format!("Name: {}", entry.name), t.fg_on_accent);
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

        // Copy info_plane onto side_rect of main plane
        for (i, cell) in info_plane.cells.iter().enumerate() {
            let x = (i as u16) % side_rect.width;
            let y = (i as u16) / side_rect.width;
            let target_x = side_rect.x + x;
            let target_y = side_rect.y + y;
            if target_y < area.height && target_x < area.width {
                let idx = (target_y * area.width + target_x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = cell.clone();
                }
            }
        }

        // Help overlay with rounded corners
        if self.show_help {
            let help_text = vec![
                "HELP",
                "",
                "q - Quit",
                "t - Cycle theme",
                "Arrows - Navigate",
                "Enter - Open",
                "? - Toggle help",
            ];
            let overlay_w = 28.min(area.width.saturating_sub(4));
            let overlay_h = (help_text.len() as u16 + 2).min(area.height.saturating_sub(2));
            let overlay_x = (area.width.saturating_sub(overlay_w)) / 2;
            let overlay_y = (area.height.saturating_sub(overlay_h)) / 2;

            let mut overlay = Plane::new(100, overlay_w, overlay_h);
            for row in 0..overlay_h {
                for col in 0..overlay_w {
                    let idx = (row * overlay_w + col) as usize;
                    if idx < overlay.cells.len() {
                        let is_corner = (row == 0 && col == 0)
                            || (row == 0 && col == overlay_w - 1)
                            || (row == overlay_h - 1 && col == 0)
                            || (row == overlay_h - 1 && col == overlay_w - 1);
                        let is_border = row == 0 || row == overlay_h - 1 || col == 0 || col == overlay_w - 1;
                        overlay.cells[idx].bg = t.surface;
                        overlay.cells[idx].fg = t.fg;
                        overlay.cells[idx].transparent = false;
                        overlay.cells[idx].char = if is_corner {
                            '+'
                        } else if is_border {
                            '-'
                        } else {
                            ' '
                        };
                    }
                }
            }
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

            // Copy overlay onto main plane at centered position
            for (i, cell) in overlay.cells.iter().enumerate() {
                let x = (i as u16) % overlay_w;
                let y = (i as u16) / overlay_w;
                let target_x = overlay_x + x;
                let target_y = overlay_y + y;
                if target_y < area.height && target_x < area.width {
                    let idx = (target_y * area.width + target_x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = cell.clone();
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.show_help = false;
                return true;
            }
        }

        match key.code {
            KeyCode::Char('q') => {
                // Handled via quit flag
                true
            }
            KeyCode::Char('t') => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                true
            }
            _ => false,
        }
    }

    fn needs_render(&self) -> bool {
        true // Always re-render due to theme propagation
    }
}

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();
    let theme_idx = 1; // cyberpunk index

    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let crumbs: Vec<String> = current_dir
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    let entries = read_dir(&PathBuf::from(crumbs.join("/")));
    let mut app = FileManagerApp::new(crumbs, entries, theme, theme_idx);

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);
    let app_ref = std::cell::RefCell::new(None);

    App::new()?
        .title("File Manager")
        .fps(30)
        .theme(app.theme.clone())
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
            let (w, h) = ctx.compositor().size();
            let area = Rect::new(0, 0, w, h);
            let plane = app.render(area);
            ctx.add_plane(plane);
        })
}