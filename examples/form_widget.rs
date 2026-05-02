#![allow(missing_docs)]
//! Form Widget Example — demonstrates the Form builder widget.
//!
//! Shows labeled input fields with validation, focus cycling, and error display.
//!
//! Controls: Tab/Shift+Tab to cycle fields, type to enter values, Enter to validate.

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::Form;
use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct FormApp {
    form: Form,
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
        Self { form, should_quit }
    }
}

impl Widget for FormApp {
    fn id(&self) -> WidgetId { WidgetId::new(0) }
    fn set_id(&mut self, _id: WidgetId) {}
    fn area(&self) -> Rect { self.form.area() }
    fn set_area(&mut self, area: Rect) { self.form.set_area(area); }
    fn z_index(&self) -> u16 { 0 }
    fn needs_render(&self) -> bool { true }
    fn mark_dirty(&mut self) {}
    fn clear_dirty(&mut self) {}
    fn focusable(&self) -> bool { true }
    fn render(&self, area: Rect) -> Plane { self.form.render(area) }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press { return false; }
        match key.code {
            KeyCode::Char('q') => { self.should_quit.store(true, Ordering::SeqCst); true }
            _ => self.form.handle_key(key),
        }
    }
}

fn main() -> std::io::Result<()> {
    let should_quit = Arc::new(AtomicBool::new(false));
    let quit_check = Arc::clone(&should_quit);

    let theme = Theme::nord();
    let app = FormApp::new(should_quit, theme);

    App::new()?
        .title("Form Widget Demo")
        .fps(30)
        .theme(theme)
        .on_tick(move |ctx, _| {
            if quit_check.load(Ordering::SeqCst) { ctx.stop(); }
        })
        .run(|ctx| {
            let (w, h) = ctx.compositor().size();
            ctx.add_plane(app.render(Rect::new(0, 0, w, h)));
        })
}
