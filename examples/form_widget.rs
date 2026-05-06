#![allow(missing_docs)]
//! Form Widget Example — demonstrates the Form builder widget.
//!
//! Shows labeled input fields with focus cycling and value entry.
//!
//! Controls: Tab/Shift+Tab to cycle fields, type to enter values, t to cycle theme, ? for help, q to quit.

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::Form;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct FormApp {
    form: Rc<RefCell<Form>>,
    should_quit: Arc<AtomicBool>,
    theme: Theme,
    show_help: bool,
}

impl FormApp {
    fn new(should_quit: Arc<AtomicBool>, theme: Theme) -> Self {
        let form = Form::new(WidgetId::new(1))
            .with_theme(theme)
            .add_field("Username")
            .add_field("Email")
            .add_field("Password")
            .add_field("Confirm Password");
        Self {
            form: Rc::new(RefCell::new(form)),
            should_quit,
            theme,
            show_help: false,
        }
    }

    fn cycle_theme(&mut self) {
        let themes = [Theme::nord(), Theme::cyberpunk(), Theme::dracula()];
        let idx = themes.iter().position(|t| t.name == self.theme.name).unwrap_or(0);
        self.theme = themes[(idx + 1) % themes.len()];
        self.form.borrow_mut().on_theme_change(&self.theme);
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let t = &self.theme;
        let help_text = [
            "┌─ Controls ────────────────┐",
            "│ ↑/↓    Navigate fields   │",
            "│ Type    Enter value     │",
            "│ Home    Clear field     │",
            "│ t       Cycle theme     │",
            "│ ?       Toggle help     │",
            "│ q       Quit            │",
            "└─────────────────────────┘",
        ];
        let w = 30.min(area.width.saturating_sub(4));
        let h = help_text.len() as u16 + 2;
        let x = (area.width - w) / 2;
        let y = (area.height - h) / 2;

        for row in 0..h {
            for col in 0..w {
                let idx = ((y + row) * plane.width + x + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx].bg = t.surface_elevated;
                    plane.cells[idx].fg = t.fg;
                    plane.cells[idx].transparent = false;
                }
            }
        }

        let corners = [(y, x, '╭'), (y, x + w - 1, '╮'), (y + h - 1, x, '╰'), (y + h - 1, x + w - 1, '╯')];
        for (cy, cx, ch) in &corners {
            let idx = (*cy * plane.width + *cx) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = *ch;
                plane.cells[idx].fg = t.outline;
            }
        }

        for col in 1..w - 1 {
            let top_idx = (y * plane.width + x + col) as usize;
            let bot_idx = ((y + h - 1) * plane.width + x + col) as usize;
            if top_idx < plane.cells.len() { plane.cells[top_idx].char = '─'; plane.cells[top_idx].fg = t.outline; }
            if bot_idx < plane.cells.len() { plane.cells[bot_idx].char = '─'; plane.cells[bot_idx].fg = t.outline; }
        }

        for row in 1..h - 1 {
            let left_idx = ((y + row) * plane.width + x) as usize;
            let right_idx = ((y + row) * plane.width + x + w - 1) as usize;
            if left_idx < plane.cells.len() { plane.cells[left_idx].char = '│'; plane.cells[left_idx].fg = t.outline; }
            if right_idx < plane.cells.len() { plane.cells[right_idx].char = '│'; plane.cells[right_idx].fg = t.outline; }
        }

        let start_y = y + (h - help_text.len() as u16) / 2;
        for (i, line) in help_text.iter().enumerate() {
            let row = start_y + i as u16;
            for (j, c) in line.chars().enumerate() {
                let idx = (row * plane.width + x + 2 + j as u16) as usize;
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
        WidgetId::new(0)
    }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect {
        self.form.borrow().area()
    }
    fn set_area(&mut self, area: Rect) {
        self.form.borrow_mut().set_area(area);
    }
    fn z_index(&self) -> u16 {
        0
    }
    fn needs_render(&self) -> bool {
        true
    }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool {
        true
    }
    fn render(&self, area: Rect) -> Plane {
        let mut plane = self.form.borrow().render(area);

        // Status bar at bottom
        let status_y = area.height.saturating_sub(1);
        let hint = "Tab: next | t: theme | ?: help | q: quit";
        for (i, c) in hint.chars().take((area.width as usize).saturating_sub(2)).enumerate() {
            let idx = (status_y * plane.width + 2 + i as u16) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = c;
                plane.cells[idx].fg = self.theme.fg_muted;
                plane.cells[idx].bg = self.theme.surface;
            }
        }

        if self.show_help {
            self.render_help_overlay(&mut plane, area);
        }
        plane
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit.store(true, Ordering::SeqCst);
                true
            }
            KeyCode::Char('t') => {
                self.cycle_theme();
                true
            }
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                true
            }
            KeyCode::Esc => {
                if self.show_help {
                    self.show_help = false;
                    true
                } else {
                    false
                }
            }
            _ => self.form.borrow_mut().handle_key(key),
        }
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::nord();
    let app = Rc::new(RefCell::new(FormApp::new(should_quit, theme)));
    let app_for_input = Rc::clone(&app);

    let mut app_widget = App::new()?.title("Form Widget Demo").fps(30).theme(theme);

    app_widget.add_widget(
        Box::new(FormApp::new(quit_check, theme)),
        Rect::new(0, 0, 80, 24),
    );

    app_widget
        .on_input(move |key| app_for_input.borrow_mut().handle_key(key))
        .run(|_ctx| {})
}
