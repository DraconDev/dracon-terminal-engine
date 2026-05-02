#![allow(missing_docs)]
//! Dracon Terminal Engine — Example Showcase
//!
//! Interactive launcher for all framework examples.
//! Navigate with arrow keys, press Enter to see run command, q to quit.
//!
//! Run with: cargo run --example showcase

use std::os::fd::AsFd;
use dracon_terminal_engine::compositor::{Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use ratatui::layout::Rect;

struct ExampleMeta {
    name: &'static str,
    category: &'static str,
    description: &'static str,
    run_cmd: &'static str,
}

impl ExampleMeta {
    fn all() -> Vec<Self> {
        vec![
            ExampleMeta { name: "widget_gallery", category: "cookbook", description: "All interactive widgets in one App", run_cmd: "cargo run --example widget_gallery" },
            ExampleMeta { name: "tree_navigator", category: "cookbook", description: "Hierarchical navigation with Tree", run_cmd: "cargo run --example tree_navigator" },
            ExampleMeta { name: "log_monitor", category: "cookbook", description: "Real-time log viewer with filtering", run_cmd: "cargo run --example log_monitor" },
            ExampleMeta { name: "tabbed_panels", category: "cookbook", description: "Tab switching with per-tab state", run_cmd: "cargo run --example tabbed_panels" },
            ExampleMeta { name: "data_table", category: "cookbook", description: "Sortable table with search/filter", run_cmd: "cargo run --example data_table" },
            ExampleMeta { name: "split_resizer", category: "cookbook", description: "Nested SplitPane with drag-to-resize", run_cmd: "cargo run --example split_resizer" },
            ExampleMeta { name: "command_bindings", category: "cookbook", description: "5 command-bound widgets with auto-refresh", run_cmd: "cargo run --example command_bindings" },
            ExampleMeta { name: "menu_system", category: "cookbook", description: "MenuBar + ContextMenu with shortcuts", run_cmd: "cargo run --example menu_system" },
            ExampleMeta { name: "debug_overlay", category: "cookbook", description: "Debug tools overlay with F12 toggle", run_cmd: "cargo run --example debug_overlay" },
            ExampleMeta { name: "system_monitor", category: "apps", description: "htop-like dashboard with live gauges", run_cmd: "cargo run --example system_monitor" },
            ExampleMeta { name: "file_manager", category: "apps", description: "Full file manager UI with Tree + Table", run_cmd: "cargo run --example file_manager" },
            ExampleMeta { name: "chat_client", category: "apps", description: "Rich chat UI with emoji picker", run_cmd: "cargo run --example chat_client" },
            ExampleMeta { name: "dashboard_builder", category: "apps", description: "All command widgets in grid layout", run_cmd: "cargo run --example dashboard_builder" },
            ExampleMeta { name: "form_demo", category: "existing", description: "Settings form with validation", run_cmd: "cargo run --example form_demo" },
            ExampleMeta { name: "theme_switcher", category: "existing", description: "Live theme cycling through all 15 themes", run_cmd: "cargo run --example theme_switcher" },
            ExampleMeta { name: "modal_demo", category: "existing", description: "ConfirmDialog + help overlay", run_cmd: "cargo run --example modal_demo" },
            ExampleMeta { name: "widget_tutorial", category: "existing", description: "Build a custom ColorPicker widget", run_cmd: "cargo run --example widget_tutorial" },
            ExampleMeta { name: "command_dashboard", category: "existing", description: "Auto-refresh dashboard", run_cmd: "cargo run --example command_dashboard" },
            ExampleMeta { name: "framework_demo", category: "existing", description: "App + List + Breadcrumbs + SplitPane", run_cmd: "cargo run --example framework_demo" },
            ExampleMeta { name: "framework_chat", category: "existing", description: "Simple chat interface", run_cmd: "cargo run --example framework_chat" },
            ExampleMeta { name: "framework_file_manager", category: "existing", description: "File browser", run_cmd: "cargo run --example framework_file_manager" },
            ExampleMeta { name: "framework_widgets", category: "existing", description: "Instantiate all widgets and print debug", run_cmd: "cargo run --example framework_widgets" },
            ExampleMeta { name: "cyberpunk_dashboard", category: "existing", description: "Cyberpunk-themed dashboard", run_cmd: "cargo run --example cyberpunk_dashboard" },
        ]
    }
}

struct Showcase {
    id: WidgetId,
    examples: Vec<ExampleMeta>,
    selected: usize,
    area: Rect,
    show_modal: bool,
    theme_idx: usize,
    should_quit: bool,
}

impl Showcase {
    fn new(area: Rect) -> Self {
        Self {
            id: WidgetId::new(0),
            examples: ExampleMeta::all(),
            selected: 0,
            area,
            show_modal: false,
            theme_idx: 0,
            should_quit: false,
        }
    }

    fn themes() -> Vec<Theme> {
        vec![Theme::nord(), Theme::dark(), Theme::cyberpunk(), Theme::dracula()]
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
        let theme = Self::themes()[self.theme_idx];
        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 10;

        for i in 0..p.cells.len() {
            p.cells[i].transparent = false;
            p.cells[i].bg = theme.bg;
            p.cells[i].fg = theme.fg;
        }

        let title = " Dracon — Example Showcase ";
        for (i, c) in title.chars().enumerate() {
            if i < p.cells.len() {
                p.cells[i].char = c;
                p.cells[i].fg = Color::Rgb(0, 255, 200);
                p.cells[i].style = Styles::BOLD;
            }
        }

        let sep_y = 2u16;
        for x in 0..area.width {
            let idx = (sep_y * area.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = '─';
                p.cells[idx].fg = Color::Rgb(60, 65, 75);
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
                    p.cells[idx].fg = Color::Rgb(0, 200, 150);
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
                        p.cells[ci].bg = Color::Rgb(0, 80, 70);
                        p.cells[ci].fg = Color::Rgb(255, 255, 255);
                    }
                }
            }

            let prefix = if is_selected { "> " } else { "  " };
            for (j, c) in prefix.chars().enumerate() {
                let ci = (y * area.width + 1 + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = if is_selected { Color::Rgb(0, 255, 200) } else { theme.fg };
                }
            }

            let cat_color = match ex.category {
                "cookbook" => Color::Rgb(100, 150, 255),
                "apps" => Color::Rgb(255, 150, 100),
                _ => Color::Rgb(150, 150, 150),
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
                    p.cells[ci].fg = if is_selected { Color::Rgb(255, 255, 255) } else { Color::Rgb(200, 200, 200) };
                }
            }

            let desc_x = 35u16;
            for (j, c) in ex.description.chars().take(28).enumerate() {
                let ci = (y * area.width + desc_x + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = if is_selected { Color::Rgb(200, 230, 255) } else { Color::Rgb(140, 140, 140) };
                }
            }
        }

        let status_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let ci = (status_y * area.width + x) as usize;
            if ci < p.cells.len() {
                p.cells[ci].bg = Color::Rgb(20, 25, 30);
            }
        }

        let hints = ["navigate: /", "see cmd: Enter", "theme: t", "quit: q"];
        let hint_xs = [1u16, 18, 35, 50];
        for (hint, hx) in hints.iter().zip(hint_xs.iter()) {
            for (j, c) in hint.chars().enumerate() {
                let ci = (status_y * area.width + hx + j as u16) as usize;
                if ci < p.cells.len() {
                    p.cells[ci].char = c;
                    p.cells[ci].fg = Color::Rgb(0, 200, 150);
                }
            }
        }

        let count = format!("{}/{}", self.selected + 1, self.examples.len());
        let count_x = (area.width as isize - count.len() as isize - 2).max(0) as u16;
        for (j, c) in count.chars().enumerate() {
            let ci = (status_y * area.width + count_x + j as u16) as usize;
            if ci < p.cells.len() {
                p.cells[ci].char = c;
                p.cells[ci].fg = Color::Rgb(100, 150, 200);
            }
        }

        if self.show_modal {
            for i in 0..p.cells.len() {
                p.cells[i].bg = Color::Ansi(0);
                p.cells[i].transparent = false;
            }

            let mw = 50u16;
            let mh = 8u16;
            let mx = (area.width.saturating_sub(mw)) / 2;
            let my = (area.height.saturating_sub(mh)) / 2;

            for y in 0..mh {
                for x in 0..mw {
                    let ci = ((my + y) * area.width + mx + x) as usize;
                    if ci < p.cells.len() {
                        p.cells[ci].bg = Color::Rgb(15, 20, 25);
                        p.cells[ci].fg = Color::Rgb(200, 200, 200);
                    }
                }
            }

            for x in 0..mw {
                let top = (my * area.width + mx + x) as usize;
                let bot = ((my + mh - 1) * area.width + mx + x) as usize;
                if top < p.cells.len() { p.cells[top].char = '─'; p.cells[top].fg = Color::Rgb(0, 200, 150); }
                if bot < p.cells.len() { p.cells[bot].char = '─'; p.cells[bot].fg = Color::Rgb(0, 200, 150); }
            }
            for y in 0..mh {
                let l = (my + y) * area.width + mx;
                let r = l + mw - 1;
                if (l as usize) < p.cells.len() { p.cells[l as usize].char = '│'; p.cells[l as usize].fg = Color::Rgb(0, 200, 150); }
                if (r as usize) < p.cells.len() { p.cells[r as usize].char = '│'; p.cells[r as usize].fg = Color::Rgb(0, 200, 150); }
            }

            let ex = &self.examples[self.selected];
            let lines = [
                format!("  Run: {}  ", ex.run_cmd),
                "  Press any key to close  ".to_string(),
            ];
            for (i, line) in lines.iter().enumerate() {
                let ly = my + 2 + i as u16;
                for (j, c) in line.chars().enumerate() {
                    let ci = (ly * area.width + mx + 2 + j as u16) as usize;
                    if ci < p.cells.len() {
                        p.cells[ci].char = c;
                        p.cells[ci].fg = Color::Rgb(0, 255, 200);
                    }
                }
            }
        }

        p
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_modal {
            self.show_modal = false;
            return true;
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.examples.len() { self.selected += 1; }
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 { self.selected -= 1; }
                true
            }
            KeyCode::Home => { self.selected = 0; true }
            KeyCode::End => { self.selected = self.examples.len().saturating_sub(1); true }
            KeyCode::Enter => { self.show_modal = true; true }
            KeyCode::Char('t') => { self.theme_idx = (self.theme_idx + 1) % Self::themes().len(); true }
            KeyCode::Char('q') => { self.should_quit = true; true }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, row: u16) -> bool {
        if self.show_modal {
            self.show_modal = false;
            return true;
        }

        let list_start = 3u16;
        let visible_count = (self.area.height as usize).saturating_sub(5) as u16;

        if kind == MouseEventKind::Down(MouseButton::Left) {
            if row >= list_start && row < list_start + visible_count {
                let clicked = (row - list_start) as usize;
                let start = self.selected.saturating_sub((visible_count / 2) as usize);
                let idx = start + clicked;
                if idx < self.examples.len() {
                    self.selected = idx;
                    return true;
                }
            }
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

    let mut app = App::new()?.title("Showcase").fps(30).theme(Theme::nord());
    app.add_widget(Box::new(Showcase::new(Rect::new(0, 0, w, h))), Rect::new(0, 0, w, h));
    app.on_tick(|_ctx, _| {}).run(|_ctx| {})
}