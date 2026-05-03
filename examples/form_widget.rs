#![allow(missing_docs)]
//! Form Widget Example — demonstrates the Form builder widget.
//!
//! Shows labeled input fields with focus cycling and value entry.
//!
//! Controls: Tab/Shift+Tab to cycle fields, type to enter values, q to quit.

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
