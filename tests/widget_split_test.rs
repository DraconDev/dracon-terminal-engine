//! Tests for the SplitPane widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::split::SplitPane;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_split_pane_new() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let area = sp.area();
    assert!(area.width > 0);
}

#[test]
fn test_split_pane_new_with_id() {
    let sp = SplitPane::new_with_id(WidgetId::new(42), Orientation::Vertical);
    assert_eq!(sp.id(), WidgetId::new(42));
}

#[test]
fn test_split_pane_from_rect() {
    let sp = SplitPane::from_rect(Rect::new(0, 0, 100, 50));
    let area = sp.area();
    assert!(area.width > 0);
}

// ============================================================================
// Orientation Tests
// ============================================================================

#[test]
fn test_split_pane_horizontal() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let plane = sp.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_split_pane_vertical() {
    let sp = SplitPane::new(Orientation::Vertical);
    let plane = sp.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_split_pane_id() {
    let sp = SplitPane::new_with_id(WidgetId::new(42), Orientation::Horizontal);
    assert_eq!(sp.id(), WidgetId::new(42));
}

#[test]
fn test_split_pane_set_id() {
    let mut sp = SplitPane::new(Orientation::Horizontal);
    sp.set_id(WidgetId::new(99));
    assert_eq!(sp.id(), WidgetId::new(99));
}

#[test]
fn test_split_pane_area() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let area = sp.area();
    assert!(area.width > 0);
}

#[test]
fn test_split_pane_set_area() {
    let mut sp = SplitPane::new(Orientation::Horizontal);
    sp.set_area(Rect::new(0, 0, 100, 30));
    assert_eq!(sp.area(), Rect::new(0, 0, 100, 30));
}

#[test]
fn test_split_pane_needs_render() {
    let sp = SplitPane::new(Orientation::Horizontal);
    assert!(sp.needs_render());
}

#[test]
fn test_split_pane_mark_dirty() {
    let mut sp = SplitPane::new(Orientation::Horizontal);
    sp.clear_dirty();
    assert!(!sp.needs_render());
    sp.mark_dirty();
    assert!(sp.needs_render());
}

#[test]
fn test_split_pane_clear_dirty() {
    let mut sp = SplitPane::new(Orientation::Horizontal);
    sp.clear_dirty();
    assert!(!sp.needs_render());
}

#[test]
fn test_split_pane_default_dirty() {
    let sp = SplitPane::new(Orientation::Horizontal);
    assert!(sp.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_split_pane_render_basic() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let plane = sp.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_split_pane_render_has_content() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let plane = sp.render(Rect::new(0, 0, 80, 20));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_split_pane_render_wide() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let plane = sp.render(Rect::new(0, 0, 120, 20));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_split_pane_render_small() {
    let sp = SplitPane::new(Orientation::Vertical);
    let plane = sp.render(Rect::new(0, 0, 30, 10));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_split_pane_render_tall() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let plane = sp.render(Rect::new(0, 0, 80, 50));
    assert_eq!(plane.height, 50);
}

// ============================================================================
// Split/Merge Tests
// ============================================================================

#[test]
fn test_split_pane_get_ratio() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let ratio = sp.get_ratio();
    assert!((0.0..=1.0).contains(&ratio));
}

#[test]
fn test_split_pane_ratio() {
    let sp = SplitPane::new(Orientation::Horizontal).ratio(0.75);
    assert!((sp.get_ratio() - 0.75).abs() < 0.001);
}

#[test]
fn test_split_pane_split_horizontal() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let (left, right) = sp.split(Rect::new(0, 0, 80, 20));
    assert!(left.width + right.width <= 80);
}

#[test]
fn test_split_pane_split_vertical() {
    let sp = SplitPane::new(Orientation::Vertical);
    let (top, bottom) = sp.split(Rect::new(0, 0, 80, 20));
    assert!(top.height + bottom.height <= 20);
}

// ============================================================================
// Divider Tests
// ============================================================================

#[test]
fn test_split_pane_with_divider() {
    let sp = SplitPane::new(Orientation::Horizontal).with_divider('|');
    let plane = sp.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_split_pane_divider_rect() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let rect = sp.divider_rect(Rect::new(0, 0, 80, 20));
    // Should return a valid rectangle
    assert!(rect.width > 0 || rect.height > 0);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_split_pane_on_theme_change() {
    let mut sp = SplitPane::new(Orientation::Horizontal);
    sp.on_theme_change(&Theme::nord());
    assert!(sp.needs_render());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_split_pane_render_twice() {
    let sp = SplitPane::new(Orientation::Horizontal);
    let _ = sp.render(Rect::new(0, 0, 80, 20));
    let _ = sp.render(Rect::new(0, 0, 80, 20));
}

#[test]
fn test_split_pane_set_area_then_render() {
    let mut sp = SplitPane::new(Orientation::Horizontal);
    sp.set_area(Rect::new(0, 0, 100, 30));
    let plane = sp.render(Rect::new(0, 0, 100, 30));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_split_pane_min_size() {
    let sp = SplitPane::new(Orientation::Horizontal).with_min_size(20);
    let plane = sp.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}
