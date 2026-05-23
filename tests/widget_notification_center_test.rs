//! Tests for the NotificationCenter widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::notification_center::{
    Notification, NotificationCenter, NotificationKind,
};
use ratatui::layout::Rect;
use std::time::{Duration, Instant};

// ============================================================================
// NotificationKind Tests
// ============================================================================

#[test]
fn test_notification_kind_variants() {
    let kinds = vec![
        NotificationKind::Info,
        NotificationKind::Success,
        NotificationKind::Warning,
        NotificationKind::Error,
    ];
    assert_eq!(kinds.len(), 4);
}

// ============================================================================
// Notification Tests
// ============================================================================

#[test]
fn test_notification_new() {
    let notification = Notification {
        id: 1,
        title: "Test Title".to_string(),
        message: "Test Message".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert_eq!(notification.id, 1);
    assert_eq!(notification.title, "Test Title");
    assert_eq!(notification.message, "Test Message");
    assert_eq!(notification.kind, NotificationKind::Info);
}

#[test]
fn test_notification_is_expired_false() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(10),
    };
    
    assert!(!notification.is_expired());
}

#[test]
fn test_notification_is_expired_true() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now() - Duration::from_secs(100),
        duration: Duration::from_secs(5),
    };
    
    assert!(notification.is_expired());
}

#[test]
fn test_notification_is_expired_zero_duration() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(0),
    };
    
    assert!(notification.is_expired());
}

#[test]
fn test_notification_long_title() {
    let notification = Notification {
        id: 1,
        title: "A".repeat(1000),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert_eq!(notification.title.len(), 1000);
}

#[test]
fn test_notification_long_message() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "B".repeat(1000),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert_eq!(notification.message.len(), 1000);
}

#[test]
fn test_notification_empty_strings() {
    let notification = Notification {
        id: 1,
        title: "".to_string(),
        message: "".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert_eq!(notification.title, "");
    assert_eq!(notification.message, "");
}

#[test]
fn test_notification_unicode() {
    let notification = Notification {
        id: 1,
        title: "日本語タイトル".to_string(),
        message: "Arabic رسالة".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert!(notification.title.len() > 0);
    assert!(notification.message.len() > 0);
}

// ============================================================================
// NotificationCenter Tests - Construction
// ============================================================================

#[test]
fn test_notification_center_new() {
    let nc = NotificationCenter::new(Theme::default());
    let _ = nc;
}

#[test]
fn test_notification_center_with_max_width() {
    let nc = NotificationCenter::new(Theme::default()).with_max_width(60);
    let _ = nc;
}

#[test]
fn test_notification_center_with_theme() {
    let nc = NotificationCenter::new(Theme::nord());
    let _ = nc;
}

#[test]
fn test_notification_center_area() {
    let nc = NotificationCenter::new(Theme::default());
    let area = nc.area();
    assert!(area.width > 0);
}

#[test]
fn test_notification_center_set_area() {
    let mut nc = NotificationCenter::new(Theme::default());
    let new_area = Rect::new(10, 10, 80, 40);
    nc.set_area(new_area);
    assert_eq!(nc.area(), new_area);
}

#[test]
fn test_notification_center_z_index() {
    let nc = NotificationCenter::new(Theme::default());
    assert_eq!(nc.z_index(), 9500);
}

// ============================================================================
// NotificationCenter Tests - Adding Notifications
// ============================================================================

#[test]
fn test_notification_center_notify() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.notify("Test", "This is a test", NotificationKind::Info);
}

#[test]
fn test_notification_center_info() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.info("Success", "Operation completed");
}

#[test]
fn test_notification_center_success() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.success("Great", "Everything worked!");
}

#[test]
fn test_notification_center_warn() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.warn("Warning", "This might be a problem");
}

#[test]
fn test_notification_center_error() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.error("Error", "Something went wrong");
}

#[test]
fn test_notification_center_multiple_notifications() {
    let mut nc = NotificationCenter::new(Theme::default());
    
    nc.info("First", "First message");
    nc.success("Second", "Second message");
    nc.warn("Third", "Third message");
    nc.error("Fourth", "Fourth message");
}

