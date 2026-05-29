#![allow(missing_docs)]
//! Menu System  -  MenuBar, ContextMenu, and keyboard shortcuts.
//!
//! Demonstrates:
//! - Custom MenuBar with File/Edit/View/Help dropdown menus
//! - ContextMenu on right-click with item-specific actions
//! - Global keyboard shortcuts (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Q)
//! - Toast feedback for menu actions
//! - List widget with selection and context menu support
//!
//! ## Shortcuts
//! | Key | Action |
//! |-----|--------|
//! | Ctrl+N | New file |
//! | Ctrl+O | Open file |
//! | Ctrl+S | Save file |
//! | Ctrl+Q | Quit application |
//! | ESC | Close menu/modal |

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    ContextAction, ContextMenu, StatusBar, StatusSegment, Toast, ToastKind,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::os::fd::AsFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct MenuLabel(&'static str);

struct MenuApp {
    id: WidgetId,
    list: List<String>,
    status_bar: StatusBar,
    context_menu: Option<ContextMenu>,
    toasts: Vec<Toast>,
    menu_bar: Vec<MenuLabel>,
    active_menu: Option<usize>,
    selected_idx: Option<usize>,
    area: Rect,
    should_quit: Arc<AtomicBool>,
    theme: Theme,
    show_help: bool,
    keybindings: KeybindingSet,
}

impl MenuApp {
    fn new(id: WidgetId, should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let items = vec![
            "documents".to_string(),
            "images".to_string(),
            "projects".to_string(),
            "downloads".to_string(),
            "music".to_string(),
            "videos".to_string(),
        ];
        let menu_bar = vec![
            MenuLabel("File File"),
            MenuLabel("Edit Edit"),
            MenuLabel("View View"),
            MenuLabel("INFO Help"),
        ];
        let keybindings = KeybindingSet::from_config(&resolve_keybindings());
        let kb_theme = keybindings.display(actions::THEME).unwrap_or("t");
        let kb_help = keybindings.display(actions::HELP).unwrap_or("?");
        let kb_back = keybindings.display(actions::BACK).unwrap_or("Esc");
        let kb_quit = keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q");
        let status_bar = StatusBar::new(WidgetId::new(2))
            .add_segment(StatusSegment::new("Ready").with_fg(theme.success))
            .add_segment(
                StatusSegment::new(&format!(
                    "Ctrl+N|O|S: New|Open|Save | {}: theme | {}: help | {}: dismiss | {}: Quit",
                    kb_theme, kb_help, kb_back, kb_quit
                ))
                .with_fg(theme.fg_muted),
            );
        Self {
            id,
            list: List::new(items),
            status_bar,
            context_menu: None,
            toasts: Vec::new(),
            menu_bar,
            active_menu: None,
            selected_idx: None,
            area: Rect::new(0, 0, 80, 24),
            should_quit,
            theme,
            show_help: false,
            keybindings,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.status_bar.on_theme_change(&self.theme);
        self.list.on_theme_change(&self.theme);
    }

    fn toast(&mut self, msg: &str, kind: ToastKind) {
        self.toasts.push(
            Toast::new(WidgetId::new(100 + self.toasts.len()), msg)
                .with_kind(kind)
                .with_duration(std::time::Duration::from_secs(2)),
        );
    }

    fn menu_item_count(&self, menu_idx: usize) -> usize {
        match menu_idx {
            0 => 4,
            1 => 3,
            2 => 3,
            3 => 2,
            _ => 0,
        }
    }

    fn get_menu_item(&self, menu_idx: usize, item_idx: usize) -> &'static str {
        match menu_idx {
            0 => match item_idx {
                0 => "New (Ctrl+N)",
                1 => "Open (Ctrl+O)",
                2 => "Save (Ctrl+S)",
                3 => "Exit (Ctrl+Q)",
                _ => "",
            },
            1 => match item_idx {
                0 => "Copy (Ctrl+C)",
                1 => "Paste (Ctrl+V)",
                2 => "Select All (Ctrl+A)",
                _ => "",
            },
            2 => match item_idx {
                0 => "Toggle Sidebar",
                1 => "Zoom In",
                2 => "Zoom Out",
                _ => "",
            },
            3 => match item_idx {
                0 => "About",
                1 => "Documentation",
                _ => "",
            },
            _ => "",
        }
    }

    fn do_menu(&mut self, menu_idx: usize, item_idx: usize) {
        let label = self.get_menu_item(menu_idx, item_idx);
        match label {
            s if s.contains("New") => self.toast("New file created", ToastKind::Success),
            s if s.contains("Open") => self.toast("Opened file dialog", ToastKind::Info),
            s if s.contains("Save") => self.toast("Saved!", ToastKind::Success),
            s if s.contains("Exit") => self.toast("Goodbye!", ToastKind::Info),
            s if s.contains("Copy") => self.toast("Copied to clipboard", ToastKind::Info),
            s if s.contains("Paste") => self.toast("Pasted from clipboard", ToastKind::Info),
            s if s.contains("Select All") => self.toast("All items selected", ToastKind::Info),
            s if s.contains("Toggle Sidebar") => self.toast("Sidebar toggled", ToastKind::Info),
            s if s.contains("Zoom") => self.toast("Zoom changed", ToastKind::Info),
            s if s.contains("About") => self.toast("Dracon Terminal Engine v27", ToastKind::Info),
            s if s.contains("Documentation") => self.toast("Opening docs...", ToastKind::Info),
            _ => {}
        }
        self.active_menu = None;
    }

    fn do_ctx(&mut self, idx: usize) {
        match idx {
            0 => self.toast("Item copied", ToastKind::Info),
            1 => self.toast("Item pasted", ToastKind::Info),
            2 => self.toast("Rename mode", ToastKind::Info),
            3 => self.toast("Item deleted", ToastKind::Warning),
            4 => self.toast("Opening...", ToastKind::Info),
            _ => {}
        }
        self.context_menu = None;
        self.selected_idx = None;
    }

    fn render_menu_bar(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, 1);
        plane.z_index = 60;
        for cell in plane.cells.iter_mut() {
            cell.bg = self.theme.surface_elevated;
            cell.fg = self.theme.fg;
        }

        let total = self.menu_bar.len();
        let entry_w = (area.width as usize / total.max(1)) as u16;

        for (i, m) in self.menu_bar.iter().enumerate() {
            let is_active = self.active_menu == Some(i);
            let prefix = if is_active { "[" } else { " " };
            let suffix = if is_active { "]" } else { " " };
            let display = format!("{}{}{}", prefix, m.0, suffix);
            for (j, ch) in display.chars().enumerate() {
                let idx = (i as u16 * entry_w + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                }
            }
        }
        plane
    }

    fn render_dropdown(&self, menu_idx: usize, _area: Rect) -> Plane {
        let item_count = self.menu_item_count(menu_idx) as u16;
        let label = self.menu_bar.get(menu_idx).map(|m| m.0).unwrap_or("");
        let w = 20.max(label.len() as u16 + 4);
        let h = item_count;

        let mut plane = Plane::new(0, w, h);
        plane.z_index = 70;
        for cell in plane.cells.iter_mut() {
            cell.bg = self.theme.surface_elevated;
            cell.fg = self.theme.fg;
        }

        for i in 0..self.menu_item_count(menu_idx) {
            let item_label = self.get_menu_item(menu_idx, i);
            let is_selected = self.selected_idx == Some(i);
            for (j, ch) in item_label.chars().take(w as usize - 2).enumerate() {
                let idx = (i as u16 * w + 2 + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    if is_selected {
                        plane.cells[idx].bg = self.theme.selection_bg;
                        plane.cells[idx].fg = self.theme.fg;
                    }
                }
            }
            if is_selected {
                for col in 0..w {
                    let idx = (i as u16 * w + col) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = self.theme.selection_bg;
                    }
                }
            }
        }

        for col in 0..w {
            if (col as usize) < plane.cells.len() {
                plane.cells[col as usize].char = '─';
            }
        }
        plane
    }
}

