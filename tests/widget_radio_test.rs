//! Tests for the Radio widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Radio;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_radio_new() {
    let radio = Radio::new(WidgetId::new(1), "Option A");
    assert!(!radio.is_selected());
}

#[test]
fn test_radio_new_with_id() {
    let radio = Radio::new(WidgetId::new(42), "Test");
    assert_eq!(radio.id(), WidgetId::new(42));
}

#[test]
fn test_radio_new_with_label() {
    let radio = Radio::new(WidgetId::new(1), "My Label");
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_with_theme() {
    let radio = Radio::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_default_not_selected() {
    let radio = Radio::new(WidgetId::new(1), "Option");
    assert!(!radio.is_selected());
}

// ============================================================================
// Selection Tests
// ============================================================================

#[test]
fn test_radio_select() {
    let mut radio = Radio::new(WidgetId::new(1), "Option");
    radio.select();
    assert!(radio.is_selected());
}

#[test]
fn test_radio_deselect() {
    let mut radio = Radio::new(WidgetId::new(1), "Option");
    radio.select();
    radio.deselect();
    assert!(!radio.is_selected());
}

#[test]
fn test_radio_toggle_selection() {
    let mut radio = Radio::new(WidgetId::new(1), "Option");
    assert!(!radio.is_selected());
    radio.select();
    assert!(radio.is_selected());
    radio.deselect();
    assert!(!radio.is_selected());
}

#[test]
fn test_radio_select_twice() {
    let mut radio = Radio::new(WidgetId::new(1), "Option");
    radio.select();
    radio.select();
    assert!(radio.is_selected());
}

#[test]
fn test_radio_deselect_twice() {
    let mut radio = Radio::new(WidgetId::new(1), "Option");
    radio.deselect();
    assert!(!radio.is_selected());
}

#[test]
fn test_radio_is_selected() {
    let mut radio = Radio::new(WidgetId::new(1), "Option");
    assert!(!radio.is_selected());
    radio.select();
    assert!(radio.is_selected());
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_radio_id() {
    let radio = Radio::new(WidgetId::new(42), "Test");
    assert_eq!(radio.id(), WidgetId::new(42));
}

#[test]
fn test_radio_set_id() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.set_id(WidgetId::new(99));
    assert_eq!(radio.id(), WidgetId::new(99));
}

#[test]
fn test_radio_area() {
    let radio = Radio::new(WidgetId::new(1), "Test");
    let area = radio.area();
    assert!(area.width > 0);
}

#[test]
fn test_radio_set_area() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.set_area(Rect::new(10, 5, 30, 2));
    assert_eq!(radio.area(), Rect::new(10, 5, 30, 2));
}

#[test]
fn test_radio_needs_render() {
    let radio = Radio::new(WidgetId::new(1), "Test");
    assert!(radio.needs_render());
}

#[test]
fn test_radio_mark_dirty() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.clear_dirty();
    assert!(!radio.needs_render());
    radio.mark_dirty();
    assert!(radio.needs_render());
}

#[test]
fn test_radio_clear_dirty() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.clear_dirty();
    assert!(!radio.needs_render());
}

#[test]
fn test_radio_clear_dirty_after_select() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.clear_dirty();
    radio.select();
    assert!(radio.needs_render());
}

#[test]
fn test_radio_default_dirty() {
    let radio = Radio::new(WidgetId::new(1), "Test");
    assert!(radio.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_radio_render_basic() {
    let radio = Radio::new(WidgetId::new(1), "Option A");
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_radio_render_has_content() {
    let radio = Radio::new(WidgetId::new(1), "Test");
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_radio_render_fills_bg() {
    let radio = Radio::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.cells[0].bg, Theme::nord().bg);
}

#[test]
fn test_radio_render_selected() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.select();
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_render_unselected() {
    let radio = Radio::new(WidgetId::new(1), "Test");
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_render_wide() {
    let radio = Radio::new(WidgetId::new(1), "Long Option Label");
    let plane = radio.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_radio_render_small() {
    let radio = Radio::new(WidgetId::new(1), "X");
    let plane = radio.render(Rect::new(0, 0, 5, 1));
    assert_eq!(plane.width, 5);
}

#[test]
fn test_radio_render_tall() {
    let radio = Radio::new(WidgetId::new(1), "Test");
    let plane = radio.render(Rect::new(0, 0, 20, 3));
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_radio_theme_nord() {
    let radio = Radio::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_theme_dracula() {
    let radio = Radio::new(WidgetId::new(1), "Test").with_theme(Theme::dracula());
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let radio = Radio::new(WidgetId::new(1), "Test").with_theme(t);
        let _ = radio.render(Rect::new(0, 0, 20, 1));
    }
}

#[test]
fn test_radio_on_theme_change() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.on_theme_change(&Theme::nord());
    assert!(radio.needs_render());
}

#[test]
fn test_radio_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark", "catppuccin_mocha"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let radio = Radio::new(WidgetId::new(1), "Test").with_theme(t);
            let _ = radio.render(Rect::new(0, 0, 20, 1));
        }
    }
}

// ============================================================================
// Label Tests
// ============================================================================

#[test]
fn test_radio_empty_label() {
    let radio = Radio::new(WidgetId::new(1), "");
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_long_label() {
    let long_label = "A".repeat(100);
    let radio = Radio::new(WidgetId::new(1), &long_label);
    let plane = radio.render(Rect::new(0, 0, 200, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_unicode_label() {
    let radio = Radio::new(WidgetId::new(1), "日本語");
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_emoji_label() {
    let radio = Radio::new(WidgetId::new(1), "🎉 Party!");
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_radio_render_twice() {
    let radio = Radio::new(WidgetId::new(1), "Test");
    let _ = radio.render(Rect::new(0, 0, 20, 1));
    let _ = radio.render(Rect::new(0, 0, 20, 1));
}

#[test]
fn test_radio_select_and_render() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.select();
    let plane = radio.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_radio_set_area_then_render() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    radio.set_area(Rect::new(0, 0, 30, 2));
    let plane = radio.render(Rect::new(0, 0, 30, 2));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_radio_select_deselect_cycle() {
    let mut radio = Radio::new(WidgetId::new(1), "Test");
    assert!(!radio.is_selected());
    radio.select();
    assert!(radio.is_selected());
    radio.deselect();
    assert!(!radio.is_selected());
    radio.select();
    assert!(radio.is_selected());
}
