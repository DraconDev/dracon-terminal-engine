#![allow(missing_docs)]
//! Rich TextEditor Demo — mini-IDE experience with tabs, file tree, and search.
//!
//! Features:
//! - Tab bar: switch between open files
//! - File tree sidebar with icons
//! - Search bar overlay
//! - Syntax highlighting (Rust basics)
//! - Line numbers with current line highlight
//! - Status bar with position, language, encoding
//! - Theme cycling (t)
//! - Help overlay (?)
//!
//! Controls:
//!   Ctrl+T      — new tab
//!   Ctrl+W      — close tab
//!   Ctrl+F      — toggle search
//!   Ctrl+S      — save mock
//!   Tab         — next tab
//!   ↑/↓/←/→     — navigate
//!   t           — cycle theme
//!   ?           — help overlay
//!   q           — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingConfig, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, CommandItem, CommandPalette, SearchInput, StatusBar, StatusSegment, TabBar, Tree,
    TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct Tab {
    title: String,
    content: String,
    cursor_line: usize,
    cursor_col: usize,
    modified: bool,
}

struct EditorApp {
    id: WidgetId,
    tabs: Vec<Tab>,
    active_tab: usize,
    tab_bar: TabBar,
    file_tree: Tree,
    show_search: bool,
    search: SearchInput,
    status_bar: StatusBar,
    breadcrumbs: Breadcrumbs,
    show_help: bool,
    theme: Theme,
    area: Rect,
    dirty: bool,
    should_quit: Arc<AtomicBool>,
    command_palette: CommandPalette,
    cmd_bridge: Rc<RefCell<Option<String>>>,
    keybindings: KeybindingSet,
    kb_config: KeybindingConfig,
}

impl EditorApp {
    fn new(should_quit: Arc<AtomicBool>) -> Self {
        let theme = Theme::nord();

        let tabs = vec![
            Tab {
                title: "main.rs".to_string(),
                content: "use std::io::{self, Write};\n\nfn main() -> io::Result<()> {\n    println!(\"Hello, World!\");\n    Ok(())\n}\n".to_string(),
                cursor_line: 0,
                cursor_col: 0,
                modified: false,
            },
            Tab {
                title: "lib.rs".to_string(),
                content: "/// A greeting function\npub fn greet(name: &str) -> String {\n    format!(\"Hello, {}!\", name)\n}\n".to_string(),
                cursor_line: 0,
                cursor_col: 0,
                modified: false,
            },
        ];

        let tab_bar = TabBar::new_with_id(WidgetId::new(10), vec!["main.rs", "lib.rs"]);

        let tree = build_file_tree(theme);

        let search = SearchInput::new(WidgetId::new(20)).with_theme(theme);

        let kb_config = resolve_keybindings();
        let keybindings = KeybindingSet::from_config(&kb_config);
        let kb_theme = kb_config.get(actions::THEME).unwrap_or("t");
        let kb_help = kb_config.get(actions::HELP).unwrap_or("?");
        let kb_back = kb_config.get(actions::BACK).unwrap_or("Esc");
        let kb_quit = kb_config.get(actions::QUIT).unwrap_or("q");

        let status = StatusBar::new(WidgetId::new(30))
            .add_segment(StatusSegment::new("󰄛 Rust").with_fg(theme.info))
            .add_segment(StatusSegment::new("Ln 1, Col 1").with_fg(theme.fg_muted))
            .add_segment(StatusSegment::new("UTF-8").with_fg(theme.fg_muted))
            .add_segment(
                StatusSegment::new(&format!(
                    "{}: theme | {}: help | {}: dismiss | {}: quit",
                    kb_theme, kb_help, kb_back, kb_quit
                )).with_fg(theme.fg_muted),
            );

        let breadcrumbs = Breadcrumbs::new(vec!["src".into(), "main.rs".into()]);

        let palette_commands: Vec<CommandItem> = vec![
            CommandItem {
                id: "new-tab",
                name: "New Tab",
                category: "file",
            },
            CommandItem {
                id: "open",
                name: "Open File",
                category: "file",
            },
            CommandItem {
                id: "save",
                name: "Save",
                category: "file",
            },
            CommandItem {
                id: "close-tab",
                name: "Close Tab",
                category: "file",
            },
            CommandItem {
                id: "search",
                name: "Search",
                category: "edit",
            },
            CommandItem {
                id: "cut",
                name: "Cut",
                category: "edit",
            },
            CommandItem {
                id: "copy",
                name: "Copy",
                category: "edit",
            },
            CommandItem {
                id: "paste",
                name: "Paste",
                category: "edit",
            },
            CommandItem {
                id: "cycle-theme",
                name: "Cycle Theme",
                category: "view",
            },
            CommandItem {
                id: "toggle-profiler",
                name: "Toggle Profiler",
                category: "view",
            },
            CommandItem {
                id: "show-shortcuts",
                name: "Show Shortcuts",
                category: "help",
            },
            CommandItem {
                id: "about",
                name: "About",
                category: "help",
            },
        ];

        let cmd_bridge: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let cmd_bridge_clone = cmd_bridge.clone();
        let command_palette = CommandPalette::new(palette_commands)
            .with_size(45, 18)
            .with_theme(theme)
            .on_execute(move |cmd_id| {
                *cmd_bridge_clone.borrow_mut() = Some(cmd_id.to_string());
            });

        Self {
            id: WidgetId::new(0),
            tabs,
            active_tab: 0,
            tab_bar,
            file_tree: tree,
            show_search: false,
            search,
            status_bar: status,
            breadcrumbs,
            show_help: false,
            theme,
            area: Rect::new(0, 0, 80, 24),
            dirty: true,
            should_quit,
            command_palette,
            cmd_bridge,
            keybindings,
            kb_config,
        }
    }

    fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }

    fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab)
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::dark(),
            Theme::light(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::nord(),
            Theme::catppuccin_mocha(),
            Theme::gruvbox_dark(),
            Theme::tokyo_night(),
            Theme::solarized_dark(),
            Theme::solarized_light(),
            Theme::one_dark(),
            Theme::rose_pine(),
            Theme::kanagawa(),
            Theme::everforest(),
            Theme::monokai(),
            Theme::warm(),
            Theme::cool(),
            Theme::forest(),
            Theme::sunset(),
            Theme::mono(),
        ];
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.tab_bar.on_theme_change(&self.theme);
        self.file_tree.on_theme_change(&self.theme);
        self.search.on_theme_change(&self.theme);
        self.status_bar.on_theme_change(&self.theme);
        self.breadcrumbs.on_theme_change(&self.theme);
        self.dirty = true;
    }

    fn sync_tab_bar(&mut self) {
        let labels: Vec<String> = self
            .tabs
            .iter()
            .map(|t| {
                if t.modified {
                    format!("{} ×", t.title)
                } else {
                    t.title.clone()
                }
            })
            .collect();
        self.tab_bar = TabBar::new_with_id(
            WidgetId::new(10),
            labels.iter().map(|s| s.as_str()).collect(),
        );
        self.tab_bar.set_active(self.active_tab);
        self.tab_bar.on_theme_change(&self.theme);
    }

    fn open_command_palette(&mut self) {
        let pw = 45u16;
        let ph = 18u16;
        let ox = (self.area.width.saturating_sub(pw)) / 2;
        let oy = (self.area.height.saturating_sub(ph)) / 2;
        self.command_palette.set_area(Rect::new(ox, oy, pw, ph));
        self.command_palette.show();
    }

    fn dispatch_palette_command(&mut self, cmd_id: &str) {
        match cmd_id {
            "new-tab" => {
                let new_id = self.tabs.len();
                self.tabs.push(Tab {
                    title: format!("untitled-{}.rs", new_id + 1),
                    content: "// New file\n".to_string(),
                    cursor_line: 0,
                    cursor_col: 0,
                    modified: false,
                });
                self.active_tab = new_id;
                self.sync_tab_bar();
            }
            "close-tab" if self.tabs.len() > 1 => {
                self.tabs.remove(self.active_tab);
                self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
                self.sync_tab_bar();
            }
            "save" => {
                if let Some(tab) = self.active_tab_mut() {
                    tab.modified = false;
                }
                self.sync_tab_bar();
            }
            "search" => {
                self.show_search = true;
            }
            "cycle-theme" => {
                self.cycle_theme();
            }
            "show-shortcuts" => {
                self.show_help = true;
            }
            _ => {}
        }
        self.dirty = true;
    }
}

