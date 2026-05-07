//! Gauge widget tests — value thresholds, color changes, label formatting.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::Gauge;
use ratatui::layout::Rect;

#[test]
fn test_gauge_new() {
    let gauge = Gauge::new("CPU");
    assert_eq!(gauge.value(), 0.0);
    assert_eq!(gauge.percentage(), 0.0);
}

#[test]
fn test_gauge_with_id() {
    let gauge = Gauge::with_id(WidgetId::new(1), "Memory");
    let plane = gauge.render(Rect::new(0, 0, 30, 3));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_gauge_set_value() {
    let mut gauge = Gauge::new("CPU");
    gauge.set_value(50.0);
    assert_eq!(gauge.value(), 50.0);
    assert_eq!(gauge.percentage(), 50.0);
}

#[test]
fn test_gauge_value_clamped_to_max() {
    let mut gauge = Gauge::new("CPU").max(100.0);
    gauge.set_value(150.0);
    assert_eq!(gauge.value(), 100.0);
}

#[test]
fn test_gauge_percentage_zero_max() {
    let gauge = Gauge::new("CPU").max(0.0);
    assert_eq!(gauge.percentage(), 0.0);
}

#[test]
fn test_gauge_fill_color_normal() {
    let gauge = Gauge::new("CPU")
        .max(100.0)
        .warn_threshold(70.0)
        .crit_threshold(90.0)
        .with_theme(Theme::nord());

    // Below warning threshold
    let color = gauge.fill_color();
    assert!(color != Color::Reset);
}

#[test]
fn test_gauge_fill_color_warning() {
    let mut gauge = Gauge::new("CPU")
        .max(100.0)
        .warn_threshold(70.0)
        .crit_threshold(90.0)
        .with_theme(Theme::nord());

    gauge.set_value(75.0);
    let color = gauge.fill_color();
    assert!(color != Color::Reset);
}

#[test]
fn test_gauge_fill_color_critical() {
    let mut gauge = Gauge::new("CPU")
        .max(100.0)
        .warn_threshold(70.0)
        .crit_threshold(90.0)
        .with_theme(Theme::nord());

    gauge.set_value(95.0);
    let color = gauge.fill_color();
    assert!(color != Color::Reset);
}

#[test]
fn test_gauge_render() {
    let gauge = Gauge::new("CPU").with_theme(Theme::nord());
    let plane = gauge.render(Rect::new(0, 0, 30, 3));
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_gauge_render_with_value() {
    let mut gauge = Gauge::new("Memory").max(100.0).with_theme(Theme::nord());
    gauge.set_value(42.0);
    let plane = gauge.render(Rect::new(0, 0, 30, 3));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_gauge_theme_change() {
    let mut gauge = Gauge::new("Disk");
    gauge.on_theme_change(&Theme::cyberpunk());
    let plane = gauge.render(Rect::new(0, 0, 30, 3));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_gauge_no_black_background() {
    let gauge = Gauge::new("CPU").with_theme(Theme::nord());
    let plane = gauge.render(Rect::new(0, 0, 30, 3));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_gauge_custom_thresholds() {
    let mut gauge = Gauge::new("Network")
        .max(1000.0)
        .warn_threshold(500.0)
        .crit_threshold(800.0);

    gauge.set_value(600.0);
    assert_eq!(gauge.percentage(), 60.0);
}

#[test]
fn test_gauge_zero_value() {
    let mut gauge = Gauge::new("CPU").max(100.0);
    gauge.set_value(0.0);
    assert_eq!(gauge.percentage(), 0.0);
}

#[test]
fn test_gauge_full_value() {
    let mut gauge = Gauge::new("CPU").max(100.0);
    gauge.set_value(100.0);
    assert_eq!(gauge.percentage(), 100.0);
}
