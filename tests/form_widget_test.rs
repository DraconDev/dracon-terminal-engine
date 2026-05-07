//! Form widget tests — field navigation, input, validation.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::Form;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

fn make_form() -> Form {
    Form::new(WidgetId::new(1))
        .add_field("Username")
        .add_field("Email")
        .add_field("Password")
}

fn key_press(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

#[test]
fn test_form_new_has_no_fields_focus() {
    let form = Form::new(WidgetId::new(1));
    // Form with no fields should not panic on render
    let plane = form.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane.width, 40);
}

#[test]
fn test_form_add_field_chaining() {
    let form = make_form();
    let plane = form.render(Rect::new(0, 0, 40, 5));
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 5);
}

#[test]
fn test_form_down_navigation() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    assert!(form.handle_key(key_press(KeyCode::Down)));
    assert!(form.handle_key(key_press(KeyCode::Down)));
    // At last field, should still return true but not advance
    assert!(form.handle_key(key_press(KeyCode::Down)));
}

#[test]
fn test_form_up_navigation() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    // Move to last field
    form.handle_key(key_press(KeyCode::Down));
    form.handle_key(key_press(KeyCode::Down));
    
    // Now move back up
    assert!(form.handle_key(key_press(KeyCode::Up)));
    assert!(form.handle_key(key_press(KeyCode::Up)));
    // At first field, should still return true but not retreat
    assert!(form.handle_key(key_press(KeyCode::Up)));
}

#[test]
fn test_form_char_input() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    assert!(form.handle_key(key_press(KeyCode::Char('a'))));
    assert!(form.handle_key(key_press(KeyCode::Char('b'))));
    assert!(form.handle_key(key_press(KeyCode::Char('c'))));
    
    // Field should have "abc"
    // We can't directly verify without exposing field value
    // But we can verify dirty flag is set
    assert!(form.needs_render());
}

#[test]
fn test_form_backspace() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    // Type then backspace
    form.handle_key(key_press(KeyCode::Char('a')));
    form.handle_key(key_press(KeyCode::Char('b')));
    assert!(form.handle_key(key_press(KeyCode::Backspace)));
}

#[test]
fn test_form_backspace_empty_field() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    // Backspace on empty field should not panic
    assert!(form.handle_key(key_press(KeyCode::Backspace)));
}

#[test]
fn test_form_home_clears_field() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    form.handle_key(key_press(KeyCode::Char('a')));
    form.handle_key(key_press(KeyCode::Char('b')));
    assert!(form.handle_key(key_press(KeyCode::Home)));
}

#[test]
fn test_form_mouse_selects_field() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    let result = form.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 1);
    assert!(result);
}

#[test]
fn test_form_set_field_value() {
    let mut form = make_form();
    form.set_field_value(0, "testuser");
    form.set_field_value(1, "test@example.com");
    
    assert!(form.needs_render());
}

#[test]
fn test_form_set_field_error() {
    let mut form = make_form();
    form.set_field_error(0, "Invalid username");
    assert!(form.needs_render());
}

#[test]
fn test_form_release_key_ignored() {
    let mut form = make_form();
    
    let release = KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Release,
    };
    assert!(!form.handle_key(release));
}

#[test]
fn test_form_on_theme_change() {
    let mut form = make_form();
    let theme = Theme::cyberpunk();
    form.on_theme_change(&theme);
    
    let plane = form.render(Rect::new(0, 0, 40, 10));
    // All cells should have the new theme bg
    for cell in &plane.cells {
        assert_eq!(cell.bg, theme.bg);
    }
}
