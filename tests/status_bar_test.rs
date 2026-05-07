//! StatusBar widget tests — segment rendering, alignment, theme.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{StatusBar, StatusSegment};
use ratatui::layout::Rect;

#[test]
fn test_status_bar_new() {
    let bar = StatusBar::new(WidgetId::new(1));
    let plane = bar.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.height, 1);
    assert_eq!(plane.width, 80);
}

#[test]
fn test_status_bar_with_segments() {
    let bar = StatusBar::new(WidgetId::new(1))
        .add_segment(StatusSegment::new("Mode: Normal"))
        .add_segment(StatusSegment::new("Line: 1/10"))
        .add_segment(StatusSegment::new("UTF-8"));

    let plane = bar.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_status_bar_segment_with_colors() {
    let segment = StatusSegment::new("Error")
        .with_fg(Color::Ansi(1))
        .with_bg(Color::Ansi(0));

    assert_eq!(segment.fg, Color::Ansi(1));
    assert_eq!(segment.bg, Color::Ansi(0));
}

#[test]
fn test_status_bar_theme_change() {
    let mut bar = StatusBar::new(WidgetId::new(1)).add_segment(StatusSegment::new("Test"));

    bar.on_theme_change(&Theme::cyberpunk());
    let plane = bar.render(Rect::new(0, 0, 80, 1));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_status_bar_no_black_background() {
    let bar = StatusBar::new(WidgetId::new(1))
        .add_segment(StatusSegment::new("Ready"))
        .with_theme(Theme::nord());

    let plane = bar.render(Rect::new(0, 0, 80, 1));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_status_bar_multiple_segments() {
    let bar = StatusBar::new(WidgetId::new(1))
        .add_segment(StatusSegment::new("File: main.rs"))
        .add_segment(StatusSegment::new("Rust"))
        .add_segment(StatusSegment::new("Ln 42, Col 10"))
        .add_segment(StatusSegment::new("100%"));

    let plane = bar.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_status_bar_long_segments() {
    let bar = StatusBar::new(WidgetId::new(1))
        .add_segment(StatusSegment::new(
            "This is a very long status message that might overflow",
        ))
        .add_segment(StatusSegment::new("Short"));

    let plane = bar.render(Rect::new(0, 0, 40, 1));
    assert_eq!(plane.width, 40);
}

#[test]
fn test_status_bar_empty() {
    let bar = StatusBar::new(WidgetId::new(1));
    let plane = bar.render(Rect::new(0, 0, 80, 1));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_status_bar_z_index() {
    let bar = StatusBar::new(WidgetId::new(1));
    assert_eq!(bar.z_index(), 50);
}

#[test]
fn test_status_bar_single_segment_full_width() {
    let bar = StatusBar::new(WidgetId::new(1)).add_segment(StatusSegment::new(
        "Full width status bar with single segment",
    ));

    let plane = bar.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.width, 80);
}