fn build_file_tree(theme: Theme) -> Tree {
    let root = TreeNode {
        label: "󰉋 project".into(),
        expanded: true,
        children: vec![
            TreeNode {
                label: format!("{}Cargo.toml", file_icon("Cargo.toml")),
                expanded: false,
                children: vec![],
            },
            TreeNode {
                label: "󰉋 src".into(),
                expanded: true,
                children: vec![
                    TreeNode {
                        label: format!("{}main.rs", file_icon("main.rs")),
                        expanded: false,
                        children: vec![],
                    },
                    TreeNode {
                        label: format!("{}lib.rs", file_icon("lib.rs")),
                        expanded: false,
                        children: vec![],
                    },
                ],
            },
            TreeNode {
                label: "󰉋 examples".into(),
                expanded: false,
                children: vec![TreeNode {
                    label: format!("{}demo.rs", file_icon("demo.rs")),
                    expanded: false,
                    children: vec![],
                }],
            },
            TreeNode {
                label: format!("{}README.md", file_icon("README.md")),
                expanded: false,
                children: vec![],
            },
        ],
    };
    Tree::new(WidgetId::new(11))
        .with_root(vec![root])
        .with_theme(theme)
}

fn file_icon(name: &str) -> &'static str {
    if name.ends_with(".rs") {
        " "
    } else if name.ends_with(".toml") {
        " "
    } else if name.ends_with(".md") {
        " "
    } else if name.ends_with(".json") {
        " "
    } else if name.ends_with(".js") || name.ends_with(".ts") {
        " "
    } else if name.ends_with(".py") {
        " "
    } else {
        " "
    }
}

