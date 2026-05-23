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
    
    assert_eq!(cal.month(), today.month() as u8);
    assert_eq!(cal.year(), today.year());
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
    let cal = Calendar::new().with_range_mode();
    // Range mode should be enabled
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
    let cal = Calendar::new()
        .with_theme(Theme::cyberpunk())
        .with_range_mode()
        .on_select(|_| {});
    
    // Just verify it compiles and runs
}

// ============================================================================
// Month Navigation Tests
// ============================================================================

#[test]
fn test_calendar_set_month_valid() {
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    assert_eq!(cal.month(), 6);
    assert_eq!(cal.year(), 2024);
}

#[test]
fn test_calendar_set_month_january() {
    let mut cal = Calendar::new();
    cal.set_month(1, 2024);
    
    assert_eq!(cal.month(), 1);
    assert_eq!(cal.year(), 2024);
}

#[test]
fn test_calendar_set_month_december() {
    let mut cal = Calendar::new();
    cal.set_month(12, 2024);
    
    assert_eq!(cal.month(), 12);
    assert_eq!(cal.year(), 2024);
}

#[test]
fn test_calendar_set_month_clamp_low() {
    let mut cal = Calendar::new();
    cal.set_month(0, 2024);
    
    assert_eq!(cal.month(), 1); // Clamped to 1
}

#[test]
fn test_calendar_set_month_clamp_high() {
    let mut cal = Calendar::new();
    cal.set_month(13, 2024);
    
    assert_eq!(cal.month(), 12); // Clamped to 12
}

