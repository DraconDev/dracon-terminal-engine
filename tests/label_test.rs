//! Tests for framework Label widget.

mod common;
use common::make_area;

use dracon_terminal_engine::compositor::Styles;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::label::Label;

#[test]
fn test_label_new() {
    let label = Label::new("Hello");
    let plane = label.render(make_area(40, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_label_with_id() {
    let id = WidgetId::new(5);
    let label = Label::with_id(id, "Text");
    assert_eq!(label.id(), id);
}

#[test]
fn test_label_with_theme() {
    let label = Label::new("test").with_theme(Theme::cyberpunk());
    let plane = label.render(make_area(40, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_label_with_style() {
    let label = Label::new("test").with_style(Styles::BOLD);
    let area = make_area(40, 1);
    let plane = label.render(area);
    assert!(plane.cells[0].style.contains(Styles::BOLD));
}

#[test]
fn test_label_default_area() {
    let label = Label::new("test");
    let area = label.area();
    assert_eq!(area.width, 40);
    assert_eq!(area.height, 1);
}

#[test]
fn test_label_render_width_matches_area() {
    let label = Label::new("test");
    let area = make_area(20, 3);
    let plane = label.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_label_render_text_chars() {
    let label = Label::new("ABC");
    let area = make_area(20, 3);
    let plane = label.render(area);
    assert_eq!(plane.cells[0].char, 'A');
    assert_eq!(plane.cells[1].char, 'B');
    assert_eq!(plane.cells[2].char, 'C');
}

#[test]
fn test_label_render_truncates_long_text() {
    let label = Label::new("This is a very long text that should be truncated");
    let area = make_area(10, 1);
    let plane = label.render(area);
    assert_eq!(plane.cells[0].char, 'T');
}

#[test]
fn test_label_render_theme_colors() {
    let label = Label::new("C").with_theme(Theme::cyberpunk());
    let area = make_area(10, 1);
    let plane = label.render(area);
    assert_eq!(plane.cells[0].fg, Theme::cyberpunk().fg);
    assert_eq!(plane.cells[0].bg, Theme::cyberpunk().bg);
}

#[test]
fn test_label_set_text() {
    let mut label = Label::new("old");
    label.set_text("new");
    let plane = label.render(make_area(40, 1));
    assert_eq!(plane.cells[0].char, 'n');
}

#[test]
fn test_label_set_text_empty() {
    let mut label = Label::new("old");
    label.set_text("");
    let plane = label.render(make_area(40, 1));
    assert_eq!(plane.cells[0].char, ' ');
}

#[test]
fn test_label_clear_dirty() {
    let mut label = Label::new("test");
    assert!(label.needs_render());
    label.clear_dirty();
    assert!(!label.needs_render());
}

#[test]
fn test_label_mark_dirty() {
    let mut label = Label::new("test");
    label.clear_dirty();
    assert!(!label.needs_render());
    label.mark_dirty();
    assert!(label.needs_render());
}

#[test]
fn test_label_set_area_marks_dirty() {
    let mut label = Label::new("test");
    label.clear_dirty();
    assert!(!label.needs_render());
    label.set_area(make_area(5, 1));
    assert!(label.needs_render());
}

#[test]
fn test_label_focusable_returns_false() {
    let label = Label::new("test");
    assert!(!label.focusable());
}

#[test]
fn test_label_needs_render() {
    let mut label = Label::new("test");
    assert!(label.needs_render());
    label.clear_dirty();
    assert!(!label.needs_render());
}

#[test]
fn test_label_with_area() {
    let label = Label::new("test").with_area(make_area(15, 2));
    let area = label.area();
    assert_eq!(area.width, 15);
    assert_eq!(area.height, 2);
}

#[test]
fn test_label_z_index() {
    let label = Label::new("test");
    assert_eq!(label.z_index(), 0);
}

#[test]
fn test_label_set_id() {
    let mut label = Label::new("test");
    let id = WidgetId::new(99);
    label.set_id(id);
    assert_eq!(label.id(), id);
}

#[test]
fn test_label_cursor_position_returns_none() {
    let label = Label::new("test");
    assert!(label.cursor_position().is_none());
}
