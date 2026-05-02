#![allow(missing_docs)]
//! Dracon Terminal Engine — Example Showcase
//!
//! Interactive launcher for all framework examples.
//! Navigate with arrow keys, press Enter to see run command, q to quit.
//!
//! Run with: cargo run --example showcase

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use dracon_terminal_engine::compositor::{Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use ratatui::layout::Rect;

struct ExampleMeta {
    name: &'static str,
    category: &'static str,
    description: &'static str,
    widgets: &'static str,
    run_cmd: &'static str,
}

impl ExampleMeta {
    fn all() -> Vec<Self> {
        vec![
            // COOKBOOK
            ExampleMeta {
                name: "widget_gallery",
                category: "cookbook",
                description: "All interactive widgets in one App",
                widgets: "Checkbox, Radio, Slider, Spinner, Toggle, Select, SearchInput, ProgressBar, Button",
                run_cmd: "cargo run --example widget_gallery",
            },
            ExampleMeta {
                name: "tree_navigator",
                category: "cookbook",
                description: "Hierarchical navigation with Tree + Breadcrumbs",
                widgets: "Tree, Breadcrumbs, SplitPane, StatusBar",
                run_cmd: "cargo run --example tree_navigator",
            },
            ExampleMeta {
                name: "log_monitor",
                category: "cookbook",
                description: "Real-time log viewer with severity filtering",
                widgets: "LogViewer, StatusBadge",
                run_cmd: "cargo run --example log_monitor",
            },
            ExampleMeta {
                name: "tabbed_panels",
                category: "cookbook",
                description: "Tab switching with per-tab widget state",
                widgets: "Tabbar, Gauge, List, Select, Toggle, Slider, KeyValueGrid",
                run_cmd: "cargo run --example tabbed_panels",
            },
            ExampleMeta {
                name: "data_table",
                category: "cookbook",
                description: "Sortable table with search/filter",
                widgets: "SearchInput (custom table)",
                run_cmd: "cargo run --example data_table",
            },
            ExampleMeta {
                name: "split_resizer",
                category: "cookbook",
                description: "Nested SplitPane with drag-to-resize",
                widgets: "SplitPane (nested)",
                run_cmd: "cargo run --example split_resizer",
            },
            ExampleMeta {
                name: "command_bindings",
                category: "cookbook",
                description: "All 5 command-bound widgets with auto-refresh",
                widgets: "Gauge, KeyValueGrid, StatusBadge, LogViewer, StreamingText",
                run_cmd: "cargo run --example command_bindings",
            },
            ExampleMeta {
                name: "menu_system",
                category: "cookbook",
                description: "MenuBar + ContextMenu with keyboard shortcuts",
                widgets: "MenuBar, ContextMenu, List, Toast",
                run_cmd: "cargo run --example menu_system",
            },
            ExampleMeta {
                name: "debug_overlay",
                category: "cookbook",
                description: "Debug tools overlay with F12 toggle",
                widgets: "DebugOverlay, Profiler, WidgetInspector, EventLogger",
                run_cmd: "cargo run --example debug_overlay",
            },
            // APPS
            ExampleMeta {
                name: "system_monitor",
                category: "apps",
                description: "htop-like dashboard with live gauges",
                widgets: "Gauge×4, KeyValueGrid, StatusBadge, StreamingText, SplitPane",
                run_cmd: "cargo run --example system_monitor",
            },
            ExampleMeta {
                name: "file_manager",
                category: "apps",
                description: "Full file manager UI with Tree + Table",
                widgets: "Tree, Table, Breadcrumbs, StatusBar, SplitPane, ContextMenu",
                run_cmd: "cargo run --example file_manager",
            },
            ExampleMeta {
                name: "chat_client",
                category: "apps",
                description: "Rich chat UI with emoji picker and settings",
                widgets: "List, TextInput, Toast, Modal, StatusBar",
                run_cmd: "cargo run --example chat_client",
            },
            ExampleMeta {
                name: "dashboard_builder",
                category: "apps",
                description: "All command widgets in grid layout",
                widgets: "Gauge, KeyValueGrid, StatusBadge, LogViewer, StreamingText",
                run_cmd: "cargo run --example dashboard_builder",
            },
            // EXISTING EXAMPLES
            ExampleMeta {
                name: "form_demo",
                category: "existing",
                description: "Settings form with validation and focus cycling",
                widgets: "SearchInput, PasswordInput, Select, Toggle, Button, Toast",
                run_cmd: "cargo run --example form_demo",
            },
            ExampleMeta {
                name: "theme_switcher",
                category: "existing",
                description: "Live theme cycling through all 15 themes",
                widgets: "StatusBadge, Gauge, List, Breadcrumbs",
                run_cmd: "cargo run --example theme_switcher",
            },
            ExampleMeta {
                name: "modal_demo",
                category: "existing",
                description: "ConfirmDialog + help overlay with shortcuts",
                widgets: "Modal, ConfirmDialog, Button, Toast",
                run_cmd: "cargo run --example modal_demo",
            },
            ExampleMeta {
                name: "widget_tutorial",
                category: "existing",
                description: "Build a custom ColorPicker widget from scratch",
                widgets: "Custom (ColorPicker) + HitZone + Plane rendering",
                run_cmd: "cargo run --example widget_tutorial",
            },
            ExampleMeta {
                name: "command_dashboard",
                category: "existing",
                description: "Auto-refresh dashboard with Gauge + KeyValueGrid + StatusBadge",
                widgets: "Gauge, KeyValueGrid, StatusBadge",
                run_cmd: "cargo run --example command_dashboard",
            },
            ExampleMeta {
                name: "framework_demo",
                category: "existing",
                description: "App + List + Breadcrumbs + SplitPane + Hud",
                widgets: "List, Breadcrumbs, SplitPane, Hud",
                run_cmd: "cargo run --example framework_demo",
            },
            ExampleMeta {
                name: "framework_chat",
                category: "existing",
                description: "Simple chat interface with message list and input",
                widgets: "List, Input, theme, App event loop",
                run_cmd: "cargo run --example framework_chat",
            },
            ExampleMeta {
                name: "framework_file_manager",
                category: "existing",
                description: "File browser with List + Breadcrumbs + ContextMenu",
                widgets: "List, Breadcrumbs, ContextMenu",
                run_cmd: "cargo run --example framework_file_manager",
            },
            ExampleMeta {
                name: "framework_widgets",
                category: "existing",
                description: "Instantiate all widgets and print debug info",
                widgets: "Checkbox, Radio, Slider, Spinner, Toggle, Select, SearchInput, ProgressBar, Form",
                run_cmd: "cargo run --example framework_widgets",
            },
            ExampleMeta {
                name: "cyberpunk_dashboard",
                category: "existing",
                description: "Cyberpunk-themed dashboard with RatatuiBackend",
                widgets: "SplitPane, List, Hud",
                run_cmd: "cargo run --example cyberpunk_dashboard",
            },
        ]
    }
}

struct Showcase {
    id: WidgetId,
    examples: Vec<ExampleMeta>,
    selected: usize,
    area: Rect,
    dirty: bool,
    show_modal: bool,
    theme_idx: usize,
    should_quit: bool,
}

impl Showcase {
    fn new() -> Self {
        Self {
            id: WidgetId::new(0),
            examples: ExampleMeta::all(),
            selected: 0,
            area: Rect::new(0, 0, 100, 30),
            dirty: true,
            show_modal: false,
            theme_idx: 0,
            should_quit: false,
        }
    }

    fn themes() -> Vec<Theme> {
        vec![
            Theme::nord(),
            Theme::dark(),
            Theme::cyberpunk(),
            Theme::dracula(),
        ]
    }
}

impl Default for Showcase {
    fn default() -> Self { Self::new() }
}

struct ShowcaseWidget(Rc<RefCell<Showcase>>);

impl ShowcaseWidget {
    fn new(inner: Rc<RefCell<Showcase>>) -> Self {
        Self(inner)
    }
}

impl Widget for ShowcaseWidget {
    fn id(&self) -> WidgetId { self.0.borrow().id() }
    fn set_id(&mut self, id: WidgetId) { self.0.borrow_mut().set_id(id); }
    fn area(&self) -> Rect { self.0.borrow().area() }
    fn set_area(&mut self, area: Rect) { self.0.borrow_mut().set_area(area); }
    fn z_index(&self) -> u16 { self.0.borrow().z_index() }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.0.borrow_mut().mark_dirty(); }
    fn clear_dirty(&mut self) { self.0.borrow_mut().clear_dirty(); }
    fn focusable(&self) -> bool { self.0.borrow().focusable() }

    fn render(&self, area: Rect) -> Plane {
        self.0.borrow().render(area)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.0.borrow_mut().handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.0.borrow_mut().handle_mouse(kind, col, row)
    }

    fn on_mount(&mut self) { self.0.borrow_mut().on_mount(); }
    fn on_unmount(&mut self) { self.0.borrow_mut().on_unmount(); }
    fn on_focus(&mut self) { self.0.borrow_mut().on_focus(); }
    fn on_blur(&mut self) { self.0.borrow_mut().on_blur(); }
    fn on_theme_change(&mut self, theme: &Theme) { self.0.borrow_mut().on_theme_change(theme); }
}

