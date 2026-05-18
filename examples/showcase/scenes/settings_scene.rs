//! Settings Panel scene — Form + KeyValueGrid demonstration.
//!
//! Shows a real Form widget with validation rules, alongside a KeyValueGrid
//! showing configuration values. Users can fill in the form, validate,
//! and see the config grid update.

#![allow(dead_code)]

use crate::scenes::shared_helpers::{blit_to, draw_text};
use dracon_terminal_engine::compositor::plane::Plane;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Form, FormField, KeyValueGrid, StatusBar, StatusSegment, ValidationRule,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::collections::BTreeMap;

pub struct SettingsScene {
    theme: Theme,
    keybindings: KeybindingSet,
    form: RefCell<Form>,
    grid: RefCell<KeyValueGrid>,
    status_bar: RefCell<StatusBar>,
    show_help: bool,
    dirty: bool,
}

impl SettingsScene {
    pub fn new(theme: Theme) -> Self {
        let form = Form::new(WidgetId::new(400))
            .with_theme(theme.clone())
            .add_field("Username")
            .add_field("Email")
            .add_field("Password")
            .add_field("API Key")
            .add_field("Server URL")
            .with_validation(0, vec![
                ValidationRule::from_regex_pattern(".+").unwrap(),
            ])
            .with_validation(1, vec![
                ValidationRule::from_regex_pattern(r"^[^@]+@[^@]+\.[^@]+$").unwrap(),
            ])
            .with_validation(2, vec![
                ValidationRule::from_regex_pattern(".{8,}").unwrap(),
            ]);

        let mut grid = KeyValueGrid::with_id(WidgetId::new(401))
            .with_theme(theme.clone())
            .separator(" : ");
        let mut pairs = BTreeMap::new();
        pairs.insert("app.name".into(), "Dracon Settings".into());
        pairs.insert("app.version".into(), "0.1.10".into());
        pairs.insert("app.theme".into(), "nord".into());
        pairs.insert("server.host".into(), "localhost".into());
        pairs.insert("server.port".into(), "8080".into());
        pairs.insert("log.level".into(), "info".into());
        pairs.insert("log.max_size".into(), "10MB".into());
        pairs.insert("cache.enabled".into(), "true".into());
        pairs.insert("cache.ttl".into(), "3600s".into());
        grid.set_pairs(pairs);

        let status_bar = StatusBar::new(WidgetId::new(402))
            .add_segment(StatusSegment::new("Tab: fields | Enter: validate | S: save | F1: help | Esc: back"))
            .with_theme(theme.clone());

        Self {
            theme,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            form: RefCell::new(form),
            grid: RefCell::new(grid),
            status_bar: RefCell::new(status_bar),
            show_help: false,
            dirty: true,
        }
    }

    fn validate_form(&mut self) {
        let result = self.form.borrow_mut().validate();
        match result {
            Ok(()) => self.form.borrow_mut().set_field_error(0, ""), // Clear errors
            Err(errors) => {
                for (idx, msg) in errors {
                    self.form.borrow_mut().set_field_error(idx, &msg);
                }
            }
        }
        self.dirty = true;
    }

    fn save_settings(&mut self) {
        self.validate_form();
        let form = self.form.borrow();
        let mut pairs = BTreeMap::new();
        pairs.insert("user.username".into(), "dracon".into());
        pairs.insert("user.email".into(), "user@example.com".into());
        pairs.insert("server.host".into(), "localhost".into());
        pairs.insert("server.port".into(), "8080".into());
        pairs.insert("status".into(), "saved ✓".into());
        pairs.insert("saved_at".into(), "just now".into());
        drop(form);
        self.grid.borrow_mut().set_pairs(pairs);
        self.dirty = true;
    }
}

impl Scene for SettingsScene {
    fn on_enter(&mut self) {}
    fn on_exit(&mut self) {}

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(self.theme.bg);
        let t = &self.theme;

        // ── Title bar ──────────────────────────────────────────────
        draw_text(&mut plane, 1, 0, "Settings Panel", t.primary, t.bg, true);
        draw_text(&mut plane, 18, 0, "— Form + KeyValueGrid Demo", t.fg_muted, t.bg, false);

        // ── Form (left half) ───────────────────────────────────────
        let form_w = (area.width / 2).max(28);
        let content_h = area.height.saturating_sub(2);
        let form_area = Rect::new(0, 1, form_w, content_h);
        self.form.borrow_mut().set_area(form_area);
        let form_plane = self.form.borrow().render(form_area);
        blit_to(&mut plane, &form_plane, 0, 1);

