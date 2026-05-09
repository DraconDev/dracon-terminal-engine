//! Embedded Form Demo scene for the showcase.
//!
//! Demonstrates form fields with validation and submit.
//! Press `Tab` to cycle focus, `B`/`Esc` to go back.

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, PasswordInput, SearchInput, Select, Toggle};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Rect;

const FIELD_USERNAME: usize = 0;
const FIELD_EMAIL: usize = 1;
const FIELD_PASSWORD: usize = 2;
const FIELD_THEME: usize = 3;
const FIELD_NOTIFICATIONS: usize = 4;
const FIELD_SUBMIT: usize = 5;
const FIELD_COUNT: usize = 6;

pub struct FormDemoScene {
    theme: Theme,
    show_help: bool,
    focused_field: usize,
    username: SearchInput,
    email: SearchInput,
    password: PasswordInput,
    theme_select: Select,
    notifications: Toggle,
    submit: Button,
    toast: Option<String>,
    area: std::cell::Cell<Rect>,
}

impl FormDemoScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            show_help: false,
            focused_field: 0,
            username: SearchInput::new(WidgetId::new(10)),
            email: SearchInput::new(WidgetId::new(11)),
            password: PasswordInput::new(WidgetId::new(12)),
            theme_select: Select::new(WidgetId::new(13))
                .with_options(vec!["Dark".into(), "Light".into(), "Cyberpunk".into()]),
            notifications: Toggle::new(WidgetId::new(14), "Enabled"),
            submit: Button::with_id(WidgetId::new(15), "Submit"),
            toast: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
        }
    }

    fn cycle_focus(&mut self, forward: bool) {
        if forward {
            self.focused_field = (self.focused_field + 1) % FIELD_COUNT;
        } else {
            self.focused_field = (self.focused_field + FIELD_COUNT - 1) % FIELD_COUNT;
        }
    }

    fn validate(&self) -> Option<String> {
        if self.username.query().is_empty() {
            return Some("Username required".into());
        }
        if self.email.query().is_empty() || !self.email.query().contains('@') {
            return Some("Valid email required".into());
        }
        if self.password.password().len() < 6 {
            return Some("Password must be 6+ chars".into());
        }
        None
    }

    fn submit(&mut self) {
        if let Some(err) = self.validate() {
            self.toast = Some(format!("Error: {}", err));
        } else {
            self.toast = Some("Settings saved!".into());
        }
    }

    fn cycle_theme(&mut self) {
        let themes = [
            Theme::nord(),
            Theme::cyberpunk(),
            Theme::dracula(),
            Theme::gruvbox_dark(),
            Theme::tokyo_night(),
        ];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.on_theme_change(&self.theme);
    }
}

impl Scene for FormDemoScene {
    fn scene_id(&self) -> &str { "form_demo" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = self.theme;
        let mut plane = Plane::new(0, area.width, area.height);
        for cell in plane.cells.iter_mut() {
            cell.bg = t.bg;
            cell.fg = t.fg;
            cell.transparent = false;
        }

        // Title
        draw_text(&mut plane, 2, 0, " Settings Form ", t.primary, t.bg, true);

        // Divider
        for x in 0..area.width {
            let idx = (area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }

        let labels = [
            ("󰀄 Username", FIELD_USERNAME),
            ("󰇰 Email", FIELD_EMAIL),
            ("󰌆 Password", FIELD_PASSWORD),
            ("󰔎 Theme", FIELD_THEME),
            ("󰂚 Notifications", FIELD_NOTIFICATIONS),
            ("", FIELD_SUBMIT),
        ];

        let start_y = 2u16;
        let field_h = 2u16;

        for (i, (label, field_id)) in labels.iter().enumerate() {
            let y = start_y + i as u16 * field_h;
            let is_focused = self.focused_field == *field_id;
            let row_bg = if is_focused { t.focus_bg } else { t.surface };

            // Row background
            for row in y..y + field_h {
                for col in 0..area.width {
                    let idx = (row * area.width + col) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = row_bg;
                    }
                }
            }

            if !label.is_empty() {
                draw_text(&mut plane, 2, y, label, t.primary, row_bg, false);
            }

            let widget_x = 20u16;
            let widget_w = area.width.saturating_sub(widget_x + 2);
            let widget_area = Rect::new(widget_x, y, widget_w, 1);

            if widget_area.width > 0 {
                let mut w_plane = match *field_id {
                    FIELD_USERNAME => self.username.render(widget_area),
                    FIELD_EMAIL => self.email.render(widget_area),
                    FIELD_PASSWORD => self.password.render(widget_area),
                    FIELD_THEME => self.theme_select.render(widget_area),
                    FIELD_NOTIFICATIONS => self.notifications.render(widget_area),
                    FIELD_SUBMIT => self.submit.render(widget_area),
                    _ => Plane::new(0, 0, 0),
                };
                blit_to(&mut plane, &mut w_plane, widget_x as usize, y as usize);
            }
        }

        // Toast
        if let Some(ref msg) = self.toast {
            let toast_y = area.height.saturating_sub(3);
            let toast_x = (area.width.saturating_sub(msg.len() as u16 + 4)) / 2;
            for x in toast_x..toast_x + msg.len() as u16 + 4 {
                let idx = (toast_y * area.width + x) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.success_bg;
                    plane.cells[idx].fg = t.success;
                }
            }
            draw_text(&mut plane, toast_x + 2, toast_y, msg, t.success, t.success_bg, true);
        }

