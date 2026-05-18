//! Navigator scene — Breadcrumbs + MenuBar + Divider + Label.
//!
//! A file explorer-like UI demonstrating breadcrumb navigation, menu bar
//! with dropdowns, labeled dividers, and interactive path traversal.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, Divider, MenuBar, MenuEntry, MenuItem, StatusBar, StatusSegment,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

#[derive(Clone)]
struct FileEntry {
    name: &'static str,
    is_dir: bool,
    size: &'static str,
}

struct NavState {
    path: Vec<String>,
    entries: Vec<FileEntry>,
    selected: usize,
    scroll: usize,
}

impl NavState {
    fn new() -> Self {
        let mut s = Self {
            path: vec!["home".into(), "user".into()],
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
        };
        s.refresh();
        s
    }

    fn refresh(&mut self) {
        let path_str = self.path.join("/");
        self.entries = match path_str.as_str() {
            "home/user" => vec![
                FileEntry { name: "Documents", is_dir: true, size: "—" },
                FileEntry { name: "Downloads", is_dir: true, size: "—" },
                FileEntry { name: "Pictures", is_dir: true, size: "—" },
                FileEntry { name: "Projects", is_dir: true, size: "—" },
                FileEntry { name: ".bashrc", is_dir: false, size: "3.2K" },
                FileEntry { name: ".profile", is_dir: false, size: "807B" },
                FileEntry { name: "notes.txt", is_dir: false, size: "1.4K" },
            ],
            "home/user/Documents" => vec![
                FileEntry { name: "work", is_dir: true, size: "—" },
                FileEntry { name: "personal", is_dir: true, size: "—" },
                FileEntry { name: "resume.pdf", is_dir: false, size: "245K" },
                FileEntry { name: "budget.csv", is_dir: false, size: "12K" },
                FileEntry { name: "meeting-notes.md", is_dir: false, size: "8.3K" },
            ],
            "home/user/Projects" => vec![
                FileEntry { name: "dracon-engine", is_dir: true, size: "—" },
                FileEntry { name: "tiles", is_dir: true, size: "—" },
                FileEntry { name: "scripts", is_dir: true, size: "—" },
            ],
            _ => vec![
                FileEntry { name: "..", is_dir: true, size: "—" },
                FileEntry { name: "(empty)", is_dir: false, size: "0B" },
            ],
        };
        self.selected = 0;
        self.scroll = 0;
    }

    fn enter_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                if entry.name == ".." {
                    self.path.pop();
                } else {
                    self.path.push(entry.name.to_string());
                }
                self.refresh();
            }
        }
    }

    fn go_up(&mut self) {
        if self.path.len() > 1 {
            self.path.pop();
            self.refresh();
        }
    }

    fn navigate_to(&mut self, depth: usize) {
        self.path.truncate(depth.max(1));
        self.refresh();
    }
}

pub struct NavigatorScene {
    theme: Theme,
    keybindings: KeybindingSet,
    nav: RefCell<NavState>,
    breadcrumbs: RefCell<Breadcrumbs>,
    menu_bar: RefCell<MenuBar>,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    dirty: bool,
}

impl NavigatorScene {
    pub fn new(theme: Theme) -> Self {
        let bc_segments = vec!["home".into(), "user".into()];
        let breadcrumbs = Breadcrumbs::new_with_id(WidgetId::new(900), bc_segments)
            .clickable(true)
            .with_theme(theme.clone());

        let menu_bar = MenuBar::new(WidgetId::new(901))
            .with_entries(vec![
                MenuEntry::new("File").add_item(
                    MenuItem::new("New Tab").with_action(|| {})
                ).add_item(
                    MenuItem::new("Close Tab").with_action(|| {})
                ),
                MenuEntry::new("Edit").add_item(
                    MenuItem::new("Copy").with_action(|| {})
                ).add_item(
                    MenuItem::new("Paste").with_action(|| {})
                ),
                MenuEntry::new("View").add_item(
                    MenuItem::new("Toggle Hidden").with_action(|| {})
                ).add_item(
                    MenuItem::new("Refresh").with_action(|| {})
                ),
                MenuEntry::new("Help").add_item(
                    MenuItem::new("About").with_action(|| {})
                ),
            ])
            .with_theme(theme.clone());

        let status_bar = StatusBar::new(WidgetId::new(902))
            .add_segment(StatusSegment::new(
                "Enter: open | Backspace: up | ↑/↓: navigate | F1: help | Esc: back",
            ))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            nav: RefCell::new(NavState::new()),
            breadcrumbs: RefCell::new(breadcrumbs),
            menu_bar: RefCell::new(menu_bar),
            status_bar: RefCell::new(status_bar),
            show_help: false,
            dirty: true,
        }
    }
}

