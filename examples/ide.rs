#![allow(missing_docs)]
//! IDE Example — Flagship demo showing ALL framework widgets.
//!
//! A mini-IDE with file tree, editor tabs, menu bar, search, and more.
//!
//! Controls:
//!   Ctrl+O      — open file
//!   Ctrl+S      — save file
//!   Ctrl+F      — toggle search
//!   Ctrl+T      — new tab
//!   Ctrl+W      — close tab
//!   Ctrl+G      — go to line
//!   F12         — toggle profiler overlay
//!   t           — cycle theme
//!   Ctrl+P      — command palette
//!   q / Ctrl+Q  — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, CommandItem, CommandPalette, ContextAction, ContextMenu, MenuBar, MenuEntry,
    MenuItem, Modal, Profiler, SearchInput, StatusBar, StatusSegment, TabBar,
    Toast, ToastKind, Tooltip, Tree, TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════════════════
// EDITOR TAB
// ═══════════════════════════════════════════════════════════════════════════════

struct EditorTab {
    title: String,
    path: Option<PathBuf>,
    content: String,
    cursor_line: usize,
    cursor_col: usize,
    modified: bool,
}

impl EditorTab {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            path: None,
            content: String::new(),
            cursor_line: 0,
            cursor_col: 0,
            modified: false,
        }
    }

    fn with_content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IDE STATE
// ═══════════════════════════════════════════════════════════════════════════════

struct IdeApp {
    should_quit: Arc<AtomicBool>,
    theme: Theme,

    // Layout
    area: Rect,

    // Menu
    menu_bar: MenuBar,
    show_settings: bool,
    settings_modal: Modal<'static>,

    // Tabs
    tabs: Vec<EditorTab>,
    active_tab: usize,
    tab_bar: TabBar,

    // File tree
    file_tree: Tree,

    // Search
    show_search: bool,
    search_input: SearchInput,

    // Status
    status_bar: StatusBar,

    // Toasts
    toasts: Vec<Toast>,

    // Tooltip
    tooltip: Option<Tooltip>,
    tooltip_timer: Option<Instant>,

    // Context menu
    context_menu: Option<ContextMenu>,

    // Profiler overlay
    profiler: Profiler,
    show_profiler: bool,

    // Breadcrumbs
    breadcrumbs: Breadcrumbs,

    // Command palette
    command_palette: CommandPalette,
    cmd_bridge: Rc<RefCell<Option<String>>>,

    // Animation
    anim_frame: u8,
    last_anim: Instant,
}

impl IdeApp {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let tabs = vec![
            EditorTab::new("main.rs").with_content(
                "use std::io;\n\nfn main() {\n    println!(\"Hello, Dracon!\");\n}\n"
            ),
            EditorTab::new("lib.rs").with_content(
                "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {}!\", name)\n}\n"
            ),
        ];

        let tab_titles = vec!["main.rs", "lib.rs"];
        let tab_bar = TabBar::new_with_id(WidgetId::new(2), tab_titles);

        let file_tree = build_sample_tree();

        let menu_bar = MenuBar::new(WidgetId::new(1))
            .with_theme(theme)
            .with_entries(vec![
                MenuEntry::new("File")
                    .add_item(MenuItem::new("New Tab (Ctrl+T)"))
                    .add_item(MenuItem::new("Open (Ctrl+O)"))
                    .add_item(MenuItem::new("Save (Ctrl+S)"))
                    .add_item(MenuItem::new("Exit (Ctrl+Q)")),
                MenuEntry::new("Edit")
                    .add_item(MenuItem::new("Cut"))
                    .add_item(MenuItem::new("Copy"))
                    .add_item(MenuItem::new("Paste")),
                MenuEntry::new("View")
                    .add_item(MenuItem::new("Search (Ctrl+F)"))
                    .add_item(MenuItem::new("Profiler (F12)")),
                MenuEntry::new("Theme")
                    .add_item(MenuItem::new("Cycle (t)")),
                MenuEntry::new("Help")
                    .add_item(MenuItem::new("Shortcuts"))
                    .add_item(MenuItem::new("About")),
            ]);

        let search_input = SearchInput::new(WidgetId::new(3)).with_theme(theme);

