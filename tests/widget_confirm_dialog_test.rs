//! Tests for the ConfirmDialog widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::ConfirmDialog;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_confirm_dialog_new() {
    let d = ConfirmDialog::new("Confirm", "Are you sure?");
    let area = d.area();
    assert!(area.width > 0);
}

#[test]
fn test_confirm_dialog_new_with_id() {
    let d = ConfirmDialog::with_id(WidgetId::new(42), "Title", "Message");
    assert_eq!(d.id(), WidgetId::new(42));
}

#[test]
fn test_confirm_dialog_with_theme() {
    let d = ConfirmDialog::new("Test", "Message").with_theme(Theme::nord());
    let plane = d.render(Rect::new(0, 0, 50, 10));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_confirm_dialog_id() {
    let d = ConfirmDialog::with_id(WidgetId::new(42), "Test", "Msg");
    assert_eq!(d.id(), WidgetId::new(42));
}

#[test]
fn test_confirm_dialog_set_id() {
    let mut d = ConfirmDialog::new("Test", "Msg");
    d.set_id(WidgetId::new(99));
    assert_eq!(d.id(), WidgetId::new(99));
}

#[test]
fn test_confirm_dialog_area() {
    let d = ConfirmDialog::new("Test", "Msg");
    let area = d.area();
    assert!(area.width > 0);
}

#[test]
fn test_confirm_dialog_set_area() {
    let mut d = ConfirmDialog::new("Test", "Msg");
    d.set_area(Rect::new(0, 0, 80, 20));
    assert_eq!(d.area(), Rect::new(0, 0, 80, 20));
}

#[test]
fn test_confirm_dialog_needs_render() {
    let d = ConfirmDialog::new("Test", "Msg");
    assert!(d.needs_render());
}

#[test]
fn test_confirm_dialog_mark_dirty() {
    let mut d = ConfirmDialog::new("Test", "Msg");
    d.clear_dirty();
    assert!(!d.needs_render());
    d.mark_dirty();
    assert!(d.needs_render());
}

#[test]
fn test_confirm_dialog_clear_dirty() {
    let mut d = ConfirmDialog::new("Test", "Msg");
    d.clear_dirty();
    assert!(!d.needs_render());
}

#[test]
fn test_confirm_dialog_default_dirty() {
    let d = ConfirmDialog::new("Test", "Msg");
    assert!(d.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_confirm_dialog_render_basic() {
    let d = ConfirmDialog::new("Confirm", "Delete this?");
    let plane = d.render(Rect::new(0, 0, 50, 10));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_confirm_dialog_render_has_content() {
    let d = ConfirmDialog::new("Test", "Msg");
    let plane = d.render(Rect::new(0, 0, 50, 10));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_confirm_dialog_render_wide() {
    let d = ConfirmDialog::new("Test", "Msg");
    let plane = d.render(Rect::new(0, 0, 80, 10));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_confirm_dialog_render_small() {
    let d = ConfirmDialog::new("T", "M");
    let plane = d.render(Rect::new(0, 0, 20, 5));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_confirm_dialog_render_tall() {
    let d = ConfirmDialog::new("Test", "Msg");
    let plane = d.render(Rect::new(0, 0, 50, 20));
    assert_eq!(plane.height, 20);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_confirm_dialog_theme_nord() {
    let d = ConfirmDialog::new("Test", "Msg").with_theme(Theme::nord());
    let plane = d.render(Rect::new(0, 0, 50, 10));
    assert!(plane.width > 0);
}

#[test]
fn test_confirm_dialog_theme_dracula() {
    let d = ConfirmDialog::new("Test", "Msg").with_theme(Theme::dracula());
    let plane = d.render(Rect::new(0, 0, 50, 10));
    assert!(plane.width > 0);
}

#[test]
fn test_confirm_dialog_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let d = ConfirmDialog::new("Test", "Msg").with_theme(t);
        let _ = d.render(Rect::new(0, 0, 50, 10));
    }
}

#[test]
fn test_confirm_dialog_on_theme_change() {
    let mut d = ConfirmDialog::new("Test", "Msg");
    d.on_theme_change(&Theme::nord());
    assert!(d.needs_render());
}

#[test]
fn test_confirm_dialog_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let d = ConfirmDialog::new("Test", "Msg").with_theme(t);
            let _ = d.render(Rect::new(0, 0, 50, 10));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_confirm_dialog_render_twice() {
    let d = ConfirmDialog::new("Test", "Msg");
    let _ = d.render(Rect::new(0, 0, 50, 10));
    let _ = d.render(Rect::new(0, 0, 50, 10));
}

#[test]
fn test_confirm_dialog_set_area_then_render() {
    let mut d = ConfirmDialog::new("Test", "Msg");
    d.set_area(Rect::new(0, 0, 80, 20));
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_confirm_dialog_empty_title() {
    let d = ConfirmDialog::new("", "Message");
    let plane = d.render(Rect::new(0, 0, 50, 10));
    assert!(plane.width > 0);
}

#[test]
fn test_confirm_dialog_empty_message() {
    let d = ConfirmDialog::new("Title", "");
    let plane = d.render(Rect::new(0, 0, 50, 10));
    assert!(plane.width > 0);
}

#[test]
fn test_confirm_dialog_long_text() {
    let long = "A".repeat(100);
    let d = ConfirmDialog::new(&long, &long);
    let plane = d.render(Rect::new(0, 0, 50, 10));
    assert!(plane.width > 0);
}

#[test]
fn test_confirm_dialog_clear_result() {
    let mut d = ConfirmDialog::new("Test", "Msg");
    d.clear_result();
    let _ = d.render(Rect::new(0, 0, 50, 10));
}
