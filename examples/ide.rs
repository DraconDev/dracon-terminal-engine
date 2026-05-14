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
//!   Ctrl+P      — command palette
//!   F1          — toggle help overlay
//!   t           — cycle theme
//!   Ctrl+Q      — quit

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, CommandItem, CommandPalette, ContextAction, ContextMenu, MenuBar, MenuEntry,
    MenuItem, Metric, Modal, Profiler, SearchInput, StatusBar, StatusSegment, TabBar, TextEditorAdapter, Toast, ToastKind,
    Tooltip, Tree, TreeNode,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use dracon_terminal_engine::widgets::editor::TextEditor;
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
    adapter: TextEditorAdapter,
}

impl EditorTab {
    fn new(title: &str, adapter: TextEditorAdapter) -> Self {
        Self {
            title: title.to_string(),
            path: None,
            adapter,
        }
    }

    fn with_content(title: &str, content: &str, id: WidgetId) -> Self {
        let editor = TextEditor::with_content(content);
        Self {
            title: title.to_string(),
            path: None,
            adapter: TextEditorAdapter::new(id, editor),
        }
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
    search_submit: Arc<Mutex<String>>,

    // Status
    status_bar: StatusBar,

    // Toasts
    toasts: Vec<Toast>,

    // Tooltip
    tooltip: Option<Tooltip>,
    tooltip_timer: Option<Instant>,

    // Context menu
    context_menu: Option<ContextMenu>,
    context_menu_pos: Option<(u16, u16)>,

    // Profiler overlay
    profiler: Profiler,
    show_profiler: bool,

    // Breadcrumbs
    breadcrumbs: Breadcrumbs,

    // Command palette
    command_palette: CommandPalette,
    cmd_bridge: Rc<RefCell<Option<String>>>,

    // Help overlay
    show_help: bool,

    // Keybindings
    keybindings: KeybindingSet,

    // Animation
    anim_frame: u8,
    last_anim: Instant,
}

impl IdeApp {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let tabs = vec![
            EditorTab::with_content(
                "main.rs",
                "use std::io::{self, Write};\nuse std::process;\n\nfn main() -> io::Result<()> {\n    print!(\"Enter your name: \");\n    io::stdout().flush()?;\n\n    let mut name = String::new();\n    io::stdin().read_line(&mut name)?;\n\n    let name = name.trim();\n    if name.is_empty() {\n        eprintln!(\"Error: name cannot be empty\");\n        process::exit(1);\n    }\n\n    println!(\"Hello, {}! Welcome to Dracon.\", name);\n    Ok(())\n}\n",
                WidgetId::new(200),
            ),
            EditorTab::with_content(
                "lib.rs",
                "/// A simple greeting module\npub fn greet(name: &str) -> String {\n    format!(\"Hello, {}!\", name)\n}\n\n/// Calculate the factorial of n\npub fn factorial(n: u64) -> u64 {\n    match n {\n        0 | 1 => 1,\n        n => n * factorial(n - 1),\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_greet() {\n        assert_eq!(greet(\"World\"), \"Hello, World!\");\n    }\n\n    #[test]\n    fn test_factorial() {\n        assert_eq!(factorial(5), 120);\n    }\n}\n",
                WidgetId::new(201),
            ),
        ];

        let tab_titles = vec!["main.rs", "lib.rs"];
        let tab_bar = TabBar::new_with_id(WidgetId::new(2), tab_titles);

        let file_tree = build_sample_tree(theme.clone());

