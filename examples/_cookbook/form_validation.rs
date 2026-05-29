#![allow(missing_docs)]
//! Form Validation — Multi-field form with live validation, theme cycling, and help overlay.
//!
//! Demonstrates the `Form` widget with `ValidationRule`s, status bar, and keyboard shortcuts.

use std::cell::RefCell;
use std::io::Result;
use std::os::fd::AsFd;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dracon_terminal_engine::compositor::{Cell, Plane, Styles};
use dracon_terminal_engine::framework::keybindings::{actions, resolve_keybindings, KeybindingSet};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Form, StatusBar, ValidationRule};
use dracon_terminal_engine::input::event::{KeyEventKind, MouseEventKind};
use ratatui::layout::Rect;

struct FormApp {
    id: WidgetId,
    area: Rect,
    theme: Theme,
    dirty: bool,
    form: Form,
    status: StatusBar,
    show_help: bool,
    keybindings: KeybindingSet,
    submit_message: Option<String>,
}

impl FormApp {
    fn new(theme: Theme) -> Self {
        let form = Form::new(WidgetId::new(10))
            .add_field("Username")
            .add_field("Email")
            .add_field("Password")
            .add_field("Bio")
            .with_theme(theme.clone())
            .with_validation(
                0,
                vec![
                    ValidationRule::Required,
                    ValidationRule::MinLength(3),
                    ValidationRule::MaxLength(20),
                ],
            )
            .with_validation(1, vec![ValidationRule::Required, ValidationRule::Email])
            .with_validation(
                2,
                vec![ValidationRule::Required, ValidationRule::MinLength(8)],
            )
            .with_validation(3, vec![ValidationRule::MaxLength(200)]);

        let status = StatusBar::new(WidgetId::new(20))
            .add_segment(
                dracon_terminal_engine::framework::widgets::StatusSegment::new(
                    "Ctrl+T: theme | F1: help | Esc: dismiss | Ctrl+S: submit | Ctrl+Q: quit",
                ),
            )
            .with_theme(theme.clone());

        Self {
            id: WidgetId::new(1),
            area: Rect::new(0, 0, 80, 24),
            theme,
            dirty: true,
            form,
            status,
            show_help: false,
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
            submit_message: None,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|t| t.name == self.theme.name)
            .unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.form.on_theme_change(&self.theme);
        self.status.on_theme_change(&self.theme);
        self.dirty = true;
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let hw = 42u16.min(area.width.saturating_sub(4));
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
        for (ch, cx, cy) in &corners {
            let idx = (*cy * area.width + *cx) as usize;
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

        let title = "Form Validation Help";
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
            ("Tab", "Next field"),
            ("Shift+Tab", "Previous field"),
            ("Ctrl+S", "Submit form"),
            ("Ctrl+T", "Cycle theme"),
            ("F1", "Toggle help"),
            ("Esc", "Dismiss help"),
            ("Ctrl+Q", "Quit"),
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
}

impl Widget for FormApp {
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
        self.dirty = true;
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

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.form.on_theme_change(theme);
        self.status.on_theme_change(theme);
        self.dirty = true;
    }

    fn render(&self, area: Rect) -> Plane {
        let t = &self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        plane.fill_bg(t.bg);

        let title = " Form Validation ";
        for (i, c) in title.chars().enumerate() {
            let idx = 1 + i;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell {
                    char: c,
                    fg: t.fg_on_accent,
                    bg: t.primary,
                    style: Styles::BOLD,
                    transparent: false,
                    skip: false,
                };
            }
        }

        let form_area = Rect::new(
            area.x + 2,
            area.y + 2,
            area.width.saturating_sub(4),
            area.height.saturating_sub(5),
        );
        let form_plane = self.form.render(form_area);
        for (i, c) in form_plane.cells.iter().enumerate() {
            if c.transparent || c.char == '\0' {
                continue;
            }
            let row = i / form_plane.width as usize;
            let col = i % form_plane.width as usize;
            let dy = form_area.y as usize + row;
            let dx = form_area.x as usize + col;
            if dy < area.height as usize && dx < area.width as usize {
                let idx = dy * area.width as usize + dx;
                if idx < plane.cells.len() {
                    plane.cells[idx] = *c;
                }
            }
        }

        let indicator_x = area.width.saturating_sub(2);
        if indicator_x >= form_area.x + form_area.width {
            let mut field_row: u16 = 0;
            for i in 0..self.form.field_count() {
                if field_row >= form_area.height {
                    break;
                }
                let abs_y = form_area.y + field_row;
                if abs_y < area.height {
                    let idx = (abs_y as usize) * area.width as usize + indicator_x as usize;
                    if idx < plane.cells.len() {
                        let valid = self.form.is_field_valid(i);
                        plane.cells[idx] = Cell {
                            char: if valid { '✓' } else { '✗' },
                            fg: if valid { t.success } else { t.error },
                            bg: t.bg,
                            style: Styles::BOLD,
                            transparent: false,
                            skip: false,
                        };
                    }
                }
                field_row += if self.form.has_field_error(i) { 2 } else { 1 };
            }
        }

        if let Some(ref msg) = self.submit_message {
            let msg_y = area.height.saturating_sub(3);
            for (i, c) in msg.chars().enumerate().take(area.width as usize - 4) {
                let idx = (msg_y as usize) * area.width as usize + 2 + i;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: t.success,
                        bg: t.surface_elevated,
                        style: Styles::BOLD,
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        let sb_area = Rect::new(0, area.height.saturating_sub(1), area.width, 1);
        let sb_plane = self.status.render(sb_area);
        for (i, c) in sb_plane.cells.iter().enumerate() {
            if c.transparent || c.char == '\0' {
                continue;
            }
            let col = i % sb_plane.width as usize;
            if col < area.width as usize {
                let idx = (area.height as usize - 1) * area.width as usize + col;
                if idx < plane.cells.len() {
                    plane.cells[idx] = *c;
                }
            }
        }

        if self.show_help {
            self.render_help_overlay(&mut plane, area);
        }

        plane
    }

    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
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

        if self.keybindings.matches(actions::QUIT, &key) {
            return false;
        }
        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = true;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::THEME, &key) {
            self.cycle_theme();
            return true;
        }
        if self.keybindings.matches(actions::SAVE, &key) {
            match self.form.validate() {
                Ok(()) => {
                    self.submit_message = Some("Form submitted successfully!".to_string());
                    self.dirty = true;
                }
                Err(errors) => {
                    self.submit_message =
                        Some(format!("Validation failed: {} error(s)", errors.len()));
                    self.dirty = true;
                }
            }
            return true;
        }

        self.form.handle_key(key)
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        match kind {
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => true,
            MouseEventKind::Down(_) => {
                let form_x = self.area.x + 2;
                let form_y = self.area.y + 2;
                let form_w = self.area.width.saturating_sub(4);
                let form_h = self.area.height.saturating_sub(5);
                if col >= form_x && col < form_x + form_w && row >= form_y && row < form_y + form_h
                {
                    let rel_row = row.saturating_sub(form_y);
                    if self.form.handle_mouse(kind, col, rel_row) {
                        self.dirty = true;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

struct InputRouter {
    target: Rc<RefCell<FormApp>>,
    id: WidgetId,
    area: Rect,
}

impl Widget for InputRouter {
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
        false
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, _area: Rect) -> Plane {
        Plane::new(0, 0, 0)
    }
    fn handle_key(&mut self, key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        self.target.borrow_mut().handle_key(key)
    }
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.target.borrow_mut().handle_mouse(kind, col, row)
    }
    fn on_theme_change(&mut self, theme: &Theme) {
        self.target.borrow_mut().on_theme_change(theme);
    }
    fn current_theme(&self) -> Option<Theme> {
        Some(self.target.borrow().theme.clone())
    }
}

fn main() -> Result<()> {
    std::thread::sleep(std::time::Duration::from_millis(300));

    let (w, h) = dracon_terminal_engine::backend::tty::get_window_size(std::io::stdout().as_fd())
        .unwrap_or((80, 24));

    let env_theme = Theme::from_env_or(Theme::nord());
    let app = Rc::new(RefCell::new(FormApp::new(env_theme.clone())));
    let app_tick = Rc::clone(&app);
    let app_router = Rc::clone(&app);
    let app_input = Rc::clone(&app);

    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let mut app_ctx = App::new()?
        .title("Form Validation")
        .fps(30)
        .set_theme(env_theme.clone());

    let kb = app_input.borrow().keybindings.clone();

    let router = InputRouter {
        target: app_router,
        id: WidgetId::new(100),
        area: Rect::new(0, 0, w, h),
    };
    app_ctx.add_widget(Box::new(router), Rect::new(0, 0, w, h));

    app_ctx
        .on_input(move |key| {
            if key.kind != KeyEventKind::Press {
                return false;
            }
            let mut a = app_input.borrow_mut();
            if kb.matches(actions::QUIT, &key) {
                should_quit.store(true, Ordering::SeqCst);
                true
            } else {
                a.handle_key(key)
            }
        })
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) {
                ctx.stop();
                return;
            }
            let mut a = app_tick.borrow_mut();
            let (w, h) = ctx.compositor().size();
            if a.area.width != w || a.area.height != h {
                a.set_area(Rect::new(0, 0, w, h));
            }
            if a.needs_render() {
                ctx.add_plane(a.render(a.area));
                a.clear_dirty();
            }
        })
        .run(|_| {})
}
