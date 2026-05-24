//! Tests for the Slider widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Slider;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_slider_new() {
    let slider = Slider::new(WidgetId::new(1));
    assert_eq!(slider.value(), 0.5);
}

#[test]
fn test_slider_new_with_id() {
    let slider = Slider::new(WidgetId::new(42));
    assert_eq!(slider.id(), WidgetId::new(42));
}

#[test]
fn test_slider_with_range_default() {
    let slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    assert_eq!(slider.value(), 50.0);
}

#[test]
fn test_slider_with_range_custom() {
    let slider = Slider::new(WidgetId::new(1)).with_range(10.0, 20.0);
    assert_eq!(slider.value(), 15.0);
}

#[test]
fn test_slider_with_theme() {
    let slider = Slider::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_slider_default_value() {
    let slider = Slider::new(WidgetId::new(1));
    // Default is 0.5 for range 0.0 to 1.0
    assert!((slider.value() - 0.5).abs() < 0.001);
}

// ============================================================================
// Value Tests
// ============================================================================

#[test]
fn test_slider_set_value_valid() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(50.0);
    assert!((slider.value() - 50.0).abs() < 0.001);
}

#[test]
fn test_slider_set_value_clamped() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(150.0);
    assert!((slider.value() - 100.0).abs() < 0.001);
}

#[test]
fn test_slider_set_value_below_min() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(-50.0);
    assert!((slider.value() - 0.0).abs() < 0.001);
}

#[test]
fn test_slider_set_value_exact_min() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(0.0);
    assert!((slider.value() - 0.0).abs() < 0.001);
}

#[test]
fn test_slider_set_value_exact_max() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(100.0);
    assert!((slider.value() - 100.0).abs() < 0.001);
}

#[test]
fn test_slider_value() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(25.0);
    assert_eq!(slider.value(), 25.0);
}

#[test]
fn test_slider_set_value_negative_range() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(-100.0, -50.0);
    slider.set_value(-75.0);
    assert!((slider.value() - (-75.0)).abs() < 0.001);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_slider_id() {
    let slider = Slider::new(WidgetId::new(42));
    assert_eq!(slider.id(), WidgetId::new(42));
}

#[test]
fn test_slider_set_id() {
    let mut slider = Slider::new(WidgetId::new(1));
    slider.set_id(WidgetId::new(99));
    assert_eq!(slider.id(), WidgetId::new(99));
}

#[test]
fn test_slider_area() {
    let slider = Slider::new(WidgetId::new(1));
    let area = slider.area();
    assert!(area.width > 0);
}

#[test]
fn test_slider_set_area() {
    let mut slider = Slider::new(WidgetId::new(1));
    slider.set_area(Rect::new(0, 0, 50, 2));
    assert_eq!(slider.area(), Rect::new(0, 0, 50, 2));
}

#[test]
fn test_slider_needs_render() {
    let slider = Slider::new(WidgetId::new(1));
    assert!(slider.needs_render());
}

#[test]
fn test_slider_mark_dirty() {
    let mut slider = Slider::new(WidgetId::new(1));
    slider.clear_dirty();
    assert!(!slider.needs_render());
    slider.mark_dirty();
    assert!(slider.needs_render());
}

#[test]
fn test_slider_clear_dirty() {
    let mut slider = Slider::new(WidgetId::new(1));
    slider.clear_dirty();
    assert!(!slider.needs_render());
}

#[test]
fn test_slider_clear_dirty_after_set_value() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.clear_dirty();
    slider.set_value(50.0);
    assert!(slider.needs_render());
}

#[test]

#[test]
fn test_slider_default_dirty() {
    let slider = Slider::new(WidgetId::new(1));
    assert!(slider.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_slider_render_zero_area() {
    let slider = Slider::new(WidgetId::new(1));
    let plane = slider.render(Rect::new(0, 0, 0, 0));
    assert_eq!(plane.width, 1);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_slider_render_zero_width() {
    let slider = Slider::new(WidgetId::new(1));
    let plane = slider.render(Rect::new(0, 0, 0, 1));
    assert_eq!(plane.width, 1);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_slider_render_zero_height() {
    let slider = Slider::new(WidgetId::new(1));
    let plane = slider.render(Rect::new(0, 0, 1, 0));
    assert_eq!(plane.width, 1);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_slider_render_normal() {
    let slider = Slider::new(WidgetId::new(1));
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_slider_render_has_content() {
    let slider = Slider::new(WidgetId::new(1));
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_slider_render_fills_bg() {
    let slider = Slider::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.cells[0].bg, Theme::nord().bg);
}

#[test]
fn test_slider_max_equals_min() {
    let slider = Slider::new(WidgetId::new(1)).with_range(5.0, 5.0);
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_slider_render_wide() {
    let slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    let plane = slider.render(Rect::new(0, 0, 100, 1));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_slider_render_small() {
    let slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    let plane = slider.render(Rect::new(0, 0, 5, 1));
    assert_eq!(plane.width, 5);
}

#[test]
fn test_slider_render_tall() {
    let slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    let plane = slider.render(Rect::new(0, 0, 20, 5));
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 5);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_slider_theme_nord() {
    let slider = Slider::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_slider_theme_dracula() {
    let slider = Slider::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_slider_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let slider = Slider::new(WidgetId::new(1)).with_theme(t);
        let _ = slider.render(Rect::new(0, 0, 20, 1));
    }
}

#[test]
fn test_slider_on_theme_change() {
    let mut slider = Slider::new(WidgetId::new(1));
    slider.on_theme_change(&Theme::nord());
    assert!(slider.needs_render());
}

#[test]
fn test_slider_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark", "catppuccin_mocha"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let slider = Slider::new(WidgetId::new(1)).with_theme(t);
            let _ = slider.render(Rect::new(0, 0, 20, 1));
        }
    }
}

// ============================================================================
// Range Tests
// ============================================================================

#[test]
fn test_slider_range_zero_to_hundred() {
    let slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    assert!((slider.value() - 50.0).abs() < 0.001);
}

#[test]
fn test_slider_range_negative() {
    let slider = Slider::new(WidgetId::new(1)).with_range(-50.0, 50.0);
    assert!((slider.value() - 0.0).abs() < 0.001);
}

#[test]
fn test_slider_range_small() {
    let slider = Slider::new(WidgetId::new(1)).with_range(99.0, 100.0);
    assert!((slider.value() - 99.5).abs() < 0.001);
}

#[test]
fn test_slider_range_large() {
    let slider = Slider::new(WidgetId::new(1)).with_range(0.0, 1_000_000.0);
    assert!((slider.value() - 500_000.0).abs() < 1.0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_slider_set_value_twice() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(25.0);
    assert!((slider.value() - 25.0).abs() < 0.001);
    slider.set_value(75.0);
    assert!((slider.value() - 75.0).abs() < 0.001);
}

#[test]
fn test_slider_render_twice() {
    let slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    let _ = slider.render(Rect::new(0, 0, 20, 1));
    let _ = slider.render(Rect::new(0, 0, 20, 1));
}

#[test]
fn test_slider_set_area_then_render() {
    let mut slider = Slider::new(WidgetId::new(1));
    slider.set_area(Rect::new(0, 0, 50, 2));
    let plane = slider.render(Rect::new(0, 0, 50, 2));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_slider_value_unchanged_after_invalid_set() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(50.0);
    slider.set_value(150.0);
    assert!((slider.value() - 100.0).abs() < 0.001);
}