#[test]
fn test_calendar_set_month_year_change() {
    let mut cal = Calendar::new();
    cal.set_month(1, 2023);
    
    assert_eq!(cal.month(), 1);
    assert_eq!(cal.year(), 2023);
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
// Date Logic Tests
// ============================================================================

#[test]
fn test_calendar_days_in_month_january() {
    let mut cal = Calendar::new();
    cal.set_month(1, 2024);
    assert_eq!(cal.days_in_month(), 31);
}

#[test]
fn test_calendar_days_in_month_february_leap() {
    let mut cal = Calendar::new();
    cal.set_month(2, 2024); // 2024 is a leap year
    assert_eq!(cal.days_in_month(), 29);
}

#[test]
fn test_calendar_days_in_month_february_non_leap() {
    let mut cal = Calendar::new();
    cal.set_month(2, 2023); // 2023 is not a leap year
    assert_eq!(cal.days_in_month(), 28);
}

#[test]
fn test_calendar_days_in_month_april() {
    let mut cal = Calendar::new();
    cal.set_month(4, 2024);
    assert_eq!(cal.days_in_month(), 30);
}

#[test]
fn test_calendar_days_in_month_all_months() {
    let expected_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut cal = Calendar::new();
    
    for (month, expected) in expected_days.iter().enumerate() {
        cal.set_month((month + 1) as u8, 2024);
        // For February in leap year, adjust expectation
        let actual = if month == 1 && *expected == 28 { 29 } else { *expected };
        assert_eq!(cal.days_in_month(), actual as u32, "Month {}", month + 1);
    }
}

// ============================================================================
// Selection Tests
// ============================================================================

#[test]
fn test_calendar_select_date() {
    let mut cal = Calendar::new();
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    
    assert!(cal.selected().is_some());
    assert_eq!(cal.selected().unwrap(), NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
}

#[test]
fn test_calendar_select_date_changes_selection() {
    let mut cal = Calendar::new();
    
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    assert_eq!(cal.selected().unwrap(), NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    
    cal.select_date(NaiveDate::from_ymd_opt(2024, 7, 20).unwrap());
    assert_eq!(cal.selected().unwrap(), NaiveDate::from_ymd_opt(2024, 7, 20).unwrap());
}

// ============================================================================
// Range Selection Tests
// ============================================================================

#[test]
fn test_calendar_range_mode_select() {
    let mut cal = Calendar::new().with_range_mode();
    
    let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    
    cal.select_date(start);
    assert_eq!(cal.range_start(), Some(start));
    assert!(cal.range_end().is_none());
    
    cal.select_date(end);
    assert_eq!(cal.range_start(), Some(start));
    assert_eq!(cal.range_end(), Some(end));
}

#[test]
fn test_calendar_range_swaps_if_reversed() {
    let mut cal = Calendar::new().with_range_mode();
    
    // Select end first, then start
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
    
    // Should swap to maintain start <= end
    assert_eq!(cal.range_start(), Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()));
    assert_eq!(cal.range_end(), Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()));
}

#[test]
fn test_calendar_range_reset_after_selection() {
    let mut cal = Calendar::new().with_range_mode();
    
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    
    // Third click should reset
    cal.select_date(NaiveDate::from_ymd_opt(2024, 7, 1).unwrap());
    
    assert_eq!(cal.range_start(), Some(NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()));
    assert!(cal.range_end().is_none());
}

// ============================================================================
// Month Name Tests
// ============================================================================

#[test]
fn test_calendar_month_name() {
    let months = [
        "January", "February", "March", "April",
        "May", "June", "July", "August",
        "September", "October", "November", "December"
    ];
    
    let mut cal = Calendar::new();
    
    for (i, name) in months.iter().enumerate() {
        cal.set_month((i + 1) as u8, 2024);
        assert_eq!(cal.month_name(), *name, "Month {}", i + 1);
    }
}

// ============================================================================
// Week Start Offset Tests
// ============================================================================

#[test]
fn test_calendar_start_offset() {
    let mut cal = Calendar::new();
    
    // January 2024 starts on Monday (weekday 1 in chrono, Monday=0 for us)
    cal.set_month(1, 2024);
    assert_eq!(cal.start_offset(), 0); // Monday
}

// ============================================================================
// Date for Index Tests
// ============================================================================

#[test]
fn test_calendar_date_for_index_valid() {
    let mut cal = Calendar::new();
    cal.set_month(1, 2024);
    
    // First day of January 2024 is Monday, so index 0 should be None (offset)
    assert!(cal.date_for_index(0).is_none());
    
    // Index 1 should be January 1st
    let first = cal.date_for_index(1);
    assert!(first.is_some());
    assert_eq!(first.unwrap().day(), 1);
}

#[test]
fn test_calendar_date_for_index_invalid() {
    let mut cal = Calendar::new();
    cal.set_month(2, 2024); // February has 29 days in leap year
    
    // Index 31+offset should be None
    let offset = cal.start_offset() as usize;
    let beyond = 30 + offset + 1;
    assert!(cal.date_for_index(beyond).is_none());
}

#[test]
fn test_calendar_date_for_index_out_of_bounds() {
    let cal = Calendar::new();
    
    assert!(cal.date_for_index(42).is_none()); // Grid has 0-41
    assert!(cal.date_for_index(100).is_none());
}

// ============================================================================
// Today Tests
// ============================================================================

#[test]
fn test_calendar_today() {
    let cal = Calendar::new();
    let today = Local::now().date_naive();
    
    assert_eq!(cal.today(), today);
}

// ============================================================================
// Handle Key Tests
// ============================================================================

#[test]
fn test_calendar_handle_key_arrow_keys() {
    use ratatui::key_code::KeyCode;
    use ratatui::key_event::KeyEvent;
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    // ArrowRight
    let right = KeyEvent::new(ratatui::key_event::KeyEventKind::Press, KeyCode::Right, ratatui::key_code::KeyModifiers::empty());
    let result = cal.handle_key(right);
    assert!(result);
    
    // ArrowLeft
    let left = KeyEvent::new(ratatui::key_event::KeyEventKind::Press, KeyCode::Left, ratatui::key_code::KeyModifiers::empty());
    let result = cal.handle_key(left);
    assert!(result);
    
    // ArrowUp
    let up = KeyEvent::new(ratatui::key_event::KeyEventKind::Press, KeyCode::Up, ratatui::key_code::KeyModifiers::empty());
    let result = cal.handle_key(up);
    assert!(result);
    
    // ArrowDown
    let down = KeyEvent::new(ratatui::key_event::KeyEventKind::Press, KeyCode::Down, ratatui::key_code::KeyModifiers::empty());
    let result = cal.handle_key(down);
    assert!(result);
}

#[test]
fn test_calendar_handle_key_enter() {
    use ratatui::key_code::KeyCode;
    use ratatui::key_event::KeyEvent;
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    cal.handle_key(ratatui::key_event::KeyEvent::new(
        ratatui::key_event::KeyEventKind::Press,
        KeyCode::Enter,
        ratatui::key_code::KeyModifiers::empty()
    ));
    // Enter should select the hovered day
}

#[test]
fn test_calendar_handle_key_escape() {
    use ratatui::key_code::KeyCode;
    use ratatui::key_event::KeyEvent;
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    // Escape should not change state
    let esc = KeyEvent::new(ratatui::key_event::KeyEventKind::Press, KeyCode::Esc, ratatui::key_code::KeyModifiers::empty());
    let result = cal.handle_key(esc);
    assert!(result);
}

#[test]
fn test_calendar_handle_key_non_navigation() {
    use ratatui::key_code::KeyCode;
    use ratatui::key_event::KeyEvent;
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    // Non-navigation keys should return false (not handled)
    let tab = KeyEvent::new(ratatui::key_event::KeyEventKind::Press, KeyCode::Tab, ratatui::key_code::KeyModifiers::empty());
    let result = cal.handle_key(tab);
    assert!(!result);
}

// ============================================================================
// Handle Mouse Tests
// ============================================================================

#[test]
fn test_calendar_handle_mouse_prev_next_buttons() {
    use dracon_terminal_engine::compositor::MouseEventKind;
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area); // Must render to register zones
    
    // Click on prev month button
    let prev_result = cal.handle_mouse(MouseEventKind::Down(ratatui::mouse::MouseButton::Left), 0, 0);
    // Result depends on zone detection
    assert!(prev_result);
    
    // Click on next month button
    let next_result = cal.handle_mouse(MouseEventKind::Down(ratatui::mouse::MouseButton::Left), 24, 0);
    assert!(next_result);
}

