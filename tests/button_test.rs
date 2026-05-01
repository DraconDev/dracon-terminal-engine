//! Tests for framework Button and standalone Button widgets.

mod common;
use common::{make_area, make_key, rect};

use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::button::Button as FrameworkButton;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::input::event::{KeyCode, MouseButton, MouseEventKind};
use dracon_terminal_engine::widgets::button::Button as StandaloneButton;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_framework_button_new() {
    let btn = FrameworkButton::new("Click me");
    let plane = btn.render(make_area(20, 1));
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_framework_button_with_id() {
    let id = WidgetId::new(99);
    let btn = FrameworkButton::with_id(id, "Label");
    assert_eq!(btn.id(), id);
}

#[test]
fn test_framework_button_with_theme() {
    let btn = FrameworkButton::new("test").with_theme(Theme::cyberpunk());
    let plane = btn.render(make_area(20, 1));
    assert!(plane.width == 20);
}

#[test]
fn test_framework_button_default_area() {
    let btn = FrameworkButton::new("test");
    let area = btn.area();
    assert_eq!(area.width, 10);
    assert_eq!(area.height, 1);
}

#[test]
fn test_framework_button_render_size() {
    let btn = FrameworkButton::new("Hi");
    let area = make_area(20, 3);
    let plane = btn.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_framework_button_render_brackets() {
    let btn = FrameworkButton::new("OK");
    let area = make_area(10, 1);
    let plane = btn.render(area);
    assert_eq!(plane.cells[0].char, '[');
    let label_end = 1 + 2;
    assert_eq!(plane.cells[label_end].char, ']');
}

#[test]
fn test_framework_button_render_label_chars() {
    let btn = FrameworkButton::new("ABC");
    let area = make_area(10, 1);
    let plane = btn.render(area);
    assert_eq!(plane.cells[1].char, 'A');
    assert_eq!(plane.cells[2].char, 'B');
    assert_eq!(plane.cells[3].char, 'C');
}

#[test]
fn test_framework_button_truncates_long_label() {
    let btn = FrameworkButton::new("This is a very long button label");
    let area = make_area(10, 1);
    let plane = btn.render(area);
    assert_eq!(plane.cells[1].char, 'T');
    assert_eq!(plane.cells[2].char, 'h');
    assert_eq!(plane.cells[3].char, 'i');
    assert_eq!(plane.cells[4].char, 's');
}

#[test]
fn test_framework_button_with_theme_colors() {
    let btn = FrameworkButton::new("C").with_theme(Theme::cyberpunk());
    let area = make_area(10, 1);
    let plane = btn.render(area);
    assert_eq!(plane.cells[1].bg, Theme::cyberpunk().bg);
}

#[test]
fn test_framework_button_clear_dirty() {
    let mut btn = FrameworkButton::new("test");
    assert!(btn.needs_render());
    btn.clear_dirty();
    assert!(!btn.needs_render());
}

#[test]
fn test_framework_button_mark_dirty() {
    let mut btn = FrameworkButton::new("test");
    btn.clear_dirty();
    btn.mark_dirty();
    assert!(btn.needs_render());
}

#[test]
fn test_framework_button_set_area() {
    let mut btn = FrameworkButton::new("test");
    assert!(btn.needs_render());
    btn.set_area(make_area(5, 1));
    assert!(btn.needs_render());
}

#[test]
fn test_framework_button_handle_key_enter_triggers_callback() {
    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    {
        let mut btn = FrameworkButton::new("OK").on_click(move || {
            called_clone.set(true);
        });
        btn.handle_key(make_key(KeyCode::Enter));
    }
    assert!(called.get());
}

#[test]
fn test_framework_button_handle_key_non_enter_returns_false() {
    let mut btn = FrameworkButton::new("OK").on_click(|| {});
    let result = btn.handle_key(make_key(KeyCode::Backspace));
    assert!(!result);
}

#[test]
fn test_framework_button_handle_mouse_click_triggers_callback() {
    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    {
        let mut btn = FrameworkButton::new("OK").on_click(move || {
            called_clone.set(true);
        });
        btn.set_area(rect(5, 5, 10, 1));
        let result = btn.handle_mouse(MouseEventKind::Down(MouseButton::Left), 6, 5);
        assert!(result);
    }
    assert!(called.get());
}

#[test]
fn test_framework_button_handle_mouse_outside_area_returns_false() {
    let mut btn = FrameworkButton::new("OK").on_click(|| {});
    btn.set_area(rect(5, 5, 10, 1));
    let result = btn.handle_mouse(MouseEventKind::Down(MouseButton::Left), 20, 5);
    assert!(!result);
}

#[test]
fn test_framework_button_handle_mouse_right_click_returns_true_when_in_area() {
    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    let mut btn = FrameworkButton::new("OK").on_click(move || {
        called_clone.set(true);
    });
    btn.set_area(rect(5, 5, 10, 1));
    let result = btn.handle_mouse(MouseEventKind::Down(MouseButton::Right), 6, 5);
    assert!(result);
}

#[test]
fn test_framework_button_multiple_clicks() {
    let count = Rc::new(Cell::new(0));
    let count_clone = count.clone();
    {
        let mut btn = FrameworkButton::new("OK").on_click(move || {
            count_clone.set(count_clone.get() + 1);
        });
        btn.set_area(rect(5, 5, 10, 1));
        btn.handle_mouse(MouseEventKind::Down(MouseButton::Left), 6, 5);
        btn.handle_mouse(MouseEventKind::Down(MouseButton::Left), 6, 5);
        btn.handle_mouse(MouseEventKind::Down(MouseButton::Left), 6, 5);
    }
    assert_eq!(count.get(), 3);
}

#[test]
fn test_framework_button_empty_label_fallback() {
    let btn = FrameworkButton::new("");
    let area = make_area(20, 3);
    let plane = btn.render(area);
    assert_eq!(plane.cells[0].char, '[');
    let end_idx = (1 + "Button".len()) as usize;
    assert_eq!(plane.cells[end_idx].char, ']');
}

#[test]
fn test_framework_button_id_method() {
    let btn = FrameworkButton::new("test");
    let id = btn.id();
    assert_eq!(id, WidgetId::default_id());
}

#[test]
fn test_framework_button_focusable() {
    let btn = FrameworkButton::new("test");
    assert!(btn.focusable());
}

#[test]
fn test_framework_button_z_index() {
    let btn = FrameworkButton::new("test");
    assert_eq!(btn.z_index(), 0);
}

#[test]
fn test_framework_button_set_id() {
    let mut btn = FrameworkButton::new("test");
    let id = WidgetId::new(42);
    btn.set_id(id);
    assert_eq!(btn.id(), id);
}

#[test]
fn test_framework_button_cursor_position_returns_none() {
    let btn = FrameworkButton::new("test");
    assert!(btn.cursor_position().is_none());
}

#[test]
fn test_framework_button_render_idempotent() {
    let btn = FrameworkButton::new("test");
    let area = make_area(10, 1);
    let plane1 = btn.render(area);
    let plane2 = btn.render(area);
    assert_eq!(plane1.cells[0].char, plane2.cells[0].char);
    assert_eq!(plane1.cells[1].char, plane2.cells[1].char);
}

// ========== Standalone Button ==========

#[test]
fn test_standalone_button_new() {
    let btn = StandaloneButton::new("Press", false);
    let area = make_area(20, 3);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    ratatui::widgets::Widget::render(btn, area, &mut buf);
}

#[test]
fn test_standalone_button_active_state() {
    let btn = StandaloneButton::new("Active", true);
    let area = make_area(20, 3);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    ratatui::widgets::Widget::render(btn, area, &mut buf);
}

#[test]
fn test_standalone_button_inactive_state() {
    let btn = StandaloneButton::new("Inactive", false);
    let area = make_area(20, 3);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    ratatui::widgets::Widget::render(btn, area, &mut buf);
}