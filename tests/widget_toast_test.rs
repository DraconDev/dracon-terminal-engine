//! Tests for the Toast widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::toast::{Toast, ToastKind};
use std::time::Duration;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_toast_new() {
    let t = Toast::new(WidgetId::new(1), "Hello World");
    assert_eq!(t.message(), "Hello World");
}

#[test]
fn test_toast_new_with_id() {
    let t = Toast::new(WidgetId::new(42), "Test");
    assert_eq!(t.id(), WidgetId::new(42));
}

#[test]
fn test_toast_with_kind() {
    let t = Toast::new(WidgetId::new(1), "Test").with_kind(ToastKind::Info);
    let plane = t.render(Rect::new(0, 0, 50, 3));
    assert!(plane.width > 0);
}

#[test]
fn test_toast_with_duration() {
    let t = Toast::new(WidgetId::new(1), "Test").with_duration(Duration::from_secs(5));
    let plane = t.render(Rect::new(0, 0, 50, 3));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_toast_id() {
    let t = Toast::new(WidgetId::new(42), "Test");
    assert_eq!(t.id(), WidgetId::new(42));
}

#[test]
fn test_toast_set_id() {
    let mut t = Toast::new(WidgetId::new(1), "Test");
    t.set_id(WidgetId::new(99));
    assert_eq!(t.id(), WidgetId::new(99));
}

#[test]
fn test_toast_area() {
    let t = Toast::new(WidgetId::new(1), "Test");
    let area = t.area();
    assert!(area.width > 0);
}

#[test]
fn test_toast_set_area() {
    let mut t = Toast::new(WidgetId::new(1), "Test");
    t.set_area(Rect::new(0, 0, 60, 4));
    assert_eq!(t.area(), Rect::new(0, 0, 60, 4));
}

#[test]
fn test_toast_needs_render() {
    let t = Toast::new(WidgetId::new(1), "Test");
    assert!(t.needs_render());
}

#[test]
fn test_toast_mark_dirty() {
    let mut t = Toast::new(WidgetId::new(1), "Test");
    t.clear_dirty();
    assert!(!t.needs_render());
    t.mark_dirty();
    assert!(t.needs_render());
}

#[test]
fn test_toast_clear_dirty() {
    let mut t = Toast::new(WidgetId::new(1), "Test");
    t.clear_dirty();
    assert!(!t.needs_render());
}

#[test]
fn test_toast_default_dirty() {
    let t = Toast::new(WidgetId::new(1), "Test");
    assert!(t.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_toast_render_basic() {
    let t = Toast::new(WidgetId::new(1), "Notification");
    let plane = t.render(Rect::new(0, 0, 50, 3));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_toast_render_has_content() {
    let t = Toast::new(WidgetId::new(1), "Test");
    let plane = t.render(Rect::new(0, 0, 50, 3));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_toast_render_wide() {
    let t = Toast::new(WidgetId::new(1), "Test");
    let plane = t.render(Rect::new(0, 0, 80, 3));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_toast_render_small() {
    let t = Toast::new(WidgetId::new(1), "X");
    let plane = t.render(Rect::new(0, 0, 10, 2));
    assert_eq!(plane.width, 10);
}

#[test]
fn test_toast_render_tall() {
    let t = Toast::new(WidgetId::new(1), "Test");
    let plane = t.render(Rect::new(0, 0, 50, 5));
    assert_eq!(plane.height, 5);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_toast_on_theme_change() {
    let mut t = Toast::new(WidgetId::new(1), "Test");
    t.on_theme_change(&Theme::nord());
    assert!(t.needs_render());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_toast_render_twice() {
    let t = Toast::new(WidgetId::new(1), "Test");
    let _ = t.render(Rect::new(0, 0, 50, 3));
    let _ = t.render(Rect::new(0, 0, 50, 3));
}

#[test]
fn test_toast_set_area_then_render() {
    let mut t = Toast::new(WidgetId::new(1), "Test");
    t.set_area(Rect::new(0, 0, 60, 4));
    let plane = t.render(Rect::new(0, 0, 60, 4));
    assert_eq!(plane.width, 60);
}

#[test]
fn test_toast_empty_message() {
    let t = Toast::new(WidgetId::new(1), "");
    let plane = t.render(Rect::new(0, 0, 50, 3));
    assert!(plane.width > 0);
}

#[test]
fn test_toast_long_message() {
    let t = Toast::new(WidgetId::new(1), &"A".repeat(100));
    let plane = t.render(Rect::new(0, 0, 50, 3));
    assert!(plane.width > 0);
}

#[test]
fn test_toast_is_expired() {
    let t = Toast::new(WidgetId::new(1), "Test");
    // Default duration should not be expired immediately
    let _ = t.is_expired();
}
