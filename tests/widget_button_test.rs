//! Tests for the Button widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Button;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_button_new() {
    let b = Button::new("Click Me");
    let area = b.area();
    assert!(area.width > 0);
}

#[test]
fn test_button_new_with_id() {
    let b = Button::with_id(WidgetId::new(42), "Test");
    assert_eq!(b.id(), WidgetId::new(42));
}

#[test]
fn test_button_with_theme() {
    let b = Button::new("Test").with_theme(Theme::nord());
    let plane = b.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_button_id() {
    let b = Button::with_id(WidgetId::new(42), "Test");
    assert_eq!(b.id(), WidgetId::new(42));
}

#[test]
fn test_button_set_id() {
    let mut b = Button::new("Test");
    b.set_id(WidgetId::new(99));
    assert_eq!(b.id(), WidgetId::new(99));
}

#[test]
fn test_button_area() {
    let b = Button::new("Test");
    let area = b.area();
    assert!(area.width > 0);
}

#[test]
fn test_button_set_area() {
    let mut b = Button::new("Test");
    b.set_area(Rect::new(0, 0, 30, 2));
    assert_eq!(b.area(), Rect::new(0, 0, 30, 2));
}

#[test]
fn test_button_needs_render() {
    let b = Button::new("Test");
    assert!(b.needs_render());
}

#[test]
fn test_button_mark_dirty() {
    let mut b = Button::new("Test");
    b.clear_dirty();
    assert!(!b.needs_render());
    b.mark_dirty();
    assert!(b.needs_render());
}

#[test]
fn test_button_clear_dirty() {
    let mut b = Button::new("Test");
    b.clear_dirty();
    assert!(!b.needs_render());
}

#[test]
fn test_button_default_dirty() {
    let b = Button::new("Test");
    assert!(b.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_button_render_basic() {
    let b = Button::new("Click Me");
    let plane = b.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_button_render_has_content() {
    let b = Button::new("Test");
    let plane = b.render(Rect::new(0, 0, 20, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_button_render_wide() {
    let b = Button::new("Test");
    let plane = b.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_button_render_small() {
    let b = Button::new("X");
    let plane = b.render(Rect::new(0, 0, 5, 1));
    assert_eq!(plane.width, 5);
}

#[test]
fn test_button_render_tall() {
    let b = Button::new("Test");
    let plane = b.render(Rect::new(0, 0, 20, 3));
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_button_theme_nord() {
    let b = Button::new("Test").with_theme(Theme::nord());
    let plane = b.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_button_theme_dracula() {
    let b = Button::new("Test").with_theme(Theme::dracula());
    let plane = b.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_button_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let b = Button::new("Test").with_theme(t);
        let _ = b.render(Rect::new(0, 0, 20, 1));
    }
}

#[test]
fn test_button_on_theme_change() {
    let mut b = Button::new("Test");
    b.on_theme_change(&Theme::nord());
    assert!(b.needs_render());
}

#[test]
fn test_button_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let b = Button::new("Test").with_theme(t);
            let _ = b.render(Rect::new(0, 0, 20, 1));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_button_render_twice() {
    let b = Button::new("Test");
    let _ = b.render(Rect::new(0, 0, 20, 1));
    let _ = b.render(Rect::new(0, 0, 20, 1));
}

#[test]
fn test_button_set_area_then_render() {
    let mut b = Button::new("Test");
    b.set_area(Rect::new(0, 0, 30, 2));
    let plane = b.render(Rect::new(0, 0, 30, 2));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_button_empty_label() {
    let b = Button::new("");
    let plane = b.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_button_long_label() {
    let b = Button::new(&"A".repeat(100));
    let plane = b.render(Rect::new(0, 0, 200, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_button_unicode_label() {
    let b = Button::new("日本語");
    let plane = b.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}
