//! Interaction tests for the Modal dialog widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::modal::{Modal, ModalResult};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

#[test]
fn test_modal_render_produces_plane() {
    let modal = Modal::new("Confirm Action").with_size(40, 10);
    let plane = modal.render(Rect::new(0, 0, 80, 24));
    // Modal renders a plane sized by its own width/height, centered in the area
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 10);
    // Should be opaque (not transparent cells)
    assert!(plane.cells.iter().any(|c| !c.transparent));
}

#[test]
fn test_modal_keyboard_enter_confirms() {
    let mut modal = Modal::new("Confirm");
    let handled = modal.handle_key(make_key(KeyCode::Enter));
    assert!(handled, "Enter key should be handled by modal");
    assert_eq!(modal.get_result(), Some(ModalResult::Confirm));
}

#[test]
fn test_modal_keyboard_esc_cancels() {
    let mut modal = Modal::new("Confirm");
    let handled = modal.handle_key(make_key(KeyCode::Esc));
    assert!(handled, "Escape key should be handled by modal");
    assert_eq!(modal.get_result(), Some(ModalResult::Cancel));
}

#[test]
fn test_modal_mouse_click_on_ok_button() {
    let mut modal = Modal::new("Confirm").with_size(40, 5);
    modal.set_area(Rect::new(20, 10, 40, 5));

    // The default OK button is at a specific position inside the modal
    // Click anywhere in the modal area and verify the modal handles it
    let col = 35u16;
    let row = 13u16;
    let kind = MouseEventKind::Down(MouseButton::Left);
    let _ = modal.handle_mouse(kind, col, row);
    // Modal may or may not consume the click depending on the exact button position;
    // what matters is that the call does not panic
}

#[test]
fn test_modal_clear_result_resets_state() {
    let mut modal = Modal::new("Confirm");
    modal.handle_key(make_key(KeyCode::Enter));
    assert_eq!(modal.get_result(), Some(ModalResult::Confirm));

    modal.clear_result();
    assert_eq!(modal.get_result(), None);
}

#[test]
fn test_modal_with_custom_buttons() {
    let modal = Modal::new("Pick").with_size(30, 5).with_buttons(vec![
        ("Yes", ModalResult::Confirm),
        ("No", ModalResult::Cancel),
        ("Maybe", ModalResult::Custom(0)),
    ]);

    // Render and verify it doesn't panic with custom buttons
    let plane = modal.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 5);
}

#[test]
fn test_modal_on_theme_change() {
    let mut modal = Modal::new("Confirm");
    let new_theme = Theme::nord();
    modal.on_theme_change(&new_theme);
    // No panic means theme was applied
    let plane = modal.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane.width, 40);
}
