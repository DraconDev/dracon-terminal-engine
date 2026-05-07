//! Multi-monitor / terminal resize tests.

use dracon_terminal_engine::compositor::{Compositor, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, Label, List};
use ratatui::layout::Rect;

#[test]
fn test_compositor_resize_smaller() {
    let mut compositor = Compositor::new(80, 24);
    let plane = Plane::new(0, 80, 24);
    compositor.add_plane(plane);

    compositor.resize(40, 12);
    let (w, h) = compositor.size();
    assert_eq!(w, 40);
    assert_eq!(h, 12);
}

#[test]
fn test_compositor_resize_larger() {
    let mut compositor = Compositor::new(40, 12);
    compositor.resize(120, 40);
    let (w, h) = compositor.size();
    assert_eq!(w, 120);
    assert_eq!(h, 40);
}

#[test]
fn test_compositor_resize_preserves_planes() {
    let mut compositor = Compositor::new(80, 24);
    let plane = Plane::new(0, 10, 5);
    compositor.add_plane(plane);

    compositor.resize(40, 12);
    assert_eq!(compositor.planes.len(), 1);
}

#[test]
fn test_button_render_after_resize() {
    let mut btn = Button::with_id(WidgetId::new(1), "Click");
    btn.on_theme_change(&Theme::nord());

    // Render at original size
    let _ = btn.render(Rect::new(0, 0, 20, 3));

    // Resize and render again
    let plane = btn.render(Rect::new(0, 0, 10, 2));
    assert_eq!(plane.width, 10);
    assert_eq!(plane.height, 2);
}

#[test]
fn test_list_render_after_resize() {
    let items = vec![
        "Item 1".to_string(),
        "Item 2".to_string(),
        "Item 3".to_string(),
    ];
    let mut list = List::new_with_id(WidgetId::new(1), items);
    list.on_theme_change(&Theme::nord());

    let _ = list.render(Rect::new(0, 0, 40, 10));
    let plane = list.render(Rect::new(0, 0, 20, 5));
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 5);
}

#[test]
fn test_label_render_at_various_sizes() {
    let mut label = Label::new("Hello World");
    label.on_theme_change(&Theme::nord());

    let plane1 = label.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane1.width, 20);

    let plane2 = label.render(Rect::new(0, 0, 5, 1));
    assert_eq!(plane2.width, 5);
}

#[test]
fn test_compositor_hit_test_after_resize() {
    let mut compositor = Compositor::new(80, 24);
    let mut plane = Plane::new(0, 10, 10);
    plane.x = 70;
    plane.y = 20;
    compositor.add_plane(plane);

    // Resize to smaller - plane is now outside bounds
    compositor.resize(60, 15);

    let hit = compositor.hit_test(75, 22);
    assert!(hit.is_none()); // Outside new bounds
}

#[test]
fn test_splitpane_after_resize() {
    let pane = SplitPane::new(Orientation::Horizontal).ratio(0.5);

    let (left1, right1) = pane.split(Rect::new(0, 0, 80, 24));
    assert_eq!(left1.width + right1.width, 80);

    let (left2, right2) = pane.split(Rect::new(0, 0, 40, 12));
    assert_eq!(left2.width + right2.width, 40);
}

#[test]
fn test_widget_area_update() {
    let mut btn = Button::with_id(WidgetId::new(1), "Test");
    btn.set_area(Rect::new(0, 0, 20, 3));
    assert_eq!(btn.area().width, 20);

    btn.set_area(Rect::new(0, 0, 10, 2));
    assert_eq!(btn.area().width, 10);
}

#[test]
fn test_compositor_extreme_resize() {
    let mut compositor = Compositor::new(80, 24);
    compositor.resize(1, 1);
    let (w, h) = compositor.size();
    assert_eq!(w, 1);
    assert_eq!(h, 1);
}