impl Widget for EditorApp {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
        self.dirty = true;
    }
    fn needs_render(&self) -> bool {
        self.dirty
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
    fn focusable(&self) -> bool {
        true
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.file_tree.on_theme_change(theme);
        self.search.on_theme_change(theme);
        self.tab_bar.on_theme_change(theme);
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.theme;

        // Background
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let sidebar_w = 16u16;
        let tab_h = 1u16;
        let search_h = if self.show_search { 2u16 } else { 0u16 };
        let status_h = 1u16;
        let content_y = tab_h;
        let content_h = area.height.saturating_sub(tab_h + status_h + search_h);

        // Tab bar
        let tab_plane = self.tab_bar.render(Rect::new(0, 0, area.width, tab_h));
        blit(&mut plane, &tab_plane, 0, 0);

        // Sidebar (file tree)
        let sidebar_rect = Rect::new(0, content_y, sidebar_w, content_h);
        let sidebar_plane = self.file_tree.render(sidebar_rect);
        blit(&mut plane, &sidebar_plane, 0, content_y);

        // Separator
        for y in content_y..content_y + content_h {
            let idx = (y * area.width + sidebar_w) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Editor area
        let editor_x = sidebar_w + 1;
        let editor_w = area.width.saturating_sub(editor_x);
        let editor_y = content_y;

        if editor_w > 0 && content_h > 2 {
            // Rounded border
            draw_rounded_border(&mut plane, editor_x, editor_y, editor_w, content_h, t);

            // Breadcrumbs inside editor
            let bc_plane =
                self.breadcrumbs
                    .render(Rect::new(editor_x + 1, editor_y, editor_w - 2, 1));
            blit(&mut plane, &bc_plane, editor_x + 1, editor_y);

            // Editor content
            let text_y = editor_y + 1;
            let text_h = content_h.saturating_sub(2);
            if let Some(tab) = self.active_tab() {
                render_editor_content(
                    &mut plane,
                    editor_x + 1,
                    text_y,
                    editor_w - 2,
                    text_h,
                    tab,
                    t,
                );
            }
        }

        // Search overlay
        if self.show_search && search_h > 0 {
            let search_y = area.height - status_h - search_h;
            for y in search_y..search_y + search_h {
                for x in 0..area.width {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                    }
                }
            }
            draw_text(
                &mut plane,
                2,
                search_y,
                "Find:",
                t.primary,
                t.surface_elevated,
                true,
            );
            let search_plane = self.search.render(Rect::new(8, search_y, 30, 1));
            blit(&mut plane, &search_plane, 8, search_y);
        }

        // Status bar
        let status_y = area.height - status_h;
        let status_plane = self
            .status_bar
            .render(Rect::new(0, status_y, area.width, status_h));
        blit(&mut plane, &status_plane, 0, status_y);

        // Help overlay
        if self.show_help {
            render_help_overlay(&mut plane, area, t, &self.kb_config);
        }

        // Command palette overlay
        if self.command_palette.is_visible() {
            let pal_plane = self.command_palette.render(area);
            blit(&mut plane, &pal_plane, 0, 0);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Command palette takes priority when visible
        if self.command_palette.is_visible() {
            let _handled = self.command_palette.handle_key(key);
            let cmd = self.cmd_bridge.borrow_mut().take();
            if let Some(ref cmd_id) = cmd {
                self.dispatch_palette_command(cmd_id);
            }
            return true;
        }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.show_search {
            if self.keybindings.matches(actions::BACK, &key) {
                self.show_search = false;
                self.dirty = true;
                return true;
            }
            let handled = self.search.handle_key(key);
            self.dirty = true;
            return handled;
        }

        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }

        match key.code {
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.show_search = !self.show_search;
                self.dirty = true;
                true
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(tab) = self.active_tab_mut() {
                    tab.modified = false;
                }
                self.sync_tab_bar();
                self.dirty = true;
                true
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_command_palette();
                self.dirty = true;
                true
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let new_id = self.tabs.len();
                self.tabs.push(Tab {
                    title: format!("untitled-{}.rs", new_id + 1),
                    content: "// New file\n".to_string(),
                    cursor_line: 0,
                    cursor_col: 0,
                    modified: false,
                });
                self.active_tab = new_id;
                self.sync_tab_bar();
                self.dirty = true;
                true
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.tabs.len() > 1 {
                    self.tabs.remove(self.active_tab);
                    self.active_tab = self.active_tab.min(self.tabs.len() - 1);
                    self.sync_tab_bar();
                    self.dirty = true;
                }
                true
            }
            KeyCode::Tab => {
                self.active_tab = (self.active_tab + 1) % self.tabs.len();
                self.tab_bar.set_active(self.active_tab);
                self.dirty = true;
                true
            }
            KeyCode::Up => {
                if let Some(tab) = self.active_tab_mut() {
                    if tab.cursor_line > 0 {
                        tab.cursor_line -= 1;
                    }
                }
                self.dirty = true;
                true
            }
            KeyCode::Down => {
                if let Some(tab) = self.active_tab_mut() {
                    let lines: Vec<&str> = tab.content.lines().collect();
                    if tab.cursor_line + 1 < lines.len() {
                        tab.cursor_line += 1;
                    }
                }
                self.dirty = true;
                true
            }
            KeyCode::Left => {
                if let Some(tab) = self.active_tab_mut() {
                    if tab.cursor_col > 0 {
                        tab.cursor_col -= 1;
                    }
                }
                self.dirty = true;
                true
            }
            KeyCode::Right => {
                if let Some(tab) = self.active_tab_mut() {
                    let lines: Vec<&str> = tab.content.lines().collect();
                    if let Some(line) = lines.get(tab.cursor_line) {
                        if tab.cursor_col < line.len() {
                            tab.cursor_col += 1;
                        }
                    }
                }
                self.dirty = true;
                true
            }
            KeyCode::Backspace => {
                if let Some(tab) = self.active_tab_mut() {
                    let lines: Vec<&str> = tab.content.lines().collect();
                    if tab.cursor_col > 0 {
                        // Delete character before cursor on current line
                        let mut new_content = String::new();
                        for (i, line) in lines.iter().enumerate() {
                            if i > 0 { new_content.push('\n'); }
                            if i == tab.cursor_line {
                                new_content.push_str(&line[..tab.cursor_col - 1]);
                                new_content.push_str(&line[tab.cursor_col..]);
                            } else {
                                new_content.push_str(line);
                            }
                        }
                        tab.content = new_content;
                        tab.cursor_col -= 1;
                        tab.modified = true;
                    } else if tab.cursor_line > 0 {
                        // At start of line: join with previous line
                        let prev_line_len = lines[tab.cursor_line - 1].len();
                        let mut new_content = String::new();
                        for (i, line) in lines.iter().enumerate() {
                            if i == tab.cursor_line { continue; }
                            if i > 0 && i != tab.cursor_line { new_content.push('\n'); }
                            if i == tab.cursor_line - 1 {
                                new_content.push_str(line);
                                new_content.push_str(lines[tab.cursor_line]);
                            } else {
                                new_content.push_str(line);
                            }
                        }
                        tab.content = new_content;
                        tab.cursor_line -= 1;
                        tab.cursor_col = prev_line_len;
                        tab.modified = true;
                    }
                }
                self.sync_tab_bar();
                self.dirty = true;
                true
            }
            KeyCode::Char(c) => {
                if let Some(tab) = self.active_tab_mut() {
                    let lines: Vec<&str> = tab.content.lines().collect();
                    let mut new_content = String::new();
                    for (i, line) in lines.iter().enumerate() {
                        if i > 0 {
                            new_content.push('\n');
                        }
                        if i == tab.cursor_line {
                            let col = tab.cursor_col.min(line.len());
                            new_content.push_str(&line[..col]);
                            new_content.push(c);
                            new_content.push_str(&line[col..]);
                            tab.cursor_col = col + 1;
                        } else {
                            new_content.push_str(line);
                        }
                    }
                    tab.content = new_content;
                    tab.modified = true;
                }
                self.sync_tab_bar();
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Command palette intercepts all mouse events when visible
        if self.command_palette.is_visible() {
            let _handled = self.command_palette.handle_mouse(kind, col, row);
            return true;
        }

        if let MouseEventKind::Down(MouseButton::Left) = kind {
            // Tab bar click
            if row == 0 {
                let tab_count = self.tabs.len().max(1);
                let tab_width = (self.area.width / tab_count as u16).max(1);
                let idx = (col / tab_width) as usize;
                if idx < self.tabs.len() {
                    self.active_tab = idx;
                    self.tab_bar.set_active(idx);
                    self.dirty = true;
                    return true;
                }
            }
        }
        false
    }
}

fn render_editor_content(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, tab: &Tab, t: Theme) {
    let lines: Vec<&str> = tab.content.lines().collect();
    let line_num_width = lines.len().to_string().len().max(2) as u16;
    let content_x = x + line_num_width + 1;

    for (i, line) in lines.iter().enumerate().take(h as usize) {
        let row = y + i as u16;
        if row >= plane.height {
            break;
        }

        // Line number
        let num = format!("{:>width$} ", i + 1, width = line_num_width as usize);
        for (j, ch) in num.chars().enumerate() {
            let idx = (row * plane.width + x + j as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: ch,
                    fg: t.fg_muted,
                    bg: if i == tab.cursor_line {
                        t.surface_elevated
                    } else {
                        t.bg
                    },
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        // Line content with basic syntax highlighting
        for (j, ch) in line
            .chars()
            .enumerate()
            .take((w - line_num_width - 1) as usize)
        {
            let col = content_x + j as u16;
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() || col >= plane.width {
                break;
            }

            let (fg, style) = syntax_color(line, j, t);

            plane.cells[idx] = Cell {
                char: ch,
                fg,
                bg: if i == tab.cursor_line {
                    t.surface_elevated
                } else {
                    t.bg
                },
                style,
                transparent: false,
                skip: false,
            };
        }
    }

    // Cursor
    let cursor_row = y + tab.cursor_line as u16;
    let cursor_col = content_x + tab.cursor_col as u16;
    if cursor_row < plane.height && cursor_col < plane.width {
        let idx = (cursor_row * plane.width + cursor_col) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx].bg = t.primary_active;
            plane.cells[idx].fg = t.fg_on_accent;
        }
    }
}

fn syntax_color(line: &str, col: usize, t: Theme) -> (Color, Styles) {
    let keywords = [
        "fn", "let", "mut", "pub", "use", "if", "else", "for", "while", "match", "impl", "struct",
        "enum", "trait", "type", "return", "self", "true", "false", "const", "static", "async",
        "await",
    ];
    let types = [
        "String", "Vec", "Option", "Result", "i32", "u32", "f64", "bool", "char", "usize",
    ];

    // Check if we're in a comment
    if let Some(pos) = line.find("//") {
        if col >= pos {
            return (t.fg_muted, Styles::empty());
        }
    }

    // Check if we're in a string literal
    let mut in_string = false;
    let mut string_start = 0;
    for (i, c) in line.chars().enumerate() {
        if c == '"' {
            if in_string {
                if col >= string_start && col <= i {
                    return (t.success, Styles::empty());
                }
                in_string = false;
            } else {
                in_string = true;
                string_start = i;
            }
        }
    }
    if in_string && col >= string_start {
        return (t.success, Styles::empty());
    }

    // Check for keywords and types
    for word in keywords {
        if let Some(pos) = line.find(word) {
            if col >= pos && col < pos + word.len() {
                return (t.primary, Styles::BOLD);
            }
        }
    }
    for word in types {
        if let Some(pos) = line.find(word) {
            if col >= pos && col < pos + word.len() {
                return (t.warning, Styles::BOLD);
            }
        }
    }

    // Numbers
    let chars: Vec<char> = line.chars().collect();
    if col < chars.len() && chars[col].is_ascii_digit() {
        return (t.warning, Styles::empty());
    }

    (t.fg, Styles::empty())
}

fn render_help_overlay(plane: &mut Plane, area: Rect, t: Theme, kb_config: &KeybindingConfig) {
    let w = 50u16.min(area.width - 4);
    let h = 14u16.min(area.height - 4);
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;

    // Background
    for py in 0..h {
        for px in 0..w {
            let idx = ((y + py) * area.width + x + px) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].transparent = false;
            }
        }
    }

    // Border
    draw_rounded_border(plane, x, y, w, h, t);

    // Title
    let title = "Keyboard Shortcuts";
    let title_x = x + (w - title.len() as u16) / 2;
    draw_text(
        plane,
        title_x,
        y + 1,
        title,
        t.primary,
        t.surface_elevated,
        true,
    );

    let kb_theme = kb_config.get(actions::THEME).unwrap_or("t");
    let kb_help = kb_config.get(actions::HELP).unwrap_or("?");
    let kb_back = kb_config.get(actions::BACK).unwrap_or("Esc");
    let kb_quit = kb_config.get(actions::QUIT).unwrap_or("q");

    let shortcuts = [
        ("Ctrl+T", "New tab"),
        ("Ctrl+W", "Close tab"),
        ("Ctrl+F", "Search"),
        ("Ctrl+S", "Save mock"),
        ("Tab", "Next tab"),
        ("↑↓←→", "Navigate"),
        (kb_theme, "Cycle theme"),
        (kb_help, "Toggle help"),
        (kb_back, "Dismiss help / search"),
        (kb_quit, "Quit"),
    ];

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = y + 3 + i as u16;
        draw_text(plane, x + 3, row, key, t.primary, t.surface_elevated, true);
        draw_text(plane, x + 15, row, desc, t.fg, t.surface_elevated, false);
    }

    let hint = "Press ? or Esc to close";
    draw_text(
        plane,
        x + 3,
        y + h - 1,
        hint,
        t.fg_muted,
        t.surface_elevated,
        false,
    );
}

