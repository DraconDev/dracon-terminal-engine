//! Tests for the list_common shared utilities.

use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widgets::list_helpers::{
    render_scroll_indicator, ListNavigation,
};

#[test]
fn test_list_navigation_new() {
    let nav = ListNavigation::<String>::new();
    assert_eq!(nav.selected, 0);
    assert_eq!(nav.offset, 0);
    assert_eq!(nav.visible_count, 10);
    assert!(nav.hovered.is_none());
    assert!(!nav.allow_multi_select);
    assert!(nav.selected_indices.is_empty());
    assert!(nav.last_selected.is_none());
    assert!(!nav.enable_undo);
    assert!(nav.undo_stack.is_empty());
    assert!(nav.redo_stack.is_empty());
}

#[test]
fn test_list_navigation_move_down() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    // With 5 items
    let moved = nav.move_down(5);
    assert!(moved);
    assert_eq!(nav.selected, 1);

    // Can move multiple times
    nav.move_down(5);
    nav.move_down(5);
    assert_eq!(nav.selected, 3);

    // Can't go past end
    let moved = nav.move_down(5);
    assert!(moved);
    assert_eq!(nav.selected, 4);

    let moved = nav.move_down(5);
    assert!(!moved); // Can't go past last item
    assert_eq!(nav.selected, 4);
}

#[test]
fn test_list_navigation_move_up() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.selected = 3;

    let moved = nav.move_up();
    assert!(moved);
    assert_eq!(nav.selected, 2);

    // Can't go past first
    nav.selected = 0;
    let moved = nav.move_up();
    assert!(!moved);
    assert_eq!(nav.selected, 0);
}

#[test]
fn test_list_navigation_move_home() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.selected = 10;
    nav.offset = 5;

    let moved = nav.move_home();
    assert!(moved);
    assert_eq!(nav.selected, 0);
    assert_eq!(nav.offset, 0);
}

#[test]
fn test_list_navigation_move_end() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    let moved = nav.move_end(20);
    assert!(moved);
    assert_eq!(nav.selected, 19);
}

#[test]
fn test_list_navigation_page_down() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.visible_count = 5;

    let moved = nav.page_down(20);
    assert!(moved);
    assert_eq!(nav.selected, 5);
}

#[test]
fn test_list_navigation_page_up() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.selected = 10;

    let moved = nav.page_up();
    assert!(moved);
    // page_up subtracts visible_count (10) from selected
    assert_eq!(nav.selected, 0);
}

#[test]
fn test_list_navigation_scroll_offset_adjustment() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.offset = 0;
    nav.visible_count = 5;

    // Move past visible range
    nav.selected = 7;
    nav.clamp_scroll();
    assert!(nav.offset > 0);
}

#[test]
fn test_list_navigation_scroll_down() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.offset = 0;

    nav.scroll_down(20);
    assert_eq!(nav.offset, 1);

    nav.scroll_down(20);
    assert_eq!(nav.offset, 2);
}

#[test]
fn test_list_navigation_scroll_down_at_end() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.offset = 15;

    nav.scroll_down(20);
    // scroll_down clamps to (item_count - visible_count), which is 20 - 10 = 10
    assert_eq!(nav.offset, 10);
}

#[test]
fn test_list_navigation_scroll_up() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.offset = 5;

    nav.scroll_up();
    assert_eq!(nav.offset, 4);

    nav.scroll_up();
    assert_eq!(nav.offset, 3);
}

#[test]
fn test_list_navigation_scroll_up_at_start() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.offset = 0;

    nav.scroll_up();
    assert_eq!(nav.offset, 0); // Can't scroll before start
}

#[test]
fn test_list_navigation_multi_select() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.allow_multi_select = true;

    nav.select_all(10);
    assert_eq!(nav.selected_indices.len(), 10);
    assert_eq!(nav.selected, 0);

    let cleared = nav.clear_selection();
    assert!(cleared);
    assert!(nav.selected_indices.is_empty());

    // Already cleared
    let cleared = nav.clear_selection();
    assert!(!cleared);
}

