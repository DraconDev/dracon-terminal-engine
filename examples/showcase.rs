#![allow(missing_docs)]
//! Dracon Terminal Engine — Example Showcase
//!
//! Interactive launcher for all framework examples.
//! Navigate with arrow keys, press Enter to see run command, q to quit.
//!
//! Run with: cargo run --example showcase

use std::os::fd::AsFd;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{Read, Write};
use dracon_terminal_engine::compositor::{Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use ratatui::layout::Rect;

struct ExampleMeta {
    name: &'static str,
    category: &'static str,
    description: &'static str,
    binary_name: &'static str,
}

impl ExampleMeta {
    fn all() -> Vec<Self> {
        vec![
            ExampleMeta { name: "widget_gallery", category: "cookbook", description: "All interactive widgets in one App", binary_name: "widget_gallery" },
            ExampleMeta { name: "tree_navigator", category: "cookbook", description: "Hierarchical navigation with Tree", binary_name: "tree_navigator" },
            ExampleMeta { name: "log_monitor", category: "cookbook", description: "Real-time log viewer with filtering", binary_name: "log_monitor" },
            ExampleMeta { name: "tabbed_panels", category: "cookbook", description: "Tab switching with per-tab state", binary_name: "tabbed_panels" },
            ExampleMeta { name: "data_table", category: "cookbook", description: "Sortable table with search/filter", binary_name: "data_table" },
            ExampleMeta { name: "split_resizer", category: "cookbook", description: "Nested SplitPane with drag-to-resize", binary_name: "split_resizer" },
            ExampleMeta { name: "command_bindings", category: "cookbook", description: "5 command-bound widgets with auto-refresh", binary_name: "command_bindings" },
            ExampleMeta { name: "menu_system", category: "cookbook", description: "MenuBar + ContextMenu with shortcuts", binary_name: "menu_system" },
            ExampleMeta { name: "debug_overlay", category: "cookbook", description: "Debug tools overlay with F12 toggle", binary_name: "debug_overlay" },
            ExampleMeta { name: "system_monitor", category: "apps", description: "htop-like dashboard with live gauges", binary_name: "system_monitor" },
            ExampleMeta { name: "file_manager", category: "apps", description: "Full file manager UI with Tree + Table", binary_name: "file_manager" },
            ExampleMeta { name: "chat_client", category: "apps", description: "Rich chat UI with emoji picker", binary_name: "chat_client" },
            ExampleMeta { name: "dashboard_builder", category: "apps", description: "All command widgets in grid layout", binary_name: "dashboard_builder" },
            ExampleMeta { name: "form_demo", category: "existing", description: "Settings form with validation", binary_name: "form_demo" },
            ExampleMeta { name: "theme_switcher", category: "existing", description: "Live theme cycling through all 15 themes", binary_name: "theme_switcher" },
            ExampleMeta { name: "modal_demo", category: "existing", description: "ConfirmDialog + help overlay", binary_name: "modal_demo" },
            ExampleMeta { name: "widget_tutorial", category: "existing", description: "Build a custom ColorPicker widget", binary_name: "widget_tutorial" },
            ExampleMeta { name: "framework_file_manager", category: "existing", description: "File browser", binary_name: "framework_file_manager" },
            ExampleMeta { name: "desktop", category: "existing", description: "Desktop environment mockup", binary_name: "desktop" },
            ExampleMeta { name: "game_loop", category: "existing", description: "Interactive game loop demo", binary_name: "game_loop" },
            ExampleMeta { name: "text_editor_demo", category: "existing", description: "Text editor widget demo", binary_name: "text_editor_demo" },
            ExampleMeta { name: "input_debug", category: "existing", description: "Input event debugger", binary_name: "input_debug" },
        ]
    }
}

struct Showcase {
    id: WidgetId,
    examples: Vec<ExampleMeta>,
    selected: usize,
    area: Rect,
    theme_idx: usize,
    should_quit: Arc<AtomicBool>,
    last_click_time: std::time::Instant,
    last_click_row: u16,
    pending_binary: Arc<Mutex<Option<String>>>,
    status_message: Option<String>,
}

impl Showcase {
    fn new(area: Rect, pending: Arc<Mutex<Option<String>>>, should_quit: Arc<AtomicBool>) -> Self {
        Self {
            id: WidgetId::new(0),
            examples: ExampleMeta::all(),
            selected: 0,
            area,
            theme_idx: 0,
            should_quit,
            last_click_time: std::time::Instant::now(),
            last_click_row: u16::MAX,
            pending_binary: pending,
            status_message: None,
        }
    }

    fn themes() -> Vec<Theme> {
        vec![Theme::nord(), Theme::dark(), Theme::cyberpunk(), Theme::dracula()]
    }

    fn launch_selected(&mut self) {
        let ex = &self.examples[self.selected];
        *self.pending_binary.lock().unwrap() = Some(ex.binary_name.to_string());
        self.status_message = Some(format!("Launching {}...", ex.name));
    }
}

impl Widget for Showcase {
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
        let t = Self::themes()[self.theme_idx];
        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 10;

        for i in 0..p.cells.len() {
            p.cells[i].transparent = false;
            p.cells[i].bg = t.bg;
            p.cells[i].fg = t.fg;
        }

        let title = " Dracon — Example Showcase ";
        for (i, c) in title.chars().enumerate() {
            if i < p.cells.len() {
                p.cells[i].char = c;
                p.cells[i].fg = t.primary;
                p.cells[i].style = Styles::BOLD;
            }
        }

        let sep_y = 2u16;
        for x in 0..area.width {
            let idx = (sep_y * area.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = '─';
                p.cells[idx].fg = t.outline;
            }
        }

        let headers = ["Category", "Example", "Description"];
        let col_widths = [12u16, 20, 30];
        let mut x_pos = 1u16;
        let header_y = 1u16;
        for (h, w) in headers.iter().zip(col_widths.iter()) {
            for (j, c) in h.chars().enumerate() {
                let idx = (header_y * area.width + x_pos + j as u16) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].char = c;
                    p.cells[idx].fg = t.primary;
                    p.cells[idx].style = Styles::BOLD;
                }
            }
            x_pos += w + 1;
        }

        let list_start = 3u16;
        let visible_count = (area.height as usize).saturating_sub(5);

        for row in 0..visible_count {
            let idx = self.selected.saturating_sub(visible_count / 2) + row;
            if idx >= self.examples.len() { break; }

            let ex = &self.examples[idx];
            let y = list_start + row as u16;
            let is_selected = idx == self.selected;

            if is_selected {
                for x in 0..area.width {
                    let ci = (y * area.width + x) as usize;
                    if ci < p.cells.len() {
                        p.cells[ci].bg = t.primary_active;
                        p.cells[ci].fg = t.fg_on_accent;
                    }
                }
            }

            let prefix = if is_selected { "> " } else { "  " };
            for (j, c) in prefix.chars().enumerate() {
                let ci = (y * area.width + 1 + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = if is_selected { t.primary } else { t.fg_muted };
                }
            }

            let cat_color = match ex.category {
                "cookbook" => t.info,
                "apps" => t.warning,
                _ => t.fg_muted,
            };
            for (j, c) in ex.category.chars().take(10).enumerate() {
                let ci = (y * area.width + 3 + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = cat_color;
                    p.cells[ci].style = Styles::BOLD;
                }
            }

            let name_x = 15u16;
            for (j, c) in ex.name.chars().take(18).enumerate() {
                let ci = (y * area.width + name_x + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = if is_selected { t.fg_on_accent } else { t.fg };
                }
            }

            let desc_x = 35u16;
            for (j, c) in ex.description.chars().take(28).enumerate() {
                let ci = (y * area.width + desc_x + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = if is_selected { t.selection_fg } else { t.fg_muted };
                }
            }
        }

        let status_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let ci = (status_y * area.width + x) as usize;
            if ci < p.cells.len() {
                p.cells[ci].bg = t.surface_elevated;
            }
        }

        let hints = ["navigate: arrows", "open: Enter/dbl-click", "theme: t", "quit: q"];
        let hint_xs = [1u16, 18, 35, 50];
        for (hint, hx) in hints.iter().zip(hint_xs.iter()) {
            for (j, c) in hint.chars().enumerate() {
                let ci = (status_y * area.width + hx + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = t.primary;
                }
            }
        }

        let count = format!("{}/{}", self.selected + 1, self.examples.len());
        let count_x = (area.width as isize - count.len() as isize - 2).max(0) as u16;
        for (j, c) in count.chars().enumerate() {
            let ci = (status_y * area.width + count_x + j as u16) as usize;
            if ci < p.cells.len() {
                p.cells[ci].char = c;
                p.cells[ci].fg = t.secondary;
            }
        }

        // Display temporary status message (launched example, error, etc.)
        if let Some(ref msg) = self.status_message {
            let msg_text = msg.chars().take(area.width as usize - 4).collect::<String>();
            let msg_x = 2u16;
            let msg_y = status_y.saturating_sub(1);
            for (j, c) in msg_text.chars().enumerate() {
                let ci = (msg_y * area.width + msg_x + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = t.warning;
                }
            }
        }

        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        match key.code {
            KeyCode::Down => {
                if self.selected + 1 < self.examples.len() { self.selected += 1; }
                true
            }
            KeyCode::Up => {
                if self.selected > 0 { self.selected -= 1; }
                true
            }
            KeyCode::Home => { self.selected = 0; true }
            KeyCode::End => { self.selected = self.examples.len().saturating_sub(1); true }
            KeyCode::Enter => { self.launch_selected(); true }
            KeyCode::Char('t') => { self.theme_idx = (self.theme_idx + 1) % Self::themes().len(); true }
            KeyCode::Char('q') => { self.should_quit.store(true, Ordering::SeqCst); true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, row: u16) -> bool {
        let list_start = 3u16;
        let visible_count = (self.area.height as usize).saturating_sub(5) as u16;

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if row >= list_start && row < list_start + visible_count {
                    let clicked = (row - list_start) as usize;
                    let start = self.selected.saturating_sub((visible_count / 2) as usize);
                    let idx = start + clicked;
                    if idx < self.examples.len() {
                        let now = std::time::Instant::now();
                        let elapsed = now.duration_since(self.last_click_time);
                        if elapsed.as_millis() < 500 && self.last_click_row == row {
                            self.launch_selected();
                        } else {
                            self.selected = idx;
                        }
                        self.last_click_time = now;
                        self.last_click_row = row;
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }
}

fn main() -> std::io::Result<()> {
    let (w, h) = if let Ok((cw, ch)) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd()) {
        (cw, ch)
    } else {
        (80u16, 24u16)
    };

    let pending = Arc::new(Mutex::new(None));
    let should_quit = Arc::new(AtomicBool::new(false));
    let showcase = Showcase::new(Rect::new(0, 0, w, h), pending.clone(), should_quit.clone());

    let mut app = App::new()?.title("Showcase").fps(30).theme(Theme::nord());
    app.add_widget(Box::new(showcase), Rect::new(0, 0, w, h));

    app.on_tick(move |ctx, _| {
        // Check if user requested quit via 'q'
        if should_quit.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }
        
        if let Some(binary_name) = pending.lock().unwrap().take() {
            let exe_dir = match std::env::current_exe() {
                Ok(p) => p.parent().unwrap().to_path_buf(),
                Err(_) => return,
            };
            let binary_path = exe_dir.join(&binary_name);

            // Suspend showcase terminal so the user can see build/run output
            let _ = ctx.suspend_terminal();

            // Auto-build if binary is missing
            if !binary_path.exists() {
                // Find crate root by walking up from exe dir looking for Cargo.toml
                let find_crate_root = || -> Option<std::path::PathBuf> {
                    let mut dir = exe_dir.clone();
                    loop {
                        if dir.join("Cargo.toml").exists() {
                            return Some(dir);
                        }
                        if !dir.pop() {
                            return None;
                        }
                    }
                };

                if let Some(crate_root) = find_crate_root() {
                    let build_result = std::process::Command::new("cargo")
                        .args(["build", "--example", &binary_name])
                        .current_dir(&crate_root)
                        .status();
                    match build_result {
                        Ok(s) if s.success() => {
                            // Built successfully — binary should now exist
                        }
                        _ => {
                            let _ = std::fs::write("/tmp/showcase_error.log",
                                format!("Failed to build example: {}\n", binary_name));
                            let _ = std::io::stdout().write_all(
                                b"\nPress Enter to return to showcase...");
                            let _ = std::io::stdout().flush();
                            let mut buf = [0u8; 1];
                            let _ = std::io::stdin().read(&mut buf);
                            let _ = ctx.resume_terminal();
                            ctx.mark_all_dirty();
                            return;
                        }
                    }
                } else {
                    let _ = std::fs::write("/tmp/showcase_error.log",
                        format!("Binary not found and could not find crate root: {}\n", binary_path.display()));
                    let _ = std::io::stdout().write_all(
                        b"\nPress Enter to return to showcase...");
                    let _ = std::io::stdout().flush();
                    let mut buf = [0u8; 1];
                    let _ = std::io::stdin().read(&mut buf);
                    let _ = ctx.resume_terminal();
                    ctx.mark_all_dirty();
                    return;
                }
            }

            // Run example inline and wait for it to finish
            let child_result = std::process::Command::new(&binary_path)
                .current_dir(&exe_dir)
                .status();
                
            if let Err(e) = child_result {
                let _ = std::fs::write("/tmp/showcase_error.log", 
                    format!("Failed to run example: {}\n", e));
                let _ = std::io::stdout().write_all(
                    format!("\nError: could not launch {} (see /tmp/showcase_error.log)\nPress Enter to return...", binary_name).as_bytes());
                let _ = std::io::stdout().flush();
                let mut buf = [0u8; 1];
                let _ = std::io::stdin().read(&mut buf);
            }
            
            // Drain any residual stdin (keys pressed in child may linger)
            let mut drain_buf = [0u8; 256];
            let _ = std::io::stdin().read(&mut drain_buf);
            
            // Resume showcase terminal (re-enter raw mode, alt screen)
            let _ = ctx.resume_terminal();
            
            // Force full re-render
            ctx.mark_all_dirty();
        }
    }).run(|_ctx| {})
}