impl Widget for Showcase {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, id: WidgetId) { self.id = id; }
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; self.dirty = true; }
    fn z_index(&self) -> u16 { 10 }
    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let theme = Self::themes()[self.theme_idx];
        let mut p = Plane::new(0, area.width, area.height);
        p.z_index = 10;

        // Background
        for i in 0..p.cells.len() {
            p.cells[i].transparent = false;
            p.cells[i].bg = theme.bg;
            p.cells[i].fg = theme.fg;
        }

        // Title bar
        let title = " Dracon Terminal Engine — Example Showcase ";
        let title_color = Color::Rgb(0, 255, 200);
        for (i, c) in title.chars().enumerate() {
            if i < p.cells.len() {
                p.cells[i].char = c;
                p.cells[i].fg = title_color;
                p.cells[i].style = Styles::BOLD;
            }
        }
        // Top border
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = '─';
                p.cells[idx].fg = Color::Rgb(0, 150, 100);
            }
        }

        // Column headers
        let headers = ["Category", "Example", "Description", "Widgets Used"];
        let col_widths = [12u16, 20, 30, 30];
        let mut x_pos = 1u16;
        let header_y = 2;
        for (_i, (h, w)) in headers.iter().zip(col_widths.iter()).enumerate() {
            let end_x = (x_pos + w).min(area.width - 1);
            for x in x_pos..end_x {
                let idx = (header_y * area.width + x) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].bg = Color::Rgb(30, 35, 45);
                    p.cells[idx].fg = Color::Rgb(0, 200, 150);
                    p.cells[idx].style = Styles::BOLD;
                }
            }
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

        // Separator under headers
        let sep_y = 3;
        for x in 0..area.width {
            let idx = (sep_y * area.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = '─';
                p.cells[idx].fg = Color::Rgb(60, 65, 75);
            }
        }

        // List items (visible window)
        let list_start = 4;
        let list_height = (area.height as usize).saturating_sub(8);
        let visible_count = list_height.saturating_sub(2);

        // Find visible range centered on selection
        let total = self.examples.len();
        let start_idx = if self.selected < visible_count / 2 {
            0
        } else if self.selected >= total - visible_count / 2 {
            (total as isize - visible_count as isize).max(0) as usize
        } else {
            self.selected.saturating_sub(visible_count / 2)
        };
        let _end_idx = (start_idx + visible_count).min(total);

        for row in 0..visible_count {
            let idx = start_idx + row;
            if idx >= total { break; }

            let ex = &self.examples[idx];
            let y = (list_start + row) as u16;
            let is_selected = idx == self.selected;

            // Background for selected row
            if is_selected {
                for x in 0..area.width {
                    let cell_idx = (y * area.width + x) as usize;
                    if cell_idx < p.cells.len() {
                        p.cells[cell_idx].bg = Color::Rgb(0, 80, 70);
                        p.cells[cell_idx].fg = Color::Rgb(255, 255, 255);
                    }
                }
            }

            // Arrow prefix for selected
            let prefix = if is_selected { "> " } else { "  " };
            for (i, c) in prefix.chars().enumerate() {
                let cell_idx = (y * area.width + 1 + i as u16) as usize;
                if cell_idx < p.cells.len() {
                    p.cells[cell_idx].char = c;
                    p.cells[cell_idx].fg = if is_selected { Color::Rgb(0, 255, 200) } else { theme.fg };
                    p.cells[cell_idx].style = if is_selected { Styles::BOLD } else { Styles::empty() };
                }
            }

            // Category badge
            let cat_color = match ex.category {
                "cookbook" => Color::Rgb(100, 150, 255),
                "apps" => Color::Rgb(255, 150, 100),
                _ => Color::Rgb(150, 150, 150),
            };
            let cat_x = 3u16;
            for (i, c) in ex.category.chars().take(10).enumerate() {
                let cell_idx = (y * area.width + cat_x + i as u16) as usize;
                if cell_idx < p.cells.len() {
                    p.cells[cell_idx].char = c;
                    p.cells[cell_idx].fg = cat_color;
                    p.cells[cell_idx].style = Styles::BOLD;
                }
            }

            // Example name
            x_pos = 15;
            for (i, c) in ex.name.chars().take(18).enumerate() {
                let cell_idx = (y * area.width + x_pos + i as u16) as usize;
                if cell_idx < p.cells.len() {
                    p.cells[cell_idx].char = c;
                    p.cells[cell_idx].fg = if is_selected { Color::Rgb(255, 255, 255) } else { Color::Rgb(200, 200, 200) };
                }
            }

            // Description
            x_pos = 35;
            for (i, c) in ex.description.chars().take(28).enumerate() {
                let cell_idx = (y * area.width + x_pos + i as u16) as usize;
                if cell_idx < p.cells.len() {
                    p.cells[cell_idx].char = c;
                    p.cells[cell_idx].fg = if is_selected { Color::Rgb(200, 230, 255) } else { Color::Rgb(140, 140, 140) };
                }
            }

            // Widgets (truncated)
            x_pos = 65;
            for (i, c) in ex.widgets.chars().take(30).enumerate() {
                let cell_idx = (y * area.width + x_pos + i as u16) as usize;
                if cell_idx < p.cells.len() {
                    p.cells[cell_idx].char = c;
                    p.cells[cell_idx].fg = Color::Rgb(100, 100, 120);
                }
            }
        }

        // Bottom bar
        let status_y = area.height - 3;
        for x in 0..area.width {
            let idx = (status_y * area.width + x) as usize;
            if idx < p.cells.len() {
                p.cells[idx].bg = Color::Rgb(20, 25, 30);
            }
        }

        // Instructions
        let hint1 = "↑/↓ navigate";
        let hint2 = "Enter see run command";
        let hint3 = "t toggle theme";
        let hint4 = "q quit";
        let hint_x1 = 2;
        let hint_x2 = 22;
        let hint_x3 = 45;
        let hint_x4 = 62;

        for (i, c) in hint1.chars().enumerate() {
            let idx = (status_y * area.width + hint_x1 + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = Color::Rgb(0, 200, 150);
            }
        }
        for (i, c) in hint2.chars().enumerate() {
            let idx = (status_y * area.width + hint_x2 + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = Color::Rgb(150, 150, 150);
            }
        }
        for (i, c) in hint3.chars().enumerate() {
            let idx = (status_y * area.width + hint_x3 + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = Color::Rgb(150, 150, 150);
            }
        }
        for (i, c) in hint4.chars().enumerate() {
            let idx = (status_y * area.width + hint_x4 + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = Color::Rgb(150, 150, 150);
            }
        }

        // Count display
        let count_text = format!("{}/{} examples", self.selected + 1, total);
        let count_x = (area.width as isize - count_text.len() as isize - 2).max(0) as u16;
        for (i, c) in count_text.chars().enumerate() {
            let idx = (status_y * area.width + count_x + i as u16) as usize;
            if idx < p.cells.len() {
                p.cells[idx].char = c;
                p.cells[idx].fg = Color::Rgb(100, 150, 200);
            }
        }

        // Modal overlay (if shown)
        if self.show_modal {
            // Dim background
            for i in 0..p.cells.len() {
                p.cells[i].bg = Color::Ansi(0);
                p.cells[i].transparent = false;
            }

            // Modal box
            let mw = 55;
            let mh = 10u16;
            let mx = (area.width.saturating_sub(mw)) / 2;
            let my = (area.height.saturating_sub(mh)) / 2;

            // Box background
            for y in 0..mh {
                for x in 0..mw {
                    let idx = ((my + y) * area.width + mx + x) as usize;
                    if idx < p.cells.len() {
                        p.cells[idx].bg = Color::Rgb(15, 20, 25);
                        p.cells[idx].fg = Color::Rgb(200, 200, 200);
                        p.cells[idx].transparent = false;
                    }
                }
            }

            // Box border
            for x in 0..mw {
                let top_idx = (my * area.width + mx + x) as usize;
                let bot_idx = ((my + mh - 1) * area.width + mx + x) as usize;
                if top_idx < p.cells.len() { p.cells[top_idx].char = '─'; p.cells[top_idx].fg = Color::Rgb(0, 200, 150); }
                if bot_idx < p.cells.len() { p.cells[bot_idx].char = '─'; p.cells[bot_idx].fg = Color::Rgb(0, 200, 150); }
            }
            for y in 0..mh {
                let l_idx = (my + y) * area.width + mx;
                let r_idx = l_idx + mw - 1;
                if (l_idx as usize) < p.cells.len() { p.cells[l_idx as usize].char = '│'; p.cells[l_idx as usize].fg = Color::Rgb(0, 200, 150); }
                if (r_idx as usize) < p.cells.len() { p.cells[r_idx as usize].char = '│'; p.cells[r_idx as usize].fg = Color::Rgb(0, 200, 150); }
            }
            // Corners
            let corners = [(my, mx), (my, mx + mw - 1), (my + mh - 1, mx), (my + mh - 1, mx + mw - 1)];
            for (cy, cx) in corners {
                let idx = (cy * area.width + cx) as usize;
                if idx < p.cells.len() {
                    p.cells[idx].char = '+';
                    p.cells[idx].fg = Color::Rgb(0, 200, 150);
                }
            }

            // Modal content
            let ex = &self.examples[self.selected];
            let lines = [
                "┌─ Run Command ─────────────────────────────┐",
                &format!("│                                            │"),
                &format!("│   {}", ex.run_cmd),
                &format!("│                                            │"),
                "├────────────────────────────────────────────┤",
                "│                                            │",
                "│   Press any key to close                  │",
                "│                                            │",
                "└────────────────────────────────────────────┘",
            ];

            for (i, line) in lines.iter().enumerate() {
                let line_y = my + 1 + i as u16;
                for (j, c) in line.chars().enumerate() {
                    let idx = (line_y * area.width + mx + 1 + j as u16) as usize;
                    if idx < p.cells.len() {
                        p.cells[idx].char = c;
                        p.cells[idx].fg = if c == '─' || c == '│' || c == '+' || c == '┌' || c == '┐' || c == '└' || c == '┘' || c == '├' || c == '┤' {
                            Color::Rgb(0, 200, 150)
                        } else if c == '$' || c == 'c' || c == 'r' || c == 'g' || c == 'o' || c == '-' || c == 'e' || c == 'x' || c == 'm' || c == 'p' || c == 'l' || c == 'a' {
                            Color::Rgb(0, 255, 200)
                        } else {
                            Color::Rgb(180, 180, 180)
                        };
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
            self.dirty = true;
            return true;
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.examples.len() {
                    self.selected += 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Home => {
                self.selected = 0;
                self.dirty = true;
                true
            }
            KeyCode::End => {
                self.selected = self.examples.len().saturating_sub(1);
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                self.show_modal = true;
                self.dirty = true;
                true
            }
            KeyCode::Char('t') => {
                self.theme_idx = (self.theme_idx + 1) % Self::themes().len();
                self.dirty = true;
                true
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, row: u16) -> bool {
        if self.show_modal {
            self.show_modal = false;
            self.dirty = true;
            return true;
        }

        let list_start = 4u16;
        let list_height = (self.area.height as usize).saturating_sub(8) as u16;

        if kind == MouseEventKind::Down(MouseButton::Left) {
            if row >= list_start && row < list_start + list_height {
                let clicked = (row - list_start) as usize;
                // Map click to example index
                let visible_count = list_height.saturating_sub(2);
                let total = self.examples.len();

                let start_idx = if self.selected < visible_count as usize / 2 {
                    0
                } else if self.selected >= total - visible_count as usize / 2 {
                    (total as isize - visible_count as isize).max(0) as usize
                } else {
                    self.selected.saturating_sub(visible_count as usize / 2)
                };

                let idx = start_idx + clicked;
                if idx < total {
                    self.selected = idx;
                    self.dirty = true;
                    return true;
                }
            }
        }
        false
    }
}

fn main() -> io::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║         Dracon Terminal Engine — Example Showcase                ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Navigate: ↑/↓  |  See command: Enter  |  Quit: q              ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Starting showcase...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    let showcase = Rc::new(RefCell::new(Showcase::new()));
    showcase.borrow_mut().theme_idx = 0;

    let showcase_tick = showcase.clone();

    let mut app = App::new()?
        .title("Example Showcase")
        .fps(30)
        .theme(Theme::nord());
    app.add_widget(Box::new(ShowcaseWidget::new(showcase)), Rect::new(0, 0, 80, 24));
    app.on_tick(move |ctx, _tick| {
        let mut s = showcase_tick.borrow_mut();
        if s.should_quit {
            ctx.stop();
            return;
        }
        let (w, h) = ctx.compositor().size();
        if s.area.width != w || s.area.height != h {
            s.set_area(Rect::new(0, 0, w, h));
        }
    }).run(|_ctx| {})
}
