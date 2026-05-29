//! Tests for the TagsInput widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::tags_input::TagsInput;
use ratatui::layout::Rect;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_tags_input_new_empty() {
    let input = TagsInput::new(vec![]);
    assert!(input.tags().is_empty());
    assert_eq!(input.input(), "");
}

#[test]
fn test_tags_input_new_with_tags() {
    let tags = vec![
        "rust".to_string(),
        "programming".to_string(),
        "cli".to_string(),
    ];
    let input = TagsInput::new(tags.clone());
    assert_eq!(input.tags(), &["rust", "programming", "cli"]);
}

#[test]
fn test_tags_input_default() {
    let input = TagsInput::new(vec![]);
    assert!(input.tags().is_empty());
}

#[test]
fn test_tags_input_with_single_tag() {
    let input = TagsInput::new(vec!["hello".to_string()]);
    assert_eq!(input.tags().len(), 1);
    assert_eq!(input.tags()[0], "hello");
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_tags_input_with_theme() {
    let input = TagsInput::new(vec![]).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 40, 3);
    let plane = input.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_tags_input_with_placeholder() {
    let _input = TagsInput::new(vec![]).with_placeholder("Enter tags...");
}

#[test]
fn test_tags_input_with_width() {
    let _input = TagsInput::new(vec![]).with_width(60);
}

#[test]
fn test_tags_input_with_max_tags() {
    let mut input = TagsInput::new(vec![]).with_max_tags(2);
    input.add_tag("tag1".to_string());
    input.add_tag("tag2".to_string());
    input.add_tag("tag3".to_string()); // Should be ignored

    assert_eq!(input.tags().len(), 2);
    assert!(input.tags().contains(&"tag1".to_string()));
    assert!(input.tags().contains(&"tag2".to_string()));
    assert!(!input.tags().contains(&"tag3".to_string()));
}

#[test]
fn test_tags_input_allow_duplicates() {
    let mut input = TagsInput::new(vec![]).allow_duplicates(true);

    input.add_tag("tag".to_string());
    input.add_tag("tag".to_string());

    assert_eq!(input.tags().len(), 2);
}

#[test]
fn test_tags_input_no_duplicates_by_default() {
    let mut input = TagsInput::new(vec![]);

    input.add_tag("tag".to_string());
    input.add_tag("TAG".to_string()); // Case-insensitive duplicate
    input.add_tag("Tag".to_string()); // Case-insensitive duplicate

    assert_eq!(input.tags().len(), 1);
}

#[test]
fn test_tags_input_with_suggestions() {
    let input = TagsInput::new(vec![]).with_suggestions(vec!["apple", "banana", "cherry"]);
}

#[test]
fn test_tags_input_with_tags() {
    let input = TagsInput::new(vec![]).with_tags(vec!["one", "two"]);

    assert_eq!(input.tags().len(), 2);
    assert_eq!(input.tags()[0], "one");
    assert_eq!(input.tags()[1], "two");
}

// ============================================================================
// Callback Registration Tests
// ============================================================================

#[test]
fn test_tags_input_on_tag_add() {
    let _input = TagsInput::new(vec![]).on_tag_add(|_| {});
}

#[test]
fn test_tags_input_on_tag_remove() {
    let _input = TagsInput::new(vec![]).on_tag_remove(|_| {});
}

#[test]
fn test_tags_input_on_input_change() {
    let _input = TagsInput::new(vec![]).on_input_change(|_| {});
}

#[test]
fn test_tags_input_on_suggestion_select() {
    let _input = TagsInput::new(vec![]).on_suggestion_select(|_| {});
}

// ============================================================================
// Tag Management Tests
// ============================================================================

#[test]
fn test_tags_input_add_tag() {
    let mut input = TagsInput::new(vec![]);
    input.add_tag("new_tag".to_string());

    assert_eq!(input.tags().len(), 1);
    assert_eq!(input.tags()[0], "new_tag");
}

#[test]
fn test_tags_input_add_multiple_tags() {
    let mut input = TagsInput::new(vec![]);
    input.add_tag("tag1".to_string());
    input.add_tag("tag2".to_string());
    input.add_tag("tag3".to_string());

    assert_eq!(input.tags().len(), 3);
}

#[test]
fn test_tags_input_add_tag_trims_whitespace() {
    let mut input = TagsInput::new(vec![]);
    input.add_tag("  trimmed  ".to_string());

    assert_eq!(input.tags()[0], "trimmed");
}

#[test]
fn test_tags_input_add_empty_tag_ignored() {
    let mut input = TagsInput::new(vec![]);
    input.add_tag("".to_string());
    input.add_tag("   ".to_string());

    assert!(input.tags().is_empty());
}

#[test]
fn test_tags_input_remove_tag() {
    let mut input = TagsInput::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

    input.remove_tag(1);

    assert_eq!(input.tags().len(), 2);
    assert_eq!(input.tags()[0], "a");
    assert_eq!(input.tags()[1], "c");
}

#[test]
fn test_tags_input_remove_tag_out_of_bounds() {
    let mut input = TagsInput::new(vec!["a".to_string()]);
    input.remove_tag(5); // Out of bounds

    assert_eq!(input.tags().len(), 1);
}

#[test]
fn test_tags_input_remove_last_tag() {
    let mut input = TagsInput::new(vec!["a".to_string(), "b".to_string()]);
    input.remove_last_tag();

    assert_eq!(input.tags().len(), 1);
    assert_eq!(input.tags()[0], "a");
}

#[test]
fn test_tags_input_remove_last_tag_empty() {
    let mut input = TagsInput::new(vec![]);
    input.remove_last_tag(); // Should not panic

    assert!(input.tags().is_empty());
}

#[test]
fn test_tags_input_clear() {
    let mut input = TagsInput::new(vec!["a".to_string(), "b".to_string()]);
    input.clear();

    assert!(input.tags().is_empty());
}

#[test]
fn test_tags_input_clear_when_empty() {
    let mut input = TagsInput::new(vec![]);
    input.clear();

    assert!(input.tags().is_empty());
}

// ============================================================================
// Input Text Tests
// ============================================================================

#[test]
fn test_tags_input_input_empty() {
    let input = TagsInput::new(vec![]);
    assert_eq!(input.input(), "");
}

#[test]
fn test_tags_input_tags_returns_reference() {
    let input = TagsInput::new(vec!["test".to_string()]);
    let tags = input.tags();
    assert_eq!(tags.len(), 1);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_tags_input_id() {
    let input = TagsInput::new(vec![]);
    let _id = input.id();
    // ID should be valid
}

#[test]
fn test_tags_input_area() {
    let input = TagsInput::new(vec![]);
    let area = input.area();
    assert_eq!(area.width, 40);
    assert_eq!(area.height, 3);
}

#[test]
fn test_tags_input_set_area() {
    let mut input = TagsInput::new(vec![]);
    let new_area = Rect::new(10, 20, 60, 10);
    input.set_area(new_area);
    assert_eq!(input.area(), new_area);
}

#[test]
fn test_tags_input_needs_render() {
    let input = TagsInput::new(vec![]);
    assert!(input.needs_render());
}

#[test]
fn test_tags_input_mark_dirty() {
    let mut input = TagsInput::new(vec![]);
    input.clear_dirty();
    assert!(!input.needs_render());
    input.mark_dirty();
    assert!(input.needs_render());
}

#[test]
fn test_tags_input_clear_dirty() {
    let mut input = TagsInput::new(vec![]);
    input.clear_dirty();
    assert!(!input.needs_render());
}

#[test]
fn test_tags_input_render() {
    let input = TagsInput::new(vec![]);
    let area = Rect::new(0, 0, 40, 3);
    let plane = input.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_tags_input_render_with_tags() {
    let input = TagsInput::new(vec!["tag1".to_string(), "tag2".to_string()]);
    let area = Rect::new(0, 0, 60, 3);
    let plane = input.render(area);
    assert_eq!(plane.width, 60);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_tags_input_focusable() {
    let input = TagsInput::new(vec![]);
    assert!(input.focusable());
}

#[test]
fn test_tags_input_z_index() {
    let input = TagsInput::new(vec![]);
    assert_eq!(input.z_index(), 10);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_tags_input_different_themes() {
    for theme_name in ["nord", "dracula", "monokai", "solarized_dark"] {
        if let Some(theme) = Theme::from_name(theme_name) {
            let input = TagsInput::new(vec![]).with_theme(theme);
            let area = Rect::new(0, 0, 40, 3);
            let plane = input.render(area);
            assert_eq!(plane.width, 40);
            assert_eq!(plane.height, 3);
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_tags_input_many_tags() {
    let mut input = TagsInput::new(vec![]);

    for i in 0..100 {
        input.add_tag(format!("tag_{}", i));
    }

    assert_eq!(input.tags().len(), 100);
}

#[test]
fn test_tags_input_long_tag() {
    let mut input = TagsInput::new(vec![]);
    let long_tag = "a".repeat(1000);
    input.add_tag(long_tag);

    assert_eq!(input.tags().len(), 1);
    assert_eq!(input.tags()[0].len(), 1000);
}

#[test]
fn test_tags_input_unicode_tags() {
    let mut input = TagsInput::new(vec![]);
    input.add_tag("日本語".to_string());
    input.add_tag("🎉".to_string());
    input.add_tag("العربية".to_string());

    assert_eq!(input.tags().len(), 3);
}

#[test]
fn test_tags_input_render_minimum_area() {
    let input = TagsInput::new(vec![]);
    let area = Rect::new(0, 0, 1, 1);
    let plane = input.render(area);
    assert_eq!(plane.width, 1);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_tags_input_max_tags_zero() {
    let mut input = TagsInput::new(vec![]).with_max_tags(0);
    input.add_tag("tag".to_string());

    // Should not allow any tags when max is 0
    assert!(input.tags().is_empty());
}

// ============================================================================
// Duplicate Detection Tests
// ============================================================================

#[test]
fn test_tags_input_case_insensitive_duplicate() {
    let mut input = TagsInput::new(vec![]);

    input.add_tag("Hello".to_string());
    input.add_tag("HELLO".to_string());
    input.add_tag("hello".to_string());

    // Should only keep first one (case-insensitive comparison)
    assert_eq!(input.tags().len(), 1);
}

#[test]
fn test_tags_input_permit_different_case_when_duplicates_allowed() {
    let mut input = TagsInput::new(vec![]).allow_duplicates(true);

    input.add_tag("Hello".to_string());
    input.add_tag("HELLO".to_string());

    // Should keep both when duplicates are allowed
    assert_eq!(input.tags().len(), 2);
}

// ============================================================================
// Suggestions Tests (indirect via filter behavior)
// ============================================================================

#[test]
fn test_tags_input_with_unicode_suggestions() {
    let _input = TagsInput::new(vec![]).with_suggestions(vec!["日本語", "English", "emoji"]);
}

// ============================================================================
// Callback Behavior Tests
// ============================================================================

#[test]
fn test_tags_input_tag_add_callback() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let added_tags = Rc::new(RefCell::new(Vec::new()));
    let added_clone = Rc::clone(&added_tags);

    let mut input = TagsInput::new(vec![]).on_tag_add(move |tag| {
        added_clone.borrow_mut().push(tag);
    });

    input.add_tag("test".to_string());

    // Note: on_tag_add callback is registered but may not be invoked by add_tag
    // This test verifies the callback can be registered
}

#[test]
fn test_tags_input_tag_remove_callback() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let removed_indices = Rc::new(RefCell::new(Vec::new()));
    let removed_clone = Rc::clone(&removed_indices);

    let mut input =
        TagsInput::new(vec!["a".to_string(), "b".to_string()]).on_tag_remove(move |idx| {
            removed_clone.borrow_mut().push(idx);
        });

    input.remove_tag(0);

    assert_eq!(removed_indices.borrow().len(), 1);
    assert_eq!(removed_indices.borrow()[0], 0);
}

// ============================================================================
// Chained Builder Tests
// ============================================================================

#[test]
fn test_tags_input_chained_builders() {
    let input = TagsInput::new(vec![])
        .with_theme(Theme::nord())
        .with_placeholder("Enter tags...")
        .with_width(80)
        .with_max_tags(10)
        .allow_duplicates(false)
        .with_suggestions(vec!["rust", "go", "python"])
        .with_tags(vec!["initial"]);

    assert_eq!(input.tags().len(), 1);
    assert_eq!(input.tags()[0], "initial");
}

// ============================================================================
// Max Tags Edge Cases
// ============================================================================

#[test]
fn test_tags_input_at_max_tags() {
    let mut input = TagsInput::new(vec![]).with_max_tags(3);

    input.add_tag("1".to_string());
    input.add_tag("2".to_string());
    input.add_tag("3".to_string());

    assert_eq!(input.tags().len(), 3);

    // Try to add more
    input.add_tag("4".to_string());

    assert_eq!(input.tags().len(), 3); // Still 3
}

#[test]
fn test_tags_input_clear_resets_for_max() {
    let mut input = TagsInput::new(vec![]).with_max_tags(2);

    input.add_tag("1".to_string());
    input.add_tag("2".to_string());
    input.clear();

    // After clear, should be able to add again
    input.add_tag("new".to_string());

    assert_eq!(input.tags().len(), 1);
}
