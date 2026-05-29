//! Tests for the Sparkline widget.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::sparkline::Sparkline;
use ratatui::layout::Rect;

#[test]
fn test_sparkline_new() {
    let data = vec![1.0, 2.0, 3.0, 2.5, 4.0];
    let sp = Sparkline::new(data);
    let area = Rect::new(0, 0, 40, 5);
    let plane = sp.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 5);
}

#[test]
fn test_sparkline_with_theme() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_with_color() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]).with_color(Color::Ansi(10));
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_with_fill_color() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]).with_fill_color(Color::Ansi(24));
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_with_height() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]).with_height(5);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_height_clamp() {
    let sp_small = Sparkline::new(vec![1.0, 2.0]).with_height(0);
    let sp_large = Sparkline::new(vec![1.0, 2.0]).with_height(100);
    let area = Rect::new(0, 0, 40, 10);
    let _ = sp_small.render(area);
    let _ = sp_large.render(area);
}

#[test]
fn test_sparkline_with_dots() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]).with_dots(true);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_with_min_max() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]).with_min_max(true);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_with_data() {
    let sp = Sparkline::new(vec![]).with_data(vec![10.0, 20.0, 15.0]);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_render_empty_data() {
    let sp = Sparkline::new(vec![]);
    let area = Rect::new(0, 0, 40, 5);
    let plane = sp.render(area);
    // Should show "No data" message
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 5);
}

#[test]
fn test_sparkline_render_narrow_area() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    let area = Rect::new(0, 0, 2, 5);
    let plane = sp.render(area);
    assert_eq!(plane.width, 2);
}

#[test]
fn test_sparkline_render_zero_height() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    let area = Rect::new(0, 0, 40, 0);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_set_data() {
    let mut sp = Sparkline::new(vec![1.0, 2.0]);
    sp.set_data(vec![5.0, 10.0, 15.0]);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_set_value() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    // set_value modifies internal data - just verify it doesn't panic
    sp.set_value(1, 5.0);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_set_value_out_of_bounds() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    // Out of bounds - should not panic
    sp.set_value(99, 5.0);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_push() {
    let mut sp = Sparkline::new(vec![1.0, 2.0]);
    sp.push(3.0);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_clear() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    sp.clear();
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_area() {
    let sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    let area = Rect::new(5, 10, 40, 5);
    let mut sp = sp;
    sp.set_area(area);
    assert_eq!(sp.area().x, 5);
    assert_eq!(sp.area().y, 10);
}

#[test]
fn test_sparkline_z_index() {
    let sp = Sparkline::new(vec![1.0, 2.0]);
    assert_eq!(sp.z_index(), 10);
}

#[test]
fn test_sparkline_needs_render() {
    let sp = Sparkline::new(vec![1.0, 2.0]);
    assert!(sp.needs_render());
}

#[test]
fn test_sparkline_clear_dirty() {
    let mut sp = Sparkline::new(vec![1.0, 2.0]);
    sp.clear_dirty();
    // Should still be dirty per implementation (intentional)
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_handle_key() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    use dracon_terminal_engine::input::event::{KeyCode, KeyEventKind};
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Right,
        modifiers: Default::default(),
    };
    let handled = sp.handle_key(key);
    assert!(!handled); // Sparkline doesn't handle keys
}

#[test]
fn test_sparkline_handle_mouse_move() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    sp.set_area(Rect::new(0, 0, 40, 5));
    use dracon_terminal_engine::input::event::MouseEventKind;
    let handled = sp.handle_mouse(MouseEventKind::Moved, 10, 2);
    assert!(handled);
}

#[test]
fn test_sparkline_handle_mouse_move_empty_data() {
    let mut sp = Sparkline::new(vec![]);
    sp.set_area(Rect::new(0, 0, 40, 5));
    use dracon_terminal_engine::input::event::MouseEventKind;
    let handled = sp.handle_mouse(MouseEventKind::Moved, 10, 2);
    assert!(!handled);
}

#[test]
fn test_sparkline_handle_mouse_move_narrow() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    sp.set_area(Rect::new(0, 0, 2, 5));
    use dracon_terminal_engine::input::event::MouseEventKind;
    let handled = sp.handle_mouse(MouseEventKind::Moved, 10, 2);
    assert!(!handled);
}

#[test]
fn test_sparkline_handle_mouse_move_row_too_tall() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0]).with_height(3);
    sp.set_area(Rect::new(0, 0, 40, 5));
    use dracon_terminal_engine::input::event::MouseEventKind;
    // Row is beyond the sparkline height
    let handled = sp.handle_mouse(MouseEventKind::Moved, 10, 5);
    assert!(!handled);
}

#[test]
fn test_sparkline_handle_mouse_down() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    sp.set_area(Rect::new(0, 0, 40, 5));
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    let handled = sp.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 2);
    assert!(handled);
}

#[test]
fn test_sparkline_on_point_click_callback() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let called = Rc::new(RefCell::new(false));
    let called_clone = Rc::clone(&called);
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    sp.set_area(Rect::new(0, 0, 40, 5));
    sp = sp.on_point_click(move |_idx, _val| {
        *called_clone.borrow_mut() = true;
    });
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    // First move to hover over a point
    let moved = sp.handle_mouse(MouseEventKind::Moved, 10, 2);
    assert!(moved);
    // Then click to trigger callback
    let clicked = sp.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 2);
    assert!(clicked);
    assert!(*called.borrow());
}

#[test]
fn test_sparkline_hover_tracking() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    sp.set_area(Rect::new(0, 0, 40, 5));
    use dracon_terminal_engine::input::event::MouseEventKind;
    sp.handle_mouse(MouseEventKind::Moved, 10, 2);
    // Move to different position
    sp.handle_mouse(MouseEventKind::Moved, 20, 2);
}

#[test]
fn test_sparkline_on_theme_change() {
    let mut sp = Sparkline::new(vec![1.0, 2.0, 3.0]);
    let new_theme = Theme::dracula();
    sp.on_theme_change(&new_theme);
    // Just verify it doesn't panic
}

#[test]
fn test_sparkline_id_stability() {
    let sp1 = Sparkline::new(vec![1.0, 2.0]);
    let sp2 = Sparkline::new(vec![1.0, 2.0]);
    assert_ne!(sp1.id(), sp2.id());
}

#[test]
fn test_sparkline_set_id() {
    let mut sp = Sparkline::new(vec![1.0, 2.0]);
    let id = dracon_terminal_engine::framework::widget::WidgetId::new(99);
    sp.set_id(id);
    assert_eq!(sp.id(), id);
}

#[test]
fn test_sparkline_single_point() {
    let sp = Sparkline::new(vec![5.0]);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_two_points() {
    let sp = Sparkline::new(vec![1.0, 2.0]);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_many_points() {
    let data: Vec<f64> = (0..100).map(|i| (i as f64) * 1.5).collect();
    let sp = Sparkline::new(data);
    let area = Rect::new(0, 0, 80, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_negative_values() {
    let sp = Sparkline::new(vec![-10.0, -5.0, 0.0, 5.0, 10.0]);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}

#[test]
fn test_sparkline_mixed_values() {
    let sp = Sparkline::new(vec![-100.0, 50.0, -25.0, 75.0, 0.0]);
    let area = Rect::new(0, 0, 40, 5);
    let _plane = sp.render(area);
}
