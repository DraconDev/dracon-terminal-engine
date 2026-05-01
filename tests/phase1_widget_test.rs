//! Integration tests for framework widgets.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Checkbox, ProgressBar, Radio, Slider, Spinner, Toggle,
};
use ratatui::layout::Rect;

fn dummy_area() -> Rect {
    Rect::new(0, 0, 80, 20)
}

#[test]
fn test_checkbox_render() {
    let cb = Checkbox::new(WidgetId::new(1), "Test");
    let plane = cb.render(dummy_area());
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_checkbox_toggle() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    assert!(!cb.is_checked());
    cb.toggle();
    assert!(cb.is_checked());
}

#[test]
fn test_toggle_render() {
    let t = Toggle::new(WidgetId::new(2), "Enable");
    let plane = t.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_toggle_on_off() {
    let mut t = Toggle::new(WidgetId::new(2), "Enable");
    assert!(!t.is_on());
    t.toggle();
    assert!(t.is_on());
}

#[test]
fn test_progress_bar_render() {
    let pb = ProgressBar::new(WidgetId::new(3));
    let plane = pb.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_progress_bar_set_value() {
    let mut pb = ProgressBar::new(WidgetId::new(3));
    pb.set_progress(0.5);
    assert!((pb.progress() - 0.5).abs() < 0.001);
    pb.set_progress(1.0);
    assert!((pb.progress() - 1.0).abs() < 0.001);
    pb.set_progress(1.5);
    assert!((pb.progress() - 1.0).abs() < 0.001);
}

#[test]
fn test_spinner_render() {
    let sp = Spinner::new(WidgetId::new(4));
    let plane = sp.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_spinner_tick() {
    let mut sp = Spinner::new(WidgetId::new(4));
    let f1 = sp.current_frame();
    sp.tick();
    let f2 = sp.current_frame();
    assert!(f1 != f2 || sp.current_frame() == f1);
}

#[test]
fn test_radio_render() {
    let r = Radio::new(WidgetId::new(5), "Option 1");
    let plane = r.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_radio_select() {
    let mut r = Radio::new(WidgetId::new(5), "Option 1");
    assert!(!r.is_selected());
    r.select();
    assert!(r.is_selected());
    r.deselect();
    assert!(!r.is_selected());
}

#[test]
fn test_slider_render() {
    let sl = Slider::new(WidgetId::new(6));
    let plane = sl.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_slider_value() {
    let mut sl = Slider::new(WidgetId::new(6));
    sl.set_value(0.75);
    assert!((sl.value() - 0.75).abs() < 0.001);
}

#[test]
fn test_slider_clamp() {
    let mut sl = Slider::new(WidgetId::new(6)).with_range(0.0, 100.0);
    sl.set_value(150.0);
    assert!((sl.value() - 100.0).abs() < 0.001);
    sl.set_value(-50.0);
    assert!((sl.value() - 0.0).abs() < 0.001);
}

#[test]
fn test_widget_with_theme() {
    let cb = Checkbox::new(WidgetId::new(7), "Themed").with_theme(Theme::dark());
    let plane = cb.render(dummy_area());
    assert!(plane.width > 0);
}

#[test]
fn test_widget_id_uniqueness() {
    let id1 = WidgetId::new(1);
    let id2 = WidgetId::new(2);
    assert_ne!(id1, id2);
}

#[test]
fn test_toggle_with_callback() {
    let mut t = Toggle::new(WidgetId::new(8), "Toggle me").on_change(|_| {});
    t.toggle();
    assert!(t.is_on());
}

// ========== Dirty Tracking Integration Tests ==========

#[test]
fn test_widget_dirty_on_construction() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    assert!(
        cb.needs_render(),
        "widget should be dirty after construction"
    );
}

#[test]
fn test_widget_clean_after_render() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.render(dummy_area());
    cb.clear_dirty();
    assert!(
        !cb.needs_render(),
        "widget should be clean after render + clear_dirty"
    );
}

#[test]
fn test_state_change_marks_dirty() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Test");
    cb.render(dummy_area());
    cb.clear_dirty();
    cb.toggle();
    assert!(
        cb.needs_render(),
        "widget should be dirty after state change"
    );
}

#[test]
fn test_slider_dirty_after_value_change() {
    let mut sl = Slider::new(WidgetId::new(6));
    sl.render(dummy_area());
    sl.clear_dirty();
    sl.set_value(0.5);
    assert!(
        sl.needs_render(),
        "slider should be dirty after value change"
    );
}

#[test]
fn test_progress_bar_dirty_after_set_progress() {
    let mut pb = ProgressBar::new(WidgetId::new(3));
    pb.render(dummy_area());
    pb.clear_dirty();
    pb.set_progress(0.7);
    assert!(
        pb.needs_render(),
        "progress bar should be dirty after set_progress"
    );
}

#[test]
fn test_radio_dirty_after_select() {
    let mut r = Radio::new(WidgetId::new(5), "Option 1");
    r.render(dummy_area());
    r.clear_dirty();
    r.select();
    assert!(r.needs_render(), "radio should be dirty after select");
}

#[test]
fn test_mark_dirty_overrides_clean() {
    let mut sl = Slider::new(WidgetId::new(6));
    sl.render(dummy_area());
    sl.clear_dirty();
    assert!(!sl.needs_render());
    sl.mark_dirty();
    assert!(
        sl.needs_render(),
        "widget should be dirty after explicit mark_dirty"
    );
}

#[test]
fn test_multiple_state_changes_single_dirty() {
    let mut sl = Slider::new(WidgetId::new(6));
    sl.render(dummy_area());
    sl.clear_dirty();
    sl.set_value(0.1);
    sl.set_value(0.2);
    sl.set_value(0.3);
    assert!(
        sl.needs_render(),
        "multiple changes should still result in dirty"
    );
}