        let menu_bar = MenuBar::new(WidgetId::new(1))
            .with_theme(theme.clone())
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
                MenuEntry::new("Theme").add_item(MenuItem::new("Cycle (t)")),
                MenuEntry::new("Help")
                    .add_item(MenuItem::new("Shortcuts"))
                    .add_item(MenuItem::new("About")),
            ]);

        let search_submit = Arc::new(Mutex::new(String::new()));
        let search_submit_cb = search_submit.clone();
        let search_input = SearchInput::new(WidgetId::new(3))
            .with_theme(theme.clone())
            .on_submit(move |query| {
                *search_submit_cb.lock().unwrap() = query.to_string();
            });

        let keybindings = KeybindingSet::from_config(&resolve_keybindings());

        let status_bar = StatusBar::new(WidgetId::new(4))
            .add_segment(StatusSegment::new("Ready").with_fg(theme.success))
            .add_segment(StatusSegment::new("Ln 1, Col 1").with_fg(theme.fg_muted))
            .add_segment(StatusSegment::new("Rust").with_fg(theme.info))
            .add_segment(StatusSegment::new("UTF-8").with_fg(theme.fg_muted))
            .add_segment(
                StatusSegment::new(&keybindings.format_hint(&[
                    (actions::THEME, "theme"),
                    (actions::HELP, "help"),
                    (actions::BACK, "dismiss"),
                    (actions::QUIT, "quit"),
                ])).with_fg(theme.fg_muted),
            );

        let breadcrumbs =
            Breadcrumbs::new(vec!["workspace".into(), "src".into(), "main.rs".into()]);

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
                id: "save-all",
                name: "Save All",
                category: "file",
            },
            CommandItem {
                id: "close-tab",
                name: "Close Tab",
                category: "file",
            },
            CommandItem {
                id: "search",
                name: "Search (Ctrl+F)",
                category: "edit",
            },
            CommandItem {
                id: "replace",
                name: "Find and Replace",
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
                name: "Cycle Theme (t)",
                category: "view",
            },
            CommandItem {
                id: "toggle-profiler",
                name: "Toggle Profiler (F12)",
                category: "view",
            },
            CommandItem {
                id: "toggle-search",
                name: "Toggle Search Panel",
                category: "view",
            },
            CommandItem {
                id: "show-shortcuts",
                name: "Keyboard Shortcuts",
                category: "help",
            },
            CommandItem {
                id: "about",
                name: "About Dracon IDE",
                category: "help",
            },
        ];

        let cmd_bridge: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let cmd_bridge_clone = cmd_bridge.clone();
        let command_palette = CommandPalette::new(palette_commands)
            .with_size(45, 18)
            .with_theme(theme.clone())
            .on_execute(move |cmd_id| {
                *cmd_bridge_clone.borrow_mut() = Some(cmd_id.to_string());
            });

        let mut app = Self {
            should_quit,
            theme,
            area: Rect::new(0, 0, 80, 24),
            menu_bar,
            show_settings: false,
            settings_modal: Modal::new("Settings").with_size(40, 10).with_buttons(vec![
                ("Save", ModalResult::Confirm),
                ("Cancel", ModalResult::Cancel),
            ]),
            tabs,
            active_tab: 0,
            tab_bar,
            file_tree,
            show_search: false,
            search_input,
            search_submit,
            status_bar,
            toasts: Vec::new(),
            tooltip: None,
            tooltip_timer: None,
            context_menu: None,
            context_menu_pos: None,
            profiler: Profiler::new(WidgetId::new(5)),
            show_profiler: false,
            show_help: false,
            breadcrumbs,
            command_palette,
            cmd_bridge,
            keybindings,
            anim_frame: 0,
            last_anim: Instant::now(),
        };

        for tab in &mut app.tabs {
            tab.adapter.on_theme_change(&app.theme);
        }

        app
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        let toast = Toast::new(WidgetId::new(100 + self.toasts.len()), msg)
            .with_kind(kind)
            .with_duration(Duration::from_secs(2))
            .with_theme(self.theme.clone());
        self.toasts.push(toast);
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.menu_bar.on_theme_change(&self.theme);
        self.search_input.on_theme_change(&self.theme);
        self.status_bar.on_theme_change(&self.theme);
        self.tab_bar.on_theme_change(&self.theme);
        self.file_tree.on_theme_change(&self.theme);
        self.breadcrumbs.on_theme_change(&self.theme);
        self.command_palette.on_theme_change(&self.theme);
        self.profiler.on_theme_change(&self.theme);
        for tab in &mut self.tabs {
            tab.adapter.on_theme_change(&self.theme);
        }
        self.toasts.clear();
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();

        self.menu_bar.on_theme_change(&self.theme);
        self.search_input.on_theme_change(&self.theme);
        self.status_bar.on_theme_change(&self.theme);
        self.tab_bar.on_theme_change(&self.theme);
        self.file_tree.on_theme_change(&self.theme);
        self.breadcrumbs.on_theme_change(&self.theme);
        self.command_palette.on_theme_change(&self.theme);
        self.profiler.on_theme_change(&self.theme);
        for tab in &mut self.tabs {
            tab.adapter.on_theme_change(&self.theme);
        }
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
            let editor = tab.adapter.editor();
            let lang = tab
                .path
                .as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .unwrap_or("Plain");
            self.status_bar = StatusBar::new(WidgetId::new(4))
                .add_segment(
                    StatusSegment::new(if editor.modified {
                        "● Modified"
                    } else {
                        "✓ Ready"
                    })
                    .with_fg(if editor.modified {
                        self.theme.warning
                    } else {
                        self.theme.success
                    }),
                )
                .add_segment(
                    StatusSegment::new(&format!(
                        "Ln {}, Col {}",
                        editor.cursor_row + 1,
                        editor.cursor_col + 1
                    ))
                    .with_fg(self.theme.fg_muted),
                )
                .add_segment(StatusSegment::new("󱘫 ").with_fg(self.theme.info))
                .add_segment(StatusSegment::new(lang).with_fg(self.theme.info))
                .add_segment(StatusSegment::new("UTF-8").with_fg(self.theme.fg_muted))
                .add_segment(
                    StatusSegment::new(&self.keybindings.format_hint(&[
                        (actions::THEME, "theme"),
                        (actions::HELP, "help"),
                        (actions::BACK, "dismiss"),
                        (actions::QUIT, "quit"),
                    ])).with_fg(self.theme.fg_muted),
                );
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
                let mut adapter = TextEditorAdapter::new(
                    WidgetId::new(200 + new_id),
                    TextEditor::default(),
                );
                adapter.on_theme_change(&self.theme);
                self.tabs
                    .push(EditorTab::new(&format!("untitled-{}.rs", new_id + 1), adapter));
                self.active_tab = new_id;
                self.sync_tab_bar();
            }
            "open" => self.toast("Open file dialog (mock)", ToastKind::Info),
            "save" => {
                if let Some(tab) = self.active_tab_mut() {
                    tab.adapter.editor_mut().modified = false;
                }
                self.update_status();
                self.toast("File saved", ToastKind::Success);
            }
            "close-tab" if self.tabs.len() > 1 => {
                self.tabs.remove(self.active_tab);
                self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
                self.sync_tab_bar();
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
                self.show_help = true;
            }
            "about" => {
                self.toast(
                    "Dracon IDE v28.125 — A terminal-native IDE demo",
                    ToastKind::Info,
                );
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

        // Update profiler metrics
        let frame = self.anim_frame as u64 + 1;
        let variable = ((frame as f64 / 10.0).sin() * 3.0 + 8.0) as u64;
        self.profiler.set_metrics(vec![
            Metric {
                name: "render".to_string(),
                value: Duration::from_micros(600 + (frame % 150) * 4),
                call_count: frame,
            },
            Metric {
                name: "layout".to_string(),
                value: Duration::from_micros(200 + (frame % 80) * 2),
                call_count: frame,
            },
            Metric {
                name: "input".to_string(),
                value: Duration::from_micros(80 + (frame % 40)),
                call_count: frame,
            },
            Metric {
                name: "composite".to_string(),
                value: Duration::from_micros(400 + (frame % 100) * 3),
                call_count: frame,
            },
            Metric {
                name: "memory".to_string(),
                value: Duration::from_millis(variable),
                call_count: 1,
            },
        ]);
    }
}

