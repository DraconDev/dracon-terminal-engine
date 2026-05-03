#![allow(missing_docs)]
//! Dracon Terminal Engine — Example Showcase Launcher
//!
//! Interactive grid-based launcher for all framework examples.
//! Features: category filtering, real-time search, animated selection,
//! card-based layout with mini previews, and keyboard shortcuts.
//!
//! Controls:
//!   arrows / hjkl — navigate cards
//!   Enter — launch selected example
//!   / — focus search bar
//!   Tab — cycle categories
//!   t — cycle theme
//!   q — quit

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;

// ═══════════════════════════════════════════════════════════════════════════════
// DATA
// ═══════════════════════════════════════════════════════════════════════════════

struct ExampleMeta {
    name: &'static str,
    category: &'static str,
    description: &'static str,
    binary_name: &'static str,
    preview: &'static [&'static str],
}

impl ExampleMeta {
    fn all() -> Vec<Self> {
        vec![
            // Apps
            ExampleMeta { name: "system_monitor", category: "apps", description: "htop-like dashboard with live gauges", binary_name: "system_monitor", preview: &["┌ CPU ████████░░ 75%", "├ MEM ██████░░░░ 60%", "├ NET ▓▓▓▓▓▓▓▓▓▓", "└ PROC 142 running",] },
            ExampleMeta { name: "file_manager", category: "apps", description: "Full file manager UI with Tree + Table", binary_name: "file_manager", preview: &["> src/", "  ├ lib.rs", "  ├ main.rs", "  └ Cargo.toml",] },
            ExampleMeta { name: "chat_client", category: "apps", description: "Rich chat UI with emoji picker", binary_name: "chat_client", preview: &["[10:42] Alice: Hey!", "[10:43] Bob: 👋 Hi", "[10:44] Alice: 🎉", "> _",] },
            ExampleMeta { name: "dashboard_builder", category: "apps", description: "All command widgets in grid layout", binary_name: "dashboard_builder", preview: &["┌───┐ ┌───┐ ┌───┐", "│ 75│ │ 42│ │ 88│", "└───┘ └───┘ └───┘", "  CPU  MEM  DISK",] },
            // Cookbook
            ExampleMeta { name: "widget_gallery", category: "cookbook", description: "All interactive widgets in one App", binary_name: "widget_gallery", preview: &["[✓] Checkbox", "(•) Radio", "[||||] Slider", "Loading ████",] },
            ExampleMeta { name: "tree_navigator", category: "cookbook", description: "Hierarchical navigation with Tree", binary_name: "tree_navigator", preview: &["> home/", "  > user/", "    > src/", "      main.rs",] },
            ExampleMeta { name: "log_monitor", category: "cookbook", description: "Real-time log viewer with filtering", binary_name: "log_monitor", preview: &["[INFO] Starting", "[WARN] High mem", "[ERR] Timeout", "[INFO] Done",] },
            ExampleMeta { name: "tabbed_panels", category: "cookbook", description: "Tab switching with per-tab state", binary_name: "tabbed_panels", preview: &["[Tab1][Tab2][Tab3]", "┌──────────────┐", "│ Dashboard    │", "│ CPU: 45%     │",] },
            ExampleMeta { name: "data_table", category: "cookbook", description: "Sortable table with search/filter", binary_name: "data_table", preview: &["Name   │ Role  │", "───────┼───────", "Alice  │ Admin │", "Bob    │ User  │",] },
            ExampleMeta { name: "split_resizer", category: "cookbook", description: "Nested SplitPane with drag-to-resize", binary_name: "split_resizer", preview: &["┌─────┬─────┐", "│  A  │  B  │", "├─────┼─────┤", "│  C  │  D  │",] },
            ExampleMeta { name: "command_bindings", category: "cookbook", description: "5 command-bound widgets with auto-refresh", binary_name: "command_bindings", preview: &["Load: 0.45 0.32", "CPU:  ████░░", "Mem:  ██████", "Net:  ▓▓▓▓▓▓",] },
            ExampleMeta { name: "menu_system", category: "cookbook", description: "MenuBar + ContextMenu with shortcuts", binary_name: "menu_system", preview: &["[File][Edit][View]", "┌──────────┐", "│ New      │", "│ Open     │",] },
            ExampleMeta { name: "debug_overlay", category: "cookbook", description: "Debug tools overlay with F12 toggle", binary_name: "debug_overlay", preview: &["FPS: 60", "Frame: 16ms", "Widgets: 12", "Events: 45",] },
            // Tools
            ExampleMeta { name: "form_demo", category: "tools", description: "Settings form with validation", binary_name: "form_demo", preview: &["Username: alice", "Email: a@b.com", "Password: ****", "[Submit]",] },
            ExampleMeta { name: "modal_demo", category: "tools", description: "ConfirmDialog + help overlay", binary_name: "modal_demo", preview: &["┌─────────────┐", "│ Confirm?    │", "│ [Yes] [No]  │", "└─────────────┘",] },
            ExampleMeta { name: "theme_switcher", category: "tools", description: "Live theme cycling through all 15 themes", binary_name: "theme_switcher", preview: &["Theme: Nord", "┌──────────┐", "│ ■ ■ ■ ■  │", "│ ■ ■ ■ ■  │",] },
            ExampleMeta { name: "widget_tutorial", category: "tools", description: "Build a custom ColorPicker widget", binary_name: "widget_tutorial", preview: &["ColorPicker:", "┌──────────┐", "│ 🟥🟩🟦🟨 │", "│ 🟪🟧⬜⬛ │",] },
            ExampleMeta { name: "text_editor_demo", category: "tools", description: "Text editor widget demo", binary_name: "text_editor_demo", preview: &["fn main() {", "  println!(", "    \"Hello\"", "  );",] },
            ExampleMeta { name: "desktop", category: "tools", description: "Desktop environment mockup", binary_name: "desktop", preview: &["┌────┐ ┌────┐", "│Win1│ │Win2│", "│    │ │    │", "└────┘ └────┘",] },
            ExampleMeta { name: "game_loop", category: "tools", description: "Interactive game loop demo", binary_name: "game_loop", preview: &["  @     *", "     *   ", "  *   @  ", "    @    ",] },
            ExampleMeta { name: "input_debug", category: "tools", description: "Input event debugger", binary_name: "input_debug", preview: &["Key: Enter", "Code: 13", "Kind: Press", "Mod: Ctrl",] },
            ExampleMeta { name: "framework_file_manager", category: "tools", description: "File browser", binary_name: "framework_file_manager", preview: &["> src/", "  lib.rs", "  main.rs", "  Cargo.toml",] },
            ExampleMeta { name: "form_widget", category: "tools", description: "Form builder with labeled fields", binary_name: "form_widget", preview: &["Name:     ____", "Email:    ____", "Password: ****", "[Submit]",] },
            ExampleMeta { name: "table_widget", category: "tools", description: "Sortable data table", binary_name: "table_widget", preview: &["Name  │ Role │", "──────┼──────", "Alice │ Admin", "Bob   │ User",] },
            ExampleMeta { name: "ide", category: "apps", description: "Mini IDE with all widgets", binary_name: "ide", preview: &["[File][Edit][View]", "├─src/ ┌────────┐", "│ main │fn main│", "│ lib  │{      │", "└──────┴────────┘",] },
            ExampleMeta { name: "git_tui", category: "apps", description: "Real Git interface with status/log/diff/branches", binary_name: "git_tui", preview: &["[Status][Log][Diff]", " M src/main.rs", " A Cargo.toml", "?? README.md", "2 files changed",] },
            ExampleMeta { name: "sqlite_browser", category: "apps", description: "Database browser with query editor", binary_name: "sqlite_browser", preview: &["Tables  │ Query    │", "users   │ SELECT * │", "posts   │ FROM     │", "        │ users    │",] },
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHOWCASE STATE
// ═══════════════════════════════════════════════════════════════════════════════

struct Showcase {
    examples: Vec<ExampleMeta>,
    filtered: Vec<usize>,
    selected: usize,
    category_filter: Option<&'static str>,
    search_query: String,
    search_active: bool,
    theme_idx: usize,
    should_quit: Arc<AtomicBool>,
    pending_binary: Arc<Mutex<Option<String>>>,
    status_message: Option<(String, Instant)>,
}

impl Showcase {
    fn new(should_quit: Arc<AtomicBool>, pending: Arc<Mutex<Option<String>>>) -> Self {
        let examples = ExampleMeta::all();
        let filtered: Vec<usize> = (0..examples.len()).collect();
        Self {
            examples,
            filtered,
            selected: 0,
            category_filter: None,
            search_query: String::new(),
            search_active: false,
            theme_idx: 0,
            should_quit,
            pending_binary: pending,
            status_message: None,
        }
    }

    fn themes() -> Vec<Theme> {
        vec![Theme::nord(), Theme::cyberpunk(), Theme::dracula(), Theme::gruvbox_dark(), Theme::tokyo_night()]
    }

    fn current_theme(&self) -> Theme {
        Self::themes()[self.theme_idx % Self::themes().len()]
    }

    fn apply_filter(&mut self) {
        self.filtered = self.examples.iter().enumerate()
            .filter(|(_, ex)| {
                let matches_category = self.category_filter.map_or(true, |cat| ex.category == cat);
                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    ex.name.to_lowercase().contains(&q) ||
                    ex.description.to_lowercase().contains(&q) ||
                    ex.category.to_lowercase().contains(&q)
                };
                matches_category && matches_search
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
    }

    fn selected_example(&self) -> Option<&ExampleMeta> {
        self.filtered.get(self.selected).and_then(|&idx| self.examples.get(idx))
    }

    fn launch_selected(&mut self) {
        if let Some(ex) = self.selected_example() {
            *self.pending_binary.lock().unwrap() = Some(ex.binary_name.to_string());
            self.status_message = Some((format!("Launching {}...", ex.name), Instant::now()));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_rounded_border(plane: &mut Plane, area: Rect, fg: Color, bg: Color, selected: bool) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w < 2 || h < 2 { return; }

    let chars = if selected {
        ('╭', '╮', '╰', '╯', '─', '│', '▓')
    } else {
        ('┌', '┐', '└', '┘', '─', '│', '░')
    };

    // Corners
    set_cell(plane, 0, 0, chars.0, fg, bg);
    set_cell(plane, w - 1, 0, chars.1, fg, bg);
    set_cell(plane, 0, h - 1, chars.2, fg, bg);
    set_cell(plane, w - 1, h - 1, chars.3, fg, bg);

    // Top/bottom edges
    for x in 1..w - 1 {
        set_cell(plane, x, 0, chars.4, fg, bg);
        set_cell(plane, x, h - 1, chars.4, fg, bg);
    }

    // Left/right edges
    for y in 1..h - 1 {
        set_cell(plane, 0, y, chars.5, fg, bg);
        set_cell(plane, w - 1, y, chars.5, fg, bg);
    }

    // Fill background
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            set_cell(plane, x, y, ' ', fg, bg);
        }
    }
}

fn set_cell(plane: &mut Plane, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
    let idx = y * plane.width as usize + x;
    if idx < plane.cells.len() {
        plane.cells[idx] = Cell {
            char: ch,
            fg,
            bg,
            style: Styles::empty(),
            transparent: false,
            skip: false,
        };
    }
}

fn draw_text(plane: &mut Plane, x: usize, y: usize, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = y * plane.width as usize + x + i;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };
        }
    }
}

fn category_color(t: Theme, cat: &str) -> Color {
    match cat {
        "apps" => t.warning,
        "cookbook" => t.info,
        "tools" => t.secondary,
        _ => t.fg_muted,
    }
}

fn render_card(ex: &ExampleMeta, idx: usize, selected_idx: usize, t: Theme) -> Plane {
    let card_w = 28u16;
    let card_h = 14u16;
    let mut plane = Plane::new(0, card_w, card_h);

    let is_selected = idx == selected_idx;
    let cat_color = category_color(t, ex.category);

    // Border
    let border_fg = if is_selected { t.primary } else { t.outline };
    let bg = if is_selected { t.surface_elevated } else { t.surface };
    draw_rounded_border(&mut plane, Rect::new(0, 0, card_w, card_h), border_fg, bg, is_selected);

    // Category badge (top)
    let badge = format!(" {} ", ex.category.to_uppercase());
    let badge_x = 2usize;
    let badge_y = 1usize;
    for (i, ch) in badge.chars().enumerate() {
        let px = badge_x + i;
        if px < plane.width as usize - 2 {
            set_cell(&mut plane, px, badge_y, ch, t.fg_on_accent, cat_color);
        }
    }

    // Name (bold)
    let name_y = 3usize;
    let name_truncated = if ex.name.len() > 24 { &ex.name[..24] } else { ex.name };
    draw_text(&mut plane, 2, name_y, name_truncated, t.fg, bg, true);

    // Description
    let desc_y = 4usize;
    let desc = if ex.description.len() > 24 { &ex.description[..24] } else { ex.description };
    draw_text(&mut plane, 2, desc_y, desc, t.fg_muted, bg, false);

    // Preview (mini ASCII art)
    for (i, line) in ex.preview.iter().enumerate() {
        let py = 6 + i;
        if py < card_h as usize - 1 {
            let preview_line = if line.len() > 24 { &line[..24] } else { line };
            draw_text(&mut plane, 2, py, preview_line, t.fg_subtle, bg, false);
        }
    }

    // Selection indicator
    if is_selected {
        let indicator = "►";
        draw_text(&mut plane, 1, card_h as usize / 2, indicator, t.primary, bg, true);
    }

    plane
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET IMPL
// ═══════════════════════════════════════════════════════════════════════════════

impl Widget for Showcase {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
    fn set_area(&mut self, area: Rect) { let _ = area; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.current_theme();

        // Background fill
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title bar
        let title = " Dracon Terminal Engine ";
        let title_x = (area.width as usize - title.len()) / 2;
        draw_text(&mut plane, title_x, 0, title, t.primary, t.bg, true);

        // Search bar
        let search_y = 2usize;
        let search_prompt = if self.search_active { "> " } else { "  " };
        let search_text = format!("{}Search: {}_", search_prompt, self.search_query);
        let search_fg = if self.search_active { t.primary } else { t.fg_muted };
        draw_text(&mut plane, 2, search_y, &search_text, search_fg, t.surface, false);
        // Fill rest of search bar
        for x in search_text.len() + 2..area.width as usize - 2 {
            set_cell(&mut plane, x, search_y, ' ', search_fg, t.surface);
        }

        // Category sidebar
        let sidebar_w = 12usize;
        let categories = ["all", "apps", "cookbook", "tools"];
        for (i, cat) in categories.iter().enumerate() {
            let cat_y = 4 + i * 2;
            let is_active = self.category_filter.map_or(*cat == "all", |f| f == *cat);
            let (fg, bg_cat) = if is_active {
                (t.fg_on_accent, t.primary_active)
            } else {
                (t.fg_muted, t.bg)
            };
            let label = format!(" {} ", cat.to_uppercase());
            draw_text(&mut plane, 1, cat_y, &label, fg, bg_cat, is_active);

            // Count badge
            let count = if *cat == "all" {
                self.examples.len()
            } else {
                self.examples.iter().filter(|e| e.category == *cat).count()
            };
            let count_str = format!("{}", count);
            draw_text(&mut plane, 10, cat_y, &count_str, fg, bg_cat, false);
        }

        // Grid of cards
        let grid_start_x = sidebar_w + 2;
        let grid_start_y = 4usize;
        let card_w = 28usize;
        let card_h = 14usize;
        let cols = ((area.width as usize - grid_start_x) / (card_w + 2)).max(1);

        for (grid_idx, &ex_idx) in self.filtered.iter().enumerate() {
            if let Some(ex) = self.examples.get(ex_idx) {
                let col = grid_idx % cols;
                let row = grid_idx / cols;
                let x = grid_start_x + col * (card_w + 2);
                let y = grid_start_y + row * (card_h + 1);

                if x + card_w > area.width as usize || y + card_h > area.height as usize - 2 {
                    continue;
                }

                let card = render_card(ex, grid_idx, self.selected, t);
                for cy in 0..card_h {
                    for cx in 0..card_w {
                        let src_idx = (cy * card_w + cx) as usize;
                        let dst_idx = ((y + cy as usize) * area.width as usize + x + cx as usize) as usize;
                        if src_idx < card.cells.len() && dst_idx < plane.cells.len() {
                            if !card.cells[src_idx].transparent {
                                plane.cells[dst_idx] = card.cells[src_idx].clone();
                            }
                        }
                    }
                }
            }
        }

        // Scroll indicator
        let total_cards = self.filtered.len();
        let visible_cards = cols * ((area.height as usize - grid_start_y - 2) / (card_h + 1)).max(1);
        if total_cards > visible_cards {
            let scroll_text = format!("{} more", total_cards - visible_cards);
            draw_text(&mut plane, area.width as usize - scroll_text.len() - 2, area.height as usize - 3, &scroll_text, t.fg_muted, t.bg, false);
        }

        // Status bar
        let status_y = area.height as usize - 1;
        for x in 0..area.width as usize {
            set_cell(&mut plane, x, status_y, ' ', t.fg, t.surface_elevated);
        }

        let hints = ["↑↓←→ nav", "Enter launch", "/ search", "Tab category", "t theme", "q quit"];
        let mut hint_x = 2usize;
        for hint in hints.iter() {
            draw_text(&mut plane, hint_x, status_y, hint, t.primary, t.surface_elevated, false);
            hint_x += hint.len() + 3;
        }

        // Status message (temporary)
        if let Some((ref msg, time)) = self.status_message {
            if time.elapsed() < Duration::from_secs(2) {
                let msg_x = 2usize;
                let msg_y = area.height as usize - 2;
                draw_text(&mut plane, msg_x, msg_y, msg, t.warning, t.bg, true);
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        // Search mode
        if self.search_active {
            match key.code {
                KeyCode::Esc => {
                    self.search_active = false;
                    self.search_query.clear();
                    self.apply_filter();
                    true
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.apply_filter();
                    true
                }
                KeyCode::Char(ch) => {
                    self.search_query.push(ch);
                    self.apply_filter();
                    true
                }
                _ => false,
            }
        } else {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit.store(true, Ordering::SeqCst);
                    true
                }
                KeyCode::Char('t') => {
                    self.theme_idx = (self.theme_idx + 1) % Self::themes().len();
                    true
                }
                KeyCode::Char('/') => {
                    self.search_active = true;
                    true
                }
                KeyCode::Tab => {
                    let categories = [None, Some("apps"), Some("cookbook"), Some("tools")];
                    let current = categories.iter().position(|&c| c == self.category_filter).unwrap_or(0);
                    self.category_filter = categories[(current + 1) % categories.len()];
                    self.apply_filter();
                    true
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected + 1 < self.filtered.len() {
                        self.selected += 1;
                    }
                    true
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    true
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    let cols = 3; // approximate
                    if self.selected + cols < self.filtered.len() {
                        self.selected += cols;
                    }
                    true
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    let cols = 3; // approximate
                    if self.selected >= cols {
                        self.selected -= cols;
                    }
                    true
                }
                KeyCode::Enter => {
                    self.launch_selected();
                    true
                }
                _ => false,
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Dracon Terminal Engine — Example Showcase");
    println!("Grid launcher with search, categories, and live previews");
    std::thread::sleep(Duration::from_millis(500));

    let pending = Arc::new(Mutex::new(None));
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let showcase = Showcase::new(should_quit, pending.clone());

    let mut app = App::new()?.title("Dracon Showcase").fps(30).theme(Theme::nord());
    app.add_widget(Box::new(showcase), Rect::new(0, 0, 80, 24));

    app.on_tick(move |ctx, _tick| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }

        // Handle pending binary launch
        if let Some(binary_name) = pending.lock().unwrap().take() {
            let exe_dir = match std::env::current_exe() {
                Ok(p) => p.parent().unwrap().to_path_buf(),
                Err(_) => return,
            };
            let binary_path = exe_dir.join(&binary_name);

            let _ = ctx.suspend_terminal();

            // Auto-build if missing
            if !binary_path.exists() {
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
                    let _ = std::process::Command::new("cargo")
                        .args(["build", "--example", &binary_name])
                        .current_dir(&crate_root)
                        .status();
                }
            }

            let _ = std::process::Command::new(&binary_path)
                .current_dir(&exe_dir)
                .status();

            let mut drain_buf = [0u8; 256];
            let _ = std::io::stdin().read(&mut drain_buf);

            let _ = ctx.resume_terminal();
            ctx.mark_all_dirty();
        }
    }).run(|ctx| {
        let (w, h) = ctx.compositor().size();
        // Re-render with current size
        let _ = (w, h);
    })
}