fn draw_rounded_border(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: Theme) {
    if w < 3 || h < 2 {
        return;
    }
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() {
                continue;
            }
            let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
            if is_border {
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].char = if row == y && col == x {
                    '╭'
                } else if row == y && col == x + w - 1 {
                    '╮'
                } else if row == y + h - 1 && col == x {
                    '╰'
                } else if row == y + h - 1 && col == x + w - 1 {
                    '╯'
                } else if row == y || row == y + h - 1 {
                    '─'
                } else {
                    '│'
                };
                plane.cells[idx].transparent = false;
            }
        }
    }
}

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
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

fn blit(dst: &mut Plane, src: &Plane, dx: u16, dy: u16) {
    for (i, cell) in src.cells.iter().enumerate() {
        if cell.transparent {
            continue;
        }
        let x = (i % src.width as usize) as u16 + dx;
        let y = (i / src.width as usize) as u16 + dy;
        let idx = (y * dst.width + x) as usize;
        if idx < dst.cells.len() && x < dst.width && y < dst.height {
            dst.cells[idx] = cell.clone();
        }
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let (_w, _h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let app_widget = EditorApp::new(should_quit.clone());

    App::new()?
        .title("Text Editor Demo")
        .fps(30)
        .theme(Theme::from_env_or(Theme::nord()))
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
        .run(move |ctx| {
            let (w, h) = ctx.compositor().size();
            ctx.add_plane(app_widget.render(Rect::new(0, 0, w, h)));
        })
}
