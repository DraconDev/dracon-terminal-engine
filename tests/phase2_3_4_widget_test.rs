//! Integration tests for Phase 2-4 widgets.

use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    DebugOverlay, EventLogger, Form, MenuBar, MenuEntry, MenuItem,
    Profiler, SearchInput, Select, Slider, StatusBar, StatusSegment,
    Toast, ToastKind, Tooltip, Tree, TreeNode, WidgetInspector, WidgetNode,
};
use dracon_terminal_engine::framework::theme::Theme;
use ratatui::layout::Rect;

fn dummy_area() -> Rect {
    Rect::new(0, 0, 80, 20)
}

// ========== Phase 2 Widgets ==========

#[test]
fn test_tree_render() {
    let tree = Tree::new(WidgetId::new(1));
    let plane = tree.render(dummy_area());
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_tree_with_nodes() {
    let root = vec![
        TreeNode::new("Root"),
    ];
    let tree = Tree::new(WidgetId::new(1)).with_root(root);
    let plane = tree.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_search_input_render() {
    let si = SearchInput::new(WidgetId::new(2));
    let plane = si.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_search_input_query() {
    let mut si = SearchInput::new(WidgetId::new(2));
    assert_eq!(si.query(), "");
    si.clear();
    assert_eq!(si.query(), "");
}

#[test]
fn test_form_render() {
    let form = Form::new(WidgetId::new(3))
        .add_field("Username")
        .add_field("Password");
    let plane = form.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_select_render() {
    let select = Select::new(WidgetId::new(4))
        .with_options(vec!["A".to_string(), "B".to_string()]);
    let plane = select.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_select_selected_index() {
    let select = Select::new(WidgetId::new(4))
        .with_options(vec!["A".to_string(), "B".to_string()]);
    assert_eq!(select.selected_index(), 0);
    assert_eq!(select.selected_label(), Some("A"));
}

#[test]
fn test_slider_render() {
    let slider = Slider::new(WidgetId::new(5));
    let plane = slider.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_slider_with_range() {
    let mut slider = Slider::new(WidgetId::new(5)).with_range(0.0, 100.0);
    assert!((slider.value() - 50.0).abs() < 0.001);
    slider.set_value(75.0);
    assert!((slider.value() - 75.0).abs() < 0.001);
}

// ========== Phase 3 Widgets ==========

#[test]
fn test_tooltip_render() {
    let tt = Tooltip::new(WidgetId::new(6), "Help text");
    let plane = tt.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_tooltip_text() {
    let tt = Tooltip::new(WidgetId::new(6), "Help text");
    assert_eq!(tt.text(), "Help text");
}

#[test]
fn test_toast_render() {
    let toast = Toast::new(WidgetId::new(7), "Done").with_kind(ToastKind::Success);
    let plane = toast.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_toast_message() {
    let toast = Toast::new(WidgetId::new(7), "Done");
    assert_eq!(toast.message(), "Done");
}

#[test]
fn test_toast_not_expired_immediately() {
    let toast = Toast::new(WidgetId::new(7), "Done");
    assert!(!toast.is_expired());
}

#[test]
fn test_status_bar_render() {
    let sb = StatusBar::new(WidgetId::new(8))
        .add_segment(StatusSegment::new("Ready"));
    let plane = sb.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_menu_bar_render() {
    let mb = MenuBar::new(WidgetId::new(9))
        .with_entries(vec![
            MenuEntry::new("File").add_item(MenuItem::new("Open")),
        ]);
    let plane = mb.render(dummy_area());
    assert!(plane.width > 0);
}

// ========== Phase 4 Widgets ==========

#[test]
fn test_debug_overlay_render() {
    let mut overlay = DebugOverlay::new(WidgetId::new(10));
    overlay.add_line("Debug info");
    let plane = overlay.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_debug_overlay_clear() {
    let mut overlay = DebugOverlay::new(WidgetId::new(10));
    overlay.add_line("Line 1");
    overlay.clear();
    let plane = overlay.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_widget_inspector_render() {
    let inspector = WidgetInspector::new(WidgetId::new(11));
    let plane = inspector.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_widget_inspector_with_hierarchy() {
    let mut inspector = WidgetInspector::new(WidgetId::new(11));
    let nodes = vec![
        WidgetNode::new(WidgetId::new(1), "Root"),
    ];
    inspector.set_hierarchy(nodes);
    let plane = inspector.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_event_logger_render() {
    let mut logger = EventLogger::new(WidgetId::new(12));
    logger.log("12:00", "Key pressed");
    let plane = logger.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_event_logger_clear() {
    let mut logger = EventLogger::new(WidgetId::new(12));
    logger.log("12:00", "Event");
    logger.clear();
    let plane = logger.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_profiler_render() {
    let mut profiler = Profiler::new(WidgetId::new(13));
    profiler.record("render", std::time::Duration::from_millis(16), 1);
    let plane = profiler.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_widget_with_theme_phase2() {
    let tree = Tree::new(WidgetId::new(14)).with_theme(Theme::dark());
    let plane = tree.render(dummy_area());
    assert!(plane.width > 0);
}