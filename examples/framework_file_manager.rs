#![allow(missing_docs)]
//! File manager demo — shows List + Breadcrumbs + SplitPane.
//!
//! Keyboard: arrows navigate, Enter opens, Left goes up, t=cycle theme, ?=help.
//! Mouse: click to select, scroll to browse.

use dracon_terminal_engine::compositor::{Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingConfig, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Breadcrumbs, List, SplitPane};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
}

impl std::fmt::Display for FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = if self.is_dir { "󰉋" } else { "󰈔" };
        write!(f, "{} {} ({})", icon, self.name, self.size)
    }
}

fn read_dir(path: &PathBuf) -> Vec<FileEntry> {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let meta = e.metadata().ok();
                    FileEntry {
                        name: e.file_name().to_string_lossy().into_owned(),
                        is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                        size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

struct FileManagerApp {
    id: WidgetId,
    entries: Vec<FileEntry>,
    crumbs: Vec<String>,
    list: List<FileEntry>,
    breadcrumbs: Breadcrumbs,
    selected: usize,
    scroll_offset: usize,
    visible_count: usize,
    theme: Theme,
    area: Rect,
    dirty: bool,
    show_help: bool,
    keybindings: KeybindingSet,
    kb_config: KeybindingConfig,
}

impl FileManagerApp {
    fn new(theme: Theme) -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let crumbs: Vec<String> = current_dir
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        let path = PathBuf::from(crumbs.join("/"));
        let entries = read_dir(&path);
        let list = List::new(entries.clone());
        let breadcrumbs = Breadcrumbs::new(crumbs.clone());
        let kb_config = resolve_keybindings();
        let keybindings = KeybindingSet::from_config(&kb_config);
        Self {
            id: WidgetId::new(0),
            entries,
            crumbs,
            list,
            breadcrumbs,
            selected: 0,
            scroll_offset: 0,
            visible_count: 10,
            theme,
            area: Rect::new(0, 0, 80, 24),
            dirty: true,
            show_help: false,
            keybindings,
            kb_config,
        }
    }

    fn refresh(&mut self) {
        let path = PathBuf::from(self.crumbs.join("/"));
        self.entries = read_dir(&path);
        self.list.set_items(self.entries.clone());
        self.selected = 0;
        self.scroll_offset = 0;
        self.dirty = true;
    }

    fn enter_dir(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                self.crumbs.push(entry.name.clone());
                self.breadcrumbs = Breadcrumbs::new(self.crumbs.clone());
                self.refresh();
            }
        }
    }

    fn go_up(&mut self) {
        if self.crumbs.len() > 1 {
            self.crumbs.pop();
            self.breadcrumbs = Breadcrumbs::new(self.crumbs.clone());
            self.refresh();
        }
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
        self.list.on_theme_change(&self.theme);
        self.breadcrumbs.on_theme_change(&self.theme);
        self.dirty = true;
    }
}

