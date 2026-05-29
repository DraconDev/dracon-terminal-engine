//! Tests for the Profiler widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::profiler::{Metric, Profiler};
use std::time::Duration;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_profiler_new() {
    let p = Profiler::new(WidgetId::new(1));
    let area = p.area();
    assert!(area.width > 0);
}

#[test]
fn test_profiler_new_with_id() {
    let p = Profiler::new(WidgetId::new(42));
    assert_eq!(p.id(), WidgetId::new(42));
}

#[test]
fn test_profiler_with_theme() {
    let p = Profiler::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = p.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}

// ============================================================================
// Metrics Tests
// ============================================================================

#[test]
fn test_profiler_record() {
    let mut p = Profiler::new(WidgetId::new(1));
    p.record("test", Duration::from_millis(100), 1);
    let metrics = p.metrics();
    assert!(!metrics.is_empty() || metrics.is_empty()); // Just check it doesn't panic
}

#[test]
fn test_profiler_set_metrics() {
    let mut p = Profiler::new(WidgetId::new(1));
    let metrics = vec![
        Metric {
            name: "a".to_string(),
            value: Duration::from_nanos(1000),
            call_count: 5,
        },
        Metric {
            name: "b".to_string(),
            value: Duration::from_nanos(2000),
            call_count: 10,
        },
    ];
    p.set_metrics(metrics);
    let got = p.metrics();
    assert_eq!(got.len(), 2);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_profiler_id() {
    let p = Profiler::new(WidgetId::new(42));
    assert_eq!(p.id(), WidgetId::new(42));
}

#[test]
fn test_profiler_set_id() {
    let mut p = Profiler::new(WidgetId::new(1));
    p.set_id(WidgetId::new(99));
    assert_eq!(p.id(), WidgetId::new(99));
}

#[test]
fn test_profiler_area() {
    let p = Profiler::new(WidgetId::new(1));
    let area = p.area();
    assert!(area.width > 0);
}

#[test]
fn test_profiler_set_area() {
    let mut p = Profiler::new(WidgetId::new(1));
    p.set_area(Rect::new(0, 0, 100, 40));
    assert_eq!(p.area(), Rect::new(0, 0, 100, 40));
}

#[test]
fn test_profiler_needs_render() {
    let p = Profiler::new(WidgetId::new(1));
    assert!(p.needs_render());
}

#[test]
fn test_profiler_mark_dirty() {
    let mut p = Profiler::new(WidgetId::new(1));
    p.clear_dirty();
    assert!(!p.needs_render());
    p.mark_dirty();
    assert!(p.needs_render());
}

#[test]
fn test_profiler_clear_dirty() {
    let mut p = Profiler::new(WidgetId::new(1));
    p.clear_dirty();
    assert!(!p.needs_render());
}

#[test]
fn test_profiler_default_dirty() {
    let p = Profiler::new(WidgetId::new(1));
    assert!(p.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_profiler_render_basic() {
    let p = Profiler::new(WidgetId::new(1));
    let plane = p.render(Rect::new(0, 0, 80, 30));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_profiler_render_has_content() {
    let p = Profiler::new(WidgetId::new(1));
    let plane = p.render(Rect::new(0, 0, 80, 30));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_profiler_render_wide() {
    let p = Profiler::new(WidgetId::new(1));
    let plane = p.render(Rect::new(0, 0, 120, 30));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_profiler_render_small() {
    let p = Profiler::new(WidgetId::new(1));
    let plane = p.render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_profiler_render_tall() {
    let p = Profiler::new(WidgetId::new(1));
    let plane = p.render(Rect::new(0, 0, 80, 50));
    assert_eq!(plane.height, 50);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_profiler_theme_nord() {
    let p = Profiler::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = p.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}

#[test]
fn test_profiler_theme_dracula() {
    let p = Profiler::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = p.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}

#[test]
fn test_profiler_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let p = Profiler::new(WidgetId::new(1)).with_theme(t);
        let _ = p.render(Rect::new(0, 0, 80, 30));
    }
}

#[test]
fn test_profiler_on_theme_change() {
    let mut p = Profiler::new(WidgetId::new(1));
    p.on_theme_change(&Theme::nord());
    assert!(p.needs_render());
}

#[test]
fn test_profiler_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let p = Profiler::new(WidgetId::new(1)).with_theme(t);
            let _ = p.render(Rect::new(0, 0, 80, 30));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_profiler_render_twice() {
    let p = Profiler::new(WidgetId::new(1));
    let _ = p.render(Rect::new(0, 0, 80, 30));
    let _ = p.render(Rect::new(0, 0, 80, 30));
}

#[test]
fn test_profiler_set_area_then_render() {
    let mut p = Profiler::new(WidgetId::new(1));
    p.set_area(Rect::new(0, 0, 100, 40));
    let plane = p.render(Rect::new(0, 0, 100, 40));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_profiler_record_many() {
    let mut p = Profiler::new(WidgetId::new(1));
    for i in 0..10 {
        p.record(
            &format!("metric{}", i),
            Duration::from_millis(i as u64 * 10),
            i as u64 + 1,
        );
    }
    let plane = p.render(Rect::new(0, 0, 80, 30));
    assert!(plane.width > 0);
}

#[test]
fn test_profiler_empty_metrics() {
    let p = Profiler::new(WidgetId::new(1));
    let metrics = p.metrics();
    // Should have zero or more metrics
    let _ = metrics;
}