fn file_icon(name: &str) -> &'static str {
    if name.ends_with(".rs") {
        " "
    } else if name.ends_with(".toml") {
        " "
    } else if name.ends_with(".md") {
        " "
    } else if name.ends_with(".json") || name.ends_with(".yaml") || name.ends_with(".yml") {
        " "
    } else if name.ends_with(".js") || name.ends_with(".ts") {
        " "
    } else if name.ends_with(".py") {
        " "
    } else if name.ends_with(".sh") || name.ends_with(".bash") {
        " "
    } else if name.ends_with(".html") || name.ends_with(".css") {
        " "
    } else if name.ends_with(".gitignore") || name.ends_with(".lock") {
        "﬍ "
    } else {
        " "
    }
}

fn build_sample_tree(theme: Theme) -> Tree {
    let root = TreeNode {
        label: "󰉋 workspace".into(),
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
                    TreeNode {
                        label: "󰉋 widgets".into(),
                        expanded: false,
                        children: vec![
                            TreeNode {
                                label: format!("{}mod.rs", file_icon("mod.rs")),
                                expanded: false,
                                children: vec![],
                            },
                            TreeNode {
                                label: format!("{}button.rs", file_icon("button.rs")),
                                expanded: false,
                                children: vec![],
                            },
                        ],
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

    Tree::new(WidgetId::new(10))
        .with_root(vec![root])
        .with_theme(theme)
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET IMPL
// ═══════════════════════════════════════════════════════════════════════════════

impl Widget for IdeApp {
    fn id(&self) -> WidgetId {
        WidgetId::new(0)
    }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect {
        self.area
    }
    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        let t = &self.theme;

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
            let tree_plane = self
                .file_tree
                .render(Rect::new(0, content_y, tree_w, content_h));
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
            // Rounded border around editor
            draw_rounded_border(&mut plane, editor_x, content_y, editor_w, content_h, t);

            // Breadcrumbs
            let bc_plane = self.breadcrumbs.render(Rect::new(
                editor_x + 1,
                content_y,
                editor_w.saturating_sub(2),
                1,
            ));
            self.blit(&mut plane, &bc_plane, editor_x + 1, content_y);

            // Editor content via TextEditorAdapter
            let editor_y = content_y + 1;
            let editor_content_h = content_h.saturating_sub(1);
            let editor_content_w = editor_w.saturating_sub(2);
            if let Some(tab) = self.active_tab_ref() {
                if editor_content_w > 0 && editor_content_h > 0 {
                    let editor_area = Rect::new(editor_x + 1, editor_y, editor_content_w, editor_content_h);
                    let editor_plane = tab.adapter.render(editor_area);
                    self.blit(&mut plane, &editor_plane, editor_x + 1, editor_y);
                }
            } else {
                // Empty state - no tabs open
                let empty_msg = " 󰈙 No file open ";
                let empty_y = editor_y + editor_content_h / 2;
                let empty_x =
                    editor_x + 1 + (editor_w.saturating_sub(2) - empty_msg.len() as u16) / 2;
                draw_text(
                    &mut plane, empty_x, empty_y, empty_msg, t.fg_muted, t.bg, false,
                );
                let hint_msg = "Press Ctrl+O to open a file or Ctrl+T for a new tab";
                let hint_y = empty_y + 2;
                let hint_x =
                    editor_x + 1 + (editor_w.saturating_sub(2) - hint_msg.len() as u16) / 2;
                draw_text(
                    &mut plane,
                    hint_x,
                    hint_y,
                    hint_msg,
                    t.fg_subtle,
                    t.bg,
                    false,
                );
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
            draw_text(
                &mut plane,
                2,
                search_y,
                "Find:",
                t.primary,
                t.surface_elevated,
                true,
            );
            let search_plane =
                self.search_input
                    .render(Rect::new(8, search_y, editor_w.saturating_sub(10), 1));
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
        let status_plane = self
            .status_bar
            .render(Rect::new(0, status_y, area.width, status_h));
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
            let modal_plane = self
                .settings_modal
                .render(Rect::new(0, 0, area.width, area.height));
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
                        plane.cells[dst_idx] = modal_plane.cells[src_idx];
                    }
                }
            }
        }

        // 10. Profiler overlay
        if self.show_profiler {
            let prof_plane =
                self.profiler
                    .render(Rect::new(area.width - 25, menu_h + tab_h, 24, 6));
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
                            plane.cells[dst_idx] = *src_cell;
                        }
                    }
                }
            }
        }

        // 12. Help overlay
        if self.show_help {
            self.render_help_overlay(&mut plane);
        }

        // 13. Context menu
        if let Some(ref cm) = self.context_menu {
            let menu_plane = cm.render(area);
            self.blit(&mut plane, &menu_plane, menu_plane.x, menu_plane.y);
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
            // Check if a command was executed via the bridge
            let cmd = self.cmd_bridge.borrow_mut().take();
            if let Some(ref cmd_id) = cmd {
                self.dispatch_palette_command(cmd_id);
            }
            return true;
        }

        // Modal takes priority
        if self.show_settings {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::QUIT, &key)
            {
                self.show_settings = false;
                return true;
            }
            return true;
        }

        // Context menu
        if self.context_menu.is_some() {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::CANCEL, &key)
                || self.keybindings.matches(actions::DISMISS, &key)
            {
                self.context_menu = None;
                self.context_menu_pos = None;
                return true;
            }
            return true;
        }

        // Help overlay
        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::CANCEL, &key)
                || self.keybindings.matches(actions::DISMISS, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
                return true;
            }
            return true;
        }

        // Search mode
        if self.show_search {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::CANCEL, &key)
                || self.keybindings.matches(actions::DISMISS, &key)
            {
                self.show_search = false;
                return true;
            }
            let query_before = self.search_submit.lock().unwrap().clone();
            let handled = self.search_input.handle_key(key);
            if handled {
                let query_after = self.search_submit.lock().unwrap().clone();
                if !query_after.is_empty() && query_after != query_before {
                    if let Some(tab) = self.active_tab_ref() {
                        tab.adapter.editor_mut().set_filter(&query_after);
                    }
                }
                return true;
            }
        }

        // Global shortcuts
        if self.keybindings.matches(actions::QUIT, &key) {
            self.should_quit.store(true, Ordering::SeqCst);
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            return true;
        }
        if self.keybindings.matches(actions::SAVE, &key) {
            if let Some(tab) = self.active_tab_mut() {
                tab.adapter.editor_mut().modified = false;
            }
            self.update_status();
            self.toast("File saved", ToastKind::Success);
            return true;
        }
        if self.keybindings.matches(actions::SEARCH, &key) {
            self.show_search = !self.show_search;
            return true;
        }
        if self.keybindings.matches(actions::NEW_TAB, &key) {
            let new_id = self.tabs.len();
            let mut adapter = TextEditorAdapter::new(
                WidgetId::new(200 + new_id),
                TextEditor::default(),
            );
            adapter.on_theme_change(&self.theme);
            self.tabs
                .push(EditorTab::new(&format!("untitled-{}.rs", new_id + 1), adapter));
            self.active_tab = new_id;
            self.sync_tab_bar();
            return true;
        }
        if self.keybindings.matches(actions::CLOSE_TAB, &key) {
            if self.tabs.len() > 1 {
                self.tabs.remove(self.active_tab);
                self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
                self.sync_tab_bar();
            }
            return true;
        }

        match key.code {
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.toast("Open file dialog (mock)", ToastKind::Info);
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
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.active_tab = (self.active_tab + 1) % self.tabs.len();
                self.tab_bar.set_active(self.active_tab);
                self.update_breadcrumbs();
                true
            }
            _ => {
                if self.tabs.get(self.active_tab).is_some() {
                    let menu_h = 1u16;
                    let tab_h = 1u16;
                    let tree_w = 18u16;
                    let search_h = if self.show_search { 3u16 } else { 0u16 };
                    let status_h = 1u16;
                    let content_y = menu_h + tab_h;
                    let content_h = self.area.height.saturating_sub(content_y + status_h + search_h);
                    let editor_x = tree_w + 1;
                    let editor_content_h = content_h.saturating_sub(1);
                    let editor_content_w = self.area.width.saturating_sub(editor_x + 2);
                    let editor_area = Rect::new(editor_x + 1, content_y + 1, editor_content_w, editor_content_h);
                    let tab = &mut self.tabs[self.active_tab];
                    tab.adapter.set_area(editor_area);
                    let handled = tab.adapter.handle_key(key);
                    if handled {
                        self.update_status();
                        self.sync_tab_bar();
                    }
                    handled
                } else {
                    false
                }
            }
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

        // Context menu intercepts when visible
        if let Some(ref mut cm) = self.context_menu {
            if kind == MouseEventKind::Down(MouseButton::Left) {
                if cm.handle_mouse(kind, col, row) {
                    if let Some((_, anchor_y)) = self.context_menu_pos {
                        let idx = (row - anchor_y) as usize;
                        match idx {
                            0 => {
                                let new_id = self.tabs.len();
                                let mut adapter = TextEditorAdapter::new(
                                    WidgetId::new(200 + new_id),
                                    TextEditor::default(),
                                );
                                adapter.on_theme_change(&self.theme);
                                self.tabs.push(EditorTab::new(&format!("untitled-{}.rs", new_id + 1), adapter));
                                self.active_tab = new_id;
                                self.sync_tab_bar();
                                self.update_breadcrumbs();
                                self.update_status();
                            }
                            1 if self.tabs.len() > 1 => {
                                self.tabs.remove(self.active_tab);
                                self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
                                self.sync_tab_bar();
                                self.update_breadcrumbs();
                                self.update_status();
                            }
                            2 => {
                                if let Some(tab) = self.active_tab_mut() {
                                    tab.adapter.editor_mut().modified = false;
                                }
                                self.update_status();
                                self.toast("File saved", ToastKind::Success);
                            }
                            3 => self.toast("Cut (mock)", ToastKind::Info),
                            4 => self.toast("Copy (mock)", ToastKind::Info),
                            5 => self.toast("Paste (mock)", ToastKind::Info),
                            _ => {}
                        }
                    }
                    self.context_menu = None;
                    self.context_menu_pos = None;
                    return true;
                } else {
                    self.context_menu = None;
                    self.context_menu_pos = None;
                    return true;
                }
            }
            if kind == MouseEventKind::Down(MouseButton::Right) {
                self.context_menu = None;
                self.context_menu_pos = None;
            } else {
                return true;
            }
        }

        // Menu bar
        if row == 0 {
            return self.menu_bar.handle_mouse(kind, col, row);
        }

        // Tab bar
        if row == 1 {
            return self.tab_bar.handle_mouse(kind, col, row);
        }

        // Context menu on right-click
        if kind == MouseEventKind::Down(MouseButton::Right) {
            self.context_menu = Some(
                ContextMenu::new_with_id(
                    WidgetId::new(50),
                    vec![
                        ("New Tab", ContextAction::Open),
                        ("Close Tab", ContextAction::Delete),
                        ("Save", ContextAction::Edit),
                        ("Cut", ContextAction::Cut),
                        ("Copy", ContextAction::Copy),
                        ("Paste", ContextAction::Paste),
                    ],
                )
                .with_anchor(col, row)
                .with_theme(self.theme.clone()),
            );
            self.context_menu_pos = Some((col, row));
            return true;
        }

        // Editor mouse events
        {
            let menu_h = 1u16;
            let tab_h = 1u16;
            let tree_w = 18u16;
            let content_y = menu_h + tab_h;
            let editor_x = tree_w + 1;
            let editor_w = self.area.width.saturating_sub(editor_x);
            let status_h = 1u16;
            let search_h = if self.show_search { 3u16 } else { 0u16 };
            let content_h = self.area.height.saturating_sub(content_y + status_h + search_h);
            let has_tab = self.tabs.get(self.active_tab).is_some();

            if has_tab && editor_w > 0 && content_h > 1 {
                let text_y = content_y + 1;
                let text_h = content_h.saturating_sub(1);
                let text_w = editor_w.saturating_sub(2);
                if text_w > 0 && text_h > 0
                    && col >= editor_x + 1 && col < editor_x + 1 + text_w
                    && row >= text_y && row < text_y + text_h
                {
                    let rel_col = col - (editor_x + 1);
                    let rel_row = row - text_y;
                    let handled = self.tabs[self.active_tab].adapter.handle_mouse(kind, rel_col, rel_row);
                    if handled {
                        self.update_status();
                    }
                    return handled;
                }
            }
        }

        // Tooltip on hover
        if kind == MouseEventKind::Moved {
            let text = match (col, row) {
                (0..=17, 2..) => "File explorer\nNavigate project files",
                (_, 0) => "Menu bar\nApplication menus",
                (_, 1) => "Tabs\nSwitch between open files",
                (_, _) => return false,
            };
            self.tooltip = Some(Tooltip::new(WidgetId::new(60), text).with_theme(self.theme.clone()));
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
            if cell.transparent {
                continue;
            }
            let x = (i % src.width as usize) as u16 + dx;
            let y = (i / src.width as usize) as u16 + dy;
            let idx = (y * dst.width + x) as usize;
            if idx < dst.cells.len() && x < dst.width && y < dst.height {
                dst.cells[idx] = *cell;
            }
        }
    }

    fn sync_tab_bar(&mut self) {
        let labels: Vec<String> = self
            .tabs
            .iter()
            .map(|t| {
                if t.adapter.editor().modified {
                    format!("{} ×", t.title)
                } else {
                    t.title.clone()
                }
            })
            .collect();
        self.tab_bar = TabBar::new_with_id(
            WidgetId::new(2),
            labels.iter().map(|s| s.as_str()).collect(),
        );
        self.tab_bar.set_active(self.active_tab);
        self.tab_bar.on_theme_change(&self.theme);
    }

    fn update_breadcrumbs(&mut self) {
        if let Some(tab) = self.active_tab_ref() {
            let segments = if let Some(ref path) = tab.path {
                path.components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect()
            } else {
                vec!["src".into(), tab.title.clone()].clone()
            };
            self.breadcrumbs = Breadcrumbs::new(segments);
        }
    }

    fn render_help_overlay(&self, plane: &mut Plane) {
        let t = &self.theme;
        let w = 60.min(plane.width);
        let h = 20.min(plane.height);
        let x = (plane.width - w) / 2;
        let y = (plane.height - h) / 2;

        draw_rounded_box(plane, x, y, w, h, t);

        let title = "Keyboard Shortcuts";
        for (i, ch) in title.chars().enumerate() {
            let idx = ((y + 1) * plane.width + x + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].style = Styles::BOLD;
            }
        }

        let sep_y = y + 2;
        for px in 0..w {
            let idx = (sep_y * plane.width + x + px) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let kb_save = self.keybindings.display(actions::SAVE).unwrap_or("ctrl+s");
        let kb_new_tab = self.keybindings.display(actions::NEW_TAB).unwrap_or("ctrl+t");
        let kb_close_tab = self.keybindings.display(actions::CLOSE_TAB).unwrap_or("ctrl+w");
        let kb_search = self.keybindings.display(actions::SEARCH).unwrap_or("ctrl+f");

        let shortcuts: [(&str, &[(&str, &str)]); 4] = [
            (
                "File",
                &[
                    ("Ctrl+O", "Open file"),
                    (kb_save, "Save file"),
                    (kb_new_tab, "New tab"),
                    (kb_close_tab, "Close tab"),
                ],
            ),
            (
                "Edit",
                &[
                    ("Type", "Insert text"),
                    ("←↑↓→", "Move cursor"),
                    ("Home/End", "Line start/end"),
                    ("Backspace", "Delete char"),
                ],
            ),
            (
                "View",
                &[
                    (kb_search, "Search"),
                    ("F12", "Profiler"),
                    ("Ctrl+P", "Palette"),
                    (self.keybindings.display(actions::THEME).unwrap_or("t"), "Cycle theme"),
                ],
            ),
            ("General", &[
                (self.keybindings.display(actions::HELP).unwrap_or("F1"), "Toggle this help"),
                (self.keybindings.display(actions::BACK).unwrap_or("Esc"), "Dismiss help"),
                (self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q"), "Quit"),
            ]),
        ];

        let mut row = sep_y + 1;
        for (category, items) in shortcuts.iter() {
            let cat_col = x + 2;
            for (i, ch) in category.chars().enumerate() {
                let idx = (row * plane.width + cat_col + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = t.warning;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }

            for (i, (key, desc)) in items.iter().enumerate() {
                let key_col = x + 8;
                let desc_col = x + 20;
                let item_y = row + i as u16;

                for (j, ch) in key.chars().enumerate() {
                    let idx = (item_y * plane.width + key_col + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = t.primary;
                        plane.cells[idx].style = Styles::BOLD;
                    }
                }

                for (j, ch) in desc.chars().enumerate() {
                    let idx = (item_y * plane.width + desc_col + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = t.fg_muted;
                    }
                }
            }
            row += 1 + items.len() as u16 + 1;
        }

        let hint = "Press ? or Esc to close";
        for (i, ch) in hint.chars().enumerate() {
            let idx = ((y + h - 1) * plane.width + x + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].fg = t.fg_muted;
            }
        }
    }
}

