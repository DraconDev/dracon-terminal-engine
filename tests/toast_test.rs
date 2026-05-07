//! Toast notification tests — lifecycle, auto-dismiss, stacking.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Toast, ToastKind};
use ratatui::layout::Rect;
use std::time::Duration;

#[test]
fn test_toast_new() {
    let toast = Toast::new(WidgetId::new(1), "Hello World");
    assert_eq!(toast.message(), "Hello World");
    assert!(!toast.is_expired());
}

#[test]
fn test_toast_with_kind() {
    let toast = Toast::new(WidgetId::new(1), "Success!").with_kind(ToastKind::Success);
    assert_eq!(toast.message(), "Success!");

    let toast = Toast::new(WidgetId::new(1), "Warning!").with_kind(ToastKind::Warning);
    assert_eq!(toast.message(), "Warning!");

    let toast = Toast::new(WidgetId::new(1), "Error!").with_kind(ToastKind::Error);
    assert_eq!(toast.message(), "Error!");
}

#[test]
fn test_toast_with_duration() {
    let toast = Toast::new(WidgetId::new(1), "Quick").with_duration(Duration::from_millis(100));
    assert!(!toast.is_expired());
}

#[test]
fn test_toast_expires_after_duration() {
    let toast = Toast::new(WidgetId::new(1), "Quick").with_duration(Duration::from_millis(50));
    std::thread::sleep(Duration::from_millis(60));
    assert!(toast.is_expired());
}

#[test]
fn test_toast_not_expired_yet() {
    let toast = Toast::new(WidgetId::new(1), "Long").with_duration(Duration::from_secs(10));
    assert!(!toast.is_expired());
}

#[test]
fn test_toast_render() {
    let toast = Toast::new(WidgetId::new(1), "Test Message").with_theme(Theme::nord());
    let plane = toast.render(Rect::new(0, 0, 40, 3));
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_toast_info_colors() {
    let toast = Toast::new(WidgetId::new(1), "Info")
        .with_kind(ToastKind::Info)
        .with_theme(Theme::nord());
    let plane = toast.render(Rect::new(0, 0, 40, 3));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_toast_success_colors() {
    let toast = Toast::new(WidgetId::new(1), "Success")
        .with_kind(ToastKind::Success)
        .with_theme(Theme::nord());
    let plane = toast.render(Rect::new(0, 0, 40, 3));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_toast_warning_colors() {
    let toast = Toast::new(WidgetId::new(1), "Warning")
        .with_kind(ToastKind::Warning)
        .with_theme(Theme::nord());
    let plane = toast.render(Rect::new(0, 0, 40, 3));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_toast_error_colors() {
    let toast = Toast::new(WidgetId::new(1), "Error")
        .with_kind(ToastKind::Error)
        .with_theme(Theme::nord());
    let plane = toast.render(Rect::new(0, 0, 40, 3));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_toast_theme_change() {
    let mut toast = Toast::new(WidgetId::new(1), "Test");
    toast.on_theme_change(&Theme::cyberpunk());
    let plane = toast.render(Rect::new(0, 0, 40, 3));
    assert!(!plane.cells.is_empty());
}

#[test]
fn test_toast_no_black_background() {
    let toast = Toast::new(WidgetId::new(1), "Test").with_theme(Theme::nord());
    let plane = toast.render(Rect::new(0, 0, 40, 3));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_toast_z_index() {
    let toast = Toast::new(WidgetId::new(1), "Test");
    assert_eq!(toast.z_index(), 90);
}

#[test]
fn test_toast_long_message() {
    let message = "This is a very long message that might need to be truncated or wrapped depending on the available space";
    let toast = Toast::new(WidgetId::new(1), message);
    let plane = toast.render(Rect::new(0, 0, 40, 3));
    assert_eq!(plane.width, 40);
}