impl Scene for NavigatorScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;
        let nav = self.nav.borrow();

        // ── Menu bar (row 0) ───────────────────────────────────────
        let mb_area = Rect::new(0, 0, area.width, 1);
        self.menu_bar.borrow_mut().set_area(mb_area);
        let mb_plane = self.menu_bar.borrow().render(mb_area);
        blit_to(&mut plane, &mb_plane, 0, 0);

        // ── Breadcrumbs (row 1) ─────────────────────────────────────
        let bc_area = Rect::new(0, 1, area.width, 1);
        self.breadcrumbs.borrow_mut().set_area(bc_area);
        let bc_plane = self.breadcrumbs.borrow().render(bc_area);
        blit_to(&mut plane, &bc_plane, 0, 1);

        // ── Divider (row 2) ────────────────────────────────────────
        let divider = Divider::new()
            .with_label(&format!("{} items", nav.entries.len()))
            .with_theme(t.clone());
        let div_plane = divider.render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &div_plane, 0, 2);

        // ── File list (rows 3..height-2) ──────────────────────────
        let list_h = area.height.saturating_sub(5);
        let max_scroll = nav.entries.len().saturating_sub(list_h as usize);
        let scroll = nav.scroll.min(max_scroll);

        for vi in 0..list_h as usize {
            let idx = scroll + vi;
            let y = 3 + vi as u16;
            if let Some(entry) = nav.entries.get(idx) {
                let is_selected = idx == nav.selected;
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let name_color = if entry.is_dir { t.primary } else { t.fg };
                let bg = if is_selected { t.selection_bg } else { t.bg };
                let fg = if is_selected { t.selection_fg } else { name_color };

                // Row background
                if is_selected {
                    for x in 1..area.width.saturating_sub(1) {
                        let ci = (y as usize) * area.width as usize + x as usize;
                        if ci < plane.cells.len() {
                            plane.cells[ci].bg = bg;
                            plane.cells[ci].transparent = false;
                        }
                    }
                }

                // Icon + name
                draw_text(&mut plane, 2, y, icon, fg, bg, false);
                let name_str = format!("{:<30}", entry.name);
                draw_text(&mut plane, 5, y, &name_str, fg, bg, entry.is_dir && !is_selected);

                // Size (right-aligned)
                let size_x = area.width.saturating_sub(8);
                draw_text(&mut plane, size_x, y, entry.size, t.fg_muted, bg, false);
            }
        }

        // ── Scrollbar indicator ─────────────────────────────────────
        if nav.entries.len() > list_h as usize {
            let sb_x = area.width.saturating_sub(1);
            let thumb_h = ((list_h as f32 / nav.entries.len() as f32) * list_h as f32).max(1.0) as u16;
            let max_offset = (nav.entries.len() - list_h as usize).max(1);
            let thumb_y = 3 + (scroll as f32 / max_offset as f32 * (list_h - thumb_h) as f32) as u16;
            for i in 0..thumb_h {
                let y = thumb_y + i;
                if y >= 3 && y < 3 + list_h {
                    let idx = (y as usize) * area.width as usize + sb_x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '▐';
                        plane.cells[idx].fg = t.primary;
                    }
                }
            }
        }

        // ── Status bar ─────────────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            render_help_overlay(&mut plane, area, t, "Navigator — Help", &[("Up/Dn", "Navigate file list"), ("Enter", "Open directory"), ("Backspace", "Go to parent"), ("F", "Open File menu"), ("Click row", "Select file"), ("Scroll", "Scroll file list"), ("F1", "Toggle this help"), ("Esc", "Back to showcase")]);
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

        match key.code {
            KeyCode::Up => {
                let mut nav = self.nav.borrow_mut();
                if nav.selected > 0 {
                    nav.selected -= 1;
                    if nav.selected < nav.scroll {
                        nav.scroll = nav.selected;
                    }
                }
                self.dirty = true;
                true
            }
            KeyCode::Down => {
                let mut nav = self.nav.borrow_mut();
                if nav.selected < nav.entries.len().saturating_sub(1) {
                    nav.selected += 1;
                }
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                self.nav.borrow_mut().enter_selected();
                // Update breadcrumbs
                let segments = self.nav.borrow().path.clone();
                self.breadcrumbs = RefCell::new(
                    Breadcrumbs::new_with_id(WidgetId::new(900), segments)
                        .clickable(true)
                        .with_theme(self.theme.clone()),
                );
                self.dirty = true;
                true
            }
            KeyCode::Backspace => {
                self.nav.borrow_mut().go_up();
                let segments = self.nav.borrow().path.clone();
                self.breadcrumbs = RefCell::new(
                    Breadcrumbs::new_with_id(WidgetId::new(900), segments)
                        .clickable(true)
                        .with_theme(self.theme.clone()),
                );
                self.dirty = true;
                true
            }
            KeyCode::Char('f') if key.modifiers.is_empty() => {
                self.menu_bar.borrow_mut().handle_key(key);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, row: u16) -> bool {
        // File list clicks (row 3+)
        if row >= 3 {
            if let MouseEventKind::Down(_) = kind {
                let list_y = row - 3;
                let nav = self.nav.borrow();
                let idx = nav.scroll + list_y as usize;
                if idx < nav.entries.len() {
                    drop(nav);
                    self.nav.borrow_mut().selected = idx;
                    self.dirty = true;
                    return true;
                }
            }
            // Double-click (Enter) via Down on already-selected
            if let MouseEventKind::Up(_) = kind {
                // Could handle double-click here
            }
        }

        // Scroll
        if let MouseEventKind::ScrollUp = kind {
            let mut nav = self.nav.borrow_mut();
            if nav.scroll > 0 {
                nav.scroll -= 1;
                self.dirty = true;
            }
            return true;
        }
        if let MouseEventKind::ScrollDown = kind {
            let nav = self.nav.borrow();
            let max_scroll = nav.entries.len().saturating_sub(1);
            drop(nav);
            let mut nav = self.nav.borrow_mut();
            if nav.scroll < max_scroll {
                nav.scroll += 1;
                self.dirty = true;
            }
            return true;
        }

        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.breadcrumbs.borrow_mut().on_theme_change(theme);
        self.menu_bar.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str {
        "navigator"
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