#[test]
fn test_list_navigation_undo_stack_disabled() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.enable_undo = false;

    nav.push_undo("snapshot".to_string());

    let result = nav.undo("current".to_string());
    assert!(result.is_none());
}

#[test]
fn test_list_navigation_undo_stack_enabled() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.enable_undo = true;

    nav.push_undo("state1".to_string());
    nav.push_undo("state2".to_string());

    let result = nav.undo("current".to_string());
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "state2".to_string());

    // Redo
    // redo returns the popped redo item (which was "current" that was pushed during undo)
    let result = nav.redo("snapshot".to_string());
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "current".to_string()); // Returns the popped redo item
}

#[test]
fn test_list_navigation_undo_empty_stack() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.enable_undo = true;

    let result = nav.undo("current".to_string());
    assert!(result.is_none());
}

#[test]
fn test_list_navigation_redo_empty_stack() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.enable_undo = true;

    let result = nav.redo("current".to_string());
    assert!(result.is_none());
}

#[test]
fn test_list_navigation_undo_stack_max_size() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.enable_undo = true;

    // Push more than MAX_UNDO_STACK (50)
    for i in 0..60 {
        nav.push_undo(format!("state{}", i));
    }

    // Oldest should be dropped
    assert!(nav.undo_stack.len() <= 50);
}

#[test]
fn test_list_navigation_redo_clears_on_push() {
    let mut nav: ListNavigation<String> = ListNavigation::new();
    nav.enable_undo = true;

    nav.push_undo("state1".to_string());
    nav.undo("current".to_string()).unwrap(); // Pop state1, state1 in redo

    // Push new state - clears redo stack
    nav.push_undo("state2".to_string());

    let result = nav.redo("current".to_string());
    assert!(result.is_none()); // Redo stack was cleared
}

#[test]
fn test_list_navigation_default() {
    let nav: ListNavigation<String> = ListNavigation::default();
    assert_eq!(nav.selected, 0);
}

#[test]
fn test_render_scroll_indicator_needed() {
    let mut plane = Plane::new(0, 20, 5);
    plane.fill_bg(Theme::nord().bg);
    let theme = Theme::nord();

    // total > visible, should render
    render_scroll_indicator(
        &mut plane,
        ratatui::layout::Rect::new(0, 0, 20, 5),
        5,
        20,
        10,
        &theme,
    );
}

#[test]
fn test_render_scroll_indicator_at_start() {
    let mut plane = Plane::new(0, 20, 5);
    plane.fill_bg(Theme::nord().bg);
    let theme = Theme::nord();

    render_scroll_indicator(
        &mut plane,
        ratatui::layout::Rect::new(0, 0, 20, 5),
        0,
        20,
        10,
        &theme,
    );
}

#[test]
fn test_render_scroll_indicator_at_end() {
    let mut plane = Plane::new(0, 20, 5);
    plane.fill_bg(Theme::nord().bg);
    let theme = Theme::nord();

    render_scroll_indicator(
        &mut plane,
        ratatui::layout::Rect::new(0, 0, 20, 5),
        10,
        20,
        10,
        &theme,
    );
}

#[test]
fn test_render_scroll_indicator_not_needed() {
    let mut plane = Plane::new(0, 20, 5);
    plane.fill_bg(Theme::nord().bg);
    let theme = Theme::nord();

    // total == visible, no scroll needed
    render_scroll_indicator(
        &mut plane,
        ratatui::layout::Rect::new(0, 0, 20, 5),
        0,
        10,
        10,
        &theme,
    );
}

#[test]
fn test_render_scroll_indicator_height_one() {
    let mut plane = Plane::new(0, 20, 1);
    plane.fill_bg(Theme::nord().bg);
    let theme = Theme::nord();

    // height <= 1, should not render
    render_scroll_indicator(
        &mut plane,
        ratatui::layout::Rect::new(0, 0, 20, 1),
        0,
        20,
        10,
        &theme,
    );
}
