//! Embedded Form Demo scene for the showcase.
//!
//! Demonstrates form fields with validation and submit.
//! Press `Tab` to cycle focus, `B`/`Esc` to go back.

use crate::scenes::shared_helpers::{blit_to, draw_text};
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::keybindings::{resolve_keybindings, KeybindingSet, actions};
use dracon_terminal_engine::framework::scene_router::Scene;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, PasswordInput, SearchInput, Select, Toggle};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};
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
    dirty: bool,
    focused_field: usize,
    field_order: [usize; FIELD_COUNT],
    dragging: Option<usize>,
    drag_hover: Option<usize>,
    username: SearchInput,
    email: SearchInput,
    password: PasswordInput,
    theme_select: Select,
    notifications: Toggle,
    submit: Button,
    toast: Option<String>,
    area: std::cell::Cell<Rect>,
    keybindings: KeybindingSet,
}

impl FormDemoScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            show_help: false,
            dirty: true,
            focused_field: 0,
            field_order: [FIELD_USERNAME, FIELD_EMAIL, FIELD_PASSWORD, FIELD_THEME, FIELD_NOTIFICATIONS, FIELD_SUBMIT],
            dragging: None,
            drag_hover: None,
            username: SearchInput::new(WidgetId::new(10)),
            email: SearchInput::new(WidgetId::new(11)),
            password: PasswordInput::new(WidgetId::new(12)),
            theme_select: Select::new(WidgetId::new(13))
                .with_options(vec!["Dark".into(), "Light".into(), "Cyberpunk".into()]),
            notifications: Toggle::new(WidgetId::new(14), "Enabled"),
            submit: Button::with_id(WidgetId::new(15), "Submit"),
            toast: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
            keybindings: KeybindingSet::from_config(&resolve_keybindings()),
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

}

impl Scene for FormDemoScene {
    fn scene_id(&self) -> &str { "form_demo" }

    fn render(&self, area: Rect) -> Plane {
        self.area.set(area);
        let t = &self.theme;
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
            ("Usr Username", FIELD_USERNAME),
            ("@ Email", FIELD_EMAIL),
            ("Pwd Password", FIELD_PASSWORD),
            ("Thm Theme", FIELD_THEME),
            ("Bell Notifications", FIELD_NOTIFICATIONS),
            ("", FIELD_SUBMIT),
        ];

        let start_y = 2u16;
        let field_h = 2u16;

        // Build position map: field_id -> visual row index
        let mut field_to_row = [0usize; FIELD_COUNT];
        for (row_idx, &field_id) in self.field_order.iter().enumerate() {
            field_to_row[field_id] = row_idx;
        }

