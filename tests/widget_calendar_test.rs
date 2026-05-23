//! Tests for the Calendar widget.

use chrono::{Datelike, Local, NaiveDate};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::calendar::Calendar;
use ratatui::layout::Rect;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_calendar_new() {
    let cal = Calendar::new();
    let today = Local::now().date_naive();
    
    // month and year are private, but we can verify the calendar renders correctly
    // for the current date
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_calendar_with_id() {
    use dracon_terminal_engine::framework::widget::WidgetId;
    
    let id = WidgetId::new(42);
    let cal = Calendar::with_id(id);
    
    assert_eq!(cal.id(), id);
}

#[test]
fn test_calendar_selected_none_initially() {
    let cal = Calendar::new();
    assert!(cal.selected().is_none());
}

#[test]
fn test_calendar_range_start_none_initially() {
    let cal = Calendar::new();
    assert!(cal.range_start().is_none());
}

#[test]
fn test_calendar_range_end_none_initially() {
    let cal = Calendar::new();
    assert!(cal.range_end().is_none());
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_calendar_with_theme() {
    let cal = Calendar::new().with_theme(Theme::nord());
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    assert_eq!(plane.width, 25);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_calendar_with_range_mode() {
    let _cal = Calendar::new().with_range_mode();
}

#[test]
fn test_calendar_on_select() {
    let _cal = Calendar::new()
        .on_select(|_| {});
}

#[test]
fn test_calendar_on_range_select() {
    let _cal = Calendar::new()
        .on_range_select(|_, _| {});
}

#[test]
fn test_calendar_chained_builders() {
    let _cal = Calendar::new()
        .with_theme(Theme::cyberpunk())
        .with_range_mode()
        .on_select(|_| {});
}

// ============================================================================
// Month Navigation Tests
// ============================================================================

#[test]
fn test_calendar_set_month_valid() {
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    // Verify by rendering - the calendar should show June 2024
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    assert_eq!(plane.width, 25);
    assert!(plane.height > 0);
}

#[test]
fn test_calendar_set_month_january() {
    let mut cal = Calendar::new();
    cal.set_month(1, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
}

#[test]
fn test_calendar_set_month_december() {
    let mut cal = Calendar::new();
    cal.set_month(12, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
}

#[test]
fn test_calendar_set_month_clamp_low() {
    let mut cal = Calendar::new();
    cal.set_month(0, 2024);
    
    // Should clamp to 1, verified by no panic
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
}

#[test]
fn test_calendar_set_month_clamp_high() {
    let mut cal = Calendar::new();
    cal.set_month(13, 2024);
    
    // Should clamp to 12, verified by no panic
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
}

#[test]
fn test_calendar_set_month_year_change() {
    let mut cal = Calendar::new();
    cal.set_month(1, 2023);
    
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_calendar_id() {
    let cal = Calendar::new();
    let _id = cal.id();
}

#[test]
fn test_calendar_area() {
    let cal = Calendar::new();
    let area = cal.area();
    assert_eq!(area.width, 25);
    assert_eq!(area.height, 10);
}

#[test]
fn test_calendar_set_area() {
    let mut cal = Calendar::new();
    let new_area = Rect::new(10, 20, 30, 15);
    cal.set_area(new_area);
    assert_eq!(cal.area(), new_area);
}

#[test]
fn test_calendar_needs_render() {
    let cal = Calendar::new();
    assert!(cal.needs_render());
}

#[test]
fn test_calendar_mark_dirty() {
    let mut cal = Calendar::new();
    cal.clear_dirty();
    assert!(!cal.needs_render());
    cal.mark_dirty();
    assert!(cal.needs_render());
}

#[test]
fn test_calendar_clear_dirty() {
    let mut cal = Calendar::new();
    cal.clear_dirty();
    assert!(!cal.needs_render());
}

#[test]
fn test_calendar_render() {
    let cal = Calendar::new();
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    assert_eq!(plane.width, 25);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_calendar_render_different_size() {
    let cal = Calendar::new();
    let area = Rect::new(0, 0, 40, 20);
    let plane = cal.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 20);
}

#[test]
fn test_calendar_render_minimal_area() {
    let cal = Calendar::new();
    let area = Rect::new(0, 0, 10, 5);
    let plane = cal.render(area);
    assert_eq!(plane.width, 10);
    assert_eq!(plane.height, 5);
}

#[test]
fn test_calendar_focusable() {
    let cal = Calendar::new();
    assert!(cal.focusable());
}

#[test]
fn test_calendar_z_index() {
    let cal = Calendar::new();
    assert_eq!(cal.z_index(), 10);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_calendar_different_themes() {
    for theme_name in ["nord", "dracula", "monokai", "solarized_dark"] {
        if let Some(theme) = Theme::from_name(theme_name) {
            let cal = Calendar::new().with_theme(theme);
            let area = Rect::new(0, 0, 25, 10);
            let plane = cal.render(area);
            assert_eq!(plane.width, 25);
            assert_eq!(plane.height, 10);
        }
    }
}

// ============================================================================
// Internal Logic Tests (via rendering behavior)
// ============================================================================

#[test]
fn test_calendar_february_leap_year() {
    let mut cal = Calendar::new();
    
    // 2024 is a leap year - set to Feb and verify rendering
    cal.set_month(2, 2024);
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_calendar_february_non_leap_year() {
    let mut cal = Calendar::new();
    
    // 2023 is not a leap year
    cal.set_month(2, 2023);
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_calendar_all_months() {
    let mut cal = Calendar::new();
    
    for month in 1..=12 {
        cal.set_month(month, 2024);
        let area = Rect::new(0, 0, 25, 10);
        let plane = cal.render(area);
        assert!(plane.width > 0, "Month {} should render", month);
    }
}

// ============================================================================
// Handle Key Tests
// ============================================================================

#[test]
fn test_calendar_handle_key_arrow_right() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Right, KeyModifiers::empty());
    let result = cal.handle_key(key);
    assert!(result);
}

#[test]
fn test_calendar_handle_key_arrow_left() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Left, KeyModifiers::empty());
    let result = cal.handle_key(key);
    assert!(result);
}

#[test]
fn test_calendar_handle_key_arrow_up() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Up, KeyModifiers::empty());
    let result = cal.handle_key(key);
    assert!(result);
}

#[test]
fn test_calendar_handle_key_arrow_down() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Down, KeyModifiers::empty());
    let result = cal.handle_key(key);
    assert!(result);
}

#[test]
fn test_calendar_handle_key_enter() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Enter, KeyModifiers::empty());
    let result = cal.handle_key(key);
    assert!(result);
}

