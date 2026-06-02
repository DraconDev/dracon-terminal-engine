//! Note Editor scene — TextEditorAdapter + StatusBar + Breadcrumbs.
//!
//! A note editor demonstrating the TextEditorAdapter widget with full
//! editing, built-in context menu (right-click), and breadcrumb navigation.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, ContextMenu, ContextMenuItem, Divider, Label, StatusBar, StatusSegment,
    TextEditorAdapter,
};
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use dracon_terminal_engine::widgets::editor::TextEditor;
use ratatui::layout::Rect;
use std::cell::{Cell, RefCell};

pub struct NoteEditorScene {
    theme: Theme,
    keybindings: KeybindingSet,
    breadcrumbs: RefCell<Breadcrumbs>,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    // File I/O
    current_path: RefCell<Option<String>>,
    save_status: RefCell<Option<String>>,
    // Multi-file tabs
    tabs: RefCell<Vec<Tab>>,
    active_tab: Cell<usize>,
    dirty: bool,
}

struct Tab {
    name: String,
    adapter: TextEditorAdapter,
    path: Option<String>,
}

impl Tab {
    fn new(name: &str) -> Self {
        let mut editor = TextEditor::new();
        editor.lines = vec![String::new()];
        let adapter = TextEditorAdapter::new(WidgetId::new(0), editor);
        Self {
            name: name.to_string(),
            adapter,
            path: None,
        }
    }

    fn from_content(name: &str, content: Vec<String>) -> Self {
        let mut editor = TextEditor::new();
        editor.lines = content;
        let adapter = TextEditorAdapter::new(WidgetId::new(0), editor);
        Self {
            name: name.to_string(),
            adapter,
            path: None,
        }
    }
}

impl NoteEditorScene {
    pub fn new(theme: Theme) -> Self {
        let theme_for_closures = theme.clone();
        let make_menu = |t: &Theme| {
            ContextMenu::new_with_id(
                WidgetId::new(1201),
                vec![
                    ContextMenuItem::new("copy", "Copy"),
                    ContextMenuItem::new("cut", "Cut"),
                    ContextMenuItem::new("paste", "Paste"),
                    ContextMenuItem::separator(),
                    ContextMenuItem::new("select_all", "Select All"),
                    ContextMenuItem::separator(),
                    ContextMenuItem::new("undo", "Undo"),
                    ContextMenuItem::new("redo", "Redo"),
                ],
            )
            .with_theme(t.clone())
        };

        let breadcrumbs = Breadcrumbs::new_with_id(
            WidgetId::new(1202),
            vec![
                "home".into(),
                "user".into(),
                "notes".into(),
                "hello.rs".into(),
            ],
        )
        .clickable(true)
        .with_theme(theme.clone());

        let status_bar = StatusBar::new(WidgetId::new(1203))
            .add_segment(StatusSegment::new(
                "Type to edit | Right-click: menu | Ctrl+S: save | Ctrl+T: new tab | F1: help | Esc: back",
            ))
            .add_segment(StatusSegment::new("TextEditorAdapter"))
            .with_theme(theme.clone());

        let make_tab = |id: usize, name: &str, content: Vec<String>| -> Tab {
            let mut ed = TextEditor::new();
            ed.lines = content;
            let adapter = TextEditorAdapter::new(WidgetId::new(id), ed)
                .with_context_menu(make_menu(&theme_for_closures));
            Tab {
                name: name.to_string(),
                adapter,
                path: None,
            }
        };

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            breadcrumbs: RefCell::new(breadcrumbs),
            status_bar: RefCell::new(status_bar),
            show_help: false,
            current_path: RefCell::new(None),
            save_status: RefCell::new(None),
            tabs: RefCell::new(vec![
                make_tab(
                    1200usize,
                    "welcome.rs",
                    vec![
                        "// Welcome to Dracon Note Editor".into(),
                        "// A demo of TextEditorAdapter + ContextMenu".into(),
                        "".into(),
                        "fn main() {".into(),
                        "    println!(\"Hello, world!\");".into(),
                        "}".into(),
                        "".into(),
                        "// Features:".into(),
                        "//   - Multi-file tabs (Ctrl+T to add)".into(),
                        "//   - Save/load via Ctrl+S / Ctrl+O".into(),
                        "//   - Right-click for context menu".into(),
                        "//   - Tab cycles files; Shift+Tab cycles back".into(),
                    ],
                ),
                make_tab(
                    1201usize,
                    "scratch.txt",
                    vec![
                        "Scratch pad — free writing.".into(),
                        "Use Ctrl+S to save this to a file.".into(),
                        "".into(),
                    ],
                ),
            ]),
            active_tab: Cell::new(0),
            dirty: true,
        }
    }
}

