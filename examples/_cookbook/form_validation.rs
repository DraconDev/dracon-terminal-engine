use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::rc::Rc;

fn main() -> std::io::Result<()> {
    let should_quit = Rc::new(AtomicBool::new(false));
    let quit = Rc::clone(&should_quit);
    let theme = Theme::from_env_or(Theme::nord());

    let form = Form::new(WidgetId::new(1))
        .add_field("Username")
        .add_field("Email")
        .add_field("Password")
        .add_field("Bio")
        .with_theme(theme)
        .with_validation(0, vec![
            ValidationRule::Required,
            ValidationRule::MinLength(3),
            ValidationRule::MaxLength(20),
        ])
        .with_validation(1, vec![
            ValidationRule::Required,
            ValidationRule::Email,
        ])
        .with_validation(2, vec![
            ValidationRule::Required,
            ValidationRule::MinLength(8),
        ])
        .with_validation(3, vec![
            ValidationRule::MaxLength(200),
        ]);

    App::new()?
        .title("Form Validation Demo")
        .fps(30)
        .theme(theme)
        .add_widget(Box::new(form), Rect::new(2, 2, 50, 18))
        .on_input(move |key| {
            use dracon_terminal_engine::input::event::{KeyCode, KeyModifiers};
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                quit.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        })
        .run(move |ctx| {
            if should_quit.load(Ordering::SeqCst) {
                ctx.stop();
            }
        })
}
