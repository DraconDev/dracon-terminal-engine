#![allow(dead_code)]
//! IDE Lite scene — CommandPalette + MenuBar demonstration.
//!
//! Shows a miniature IDE-like interface with:
//! - MenuBar at top (File, Edit, View menus)
//! - CommandPalette overlay (Ctrl+P)
//! - Action log showing executed commands
//! - Status bar at bottom
//!
//! Press Ctrl+P to open the command palette, use menus, or click items.

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    CommandItem, CommandPalette, MenuBar, MenuEntry, MenuItem, StatusBar, StatusSegment,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

// ── Action types ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum IdeAction {
    NewFile,
    OpenFile,
    SaveFile,
    CloseTab,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Find,
    ToggleSidebar,
    ToggleMinimap,
    ZoomIn,
    ZoomOut,
    ShowShortcuts,
    About,
}

impl IdeAction {
    fn label(&self) -> &'static str {
        match self {
            Self::NewFile => "New File",
            Self::OpenFile => "Open File",
            Self::SaveFile => "Save File",
            Self::CloseTab => "Close Tab",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::Find => "Find",
            Self::ToggleSidebar => "Toggle Sidebar",
            Self::ToggleMinimap => "Toggle Minimap",
            Self::ZoomIn => "Zoom In",
            Self::ZoomOut => "Zoom Out",
            Self::ShowShortcuts => "Keyboard Shortcuts",
            Self::About => "About Dracon IDE",
        }
    }

    fn category(&self) -> &'static str {
        match self {
            Self::NewFile | Self::OpenFile | Self::SaveFile | Self::CloseTab => "file",
            Self::Undo | Self::Redo | Self::Cut | Self::Copy | Self::Paste | Self::Find => "edit",
            Self::ToggleSidebar | Self::ToggleMinimap | Self::ZoomIn | Self::ZoomOut => "view",
            Self::ShowShortcuts | Self::About => "help",
        }
    }

    fn shortcut(&self) -> &'static str {
        match self {
            Self::NewFile => "Ctrl+N",
            Self::OpenFile => "Ctrl+O",
            Self::SaveFile => "Ctrl+S",
            Self::CloseTab => "Ctrl+W",
            Self::Undo => "Ctrl+Z",
            Self::Redo => "Ctrl+Shift+Z",
            Self::Cut => "Ctrl+X",
            Self::Copy => "Ctrl+C",
            Self::Paste => "Ctrl+V",
            Self::Find => "Ctrl+F",
            Self::ToggleSidebar => "Ctrl+B",
            Self::ToggleMinimap => "",
            Self::ZoomIn => "Ctrl++",
            Self::ZoomOut => "Ctrl+-",
            Self::ShowShortcuts => "F1",
            Self::About => "",
        }
    }
}

// ── Log entry ──────────────────────────────────────────────────────────────

#[derive(Clone)]
struct LogEntry {
    time: Instant,
    action: String,
    category: String,
    shortcut: String,
}

// ── Scene struct ────────────────────────────────────────────────────────────

pub struct CommandPaletteScene {
    theme: Theme,
    keybindings: KeybindingSet,
    menu_bar: RefCell<MenuBar>,
    command_palette: RefCell<CommandPalette>,
    cmd_bridge: Rc<RefCell<Option<String>>>,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    show_sidebar: bool,
    show_minimap: bool,
    zoom_level: u8,
    log: Vec<LogEntry>,
    max_log: usize,
    log_scroll: usize,
    dirty: bool,
}