impl Scene for NoteEditorScene {
    fn on_enter(&mut self) {
        // Reset editor state when entering the scene
        self.show_help = false;
        self.dirty = true;
    }
    fn on_exit(&mut self) {
        self.show_help = false;
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Title row
        let title = Label::new("Note Editor")
            .with_style(Styles::BOLD)
            .with_theme(t.clone());
        let title_plane = title.render(Rect::new(0, 0, 12, 1));
        blit_to(&mut plane, &title_plane, 1, 0);
        draw_text(
            &mut plane,
            14,
            0,
            "— TextEditorAdapter + Tabs + Save/Load",
            t.fg_muted,
            t.bg,
            false,
        );

        // Tab bar (row 1): show each open file
        let tabs = self.tabs.borrow();
        let active = self.active_tab.get();
        let mut x = 1u16;
        for (i, tab) in tabs.iter().enumerate() {
            let marker = if i == active { "●" } else { "○" };
            let label = format!("{} {}", marker, tab.name);
            let color = if i == active { t.primary } else { t.fg_muted };
            draw_text(&mut plane, x, 1, &label, color, t.bg, true);
            x += label.len() as u16 + 2;
        }
        // Save status indicator
        if let Some(status) = self.save_status.borrow().as_ref() {
            let remaining_w = area.width.saturating_sub(x + 1);
            if remaining_w > status.len() as u16 {
                draw_text(&mut plane, x + 1, 1, status, t.success, t.bg, false);
            }
        }
        drop(tabs);

        // Breadcrumbs (row 2)
        let bc_area = Rect::new(0, 2, area.width, 1);
        self.breadcrumbs.borrow_mut().set_area(bc_area);
        let bc_plane = self.breadcrumbs.borrow().render(bc_area);
        blit_to(&mut plane, &bc_plane, 0, 2);

        // Divider (row 3)
        let div = Divider::new().with_label("Editor").with_theme(t.clone());
        let div_plane = div.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div_plane, 0, 3);

        // Editor (rows 4..height-2) — use the active tab's adapter
        let ed_y = 4;
        let ed_h = area.height.saturating_sub(6);
        let ed_area = Rect::new(0, ed_y, area.width, ed_h);
        let mut tabs = self.tabs.borrow_mut();
        let tab = &mut tabs[active];
        tab.adapter.set_area(ed_area);
        let ed_plane = tab.adapter.render(ed_area);
        blit_to(&mut plane, &ed_plane, 0, ed_y as usize);

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        // Cursor info on status bar
        let (crow, ccol) = (
            tab.adapter.editor().cursor_row,
            tab.adapter.editor().cursor_col,
        );
        drop(tabs);
        let cursor_info = format!("Ln {}, Col {}  ", crow + 1, ccol + 1);
        draw_text(
            &mut plane,
            area.width.saturating_sub(cursor_info.len() as u16 + 2),
            sb_y,
            &cursor_info,
            t.fg_muted,
            t.bg,
            false,
        );