        for (row_idx, &field_id) in self.field_order.iter().enumerate() {
            let y = start_y + row_idx as u16 * field_h;
            let is_focused = self.focused_field == field_id;
            let is_dragged = self.dragging == Some(row_idx);
            let is_hover_target = self.drag_hover == Some(row_idx) && self.dragging.is_some();

            let row_bg = if is_dragged {
                t.selection_bg
            } else if is_hover_target {
                t.primary_hover
            } else if is_focused {
                t.focus_bg
            } else {
                t.surface
            };

            // Row background
            for row in y..y + field_h {
                for col in 0..area.width {
                    let idx = (row * area.width + col) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx].bg = row_bg;
                    }
                }
            }

            // Drag handle indicator on left
            if !is_dragged {
                let handle = if is_hover_target { ">" } else { "=" };
                draw_text(&mut plane, 0, y, handle, t.fg_muted, row_bg, false);
            }

            let label = labels[field_id].0;
            if !label.is_empty() {
                draw_text(&mut plane, 2, y, label, t.primary, row_bg, false);
            }

            let widget_x = 20u16;
            let widget_w = area.width.saturating_sub(widget_x + 2);
            let widget_area = Rect::new(widget_x, y, widget_w, 1);

            if widget_area.width > 0 {
                let mut w_plane = match field_id {
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
        let nav = " Tab: next | Enter: submit | Drag =: reorder | B/Esc: back | ?: help ";
        draw_text(&mut plane, 2, footer_y, nav, t.fg_muted, t.bg, false);

        if self.show_help {
            draw_help(&mut plane, area, t);
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

        // If toast is showing, any key dismisses it
        if self.toast.is_some() {
            self.toast = None;
            self.dirty = true;
            return true;
        }

        if self.keybindings.matches(actions::HELP, &key) {
            self.show_help = !self.show_help;
            self.dirty = true;
            return true;
        }
        if self.keybindings.matches(actions::BACK, &key) {
            return false; // Let parent (showcase) handle back > pop scene
        }

        match key.code {
            KeyCode::Tab => { self.cycle_focus(true); self.dirty = true; true }
            KeyCode::BackTab => { self.cycle_focus(false); self.dirty = true; true }
            KeyCode::Enter => {
                if self.focused_field == FIELD_SUBMIT {
                    self.submit();
                }
                self.dirty = true;
                true
            }
            _ => {
                // Delegate to focused field
                let handled = match self.focused_field {
                    FIELD_USERNAME => self.username.handle_key(key),
                    FIELD_EMAIL => self.email.handle_key(key),
                    FIELD_PASSWORD => self.password.handle_key(key),
                    FIELD_THEME => self.theme_select.handle_key(key),
                    FIELD_NOTIFICATIONS => self.notifications.handle_key(key),
                    FIELD_SUBMIT => self.submit.handle_key(key),
                    _ => false,
                };
                if handled { self.dirty = true; }
                handled
            }
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.show_help {
            return true;
        }

        let start_y = 2u16;
        let field_h = 2u16;

        // Calculate which row the mouse is over (if any)
        let row_idx = if row >= start_y && row < start_y + FIELD_COUNT as u16 * field_h {
            Some(((row - start_y) / field_h) as usize)
        } else {
            None
        };

        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(idx) = row_idx {
                    if col < 2 && self.dragging.is_none() {
                        // Start drag on handle
                        self.dragging = Some(idx);
                        self.drag_hover = Some(idx);
                        self.dirty = true;
                        return true;
                    }
                    if self.dragging.is_some() {
                        // Complete drag
                        if let Some(drag_idx) = self.dragging {
                            if let Some(hover_idx) = self.drag_hover {
                                if drag_idx != hover_idx {
                                    // Swap fields in field_order
                                    self.field_order.swap(drag_idx, hover_idx);
                                }
                            }
                        }
                        self.dragging = None;
                        self.drag_hover = None;
                        self.dirty = true;
                        return true;
                    }
                    // Normal click: focus field or submit
                    let field_id = self.field_order[idx];
                    if field_id == FIELD_SUBMIT {
                        self.submit();
                    } else {
                        self.focused_field = field_id;
                    }
                    self.dirty = true;
                    return true;
                }
                false
            }
            MouseEventKind::Moved if self.dragging.is_some() => {
                if let Some(idx) = row_idx {
                    if self.drag_hover != Some(idx) {
                        self.drag_hover = Some(idx);
                        self.dirty = true;
                    }
                }
                true
            }
            MouseEventKind::Moved => false,
            MouseEventKind::Down(MouseButton::Right) if self.dragging.is_some() => {
                self.dragging = None;
                self.drag_hover = None;
                self.dirty = true;
                true
            }
            _ => false,
        }
    }

    fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = theme.clone();
        self.username.on_theme_change(theme);
        self.email.on_theme_change(theme);
        self.password.on_theme_change(theme);
        self.theme_select.on_theme_change(theme);
        self.notifications.on_theme_change(theme);
        self.submit.on_theme_change(theme);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool { self.dirty }
    fn mark_dirty(&mut self) { self.dirty = true; }
    fn clear_dirty(&mut self) { self.dirty = false; }
}

fn draw_help(plane: &mut Plane, area: Rect, t: &Theme) {
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
    // Rounded corners
    let corners = [('╭', hx, hy), ('╮', hx + hw - 1, hy), ('╰', hx, hy + hh - 1), ('╯', hx + hw - 1, hy + hh - 1)];
    for (ch, cx, cy) in corners {
        let idx = (cy * area.width + cx) as usize;
        if idx < plane.cells.len() { plane.cells[idx].char = ch; plane.cells[idx].fg = t.outline; }
    }

    let title = "Form Help";
    let tx = hx + (hw - title.len() as u16) / 2;
    draw_text(plane, tx, hy + 1, title, t.primary, t.surface_elevated, true);

    let shortcuts = [
        ("Tab", "Next field"),
        ("Shift+Tab", "Previous field"),
        ("Enter", "Submit form"),
        ("Drag =", "Reorder fields"),
        ("B/Esc", "Back to showcase"),
        ("?", "Toggle help"),
    ];
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = hy + 3 + i as u16;
        draw_text(plane, hx + 2, row, key, t.primary, t.surface_elevated, false);
        draw_text(plane, hx + 16, row, desc, t.fg, t.surface_elevated, false);
    }
}
