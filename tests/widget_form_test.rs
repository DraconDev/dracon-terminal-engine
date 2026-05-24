//! Tests for the Form widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::form::{Form, ValidationRule};

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_form_new() {
    let f = Form::new(WidgetId::new(1));
    let area = f.area();
    assert!(area.width > 0);
}

#[test]
fn test_form_new_with_id() {
    let f = Form::new(WidgetId::new(42));
    assert_eq!(f.id(), WidgetId::new(42));
}

#[test]
fn test_form_with_theme() {
    let f = Form::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = f.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_form_add_field() {
    let f = Form::new(WidgetId::new(1)).add_field("Name");
    let plane = f.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_form_multiple_fields() {
    let f = Form::new(WidgetId::new(1))
        .add_field("Name")
        .add_field("Email")
        .add_field("Phone");
    let plane = f.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_form_id() {
    let f = Form::new(WidgetId::new(42));
    assert_eq!(f.id(), WidgetId::new(42));
}

#[test]
fn test_form_set_id() {
    let mut f = Form::new(WidgetId::new(1));
    f.set_id(WidgetId::new(99));
    assert_eq!(f.id(), WidgetId::new(99));
}

#[test]
fn test_form_area() {
    let f = Form::new(WidgetId::new(1));
    let area = f.area();
    assert!(area.width > 0);
}

#[test]
fn test_form_set_area() {
    let mut f = Form::new(WidgetId::new(1));
    f.set_area(Rect::new(0, 0, 100, 30));
    assert_eq!(f.area(), Rect::new(0, 0, 100, 30));
}

#[test]
fn test_form_needs_render() {
    let f = Form::new(WidgetId::new(1));
    assert!(f.needs_render());
}

#[test]
fn test_form_mark_dirty() {
    let mut f = Form::new(WidgetId::new(1));
    f.clear_dirty();
    assert!(!f.needs_render());
    f.mark_dirty();
    assert!(f.needs_render());
}

#[test]
fn test_form_clear_dirty() {
    let mut f = Form::new(WidgetId::new(1));
    f.clear_dirty();
    assert!(!f.needs_render());
}

#[test]
fn test_form_default_dirty() {
    let f = Form::new(WidgetId::new(1));
    assert!(f.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_form_render_basic() {
    let f = Form::new(WidgetId::new(1));
    let plane = f.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_form_render_has_content() {
    let f = Form::new(WidgetId::new(1));
    let plane = f.render(Rect::new(0, 0, 80, 20));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_form_render_wide() {
    let f = Form::new(WidgetId::new(1));
    let plane = f.render(Rect::new(0, 0, 120, 20));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_form_render_small() {
    let f = Form::new(WidgetId::new(1));
    let plane = f.render(Rect::new(0, 0, 30, 10));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_form_render_tall() {
    let f = Form::new(WidgetId::new(1));
    let plane = f.render(Rect::new(0, 0, 80, 50));
    assert_eq!(plane.height, 50);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_form_theme_nord() {
    let f = Form::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = f.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_form_theme_dracula() {
    let f = Form::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = f.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_form_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let f = Form::new(WidgetId::new(1)).with_theme(t);
        let _ = f.render(Rect::new(0, 0, 80, 20));
    }
}

#[test]
fn test_form_on_theme_change() {
    let mut f = Form::new(WidgetId::new(1));
    f.on_theme_change(&Theme::nord());
    assert!(f.needs_render());
}

#[test]
fn test_form_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let f = Form::new(WidgetId::new(1)).with_theme(t);
            let _ = f.render(Rect::new(0, 0, 80, 20));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_form_render_twice() {
    let f = Form::new(WidgetId::new(1));
    let _ = f.render(Rect::new(0, 0, 80, 20));
    let _ = f.render(Rect::new(0, 0, 80, 20));
}

#[test]
fn test_form_set_area_then_render() {
    let mut f = Form::new(WidgetId::new(1));
    f.set_area(Rect::new(0, 0, 100, 30));
    let plane = f.render(Rect::new(0, 0, 100, 30));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_form_many_fields() {
    let mut f = Form::new(WidgetId::new(1));
    for i in 0..15 {
        f = f.add_field(&format!("Field {}", i));
    }
    let plane = f.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_form_empty_label() {
    let f = Form::new(WidgetId::new(1)).add_field("");
    let plane = f.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}
