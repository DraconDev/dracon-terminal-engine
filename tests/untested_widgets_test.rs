//! Basic smoke tests for previously untested widget modules.
//!
//! Each test verifies that the widget can be constructed and rendered
//! without panicking, and that basic properties hold.

use dracon_terminal_engine::compositor::Color as DteColor;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Autocomplete, Calendar, ColorPicker, Kanban, KanbanCard, NotificationCenter, RichText,
    TagsInput,
};
use ratatui::layout::Rect;

fn test_area() -> Rect {
    Rect::new(0, 0, 40, 10)
}

// ── Autocomplete ──────────────────────────────────────────────────────────────

#[test]
fn test_autocomplete_new() {
    let ac = Autocomplete::new(WidgetId::new(1), vec!["foo".into(), "bar".into()]);
    assert_eq!(ac.id(), WidgetId::new(1));
}

#[test]
fn test_autocomplete_render() {
    let ac = Autocomplete::new(WidgetId::new(1), vec!["alpha".into(), "beta".into()])
        .with_theme(Theme::nord());
    let plane = ac.render(test_area());
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_autocomplete_with_max_visible() {
    let ac = Autocomplete::new(WidgetId::new(2), vec!["a".into()])
        .with_max_visible(3);
    let _plane = ac.render(test_area());
}

// ── ColorPicker ───────────────────────────────────────────────────────────────

#[test]
fn test_color_picker_new() {
    let cp = ColorPicker::new();
    let plane = cp.render(test_area());
    assert_eq!(plane.width, 40);
}

#[test]
fn test_color_picker_with_color() {
    let cp = ColorPicker::with_color(DteColor::Rgb(255, 0, 0));
    let _plane = cp.render(test_area());
}

#[test]
fn test_color_picker_with_hex() {
    let cp = ColorPicker::with_hex("#ff0000");
    let _plane = cp.render(test_area());
}

#[test]
fn test_color_picker_with_theme() {
    let cp = ColorPicker::new().with_theme(Theme::cyberpunk());
    let plane = cp.render(test_area());
    assert!(plane.width > 0);
}

// ── TagsInput ─────────────────────────────────────────────────────────────────

#[test]
fn test_tags_input_new() {
    let ti = TagsInput::new(vec!["rust".into(), "tui".into()]);
    assert!(!ti.tags().is_empty());
}

#[test]
fn test_tags_input_with_placeholder() {
    let ti = TagsInput::new(vec![]).with_placeholder("Add tags...");
    assert_eq!(ti.tags().len(), 0);
}

#[test]
fn test_tags_input_with_theme() {
    let ti = TagsInput::new(vec!["test".into()]).with_theme(Theme::dracula());
    assert_eq!(ti.tags().len(), 1);
}

// ── Kanban ────────────────────────────────────────────────────────────────────

#[test]
fn test_kanban_card_new() {
    let card = KanbanCard::new("1", "Task A");
    assert_eq!(card.title, "Task A");
}

#[test]
fn test_kanban_card_with_description() {
    let card = KanbanCard::new("2", "Task B").with_description("Details here");
    assert_eq!(card.description, Some("Details here".to_string()));
}

#[test]
fn test_kanban_new() {
    let kb = Kanban::new();
    let plane = kb.render(test_area());
    assert!(plane.width > 0);
}

#[test]
fn test_kanban_with_columns() {
    let kb = Kanban::with_columns(vec![
        ("Todo", vec!["Task 1"]),
        ("Done", vec!["Task 2"]),
    ])
    .with_theme(Theme::nord());
    let plane = kb.render(Rect::new(0, 0, 60, 12));
    assert_eq!(plane.width, 60);
}

// ── NotificationCenter ────────────────────────────────────────────────────────

#[test]
fn test_notification_center_new() {
    let nc = NotificationCenter::new(Theme::nord());
    assert!(nc.is_empty());
}

#[test]
fn test_notification_center_info() {
    let mut nc = NotificationCenter::new(Theme::nord());
    nc.info("Title", "Message");
    assert_eq!(nc.len(), 1);
}

#[test]
fn test_notification_center_success() {
    let mut nc = NotificationCenter::new(Theme::nord());
    nc.success("OK", "It worked");
    assert_eq!(nc.len(), 1);
}

#[test]
fn test_notification_center_warn() {
    let mut nc = NotificationCenter::new(Theme::nord());
    nc.warn("Warning", "Check this");
    assert_eq!(nc.len(), 1);
}

#[test]
fn test_notification_center_error() {
    let mut nc = NotificationCenter::new(Theme::nord());
    nc.error("Error", "Something failed");
    assert_eq!(nc.len(), 1);
}

// ── RichText ──────────────────────────────────────────────────────────────────

#[test]
fn test_rich_text_new() {
    let rt = RichText::new("# Hello\nWorld");
    let plane = rt.render(test_area());
    assert_eq!(plane.width, 40);
}

#[test]
fn test_rich_text_with_id() {
    let rt = RichText::with_id(WidgetId::new(10), "**bold**");
    assert_eq!(rt.id(), WidgetId::new(10));
}

#[test]
fn test_rich_text_with_theme() {
    let rt = RichText::new("*italic*").with_theme(Theme::dracula());
    let _plane = rt.render(test_area());
}

#[test]
fn test_rich_text_empty_content() {
    let rt = RichText::new("");
    let _plane = rt.render(test_area());
}

#[test]
fn test_rich_text_multiline() {
    let content = "# Title\n\nParagraph\n- item 1\n- item 2\n```\ncode\n```";
    let rt = RichText::new(content).with_theme(Theme::nord());
    let _plane = rt.render(Rect::new(0, 0, 60, 20));
}

// ── Calendar ──────────────────────────────────────────────────────────────────

#[test]
fn test_calendar_new() {
    let cal = Calendar::new();
    let plane = cal.render(test_area());
    assert_eq!(plane.width, 40);
}

#[test]
fn test_calendar_with_id() {
    let cal = Calendar::with_id(WidgetId::new(5));
    assert_eq!(cal.id(), WidgetId::new(5));
}

#[test]
fn test_calendar_with_theme() {
    let cal = Calendar::new().with_theme(Theme::cyberpunk());
    let _plane = cal.render(test_area());
}

#[test]
fn test_calendar_with_range_mode() {
    let cal = Calendar::new().with_range_mode();
    let _plane = cal.render(test_area());
}
