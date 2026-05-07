//! Profiler widget tests — metric collection, thresholds.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Metric, Profiler};
use ratatui::layout::Rect;
use std::time::Duration;

#[test]
fn test_profiler_new() {
    let profiler = Profiler::new(WidgetId::new(1));
    let plane = profiler.render(Rect::new(0, 0, 60, 15));
    assert_eq!(plane.width, 60);
    assert_eq!(plane.height, 15);
}

#[test]
fn test_profiler_record_metric() {
    let mut profiler = Profiler::new(WidgetId::new(1));
    profiler.record("render", Duration::from_millis(16), 1);
    assert_eq!(profiler.metrics().len(), 1);
}

#[test]
fn test_profiler_record_multiple() {
    let mut profiler = Profiler::new(WidgetId::new(1));
    profiler.record("render", Duration::from_millis(16), 1);
    profiler.record("input", Duration::from_micros(100), 1);
    profiler.record("update", Duration::from_millis(5), 1);
    assert_eq!(profiler.metrics().len(), 3);
}

#[test]
fn test_profiler_set_metrics() {
    let metrics = vec![
        Metric {
            name: "A".to_string(),
            value: Duration::from_millis(10),
            call_count: 1,
        },
        Metric {
            name: "B".to_string(),
            value: Duration::from_millis(20),
            call_count: 2,
        },
    ];
    let mut profiler = Profiler::new(WidgetId::new(1));
    profiler.set_metrics(metrics);
    assert_eq!(profiler.metrics().len(), 2);
}

#[test]
fn test_profiler_render_with_metrics() {
    let mut profiler = Profiler::new(WidgetId::new(1)).with_theme(Theme::nord());
    profiler.record("render", Duration::from_millis(16), 1);
    let plane = profiler.render(Rect::new(0, 0, 60, 15));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_profiler_no_black_background() {
    let profiler = Profiler::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = profiler.render(Rect::new(0, 0, 60, 15));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_profiler_theme_change() {
    let mut profiler = Profiler::new(WidgetId::new(1));
    profiler.on_theme_change(&Theme::cyberpunk());
    let plane = profiler.render(Rect::new(0, 0, 60, 15));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_profiler_empty_metrics() {
    let profiler = Profiler::new(WidgetId::new(1));
    let plane = profiler.render(Rect::new(0, 0, 60, 15));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_profiler_metric_values() {
    let mut profiler = Profiler::new(WidgetId::new(1));
    profiler.record("test", Duration::from_millis(100), 5);

    let metric = &profiler.metrics()[0];
    assert_eq!(metric.name, "test");
    assert_eq!(metric.value, Duration::from_millis(100));
    assert_eq!(metric.call_count, 5);
}

#[test]
fn test_profiler_large_number_of_metrics() {
    let mut profiler = Profiler::new(WidgetId::new(1));
    for i in 0..50 {
        profiler.record(
            &format!("metric{}", i),
            Duration::from_micros(i as u64 * 100),
            i as u64,
        );
    }
    let plane = profiler.render(Rect::new(0, 0, 60, 15));
    assert!(plane.cells.len() > 0);
}
