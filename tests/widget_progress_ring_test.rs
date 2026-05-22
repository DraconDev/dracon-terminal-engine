//! Tests for the ProgressRing widget.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::progress_ring::ProgressRing;
use ratatui::layout::Rect;

#[test]
fn test_progress_ring_new() {
    let pr = ProgressRing::new(0.5);
    assert!((pr.progress() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_default() {
    let pr = ProgressRing::default();
    assert!((pr.progress() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_with_theme() {
    let pr = ProgressRing::new(0.3).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 20, 10);
    let plane = pr.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_progress_ring_with_progress() {
    let pr = ProgressRing::new(0.0).with_progress(0.75);
    assert!((pr.progress() - 0.75).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_clamp_progress() {
    // Values outside 0.0-1.0 should be clamped
    let pr_max = ProgressRing::new(2.0);
    assert!((pr_max.progress() - 1.0).abs() < f64::EPSILON);

    let pr_min = ProgressRing::new(-0.5);
    assert!((pr_min.progress() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_with_size() {
    let pr = ProgressRing::new(0.5).with_size(10);
    let area = Rect::new(0, 0, 20, 10);
    let _plane = pr.render(area);
    // Size clamped to min 3, max 15
    let pr_small = ProgressRing::new(0.5).with_size(1);
    let pr_large = ProgressRing::new(0.5).with_size(100);
    let _ = pr_small;
    let _ = pr_large;
}

#[test]
fn test_progress_ring_with_color() {
    let pr = ProgressRing::new(0.5).with_color(Color::Ansi(9));
    let area = Rect::new(0, 0, 20, 10);
    let _plane = pr.render(area);
}

#[test]
fn test_progress_ring_with_bg_color() {
    let pr = ProgressRing::new(0.5).with_bg_color(Color::Ansi(0));
    let area = Rect::new(0, 0, 20, 10);
    let _plane = pr.render(area);
}

#[test]
fn test_progress_ring_show_percentage() {
    let pr = ProgressRing::new(0.65).show_percentage(false);
    let area = Rect::new(0, 0, 20, 10);
    let _plane = pr.render(area);
}

#[test]
fn test_progress_ring_with_label() {
    let pr = ProgressRing::new(0.5).with_label("Loading...");
    let area = Rect::new(0, 0, 20, 10);
    let _plane = pr.render(area);
}

#[test]
fn test_progress_ring_render_empty_area() {
    let pr = ProgressRing::new(0.5);
    let area = Rect::new(0, 0, 0, 0);
    let _plane = pr.render(area);
}

#[test]
fn test_progress_ring_render_narrow_area() {
    let pr = ProgressRing::new(0.5);
    let area = Rect::new(0, 0, 3, 3);
    let _plane = pr.render(area);
}

#[test]
fn test_progress_ring_increment() {
    let mut pr = ProgressRing::new(0.5);
    pr.increment(0.2);
    assert!((pr.progress() - 0.7).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_decrement() {
    let mut pr = ProgressRing::new(0.5);
    pr.decrement(0.2);
    assert!((pr.progress() - 0.3).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_increment_clamp() {
    let mut pr = ProgressRing::new(0.95);
    pr.increment(0.2);
    assert!((pr.progress() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_decrement_clamp() {
    let mut pr = ProgressRing::new(0.05);
    pr.decrement(0.2);
    assert!((pr.progress() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_set_progress_no_change() {
    let mut pr = ProgressRing::new(0.5);
    pr.set_progress(0.5);
    // Should not re-trigger callback for same value
    assert!((pr.progress() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_area() {
    let pr = ProgressRing::new(0.5);
    let area = Rect::new(5, 10, 20, 12);
    let mut pr = pr;
    pr.set_area(area);
    assert_eq!(pr.area().x, 5);
    assert_eq!(pr.area().y, 10);
}

#[test]
fn test_progress_ring_z_index() {
    let pr = ProgressRing::new(0.5);
    assert_eq!(pr.z_index(), 10);
}

#[test]
fn test_progress_ring_focusable() {
    let pr = ProgressRing::new(0.5);
    assert!(pr.focusable());
}

#[test]
fn test_progress_ring_handle_key_left() {
    let mut pr = ProgressRing::new(0.5);
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Left, Default::default());
    let handled = pr.handle_key(key);
    assert!(handled);
    assert!((pr.progress() - 0.45).abs() < 0.01);
}

#[test]
fn test_progress_ring_handle_key_right() {
    let mut pr = ProgressRing::new(0.5);
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Right, Default::default());
    let handled = pr.handle_key(key);
    assert!(handled);
    assert!((pr.progress() - 0.55).abs() < 0.01);
}

#[test]
fn test_progress_ring_handle_key_down() {
    let mut pr = ProgressRing::new(0.5);
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Down, Default::default());
    let handled = pr.handle_key(key);
    assert!(handled);
    assert!((pr.progress() - 0.45).abs() < 0.01);
}

#[test]
fn test_progress_ring_handle_key_up() {
    let mut pr = ProgressRing::new(0.5);
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Up, Default::default());
    let handled = pr.handle_key(key);
    assert!(handled);
    assert!((pr.progress() - 0.55).abs() < 0.01);
}

#[test]
fn test_progress_ring_handle_key_home() {
    let mut pr = ProgressRing::new(0.75);
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Home, Default::default());
    let handled = pr.handle_key(key);
    assert!(handled);
    assert!((pr.progress() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_handle_key_end() {
    let mut pr = ProgressRing::new(0.25);
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::End, Default::default());
    let handled = pr.handle_key(key);
    assert!(handled);
    assert!((pr.progress() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ring_handle_key_release() {
    let mut pr = ProgressRing::new(0.5);
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind};
    let key = KeyEvent::new(KeyEventKind::Release, KeyCode::Right, Default::default());
    let handled = pr.handle_key(key);
    assert!(!handled);
}

#[test]
fn test_progress_ring_handle_mouse_down() {
    let mut pr = ProgressRing::new(0.5);
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    // Click in the center area of the ring
    let handled = pr.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 5);
    assert!(handled);
}

#[test]
fn test_progress_ring_handle_mouse_drag() {
    let mut pr = ProgressRing::new(0.5);
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    let handled = pr.handle_mouse(MouseEventKind::Drag(MouseButton::Left), 12, 5);
    assert!(handled);
}

#[test]
fn test_progress_ring_handle_mouse_move() {
    let mut pr = ProgressRing::new(0.5);
    use dracon_terminal_engine::input::event::MouseEventKind;
    let handled = pr.handle_mouse(MouseEventKind::Moved, 10, 5);
    assert!(!handled);
}

#[test]
fn test_progress_ring_on_theme_change() {
    let mut pr = ProgressRing::new(0.5);
    let new_theme = Theme::dracula();
    pr.on_theme_change(&new_theme);
    // Just verify it doesn't panic and completes
}

#[test]
fn test_progress_ring_on_change_callback() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let changed = Rc::new(RefCell::new(false));
    let changed_clone = Rc::clone(&changed);
    let mut pr = ProgressRing::new(0.5).on_change(move |_| {
        *changed_clone.borrow_mut() = true;
    });
    pr.increment(0.1);
    assert!(*changed.borrow());
}

#[test]
fn test_progress_ring_id_stability() {
    let pr1 = ProgressRing::new(0.5);
    let pr2 = ProgressRing::new(0.5);
    // Each widget gets a unique ID
    assert_ne!(pr1.id(), pr2.id());
}

#[test]
fn test_progress_ring_set_id() {
    let mut pr = ProgressRing::new(0.5);
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(42);
    pr.set_id(id);
    assert_eq!(pr.id(), id);
}
