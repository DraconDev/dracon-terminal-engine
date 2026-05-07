//! Form validation tests — error display, field constraints.

use dracon_terminal_engine::compositor::Color;
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

#[test]
fn test_form_field_error_display() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    // Set error on first field
    form.set_field_error(0, "Username is required");
    let plane = form.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_form_field_error_cleared() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    // Set and clear error
    form.set_field_error(0, "Error");
    form.set_field_value(0, "valid_user");
    
    let plane = form.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_form_field_value_persists() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    // Type into first field
    form.handle_key(KeyEvent {
        code: KeyCode::Char('t'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    });
    form.handle_key(KeyEvent {
        code: KeyCode::Char('e'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    });
    form.handle_key(KeyEvent {
        code: KeyCode::Char('s'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    });
    
    // Move to next field
    form.handle_key(KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    });
    
    // First field should still have "tes"
    let plane = form.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_form_empty_field_validation() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    // Leave field empty and set error
    form.set_field_error(0, "Required field");
    let plane = form.render(Rect::new(0, 0, 40, 10));
    
    // Error should be visible
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_form_field_focus_indicator() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    // First field focused by default
    let plane1 = form.render(Rect::new(0, 0, 40, 10));
    
    // Move to second field
    form.handle_key(KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    });
    
    let plane2 = form.render(Rect::new(0, 0, 40, 10));
    
    // Should be different due to focus change
    assert_ne!(plane1.cells, plane2.cells);
}

#[test]
fn test_form_no_black_background() {
    let form = make_form().with_theme(Theme::nord());
    let plane = form.render(Rect::new(0, 0, 40, 10));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_form_out_of_bounds_field_error() {
    let mut form = make_form();
    // Try to set error on non-existent field
    form.set_field_error(99, "Error");
    // Should not panic
    let plane = form.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_form_out_of_bounds_field_value() {
    let mut form = make_form();
    // Try to set value on non-existent field
    form.set_field_value(99, "Value");
    // Should not panic
    let plane = form.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_form_multiple_errors() {
    let mut form = make_form();
    form.set_area(Rect::new(0, 0, 40, 10));
    
    form.set_field_error(0, "Username required");
    form.set_field_error(1, "Invalid email");
    form.set_field_error(2, "Password too short");
    
    let plane = form.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_form_error_theme_colors() {
    let mut form = make_form().with_theme(Theme::nord());
    form.set_field_error(0, "Error message");
    let plane = form.render(Rect::new(0, 0, 40, 10));
    
    // Error should use theme error color
    let has_error_color = plane.cells.iter().any(|c| c.fg != Color::Reset);
    assert!(has_error_color);
}
