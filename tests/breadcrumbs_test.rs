//! Breadcrumbs tests — path navigation, segment clicks.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::Breadcrumbs;
use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::path::Path;

#[test]
fn test_breadcrumbs_new() {
    let crumbs = Breadcrumbs::new(vec!["home".to_string(), "user".to_string(), "projects".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.height, 1);
}

#[test]
fn test_breadcrumbs_from_path() {
    let crumbs = Breadcrumbs::from_path(Path::new("/home/user/projects"));
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.height, 1);
}

#[test]
fn test_breadcrumbs_empty() {
    let crumbs = Breadcrumbs::new(vec![]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.height, 1);
}

#[test]
fn test_breadcrumbs_single_segment() {
    let crumbs = Breadcrumbs::new(vec!["root".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_breadcrumbs_theme_change() {
    let mut crumbs = Breadcrumbs::new(vec!["a".to_string(), "b".to_string()]);
    crumbs.on_theme_change(&Theme::cyberpunk());
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_breadcrumbs_no_black_background() {
    let crumbs = Breadcrumbs::new(vec!["home".to_string(), "user".to_string()]).with_theme(Theme::nord());
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_breadcrumbs_mouse_click_navigates() {
    let clicked = std::cell::Cell::new(None);
    let clicked_ref = &clicked;
    
    let mut crumbs = Breadcrumbs::new(vec![
        "home".to_string(),
        "user".to_string(),
        "projects".to_string(),
    ]).on_navigate(move |idx| {
        clicked_ref.set(Some(idx));
    });
    crumbs.set_area(Rect::new(0, 0, 80, 1));
    
    // Click on first segment
    let result = crumbs.handle_mouse(MouseEventKind::Down(MouseButton::Left), 2, 0);
    assert!(result);
}

#[test]
fn test_breadcrumbs_mouse_click_outside() {
    let mut crumbs = Breadcrumbs::new(vec!["home".to_string()]);
    crumbs.set_area(Rect::new(0, 0, 80, 1));
    
    // Click outside segment area
    let result = crumbs.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 0);
    assert!(!result);
}

#[test]
fn test_breadcrumbs_long_path_truncated() {
    let segments: Vec<String> = (0..20).map(|i| format!("segment{}", i)).collect();
    let crumbs = Breadcrumbs::new(segments);
    let plane = crumbs.render(Rect::new(0, 0, 40, 1));
    assert_eq!(plane.width, 40);
}

#[test]
fn test_breadcrumbs_from_path_empty() {
    let crumbs = Breadcrumbs::from_path(Path::new("/"));
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_breadcrumbs_z_index() {
    let crumbs = Breadcrumbs::new(vec!["test".to_string()]);
    assert_eq!(crumbs.z_index(), 10);
}