#[test]
fn test_calendar_handle_mouse_outside() {
    use dracon_terminal_engine::compositor::MouseEventKind;
    
    let mut cal = Calendar::new();
    cal.set_month(6, 2024);
    
    let area = Rect::new(0, 0, 25, 10);
    cal.render(area);
    
    // Click far outside any zone
    let result = cal.handle_mouse(MouseEventKind::Down(ratatui::mouse::MouseButton::Left), 100, 100);
    assert!(!result);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_calendar_leap_year_february() {
    let mut cal = Calendar::new();
    
    // 2024 is a leap year
    cal.set_month(2, 2024);
    assert_eq!(cal.days_in_month(), 29);
    
    // 2023 is not a leap year
    cal.set_month(2, 2023);
    assert_eq!(cal.days_in_month(), 28);
}

#[test]
fn test_calendar_year_boundaries() {
    let mut cal = Calendar::new();
    
    // Very old date
    cal.set_month(1, 1900);
    assert_eq!(cal.month(), 1);
    assert_eq!(cal.year(), 1900);
    
    // Far future date
    cal.set_month(12, 2100);
    assert_eq!(cal.month(), 12);
    assert_eq!(cal.year(), 2100);
}

#[test]
fn test_calendar_set_month_negative_year() {
    let mut cal = Calendar::new();
    cal.set_month(1, -100);
    
    // Should handle negative years
    assert_eq!(cal.year(), -100);
}

// ============================================================================
// Callback Tests
// ============================================================================

#[test]
fn test_calendar_select_callback_registration() {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let selected_dates = Rc::new(RefCell::new(Vec::new()));
    let selected_clone = Rc::clone(&selected_dates);
    
    let mut cal = Calendar::new()
        .on_select(move |date| {
            selected_clone.borrow_mut().push(date);
        });
    
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    
    assert_eq!(selected_dates.borrow().len(), 1);
    assert_eq!(selected_dates.borrow()[0], NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
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
    
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
    cal.select_date(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    
    assert_eq!(ranges.borrow().len(), 1);
    assert_eq!(ranges.borrow()[0].0, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
    assert_eq!(ranges.borrow()[0].1, NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
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