        let status_bar = StatusBar::new(WidgetId::new(4))
            .add_segment(StatusSegment::new("Ready").with_fg(theme.success))
            .add_segment(StatusSegment::new("Ln 1, Col 1").with_fg(theme.fg_muted))
            .add_segment(StatusSegment::new("Rust").with_fg(theme.info))
            .add_segment(StatusSegment::new("UTF-8").with_fg(theme.fg_muted));

        let breadcrumbs = Breadcrumbs::new(vec!["workspace".into(), "src".into(), "main.rs".into()]);

        let palette_commands: Vec<CommandItem> = vec![
            CommandItem { id: "new-tab", name: "New Tab", category: "file" },
            CommandItem { id: "open", name: "Open File", category: "file" },
            CommandItem { id: "save", name: "Save", category: "file" },
            CommandItem { id: "save-all", name: "Save All", category: "file" },
            CommandItem { id: "close-tab", name: "Close Tab", category: "file" },
            CommandItem { id: "search", name: "Search (Ctrl+F)", category: "edit" },
            CommandItem { id: "replace", name: "Find and Replace", category: "edit" },
            CommandItem { id: "cut", name: "Cut", category: "edit" },
            CommandItem { id: "copy", name: "Copy", category: "edit" },
            CommandItem { id: "paste", name: "Paste", category: "edit" },
            CommandItem { id: "cycle-theme", name: "Cycle Theme (t)", category: "view" },
            CommandItem { id: "toggle-profiler", name: "Toggle Profiler (F12)", category: "view" },
            CommandItem { id: "toggle-search", name: "Toggle Search Panel", category: "view" },
            CommandItem { id: "show-shortcuts", name: "Keyboard Shortcuts", category: "help" },
            CommandItem { id: "about", name: "About Dracon IDE", category: "help" },
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
            should_quit,
            theme,
            area: Rect::new(0, 0, 80, 24),
            menu_bar,
            show_settings: false,
            settings_modal: Modal::new("Settings").with_size(40, 10).with_buttons(vec![("Save", ModalResult::Confirm), ("Cancel", ModalResult::Cancel)]),
            tabs,
            active_tab: 0,
            tab_bar,
            file_tree,
            show_search: false,
            search_input,
            status_bar,
            toasts: Vec::new(),
            tooltip: None,
            tooltip_timer: None,
            context_menu: None,
            profiler: Profiler::new(WidgetId::new(5)),
            show_profiler: false,
            breadcrumbs,
            command_palette,
            cmd_bridge,
            anim_frame: 0,
            last_anim: Instant::now(),
        }
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        let toast = Toast::new(WidgetId::new(100 + self.toasts.len()), msg)
            .with_kind(kind)
            .with_duration(Duration::from_secs(2))
            .with_theme(self.theme);
        self.toasts.push(toast);
    }

    fn cycle_theme(&mut self) {
        let themes = vec![Theme::nord(), Theme::cyberpunk(), Theme::dracula(), Theme::gruvbox_dark(), Theme::tokyo_night()];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];

        self.menu_bar.on_theme_change(&self.theme);
        self.search_input.on_theme_change(&self.theme);
        self.status_bar.on_theme_change(&self.theme);
        self.tab_bar.on_theme_change(&self.theme);
        self.file_tree.on_theme_change(&self.theme);
        self.breadcrumbs.on_theme_change(&self.theme);
        self.toasts.clear();
        self.toast(&format!("Theme: {}", self.theme.name), ToastKind::Info);
    }

    fn active_tab_ref(&self) -> Option<&EditorTab> {
        self.tabs.get(self.active_tab)
    }

    fn active_tab_mut(&mut self) -> Option<&mut EditorTab> {
        self.tabs.get_mut(self.active_tab)
    }

    fn update_status(&mut self) {
        if let Some(tab) = self.active_tab_ref() {
            let lang = tab.path.as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .unwrap_or("Plain");
            self.status_bar = StatusBar::new(WidgetId::new(4))
                .add_segment(StatusSegment::new(if tab.modified { "Modified" } else { "Ready" }).with_fg(if tab.modified { self.theme.warning } else { self.theme.success }))
                .add_segment(StatusSegment::new(&format!("Ln {}, Col {}", tab.cursor_line + 1, tab.cursor_col + 1)).with_fg(self.theme.fg_muted))
                .add_segment(StatusSegment::new(lang).with_fg(self.theme.info))
                .add_segment(StatusSegment::new("UTF-8").with_fg(self.theme.fg_muted));
        }
    }

    fn open_command_palette(&mut self) {
        let (w, h) = (self.area.width, self.area.height);
        let pw = 45.min(w.saturating_sub(4));
        let ph = 18.min(h.saturating_sub(4));
        let ox = (w - pw) / 2;
        let oy = h / 6;
        self.command_palette.set_area(Rect::new(ox, oy, pw, ph));
        self.command_palette.show();
    }

    fn dispatch_palette_command(&mut self, cmd_id: &str) {
        match cmd_id {
            "new-tab" => {
                let new_id = self.tabs.len();
                self.tabs.push(EditorTab::new(&format!("untitled-{}.rs", new_id + 1)));
                self.active_tab = new_id;
                self.sync_tab_bar();
            }
            "open" => self.toast("Open file dialog (mock)", ToastKind::Info),
            "save" => {
                if let Some(tab) = self.active_tab_mut() {
                    tab.modified = false;
                }
                self.update_status();
                self.toast("File saved", ToastKind::Success);
            }
            "close-tab" => {
                if self.tabs.len() > 1 {
                    self.tabs.remove(self.active_tab);
                    self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
                    self.sync_tab_bar();
                }
            }
            "search" | "toggle-search" => {
                self.show_search = !self.show_search;
            }
            "cut" => self.toast("Cut (mock)", ToastKind::Info),
            "copy" => self.toast("Copy (mock)", ToastKind::Info),
            "paste" => self.toast("Paste (mock)", ToastKind::Info),
            "cycle-theme" => self.cycle_theme(),
            "toggle-profiler" => {
                self.show_profiler = !self.show_profiler;
            }
            "show-shortcuts" => {
                self.toast("Shortcuts: Ctrl+P palette, Ctrl+T tab, Ctrl+F search, Ctrl+S save", ToastKind::Info);
            }
            "about" => {
                self.toast("Dracon IDE v28.125 — A terminal-native IDE demo", ToastKind::Info);
            }
            _ => {}
        }
    }

    fn tick(&mut self) {
        if self.last_anim.elapsed() > Duration::from_millis(300) {
            self.anim_frame = (self.anim_frame + 1) % 4;
            self.last_anim = Instant::now();
        }

        // Expire toasts
        self.toasts.retain(|t| !t.is_expired());

        // Expire tooltip
        if let Some(timer) = self.tooltip_timer {
            if timer.elapsed() > Duration::from_secs(2) {
                self.tooltip = None;
                self.tooltip_timer = None;
            }
        }
    }
}

