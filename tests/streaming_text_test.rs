//! StreamingText widget tests — append, clear, auto-scroll, word-wrap.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::StreamingText;
use dracon_terminal_engine::framework::command::ParsedOutput;
use ratatui::layout::Rect;

#[test]
fn test_streaming_text_new() {
    let st = StreamingText::new();
    assert_eq!(st.max_lines, 200);
    assert!(st.auto_scroll);
    assert!(!st.word_wrap);
}

#[test]
fn test_streaming_text_with_id() {
    let st = StreamingText::with_id(WidgetId::new(1));
    assert_eq!(st.id, WidgetId::new(1));
}

#[test]
fn test_streaming_text_append() {
    let mut st = StreamingText::new();
    st.append("Hello\nWorld");
    assert_eq!(st.lines.len(), 2);
    assert_eq!(st.lines[0], "Hello");
    assert_eq!(st.lines[1], "World");
}

#[test]
fn test_streaming_text_append_single_line() {
    let mut st = StreamingText::new();
    st.append("Hello World");
    assert_eq!(st.lines.len(), 1);
}

#[test]
fn test_streaming_text_max_lines_limit() {
    let mut st = StreamingText::new().max_lines(3);
    st.append("line1\nline2\nline3\nline4\nline5");
    assert_eq!(st.lines.len(), 3);
    assert_eq!(st.lines.back().unwrap(), "line5");
}

#[test]
fn test_streaming_text_clear() {
    let mut st = StreamingText::new();
    st.append("Hello\nWorld");
    st.clear();
    assert!(st.lines.is_empty());
    assert!(st.content.is_empty());
}

#[test]
fn test_streaming_text_append_output_text() {
    let mut st = StreamingText::new();
    st.append_output(ParsedOutput::Text("Hello".to_string()));
    assert_eq!(st.lines.len(), 1);
}

#[test]
fn test_streaming_text_append_output_scalar() {
    let mut st = StreamingText::new();
    st.append_output(ParsedOutput::Scalar("42".to_string()));
    assert_eq!(st.lines.len(), 1);
}

#[test]
fn test_streaming_text_append_output_lines() {
    let mut st = StreamingText::new();
    let log_lines = vec![
        dracon_terminal_engine::framework::widgets::LogLine { text: "Line 1".to_string() },
        dracon_terminal_engine::framework::widgets::LogLine { text: "Line 2".to_string() },
    ];
    st.append_output(ParsedOutput::Lines(log_lines));
    assert_eq!(st.lines.len(), 2);
}

#[test]
fn test_streaming_text_render() {
    let mut st = StreamingText::new().with_theme(Theme::nord());
    st.append("Hello\nWorld\nTest");
    let plane = st.render(Rect::new(0, 0, 80, 10));
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_streaming_text_render_empty() {
    let st = StreamingText::new().with_theme(Theme::nord());
    let plane = st.render(Rect::new(0, 0, 80, 10));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_streaming_text_no_black_background() {
    let mut st = StreamingText::new().with_theme(Theme::nord());
    st.append("Test content");
    let plane = st.render(Rect::new(0, 0, 80, 10));
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_streaming_text_theme_change() {
    let mut st = StreamingText::new();
    st.append("Test");
    st.on_theme_change(&Theme::cyberpunk());
    let plane = st.render(Rect::new(0, 0, 80, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_streaming_text_auto_scroll_option() {
    let mut st = StreamingText::new().auto_scroll(false);
    st.append("Test");
    assert!(!st.auto_scroll);
}

#[test]
fn test_streaming_text_word_wrap_option() {
    let mut st = StreamingText::new().word_wrap(true);
    st.append("Test");
    assert!(st.word_wrap);
}

#[test]
fn test_streaming_text_large_content() {
    let mut st = StreamingText::new().max_lines(10);
    for i in 0..100 {
        st.append(&format!("Line {}\n", i));
    }
    assert_eq!(st.lines.len(), 10);
}
