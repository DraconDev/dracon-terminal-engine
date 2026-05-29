//! Tests for the Checkbox widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Checkbox;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_checkbox_new() {
    let cb = Checkbox::new(WidgetId::new(1), "Option A");
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_new_with_id() {
    let cb = Checkbox::new(WidgetId::new(42), "Test");
    assert_eq!(cb.id(), WidgetId::new(42));
}

#[test]
fn test_checkbox_new_with_label() {
    let cb = Checkbox::new(WidgetId::new(1), "My Label");
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_with_theme() {
    let cb = Checkbox::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_default_unchecked() {
    let cb = Checkbox::new(WidgetId::new(1), "Option");
    assert!(!cb.is_checked());
}

// ============================================================================
// Check/Uncheck Tests
// ============================================================================

#[test]
fn test_checkbox_check() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Option");
    cb.check();
    assert!(cb.is_checked());
}

#[test]
fn test_checkbox_uncheck() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Option");
    cb.check();
    cb.uncheck();
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_toggle() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Option");
    assert!(!cb.is_checked());
    cb.toggle();
    assert!(cb.is_checked());
    cb.toggle();
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_check_twice() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Option");
    cb.check();
    cb.check();
    assert!(cb.is_checked());
}

#[test]
fn test_checkbox_uncheck_twice() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Option");
    cb.uncheck();
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_is_checked() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Option");
    assert!(!cb.is_checked());
    cb.check();
    assert!(cb.is_checked());
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_checkbox_id() {
    let cb = Checkbox::new(WidgetId::new(42), "Test");
    assert_eq!(cb.id(), WidgetId::new(42));
}

#[test]
fn test_checkbox_set_id() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.set_id(WidgetId::new(99));
    assert_eq!(cb.id(), WidgetId::new(99));
}

#[test]
fn test_checkbox_area() {
    let cb = Checkbox::new(WidgetId::new(1), "Test");
    let area = cb.area();
    assert!(area.width > 0);
}

#[test]
fn test_checkbox_set_area() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.set_area(Rect::new(10, 5, 30, 2));
    assert_eq!(cb.area(), Rect::new(10, 5, 30, 2));
}

#[test]
fn test_checkbox_needs_render() {
    let cb = Checkbox::new(WidgetId::new(1), "Test");
    assert!(cb.needs_render());
}

#[test]
fn test_checkbox_mark_dirty() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.clear_dirty();
    assert!(!cb.needs_render());
    cb.mark_dirty();
    assert!(cb.needs_render());
}

#[test]
fn test_checkbox_clear_dirty() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.clear_dirty();
    assert!(!cb.needs_render());
}

#[test]
fn test_checkbox_clear_dirty_after_check() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.clear_dirty();
    cb.check();
    assert!(cb.needs_render());
}

#[test]
fn test_checkbox_default_dirty() {
    let cb = Checkbox::new(WidgetId::new(1), "Test");
    assert!(cb.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_checkbox_render_basic() {
    let cb = Checkbox::new(WidgetId::new(1), "Option A");
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_checkbox_render_has_content() {
    let cb = Checkbox::new(WidgetId::new(1), "Test");
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_checkbox_render_fills_bg() {
    let cb = Checkbox::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.cells[0].bg, Theme::nord().bg);
}

#[test]
fn test_checkbox_render_checked() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.check();
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_render_unchecked() {
    let cb = Checkbox::new(WidgetId::new(1), "Test");
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_render_wide() {
    let cb = Checkbox::new(WidgetId::new(1), "Long Option Label");
    let plane = cb.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_checkbox_render_small() {
    let cb = Checkbox::new(WidgetId::new(1), "X");
    let plane = cb.render(Rect::new(0, 0, 5, 1));
    assert_eq!(plane.width, 5);
}

#[test]
fn test_checkbox_render_tall() {
    let cb = Checkbox::new(WidgetId::new(1), "Test");
    let plane = cb.render(Rect::new(0, 0, 20, 3));
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_checkbox_theme_nord() {
    let cb = Checkbox::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_theme_dracula() {
    let cb = Checkbox::new(WidgetId::new(1), "Test").with_theme(Theme::dracula());
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let cb = Checkbox::new(WidgetId::new(1), "Test").with_theme(t);
        let _ = cb.render(Rect::new(0, 0, 20, 1));
    }
}

#[test]
fn test_checkbox_on_theme_change() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.on_theme_change(&Theme::nord());
    assert!(cb.needs_render());
}

#[test]
fn test_checkbox_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let cb = Checkbox::new(WidgetId::new(1), "Test").with_theme(t);
            let _ = cb.render(Rect::new(0, 0, 20, 1));
        }
    }
}

// ============================================================================
// Label Tests
// ============================================================================

#[test]
fn test_checkbox_empty_label() {
    let cb = Checkbox::new(WidgetId::new(1), "");
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_long_label() {
    let long_label = "A".repeat(100);
    let cb = Checkbox::new(WidgetId::new(1), &long_label);
    let plane = cb.render(Rect::new(0, 0, 200, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_unicode_label() {
    let cb = Checkbox::new(WidgetId::new(1), "日本語");
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_emoji_label() {
    let cb = Checkbox::new(WidgetId::new(1), "🎉 Party!");
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_checkbox_render_twice() {
    let cb = Checkbox::new(WidgetId::new(1), "Test");
    let _ = cb.render(Rect::new(0, 0, 20, 1));
    let _ = cb.render(Rect::new(0, 0, 20, 1));
}

#[test]
fn test_checkbox_check_and_render() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.check();
    let plane = cb.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_checkbox_set_area_then_render() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.set_area(Rect::new(0, 0, 30, 2));
    let plane = cb.render(Rect::new(0, 0, 30, 2));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_checkbox_toggle_cycle() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    assert!(!cb.is_checked());
    cb.toggle();
    assert!(cb.is_checked());
    cb.toggle();
    assert!(!cb.is_checked());
    cb.toggle();
    assert!(cb.is_checked());
}

#[test]
fn test_checkbox_check_uncheck_cycle() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.check();
    assert!(cb.is_checked());
    cb.uncheck();
    assert!(!cb.is_checked());
    cb.check();
    assert!(cb.is_checked());
}
