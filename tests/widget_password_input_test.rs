mod common;

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::password_input::PasswordInput;
use ratatui::layout::Rect;

#[test]
fn test_password_input_new() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id);
    assert_eq!(input.id, id);
    assert_eq!(input.base.placeholder, "Password...");
    assert_eq!(input.base.mask_char, Some('*'));
}

#[test]
fn test_password_input_with_theme() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id).with_theme(Theme::default());
    assert!(!input.base.theme.name.is_empty());
}

#[test]
fn test_password_input_mask_char() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id).with_mask_char('X');
    assert_eq!(input.base.mask_char, Some('X'));
}

#[test]
fn test_password_input_placeholder() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id).with_placeholder("Enter password");
    assert_eq!(input.base.placeholder, "Enter password");
}

#[test]
fn test_password_input_on_submit() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id).on_submit(|text| {
        assert_eq!(text, "secret123");
    });
    assert!(input.base.on_submit.is_some());
}

#[test]
fn test_password_input_clear() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    input.base.text = "secret".to_string();
    input.base.cursor_pos = 6;
    input.clear();
    assert!(input.base.text.is_empty());
    assert_eq!(input.base.cursor_pos, 0);
}

#[test]
fn test_password_input_password() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    input.base.text = "mypassword".to_string();
    assert_eq!(input.password(), "mypassword");
}

#[test]
fn test_password_input_widget_id() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(42);
    let input = PasswordInput::new(id);
    assert_eq!(input.id(), id);
}

#[test]
fn test_password_input_set_id() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    let new_id = dracon_terminal_engine::framework::widget::WidgetId::new(99);
    input.set_id(new_id);
    assert_eq!(input.id, new_id);
}

#[test]
fn test_password_input_area() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id);
    input.base.area.set(Rect::new(10, 5, 30, 1));
    let area = input.area();
    assert_eq!(area.x, 10);
    assert_eq!(area.y, 5);
    assert_eq!(area.width, 30);
    assert_eq!(area.height, 1);
}

#[test]
fn test_password_input_set_area() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    input.set_area(Rect::new(20, 10, 40, 2));
    let area = input.area();
    assert_eq!(area.x, 20);
    assert_eq!(area.y, 10);
    assert_eq!(area.width, 40);
    assert_eq!(area.height, 2);
}

#[test]
fn test_password_input_needs_render() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id);
    assert!(input.needs_render());
}

#[test]
fn test_password_input_mark_dirty() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    input.base.dirty = false;
    input.mark_dirty();
    assert!(input.base.dirty);
}

#[test]
fn test_password_input_clear_dirty() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    input.base.dirty = true;
    input.clear_dirty();
    assert!(!input.base.dirty);
}

#[test]
fn test_password_input_render() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id);
    let plane = input.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_password_input_cursor_position() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let input = PasswordInput::new(id);
    input.base.area.set(Rect::new(5, 3, 20, 1));
    let pos = input.cursor_position();
    assert!(pos.is_some());
    let (x, y) = pos.unwrap();
    assert_eq!(x, 5);
    assert_eq!(y, 3);
}

#[test]
fn test_password_input_handle_key_char() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Char('s'),
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = input.handle_key(key);
    assert!(result);
    assert_eq!(input.base.text, "s");
}

#[test]
fn test_password_input_handle_key_backspace() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    input.base.text = "pass".to_string();
    input.base.cursor_pos = 4;
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Backspace,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = input.handle_key(key);
    assert!(result);
    assert_eq!(input.base.text, "pas");
}

#[test]
fn test_password_input_handle_key_enter_triggers_callback() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    input.base.text = "secret".to_string();
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Enter,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = input.handle_key(key);
    assert!(result);
}

#[test]
fn test_password_input_handle_mouse() {
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(1);
    let mut input = PasswordInput::new(id);
    input.base.text = "password".to_string();
    input.base.cursor_pos = 0;
    let result = input.handle_mouse(
        dracon_terminal_engine::input::event::MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left),
        4,
        0,
    );
    assert!(result);
    assert_eq!(input.base.cursor_pos, 4);
}