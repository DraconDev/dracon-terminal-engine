//! Tests for Standalone Panel widget.

mod common;
use common::make_area;

use dracon_terminal_engine::widgets::panel::Panel;

#[test]
fn test_panel_new() {
    let panel = Panel::new("Title");
    let area = make_area(20, 10);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    ratatui::widgets::Widget::render(panel, area, &mut buf);
}

#[test]
fn test_panel_border_color() {
    let panel = Panel::new("Test").border_color(ratatui::style::Color::Red);
    let area = make_area(20, 10);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    ratatui::widgets::Widget::render(panel, area, &mut buf);
}

#[test]
fn test_panel_inner_excludes_borders() {
    let panel = Panel::new("Title");
    let area = make_area(20, 10);
    let inner = panel.inner(area);
    assert!(inner.width < area.width);
    assert!(inner.height < area.height);
}

#[test]
fn test_panel_inner_small_area() {
    let panel = Panel::new("Title");
    let area = make_area(3, 3);
    let inner = panel.inner(area);
    assert_eq!(inner.width, 1);
}

#[test]
fn test_panel_with_long_title() {
    let panel = Panel::new("A Very Long Panel Title That Might Be Truncated");
    let area = make_area(20, 10);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    ratatui::widgets::Widget::render(panel, area, &mut buf);
}

#[test]
fn test_panel_with_empty_title() {
    let panel = Panel::new("");
    let area = make_area(20, 10);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    ratatui::widgets::Widget::render(panel, area, &mut buf);
}

#[test]
fn test_panel_default_border_is_cyan() {
    let panel = Panel::new("Test");
    let area = make_area(20, 10);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    ratatui::widgets::Widget::render(panel, area, &mut buf);
}