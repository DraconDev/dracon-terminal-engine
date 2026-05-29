//! Tests for the StatusBar widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::status_bar::{StatusBar, StatusSegment};

// ============================================================================
// StatusSegment Tests
// ============================================================================

#[test]
#[test]
#[test]
// ============================================================================
// Construction Tests
// ============================================================================
#[test]
fn test_status_bar_new() {
    let sb = StatusBar::new(WidgetId::new(1));
    let area = sb.area();
    assert!(area.width > 0);
}

#[test]
fn test_status_bar_new_with_id() {
    let sb = StatusBar::new(WidgetId::new(42));
    assert_eq!(sb.id(), WidgetId::new(42));
}

#[test]
fn test_status_bar_with_theme() {
    let sb = StatusBar::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_status_bar_add_segment() {
    let sb = StatusBar::new(WidgetId::new(1)).add_segment(StatusSegment::new("Home"));
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_status_bar_multiple_segments() {
    let sb = StatusBar::new(WidgetId::new(1))
        .add_segment(StatusSegment::new("Tab1"))
        .add_segment(StatusSegment::new("Tab2"))
        .add_segment(StatusSegment::new("Tab3"));
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_status_bar_id() {
    let sb = StatusBar::new(WidgetId::new(42));
    assert_eq!(sb.id(), WidgetId::new(42));
}

#[test]
fn test_status_bar_set_id() {
    let mut sb = StatusBar::new(WidgetId::new(1));
    sb.set_id(WidgetId::new(99));
    assert_eq!(sb.id(), WidgetId::new(99));
}

#[test]
fn test_status_bar_area() {
    let sb = StatusBar::new(WidgetId::new(1));
    let area = sb.area();
    assert!(area.width > 0);
}

#[test]
fn test_status_bar_set_area() {
    let mut sb = StatusBar::new(WidgetId::new(1));
    sb.set_area(Rect::new(0, 0, 100, 2));
    assert_eq!(sb.area(), Rect::new(0, 0, 100, 2));
}

#[test]
fn test_status_bar_needs_render() {
    let sb = StatusBar::new(WidgetId::new(1));
    assert!(sb.needs_render());
}

#[test]
fn test_status_bar_mark_dirty() {
    let mut sb = StatusBar::new(WidgetId::new(1));
    sb.clear_dirty();
    assert!(!sb.needs_render());
    sb.mark_dirty();
    assert!(sb.needs_render());
}

#[test]
fn test_status_bar_clear_dirty() {
    let mut sb = StatusBar::new(WidgetId::new(1));
    sb.clear_dirty();
    assert!(!sb.needs_render());
}

#[test]
fn test_status_bar_default_dirty() {
    let sb = StatusBar::new(WidgetId::new(1));
    assert!(sb.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_status_bar_render_basic() {
    let sb = StatusBar::new(WidgetId::new(1));
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_status_bar_render_has_content() {
    let sb = StatusBar::new(WidgetId::new(1));
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_status_bar_render_wide() {
    let sb = StatusBar::new(WidgetId::new(1));
    let plane = sb.render(Rect::new(0, 0, 120, 1));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_status_bar_render_small() {
    let sb = StatusBar::new(WidgetId::new(1));
    let plane = sb.render(Rect::new(0, 0, 10, 1));
    assert_eq!(plane.width, 10);
}

#[test]
fn test_status_bar_render_tall() {
    let sb = StatusBar::new(WidgetId::new(1));
    let plane = sb.render(Rect::new(0, 0, 80, 2));
    assert_eq!(plane.height, 2);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_status_bar_theme_nord() {
    let sb = StatusBar::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_status_bar_theme_dracula() {
    let sb = StatusBar::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_status_bar_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let sb = StatusBar::new(WidgetId::new(1)).with_theme(t);
        let _ = sb.render(Rect::new(0, 0, 80, 1));
    }
}

#[test]
fn test_status_bar_on_theme_change() {
    let mut sb = StatusBar::new(WidgetId::new(1));
    sb.on_theme_change(&Theme::nord());
    assert!(sb.needs_render());
}

#[test]
fn test_status_bar_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let sb = StatusBar::new(WidgetId::new(1)).with_theme(t);
            let _ = sb.render(Rect::new(0, 0, 80, 1));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_status_bar_render_twice() {
    let sb = StatusBar::new(WidgetId::new(1));
    let _ = sb.render(Rect::new(0, 0, 80, 1));
    let _ = sb.render(Rect::new(0, 0, 80, 1));
}

#[test]
fn test_status_bar_set_area_then_render() {
    let mut sb = StatusBar::new(WidgetId::new(1));
    sb.set_area(Rect::new(0, 0, 100, 2));
    let plane = sb.render(Rect::new(0, 0, 100, 2));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_status_bar_many_segments() {
    let mut sb = StatusBar::new(WidgetId::new(1));
    for i in 0..10 {
        sb = sb.add_segment(StatusSegment::new(&format!("Seg{}", i)));
    }
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_status_bar_unicode_segments() {
    let sb = StatusBar::new(WidgetId::new(1))
        .add_segment(StatusSegment::new("ホーム"))
        .add_segment(StatusSegment::new("設定"));
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_status_bar_empty_segments() {
    let sb = StatusBar::new(WidgetId::new(1))
        .add_segment(StatusSegment::new(""))
        .add_segment(StatusSegment::new(""));
    let plane = sb.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}
