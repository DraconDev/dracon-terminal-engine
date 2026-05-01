mod common;

use dracon_terminal_engine::framework::command::{LoggedLine, ParsedOutput};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::log_viewer::{LogLevel, LogViewer};
use ratatui::layout::Rect;

#[test]
fn test_log_viewer_new() {
    let lv = LogViewer::new();
    assert!(lv.lines.is_empty());
    assert_eq!(lv.max_lines, 500);
    assert!(lv.auto_scroll);
}

#[test]
fn test_log_viewer_with_id() {
    let lv = LogViewer::with_id(dracon_terminal_engine::framework::widget::WidgetId::new(2));
    assert_eq!(lv.id, dracon_terminal_engine::framework::widget::WidgetId::new(2));
}

#[test]
fn test_log_viewer_max_lines() {
    let lv = LogViewer::new().max_lines(100);
    assert_eq!(lv.max_lines, 100);
}

#[test]
fn test_log_viewer_auto_scroll() {
    let lv = LogViewer::new().auto_scroll(false);
    assert!(!lv.auto_scroll);
}

#[test]
fn test_log_viewer_filter() {
    let lv = LogViewer::new().filter("error");
    assert_eq!(lv.filter, Some("error".to_string()));
}

#[test]
fn test_log_viewer_bind_command() {
    use dracon_terminal_engine::framework::command::BoundCommand;
    let cmd = BoundCommand::new("journalctl -f").label("logs");
    let lv = LogViewer::new().bind_command(cmd);
    assert_eq!(lv.commands().len(), 1);
}

#[test]
fn test_log_viewer_append_line() {
    let mut lv = LogViewer::new();
    lv.append_line("2024-01-01 INFO Starting up");
    assert_eq!(lv.lines.len(), 1);
    assert_eq!(lv.lines[0].message, "INFO Starting up");
}

#[test]
fn test_log_viewer_append_line_max_lines() {
    let mut lv = LogViewer::new().max_lines(3);
    for i in 0..5 {
        lv.append_line(&format!("line {}", i));
    }
    assert_eq!(lv.lines.len(), 3);
    assert_eq!(lv.lines[0].raw, "line 2");
    assert_eq!(lv.lines[2].raw, "line 4");
}

#[test]
fn test_log_viewer_parse_level_error() {
    let mut lv = LogViewer::new();
    lv.append_line("ERROR something bad happened");
    assert_eq!(lv.lines[0].level, LogLevel::Error);
}

#[test]
fn test_log_viewer_parse_level_warn() {
    let mut lv = LogViewer::new();
    lv.append_line("WARNING deprecated feature");
    assert_eq!(lv.lines[0].level, LogLevel::Warn);
}

#[test]
fn test_log_viewer_parse_level_debug() {
    let mut lv = LogViewer::new();
    lv.append_line("DEBUG connection established");
    assert_eq!(lv.lines[0].level, LogLevel::Debug);
}

#[test]
fn test_log_viewer_parse_level_fatal() {
    let mut lv = LogViewer::new();
    lv.append_line("FATAL system crash");
    assert_eq!(lv.lines[0].level, LogLevel::Fatal);
}

#[test]
fn test_log_viewer_filter_reject() {
    let mut lv = LogViewer::new().filter("error");
    lv.append_line("INFO this is info");
    assert!(lv.lines.is_empty());
}

#[test]
fn test_log_viewer_filter_accept() {
    let mut lv = LogViewer::new().filter("error");
    lv.append_line("ERROR something failed");
    assert_eq!(lv.lines.len(), 1);
}

#[test]
fn test_log_viewer_clear() {
    let mut lv = LogViewer::new();
    lv.append_line("test line");
    lv.clear();
    assert!(lv.lines.is_empty());
}

#[test]
fn test_log_viewer_render() {
    let mut lv = LogViewer::new();
    lv.append_line("INFO test message");
    let plane = lv.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane.cells[0].char, '[');
}

#[test]
fn test_log_viewer_render_empty() {
    let lv = LogViewer::new();
    let plane = lv.render(Rect::new(0, 0, 30, 5));
    assert!(plane.cells.iter().any(|c| c.char == '('));
}

#[test]
fn test_log_viewer_dirty_lifecycle() {
    let mut lv = LogViewer::new();
    assert!(lv.needs_render());
    lv.clear_dirty();
    assert!(!lv.needs_render());
    lv.append_line("new line");
    assert!(lv.needs_render());
}

#[test]
fn test_log_viewer_with_theme() {
    let theme = Theme::nord();
    let lv = LogViewer::new().with_theme(theme);
    assert_eq!(lv.theme.name, "nord");
}

#[test]
fn test_log_viewer_level_color() {
    let lv = LogViewer::new();
    assert_eq!(lv.level_color(LogLevel::Error), lv.theme.error_fg);
    assert_eq!(lv.level_color(LogLevel::Warn), lv.theme.warning_fg);
}

#[test]
fn test_log_viewer_level_prefix() {
    let lv = LogViewer::new();
    assert_eq!(lv.level_prefix(LogLevel::Error), "[E]");
    assert_eq!(lv.level_prefix(LogLevel::Info), "[I]");
}

#[test]
fn test_log_viewer_apply_command_output_text() {
    let mut lv = LogViewer::new();
    lv.apply_command_output(&ParsedOutput::Text(
        "ERROR test error\nINFO test info".to_string(),
    ));
    assert_eq!(lv.lines.len(), 2);
}

#[test]
fn test_log_viewer_apply_command_output_lines() {
    let mut lv = LogViewer::new();
    lv.apply_command_output(&ParsedOutput::Lines(vec![
        LoggedLine::new("FATAL crash", "fatal"),
        LoggedLine::new("ERROR failure", "error"),
    ]));
    assert_eq!(lv.lines.len(), 2);
}