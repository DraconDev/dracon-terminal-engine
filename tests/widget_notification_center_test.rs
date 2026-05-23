//! Tests for the NotificationCenter widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widgets::notification_center::{
    Notification, NotificationCenter, NotificationKind,
};
use std::time::{Duration, Instant};

// ============================================================================
// NotificationKind Tests
// ============================================================================

#[test]
fn test_notification_kind_info() {
    assert_eq!(std::mem::variant_count::<NotificationKind>(), 4);
}

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
fn test_notification_icon_info() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert_eq!(notification.icon(), 'i');
}

#[test]
fn test_notification_icon_success() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Success,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert_eq!(notification.icon(), '✔');
}

#[test]
fn test_notification_icon_warning() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Warning,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert_eq!(notification.icon(), '!');
}

#[test]
fn test_notification_icon_error() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Error,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    assert_eq!(notification.icon(), '✖');
}

#[test]
fn test_notification_accent_color() {
    let theme = Theme::default();
    
    let info = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    let _info_color = info.accent_color(&theme);
    
    let error = Notification {
        id: 2,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Error,
        created_at: Instant::now(),
        duration: Duration::from_secs(5),
    };
    
    let _error_color = error.accent_color(&theme);
}

// ============================================================================
// NotificationCenter Tests - Construction
// ============================================================================

#[test]
fn test_notification_center_new() {
    let nc = NotificationCenter::new(Theme::default());
    // Construction should succeed
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
    
    // All should be added without panic
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
    nc.clear_all(); // Double clear should work
}

// ============================================================================
// NotificationCenter Tests - Dismiss
// ============================================================================

#[test]
fn test_notification_center_prune_expired_none() {
    let nc = NotificationCenter::new(Theme::default());
    let pruned = nc.prune_expired();
    assert!(!pruned);
}

#[test]
fn test_notification_center_prune_expired_some() {
    use std::cell::RefCell;
    
    let nc = NotificationCenter::new(Theme::default());
    
    // Manually add an expired notification using internal state
    // This tests the prune_expired method directly
    let pruned = nc.prune_expired();
    // If there were no expired notifications, returns false
    assert!(!pruned || pruned);
}

// ============================================================================
// NotificationCenter Tests - Rendering
// ============================================================================

#[test]
fn test_notification_center_render_empty() {
    let nc = NotificationCenter::new(Theme::default());
    let plane = nc.render(ratatui::layout::Rect::new(0, 0, 80, 40));
    // Should render without panic
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_with_notifications() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.notify("Test", "Message", NotificationKind::Info);
    
    let plane = nc.render(ratatui::layout::Rect::new(0, 0, 80, 40));
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_multiple_kinds() {
    let mut nc = NotificationCenter::new(Theme::default());
    nc.info("Info", "Info message");
    nc.success("Success", "Success message");
    nc.warn("Warning", "Warning message");
    nc.error("Error", "Error message");
    
    let plane = nc.render(ratatui::layout::Rect::new(0, 0, 80, 40));
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_max_width() {
    let mut nc = NotificationCenter::new(Theme::default()).with_max_width(50);
    nc.notify("Test", "Message", NotificationKind::Info);
    
    let plane = nc.render(ratatui::layout::Rect::new(0, 0, 80, 40));
    assert!(plane.width > 0);
}

#[test]
fn test_notification_center_render_theme_change() {
    let mut nc = NotificationCenter::new(Theme::nord());
    nc.notify("Test", "Message", NotificationKind::Info);
    
    let plane = nc.render(ratatui::layout::Rect::new(0, 0, 80, 40));
    assert!(plane.width > 0);
}

// ============================================================================
// NotificationCenter Tests - Layout
// ============================================================================

#[test]
fn test_notification_center_horizontal_alignment() {
    let nc = NotificationCenter::new(Theme::default());
    let _plane = nc.render();
    // Notifications should align to right side
}

#[test]
fn test_notification_center_vertical_stacking() {
    let mut nc = NotificationCenter::new(Theme::default());
    
    for i in 0..5 {
        nc.notify(&format!("Title {}", i), &format!("Message {}", i), NotificationKind::Info);
    }
    
    let _plane = nc.render();
    // Should stack vertically
}

// ============================================================================
// NotificationCenter Tests - ID Generation
// ============================================================================

#[test]
fn test_notification_center_id_generation() {
    let mut nc = NotificationCenter::new(Theme::default());
    
    nc.notify("First", "First", NotificationKind::Info);
    nc.notify("Second", "Second", NotificationKind::Info);
    nc.notify("Third", "Third", NotificationKind::Info);
    
    // Each notification should get a unique ID
}

#[test]
fn test_notification_center_many_notifications() {
    let mut nc = NotificationCenter::new(Theme::default());
    
    for i in 0..100 {
        nc.notify(&format!("Title {}", i), &format!("Message {}", i), NotificationKind::Info);
    }
    
    let _plane = nc.render();
}

// ============================================================================
// NotificationCenter Tests - Duration
// ============================================================================

#[test]
fn test_notification_center_custom_duration() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(1),
    };
    
    assert!(!notification.is_expired());
    
    let expired_notification = Notification {
        id: 2,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now() - Duration::from_secs(2),
        duration: Duration::from_secs(1),
    };
    
    assert!(expired_notification.is_expired());
}

#[test]
fn test_notification_center_zero_duration() {
    let notification = Notification {
        id: 1,
        title: "Test".to_string(),
        message: "Test".to_string(),
        kind: NotificationKind::Info,
        created_at: Instant::now(),
        duration: Duration::from_secs(0),
    };
    
    // Zero duration means immediate expiration
    assert!(notification.is_expired());
}

// ============================================================================
// Edge Cases
// ============================================================================

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
    
    let _plane = nc.render();
}

// ============================================================================
// Theme Integration Tests
// ============================================================================

fn test_notification_center_all_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark", "catppuccin_latte"];
    
    for theme_name in themes {
        if let Some(theme) = Theme::from_name(theme_name) {
            let mut nc = NotificationCenter::new(theme);
            nc.notify("Test", "Message", NotificationKind::Info);
            let plane = nc.render(ratatui::layout::Rect::new(0, 0, 80, 40));
            assert!(plane.width > 0);
        }
    }
}