fn build_sample_tree() -> Tree {
    let root = TreeNode {
        label: "workspace".into(),
        expanded: true,
        children: vec![
            TreeNode { label: "Cargo.toml".into(), expanded: false, children: vec![] },
            TreeNode {
                label: "src".into(),
                expanded: true,
                children: vec![
                    TreeNode { label: "main.rs".into(), expanded: false, children: vec![] },
                    TreeNode { label: "lib.rs".into(), expanded: false, children: vec![] },
                    TreeNode { label: "widgets".into(), expanded: false, children: vec![
                        TreeNode { label: "mod.rs".into(), expanded: false, children: vec![] },
                        TreeNode { label: "button.rs".into(), expanded: false, children: vec![] },
                    ]},
                ],
            },
            TreeNode { label: "examples".into(), expanded: false, children: vec![
                TreeNode { label: "demo.rs".into(), expanded: false, children: vec![] },
            ]},
            TreeNode { label: "README.md".into(), expanded: false, children: vec![] },
        ],
    };
    let tree = Tree::new(WidgetId::new(10)).with_root(vec![root]).with_theme(Theme::default());
    tree
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET IMPL
// ═══════════════════════════════════════════════════════════════════════════════

impl Widget for IdeApp {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.area }
    fn set_area(&mut self, area: Rect) { self.area = area; }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = self.theme;

        // Background
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let menu_h = 1u16;
        let tab_h = 1u16;
        let status_h = 1u16;
        let tree_w = 18u16;
        let search_h = if self.show_search { 3u16 } else { 0u16 };

        let content_y = menu_h + tab_h;
        let content_h = area.height.saturating_sub(content_y + status_h + search_h);

        // 1. Menu bar
        let menu_plane = self.menu_bar.render(Rect::new(0, 0, area.width, menu_h));
        self.blit(&mut plane, &menu_plane, 0, 0);

        // 2. Tab bar
        let tab_plane = self.tab_bar.render(Rect::new(0, menu_h, area.width, tab_h));
        self.blit(&mut plane, &tab_plane, 0, menu_h);

        // 3. File tree sidebar
        if content_h > 0 {
            let tree_plane = self.file_tree.render(Rect::new(0, content_y, tree_w, content_h));
            self.blit(&mut plane, &tree_plane, 0, content_y);

            // Tree/editor separator
            for y in content_y..content_y + content_h {
                let idx = (y * area.width + tree_w) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '│';
                    plane.cells[idx].fg = t.outline;
                }
            }
        }

        // 4. Editor area
        let editor_x = tree_w + 1;
        let editor_w = area.width.saturating_sub(editor_x);
        if editor_w > 0 && content_h > 0 {
            // Breadcrumbs
            let bc_plane = self.breadcrumbs.render(Rect::new(editor_x, content_y, editor_w, 1));
            self.blit(&mut plane, &bc_plane, editor_x, content_y);

            // Editor content
            let editor_y = content_y + 1;
            let editor_content_h = content_h.saturating_sub(1);
            if let Some(tab) = self.active_tab_ref() {
                self.render_editor(&mut plane, editor_x, editor_y, editor_w, editor_content_h, tab, t);
            }
        }

        // 5. Search panel
        if self.show_search && search_h > 0 {
            let search_y = area.height.saturating_sub(status_h + search_h);
            // Search background
            for y in search_y..search_y + search_h {
                for x in 0..area.width {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                    }
                }
            }
            // Search label
            draw_text(&mut plane, 2, search_y, "Find:", t.primary, t.surface_elevated, true);
            let search_plane = self.search_input.render(Rect::new(8, search_y, editor_w.saturating_sub(10), 1));
            self.blit(&mut plane, &search_plane, 8, search_y);

            // Search separator
            let sep_y = search_y + search_h - 1;
            for x in 0..area.width {
                let idx = (sep_y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '─';
                    plane.cells[idx].fg = t.outline;
                }
            }
        }

        // 6. Status bar
        let status_y = area.height.saturating_sub(status_h);
        let status_plane = self.status_bar.render(Rect::new(0, status_y, area.width, status_h));
        self.blit(&mut plane, &status_plane, 0, status_y);

        // 7. Toasts
        for (i, toast) in self.toasts.iter().enumerate() {
            let toast_y = status_y.saturating_sub(2 + i as u16);
            let toast_plane = toast.render(Rect::new(2, toast_y, area.width.saturating_sub(4), 1));
            self.blit(&mut plane, &toast_plane, 2, toast_y);
        }

        // 8. Tooltip
        if let Some(ref tooltip) = self.tooltip {
            let tooltip_plane = tooltip.render(Rect::new(area.width / 2, 2, 20, 2));
            self.blit(&mut plane, &tooltip_plane, area.width / 2, 2);
        }

        // 9. Settings modal
        if self.show_settings {
            let modal_plane = self.settings_modal.render(Rect::new(0, 0, area.width, area.height));
            // Draw modal centered
            let mw = 40u16.min(area.width);
            let mh = 10u16.min(area.height);
            let mx = (area.width - mw) / 2;
            let my = (area.height - mh) / 2;
            for y in 0..mh {
                for x in 0..mw {
                    let src_idx = (y * area.width + x) as usize;
                    let dst_idx = ((my + y) * area.width + mx + x) as usize;
                    if src_idx < modal_plane.cells.len() && dst_idx < plane.cells.len() {
                        plane.cells[dst_idx] = modal_plane.cells[src_idx].clone();
                    }
                }
            }
        }

        // 10. Profiler overlay
        if self.show_profiler {
            let prof_plane = self.profiler.render(Rect::new(area.width - 25, menu_h + tab_h, 24, 6));
            self.blit(&mut plane, &prof_plane, area.width - 25, menu_h + tab_h);
        }

        // 11. Command palette overlay
        if self.command_palette.is_visible() {
            let pal_plane = self.command_palette.render(area);
            for y in 0..area.height {
                for x in 0..area.width {
                    let src_idx = (y * area.width + x) as usize;
                    let dst_idx = (y * area.width + x) as usize;
                    if src_idx < pal_plane.cells.len() && dst_idx < plane.cells.len() {
                        let src_cell = &pal_plane.cells[src_idx];
                        if !src_cell.transparent {
                            plane.cells[dst_idx] = src_cell.clone();
                        }
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        // Command palette takes priority when visible
        if self.command_palette.is_visible() {
            let _handled = self.command_palette.handle_key(key);
            // Check if a command was executed via the bridge
            let cmd = self.cmd_bridge.borrow_mut().take();
            if let Some(ref cmd_id) = cmd {
                self.dispatch_palette_command(cmd_id);
            }
            return true;
        }

        // Modal takes priority
        if self.show_settings {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => { self.show_settings = false; return true; }
                _ => return true,
            }
        }

        // Context menu
        if self.context_menu.is_some() {
            if key.code == KeyCode::Esc {
                self.context_menu = None;
                return true;
            }
            return true;
        }

        // Search mode
        if self.show_search {
            match key.code {
                KeyCode::Esc => { self.show_search = false; return true; }
                _ => {
                    let handled = self.search_input.handle_key(key);
                    if handled { return true; }
                }
            }
        }

        // Global shortcuts
        match key.code {
            KeyCode::Char('q') if key.modifiers.is_empty() => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('t') if key.modifiers.is_empty() => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.toast("Open file dialog (mock)", ToastKind::Info);
                true
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(tab) = self.active_tab_mut() {
                    tab.modified = false;
                }
                self.update_status();
                self.toast("File saved", ToastKind::Success);
                true
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.show_search = !self.show_search;
                true
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let new_id = self.tabs.len();
                self.tabs.push(EditorTab::new(&format!("untitled-{}.rs", new_id + 1)));
                self.active_tab = new_id;
                self.sync_tab_bar();
                true
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.tabs.len() > 1 {
                    self.tabs.remove(self.active_tab);
                    self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
                    self.sync_tab_bar();
                }
                true
            }
            KeyCode::F(12) => {
                self.show_profiler = !self.show_profiler;
                true
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_command_palette();
                true
            }
            // Tab navigation
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.active_tab = (self.active_tab + 1) % self.tabs.len();
                self.tab_bar.set_active(self.active_tab);
                self.update_breadcrumbs();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Command palette intercepts all mouse events when visible
        if self.command_palette.is_visible() {
            let handled = self.command_palette.handle_mouse(kind, col, row);
            let cmd = self.cmd_bridge.borrow_mut().take();
            if let Some(ref cmd_id) = cmd {
                self.dispatch_palette_command(cmd_id);
            }
            return handled;
        }

        // Menu bar
        if row == 0 {
            return self.menu_bar.handle_mouse(kind.clone(), col, row);
        }

        // Tab bar
        if row == 1 {
            return self.tab_bar.handle_mouse(kind.clone(), col, row);
        }

        // Context menu on right-click
        if kind == MouseEventKind::Down(MouseButton::Right) {
            self.context_menu = Some(ContextMenu::new_with_id(WidgetId::new(50), vec![
                ("Cut", ContextAction::Cut),
                ("Copy", ContextAction::Copy),
                ("Paste", ContextAction::Paste),
                ("Go to Definition", ContextAction::Open),
            ]));
            return true;
        }

        // Tooltip on hover
        if kind == MouseEventKind::Moved {
            let text = match (col, row) {
                (0..=17, 2..) => "File explorer\nNavigate project files",
                (_, 0) => "Menu bar\nApplication menus",
                (_, 1) => "Tabs\nSwitch between open files",
                (_, _) => return false,
            };
            self.tooltip = Some(Tooltip::new(WidgetId::new(60), text).with_theme(self.theme));
            self.tooltip_timer = Some(Instant::now());
            return true;
        }

        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

impl IdeApp {
    fn blit(&self, dst: &mut Plane, src: &Plane, dx: u16, dy: u16) {
        for (i, cell) in src.cells.iter().enumerate() {
            if cell.transparent { continue; }
            let x = (i % src.width as usize) as u16 + dx;
            let y = (i / src.width as usize) as u16 + dy;
            let idx = (y * dst.width + x) as usize;
            if idx < dst.cells.len() && x < dst.width && y < dst.height {
                dst.cells[idx] = cell.clone();
            }
        }
    }

    fn render_editor(&self, plane: &mut Plane, x: u16, y: u16, _w: u16, h: u16, tab: &EditorTab, t: Theme) {
        let lines: Vec<&str> = tab.content.lines().collect();
        let line_num_width = lines.len().to_string().len().max(2) as u16;

        for (i, line) in lines.iter().enumerate().take(h as usize) {
            let row = y + i as u16;
            if row >= plane.height { break; }

            // Line number
            let num_str = format!("{:>width$} │", i + 1, width = line_num_width as usize - 2);
            for (j, ch) in num_str.chars().enumerate() {
                let idx = (row * plane.width + x + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: ch,
                        fg: t.fg_muted,
                        bg: t.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }

            // Line content with basic syntax highlighting
            let content_x = x + line_num_width;
            for (j, ch) in line.chars().enumerate() {
                let col = content_x + j as u16;
                let idx = (row * plane.width + col) as usize;
                if idx >= plane.cells.len() || col >= plane.width { break; }

                let (fg, style) = if is_keyword(line, j) {
                    (t.primary, Styles::BOLD)
                } else if is_string_literal(line, j) {
                    (t.success, Styles::empty())
                } else if is_comment(line, j) {
                    (t.fg_muted, Styles::empty())
                } else {
                    (t.fg, Styles::empty())
                };

                plane.cells[idx] = Cell {
                    char: ch,
                    fg,
                    bg: t.bg,
                    style,
                    transparent: false,
                    skip: false,
                };
            }
        }

        // Cursor indicator
        let cursor_row = y + tab.cursor_line as u16;
        let cursor_col = x + line_num_width + tab.cursor_col as u16;
        if cursor_row < plane.height && cursor_col < plane.width {
            let idx = (cursor_row * plane.width + cursor_col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.primary_active;
                plane.cells[idx].fg = t.fg_on_accent;
            }
        }
    }

    fn sync_tab_bar(&mut self) {
        let labels: Vec<&str> = self.tabs.iter().map(|t| t.title.as_str()).collect();
        self.tab_bar = TabBar::new_with_id(WidgetId::new(2), labels);
        self.tab_bar.set_active(self.active_tab);
        self.tab_bar.on_theme_change(&self.theme);
    }

    fn update_breadcrumbs(&mut self) {
        if let Some(tab) = self.active_tab_ref() {
            let segments = if let Some(ref path) = tab.path {
                path.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect()
            } else {
                vec!["src".into(), tab.title.clone()]
            };
            self.breadcrumbs = Breadcrumbs::new(segments);
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

fn is_keyword(line: &str, pos: usize) -> bool {
    let keywords = ["fn", "let", "mut", "pub", "use", "struct", "impl", "if", "else", "return", "match", "for", "while", "loop"];
    let before = &line[..pos.min(line.len())];
    let after = &line[pos.min(line.len())..];
    keywords.iter().any(|kw| before.ends_with(kw) || after.starts_with(kw))
}

fn is_string_literal(_line: &str, _pos: usize) -> bool {
    false // Simplified - would need proper parser
}

fn is_comment(line: &str, pos: usize) -> bool {
    line[..pos.min(line.len())].contains("//")
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Dracon IDE — Flagship Example");
    println!("Ctrl+O open | Ctrl+S save | Ctrl+F search | F12 profiler | t theme | q quit");
    std::thread::sleep(Duration::from_millis(500));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::nord();
    let app = Rc::new(RefCell::new(IdeApp::new(should_quit, theme)));
    let app_for_tick = Rc::clone(&app);

    let mut app_widget = App::new()?.title("Dracon IDE").fps(30).theme(theme);

    let router = IdeInputRouter {
        app: Rc::clone(&app),
        id: WidgetId::new(100),
        area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
    };
    app_widget.add_widget(Box::new(router), Rect::new(0, 0, 80, 24));

    app_widget.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
            return;
        }
        app_for_tick.borrow_mut().tick();

        let (w, h) = ctx.compositor().size();
        let mut ide = app_for_tick.borrow_mut();
        if ide.area.width != w || ide.area.height != h {
            ide.area = Rect::new(0, 0, w, h);
        }
        ctx.add_plane(ide.render(Rect::new(0, 0, w, h)));
    }).run(|_| {})
}

struct IdeInputRouter {
    app: Rc<RefCell<IdeApp>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for IdeInputRouter {
    fn id(&self) -> WidgetId { self.id }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.area.get() }
    fn set_area(&mut self, area: Rect) { self.area.set(area); }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { false }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }
    fn render(&self, _area: Rect) -> Plane { Plane::new(0, 0, 0) }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.app.borrow_mut().handle_key(key)
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.app.borrow_mut().handle_mouse(kind, col, row)
    }
}