#[test]
fn test_calendar_handle_key_escape() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Esc, KeyModifiers::empty());
    let result = cal.handle_key(key);
    assert!(result);
}

#[test]
fn test_calendar_handle_key_non_navigation() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    // Tab key should not be handled
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Tab, KeyModifiers::empty());
    let result = cal.handle_key(key);
    assert!(!result);
}

#[test]
fn test_calendar_handle_key_character() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    // Character keys should not be handled
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Char('a'), KeyModifiers::empty());
    let result = cal.handle_key(key);
    assert!(!result);
}

// ============================================================================
// Handle Mouse Tests
// ============================================================================

#[test]
fn test_calendar_handle_mouse_prev_month() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area); // Must render to register zones
    
    // Click on prev month button area
    let result = cal.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(result);
}

#[test]
fn test_calendar_handle_mouse_next_month() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
    
    // Click on next month button area
    let result = cal.handle_mouse(MouseEventKind::Down(MouseButton::Left), 24, 0);
    assert!(result);
}

#[test]
fn test_calendar_handle_mouse_outside() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
    
    // Click far outside any zone
    let result = cal.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 100);
    assert!(!result);
}

#[test]
fn test_calendar_handle_mouse_middle_button() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
    
    let result = cal.handle_mouse(MouseEventKind::Down(MouseButton::Middle), 5, 5);
    assert!(!result); // Middle button not handled
}

#[test]
fn test_calendar_handle_mouse_right_button() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
    
    let result = cal.handle_mouse(MouseEventKind::Down(MouseButton::Right), 5, 5);
    // Right button handling depends on implementation
    let _ = result; // Just verify no crash
}

#[test]
fn test_calendar_handle_mouse_moved() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
    
    // Move mouse over the calendar
    let result = cal.handle_mouse(MouseEventKind::Moved, 10, 6);
    assert!(result);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_calendar_year_boundaries() {
    let mut cal = Calendar::new();
    
    // Very old date
    cal.set_month(1, 1900);
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
    
    // Far future date
    cal.set_month(12, 2100);
    let _plane = cal.render(area);
}

