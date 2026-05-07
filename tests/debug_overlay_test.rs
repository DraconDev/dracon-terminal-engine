//! Debug overlay tests — widget inspector hierarchy display.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{DebugOverlay, WidgetInspector, WidgetNode};
use ratatui::layout::Rect;

#[test]
fn test_widget_inspector_new() {
    let inspector = WidgetInspector::new(WidgetId::new(1));
    let plane = inspector.render(Rect::new(0, 0, 60, 20));
    assert_eq!(plane.width, 60);
    assert_eq!(plane.height, 20);
}

#[test]
fn test_widget_inspector_empty_hierarchy() {
    let inspector = WidgetInspector::new(WidgetId::new(1));
    let plane = inspector.render(Rect::new(0, 0, 60, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_widget_inspector_with_hierarchy() {
    let mut inspector = WidgetInspector::new(WidgetId::new(1));
    let nodes = vec![
        WidgetNode::new(WidgetId::new(1), "Root"),
        WidgetNode::new(WidgetId::new(2), "Child 1"),
        WidgetNode::new(WidgetId::new(3), "Child 2"),
    ];
    inspector.set_hierarchy(nodes);
    let plane = inspector.render(Rect::new(0, 0, 60, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_widget_inspector_nested_nodes() {
    let mut inspector = WidgetInspector::new(WidgetId::new(1));
    let mut root = WidgetNode::new(WidgetId::new(1), "Root");
    root.children.push(WidgetNode::new(WidgetId::new(2), "Child"));
    inspector.set_hierarchy(vec![root]);
    let plane = inspector.render(Rect::new(0, 0, 60, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_widget_inspector_theme_change() {
    let mut inspector = WidgetInspector::new(WidgetId::new(1));
    inspector.on_theme_change(&Theme::cyberpunk());
    let plane = inspector.render(Rect::new(0, 0, 60, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_widget_inspector_no_black_background() {
    let inspector = WidgetInspector::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = inspector.render(Rect::new(0, 0, 60, 20));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_debug_overlay_new() {
    let overlay = DebugOverlay::new(WidgetId::new(1));
    let plane = overlay.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_debug_overlay_theme_change() {
    let mut overlay = DebugOverlay::new(WidgetId::new(1));
    overlay.on_theme_change(&Theme::cyberpunk());
    let plane = overlay.render(Rect::new(0, 0, 80, 24));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_debug_overlay_no_black_background() {
    let overlay = DebugOverlay::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = overlay.render(Rect::new(0, 0, 80, 24));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_widget_node_new() {
    let node = WidgetNode::new(WidgetId::new(1), "Test");
    assert_eq!(node.label, "Test");
    assert!(node.children.is_empty());
}

#[test]
fn test_widget_node_with_children() {
    let mut node = WidgetNode::new(WidgetId::new(1), "Root");
    node.children.push(WidgetNode::new(WidgetId::new(2), "Child"));
    assert_eq!(node.children.len(), 1);
}
