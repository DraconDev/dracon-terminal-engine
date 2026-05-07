use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Slider;
use ratatui::layout::Rect;

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
fn test_slider_max_equals_min() {
    let slider = Slider::new(WidgetId::new(1)).with_range(5.0, 5.0);
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_slider_set_value_clamped() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(150.0);
    assert_eq!(slider.value(), 100.0);
    slider.set_value(-10.0);
    assert_eq!(slider.value(), 0.0);
}

#[test]
fn test_slider_set_value_midpoint() {
    let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    slider.set_value(50.0);
    assert_eq!(slider.value(), 50.0);
}

use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_slider_on_change_callback() {
    let last_value = Rc::new(RefCell::new(0.0f32));
    let last_value_clone = Rc::clone(&last_value);
    let mut slider = Slider::new(WidgetId::new(1))
        .with_range(0.0, 100.0)
        .on_change(move |v| *last_value_clone.borrow_mut() = v);
    // set_value doesn't trigger callback - only mouse events do
    slider.set_value(75.0);
    assert_eq!(slider.value(), 75.0);
    assert_eq!(*last_value.borrow(), 0.0); // callback not triggered
}

#[test]
fn test_slider_on_theme_change() {
    let mut slider = Slider::new(WidgetId::new(1));
    let theme = Theme::cyberpunk();
    slider.on_theme_change(&theme);
    let plane = slider.render(Rect::new(0, 0, 20, 1));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_slider_value_default() {
    let slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
    assert_eq!(slider.value(), 50.0);
}

#[test]
fn test_slider_value_default_0_to_1() {
    let slider = Slider::new(WidgetId::new(1));
    assert_eq!(slider.value(), 0.5);
}
