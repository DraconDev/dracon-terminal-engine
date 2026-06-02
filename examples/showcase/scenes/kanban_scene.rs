//! Embedded Kanban Board scene for the showcase.
//!
//! Displays a 3-column Kanban board with drag-and-drop cards,
//! keyboard navigation, and a stats sidebar.

use crate::scenes::shared_helpers::{blit_to, draw_text, render_help_overlay};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{Kanban, KanbanCard, StatusBar, StatusSegment};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;

const SIDEBAR_W: u16 = 18;
const DIV_X: u16 = SIDEBAR_W + 2;

pub struct KanbanScene {
    kanban: RefCell<Kanban>,
    theme: Theme,
    keybindings: KeybindingSet,
    show_help: bool,
    dirty: bool,
    status_bar: RefCell<StatusBar>,
    next_card_id: usize,
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
            (
                "t1",
                "Design API",
                Some("Define endpoints & schemas"),
                Some(Color::Rgb(100, 149, 237)),
            ),
            (
                "t2",
                "Setup CI/CD",
                Some("GitHub Actions pipeline"),
                Some(Color::Rgb(100, 149, 237)),
            ),
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
            (
                "w1",
                "Kanban widget",
                Some("Implement drag & drop"),
                Some(Color::Rgb(255, 165, 0)),
            ),
            (
                "w2",
                "Form validation",
                Some("Add validation rules"),
                Some(Color::Rgb(255, 165, 0)),
            ),
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
            (
                "d1",
                "Project setup",
                Some("Initial scaffolding"),
                Some(Color::Rgb(50, 205, 50)),
            ),
            (
                "d2",
                "Theme system",
                Some("Theme::nord, cyberpunk, etc."),
                Some(Color::Rgb(50, 205, 50)),
            ),
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

        let status_bar = RefCell::new(
            StatusBar::new(WidgetId::new(60))
                .add_segment(StatusSegment::new(
                    "Tab: nav | N: new | D: del | F1: help | Esc: back",
                ))
                .with_theme(theme.clone()),
        );

        Self {
            kanban: RefCell::new(kanban),
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            show_help: false,
            dirty: true,
            status_bar,
            next_card_id: 100,
        }
    }
}

impl Scene for KanbanScene {

    fn on_enter(&mut self) {
        self.show_help = false;
        self.dirty = true;
    }

    fn on_exit(&mut self) {
        self.show_help = false;
    }


