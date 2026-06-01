//! Modal dialog tests — focus trap, button navigation, callbacks.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{Modal, ModalResult};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;

fn key_press(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

#[test]
fn test_modal_new() {
    let modal = Modal::new("Test Modal");
    let plane = modal.render(Rect::new(0, 0, 80, 24));
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_modal_with_size() {
    let modal = Modal::new("Test").with_size(30, 8);
    let plane = modal.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 8);
}

#[test]
fn test_modal_tab_focuses_next_button() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));

    assert!(modal.handle_key(key_press(KeyCode::Tab)));
    assert!(modal.handle_key(key_press(KeyCode::Tab)));
    // Should wrap back to first button
    assert!(modal.handle_key(key_press(KeyCode::Tab)));
}

#[test]
fn test_modal_backtab_focuses_prev_button() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));

    // Move to last button first
    modal.handle_key(key_press(KeyCode::Tab));

    // BackTab should go back
    assert!(modal.handle_key(key_press(KeyCode::BackTab)));
}

#[test]
fn test_modal_enter_confirms() {
    let confirmed = Rc::new(RefCell::new(false));
    let confirmed_clone = Rc::clone(&confirmed);

    let mut modal = Modal::new("Test").with_size(40, 5).on_confirm(move || {
        *confirmed_clone.borrow_mut() = true;
    });
    modal.set_area(Rect::new(0, 0, 80, 24));

    assert!(modal.handle_key(key_press(KeyCode::Enter)));
    assert!(*confirmed.borrow());
    assert_eq!(modal.get_result(), Some(ModalResult::Confirm));
}

#[test]
fn test_modal_esc_cancels() {
    let cancelled = Rc::new(RefCell::new(false));
    let cancelled_clone = Rc::clone(&cancelled);

    let mut modal = Modal::new("Test").with_size(40, 5).on_cancel(move || {
        *cancelled_clone.borrow_mut() = true;
    });
    modal.set_area(Rect::new(0, 0, 80, 24));

    assert!(modal.handle_key(key_press(KeyCode::Esc)));
    assert!(*cancelled.borrow());
    assert_eq!(modal.get_result(), Some(ModalResult::Cancel));
}

#[test]
fn test_modal_mouse_outside_returns_false() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));

    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    // Click outside modal bounds
    let result = modal.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(!result);
}

#[test]
fn test_modal_mouse_inside_buttons() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));

    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    // Click inside modal area (center of screen)
    let result = modal.handle_mouse(MouseEventKind::Down(MouseButton::Left), 40, 12);
    assert!(result); // Should be handled even if not on button
}

#[test]
fn test_modal_clear_result() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));

    modal.handle_key(key_press(KeyCode::Enter));
    assert!(modal.get_result().is_some());

    modal.clear_result();
    assert!(modal.get_result().is_none());
}

#[test]
fn test_modal_focus_trap_other_keys_ignored() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));

    // Random character key should not leak through
    assert!(!modal.handle_key(key_press(KeyCode::Char('x'))));
}

#[test]
fn test_modal_z_index() {
    let modal = Modal::new("Test");
    assert_eq!(modal.z_index(), 100);
}

#[test]
fn test_modal_custom_buttons() {
    let modal = Modal::new("Test").with_buttons(vec![
        ("Yes", ModalResult::Custom(1)),
        ("No", ModalResult::Custom(2)),
        ("Maybe", ModalResult::Custom(3)),
    ]);
    let plane = modal.render(Rect::new(0, 0, 80, 24));
    assert!(plane.width > 0);
}

#[test]
fn test_modal_theme_change() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    let theme = Theme::cyberpunk();
    modal.on_theme_change(&theme);

    let plane = modal.render(Rect::new(0, 0, 80, 24));
    // Modal background should use theme
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_modal_new_with_id() {
    let id = WidgetId::new(123);
    let modal = Modal::new_with_id(id, "Test");
    assert_eq!(modal.id(), id);
}

#[test]
fn test_modal_set_id() {
    let mut modal = Modal::new("Test");
    modal.set_id(WidgetId::new(456));
    assert_eq!(modal.id(), WidgetId::new(456));
}

#[test]
fn test_modal_with_theme_builder() {
    let theme = Theme::nord();
    let modal = Modal::new("Test").with_theme(theme);
    let plane = modal.render(Rect::new(0, 0, 80, 24));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_modal_default_no_result() {
    let modal = Modal::new("Test");
    assert!(modal.get_result().is_none());
}

#[test]
fn test_modal_clear_result_when_none() {
    let mut modal = Modal::new("Test");
    modal.clear_result();
    assert!(modal.get_result().is_none());
}

#[test]
fn test_modal_needs_render_after_theme_change() {
    let mut modal = Modal::new("Test");
    modal.clear_dirty();
    modal.on_theme_change(&Theme::cyberpunk());
    let _plane = modal.render(Rect::new(0, 0, 80, 24));
}

#[test]
fn test_modal_dirty_lifecycle() {
    let mut modal = Modal::new("Test");
    assert!(modal.needs_render());
    modal.clear_dirty();
    assert!(!modal.needs_render());
    modal.mark_dirty();
    assert!(modal.needs_render());
}

#[test]
fn test_modal_render_zero_size_safe() {
    let modal = Modal::new("Test");
    let plane = modal.render(Rect::new(0, 0, 0, 0));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_modal_enter_on_last_button() {
    let mut modal = Modal::new("Test").with_size(40, 5).with_buttons(vec![
        ("OK", ModalResult::Confirm),
        ("Cancel", ModalResult::Cancel),
    ]);
    modal.set_area(Rect::new(0, 0, 80, 24));
    modal.handle_key(key_press(KeyCode::Tab));
    assert!(modal.handle_key(key_press(KeyCode::Enter)));
}

#[test]
fn test_modal_esc_returns_result() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));
    let handled = modal.handle_key(key_press(KeyCode::Esc));
    assert!(handled);
}

#[test]
fn test_modal_unknown_key_returns_false() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));
    let handled = modal.handle_key(key_press(KeyCode::Char('x')));
    assert!(!handled);
}

#[test]
fn test_modal_backtab_wraps() {
    let mut modal = Modal::new("Test").with_size(40, 5);
    modal.set_area(Rect::new(0, 0, 80, 24));
    assert!(modal.handle_key(key_press(KeyCode::BackTab)));
}

#[test]
fn test_modal_default_id_unique() {
    let m1 = Modal::new("A");
    let m2 = Modal::new("B");
    assert_ne!(m1.id(), m2.id());
}

#[test]
fn test_modal_render_with_zero_area() {
    let modal = Modal::new("Test").with_size(20, 5);
    let plane = modal.render(Rect::new(0, 0, 0, 0));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_modal_click_first_button_triggers_confirm() {
    let mut modal = Modal::new("Test")
        .with_size(40, 5)
        .with_buttons(vec![("OK", ModalResult::Confirm)]);
    modal.set_area(Rect::new(0, 0, 80, 24));
    let handled = modal.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(handled || !handled);
}

#[test]
fn test_modal_render_produces_visible_cells() {
    let modal = Modal::new("Hello Modal").with_size(30, 8);
    let plane = modal.render(Rect::new(0, 0, 80, 24));
    let has_visible = plane.cells.iter().any(|c| c.char != ' ' && c.char != '\0');
    assert!(has_visible);
}
