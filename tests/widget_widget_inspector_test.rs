//! Tests for the WidgetInspector widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::widget_inspector::{WidgetInspector, WidgetNode};

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_widget_inspector_new() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    let area = wi.area();
    assert!(area.width > 0);
}

#[test]
fn test_widget_inspector_new_with_id() {
    let wi = WidgetInspector::new(WidgetId::new(42));
    assert_eq!(wi.id(), WidgetId::new(42));
}

#[test]
fn test_widget_inspector_with_theme() {
    let wi = WidgetInspector::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = wi.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}

// ============================================================================
// WidgetNode Tests
// ============================================================================

// ============================================================================
// Widget Trait Tests
// ============================================================================
#[test]
fn test_widget_inspector_id() {
    let wi = WidgetInspector::new(WidgetId::new(42));
    assert_eq!(wi.id(), WidgetId::new(42));
}

#[test]
fn test_widget_inspector_set_id() {
    let mut wi = WidgetInspector::new(WidgetId::new(1));
    wi.set_id(WidgetId::new(99));
    assert_eq!(wi.id(), WidgetId::new(99));
}

#[test]
fn test_widget_inspector_area() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    let area = wi.area();
    assert!(area.width > 0);
}

#[test]
fn test_widget_inspector_set_area() {
    let mut wi = WidgetInspector::new(WidgetId::new(1));
    wi.set_area(Rect::new(0, 0, 100, 40));
    assert_eq!(wi.area(), Rect::new(0, 0, 100, 40));
}

#[test]
fn test_widget_inspector_needs_render() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    assert!(wi.needs_render());
}

#[test]
fn test_widget_inspector_mark_dirty() {
    let mut wi = WidgetInspector::new(WidgetId::new(1));
    wi.clear_dirty();
    assert!(!wi.needs_render());
    wi.mark_dirty();
    assert!(wi.needs_render());
}

#[test]
fn test_widget_inspector_clear_dirty() {
    let mut wi = WidgetInspector::new(WidgetId::new(1));
    wi.clear_dirty();
    assert!(!wi.needs_render());
}

#[test]
fn test_widget_inspector_default_dirty() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    assert!(wi.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_widget_inspector_render_basic() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    let plane = wi.render(Rect::new(0, 0, 80, 30));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_widget_inspector_render_has_content() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    let plane = wi.render(Rect::new(0, 0, 80, 30));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_widget_inspector_render_wide() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    let plane = wi.render(Rect::new(0, 0, 120, 30));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_widget_inspector_render_small() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    let plane = wi.render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_widget_inspector_render_tall() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    let plane = wi.render(Rect::new(0, 0, 80, 50));
    assert_eq!(plane.height, 50);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_widget_inspector_theme_nord() {
    let wi = WidgetInspector::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = wi.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}

#[test]
fn test_widget_inspector_theme_dracula() {
    let wi = WidgetInspector::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = wi.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}

#[test]
fn test_widget_inspector_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let wi = WidgetInspector::new(WidgetId::new(1)).with_theme(t);
        let _ = wi.render(Rect::new(0, 0, 80, 30));
    }
}

#[test]
fn test_widget_inspector_on_theme_change() {
    let mut wi = WidgetInspector::new(WidgetId::new(1));
    wi.on_theme_change(&Theme::nord());
    assert!(wi.needs_render());
}

#[test]
fn test_widget_inspector_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let wi = WidgetInspector::new(WidgetId::new(1)).with_theme(t);
            let _ = wi.render(Rect::new(0, 0, 80, 30));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_widget_inspector_render_twice() {
    let wi = WidgetInspector::new(WidgetId::new(1));
    let _ = wi.render(Rect::new(0, 0, 80, 30));
    let _ = wi.render(Rect::new(0, 0, 80, 30));
}

#[test]
fn test_widget_inspector_set_area_then_render() {
    let mut wi = WidgetInspector::new(WidgetId::new(1));
    wi.set_area(Rect::new(0, 0, 100, 40));
    let plane = wi.render(Rect::new(0, 0, 100, 40));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_widget_inspector_with_hierarchy() {
    let mut wi = WidgetInspector::new(WidgetId::new(1));
    let nodes = vec![
        WidgetNode::new(WidgetId::new(1), "Root"),
        WidgetNode::new(WidgetId::new(2), "Child1"),
        WidgetNode::new(WidgetId::new(3), "Child2"),
    ];
    wi.set_hierarchy(nodes);
    let plane = wi.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}

#[test]
fn test_widget_inspector_with_empty_hierarchy() {
    let mut wi = WidgetInspector::new(WidgetId::new(1));
    wi.set_hierarchy(vec![]);
    let plane = wi.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}
