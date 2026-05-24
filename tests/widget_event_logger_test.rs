//! Tests for the EventLogger widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::event_logger::EventLogger;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_event_logger_new() {
    let el = EventLogger::new(WidgetId::new(1));
    let area = el.area();
    assert!(area.width > 0);
}

#[test]
fn test_event_logger_new_with_id() {
    let el = EventLogger::new(WidgetId::new(42));
    assert_eq!(el.id(), WidgetId::new(42));
}

#[test]
fn test_event_logger_with_theme() {
    let el = EventLogger::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = el.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_event_logger_with_max_events() {
    let el = EventLogger::new(WidgetId::new(1)).with_max_events(100);
    let plane = el.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_event_logger_id() {
    let el = EventLogger::new(WidgetId::new(42));
    assert_eq!(el.id(), WidgetId::new(42));
}

#[test]
fn test_event_logger_set_id() {
    let mut el = EventLogger::new(WidgetId::new(1));
    el.set_id(WidgetId::new(99));
    assert_eq!(el.id(), WidgetId::new(99));
}

#[test]
fn test_event_logger_area() {
    let el = EventLogger::new(WidgetId::new(1));
    let area = el.area();
    assert!(area.width > 0);
}

#[test]
fn test_event_logger_set_area() {
    let mut el = EventLogger::new(WidgetId::new(1));
    el.set_area(Rect::new(0, 0, 100, 30));
    assert_eq!(el.area(), Rect::new(0, 0, 100, 30));
}

#[test]
fn test_event_logger_needs_render() {
    let el = EventLogger::new(WidgetId::new(1));
    assert!(el.needs_render());
}

#[test]
fn test_event_logger_mark_dirty() {
    let mut el = EventLogger::new(WidgetId::new(1));
    el.clear_dirty();
    assert!(!el.needs_render());
    el.mark_dirty();
    assert!(el.needs_render());
}

#[test]
fn test_event_logger_clear_dirty() {
    let mut el = EventLogger::new(WidgetId::new(1));
    el.clear_dirty();
    assert!(!el.needs_render());
}

#[test]
fn test_event_logger_default_dirty() {
    let el = EventLogger::new(WidgetId::new(1));
    assert!(el.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_event_logger_render_basic() {
    let el = EventLogger::new(WidgetId::new(1));
    let plane = el.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_event_logger_render_has_content() {
    let el = EventLogger::new(WidgetId::new(1));
    let plane = el.render(Rect::new(0, 0, 80, 20));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_event_logger_render_wide() {
    let el = EventLogger::new(WidgetId::new(1));
    let plane = el.render(Rect::new(0, 0, 120, 20));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_event_logger_render_small() {
    let el = EventLogger::new(WidgetId::new(1));
    let plane = el.render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_event_logger_render_tall() {
    let el = EventLogger::new(WidgetId::new(1));
    let plane = el.render(Rect::new(0, 0, 80, 50));
    assert_eq!(plane.height, 50);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_event_logger_theme_nord() {
    let el = EventLogger::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = el.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_event_logger_theme_dracula() {
    let el = EventLogger::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = el.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_event_logger_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let el = EventLogger::new(WidgetId::new(1)).with_theme(t);
        let _ = el.render(Rect::new(0, 0, 80, 20));
    }
}

#[test]
fn test_event_logger_on_theme_change() {
    let mut el = EventLogger::new(WidgetId::new(1));
    el.on_theme_change(&Theme::nord());
    assert!(el.needs_render());
}

#[test]
fn test_event_logger_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark", "catppuccin_mocha"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let el = EventLogger::new(WidgetId::new(1)).with_theme(t);
            let _ = el.render(Rect::new(0, 0, 80, 20));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_event_logger_render_twice() {
    let el = EventLogger::new(WidgetId::new(1));
    let _ = el.render(Rect::new(0, 0, 80, 20));
    let _ = el.render(Rect::new(0, 0, 80, 20));
}

#[test]
fn test_event_logger_set_area_then_render() {
    let mut el = EventLogger::new(WidgetId::new(1));
    el.set_area(Rect::new(0, 0, 100, 30));
    let plane = el.render(Rect::new(0, 0, 100, 30));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_event_logger_with_max_events_100() {
    let el = EventLogger::new(WidgetId::new(1)).with_max_events(100);
    let plane = el.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_event_logger_with_max_events_1() {
    let el = EventLogger::new(WidgetId::new(1)).with_max_events(1);
    let plane = el.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}