        if self.show_help {
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Note Editor — Help",
                &[
                    ("Type", "Edit text at cursor"),
                    ("Arrows", "Move cursor / scroll"),
                    ("PgUp/Dn", "Page scroll"),
                    ("Home/End", "Line start/end"),
                    ("Right-click", "Context menu (Copy/Cut/Paste)"),
                    ("Ctrl+Z/Y", "Undo / Redo"),
                    ("Ctrl+A", "Select all"),
                    ("Tab", "Insert spaces"),
                    ("Alt+→/←", "Cycle tabs"),
                    ("Ctrl+T", "New tab"),
                    ("Ctrl+W", "Close current tab"),
                    ("Ctrl+S", "Save current tab to /tmp/dracon_<name>"),
                    ("Ctrl+O", "Load file from /tmp/dracon_<name>"),
                    ("F1", "Toggle this help"),
                    (back_key, "Back"),
                ],
            );
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key)
                || self.keybindings.matches(actions::BACK, &key)
            {
                self.show_help = false;
                self.dirty = true;
                return true;
            }
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false;
        }

        // Save: Ctrl+S — write the active tab to /tmp/dracon_<name>
        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            let mut tabs = self.tabs.borrow_mut();
            let active = self.active_tab.get();
            let tab = &mut tabs[active];
            let name = tab.name.clone();
            let content: Vec<String> = tab.adapter.editor().lines.clone();
            let path = std::path::PathBuf::from("/tmp").join(format!("dracon_{}", name));
            match std::fs::write(&path, content.join("\n")) {
                Ok(_) => {
                    tab.path = Some(path.to_string_lossy().to_string());
                    *self.save_status.borrow_mut() = Some(format!("✓ Saved to {}", path.display()));
                }
                Err(e) => {
                    *self.save_status.borrow_mut() = Some(format!("✗ Save failed: {}", e));
                }
            }
            self.dirty = true;
            return true;
        }

        // Load: Ctrl+O — try to read /tmp/dracon_<name>
        if key.code == KeyCode::Char('o') && key.modifiers.contains(KeyModifiers::CONTROL) {
            let tabs = self.tabs.borrow();
            let active = self.active_tab.get();
            let name = tabs[active].name.clone();
            drop(tabs);
            let path = std::path::PathBuf::from("/tmp").join(format!("dracon_{}", name));
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                    let mut tabs = self.tabs.borrow_mut();
                    let tab = &mut tabs[active];
                    tab.adapter.editor_mut().lines = lines;
                    *self.save_status.borrow_mut() = Some(format!("← Loaded {}", path.display()));
                }
                Err(e) => {
                    *self.save_status.borrow_mut() = Some(format!("✗ Load failed: {}", e));
                }
            }
            self.dirty = true;
            return true;
        }

        // New tab: Ctrl+T
        if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
            let n = self.tabs.borrow().len() + 1;
            self.tabs
                .borrow_mut()
                .push(Tab::new(&format!("untitled{}.txt", n)));
            self.active_tab.set(self.tabs.borrow().len() - 1);
            self.dirty = true;
            return true;
        }

        // Close tab: Ctrl+W
        if key.code == KeyCode::Char('w') && key.modifiers.contains(KeyModifiers::CONTROL) {
            let len = self.tabs.borrow().len();
            if len > 1 {
                self.tabs.borrow_mut().remove(self.active_tab.get());
                let new_active = if self.active_tab.get() >= len - 1 {
                    self.active_tab.get().saturating_sub(1)
                } else {
                    self.active_tab.get()
                };
                self.active_tab.set(new_active);
            }
            self.dirty = true;
            return true;
        }

        // Tab cycling: Alt+Right / Alt+Left
        if key.code == KeyCode::Right && key.modifiers.contains(KeyModifiers::ALT) {
            let len = self.tabs.borrow().len();
            self.active_tab.set((self.active_tab.get() + 1) % len);
            self.dirty = true;
            return true;
        }
        if key.code == KeyCode::Left && key.modifiers.contains(KeyModifiers::ALT) {
            let len = self.tabs.borrow().len();
            let cur = self.active_tab.get();
            self.active_tab
                .set(if cur == 0 { len - 1 } else { cur - 1 });
            self.dirty = true;
            return true;
        }

        // Forward all other keys to the active tab's editor
        let active = self.active_tab.get();
        let handled = self.tabs.borrow_mut()[active].adapter.handle_key(key);
        if handled {
            self.dirty = true;
        }
        handled
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Tab clicks (row 1) — switch to the tab whose label is at this column
        if row == 1 {
            if let MouseEventKind::Down(_) = kind {
                let tabs = self.tabs.borrow();
                let mut x = 1u16;
                for (i, tab) in tabs.iter().enumerate() {
                    let label = format!(
                        "{} {}",
                        if i == self.active_tab.get() {
                            "●"
                        } else {
                            "○"
                        },
                        tab.name
                    );
                    if col >= x && col < x + label.len() as u16 {
                        self.active_tab.set(i);
                        self.dirty = true;
                        return true;
                    }
                    x += label.len() as u16 + 2;
                }
            }
        }

        // Editor area (row 4+)
        if row >= 4 {
            let active = self.active_tab.get();
            let tabs = self.tabs.borrow();
            let ed_area = tabs[active].adapter.area();
            drop(tabs);
            if row >= ed_area.y && row < ed_area.y + ed_area.height {
                let rel_col = col.saturating_sub(ed_area.x);
                let rel_row = row.saturating_sub(ed_area.y);
                let handled = self.tabs.borrow_mut()[active]
                    .adapter
                    .handle_mouse(kind, rel_col, rel_row);
                if handled {
                    self.dirty = true;
                }
                return handled;
            }
        }

        // Breadcrumb clicks (row 2)
        if row == 2 {
            if let MouseEventKind::Down(_) = kind {
                self.dirty = true;
                return true;
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        for tab in self.tabs.borrow_mut().iter_mut() {
            tab.adapter.on_theme_change(theme);
        }
        self.breadcrumbs.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str {
        "note_editor"
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
