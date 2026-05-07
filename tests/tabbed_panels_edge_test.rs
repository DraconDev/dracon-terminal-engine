//! Edge-case tests for tabbed_panels example.

use dracon_terminal_engine::examples::_cookbook::tabbed_panels::{TabbedApp, TAB_DASHBOARD, TAB_LOGS, TAB_SETTINGS};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn make_app() -> TabbedApp {
    let quit = Arc::new(AtomicBool::new(false));
    let mut app = TabbedApp::new(quit);
    app.set_area(Rect::new(0, 0, 80, 24));
    app
}

#[test]
fn test_tabbed_panels_mouse_at_tabbar_boundary_no_panic() {
    let mut app = make_app();
    // tabbar_height = 3, so row == 3 is the first row below the tab bar
    // This was the underflow site: row - tabbar_height - 1 = 3 - 3 - 1
    app.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 3);
    // Should not panic
}

#[test]
fn test_tabbed_panels_mouse_at_row_zero_no_panic() {
    let mut app = make_app();
    // Row 0 is inside the tab bar, should be handled by tabbar
    app.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 0);
    // Should not panic
}

#[test]
fn test_tabbed_panels_mouse_inside_logs_tab() {
    let mut app = make_app();
    // Switch to logs tab first
    app.tabbar.set_active(TAB_LOGS);
    // Click inside logs content area (row >= 4)
    let result = app.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 10);
    // Should be handled (either by list or returned as true)
    assert!(result);
}

#[test]
fn test_tabbed_panels_mouse_settings_theme_select() {
    let mut app = make_app();
    app.tabbar.set_active(TAB_SETTINGS);
    // Click on theme select area (row == tabbar_height + 2 = 5, col 20..40)
    let result = app.handle_mouse(MouseEventKind::Down(MouseButton::Left), 25, 5);
    assert!(result);
}

#[test]
fn test_tabbed_panels_mouse_settings_volume_slider() {
    let mut app = make_app();
    app.tabbar.set_active(TAB_SETTINGS);
    // Click on volume slider (row == tabbar_height + 4 = 7, col 20..45)
    let result = app.handle_mouse(MouseEventKind::Down(MouseButton::Left), 25, 7);
    assert!(result);
}

#[test]
fn test_tabbed_panels_cycle_theme_propagates() {
    let mut app = make_app();
    let original_theme = app.theme.name.clone();
    app.cycle_theme();
    assert_ne!(app.theme.name, original_theme);
    // After cycling, all child widgets should have the new theme
    // (We can't easily verify this without exposing internal state,
    // but we can verify the app doesn't panic)
}

#[test]
fn test_tabbed_panels_all_20_themes() {
    let mut app = make_app();
    let first_theme = app.theme.name.clone();
    // Cycle through all themes and verify we get back to the first
    for _ in 0..20 {
        app.cycle_theme();
    }
    assert_eq!(app.theme.name, first_theme);
}