impl CommandPaletteScene {
    pub fn new(theme: Theme) -> Self {
        let cmd_bridge = Rc::new(RefCell::new(None));
        let cmd_bridge_clone = cmd_bridge.clone();

        // Build command palette items
        let commands = vec![
            CommandItem { id: "new_file", name: "New File", category: "file" },
            CommandItem { id: "open_file", name: "Open File", category: "file" },
            CommandItem { id: "save_file", name: "Save File", category: "file" },
            CommandItem { id: "close_tab", name: "Close Tab", category: "file" },
            CommandItem { id: "undo", name: "Undo", category: "edit" },
            CommandItem { id: "redo", name: "Redo", category: "edit" },
            CommandItem { id: "cut", name: "Cut", category: "edit" },
            CommandItem { id: "copy", name: "Copy", category: "edit" },
            CommandItem { id: "paste", name: "Paste", category: "edit" },
            CommandItem { id: "find", name: "Find in File", category: "edit" },
            CommandItem { id: "toggle_sidebar", name: "Toggle Sidebar", category: "view" },
            CommandItem { id: "toggle_minimap", name: "Toggle Minimap", category: "view" },
            CommandItem { id: "zoom_in", name: "Zoom In", category: "view" },
            CommandItem { id: "zoom_out", name: "Zoom Out", category: "view" },
            CommandItem { id: "shortcuts", name: "Keyboard Shortcuts", category: "help" },
            CommandItem { id: "about", name: "About Dracon IDE", category: "help" },
        ];

        let command_palette = CommandPalette::new(commands)
            .with_size(48, 16)
            .with_theme(theme.clone())
            .on_execute(move |cmd_id| {
                *cmd_bridge_clone.borrow_mut() = Some(cmd_id.to_string());
            });

        // Build menu bar
        let bridge2 = cmd_bridge.clone();
        let bridge3 = cmd_bridge.clone();

        let file_menu = MenuEntry::new("File")
            .add_item(MenuItem::new("New File").with_action({
                let b = bridge2.clone();
                move || { *b.borrow_mut() = Some("new_file".into()); }
            }))
            .add_item(MenuItem::new("Open File").with_action({
                let b = bridge2.clone();
                move || { *b.borrow_mut() = Some("open_file".into()); }
            }))
            .add_item(MenuItem::new("Save File").with_action({
                let b = bridge2.clone();
                move || { *b.borrow_mut() = Some("save_file".into()); }
            }))
            .add_item(MenuItem::new("Close Tab").with_action({
                let b = bridge2;
                move || { *b.borrow_mut() = Some("close_tab".into()); }
            }));

        let edit_menu = MenuEntry::new("Edit")
            .add_item(MenuItem::new("Undo").with_action({
                let b = bridge3.clone();
                move || { *b.borrow_mut() = Some("undo".into()); }
            }))
            .add_item(MenuItem::new("Redo").with_action({
                let b = bridge3.clone();
                move || { *b.borrow_mut() = Some("redo".into()); }
            }))
            .add_item(MenuItem::new("Cut").with_action({
                let b = bridge3.clone();
                move || { *b.borrow_mut() = Some("cut".into()); }
            }))
            .add_item(MenuItem::new("Copy").with_action({
                let b = bridge3.clone();
                move || { *b.borrow_mut() = Some("copy".into()); }
            }))
            .add_item(MenuItem::new("Paste").with_action({
                let b = bridge3;
                move || { *b.borrow_mut() = Some("paste".into()); }
            }));

        let view_menu = MenuEntry::new("View")
            .add_item(MenuItem::new("Toggle Sidebar").with_action({
                let b = cmd_bridge.clone();
                move || { *b.borrow_mut() = Some("toggle_sidebar".into()); }
            }))
            .add_item(MenuItem::new("Toggle Minimap").with_action({
                let b = cmd_bridge.clone();
                move || { *b.borrow_mut() = Some("toggle_minimap".into()); }
            }))
            .add_item(MenuItem::new("Zoom In").with_action({
                let b = cmd_bridge.clone();
                move || { *b.borrow_mut() = Some("zoom_in".into()); }
            }))
            .add_item(MenuItem::new("Zoom Out").with_action({
                let b = cmd_bridge.clone();
                move || { *b.borrow_mut() = Some("zoom_out".into()); }
            }));

        let menu_bar = MenuBar::new(WidgetId::new(200))
            .with_entries(vec![file_menu, edit_menu, view_menu])
            .with_theme(theme.clone());

        let status_bar = StatusBar::new(WidgetId::new(61))
            .add_segment(StatusSegment::new("Ctrl+P: palette | F1: help | Esc: back"))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            menu_bar: RefCell::new(menu_bar),
            command_palette: RefCell::new(command_palette),
            cmd_bridge,
            status_bar: RefCell::new(status_bar),
            show_help: false,
            show_sidebar: true,
            show_minimap: false,
            zoom_level: 3,
            log: Vec::new(),
            max_log: 50,
            log_scroll: 0,
            dirty: true,
        }
    }

    fn log_action(&mut self, id: &str) {
        let (name, cat, shortcut) = match id {
            "new_file" => ("New File", "file", "Ctrl+N"),
            "open_file" => ("Open File", "file", "Ctrl+O"),
            "save_file" => ("Save File", "file", "Ctrl+S"),
            "close_tab" => ("Close Tab", "file", "Ctrl+W"),
            "undo" => ("Undo", "edit", "Ctrl+Z"),
            "redo" => ("Redo", "edit", "Ctrl+Shift+Z"),
            "cut" => ("Cut", "edit", "Ctrl+X"),
            "copy" => ("Copy", "edit", "Ctrl+C"),
            "paste" => ("Paste", "edit", "Ctrl+V"),
            "find" => ("Find", "edit", "Ctrl+F"),
            "toggle_sidebar" => ("Toggle Sidebar", "view", "Ctrl+B"),
            "toggle_minimap" => ("Toggle Minimap", "view", ""),
            "zoom_in" => ("Zoom In", "view", "Ctrl++"),
            "zoom_out" => ("Zoom Out", "view", "Ctrl+-"),
            "shortcuts" => ("Keyboard Shortcuts", "help", "F1"),
            "about" => ("About Dracon IDE", "help", ""),
            _ => (id, "unknown", ""),
        };

        // Apply side effects
        match id {
            "toggle_sidebar" => self.show_sidebar = !self.show_sidebar,
            "toggle_minimap" => self.show_minimap = !self.show_minimap,
            "zoom_in" => self.zoom_level = (self.zoom_level + 1).min(5),
            "zoom_out" => self.zoom_level = self.zoom_level.saturating_sub(1),
            _ => {}
        }

        self.log.push(LogEntry {
            time: Instant::now(),
            action: name.to_string(),
            category: cat.to_string(),
            shortcut: shortcut.to_string(),
        });
        if self.log.len() > self.max_log {
            self.log.remove(0);
        }
        // Auto-scroll to bottom
        self.log_scroll = self.log.len().saturating_sub(1);
        self.dirty = true;
    }

    fn resolve_bridge(&mut self) {
        let id = self.cmd_bridge.borrow_mut().take();
        if let Some(id) = id {
            self.log_action(&id);
        }
    }
}