impl Widget for FileManagerApp {
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
        let split = SplitPane::new(Orientation::Vertical).ratio(0.7);
        let (main_rect, _) = split.split(area);
        self.visible_count = (main_rect.height as usize).saturating_sub(2).max(1);
        self.list.set_visible_count(self.visible_count);
        self.dirty = true;
    }
    fn z_index(&self) -> u16 {
        10
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

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        let split = SplitPane::new(Orientation::Vertical).ratio(0.7);
        let (main_rect, side_rect) = split.split(area);

        // Rounded border around entire area
        let bw = area.width;
        let bh = area.height;
        if bw > 0 && bh > 0 {
            let corners = [
                ('╭', 0, 0),
                ('╮', bw - 1, 0),
                ('╰', 0, bh - 1),
                ('╯', bw - 1, bh - 1),
            ];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = t.outline;
                }
            }
            for x in 1..bw.saturating_sub(1) {
                let top = x as usize;
                let bot = ((bh - 1) * area.width + x) as usize;
                if top < plane.cells.len() {
                    plane.cells[top].char = '─';
                    plane.cells[top].fg = t.outline;
                }
                if bot < plane.cells.len() {
                    plane.cells[bot].char = '─';
                    plane.cells[bot].fg = t.outline;
                }
            }
            for y in 1..bh.saturating_sub(1) {
                let left = (y * area.width) as usize;
                let right = (y * area.width + bw - 1) as usize;
                if left < plane.cells.len() {
                    plane.cells[left].char = '│';
                    plane.cells[left].fg = t.outline;
                }
                if right < plane.cells.len() {
                    plane.cells[right].char = '│';
                    plane.cells[right].fg = t.outline;
                }
            }
        }

        // Breadcrumbs at top
        let bc_plane = self.breadcrumbs.render(Rect::new(1, 1, main_rect.width, 1));
        for y in 0..bc_plane.height {
            for x in 0..bc_plane.width {
                let src = (y * bc_plane.width + x) as usize;
                if bc_plane.cells[src].transparent {
                    continue;
                }
                let dst = ((1 + y) * area.width + (1 + x)) as usize;
                if src < bc_plane.cells.len() && dst < plane.cells.len() {
                    plane.cells[dst] = bc_plane.cells[src].clone();
                }
            }
        }

        // File list
        let list_plane = self.list.render(Rect::new(
            1,
            2,
            main_rect.width,
            main_rect.height.saturating_sub(2),
        ));
        for y in 0..list_plane.height {
            for x in 0..list_plane.width {
                let src = (y * list_plane.width + x) as usize;
                if list_plane.cells[src].transparent {
                    continue;
                }
                let dst = ((2 + y) * area.width + (1 + x)) as usize;
                if src < list_plane.cells.len() && dst < plane.cells.len() {
                    plane.cells[dst] = list_plane.cells[src].clone();
                }
            }
        }

        // Info panel (right side)
        for y in 1..area.height.saturating_sub(1) {
            for x in main_rect.width + 1..area.width.saturating_sub(1) {
                let idx = (y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface;
                    plane.cells[idx].fg = t.fg;
                }
            }
        }

        let mut info_y = 2u16;
        let info_x = main_rect.width + 2;
        let info_w = side_rect.width.saturating_sub(3);

        let print_info = |plane: &mut Plane, text: &str, fg: Color, y: &mut u16| {
            for (i, c) in text.chars().take(info_w as usize).enumerate() {
                let idx = (*y * area.width + info_x + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = t.surface;
                }
            }
            *y += 1;
        };

        print_info(&mut plane, "INFORMATION", t.primary, &mut info_y);
        info_y += 1;
        print_info(
            &mut plane,
            &format!("Items: {}", self.entries.len()),
            t.fg_muted,
            &mut info_y,
        );
        if let Some(entry) = self.entries.get(self.selected) {
            info_y += 1;
            print_info(
                &mut plane,
                &format!("Name: {}", entry.name),
                t.fg_on_accent,
                &mut info_y,
            );
            if entry.is_dir {
                print_info(&mut plane, "Type: Directory", t.info, &mut info_y);
            } else {
                let size_str = if entry.size < 1024 {
                    format!("Size: {} B", entry.size)
                } else if entry.size < 1024 * 1024 {
                    format!("Size: {} KB", entry.size / 1024)
                } else if entry.size < 1024 * 1024 * 1024 {
                    format!("Size: {} MB", entry.size / 1024 / 1024)
                } else {
                    format!("Size: {} GB", entry.size / 1024 / 1024 / 1024)
                };
                print_info(&mut plane, &size_str, t.warning, &mut info_y);
            }
        }

        // Scrollbar indicator
        if self.entries.len() > self.visible_count {
            let sb_x = main_rect.width;
            let content_h = main_rect.height.saturating_sub(2);
            let thumb_h = (self.visible_count as f32 / self.entries.len() as f32 * content_h as f32)
                .max(1.0) as u16;
            let thumb_y = (self.scroll_offset as f32
                / self.entries.len().saturating_sub(self.visible_count).max(1) as f32
                * (content_h - thumb_h) as f32) as u16
                + 2;
            for i in 0..thumb_h {
                let y = thumb_y + i;
                if y >= 2 && y < main_rect.height {
                    let idx = (y * area.width + sb_x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = '▐';
                        plane.cells[idx].fg = t.primary;
                    }
                }
            }
        }

        // Status bar
        let status_y = area.height.saturating_sub(2);
        let status_text = format!(
            " {} items | {} selected | t: theme | ?: help | Esc: dismiss | q: quit ",
            self.entries.len(),
            self.selected + 1
        );
        let sx = (area.width.saturating_sub(status_text.len() as u16)) / 2;
        for (i, c) in status_text
            .chars()
            .take(area.width.saturating_sub(2) as usize)
            .enumerate()
        {
            let idx = (status_y * area.width + sx + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = t.primary;
                plane.cells[idx].bg = t.surface;
            }
        }

        // Help overlay
        if self.show_help {
            let hw = 40u16.min(area.width.saturating_sub(4));
            let hh = 12u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;
            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = t.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            let corners = [
                ('╭', hx, hy),
                ('╮', hx + hw - 1, hy),
                ('╰', hx, hy + hh - 1),
                ('╯', hx + hw - 1, hy + hh - 1),
            ];
            for (ch, cx, cy) in corners.iter() {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = *ch;
                    plane.cells[idx].fg = t.outline;
                }
            }
            for x in hx + 1..hx + hw - 1 {
                let top = (hy * area.width + x) as usize;
                let bot = ((hy + hh - 1) * area.width + x) as usize;
                if top < plane.cells.len() {
                    plane.cells[top].char = '─';
                    plane.cells[top].fg = t.outline;
                }
                if bot < plane.cells.len() {
                    plane.cells[bot].char = '─';
                    plane.cells[bot].fg = t.outline;
                }
            }
            for y in hy + 1..hy + hh - 1 {
                let left = (y * area.width + hx) as usize;
                let right = (y * area.width + hx + hw - 1) as usize;
                if left < plane.cells.len() {
                    plane.cells[left].char = '│';
                    plane.cells[left].fg = t.outline;
                }
                if right < plane.cells.len() {
                    plane.cells[right].char = '│';
                    plane.cells[right].fg = t.outline;
                }
            }
            let title = "File Manager Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = t.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }
            let shortcuts = [
                ("↑/↓", "Navigate"),
                ("Enter", "Open directory"),
                ("Left", "Go up"),
                ("t", "Cycle theme"),
                ("?", "Toggle help"),
                ("q", "Quit"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.primary;
                    }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = t.fg;
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Esc if self.show_help => {
                self.show_help = false;
                self.dirty = true;
                true
            }
            KeyCode::Char('t') if key.modifiers.is_empty() => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                self.dirty = true;
                true
            }
            KeyCode::Char('q') => {
                // Handled by app on_input
                false
            }
            KeyCode::Down if self.selected + 1 < self.entries.len() => {
                self.selected += 1;
                self.list.scroll_to(self.selected);
                self.scroll_offset = self
                    .list
                    .selected_index()
                    .saturating_sub(self.visible_count);
                self.dirty = true;
                true
            }
            KeyCode::Up if self.selected > 0 => {
                self.selected -= 1;
                self.list.scroll_to(self.selected);
                self.scroll_offset = self.list.selected_index();
                self.dirty = true;
                true
            }
            KeyCode::Enter => {
                self.enter_dir();
                true
            }
            KeyCode::Left => {
                self.go_up();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, _col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::Down(MouseButton::Left)
                if row >= 2 && row < self.area.height.saturating_sub(2) =>
            {
                let idx = self.scroll_offset + (row as usize - 2);
                if idx < self.entries.len() {
                    self.selected = idx;
                    self.list.scroll_to(self.selected);
                    self.dirty = true;
                    true
                } else {
                    false
                }
            }
            MouseEventKind::ScrollDown => {
                self.scroll_offset = (self.scroll_offset + 1)
                    .min(self.entries.len().saturating_sub(self.visible_count));
                self.dirty = true;
                true
            }
            MouseEventKind::ScrollUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.list.on_theme_change(theme);
        self.dirty = true;
    }
}

fn main() -> std::io::Result<()> {
    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let theme = Theme::cyberpunk();
    let mut app = FileManagerApp::new(theme);
    app.set_area(Rect::new(0, 0, w, h));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app_widget = App::new()?.title("File Manager").fps(30).theme(theme);
    app_widget.add_widget(Box::new(app), Rect::new(0, 0, w, h));
    app_widget = app_widget
        .on_input(move |key| {
            if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
            }
        });
    app_widget.run(|_ctx| {})
}