#[test]
fn test_calendar_negative_year() {
    let mut cal = Calendar::new();
    cal.set_month(1, -100);
    
    // Should handle gracefully
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_calendar_render_fills_bg() {
    let cal = Calendar::new();
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    // Background should be filled with theme.bg
    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_calendar_render_has_content() {
    let cal = Calendar::new();
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    
    // There should be non-empty cells in the rendered plane
    let has_content = plane.cells.iter().any(|c| c.char != '\0' && c.char != ' ');
    assert!(has_content, "Calendar should render some content");
}

#[test]
fn test_calendar_render_contains_day_headers() {
    let cal = Calendar::new();
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    
    // Calendar should contain day header letters (M, T, W, T, F, S, S)
    let has_headers = plane.cells.iter().any(|c| 
        c.char == 'M' || c.char == 'T' || c.char == 'W' || c.char == 'F'
    );
    assert!(has_headers, "Calendar should contain day headers");
}

#[test]
fn test_calendar_render_contains_navigation() {
    let cal = Calendar::new();
    let area = Rect::new(0, 0, 25, 10);
    let plane = cal.render(area);
    
    // Calendar should contain < and > for navigation
    let has_nav = plane.cells.iter().any(|c| c.char == '<' || c.char == '>');
    assert!(has_nav, "Calendar should contain navigation arrows");
}

// ============================================================================
// Callback Tests
// ============================================================================

#[test]
fn test_calendar_select_callback_invocation() {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let selected_dates = Rc::new(RefCell::new(Vec::new()));
    let selected_clone = Rc::clone(&selected_dates);
    
    let mut cal = Calendar::new()
        .on_select(move |date| {
            selected_clone.borrow_mut().push(date);
        });
    
    cal.set_month(6, 2024);
    
    // Navigate and select a date
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
    
    // Navigate to a date
    for _ in 0..15 {
        cal.handle_key(dracon_terminal_engine::input::event::KeyEvent::new(
            dracon_terminal_engine::input::event::KeyEventKind::Press,
            dracon_terminal_engine::input::event::KeyCode::Right,
            dracon_terminal_engine::input::event::KeyModifiers::empty(),
        ));
    }
    
    // Press Enter to select
    cal.handle_key(dracon_terminal_engine::input::event::KeyEvent::new(
        dracon_terminal_engine::input::event::KeyEventKind::Press,
        dracon_terminal_engine::input::event::KeyCode::Enter,
        dracon_terminal_engine::input::event::KeyModifiers::empty(),
    ));
    
    // Callback should have been called
    assert_eq!(selected_dates.borrow().len(), 1);
}

#[test]
fn test_calendar_range_callback_registration() {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let ranges = Rc::new(RefCell::new(Vec::new()));
    let ranges_clone = Rc::clone(&ranges);
    
    let mut cal = Calendar::new()
        .with_range_mode()
        .on_range_select(move |start, end| {
            ranges_clone.borrow_mut().push((start, end));
        });
    
    // Verify callback registration works (no crash)
    cal.set_month(6, 2024);
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
}

// ============================================================================
// Clear Selection Tests
// ============================================================================

#[test]
fn test_calendar_clear_selection() {
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    // Select a date
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
    for _ in 0..10 {
        cal.handle_key(dracon_terminal_engine::input::event::KeyEvent::new(
            dracon_terminal_engine::input::event::KeyEventKind::Press,
            dracon_terminal_engine::input::event::KeyCode::Right,
            dracon_terminal_engine::input::event::KeyModifiers::empty(),
        ));
    }
    cal.handle_key(dracon_terminal_engine::input::event::KeyEvent::new(
        dracon_terminal_engine::input::event::KeyEventKind::Press,
        dracon_terminal_engine::input::event::KeyCode::Enter,
        dracon_terminal_engine::input::event::KeyModifiers::empty(),
    ));
    
    // Clear selection
    cal.clear_selection();
    
    assert!(cal.selected().is_none());
}

// ============================================================================
// Navigation Integration Tests
// ============================================================================

#[test]
fn test_calendar_navigate_month_forward() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    // Press right to navigate (if there's a month navigation)
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Right, KeyModifiers::empty());
    cal.handle_key(key);
    
    // Should handle without error
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
}

#[test]
fn test_calendar_navigate_week_up() {
    use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let key = KeyEvent::new(KeyEventKind::Press, KeyCode::Up, KeyModifiers::empty());
    cal.handle_key(key);
    
    let area = Rect::new(0, 0, 25, 10);
    let _plane = cal.render(area);
}