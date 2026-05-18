//! Note Editor scene — TextEditorAdapter + ContextMenu + StatusBar + Breadcrumbs.
//!
//! A note editor demonstrating the TextEditorAdapter widget with full
//! editing, context menu (right-click), and breadcrumb navigation.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, ContextMenu, ContextMenuItem, Divider, Label, StatusBar, StatusSegment,
    TextEditorAdapter,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use dracon_terminal_engine::widgets::editor::TextEditor;
use ratatui::layout::Rect;
use std::cell::RefCell;

pub struct NoteEditorScene {
    theme: Theme,
    keybindings: KeybindingSet,
    editor: RefCell<TextEditorAdapter>,
    breadcrumbs: RefCell<Breadcrumbs>,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    dirty: bool,
}

impl NoteEditorScene {
    pub fn new(theme: Theme) -> Self {
        let mut editor = TextEditor::new();
        editor.lines = vec![
            "// Welcome to Dracon Note Editor".into(),
            "// A demo of TextEditorAdapter + ContextMenu".into(),
            "".into(),
            "fn main() {".into(),
            "    println!(\"Hello, world!\");".into(),
            "}".into(),
            "".into(),
            "// Features:".into(),
            "//   - Full text editing (cursor, selection, insert)".into(),
            "//   - Right-click for context menu".into(),
            "//   - Scroll with arrow keys / PageUp/Down".into(),
            "//   - Tab inserts spaces".into(),
        ];

        let ctx_menu = ContextMenu::new_with_id(WidgetId::new(1201))
            .add_item(ContextMenuItem::new("copy", "Copy").shortcut("Ctrl+C"))
            .add_item(ContextMenuItem::new("cut", "Cut").shortcut("Ctrl+X"))
            .add_item(ContextMenuItem::new("paste", "Paste").shortcut("Ctrl+V"))
            .add_separator()
            .add_item(ContextMenuItem::new("select_all", "Select All").shortcut("Ctrl+A"))
            .add_separator()
            .add_item(ContextMenuItem::new("undo", "Undo").shortcut("Ctrl+Z"))
            .add_item(ContextMenuItem::new("redo", "Redo").shortcut("Ctrl+Y"))
            .with_theme(theme.clone());

        let adapter = TextEditorAdapter::new(WidgetId::new(1200), editor)
            .with_context_menu(ctx_menu);

        let breadcrumbs = Breadcrumbs::new_with_id(
            WidgetId::new(1202),
            vec!["home".into(), "user".into(), "notes".into(), "hello.rs".into()],
        )
        .clickable(true)
        .with_theme(theme.clone());

        let status_bar = StatusBar::new(WidgetId::new(1203))
            .add_segment(StatusSegment::new(
                "Type to edit | Right-click: menu | F1: help | Esc: back",
            ))
            .add_segment(StatusSegment::new("TextEditorAdapter"))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            editor: RefCell::new(adapter),
            breadcrumbs: RefCell::new(breadcrumbs),
            status_bar: RefCell::new(status_bar),
            show_help: false,
            dirty: true,
        }
    }
}

impl Scene for NoteEditorScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Title
        let title = Label::new("Note Editor")
            .with_style(Styles::BOLD)
            .with_theme(t.clone());
        let title_plane = title.render(Rect::new(0, 0, 12, 1));
        blit_to(&mut plane, &title_plane, 1, 0);
        draw_text(
            &mut plane,
            14,
            0,
            "— TextEditorAdapter + ContextMenu",
            t.fg_muted,
            t.bg,
            false,
        );

        // Breadcrumbs (row 1)
        let bc_area = Rect::new(0, 1, area.width, 1);
        self.breadcrumbs.borrow_mut().set_area(bc_area);
        let bc_plane = self.breadcrumbs.borrow().render(bc_area);
        blit_to(&mut plane, &bc_plane, 0, 1);

        // Divider (row 2)
        let div = Divider::new()
            .with_label("Editor")
            .with_theme(t.clone());
        let div_plane = div.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div_plane, 0, 2);

        // Editor (rows 3..height-2)
        let ed_y = 3;
        let ed_h = area.height.saturating_sub(5);
        let ed_area = Rect::new(0, ed_y, area.width, ed_h);
        self.editor.borrow_mut().set_area(ed_area);
        let ed_plane = self.editor.borrow().render(ed_area);
        blit_to(&mut plane, &ed_plane, 0, ed_y as usize);

        // Context menu overlay (if visible)
        let cm = self.editor.borrow().context_menu.borrow();
        if let Some(ref menu) = *cm {
            if menu.is_visible() {
                let cm_plane = menu.render(Rect::new(0, 0, 20, 10));
                if let Some((ax, ay)) = menu.anchor() {
                    blit_to(&mut plane, &cm_plane, ax as usize, ay as usize);
                }
            }
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self.status_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        // Cursor info
        let ed = self.editor.borrow();
        let (crow, ccol) = (ed.editor().cursor_row, ed.editor().cursor_col);
        drop(ed);
        draw_text(
            &mut plane,
            area.width.saturating_sub(18),
            sb_y,
            &format!("Ln {}, Col {}  ", crow + 1, ccol + 1),
            t.fg_muted,
            t.bg,
            false,
        );

        if self.show_help {
            render_help_overlay(
                &mut plane, area, t,
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
                    ("F1", "Toggle this help"),
                    ("Esc", "Back to showcase"),
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

        // Forward all keys to the editor
        let handled = self.editor.borrow_mut().handle_key(key);
        if handled {
            self.dirty = true;
        }
        handled
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        // Editor area (row 3+)
        if row >= 3 {
            let ed = self.editor.borrow();
            let ed_area = ed.area();
            drop(ed);
            if row >= ed_area.y && row < ed_area.y + ed_area.height {
                let rel_col = col.saturating_sub(ed_area.x);
                let rel_row = row.saturating_sub(ed_area.y);
                let handled = self.editor.borrow_mut().handle_mouse(kind, rel_col, rel_row);
                if handled {
                    self.dirty = true;
                }
                return handled;
            }
        }

        // Breadcrumb clicks (row 1)
        if row == 1 {
            if let MouseEventKind::Down(_) = kind {
                // Click breadcrumb to navigate (demo: just trigger dirty)
                self.dirty = true;
                return true;
            }
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.editor.borrow_mut().on_theme_change(theme);
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
