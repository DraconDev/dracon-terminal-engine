//! Embedded Kanban Board scene for the showcase.
//!
//! Displays a 3-column Kanban board with drag-and-drop cards,
//! keyboard navigation, and theme support.
//! Press `B`/`Esc` to go back.

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Kanban, KanbanCard, StatusBar};
use dracon_terminal_engine::input::event::{KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

pub struct KanbanScene {
    kanban: Kanban,
    theme: Theme,
    show_help: bool,
    dirty: bool,
    area: std::cell::Cell<Rect>,
    status_bar: StatusBar,
    keybindings: KeybindingSet,
}

impl KanbanScene {
    pub fn new(theme: Theme) -> Self {
        let mut kanban = Kanban::new()
            .with_theme(theme.clone())
            .with_column_width(22)
            .with_card_height(3);

        kanban.add_column("To Do");
        kanban.add_column("In Progress");
        kanban.add_column("Done");

        let todos = [
            ("t1", "Design API", Some("Define endpoints & schemas"), Some(Color::Rgb(100, 149, 237))),
            ("t2", "Setup CI/CD", Some("GitHub Actions pipeline"), Some(Color::Rgb(100, 149, 237))),
            ("t3", "Write docs", Some("User guide & API ref"), None),
            ("t4", "Add tests", Some("Unit + integration tests"), None),
            ("t5", "Code review", Some("Review PRs from team"), None),
        ];
        for (id, title, desc, color) in &todos {
            let mut card = KanbanCard::new(*id, *title);
            if let Some(d) = desc {
                card = card.with_description(*d);
            }
            if let Some(c) = color {
                card = card.with_color(*c);
            }
            kanban.add_card(0, card);
        }

        let wips = [
            ("w1", "Kanban widget", Some("Implement drag & drop"), Some(Color::Rgb(255, 165, 0))),
            ("w2", "Form validation", Some("Add validation rules"), Some(Color::Rgb(255, 165, 0))),
            ("w3", "Refactor core", Some("Extract framework crate"), None),
        ];
        for (id, title, desc, color) in &wips {
            let mut card = KanbanCard::new(*id, *title);
            if let Some(d) = desc {
                card = card.with_description(*d);
            }
            if let Some(c) = color {
                card = card.with_color(*c);
            }
            kanban.add_card(1, card);
        }

        let dones = [
            ("d1", "Project setup", Some("Initial scaffolding"), Some(Color::Rgb(50, 205, 50))),
            ("d2", "Theme system", Some("Theme::nord, cyberpunk, etc."), Some(Color::Rgb(50, 205, 50))),
        ];
        for (id, title, desc, color) in &dones {
            let mut card = KanbanCard::new(*id, *title);
            if let Some(d) = desc {
                card = card.with_description(*d);
            }
            if let Some(c) = color {
                card = card.with_color(*c);
            }
            kanban.add_card(2, card);
        }

        let status_bar = StatusBar::new(WidgetId::new(60))
            .add_segment(dracon_terminal_engine::framework::widgets::StatusSegment::new(
                "Tab/shift+tab: nav | ←→: col | ↑↓: card | B/Esc: back | ?: help",
            ))
            .with_theme(theme.clone());

        Self {
            kanban,
            theme,
            show_help: false,
            dirty: true,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            status_bar,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
        }
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 42u16.min(area.width.saturating_sub(4));
        let hh = 13u16.min(area.height.saturating_sub(4));
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
            ('╭', hx, hy), ('╮', hx + hw - 1, hy),
            ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1),
        ];
        for (ch, cx, cy) in &corners {
            let idx = (*cy * area.width + *cx) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = *ch; plane.cells[idx].fg = t.outline; }
        }
        for x in hx + 1..hx + hw - 1 {
            let top = (hy * area.width + x) as usize;
            let bot = ((hy + hh - 1) * area.width + x) as usize;
            if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
            if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
        }
        for y in hy + 1..hy + hh - 1 {
            let left = (y * area.width + hx) as usize;
            let right = (y * area.width + hx + hw - 1) as usize;
            if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
            if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
        }

        let title = "Kanban Board Help";
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
            ("Tab", "Focus next column"),
            ("Shift+Tab", "Focus previous column"),
            ("← →", "Navigate columns"),
            ("↑ ↓", "Navigate cards"),
            ("Enter", "Select card"),
            ("B/Esc", "Back to showcase"),
            ("?", "Toggle help"),
        ];
        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let row = hy + 3 + i as u16;
            for (j, c) in key.chars().enumerate() {
                let idx = (row * area.width + hx + 2 + j as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
            }
            for (j, c) in desc.chars().enumerate() {
                let idx = (row * area.width + hx + 14 + j as u16) as usize;
                if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
            }
        }
    }
}

impl Scene for KanbanScene {
    fn scene_id(&self) -> &str { "kanban" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(t.bg);

        let header = " Kanban Board ";
        for (i, c) in header.chars().enumerate() {
            let idx = 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c, fg: t.fg_on_accent, bg: t.primary,
                    style: Styles::BOLD, transparent: false, skip: false,
                };
            }
        }

        let kanban_area = Rect::new(
            0, 1,
            area.width,
            area.height.saturating_sub(3),
        );
        let k_plane = self.kanban.render(kanban_area);
        for (ci, c) in k_plane.cells.iter().enumerate() {
            if c.transparent || c.char == '\0' { continue; }
            let row = ci / k_plane.width as usize;
            let col = ci % k_plane.width as usize;
            let dy = kanban_area.y as usize + row;
            let dx = kanban_area.x as usize + col;
            if dy < area.height as usize && dx < area.width as usize {
                let idx = dy * area.width as usize + dx;
                if idx < plane.cells.len() { plane.cells[idx] = *c; }
            }
        }

        let bar_area = Rect::new(0, area.height.saturating_sub(1), area.width, 1);
        let bar_plane = self.status_bar.render(bar_area);
        for (ci, c) in bar_plane.cells.iter().enumerate() {
            if c.transparent || c.char == '\0' { continue; }
            let col = ci % bar_plane.width as usize;
            if col < area.width as usize {
                let idx = (area.height as usize - 1) * area.width as usize + col;
                if idx < plane.cells.len() { plane.cells[idx] = *c; }
            }
        }

        if self.show_help {
            self.render_help_overlay(&mut plane, area);
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
            return false; // Let scene router handle back navigation
        }

        self.kanban.handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let adjusted_row = row.saturating_sub(1); // offset by header
        self.kanban.handle_mouse(kind, col, adjusted_row)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.kanban.on_theme_change(theme);
        self.status_bar.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty || self.kanban.needs_render() }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}
