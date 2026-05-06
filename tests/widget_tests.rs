//! Tests for widget components.

// Note: Most widget tests live in widget_test.rs.
// This file contains tests for widgets that have dedicated test files.

mod common;

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::{
    Breadcrumbs, Button, Checkbox, Gauge, Label, PasswordInput, ProgressBar, Radio, SearchInput,
    Slider, Spinner, StatusBadge, StatusBar, Toggle,
};
use dracon_terminal_engine::framework::theme::Theme;
use ratatui::layout::Rect;

#[test]
fn test_button_new() {
    let btn = Button::new("Click me");
    assert_eq!(btn.label, "Click me");
}

#[test]
fn test_button_with_theme() {
    let btn = Button::new("Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 10, 1);
    let _plane = btn.render(area);
}

#[test]
fn test_button_render() {
    let btn = Button::new("Hi");
    let area = Rect::new(0, 0, 20, 1);
    let plane = btn.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_button_handle_mouse_hover() {
    let mut btn = Button::new("Test");
    let area = Rect::new(0, 0, 10, 1);
    btn.set_area(area);
    assert!(!btn.hovered);
    btn.handle_mouse(crate::input::event::MouseEventKind::Moved, 5, 0);
    assert!(btn.hovered);
}

#[test]
fn test_button_handle_mouse_click() {
    let called = std::cell::Cell::new(false);
    let called_clone = called.clone();
    let mut btn = Button::new("Test").on_click(move || {
        called_clone.set(true);
    });
    let area = Rect::new(0, 0, 10, 1);
    btn.set_area(area);
    btn.handle_mouse(crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left), 5, 0);
    assert!(called.get());
}

#[test]
fn test_toggle_new() {
    let toggle = Toggle::new(WidgetId::default_id(), "Enable");
    assert!(!toggle.is_on());
}

#[test]
fn test_toggle_with_theme() {
    let toggle = Toggle::new(WidgetId::default_id(), "Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = toggle.render(area);
}

#[test]
fn test_toggle_render() {
    let toggle = Toggle::new(WidgetId::default_id(), "Test");
    let area = Rect::new(0, 0, 20, 1);
    let plane = toggle.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_toggle_toggle() {
    let mut toggle = Toggle::new(WidgetId::default_id(), "Test");
    assert!(!toggle.is_on());
    toggle.toggle();
    assert!(toggle.is_on());
    toggle.toggle();
    assert!(!toggle.is_on());
}

#[test]
fn test_checkbox_new() {
    let cb = Checkbox::new(WidgetId::default_id(), "Agree");
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_with_theme() {
    let cb = Checkbox::new(WidgetId::default_id(), "Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = cb.render(area);
}

#[test]
fn test_checkbox_render() {
    let cb = Checkbox::new(WidgetId::default_id(), "Test");
    let area = Rect::new(0, 0, 20, 1);
    let plane = cb.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_checkbox_toggle() {
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(!cb.is_checked());
    cb.toggle();
    assert!(cb.is_checked());
    cb.toggle();
    assert!(!cb.is_checked());
}

#[test]
fn test_radio_new() {
    let radio = Radio::new(WidgetId::default_id(), "Option");
    assert!(!radio.is_selected());
}

#[test]
fn test_radio_with_theme() {
    let radio = Radio::new(WidgetId::default_id(), "Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = radio.render(area);
}

#[test]
fn test_radio_render() {
    let radio = Radio::new(WidgetId::default_id(), "Test");
    let area = Rect::new(0, 0, 20, 1);
    let plane = radio.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_radio_select() {
    let mut radio = Radio::new(WidgetId::default_id(), "Test");
    assert!(!radio.is_selected());
    radio.select();
    assert!(radio.is_selected());
}

#[test]
fn test_label_new() {
    let label = Label::new("Hello");
    assert_eq!(label.text, "Hello");
}

#[test]
fn test_label_with_theme() {
    let label = Label::new("Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = label.render(area);
}

#[test]
fn test_label_render() {
    let label = Label::new("Hello");
    let area = Rect::new(0, 0, 40, 1);
    let plane = label.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_label_set_text() {
    let mut label = Label::new("Hello");
    label.set_text("World");
    assert_eq!(label.text, "World");
}

#[test]
fn test_spinner_new() {
    let spinner = Spinner::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 10, 1);
    let _plane = spinner.render(area);
}

#[test]
fn test_spinner_with_theme() {
    let spinner = Spinner::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 10, 1);
    let _plane = spinner.render(area);
}

#[test]
fn test_spinner_render() {
    let spinner = Spinner::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 10, 1);
    let plane = spinner.render(area);
    assert_eq!(plane.width, 10);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_spinner_tick() {
    let mut spinner = Spinner::new(WidgetId::default_id());
    let initial = spinner.current_frame();
    spinner.tick();
    let next = spinner.current_frame();
    assert!(initial != next || spinner.frames.len() == 1);
}

#[test]
fn test_spinner_frames() {
    let spinner = Spinner::new(WidgetId::default_id()).with_frames(vec!['<', '>']);
    assert_eq!(spinner.frames.len(), 2);
}

#[test]
fn test_progress_bar_new() {
    let pb = ProgressBar::new(WidgetId::default_id());
    assert_eq!(pb.progress(), 0.0);
}

#[test]
fn test_progress_bar_with_theme() {
    let pb = ProgressBar::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = pb.render(area);
}

#[test]
fn test_progress_bar_render() {
    let pb = ProgressBar::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 40, 1);
    let plane = pb.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_progress_bar_set_progress() {
    let mut pb = ProgressBar::new(WidgetId::default_id());
    pb.set_progress(0.5);
    assert_eq!(pb.progress(), 0.5);
}

#[test]
fn test_progress_bar_clamp() {
    let mut pb = ProgressBar::new(WidgetId::default_id());
    pb.set_progress(1.5);
    assert_eq!(pb.progress(), 1.0);
    pb.set_progress(-0.5);
    assert_eq!(pb.progress(), 0.0);
}

#[test]
fn test_slider_new() {
    let slider = Slider::new(WidgetId::default_id());
    assert_eq!(slider.value(), 0.5);
}

#[test]
fn test_slider_with_theme() {
    let slider = Slider::new(WidgetId::default_id()).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = slider.render(area);
}

#[test]
fn test_slider_render() {
    let slider = Slider::new(WidgetId::default_id());
    let area = Rect::new(0, 0, 40, 1);
    let plane = slider.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_slider_range() {
    let slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    assert_eq!(slider.value(), 50.0);
}

#[test]
fn test_slider_set_value() {
    let mut slider = Slider::new(WidgetId::default_id());
    slider.set_value(0.75);
    assert_eq!(slider.value(), 0.75);
}

#[test]
fn test_slider_clamp() {
    let mut slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    slider.set_value(150.0);
    assert_eq!(slider.value(), 100.0);
    slider.set_value(-10.0);
    assert_eq!(slider.value(), 0.0);
}

#[test]
fn test_gauge_new() {
    let gauge = Gauge::new("CPU");
    assert_eq!(gauge.label, "CPU");
    assert_eq!(gauge.value(), 0.0);
}

#[test]
fn test_gauge_with_theme() {
    let gauge = Gauge::new("Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 30, 3);
    let _plane = gauge.render(area);
}

#[test]
fn test_gauge_render() {
    let gauge = Gauge::new("CPU");
    let area = Rect::new(0, 0, 30, 3);
    let plane = gauge.render(area);
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_gauge_set_value() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(50.0);
    assert_eq!(gauge.value(), 50.0);
}

#[test]
fn test_gauge_percentage() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(50.0);
    assert_eq!(gauge.percentage(), 50.0);
}

#[test]
fn test_gauge_fill_color() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(50.0);
    let color = gauge.fill_color();
    assert_eq!(color, Theme::default().success);
}

#[test]
fn test_gauge_warn_threshold() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(75.0);
    let color = gauge.fill_color();
    assert_eq!(color, Theme::default().warning);
}

#[test]
fn test_gauge_crit_threshold() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(95.0);
    let color = gauge.fill_color();
    assert_eq!(color, Theme::default().error);
}

#[test]
fn test_breadcrumbs_new() {
    let crumbs = vec!["home".to_string(), "user".to_string()];
    let bc = Breadcrumbs::new(crumbs.clone());
    let area = Rect::new(0, 0, 80, 1);
    let _plane = bc.render(area);
}

#[test]
fn test_breadcrumbs_render() {
    let crumbs = vec!["home".to_string(), "user".to_string()];
    let bc = Breadcrumbs::new(crumbs);
    let area = Rect::new(0, 0, 80, 1);
    let plane = bc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_status_badge_new() {
    let badge = StatusBadge::new("Active");
    assert_eq!(badge.text(), "Active");
}

#[test]
fn test_status_badge_with_theme() {
    let badge = StatusBadge::new("Test").with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 1);
    let _plane = badge.render(area);
}

#[test]
fn test_status_badge_render() {
    let badge = StatusBadge::new("Active");
    let area = Rect::new(0, 0, 20, 1);
    let plane = badge.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_status_bar_new() {
    let bar = StatusBar::new();
    let area = Rect::new(0, 0, 80, 1);
    let _plane = bar.render(area);
}

#[test]
fn test_status_bar_with_theme() {
    let bar = StatusBar::new().with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 80, 1);
    let _plane = bar.render(area);
}

#[test]
fn test_status_bar_render() {
    let bar = StatusBar::new();
    let area = Rect::new(0, 0, 80, 1);
    let plane = bar.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_status_bar_add_segment() {
    let mut bar = StatusBar::new();
    bar.add_segment(StatusSegment::new("Test segment"));
    let area = Rect::new(0, 0, 80, 1);
    let plane = bar.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_search_input_new() {
    let input = SearchInput::new(WidgetId::default_id(), "Search...");
    let area = Rect::new(0, 0, 40, 1);
    let _plane = input.render(area);
}

#[test]
fn test_search_input_with_theme() {
    let input = SearchInput::new(WidgetId::default_id(), "Search...")
        .with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = input.render(area);
}

#[test]
fn test_search_input_render() {
    let input = SearchInput::new(WidgetId::default_id(), "Search...");
    let area = Rect::new(0, 0, 40, 1);
    let plane = input.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_password_input_new() {
    let input = PasswordInput::new(WidgetId::default_id(), "Password");
    let area = Rect::new(0, 0, 40, 1);
    let _plane = input.render(area);
}

#[test]
fn test_password_input_with_theme() {
    let input = PasswordInput::new(WidgetId::default_id(), "Password")
        .with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 40, 1);
    let _plane = input.render(area);
}

#[test]
fn test_password_input_render() {
    let input = PasswordInput::new(WidgetId::default_id(), "Password");
    let area = Rect::new(0, 0, 40, 1);
    let plane = input.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
}