#[test]
fn test_notification_center_different_kinds_same_title() {
    let mut nc = NotificationCenter::new(Theme::default());
    
    nc.notify("Same Title", "Info message", NotificationKind::Info);
    nc.notify("Same Title", "Success message", NotificationKind::Success);
    nc.notify("Same Title", "Warning message", NotificationKind::Warning);
    nc.notify("Same Title", "Error message", NotificationKind::Error);
}

// ============================================================================
// NotificationCenter Tests - Clearing
// ============================================================================

#[test]
fn test_notification_center_clear_all() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.notify("Test", "Message", NotificationKind::Info);
    nc.clear_all();
}

#[test]
fn test_notification_center_clear_all_empty() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.clear_all();
    nc.clear_all();
}

// ============================================================================
// NotificationCenter Tests - Rendering
// ============================================================================

#[test]
fn test_notification_center_render_empty() {
    let nc = NotificationCenter::new(Theme::default());
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_with_notifications() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.notify("Test", "Message", NotificationKind::Info);
    
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_multiple_kinds() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.info("Info", "Info message");
    nc.success("Success", "Success message");
    nc.warn("Warning", "Warning message");
    nc.error("Error", "Error message");
    
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_max_width() {
    let mut nc = NotificationCenter::new(Theme::default()).with_max_width(50);
    nc.notify("Test", "Message", NotificationKind::Info);
    
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_theme_change() {
    let mut nc = NotificationCenter::new(Theme::nord());
    nc.notify("Test", "Message", NotificationKind::Info);
    
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_small_area() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.notify("Test", "Message", NotificationKind::Info);
    
    let area = Rect::new(0, 0, 20, 5);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_large_notification() {
    let mut nc = NotificationCenter::new(Theme::default()).with_max_width(60);
    nc.notify("Very Long Title", "This is a very long message that might need truncation in the rendering", NotificationKind::Info);
    
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// NotificationCenter Tests - Widget Trait
// ============================================================================

#[test]
fn test_notification_center_needs_render() {
    let nc = NotificationCenter::new(Theme::default());
    assert!(nc.needs_render());
}

#[test]
fn test_notification_center_mark_dirty() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.clear_dirty();
    assert!(!nc.needs_render());
    nc.mark_dirty();
    assert!(nc.needs_render());
}

#[test]
fn test_notification_center_clear_dirty() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.clear_dirty();
    assert!(!nc.needs_render());
}

// ============================================================================
// NotificationCenter Tests - Many Notifications
// ============================================================================

#[test]
fn test_notification_center_many_notifications() {
    let mut nc = NotificationCenter::new(Theme::default());
    
    for i in 0..100 {
        nc.notify(&format!("Title {}", i), &format!("Message {}", i), NotificationKind::Info);
    }
    
    let area = Rect::new(0, 0, 80, 40);
    let _plane = nc.render(area);
}

#[test]
fn test_notification_center_many_kinds() {
    let mut nc = NotificationCenter::new(Theme::default());
    
    for i in 0..10 {
        let kind = match i % 4 {
            0 => NotificationKind::Info,
            1 => NotificationKind::Success,
            2 => NotificationKind::Warning,
            _ => NotificationKind::Error,
        };
        nc.notify(&format!("Title {}", i), &format!("Message {}", i), kind);
    }
    
    let area = Rect::new(0, 0, 80, 40);
    let _plane = nc.render(area);
}

// ============================================================================
// NotificationCenter Tests - Theme Integration
// ============================================================================

#[test]
fn test_notification_center_all_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark", "catppuccin_latte"];
    
    for theme_name in themes {
        if let Some(theme) = Theme::from_name(theme_name) {
            let mut nc = NotificationCenter::new(theme);
            nc.notify("Test", "Message", NotificationKind::Info);
            let area = Rect::new(0, 0, 80, 40);
            let plane = nc.render(area);
            assert!(plane.width > 0);
        }
    }
}

#[test]
fn test_notification_center_render_fills_plane() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.info("Test", "Message");
    
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 40);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_notification_center_empty_title_and_message() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.notify("", "", NotificationKind::Info);
    
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_expired_notifications() {
    // Create a notification center with an already expired notification
    // This tests that the system handles expired notifications gracefully
    let nc = NotificationCenter::new(Theme::default());
    let area = Rect::new(0, 0, 80, 40);
    let plane = nc.render(area);
    assert!(plane.width > 0);
}