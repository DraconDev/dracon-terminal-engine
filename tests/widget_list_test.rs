//! Tests for the List widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::list::List;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_list_new() {
    let items = vec!["One".to_string(), "Two".to_string()];
    let list = List::new(items);
    let area = list.area();
    assert!(area.width > 0);
}

#[test]
fn test_list_new_with_id() {
    let items = vec!["Item".to_string()];
    let list = List::new_with_id(WidgetId::new(42), items);
    assert_eq!(list.id(), WidgetId::new(42));
}

#[test]
fn test_list_with_theme() {
    let items = vec!["Test".to_string()];
    let list = List::new(items).with_theme(Theme::nord());
    let plane = list.render(Rect::new(0, 0, 30, 10));
    assert!(plane.width > 0);
}

#[test]
fn test_list_empty() {
    let items: Vec<String> = vec![];
    let list = List::new(items);
    let plane = list.render(Rect::new(0, 0, 30, 10));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_list_id() {
    let list = List::new_with_id(WidgetId::new(42), vec!["Test".to_string()]);
    assert_eq!(list.id(), WidgetId::new(42));
}

#[test]
fn test_list_set_id() {
    let items = vec!["Test".to_string()];
    let mut list = List::new(items);
    list.set_id(WidgetId::new(99));
    assert_eq!(list.id(), WidgetId::new(99));
}

#[test]
fn test_list_area() {
    let items = vec!["Test".to_string()];
    let list = List::new(items);
    let area = list.area();
    assert!(area.width > 0);
}

#[test]
fn test_list_set_area() {
    let items = vec!["Test".to_string()];
    let mut list = List::new(items);
    list.set_area(Rect::new(0, 0, 50, 20));
    assert_eq!(list.area(), Rect::new(0, 0, 50, 20));
}

#[test]
fn test_list_needs_render() {
    let items = vec!["Test".to_string()];
    let list = List::new(items);
    assert!(list.needs_render());
}

#[test]
fn test_list_mark_dirty() {
    let items = vec!["Test".to_string()];
    let mut list = List::new(items);
    list.clear_dirty();
    assert!(!list.needs_render());
    list.mark_dirty();
    assert!(list.needs_render());
}

#[test]
fn test_list_clear_dirty() {
    let items = vec!["Test".to_string()];
    let mut list = List::new(items);
    list.clear_dirty();
    assert!(!list.needs_render());
}

#[test]
fn test_list_default_dirty() {
    let items = vec!["Test".to_string()];
    let list = List::new(items);
    assert!(list.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_list_render_basic() {
    let items = vec!["Item 1".to_string(), "Item 2".to_string()];
    let list = List::new(items);
    let plane = list.render(Rect::new(0, 0, 30, 10));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_list_render_has_content() {
    let items = vec!["Test".to_string()];
    let list = List::new(items);
    let plane = list.render(Rect::new(0, 0, 30, 10));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_list_render_wide() {
    let items = vec!["Test".to_string()];
    let list = List::new(items);
    let plane = list.render(Rect::new(0, 0, 50, 10));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_list_render_small() {
    let items = vec!["Test".to_string()];
    let list = List::new(items);
    let plane = list.render(Rect::new(0, 0, 10, 5));
    assert_eq!(plane.width, 10);
}

#[test]
fn test_list_render_tall() {
    let items = vec!["Test".to_string()];
    let list = List::new(items);
    let plane = list.render(Rect::new(0, 0, 30, 20));
    assert_eq!(plane.height, 20);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_list_on_theme_change() {
    let items = vec!["Test".to_string()];
    let mut list = List::new(items);
    list.on_theme_change(&Theme::nord());
    assert!(list.needs_render());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_list_render_twice() {
    let items = vec!["Test".to_string()];
    let list = List::new(items);
    let _ = list.render(Rect::new(0, 0, 30, 10));
    let _ = list.render(Rect::new(0, 0, 30, 10));
}

#[test]
fn test_list_set_area_then_render() {
    let items = vec!["Test".to_string()];
    let mut list = List::new(items);
    list.set_area(Rect::new(0, 0, 50, 20));
    let plane = list.render(Rect::new(0, 0, 50, 20));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_list_many_items() {
    let items: Vec<String> = (0..50).map(|i| format!("Item {}", i)).collect();
    let list = List::new(items);
    let plane = list.render(Rect::new(0, 0, 30, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_list_unicode_items() {
    let items = vec!["日本語".to_string(), "العربية".to_string()];
    let list = List::new(items);
    let plane = list.render(Rect::new(0, 0, 30, 10));
    assert!(plane.width > 0);
}
