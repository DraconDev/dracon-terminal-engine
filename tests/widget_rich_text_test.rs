//! Tests for the RichText widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::rich_text::RichText;
use ratatui::layout::Rect;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_rich_text_new() {
    let rt = RichText::new();
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_with_content() {
    let rt = RichText::new().with_content("# Hello World\n\nThis is plain text.");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_with_empty_content() {
    let rt = RichText::new().with_content("");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_rich_text_with_theme() {
    let rt = RichText::new().with_theme(Theme::nord());
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert_eq!(plane.width, 80);
}

#[test]
fn test_rich_text_chained_builders() {
    let rt = RichText::new()
        .with_theme(Theme::cyberpunk())
        .with_content("# Title\n\nParagraph.");
    let area = Rect::new(0, 0, 80, 24);
    let _plane = rt.render(area);
}

// ============================================================================
// Markdown Parsing Tests - Headers
// ============================================================================

#[test]
fn test_rich_text_header_h1() {
    let rt = RichText::new().with_content("# Header 1");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    
    // Header should render with different styling
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_header_h2() {
    let rt = RichText::new().with_content("## Header 2");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_header_h3() {
    let rt = RichText::new().with_content("### Header 3");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_header_h4_h6() {
    let rt = RichText::new().with_content("#### Header 4\n##### Header 5\n###### Header 6");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Markdown Parsing Tests - Formatting
// ============================================================================

#[test]
fn test_rich_text_bold() {
    let rt = RichText::new().with_content("This is **bold** text.");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_italic() {
    let rt = RichText::new().with_content("This is *italic* text.");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_code() {
    let rt = RichText::new().with_content("Use `printf` for output.");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_code_multiline() {
    let rt = RichText::new().with_content("```\ncode block\n```");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_link() {
    let rt = RichText::new().with_content("Click [here](https://example.com) to visit.");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Markdown Parsing Tests - Lists
// ============================================================================

#[test]
fn test_rich_text_list_dash() {
    let rt = RichText::new().with_content("- Item 1\n- Item 2\n- Item 3");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_list_asterisk() {
    let rt = RichText::new().with_content("* First\n* Second\n* Third");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_nested_list() {
    let rt = RichText::new().with_content("- Item 1\n  - Nested\n  - Item 2");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Markdown Parsing Tests - Paragraphs
// ============================================================================

#[test]
fn test_rich_text_paragraph() {
    let rt = RichText::new().with_content("This is a paragraph with multiple words.");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_multiple_paragraphs() {
    let rt = RichText::new().with_content("First paragraph.\n\nSecond paragraph.");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_empty_lines() {
    let rt = RichText::new().with_content("Line 1\n\n\n\nLine 2");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_rich_text_id() {
    let rt = RichText::new();
    let _id = rt.id();
}

#[test]
fn test_rich_text_area() {
    let rt = RichText::new();
    let area = rt.area();
    assert!(area.width > 0);
    assert!(area.height > 0);
}

#[test]
fn test_rich_text_set_area() {
    let mut rt = RichText::new();
    let new_area = Rect::new(10, 20, 100, 40);
    rt.set_area(new_area);
    assert_eq!(rt.area(), new_area);
}

#[test]
fn test_rich_text_needs_render() {
    let rt = RichText::new();
    assert!(rt.needs_render());
}

#[test]
fn test_rich_text_mark_dirty() {
    let mut rt = RichText::new();
    rt.clear_dirty();
    assert!(!rt.needs_render());
    rt.mark_dirty();
    assert!(rt.needs_render());
}

#[test]
fn test_rich_text_clear_dirty() {
    let mut rt = RichText::new();
    rt.clear_dirty();
    assert!(!rt.needs_render());
}

#[test]
fn test_rich_text_render() {
    let rt = RichText::new();
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

#[test]
fn test_rich_text_render_different_sizes() {
    let rt = RichText::new().with_content("# Test");
    
    // Small area
    let plane1 = rt.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane1.width, 40);
    
    // Large area
    let plane2 = rt.render(Rect::new(0, 0, 120, 50));
    assert_eq!(plane2.width, 120);
}

#[test]
fn test_rich_text_z_index() {
    let rt = RichText::new();
    assert_eq!(rt.z_index(), 10);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_rich_text_different_themes() {
    let content = "# Title\n\nParagraph with **bold** and *italic*.";
    
    for theme_name in ["nord", "dracula", "monokai", "solarized_dark"] {
        if let Some(theme) = Theme::from_name(theme_name) {
            let rt = RichText::new()
                .with_content(content)
                .with_theme(theme);
            let area = Rect::new(0, 0, 80, 24);
            let plane = rt.render(area);
            assert_eq!(plane.width, 80);
        }
    }
}

// ============================================================================
// Word Wrapping Tests
// ============================================================================

#[test]
fn test_rich_text_word_wrap_long_line() {
    let rt = RichText::new().with_content("This is a very long line that should wrap at the boundary of the render area.");
    let area = Rect::new(0, 0, 40, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_word_wrap_narrow() {
    let rt = RichText::new().with_content("The quick brown fox jumps over the lazy dog.");
    let area = Rect::new(0, 0, 20, 24);
    let plane = rt.render(area);
    assert_eq!(plane.width, 20);
}

// ============================================================================
// Unicode Tests
// ============================================================================

#[test]
fn test_rich_text_unicode_text() {
    let rt = RichText::new().with_content("日本語テキスト\nالعربية\nעברית");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_emoji() {
    let rt = RichText::new().with_content("Hello 🎉🎊 World 🌍");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_unicode_in_markdown() {
    let rt = RichText::new().with_content("# 日本語\n\nThis is **bold** 日本語.");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Complex Content Tests
// ============================================================================

#[test]
fn test_rich_text_complex_document() {
    let doc = r#"# Document Title

This is a paragraph with **bold**, *italic*, and `code`.

## Section 1

- List item 1
- List item 2
- List item 3

### Subsection

Another paragraph here.

## Section 2

[Link text](https://example.com) is supported.

Final paragraph.
"#;
    let rt = RichText::new().with_content(doc);
    let area = Rect::new(0, 0, 80, 50);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_markdown_edge_cases() {
    let tests = vec![
        "***triple***",
        "**nested *italic* in bold**",
        "*italic **bold** inside*",
        "```code with **bold** inside```",
        "[link with **bold**](http://example.com)",
    ];
    
    for content in tests {
        let rt = RichText::new().with_content(content);
        let area = Rect::new(0, 0, 80, 24);
        let plane = rt.render(area);
        assert!(plane.width > 0, "Failed for: {}", content);
    }
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_rich_text_render_fills_bg() {
    let rt = RichText::new().with_content("Test content");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_rich_text_render_has_content() {
    let rt = RichText::new().with_content("Hello World");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    let has_content = plane.cells.iter().any(|c| c.char != '\0' && c.char != ' ');
    assert!(has_content, "RichText should render some content");
}

#[test]
fn test_rich_text_render_minimal_area() {
    let rt = RichText::new().with_content("Test");
    let area = Rect::new(0, 0, 5, 3);
    let plane = rt.render(area);
    assert_eq!(plane.width, 5);
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_rich_text_only_headers() {
    let rt = RichText::new().with_content("# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_only_list() {
    let rt = RichText::new().with_content("- Item 1\n- Item 2\n- Item 3\n- Item 4");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_special_characters() {
    let rt = RichText::new().with_content("Special: & < > \" ' # @ $ % ^ & * ( )");
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_rich_text_very_long_content() {
    let long_content = "word ".repeat(1000);
    let rt = RichText::new().with_content(&long_content);
    let area = Rect::new(0, 0, 80, 24);
    let plane = rt.render(area);
    assert!(plane.width > 0);
}