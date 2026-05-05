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
        self.theme = themes[(idx + 1) % themes.len()].clone();
        self.form.borrow_mut().on_theme_change(&self.theme);
    }

    fn render_help_overlay(&self, plane: &mut Plane, area: Rect) {
        let help_text = [
            "─ Controls ─",
            "",
            "↑/↓    Navigate fields",
            "Type    Enter value",
            "Home    Clear field",
            "t       Cycle theme",
            "?       Toggle help",
            "q       Quit",
        ];

        let w = 30.min(area.width.saturating_sub(4));
        let h = help_text.len() as u16 + 2;
        let x = (area.width - w) / 2;
        let y = (area.height - h) / 2;

        // Draw rounded box (transparent background for proper overlay effect)
        // Top-left corner
        plane.put_str(x, y, "╭", self.theme.fg, self.theme.bg, Styles::empty());
        // Top line
        for i in 1..w - 1 {
            plane.put_str(x + i, y, "─", self.theme.fg, self.theme.bg, Styles::empty());
        }
        // Top-right corner
        plane.put_str(x + w - 1, y, "╮", self.theme.fg, self.theme.bg, Styles::empty());

        // Middle rows
        for row in 0..h - 2 {
            plane.put_str(x, y + 1 + row, "│", self.theme.fg, self.theme.bg, Styles::empty());
            plane.put_str(x + w - 1, y + 1 + row, "│", self.theme.fg, self.theme.bg, Styles::empty());
        }

        // Bottom-left corner
        plane.put_str(x, y + h - 1, "╰", self.theme.fg, self.theme.bg, Styles::empty());
        // Bottom line
        for i in 1..w - 1 {
            plane.put_str(x + i, y + h - 1, "─", self.theme.fg, self.theme.bg, Styles::empty());
        }
        // Bottom-right corner
        plane.put_str(x + w - 1, y + h - 1, "╯", self.theme.fg, self.theme.bg, Styles::empty());

        // Draw help text
        for (i, line) in help_text.iter().enumerate() {
            let text_y = y + 1 + i as u16;
            let text_x = x + 2;
            let available = w - 4;
            for (j, c) in line.chars().take(available as usize).enumerate() {
                plane.put_str(text_x + j as u16, text_y, &c.to_string(), self.theme.fg, self.theme.bg, Styles::empty());
            }
        }
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
        self.form.borrow().render(area)
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