        // ── Divider ────────────────────────────────────────────────
        for y in 1..area.height.saturating_sub(1) {
            let idx = (y as usize) * area.width as usize + form_w as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '│';
                plane.cells[idx].fg = t.outline;
                plane.cells[idx].transparent = false;
            }
        }

        // ── KeyValueGrid (right half) ───────────────────────────────
        let grid_x = form_w + 1;
        let grid_w = area.width.saturating_sub(form_w + 1);
        let grid_area = Rect::new(grid_x, 1, grid_w, content_h);
        self.grid.borrow_mut().set_area(grid_area);
        let grid_plane = self.grid.borrow().render(grid_area);
        blit_to(&mut plane, &grid_plane, grid_x as usize, 1);

        // ── Validation hints (below form, right side) ──────────────
        let hint_y = area.height.saturating_sub(4);
        draw_text(&mut plane, grid_x as u16 + 1, hint_y, "Validation Rules:", t.primary, t.bg, true);
        draw_text(&mut plane, grid_x as u16 + 1, hint_y + 1, "  Username: required", t.fg_muted, t.bg, false);
        draw_text(&mut plane, grid_x as u16 + 1, hint_y + 2, "  Email: must contain @", t.fg_muted, t.bg, false);
        draw_text(&mut plane, grid_x as u16 + 1, hint_y + 3, "  Password: min 8 chars", t.fg_muted, t.bg, false);

        // ── Status bar ─────────────────────────────────────────────
        let sb_y = area.height.saturating_sub(1);
        let sb_plane = self.status_bar.borrow().render(Rect::new(0, 0, area.width, 1));
        blit_to(&mut plane, &sb_plane, 0, sb_y as usize);

        // ── Help overlay ───────────────────────────────────────────
        if self.show_help {
            render_help_overlay(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if self.keybindings.matches(actions::HELP, &key) || self.keybindings.matches(actions::BACK, &key) {
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

        // Forward to form widget
        if self.form.borrow_mut().handle_key(key) {
            self.dirty = true;
            return true;
        }

        match key.code {
            KeyCode::Enter => {
                self.validate_form();
                true
            }
            KeyCode::Char('s') if key.modifiers.is_empty() => {
                self.save_settings();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if row >= 1 {
            return self.form.borrow_mut().handle_mouse(kind, col, row.saturating_sub(1));
        }
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.form.borrow_mut().on_theme_change(theme);
        self.grid.borrow_mut().on_theme_change(theme);
        self.status_bar.borrow_mut().on_theme_change(theme);
        self.dirty = true;
    }

    fn scene_id(&self) -> &str { "settings_panel" }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

fn render_help_overlay(plane: &mut Plane, area: Rect, t: &Theme) {
    let hw = 44u16.min(area.width.saturating_sub(4));
    let hh = 14u16.min(area.height.saturating_sub(4));
    let hx = (area.width.saturating_sub(hw)) / 2;
    let hy = (area.height.saturating_sub(hh)) / 2;

    for y in hy..hy + hh {
        for x in hx..hx + hw {
            let idx = (y as usize) * area.width as usize + x as usize;
            if idx < plane.cells.len() { plane.cells[idx].bg = t.surface_elevated; plane.cells[idx].transparent = false; }
        }
    }

    let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
    for (ch, cx, cy) in corners {
        let idx = (cy as usize) * area.width as usize + cx as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
    }
    for x in hx + 1..hx + hw - 1 {
        let ti = (hy as usize) * area.width as usize + x as usize;
        let bi = ((hy + hh - 1) as usize) * area.width as usize + x as usize;
        if ti < plane.cells.len() { plane.cells[ti].char = '─'; plane.cells[ti].fg = t.outline; }
        if bi < plane.cells.len() { plane.cells[bi].char = '─'; plane.cells[bi].fg = t.outline; }
    }
    for y in hy + 1..hy + hh - 1 {
        let li = (y as usize) * area.width as usize + hx as usize;
        let ri = (y as usize) * area.width as usize + (hx + hw - 1) as usize;
        if li < plane.cells.len() { plane.cells[li].char = '│'; plane.cells[li].fg = t.outline; }
        if ri < plane.cells.len() { plane.cells[ri].char = '│'; plane.cells[ri].fg = t.outline; }
    }

    let title = "Settings Panel — Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    for (i, c) in title.chars().enumerate() {
        let idx = ((hy + 1) as usize) * area.width as usize + (tx + i as u16) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; plane.cells[idx].style = Styles::BOLD; }
    }

    let shortcuts = [
        ("Tab", "Next form field"),
        ("Shift+Tab", "Previous form field"),
        ("Enter", "Validate form"),
        ("S", "Save settings"),
        ("Type", "Fill in form fields"),
        ("F1", "Toggle this help"),
        ("Esc", "Back to showcase"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let y = hy + 3 + i as u16;
        for (j, c) in key.chars().enumerate() {
            let idx = (y as usize) * area.width as usize + (hx + 2 + j as u16) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.primary; }
        }
        for (j, c) in desc.chars().enumerate() {
            let idx = (y as usize) * area.width as usize + (hx + 14 + j as u16) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = c; plane.cells[idx].fg = t.fg; }
        }
    }
}