impl Widget for MenuApp {
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
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.list.on_theme_change(theme);
        self.status_bar.on_theme_change(theme);
        for toast in &mut self.toasts {
            toast.on_theme_change(theme);
        }
    }

    fn current_theme(&self) -> Option<Theme> {
        Some(self.theme.clone())
    }

    fn focusable(&self) -> bool {
        true
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        for cell in plane.cells.iter_mut() {
            cell.transparent = false;
        }
        plane.z_index = 0;

        let (hdr, ftr) = (1u16, 1u16);
        let content_h = area.height.saturating_sub(hdr + ftr);

        let menu_plane = self.render_menu_bar(Rect::new(0, 0, area.width, hdr));
        for (i, c) in menu_plane.cells.iter().enumerate() {
            if !c.transparent {
                plane.cells[i] = *c;
            }
        }

        if let Some(idx) = self.active_menu {
            let dp = self.render_dropdown(idx, area);
            let base_idx = (hdr * area.width) as usize;
            for (i, c) in dp.cells.iter().enumerate() {
                let idx = base_idx + i;
                if !c.transparent && idx < plane.cells.len() {
                    plane.cells[idx] = *c;
                }
            }
        }

        let list_rect = Rect::new(2, hdr + 1, area.width - 4, content_h.saturating_sub(2));
        let list_plane = self.list.render(list_rect);
        for (i, c) in list_plane.cells.iter().enumerate() {
            let dest_x = i as u16 % list_plane.width;
            let dest_y = i as u16 / list_plane.width;
            let idx = (dest_y * area.width + dest_x + 2) as usize;
            if !c.transparent && idx < plane.cells.len() {
                plane.cells[idx] = *c;
            }
        }

        if let Some(ref cm) = self.context_menu {
            let cm_plane = cm.render(area);
            for (y, row) in cm_plane.cells.chunks(cm_plane.width as usize).enumerate() {
                for (x, c) in row.iter().enumerate() {
                    if !c.transparent {
                        let idx = (y as u16 * area.width + x as u16) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx] = *c;
                        }
                    }
                }
            }
        }

        let mut toast_y = 1u16;
        for toast in &self.toasts {
            if toast.is_expired() {
                continue;
            }
            let tp = toast.render(Rect::new(0, 0, area.width, 1));
            for x in 0..tp.width as usize {
                if x < plane.cells.len() && x < area.width as usize {
                    let src_idx = x;
                    let dst_idx = toast_y as usize * area.width as usize + x;
                    if src_idx < tp.cells.len() && dst_idx < plane.cells.len() {
                        let c = &tp.cells[src_idx];
                        if !c.transparent {
                            plane.cells[dst_idx] = *c;
                        }
                    }
                }
            }
            toast_y = toast_y.saturating_add(1);
            if toast_y >= area.height.saturating_sub(2) {
                break;
            }
        }

        let status_plane = self
            .status_bar
            .render(Rect::new(0, area.height - ftr, area.width, ftr));
        let status_base = ((area.height - ftr) * area.width) as usize;
        for (i, c) in status_plane.cells.iter().enumerate() {
            let idx = status_base + i;
            if !c.transparent && idx < plane.cells.len() {
                plane.cells[idx] = *c;
            }
        }

        // Help overlay
        if self.show_help {
            let hw = 44u16.min(area.width.saturating_sub(4));
            let hh = 12u16.min(area.height.saturating_sub(4));
            let hx = (area.width - hw) / 2;
            let hy = (area.height - hh) / 2;
            for y in hy..hy + hh {
                for x in hx..hx + hw {
                    let idx = (y * area.width + x) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = self.theme.surface_elevated;
                        plane.cells[idx].transparent = false;
                    }
                }
            }
            // Border
            for x in hx + 1..hx + hw - 1 {
                let top_idx = (hy * area.width + x) as usize;
                let bot_idx = ((hy + hh - 1) * area.width + x) as usize;
                if top_idx < plane.cells.len() {
                    plane.cells[top_idx].char = '─';
                    plane.cells[top_idx].fg = self.theme.outline;
                }
                if bot_idx < plane.cells.len() {
                    plane.cells[bot_idx].char = '─';
                    plane.cells[bot_idx].fg = self.theme.outline;
                }
            }
            for y in hy + 1..hy + hh - 1 {
                let left_idx = (y * area.width + hx) as usize;
                let right_idx = (y * area.width + hx + hw - 1) as usize;
                if left_idx < plane.cells.len() {
                    plane.cells[left_idx].char = '│';
                    plane.cells[left_idx].fg = self.theme.outline;
                }
                if right_idx < plane.cells.len() {
                    plane.cells[right_idx].char = '│';
                    plane.cells[right_idx].fg = self.theme.outline;
                }
            }
            // Rounded corners
            let corners = [
                ('╭', hx, hy),
                ('╮', hx + hw - 1, hy),
                ('╰', hx, hy + hh - 1),
                ('╯', hx + hw - 1, hy + hh - 1),
            ];
            for (ch, cx, cy) in corners {
                let idx = (cy * area.width + cx) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = self.theme.outline;
                }
            }
            // Title
            let title = "Menu System Help";
            let tx = hx + (hw - title.len() as u16) / 2;
            for (i, c) in title.chars().enumerate() {
                let idx = ((hy + 1) * area.width + tx + i as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = c;
                    plane.cells[idx].fg = self.theme.primary;
                    plane.cells[idx].style = Styles::BOLD;
                }
            }
            // Shortcuts
            let kb_theme = self.keybindings.display(actions::THEME).unwrap_or("t");
            let kb_help = self.keybindings.display(actions::HELP).unwrap_or("?");
            let kb_back = self.keybindings.display(actions::BACK).unwrap_or("Esc");
            let kb_quit = self.keybindings.display(actions::QUIT).unwrap_or("Ctrl+Q");
            let shortcuts = [
                ("^/v", "Navigate list"),
                ("Ctrl+N", "New file"),
                ("Ctrl+O", "Open"),
                ("Ctrl+S", "Save"),
                (kb_quit, "Quit"),
                (kb_theme, "Cycle theme"),
                (kb_help, "Toggle help"),
                (kb_back, "Dismiss help"),
            ];
            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let row = hy + 3 + i as u16;
                for (j, c) in key.chars().enumerate() {
                    let idx = (row * area.width + hx + 2 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = self.theme.primary;
                    }
                }
                for (j, c) in desc.chars().enumerate() {
                    let idx = (row * area.width + hx + 14 + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].char = c;
                        plane.cells[idx].fg = self.theme.fg;
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

        self.toasts.retain(|t| !t.is_expired());

        if self.show_help {
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
                self.show_help = false;
            }
            return true;
        }

        if self.context_menu.is_some() {
            if self.keybindings.matches(actions::BACK, &key) {
                self.context_menu = None;
                self.selected_idx = None;
            }
            return true;
        }

        if let Some(menu_idx) = self.active_menu {
            if self.keybindings.matches(actions::BACK, &key) {
                self.active_menu = None;
                return true;
            }
            match key.code {
                KeyCode::Enter => {
                    if let Some(item_idx) = self.selected_idx {
                        self.do_menu(menu_idx, item_idx);
                    }
                    true
                }
                KeyCode::Down | KeyCode::Up => {
                    let item_count = self.menu_item_count(menu_idx);
                    if item_count == 0 {
                        return true;
                    }
                    let current = self.selected_idx.unwrap_or(0);
                    let next = match key.code {
                        KeyCode::Down => (current + 1).min(item_count - 1),
                        KeyCode::Up => current.saturating_sub(1),
                        _ => current,
                    };
                    self.selected_idx = Some(next);
                    true
                }
                _ => {
                    self.active_menu = None;
                    false
                }
            }
        } else if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('n') if self.keybindings.matches(actions::NEW_ITEM, &key) => {
                    self.toast("New file created", ToastKind::Success);
                    true
                }
                KeyCode::Char('o') => {
                    self.toast("Opened file dialog", ToastKind::Info);
                    true
                }
                KeyCode::Char('s') if self.keybindings.matches(actions::SAVE, &key) => {
                    self.toast("Saved!", ToastKind::Success);
                    true
                }
                _ if self.keybindings.matches(actions::QUIT, &key) => {
                    self.toast("Goodbye!", ToastKind::Info);
                    self.should_quit.store(true, Ordering::SeqCst);
                    true
                }
                KeyCode::Char('v') => {
                    self.toast("Pasted from clipboard", ToastKind::Info);
                    true
                }
                KeyCode::Char('a') => {
                    self.toast("All items selected", ToastKind::Info);
                    true
                }
                _ => false,
            }
        } else if self.keybindings.matches(actions::QUIT, &key) {
            self.toast("Goodbye!", ToastKind::Info);
            self.should_quit.store(true, Ordering::SeqCst);
            true
        } else if self.keybindings.matches(actions::BACK, &key) {
            self.active_menu = None;
            self.context_menu = None;
            true
        } else if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            true
        } else if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            true
        } else {
            self.list.handle_key(key)
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let (hdr, ftr) = (1u16, 1u16);
        let content_h = self.area.height.saturating_sub(hdr + ftr);

        if self.context_menu.is_some() && matches!(kind, MouseEventKind::Down(MouseButton::Left)) {
            self.context_menu = None;
            self.selected_idx = None;
            return true;
        }

        if row == 0 {
            if let MouseEventKind::Down(MouseButton::Left) = kind {
                let total = self.menu_bar.len();
                let entry_w = (self.area.width as usize / total.max(1)) as u16;
                let menu_idx = (col / entry_w) as usize;
                if menu_idx < total {
                    if self.active_menu == Some(menu_idx) {
                        self.active_menu = None;
                    } else {
                        self.active_menu = Some(menu_idx);
                        self.selected_idx = Some(0);
                    }
                }
                return true;
            }
        }

        if let MouseEventKind::Down(MouseButton::Right) = kind {
            let list_rect = Rect::new(
                2,
                hdr + 1,
                self.area.width.saturating_sub(4),
                content_h.saturating_sub(2),
            );
            if col >= list_rect.x
                && col < list_rect.x + list_rect.width
                && row >= list_rect.y
                && row < list_rect.y + list_rect.height
            {
                self.context_menu = Some(
                    ContextMenu::from_actions_with_id(
                        WidgetId::new(50),
                        vec![
                            ("Copy Item", ContextAction::Copy),
                            ("Paste Item", ContextAction::Paste),
                            ("Rename Item", ContextAction::Rename),
                            ("Delete Item", ContextAction::Delete),
                            ("Properties", ContextAction::Open),
                        ],
                    )
                    .with_anchor(col, row),
                );
                return true;
            }
        }

        if let Some(ref mut cm) = self.context_menu {
            if cm.handle_mouse(kind, col, row) {
                if let MouseEventKind::Down(MouseButton::Left) = kind {
                    let local_row = row.saturating_sub(cm.area().y) as usize;
                    self.selected_idx = Some(local_row);
                    self.do_ctx(local_row);
                }
                return true;
            }
        }

        let list_rect = Rect::new(
            2,
            hdr + 1,
            self.area.width.saturating_sub(4),
            content_h.saturating_sub(2),
        );
        if col >= list_rect.x
            && col < list_rect.x + list_rect.width
            && row >= list_rect.y
            && row < list_rect.y + list_rect.height
        {
            return self
                .list
                .handle_mouse(kind, col - list_rect.x, row - list_rect.y);
        }
        false
    }
}

fn main() -> std::io::Result<()> {
    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::from_env_or(Theme::nord());
    let mut app_widget = MenuApp::new(WidgetId::new(0), should_quit, theme.clone());
    app_widget.set_area(Rect::new(0, 0, w, h));

    let mut app = App::new()?
        .title("Menu System Demo")
        .fps(30)
        .set_theme(theme);
    app.add_widget(Box::new(app_widget), Rect::new(0, 0, w, h));
    app.on_tick(move |ctx, _| {
        if quit_check.load(Ordering::SeqCst) {
            ctx.stop();
        }
    })
    .run(|_ctx| {})
}
