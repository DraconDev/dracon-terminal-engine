mod common;

use dracon_terminal_engine::framework::command::ParsedOutput;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::streaming_text::StreamingText;
use ratatui::layout::Rect;

#[test]
fn test_streaming_text_new() {
    let st = StreamingText::new();
    assert!(st.lines.is_empty());
    assert_eq!(st.max_lines, 200);
    assert!(st.auto_scroll);
    assert!(!st.word_wrap);
}

#[test]
fn test_streaming_text_with_id() {
    let st = StreamingText::with_id(dracon_terminal_engine::framework::widget::WidgetId::new(4));
    assert_eq!(st.id, dracon_terminal_engine::framework::widget::WidgetId::new(4));
}

#[test]
fn test_streaming_text_max_lines() {
    let st = StreamingText::new().max_lines(50);
    assert_eq!(st.max_lines, 50);
}

#[test]
fn test_streaming_text_auto_scroll() {
    let st = StreamingText::new().auto_scroll(false);
    assert!(!st.auto_scroll);
}

#[test]
fn test_streaming_text_word_wrap() {
    let st = StreamingText::new().word_wrap(true);
    assert!(st.word_wrap);
}

#[test]
fn test_streaming_text_bind_command() {
    use dracon_terminal_engine::framework::command::BoundCommand;
    let cmd = BoundCommand::new("curl -N https://stream").label("stream");
    let st = StreamingText::new().bind_command(cmd);
    assert_eq!(st.commands().len(), 1);
}

#[test]
fn test_streaming_text_append() {
    let mut st = StreamingText::new();
    st.append("Hello\nWorld");
    assert_eq!(st.lines.len(), 2);
}

#[test]
fn test_streaming_text_append_max_lines() {
    let mut st = StreamingText::new().max_lines(3);
    for i in 0..5 {
        st.append(&format!("line {}\n", i));
    }
    assert_eq!(st.lines.len(), 3);
}

#[test]
fn test_streaming_text_append_output_text() {
    let mut st = StreamingText::new();
    st.append_output(ParsedOutput::Text("foo\nbar".to_string()));
    assert_eq!(st.lines.len(), 2);
}

#[test]
fn test_streaming_text_append_output_scalar() {
    let mut st = StreamingText::new();
    st.append_output(ParsedOutput::Scalar("single value".to_string()));
    assert_eq!(st.lines.len(), 1);
}

#[test]
fn test_streaming_text_clear() {
    let mut st = StreamingText::new();
    st.append("test");
    st.clear();
    assert!(st.lines.is_empty());
    assert!(st.content.is_empty());
}

#[test]
fn test_streaming_text_content() {
    let mut st = StreamingText::new();
    st.append("hello");
    assert_eq!(st.content(), "hello");
}

#[test]
fn test_streaming_text_render() {
    let mut st = StreamingText::new();
    st.append("Test line");
    let plane = st.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane.cells[0].char, 'T');
}

#[test]
fn test_streaming_text_render_empty() {
    let st = StreamingText::new();
    let plane = st.render(Rect::new(0, 0, 30, 5));
    assert!(plane.cells.iter().any(|c| c.char == '('));
}

#[test]
fn test_streaming_text_render_word_wrap() {
    let mut st = StreamingText::new().word_wrap(true);
    st.append("This is a very long line that should wrap");
    let plane = st.render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.cells[0].char, 'T');
}

#[test]
fn test_streaming_text_dirty_lifecycle() {
    let mut st = StreamingText::new();
    assert!(st.needs_render());
    st.clear_dirty();
    assert!(!st.needs_render());
    st.append("new content");
    assert!(st.needs_render());
}

#[test]
fn test_streaming_text_with_theme() {
    let theme = Theme::solarized_dark();
    let st = StreamingText::new().with_theme(theme);
    assert_eq!(st.theme.name, "solarized-dark");
}

#[test]
fn test_streaming_text_multiline_append() {
    let mut st = StreamingText::new();
    st.append("line1\nline2\nline3");
    assert_eq!(st.lines.len(), 3);
}

#[test]
fn test_streaming_text_auto_scroll_shows_latest() {
    let mut st = StreamingText::new().max_lines(3);
    for i in 0..5 {
        st.append(&format!("line {}\n", i));
    }
    assert_eq!(st.lines[0], "line 2");
    assert_eq!(st.lines[2], "line 4");
}

#[test]
fn test_streaming_text_apply_command_output_scalar() {
    let mut st = StreamingText::new();
    st.apply_command_output(&ParsedOutput::Scalar("hello".to_string()));
    assert_eq!(st.lines.len(), 1);
    assert_eq!(st.lines[0], "hello");
}

#[test]
fn test_streaming_text_apply_command_output_text() {
    let mut st = StreamingText::new();
    st.apply_command_output(&ParsedOutput::Text("line1\nline2".to_string()));
    assert_eq!(st.lines.len(), 2);
}