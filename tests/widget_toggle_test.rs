//! Tests for the Toggle widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Toggle;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_toggle_new() {
    let toggle = Toggle::new(WidgetId::new(1), "Feature");
    assert!(!toggle.is_on());
}

#[test]
fn test_toggle_new_with_id() {
    let toggle = Toggle::new(WidgetId::new(42), "Test");
    assert_eq!(toggle.id(), WidgetId::new(42));
}

#[test]
fn test_toggle_new_with_label() {
    let toggle = Toggle::new(WidgetId::new(1), "My Label");
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_with_theme() {
    let toggle = Toggle::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_default_off() {
    let toggle = Toggle::new(WidgetId::new(1), "Option");
    assert!(!toggle.is_on());
}

// ============================================================================
// Toggle Tests
// ============================================================================

#[test]
fn test_toggle_toggle_on() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Feature");
    toggle.toggle();
    assert!(toggle.is_on());
}

#[test]
fn test_toggle_toggle_off() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Feature");
    toggle.toggle();
    toggle.toggle();
    assert!(!toggle.is_on());
}

#[test]
fn test_toggle_multiple_toggles() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Feature");
    assert!(!toggle.is_on());
    toggle.toggle();
    assert!(toggle.is_on());
    toggle.toggle();
    assert!(!toggle.is_on());
    toggle.toggle();
    assert!(toggle.is_on());
}

#[test]
fn test_toggle_is_on() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Feature");
    assert!(!toggle.is_on());
    toggle.toggle();
    assert!(toggle.is_on());
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_toggle_id() {
    let toggle = Toggle::new(WidgetId::new(42), "Test");
    assert_eq!(toggle.id(), WidgetId::new(42));
}

#[test]
fn test_toggle_set_id() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.set_id(WidgetId::new(99));
    assert_eq!(toggle.id(), WidgetId::new(99));
}

#[test]
fn test_toggle_area() {
    let toggle = Toggle::new(WidgetId::new(1), "Test");
    let area = toggle.area();
    assert!(area.width > 0);
}

#[test]
fn test_toggle_set_area() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.set_area(Rect::new(10, 5, 30, 2));
    assert_eq!(toggle.area(), Rect::new(10, 5, 30, 2));
}

#[test]
fn test_toggle_needs_render() {
    let toggle = Toggle::new(WidgetId::new(1), "Test");
    assert!(toggle.needs_render());
}

#[test]
fn test_toggle_mark_dirty() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.clear_dirty();
    assert!(!toggle.needs_render());
    toggle.mark_dirty();
    assert!(toggle.needs_render());
}

#[test]
fn test_toggle_clear_dirty() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.clear_dirty();
    assert!(!toggle.needs_render());
}

#[test]
fn test_toggle_clear_dirty_after_toggle() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.clear_dirty();
    toggle.toggle();
    assert!(toggle.needs_render());
}

#[test]
fn test_toggle_default_dirty() {
    let toggle = Toggle::new(WidgetId::new(1), "Test");
    assert!(toggle.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_toggle_render_basic() {
    let toggle = Toggle::new(WidgetId::new(1), "Feature A");
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_toggle_render_has_content() {
    let toggle = Toggle::new(WidgetId::new(1), "Test");
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_toggle_render_fills_bg() {
    let toggle = Toggle::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.cells[0].bg, Theme::nord().bg);
}

#[test]
fn test_toggle_render_on() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.toggle();
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_render_off() {
    let toggle = Toggle::new(WidgetId::new(1), "Test");
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_render_wide() {
    let toggle = Toggle::new(WidgetId::new(1), "Long Feature Label");
    let plane = toggle.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_toggle_render_small() {
    let toggle = Toggle::new(WidgetId::new(1), "X");
    let plane = toggle.render(Rect::new(0, 0, 5, 1));
    assert_eq!(plane.width, 5);
}

#[test]
fn test_toggle_render_tall() {
    let toggle = Toggle::new(WidgetId::new(1), "Test");
    let plane = toggle.render(Rect::new(0, 0, 20, 3));
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_toggle_theme_nord() {
    let toggle = Toggle::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_theme_dracula() {
    let toggle = Toggle::new(WidgetId::new(1), "Test").with_theme(Theme::dracula());
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let toggle = Toggle::new(WidgetId::new(1), "Test").with_theme(t);
        let _ = toggle.render(Rect::new(0, 0, 20, 1));
    }
}

#[test]
fn test_toggle_on_theme_change() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.on_theme_change(&Theme::nord());
    assert!(toggle.needs_render());
}

#[test]
fn test_toggle_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark", "catppuccin_mocha"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let toggle = Toggle::new(WidgetId::new(1), "Test").with_theme(t);
            let _ = toggle.render(Rect::new(0, 0, 20, 1));
        }
    }
}

// ============================================================================
// Label Tests
// ============================================================================

#[test]
fn test_toggle_empty_label() {
    let toggle = Toggle::new(WidgetId::new(1), "");
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_long_label() {
    let long_label = "A".repeat(100);
    let toggle = Toggle::new(WidgetId::new(1), &long_label);
    let plane = toggle.render(Rect::new(0, 0, 200, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_unicode_label() {
    let toggle = Toggle::new(WidgetId::new(1), "日本語");
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_emoji_label() {
    let toggle = Toggle::new(WidgetId::new(1), "🎉 Party!");
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_toggle_render_twice() {
    let toggle = Toggle::new(WidgetId::new(1), "Test");
    let _ = toggle.render(Rect::new(0, 0, 20, 1));
    let _ = toggle.render(Rect::new(0, 0, 20, 1));
}

#[test]
fn test_toggle_toggle_and_render() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.toggle();
    let plane = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_set_area_then_render() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    toggle.set_area(Rect::new(0, 0, 30, 2));
    let plane = toggle.render(Rect::new(0, 0, 30, 2));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_toggle_state_cycle() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Test");
    assert!(!toggle.is_on());
    toggle.toggle();
    assert!(toggle.is_on());
    toggle.toggle();
    assert!(!toggle.is_on());
    toggle.toggle();
    assert!(toggle.is_on());
    toggle.toggle();
    assert!(!toggle.is_on());
}

#[test]
fn test_toggle_initial_state_preserved() {
    let toggle = Toggle::new(WidgetId::new(1), "Test");
    // Should be off by default
    assert!(!toggle.is_on());
    // Render should not change state
    let _ = toggle.render(Rect::new(0, 0, 20, 1));
    assert!(!toggle.is_on());
}