        // Footer
        let footer_y = area.height.saturating_sub(1);
        for x in 0..area.width {
            let idx = (footer_y * area.width + x) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = '─';
                plane.cells[idx].fg = t.outline;
            }
        }
        let nav = " Tab: next | Shift+Tab: prev | Enter: submit | t: theme | B: back | ?: help ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.bg, false);

        if self.show_help {
            draw_help(&mut plane, area, t);
        }

        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }

        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                self.show_help = false;
            }
            return true;
        }

        // If toast is showing, any key dismisses it
        if self.toast.is_some() {
            self.toast = None;
            return true;
        }

        match key.code {
            KeyCode::Char('?') => { self.show_help = !self.show_help; true }
            KeyCode::Char('t') => { self.cycle_theme(); true }
            KeyCode::Tab => { self.cycle_focus(true); true }
            KeyCode::BackTab => { self.cycle_focus(false); true }
            KeyCode::Enter => {
                if self.focused_field == FIELD_SUBMIT {
                    self.submit();
                }
                true
            }
            KeyCode::Esc => {
                if self.toast.is_some() {
                    self.toast = None;
                    true
                } else {
                    false // Let parent (showcase) handle Esc → pop back
                }
            }
            _ => {
                // Delegate to focused field
                match self.focused_field {
                    FIELD_USERNAME => self.username.handle_key(key),
                    FIELD_EMAIL => self.email.handle_key(key),
                    FIELD_PASSWORD => self.password.handle_key(key),
                    FIELD_THEME => self.theme_select.handle_key(key),
                    FIELD_NOTIFICATIONS => self.notifications.handle_key(key),
                    FIELD_SUBMIT => self.submit.handle_key(key),
                    _ => false,
                }
            }
        }
    }

    fn handle_mouse(&mut self, _kind: dracon_terminal_engine::input::event::MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = *theme;
        self.username.on_theme_change(theme);
        self.email.on_theme_change(theme);
        self.password.on_theme_change(theme);
        self.theme_select.on_theme_change(theme);
        self.notifications.on_theme_change(theme);
        self.submit.on_theme_change(theme);
    }

    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
}

fn draw_text(plane: &mut Plane, x: u16, y: u16, text: &str, fg: Color, bg: Color, bold: bool) {
    for (i, ch) in text.chars().enumerate() {
        let idx = (y * plane.width + x + i as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell {
                char: ch, fg, bg,
                style: if bold { Styles::BOLD } else { Styles::empty() },
                transparent: false, skip: false,
            };
        }
    }
}

fn blit_to(dest: &mut Plane, src: &mut Plane, offset_x: usize, offset_y: usize) {
    for i in 0..src.cells.len() {
        let cell = &src.cells[i];
        if cell.char == '\0' || cell.transparent { continue; }
        let row = i / src.width as usize;
        let col = i % src.width as usize;
        let dy = offset_y + row;
        let dx = offset_x + col;
        if dy >= dest.height as usize || dx >= dest.width as usize { continue; }
        let idx = dy * dest.width as usize + dx;
        if idx < dest.cells.len() {
            dest.cells[idx] = cell.clone();
        }
    }
}

fn draw_help(plane: &mut Plane, area: Rect, t: Theme) {
    let hw = 42u16.min(area.width.saturating_sub(4));
    let hh = 10u16.min(area.height.saturating_sub(4));
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

    for x in hx..hx + hw {
        let top = (hy * area.width + x) as usize;
        let bot = ((hy + hh - 1) * area.width + x) as usize;
        if top < plane.cells.len() { plane.cells[top].char = '─'; plane.cells[top].fg = t.outline; }
        if bot < plane.cells.len() { plane.cells[bot].char = '─'; plane.cells[bot].fg = t.outline; }
    }
    for y in hy..hy + hh {
        let left = (y * area.width + hx) as usize;
        let right = (y * area.width + hx + hw - 1) as usize;
        if left < plane.cells.len() { plane.cells[left].char = '│'; plane.cells[left].fg = t.outline; }
        if right < plane.cells.len() { plane.cells[right].char = '│'; plane.cells[right].fg = t.outline; }
    }

    let title = "Form Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

    let shortcuts = [
        ("Tab", "Next field"),
        ("Shift+Tab", "Previous field"),
        ("Enter", "Submit form"),
        ("t", "Cycle theme"),
        ("B/Esc", "Back to showcase"),
        ("?", "Toggle help"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 16, row, desc, t.fg, t.surface_elevated, false);
    }
}