// ── Scene trait ─────────────────────────────────────────────────────────────

impl Scene for CommandPaletteScene {
    fn on_enter(&mut self) {}

    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // ── Menu bar (row 0) ────────────────────────────────────────
        //
        let menu_plane = self.menu_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &menu_plane, 0, 0);

        // ── Main content area (row 1 to height-2) ────────────────────
        let content_h = area.height.saturating_sub(2);
        let content_y = 1u16;

        // Sidebar (left 18 columns, if visible)
        let sidebar_w: u16 = if self.show_sidebar { 18 } else { 0 };
        if self.show_sidebar && sidebar_w > 0 {
            // Sidebar background
            for y in content_y..content_y + content_h {
                for x in 0..sidebar_w {
                    let idx = (y as usize) * area.width as usize + x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            // Sidebar border
            for y in content_y..content_y + content_h {
                let idx = (y as usize) * area.width as usize + (sidebar_w - 1) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = '│';
                    plane.cells[idx].fg = t.outline;
                    plane.cells[idx].transparent = false;
                }
            }
            // Sidebar items
            let sidebar_items = ["Explorer", "", "  src/", "    main.rs", "    app.rs",
                "    mod.rs", "  Cargo.toml", "  README.md", "",
                "Search", "", "  No results", "", "",
                "Source Control", "", "  0 changes", "  branch: main"];
            for (i, item) in sidebar_items.iter().enumerate() {
                let y = content_y + 1 + i as u16;
                if y >= content_y + content_h { break; }
                for (j, ch) in item.chars().enumerate() {
                    let x = j as u16;
                    if x >= sidebar_w - 1 { break; }
                    let idx = (y as usize) * area.width as usize + x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = if item.starts_with("  ") { t.fg_muted } else { t.primary };
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
        }

        // Editor area (center) — shows action log
        let editor_x = sidebar_w;
        let editor_w = area.width.saturating_sub(sidebar_w).saturating_sub(if self.show_minimap { 12 } else { 0 });

        // Minimap (right 12 columns, if visible)
        if self.show_minimap {
            let mm_x = area.width.saturating_sub(12);
            for y in content_y..content_y + content_h {
                for x in mm_x..area.width {
                    let idx = (y as usize) * area.width as usize + x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
                // Minimap divider
                let div_idx = (y as usize) * area.width as usize + mm_x.saturating_sub(1) as usize;
                if div_idx < plane.cells.len() {
                    plane.cells[div_idx].char = '│';
                    plane.cells[div_idx].fg = t.outline;
                    plane.cells[div_idx].transparent = false;
                }
            }
            // Minimap content (simplified code outline)
            let mm_lines = ["fn main()", "  app.run()", "  ok?", "",
                "struct App", "  theme", "  widgets", "",
                "impl Widget", "  render()", "  handle_key"];
            for (i, line) in mm_lines.iter().enumerate() {
                let y = content_y + 2 + i as u16;
                if y >= content_y + content_h { break; }
                for (j, ch) in line.chars().enumerate() {
                    let x = mm_x + 1 + j as u16;
                    if x >= area.width { break; }
                    let idx = (y as usize) * area.width as usize + x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = ch;
                        plane.cells[idx].fg = t.fg_muted;
                    }
                }
            }
        }

        // Action log header
        draw_text(&mut plane, editor_x + 1, content_y + 1, "Action Log", t.primary, t.bg, false);
        draw_text(&mut plane, editor_x + 14, content_y + 1, &format!("(zoom: {}x)", self.zoom_level), t.fg_muted, t.bg, false);

        // Log entries
        let log_start_y = content_y + 3;
        let visible_lines = (content_h as usize).saturating_sub(4);
        let log_len = self.log.len();

        for i in 0..visible_lines {
            let log_idx = if log_len > visible_lines {
                let scroll_start = log_len.saturating_sub(visible_lines);
                scroll_start + i
            } else {
                i
            };
            if log_idx >= log_len { break; }
            let entry = &self.log[log_idx];
            let y = log_start_y + i as u16;
            if y >= content_y + content_h { break; }

            // Category badge
            let cat_color = match entry.category.as_str() {
                "file" => t.primary,
                "edit" => t.secondary,
                "view" => t.success,
                "help" => t.warning,
                _ => t.fg_muted,
            };
            let cat_label = format!("[{}]", entry.category);
            draw_text(&mut plane, editor_x + 1, y, &cat_label, cat_color, t.bg, false);

            // Action name
            draw_text(&mut plane, editor_x + 8, y, &entry.action, t.fg, t.bg, false);

            // Shortcut
            if !entry.shortcut.is_empty() {
                let sw = area.width.min(editor_x + editor_w);
                let right_x = sw.saturating_sub(entry.shortcut.len() as u16 + 2);
                if right_x > editor_x + 8 {
                    draw_text(&mut plane, right_x, y, &entry.shortcut, t.fg_muted, t.bg, false);
                }
            }
        }

        // Empty state
        if self.log.is_empty() {
            draw_text(&mut plane, editor_x + 2, log_start_y + 2, "Press Ctrl+P to open command palette", t.fg_muted, t.bg, false);
            draw_text(&mut plane, editor_x + 2, log_start_y + 4, "or click menu items above", t.fg_muted, t.bg, false);
        }

        // ── Status bar (last row) ────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self.status_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        // ── Command palette overlay ──────────────────────────────────
        if self.command_palette.borrow().is_visible() {
            let pw = 48u16.min(area.width);
            let ph = 16u16.min(area.height.saturating_sub(2));
            let px = (area.width.saturating_sub(pw)) / 2;
            let py = 2u16;
            //
            let cp_plane = self.command_palette.borrow().render(Rect::new(0, 0, pw, ph));
            blit_to(&mut plane, &cp_plane, px as usize, py as usize);
        }

        // ── Help overlay ─────────────────────────────────────────────
        if self.show_help {
            render_help_overlay(&mut plane, area, &self.theme, "IDE Lite — Help", &[("Ctrl+P", "Open command palette"), ("Ctrl+B", "Toggle sidebar"), ("Up/Dn", "Scroll action log"), ("Click menu", "Execute menu action"), ("Click sidebar", "Open file (demo)"), ("F1", "Toggle this help"), ("Esc", "Back to showcase")]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Help overlay takes priority
        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key) || self.keybindings.matches(actions::BACK, &key) {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true; // Consume all keys while help is shown
        }

        // Command palette takes priority
        if self.command_palette.borrow().is_visible() {
            if self.keybindings.matches(actions::BACK, &key) {
                self.command_palette.borrow_mut().hide();
                self.dirty = true;
                return true;
            }
            let handled = self.command_palette.borrow_mut().handle_key(key);
            self.resolve_bridge();
            self.dirty = true;
            return handled;
        }

        // Menu bar (delegates key handling internally)
        if self.menu_bar.borrow_mut().handle_key(key) {
            self.resolve_bridge();
            self.dirty = true;
            return true;
        }

        // Global shortcuts
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false; // Let showcase pop the scene
        }

        // Ctrl+P → command palette
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
            self.command_palette.borrow_mut().show();
            self.dirty = true;
            return true;
        }

        // Ctrl+B → toggle sidebar
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('b') {
            self.log_action("toggle_sidebar");
            self.dirty = true;
            return true;
        }

        // Menu bar keyboard dispatch
        if self.menu_bar.borrow_mut().handle_key(key) {
            self.resolve_bridge();
            self.dirty = true;
            return true;
        }

        // Log scrolling
        match key.code {
            KeyCode::Up => {
                if self.log_scroll > 0 {
                    self.log_scroll -= 1;
                    self.dirty = true;
                }
                true
            }
            KeyCode::Down => {
                self.log_scroll = (self.log_scroll + 1).min(self.log.len().saturating_sub(1));
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Command palette
        if self.command_palette.borrow().is_visible() {
            return self.command_palette.borrow_mut().handle_mouse(kind, col, row);
        }

        // Menu bar (row 0)
        if row == 0 {
            return self.menu_bar.borrow_mut().handle_mouse(kind, col, row);
        }

        if let MouseEventKind::Down(_) = kind {
            // Click in sidebar area → toggle sidebar visual (for demo)
            let sidebar_w: u16 = if self.show_sidebar { 18 } else { 0 };
            if col < sidebar_w && row >= 2 {
                // Click on sidebar file items — log it
                self.log_action("open_file");
                return true;
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.menu_bar.borrow_mut().on_theme_change(theme);
        self.command_palette.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str { "command_palette" }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

