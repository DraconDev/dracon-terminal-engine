//! Tests for the Label widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Label;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_label_new() {
    let l = Label::new("Hello World");
    let area = l.area();
    assert!(area.width > 0);
}

#[test]
fn test_label_new_with_id() {
    let l = Label::with_id(WidgetId::new(42), "Test");
    assert_eq!(l.id(), WidgetId::new(42));
}

#[test]
fn test_label_with_theme() {
    let l = Label::new("Test").with_theme(Theme::nord());
    let plane = l.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_label_id() {
    let l = Label::with_id(WidgetId::new(42), "Test");
    assert_eq!(l.id(), WidgetId::new(42));
}

#[test]
fn test_label_set_id() {
    let mut l = Label::new("Test");
    l.set_id(WidgetId::new(99));
    assert_eq!(l.id(), WidgetId::new(99));
}

#[test]
fn test_label_area() {
    let l = Label::new("Test");
    let area = l.area();
    assert!(area.width > 0);
}

#[test]
fn test_label_set_area() {
    let mut l = Label::new("Test");
    l.set_area(Rect::new(0, 0, 30, 2));
    assert_eq!(l.area(), Rect::new(0, 0, 30, 2));
}

#[test]
fn test_label_needs_render() {
    let l = Label::new("Test");
    assert!(l.needs_render());
}

#[test]
fn test_label_mark_dirty() {
    let mut l = Label::new("Test");
    l.clear_dirty();
    assert!(!l.needs_render());
    l.mark_dirty();
    assert!(l.needs_render());
}

#[test]
fn test_label_clear_dirty() {
    let mut l = Label::new("Test");
    l.clear_dirty();
    assert!(!l.needs_render());
}

#[test]
fn test_label_default_dirty() {
    let l = Label::new("Test");
    assert!(l.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_label_render_basic() {
    let l = Label::new("Hello");
    let plane = l.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_label_render_has_content() {
    let l = Label::new("Test");
    let plane = l.render(Rect::new(0, 0, 20, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_label_render_wide() {
    let l = Label::new("Test");
    let plane = l.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_label_render_small() {
    let l = Label::new("X");
    let plane = l.render(Rect::new(0, 0, 5, 1));
    assert_eq!(plane.width, 5);
}

#[test]
fn test_label_render_tall() {
    let l = Label::new("Test");
    let plane = l.render(Rect::new(0, 0, 20, 3));
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_label_theme_nord() {
    let l = Label::new("Test").with_theme(Theme::nord());
    let plane = l.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_label_theme_dracula() {
    let l = Label::new("Test").with_theme(Theme::dracula());
    let plane = l.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_label_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let l = Label::new("Test").with_theme(t);
        let _ = l.render(Rect::new(0, 0, 20, 1));
    }
}

#[test]
fn test_label_on_theme_change() {
    let mut l = Label::new("Test");
    l.on_theme_change(&Theme::nord());
    assert!(l.needs_render());
}

#[test]
fn test_label_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let l = Label::new("Test").with_theme(t);
            let _ = l.render(Rect::new(0, 0, 20, 1));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_label_render_twice() {
    let l = Label::new("Test");
    let _ = l.render(Rect::new(0, 0, 20, 1));
    let _ = l.render(Rect::new(0, 0, 20, 1));
}

#[test]
fn test_label_set_area_then_render() {
    let mut l = Label::new("Test");
    l.set_area(Rect::new(0, 0, 30, 2));
    let plane = l.render(Rect::new(0, 0, 30, 2));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_label_empty_text() {
    let l = Label::new("");
    let plane = l.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_label_long_text() {
    let l = Label::new(&"A".repeat(100));
    let plane = l.render(Rect::new(0, 0, 200, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_label_unicode_text() {
    let l = Label::new("日本語");
    let plane = l.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_label_set_text() {
    let mut l = Label::new("Original");
    l.set_text("Changed");
    let plane = l.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}
