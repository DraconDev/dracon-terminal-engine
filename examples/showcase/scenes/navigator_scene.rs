//! Embedded Navigator scene for the showcase.
//!
//! A file explorer UI demonstrating breadcrumb navigation, menu bar with
//! dropdowns, and interactive path traversal.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text, draw_text_clipped, render_help_overlay};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, Divider, MenuBar, MenuEntry, MenuItem, StatusBar, StatusSegment,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

const SIDEBAR_W: u16 = 20;
const DIV_X: u16 = SIDEBAR_W + 2;

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
    search_query: String,
    filtered: Vec<usize>,
}

impl NavState {
    fn new() -> Self {
        let mut s = Self {
            path: vec!["home".into(), "user".into()],
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
            search_query: String::new(),
            filtered: Vec::new(),
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
        self.apply_search();
        self.selected = 0;
        self.scroll = 0;
    }

    fn apply_search(&mut self) {
        self.filtered = if self.search_query.is_empty() {
            (0..self.entries.len()).collect()
        } else {
            self.entries.iter().enumerate()
                .filter(|(_, e)| e.name.to_lowercase().contains(&self.search_query.to_lowercase()))
                .map(|(i, _)| i)
                .collect()
        };
    }

    fn enter_selected(&mut self) {
        if let Some(&idx) = self.filtered.get(self.selected) {
            if let Some(entry) = self.entries.get(idx) {
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
    }

    fn go_up(&mut self) {
        if self.path.len() > 1 {
            self.path.pop();
            self.refresh();
        }
    }

    fn set_search(&mut self, query: String) {
        self.search_query = query;
        self.apply_search();
        self.selected = 0;
        self.scroll = 0;
    }

    fn visible_count(&self, total_height: u16) -> usize {
        (total_height as usize).saturating_sub(8)
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

        let status_bar = RefCell::new(
            StatusBar::new(WidgetId::new(902))
                .add_segment(StatusSegment::new(
                    "Enter: open | ↑/↓: navigate | /: search | F1: help | Esc: back",
                ))
                .with_theme(theme.clone())
        );

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            nav: RefCell::new(NavState::new()),
            breadcrumbs: RefCell::new(breadcrumbs),
            menu_bar: RefCell::new(menu_bar),
            status_bar,
            show_help: false,
            dirty: true,
        }
    }

    fn update_breadcrumbs(&self) {
        let segments = self.nav.borrow().path.clone();
        let bc = Breadcrumbs::new_with_id(WidgetId::new(900), segments)
            .clickable(true)
            .with_theme(self.theme.clone());
        *self.breadcrumbs.borrow_mut() = bc;
    }
}

impl Scene for NavigatorScene {
    fn scene_id(&self) -> &str { "navigator" }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;
        let nav = self.nav.borrow();

        // Header
        draw_text(&mut plane, 2, 0, " Quick Launcher ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(&mut plane, area.width.saturating_sub(theme_label.len() as u16 + 2), 0,
                  &theme_label, t.secondary, t.bg, false);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Left sidebar
        self.render_sidebar(&mut plane, area, t);

        // Vertical divider
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y * plane.width + DIV_X) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // Main area
        let main_x = DIV_X + 2;

        // Breadcrumbs
        let bc_area = Rect::new(main_x, 2, area.width.saturating_sub(main_x + 2), 1);
        self.breadcrumbs.borrow_mut().set_area(bc_area);
        let bc_plane = self.breadcrumbs.borrow().render(bc_area);
        blit_to(&mut plane, &bc_plane, main_x as usize, 2);

        // Search indicator
        if !nav.search_query.is_empty() {
            let search_text = format!("Filter: \"{}\" ({} matches)", nav.search_query, nav.filtered.len());
            draw_text(&mut plane, main_x, 3, &search_text, t.info, t.bg, false);
        }

        // Divider with item count
        let divider = Divider::new()
            .with_label(&format!("{} items", nav.filtered.len()))
            .with_theme(t.clone());
        let div_plane = divider.render(Rect::new(0, 0, area.width.saturating_sub(main_x + 2), 1));
        blit_to(&mut plane, &div_plane, main_x as usize, 4);

        // File list
        let list_y = 5;
        let list_h = area.height.saturating_sub(8);
        let max_scroll = nav.filtered.len().saturating_sub((list_y + list_y) as usize);
        let scroll = nav.scroll.min(max_scroll);

        for vi in 0u16..list_h {
            let idx = scroll + vi as usize;
            let y = list_y + vi;
            if let Some(&entry_idx) = nav.filtered.get(idx) {
                if let Some(entry) = nav.entries.get(entry_idx) {
                    let is_selected = idx == nav.selected;
                    let icon = if entry.is_dir { "▶" } else { "•" };
                    let name_color = if entry.is_dir { t.primary } else { t.fg };
                    let bg = if is_selected { t.selection_bg } else { t.bg };
                    let fg = if is_selected { t.selection_fg } else { name_color };

                    if is_selected {
                        for cx in 1..area.width.saturating_sub(main_x + 1) {
                            let ci = (y as usize) * area.width as usize + (main_x + cx) as usize;
                            if ci < plane.cells.len() {
                                plane.cells[ci].bg = bg;
                                plane.cells[ci].transparent = false;
                            }
                        }
                    }

                    draw_text(&mut plane, main_x, y, icon, fg, bg, false);
                    draw_text_clipped(&mut plane, main_x + 2, y, entry.name,
                                     area.width.saturating_sub(10), fg, bg, entry.is_dir && !is_selected);
                    let size_x = area.width.saturating_sub(8);
                    draw_text(&mut plane, size_x, y, entry.size, t.fg_muted, bg, false);
                }
            }
        }

        // Scrollbar
        if nav.filtered.len() > list_h as usize {
            let sb_x = area.width.saturating_sub(1);
            let thumb_h = ((list_h as f32 / nav.filtered.len() as f32) * list_h as f32).max(1.0) as u16;
            let max_offset = (nav.filtered.len() - list_h as usize).max(1);
            let thumb_y = list_y + ((scroll as f32 / max_offset as f32 * (list_h as f32 - thumb_h as f32)) as u16);
            for i in 0..thumb_h {
                let y = thumb_y + i;
                if y >= list_y && y < list_y + list_h {
                    let idx = (y as usize) * area.width as usize + sb_x as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '▐';
                        plane.cells[idx].fg = t.primary;
                    }
                }
            }
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self.status_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(&mut plane, area, t, "Quick Launcher — Help", &[
                ("↑/↓", "Navigate list"),
                ("Enter", "Open directory"),
                ("Backspace", "Go to parent"),
                ("Ctrl+F", "Toggle search"),
                ("/", "Quick filter"),
                ("Esc", "Clear search"),
                ("Click row", "Select file"),
                (help_key, "Toggle this help"),
                (back_key, "Back"),
            ]);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key) || self.keybindings.matches(actions::HELP, &key) {
                self.show_help = false;
                self.dirty = true;
            }
            return true;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            let nav = self.nav.borrow();
            if nav.search_query.is_empty() {
                return false;
            }
            drop(nav);
            self.nav.borrow_mut().set_search(String::new());
            self.dirty = true;
            return true;
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
                if nav.selected < nav.filtered.len().saturating_sub(1) {
                    nav.selected += 1;
                }
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                self.nav.borrow_mut().enter_selected();
                drop(self.nav.borrow());
                self.update_breadcrumbs();
                self.dirty = true;
                true
            }
            KeyCode::Backspace => {
                self.nav.borrow_mut().go_up();
                drop(self.nav.borrow());
                self.update_breadcrumbs();
                self.dirty = true;
                true
            }
            KeyCode::Char('/') => {
                self.nav.borrow_mut().set_search(String::new());
                self.dirty = true;
                true
            }
            KeyCode::Char(c) if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' => {
                let query = {
                    let nav = self.nav.borrow();
                    nav.search_query.clone() + &c.to_string()
                };
                self.nav.borrow_mut().set_search(query);
                self.dirty = true;
                true
            }
            KeyCode::Esc => {
                let has_query = {
                    let nav = self.nav.borrow();
                    !nav.search_query.is_empty()
                };
                if has_query {
                    self.nav.borrow_mut().set_search(String::new());
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('f') if key.modifiers.is_empty() => {
                self.menu_bar.borrow_mut().handle_key(key);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let list_y = 5u16;
        let list_h: usize = 20; // Approximate visible list height
        let cat_w: u16 = SIDEBAR_W;

        // Sidebar clicks
        if col < cat_w && row >= 3 {
            let rel_row = (row - 3) as usize;
            let entry_name: Option<String> = {
                let nav = self.nav.borrow();
                nav.entries.iter()
                    .filter(|e| e.is_dir)
                    .nth(rel_row)
                    .map(|e| e.name.to_string())
            };
            if let Some(name) = entry_name {
                if let MouseEventKind::Down(_) = kind {
                    self.nav.borrow_mut().path.push(name);
                    self.nav.borrow_mut().refresh();
                    drop(self.nav.borrow());
                    self.update_breadcrumbs();
                    self.dirty = true;
                    return true;
                }
            }
        }

        // File list clicks
        if col > cat_w && row >= list_y {
            let rel_row = (row - list_y) as usize;
            let mut nav = self.nav.borrow_mut();
            let idx = nav.scroll + rel_row;
            if idx < nav.filtered.len() {
                if let MouseEventKind::Down(_) = kind {
                    nav.selected = idx;
                    self.dirty = true;
                    return true;
                }
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
            let mut nav = self.nav.borrow_mut();
            let max_scroll = nav.filtered.len().saturating_sub(list_h);
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

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

impl NavigatorScene {
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let sx = 2u16;
        let nav = self.nav.borrow();

        // Quick access title
        draw_text(plane, sx, 2, "Quick Access", t.primary, t.bg, true);

        // Directories list
        let dirs: Vec<_> = nav.entries.iter().filter(|e| e.is_dir).collect();
        let start_y = 3u16;
        for (i, entry) in dirs.iter().enumerate() {
            let y = start_y + i as u16;
            if y >= area.height.saturating_sub(6) { break; }

            let is_selected = nav.selected < nav.filtered.len()
                && nav.filtered.get(nav.selected).copied() == nav.entries.iter().position(|e| e.name == entry.name);

            let bg = if is_selected { t.selection_bg } else { t.surface };
            for cx in 0..SIDEBAR_W {
                let idx = (y * plane.width + sx + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].transparent = false;
                }
            }

            draw_text_clipped(plane, sx + 1, y, &format!(" {}{}", if entry.name == ".." { "↑ " } else { "▶ " }, entry.name),
                             sx + SIDEBAR_W, t.primary, bg, false);
        }

        // Divider
        let div_y = area.height.saturating_sub(6);
        for dx in 0..SIDEBAR_W {
            let idx = (div_y * plane.width + sx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Path info
        let path_y = div_y + 2;
        draw_text(plane, sx, path_y, "Current Path", t.secondary, t.bg, true);
        let path_text = nav.path.join(" / ");
        draw_text_clipped(plane, sx, path_y + 1, &path_text, sx + SIDEBAR_W, t.fg, t.bg, false);

        // Stats
        let stats_y = path_y + 3;
        if stats_y + 3 < area.height.saturating_sub(2) {
            draw_text(plane, sx, stats_y, "Stats", t.secondary, t.bg, true);
            let dirs_count = nav.entries.iter().filter(|e| e.is_dir).count();
            let files_count = nav.entries.iter().filter(|e| !e.is_dir).count();
            draw_text(plane, sx, stats_y + 1, &format!("Directories: {}", dirs_count), t.fg_muted, t.bg, false);
            draw_text(plane, sx, stats_y + 2, &format!("Files: {}", files_count), t.fg_muted, t.bg, false);
        }
    }
}