    fn scene_id(&self) -> &str {
        "kanban"
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // Header
        draw_text(&mut plane, 2, 0, " Kanban Board ", t.primary, t.bg, true);
        let theme_label = format!(" {} ", self.theme.name);
        draw_text(
            &mut plane,
            area.width.saturating_sub(theme_label.len() as u16 + 2),
            0,
            &theme_label,
            t.secondary,
            t.bg,
            false,
        );

        // Divider
        for x in 0..area.width {
            let idx = x as usize;
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

        // Kanban board
        let board_x = DIV_X + 1;
        let board_w = area.width.saturating_sub(board_x);
        let board_h = area.height.saturating_sub(3);
        let kanban_area = Rect::new(board_x, 2, board_w, board_h);
        self.kanban.borrow_mut().set_area(kanban_area);
        let k_plane = self.kanban.borrow().render(kanban_area);
        for (ci, c) in k_plane.cells.iter().enumerate() {
            if c.transparent || c.char == '\0' {
                continue;
            }
            let row = ci / k_plane.width as usize;
            let col = ci % k_plane.width as usize;
            let dy = kanban_area.y as usize + row;
            let dx = kanban_area.x as usize + col;
            if dy < area.height as usize && dx < area.width as usize {
                let idx = dy * area.width as usize + dx;
                if idx < plane.cells.len() {
                    plane.cells[idx] = *c;
                }
            }
        }

        // Selected card indicator
        if self.kanban.borrow().selected_card().is_some() {
            let info_y = area.height.saturating_sub(3);
            draw_text(
                &mut plane,
                board_x + 1,
                info_y,
                "Card selected — use ↑↓ to navigate",
                t.fg_muted,
                t.bg,
                false,
            );
        }

        // Status bar
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self
            .status_bar
            .borrow()
            .render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        if self.show_help {
            let help_key = self.keybindings.display(actions::HELP).unwrap_or("f1");
            let back_key = self.keybindings.display(actions::BACK).unwrap_or("esc");
            render_help_overlay(
                &mut plane,
                area,
                t,
                "Kanban Board — Help",
                &[
                    ("Tab/Shift+Tab", "Focus next/prev column"),
                    ("←/→", "Navigate columns"),
                    ("↑/↓", "Navigate cards"),
                    ("N", "Add new card to To Do"),
                    ("D", "Delete selected card"),
                    ("Click", "Select card"),
                    (help_key, "Toggle this help"),
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
            if self.keybindings.matches(actions::BACK, &key)
                || self.keybindings.matches(actions::HELP, &key)
            {
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
            return false;
        }

        match key.code {
            KeyCode::Char('n') if key.modifiers.is_empty() => {
                let id = format!("new_{}", self.next_card_id);
                self.next_card_id += 1;
                let titles = [
                    "New task",
                    "Feature request",
                    "Bug fix",
                    "Research",
                    "Refactor",
                ];
                let descs = [
                    "Needs description",
                    "From user feedback",
                    "Priority fix",
                    "Investigate options",
                    "Clean up code",
                ];
                let idx = self.next_card_id % titles.len();
                let mut card = KanbanCard::new(id, titles[idx]);
                card = card.with_description(descs[idx]);
                card = card.with_color(Color::Rgb(100, 149, 237));
                self.kanban.borrow_mut().add_card(0, card);
                self.dirty = true;
                true
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                if let Some((col, idx)) = self.kanban.borrow().selected_card() {
                    self.kanban.borrow_mut().remove_card(col, idx);
                    self.dirty = true;
                }
                true
            }
            _ => {
                let handled = self.kanban.borrow_mut().handle_key(key);
                if handled {
                    self.dirty = true;
                }
                handled
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let adjusted_row = row.saturating_sub(2);
        let adjusted_col = col.saturating_sub(DIV_X + 1);
        self.kanban
            .borrow_mut()
            .handle_mouse(kind, adjusted_col, adjusted_row)
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.kanban.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.kanban.borrow().needs_render()
    }
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl KanbanScene {
    fn render_sidebar(&self, plane: &mut Plane, area: Rect, t: &Theme) {
        let sx = 2u16;

        // Title
        draw_text(plane, sx, 2, "Board Stats", t.primary, t.bg, true);

        // Card counts per column
        let k = self.kanban.borrow();
        let column_count = k.column_count();
        let total_cards: usize = (0..column_count).filter_map(|c| k.card_count(c)).sum();

        let col_labels = ["To Do", "In Progress", "Done"];
        for i in 0..column_count.min(3) {
            let count = k.card_count(i).unwrap_or(0);
            let label = col_labels.get(i).unwrap_or(&"Col");
            let is_focused = k.selected_card().map(|(c, _)| c).unwrap_or(99) == i;
            let fg = if is_focused { t.primary } else { t.fg };
            draw_text(plane, sx, 3 + i as u16, label, t.fg_muted, t.bg, false);
            draw_text(
                plane,
                sx + 12,
                3 + i as u16,
                &format!("{}", count),
                fg,
                t.bg,
                is_focused,
            );
        }

        // Divider
        let div_y = 7;
        for dx in 0..SIDEBAR_W {
            let idx = (div_y * plane.width + sx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        // Total
        draw_text(plane, sx, 9, "Total Cards", t.fg_muted, t.bg, false);
        draw_text(
            plane,
            sx + 12,
            9,
            &format!("{}", total_cards),
            t.info,
            t.bg,
            true,
        );

        // Summary
        let sum_y = 11;
        let done_pct = if total_cards > 0 {
            let done = k.card_count(2).unwrap_or(0);
            (done * 100).checked_div(total_cards).unwrap_or(0)
        } else {
            0
        };
        draw_text(plane, sx, sum_y, "Progress", t.secondary, t.bg, true);
        draw_text(
            plane,
            sx,
            sum_y + 1,
            &format!("{}% done", done_pct),
            t.success,
            t.bg,
            false,
        );

        // Progress bar
        let bar_w = SIDEBAR_W.saturating_sub(4);
        let bar_y = sum_y + 2;
        let filled = if total_cards > 0 {
            (done_pct as u16 * bar_w) / 100
        } else {
            0
        };
        for dx in 0..bar_w {
            let idx = (bar_y * plane.width + sx + dx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '█';
                plane.cells[idx].fg = if dx < filled { t.success } else { t.surface };
                plane.cells[idx].bg = t.bg;
                plane.cells[idx].transparent = false;
            }
        }

        // Legend
        let leg_y = area.height.saturating_sub(6);
        if leg_y > bar_y + 3 {
            draw_text(plane, sx, leg_y, "Legend", t.secondary, t.bg, true);
            draw_text(
                plane,
                sx,
                leg_y + 1,
                "● Design",
                Color::Rgb(100, 149, 237),
                t.bg,
                false,
            );
            draw_text(
                plane,
                sx,
                leg_y + 2,
                "● Active",
                Color::Rgb(255, 165, 0),
                t.bg,
                false,
            );
            draw_text(
                plane,
                sx,
                leg_y + 3,
                "● Done",
                Color::Rgb(50, 205, 50),
                t.bg,
                false,
            );
        }
    }
}
