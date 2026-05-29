//! Tests for the Tooltip widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Tooltip;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_tooltip_new() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test tooltip");
    assert_eq!(tooltip.text(), "Test tooltip");
}

#[test]
fn test_tooltip_new_with_id() {
    let tooltip = Tooltip::new(WidgetId::new(42), "Hello");
    assert_eq!(tooltip.id(), WidgetId::new(42));
}

#[test]
fn test_tooltip_with_theme() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = tooltip.render(Rect::new(0, 0, 50, 3));
    assert!(plane.width > 0);
}

#[test]
fn test_tooltip_default_text() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Default");
    assert_eq!(tooltip.text(), "Default");
}

// ============================================================================
// Text Tests
// ============================================================================

#[test]
fn test_tooltip_text() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Some text");
    assert_eq!(tooltip.text(), "Some text");
}

#[test]
fn test_tooltip_empty_text() {
    let tooltip = Tooltip::new(WidgetId::new(1), "");
    assert_eq!(tooltip.text(), "");
}

#[test]
fn test_tooltip_long_text() {
    let long_text = "A".repeat(200);
    let tooltip = Tooltip::new(WidgetId::new(1), &long_text);
    assert_eq!(tooltip.text(), long_text);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_tooltip_id() {
    let tooltip = Tooltip::new(WidgetId::new(42), "Test");
    assert_eq!(tooltip.id(), WidgetId::new(42));
}

#[test]
fn test_tooltip_set_id() {
    let mut tooltip = Tooltip::new(WidgetId::new(1), "Test");
    tooltip.set_id(WidgetId::new(99));
    assert_eq!(tooltip.id(), WidgetId::new(99));
}

#[test]
fn test_tooltip_area() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test");
    let area = tooltip.area();
    assert!(area.width > 0);
}

#[test]
fn test_tooltip_set_area() {
    let mut tooltip = Tooltip::new(WidgetId::new(1), "Test");
    tooltip.set_area(Rect::new(0, 0, 100, 5));
    assert_eq!(tooltip.area(), Rect::new(0, 0, 100, 5));
}

#[test]
fn test_tooltip_needs_render() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test");
    assert!(tooltip.needs_render());
}

#[test]
fn test_tooltip_mark_dirty() {
    let mut tooltip = Tooltip::new(WidgetId::new(1), "Test");
    tooltip.clear_dirty();
    assert!(!tooltip.needs_render());
    tooltip.mark_dirty();
    assert!(tooltip.needs_render());
}

#[test]
fn test_tooltip_clear_dirty() {
    let mut tooltip = Tooltip::new(WidgetId::new(1), "Test");
    tooltip.clear_dirty();
    assert!(!tooltip.needs_render());
}

#[test]
fn test_tooltip_default_dirty() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test");
    assert!(tooltip.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_tooltip_render_basic() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test tooltip");
    let plane = tooltip.render(Rect::new(0, 0, 50, 3));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_tooltip_render_has_content() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test");
    let plane = tooltip.render(Rect::new(0, 0, 50, 3));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_tooltip_render_wide() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test");
    let plane = tooltip.render(Rect::new(0, 0, 100, 3));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_tooltip_render_small() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test");
    let plane = tooltip.render(Rect::new(0, 0, 10, 2));
    assert_eq!(plane.width, 10);
}

#[test]
fn test_tooltip_render_tall() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test");
    let plane = tooltip.render(Rect::new(0, 0, 50, 5));
    assert_eq!(plane.height, 5);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_tooltip_theme_nord() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = tooltip.render(Rect::new(0, 0, 50, 3));
    assert!(plane.width > 0);
}

#[test]
fn test_tooltip_theme_dracula() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test").with_theme(Theme::dracula());
    let plane = tooltip.render(Rect::new(0, 0, 50, 3));
    assert!(plane.width > 0);
}

#[test]
fn test_tooltip_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let tooltip = Tooltip::new(WidgetId::new(1), "Test").with_theme(t);
        let _ = tooltip.render(Rect::new(0, 0, 50, 3));
    }
}

#[test]
fn test_tooltip_on_theme_change() {
    let mut tooltip = Tooltip::new(WidgetId::new(1), "Test");
    tooltip.on_theme_change(&Theme::nord());
    assert!(tooltip.needs_render());
}

#[test]
fn test_tooltip_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let tooltip = Tooltip::new(WidgetId::new(1), "Test").with_theme(t);
            let _ = tooltip.render(Rect::new(0, 0, 50, 3));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_tooltip_render_twice() {
    let tooltip = Tooltip::new(WidgetId::new(1), "Test");
    let _ = tooltip.render(Rect::new(0, 0, 50, 3));
    let _ = tooltip.render(Rect::new(0, 0, 50, 3));
}

#[test]
fn test_tooltip_set_area_then_render() {
    let mut tooltip = Tooltip::new(WidgetId::new(1), "Test");
    tooltip.set_area(Rect::new(0, 0, 80, 4));
    let plane = tooltip.render(Rect::new(0, 0, 80, 4));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_tooltip_unicode_text() {
    let tooltip = Tooltip::new(WidgetId::new(1), "日本語ツールチップ");
    assert_eq!(tooltip.text(), "日本語ツールチップ");
}

#[test]
fn test_tooltip_emoji_text() {
    let tooltip = Tooltip::new(WidgetId::new(1), "🎉 Party!");
    assert_eq!(tooltip.text(), "🎉 Party!");
}
