//! SplitPane tests — drag-resize, ratio bounds, orientation.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Orientation, SplitPane};
use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
use ratatui::layout::Rect;

#[test]
fn test_splitpane_new_horizontal() {
    let pane = SplitPane::new(Orientation::Horizontal);
    assert_eq!(pane.get_ratio(), 0.5);
}

#[test]
fn test_splitpane_new_vertical() {
    let pane = SplitPane::new(Orientation::Vertical);
    assert_eq!(pane.get_ratio(), 0.5);
}

#[test]
fn test_splitpane_ratio_clamping() {
    let pane = SplitPane::new(Orientation::Horizontal).ratio(0.05);
    assert_eq!(pane.get_ratio(), 0.1); // Clamped to min

    let pane = SplitPane::new(Orientation::Horizontal).ratio(0.95);
    assert_eq!(pane.get_ratio(), 0.9); // Clamped to max
}

#[test]
fn test_splitpane_horizontal_split() {
    let pane = SplitPane::new(Orientation::Horizontal).ratio(0.5);
    let (left, right) = pane.split(Rect::new(0, 0, 80, 24));

    assert_eq!(left.x, 0);
    assert_eq!(left.y, 0);
    assert_eq!(left.height, 24);
    assert_eq!(right.x, left.width);
    assert_eq!(right.y, 0);
    assert_eq!(right.height, 24);
    assert_eq!(left.width + right.width, 80);
}

#[test]
fn test_splitpane_vertical_split() {
    let pane = SplitPane::new(Orientation::Vertical).ratio(0.5);
    let (top, bottom) = pane.split(Rect::new(0, 0, 80, 24));

    assert_eq!(top.x, 0);
    assert_eq!(top.y, 0);
    assert_eq!(top.width, 80);
    assert_eq!(bottom.x, 0);
    assert_eq!(bottom.y, top.height);
    assert_eq!(bottom.width, 80);
    assert_eq!(top.height + bottom.height, 24);
}

#[test]
fn test_splitpane_min_size_respected() {
    let pane = SplitPane::new(Orientation::Horizontal)
        .ratio(0.5)
        .with_min_size(20);
    let (left, right) = pane.split(Rect::new(0, 0, 80, 24));

    assert!(left.width >= 20);
    assert!(right.width >= 20);
}

#[test]
fn test_splitpane_from_rect_wide() {
    let pane = SplitPane::from_rect(Rect::new(0, 0, 80, 24));
    assert_eq!(pane.get_ratio(), 0.5);
}

#[test]
fn test_splitpane_from_rect_tall() {
    let pane = SplitPane::from_rect(Rect::new(0, 0, 40, 60));
    assert_eq!(pane.get_ratio(), 0.5);
}

#[test]
fn test_splitpane_render() {
    let pane = SplitPane::new(Orientation::Horizontal);
    let plane = pane.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_splitpane_theme_change() {
    let mut pane = SplitPane::new(Orientation::Horizontal);
    let theme = Theme::cyberpunk();
    pane.on_theme_change(&theme);

    let plane = pane.render(Rect::new(0, 0, 80, 24));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_splitpane_handle_resize_drag() {
    let mut pane = SplitPane::new(Orientation::Horizontal).ratio(0.5);
    pane.set_area(Rect::new(0, 0, 80, 24));

    // Drag on the divider (around x=40)
    let result = pane.handle_resize(
        MouseEventKind::Down(MouseButton::Left),
        40,
        10,
        Rect::new(0, 0, 80, 24),
    );
    assert!(result);
}

#[test]
fn test_splitpane_handle_resize_drag_horizontal() {
    let mut pane = SplitPane::new(Orientation::Vertical).ratio(0.5);
    pane.set_area(Rect::new(0, 0, 80, 24));

    // Drag on the divider (around y=12)
    let result = pane.handle_resize(
        MouseEventKind::Down(MouseButton::Left),
        40,
        12,
        Rect::new(0, 0, 80, 24),
    );
    assert!(result);
}

#[test]
fn test_splitpane_handle_resize_outside_divider() {
    let mut pane = SplitPane::new(Orientation::Horizontal).ratio(0.5);
    pane.set_area(Rect::new(0, 0, 80, 24));

    // Click far from divider
    let result = pane.handle_resize(
        MouseEventKind::Down(MouseButton::Left),
        5,
        10,
        Rect::new(0, 0, 80, 24),
    );
    assert!(!result);
}

#[test]
fn test_splitpane_drag_changes_ratio() {
    let mut pane = SplitPane::new(Orientation::Horizontal).ratio(0.5);
    pane.set_area(Rect::new(0, 0, 80, 24));

    // Start drag at divider
    pane.handle_resize(
        MouseEventKind::Down(MouseButton::Left),
        40,
        10,
        Rect::new(0, 0, 80, 24),
    );

    // Drag to new position
    pane.handle_resize(MouseEventKind::Moved, 60, 10, Rect::new(0, 0, 80, 24));

    // Ratio should have changed
    let ratio = pane.get_ratio();
    assert!(ratio > 0.5);
}

#[test]
fn test_splitpane_drag_release() {
    let mut pane = SplitPane::new(Orientation::Horizontal).ratio(0.5);
    pane.set_area(Rect::new(0, 0, 80, 24));

    pane.handle_resize(
        MouseEventKind::Down(MouseButton::Left),
        40,
        10,
        Rect::new(0, 0, 80, 24),
    );

    let result = pane.handle_resize(
        MouseEventKind::Up(MouseButton::Left),
        60,
        10,
        Rect::new(0, 0, 80, 24),
    );
    assert!(result);
}

#[test]
fn test_splitpane_with_divider_char() {
    let pane = SplitPane::new(Orientation::Horizontal).with_divider('┃');
    let plane = pane.render(Rect::new(0, 0, 80, 24));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_splitpane_no_black_background() {
    let pane = SplitPane::new(Orientation::Horizontal);
    let plane = pane.render(Rect::new(0, 0, 80, 24));

    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}