fn draw_rounded_border(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: &Theme) {
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
            let is_corner = (row == y || row == y + h - 1) && (col == x || col == x + w - 1);
            if is_border {
                plane.cells[idx].fg = if is_corner { t.primary } else { t.outline };
                if row == y && col == x {
                    plane.cells[idx].char = '╭';
                } else if row == y && col == x + w - 1 {
                    plane.cells[idx].char = '╮';
                } else if row == y + h - 1 && col == x {
                    plane.cells[idx].char = '╰';
                } else if row == y + h - 1 && col == x + w - 1 {
                    plane.cells[idx].char = '╯';
                } else if row == y || row == y + h - 1 {
                    plane.cells[idx].char = '─';
                } else {
                    plane.cells[idx].char = '│';
                }
                plane.cells[idx].transparent = false;
            }
        }
    }
}

fn draw_rounded_box(plane: &mut Plane, x: u16, y: u16, w: u16, h: u16, t: &Theme) {
    if w < 3 || h < 2 {
        return;
    }

    // Fill interior
    for row in (y + 1)..(y + h - 1) {
        for col in (x + 1)..(x + w - 1) {
            let idx = (row * plane.width + col) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].bg = t.surface_elevated;
                plane.cells[idx].fg = t.fg;
                plane.cells[idx].transparent = false;
            }
        }
    }

    // Draw border with rounded corners
    for row in y..(y + h) {
        for col in x..(x + w) {
            let idx = (row * plane.width + col) as usize;
            if idx >= plane.cells.len() {
                continue;
            }
            let is_border = row == y || row == y + h - 1 || col == x || col == x + w - 1;
            let is_corner = (row == y || row == y + h - 1) && (col == x || col == x + w - 1);
            if is_border {
                plane.cells[idx].fg = if is_corner { t.primary } else { t.outline };
                plane.cells[idx].transparent = false;
                if row == y && col == x {
                    plane.cells[idx].char = '╭';
                } else if row == y && col == x + w - 1 {
                    plane.cells[idx].char = '╮';
                } else if row == y + h - 1 && col == x {
                    plane.cells[idx].char = '╰';
                } else if row == y + h - 1 && col == x + w - 1 {
                    plane.cells[idx].char = '╯';
                } else if row == y || row == y + h - 1 {
                    plane.cells[idx].char = '─';
                } else {
                    plane.cells[idx].char = '│';
                }
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

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> std::io::Result<()> {
    println!("Dracon IDE — Flagship Example");
    println!("Ctrl+O open | Ctrl+S save | Ctrl+F search | F12 profiler | t theme | Ctrl+Q quit");
    std::thread::sleep(Duration::from_millis(500));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::from_env_or(Theme::nord());
    let app = Rc::new(RefCell::new(IdeApp::new(should_quit, theme.clone())));
    let app_for_tick = Rc::clone(&app);

    let mut app_widget = App::new()?.title("Dracon IDE").fps(30).theme(theme);

    let router = IdeInputRouter {
        app: Rc::clone(&app),
        id: WidgetId::new(100),
        area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
    };
    app_widget.add_widget(Box::new(router), Rect::new(0, 0, 80, 24));

    app_widget
        .on_tick(move |ctx, _| {
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
        })
        .run(|_| {})
}

struct IdeInputRouter {
    app: Rc<RefCell<IdeApp>>,
    id: WidgetId,
    area: std::cell::Cell<Rect>,
}

impl Widget for IdeInputRouter {
    fn id(&self) -> WidgetId {
        self.id
    }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect {
        self.area.get()
    }
    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        false
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, _area: Rect) -> Plane {
        Plane::new(0, 0, 0)
    }
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.app.borrow_mut().handle_key(key)
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.app.borrow_mut().handle_mouse(kind, col, row)
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.app.borrow_mut().on_theme_change(theme);
    }
    fn current_theme(&self) -> Option<Theme> {
        Some(self.app.borrow().theme.clone())
    }
}
