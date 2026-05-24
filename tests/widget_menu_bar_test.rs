//! Tests for the MenuBar widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::menu_bar::{MenuBar, MenuEntry, MenuItem};

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_menu_bar_new() {
    let mb = MenuBar::new(WidgetId::new(1));
    let area = mb.area();
    assert!(area.width > 0);
}

#[test]
fn test_menu_bar_with_entries() {
    let entries = vec![
        MenuEntry::new("File"),
        MenuEntry::new("Edit"),
    ];
    let mb = MenuBar::new(WidgetId::new(1)).with_entries(entries);
    let plane = mb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_menu_bar_with_id() {
    let mb = MenuBar::new(WidgetId::new(42));
    assert_eq!(mb.id(), WidgetId::new(42));
}

// ============================================================================
// MenuItem Tests
// ============================================================================

#[test]

#[test]

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_menu_bar_id() {
    let mb = MenuBar::new(WidgetId::new(42));
    assert_eq!(mb.id(), WidgetId::new(42));
}

#[test]
fn test_menu_bar_set_id() {
    let mut mb = MenuBar::new(WidgetId::new(1));
    mb.set_id(WidgetId::new(99));
    assert_eq!(mb.id(), WidgetId::new(99));
}

#[test]
fn test_menu_bar_area() {
    let mb = MenuBar::new(WidgetId::new(1));
    let area = mb.area();
    assert!(area.width > 0);
}

#[test]
fn test_menu_bar_set_area() {
    let mut mb = MenuBar::new(WidgetId::new(1));
    mb.set_area(Rect::new(0, 0, 100, 2));
    assert_eq!(mb.area(), Rect::new(0, 0, 100, 2));
}

#[test]
fn test_menu_bar_needs_render() {
    let mb = MenuBar::new(WidgetId::new(1));
    assert!(mb.needs_render());
}

#[test]
fn test_menu_bar_mark_dirty() {
    let mut mb = MenuBar::new(WidgetId::new(1));
    mb.clear_dirty();
    assert!(!mb.needs_render());
    mb.mark_dirty();
    assert!(mb.needs_render());
}

#[test]
fn test_menu_bar_clear_dirty() {
    let mut mb = MenuBar::new(WidgetId::new(1));
    mb.clear_dirty();
    assert!(!mb.needs_render());
}

#[test]
fn test_menu_bar_default_dirty() {
    let mb = MenuBar::new(WidgetId::new(1));
    assert!(mb.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_menu_bar_render_basic() {
    let mb = MenuBar::new(WidgetId::new(1));
    let plane = mb.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_menu_bar_render_has_content() {
    let mb = MenuBar::new(WidgetId::new(1));
    let plane = mb.render(Rect::new(0, 0, 80, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_menu_bar_render_wide() {
    let mb = MenuBar::new(WidgetId::new(1));
    let plane = mb.render(Rect::new(0, 0, 120, 1));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_menu_bar_render_small() {
    let mb = MenuBar::new(WidgetId::new(1));
    let plane = mb.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_menu_bar_render_tall() {
    let mb = MenuBar::new(WidgetId::new(1));
    let plane = mb.render(Rect::new(0, 0, 80, 2));
    assert_eq!(plane.height, 2);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_menu_bar_on_theme_change() {
    let mut mb = MenuBar::new(WidgetId::new(1));
    mb.on_theme_change(&Theme::nord());
    assert!(mb.needs_render());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_menu_bar_render_twice() {
    let mb = MenuBar::new(WidgetId::new(1));
    let _ = mb.render(Rect::new(0, 0, 80, 1));
    let _ = mb.render(Rect::new(0, 0, 80, 1));
}

#[test]
fn test_menu_bar_set_area_then_render() {
    let mut mb = MenuBar::new(WidgetId::new(1));
    mb.set_area(Rect::new(0, 0, 100, 2));
    let plane = mb.render(Rect::new(0, 0, 100, 2));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_menu_bar_many_entries() {
    let entries: Vec<MenuEntry> = (0..10).map(|i| MenuEntry::new(&format!("Menu{}", i))).collect();
    let mb = MenuBar::new(WidgetId::new(1)).with_entries(entries);
    let plane = mb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}
