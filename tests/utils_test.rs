//! Tests for utils module: visual_width, truncate, format_size, format_permissions,
//! SelectionState, FileCategory, get_file_category, and other utilities.

mod common;

use dracon_terminal_engine::utils::{
    delete_word_backwards, format_datetime_smart, format_permissions, format_size,
    get_file_category, get_open_with_suggestions, get_visual_width, is_binary_content,
    truncate_to_width, squarify, guess_icon_mode, FileCategory, IconMode, SelectionState,
};
use std::time::{Duration, SystemTime};
use ratatui::style::Color as RatatuiColor;

#[test]
fn test_get_visual_width_ascii() {
    assert_eq!(get_visual_width('a'), 1);
    assert_eq!(get_visual_width('Z'), 1);
    assert_eq!(get_visual_width(' '), 1);
}

#[test]
fn test_truncate_to_width_exact_fit() {
    let result = truncate_to_width("hello", 5, "...");
    assert_eq!(result, "hello");
}

#[test]
fn test_truncate_to_width_truncates_with_suffix() {
    let result = truncate_to_width("hello world", 8, "...");
    assert!(result.ends_with("..."));
    assert!(result.len() <= 8);
}

#[test]
fn test_truncate_to_width_no_truncation_needed() {
    let result = truncate_to_width("hi", 10, "...");
    assert_eq!(result, "hi");
}

#[test]
fn test_truncate_to_width_suffix_too_long() {
    let result = truncate_to_width("hello", 2, "...");
    assert_eq!(result, ".");
}

#[test]
fn test_truncate_to_width_empty_string() {
    let result = truncate_to_width("", 5, "...");
    assert_eq!(result, "");
}

#[test]
fn test_truncate_to_width_suffix_width_calculation() {
    let result = truncate_to_width("hello world", 8, "...");
    assert!(result.ends_with("..."));
    assert!(result.len() <= 8);
}

#[test]
fn test_truncate_to_width_with_unicode() {
    let result = truncate_to_width("日本語テスト", 10, "...");
    assert!(result.ends_with("...") || result.chars().count() <= 5);
}

#[test]
fn test_format_size_bytes() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(512), "512 B");
    assert_eq!(format_size(1023), "1023 B");
}

#[test]
fn test_format_size_kilobytes() {
    assert_eq!(format_size(1024), "1.0 KB");
    assert_eq!(format_size(2048), "2.0 KB");
    assert_eq!(format_size(1536), "1.5 KB");
}

#[test]
fn test_format_size_megabytes() {
    assert_eq!(format_size(1048576), "1.0 MB");
    assert_eq!(format_size(5242880), "5.0 MB");
}

#[test]
fn test_format_size_gigabytes() {
    assert_eq!(format_size(1073741824), "1.0 GB");
    assert_eq!(format_size(2147483648), "2.0 GB");
}

#[test]
fn test_format_permissions_rwx() {
    assert_eq!(format_permissions(0o755), "rwxr-xr-x");
    assert_eq!(format_permissions(0o644), "rw-r--r--");
    assert_eq!(format_permissions(0o700), "rwx------");
    assert_eq!(format_permissions(0o000), "---------");
    assert_eq!(format_permissions(0o777), "rwxrwxrwx");
}

#[test]
fn test_format_permissions_special_bits() {
    assert_eq!(format_permissions(0o4755), "rwxr-xr-x");
    assert_eq!(format_permissions(0o2755), "rwxr-xr-x");
    assert_eq!(format_permissions(0o1755), "rwxr-xr-x");
}

#[test]
fn test_is_binary_content_text() {
    let bytes = b"Hello, world!";
    assert!(!is_binary_content(bytes));
}

#[test]
fn test_is_binary_content_with_null() {
    let bytes = b"Hello\x00world";
    assert!(is_binary_content(bytes));
}

#[test]
fn test_is_binary_content_empty() {
    let bytes: &[u8] = &[];
    assert!(!is_binary_content(bytes));
}

#[test]
fn test_delete_word_backwards_basic() {
    let mut s = String::from("hello world");
    delete_word_backwards(&mut s);
    assert_eq!(s, "hello ");
}

#[test]
fn test_delete_word_backwards_multiple_words() {
    let mut s = String::from("hello world foo");
    delete_word_backwards(&mut s);
    assert_eq!(s, "hello world ");
}

#[test]
fn test_delete_word_backwards_empty() {
    let mut s = String::from("");
    delete_word_backwards(&mut s);
    assert_eq!(s, "");
}

#[test]
fn test_delete_word_backwards_single_word() {
    let mut s = String::from("helloworld");
    delete_word_backwards(&mut s);
    assert_eq!(s, "");
}

#[test]
fn test_delete_word_backwards_with_spaces() {
    let mut s = String::from("hello world");
    delete_word_backwards(&mut s);
    assert_eq!(s, "hello ");
}

#[test]
fn test_selection_state_new() {
    let state = SelectionState::new();
    assert!(state.selected.is_none());
    assert!(state.anchor.is_none());
    assert!(state.multi.is_empty());
}

#[test]
fn test_selection_state_clear() {
    let mut state = SelectionState::new();
    state.add(1);
    state.add(2);
    state.clear();
    assert!(state.multi.is_empty());
    assert!(state.selected.is_none());
}

#[test]
fn test_selection_state_add() {
    let mut state = SelectionState::new();
    state.add(1);
    state.add(2);
    assert!(state.multi.contains(&1));
    assert!(state.multi.contains(&2));
}

#[test]
fn test_selection_state_toggle() {
    let mut state = SelectionState::new();
    state.add(1);
    state.toggle(1);
    assert!(!state.multi.contains(&1));
}

#[test]
fn test_selection_state_select_all() {
    let mut state = SelectionState::new();
    state.select_all(5);
    assert_eq!(state.multi.len(), 5);
}

#[test]
fn test_selection_state_handle_click_single() {
    let mut state = SelectionState::new();
    state.handle_click(2, false, false, false);
    assert_eq!(state.selected, Some(2));
    assert!(state.multi.contains(&2));
}

#[test]
fn test_selection_state_handle_click_shift() {
    let mut state = SelectionState::new();
    state.handle_click(1, false, false, false);
    state.handle_click(3, true, false, false);
    assert!(state.multi.contains(&1));
    assert!(state.multi.contains(&2));
    assert!(state.multi.contains(&3));
}

#[test]
fn test_selection_state_handle_click_ctrl() {
    let mut state = SelectionState::new();
    state.handle_click(1, false, false, false);
    state.handle_click(2, false, true, false);
    assert!(state.multi.contains(&1));
    assert!(state.multi.contains(&2));
}

#[test]
fn test_selection_state_handle_move() {
    let mut state = SelectionState::new();
    state.handle_click(0, false, false, false);
    state.handle_move(3, false);
    assert_eq!(state.selected, Some(3));
    assert!(state.multi.is_empty());
}

#[test]
fn test_selection_state_handle_move_shift() {
    let mut state = SelectionState::new();
    state.handle_click(1, false, false, false);
    state.handle_move(3, true);
    assert!(state.multi.contains(&1));
    assert!(state.multi.contains(&2));
    assert!(state.multi.contains(&3));
}

#[test]
fn test_get_file_category_by_extension() {
    let path = std::path::Path::new("test.rs");
    assert_eq!(get_file_category(path), FileCategory::Text);

    let path = std::path::Path::new("image.png");
    assert_eq!(get_file_category(path), FileCategory::Image);

    let path = std::path::Path::new("archive.zip");
    assert_eq!(get_file_category(path), FileCategory::Archive);
}

#[test]
fn test_get_file_category_script() {
    let path = std::path::Path::new("script.sh");
    assert_eq!(get_file_category(path), FileCategory::Script);

    let path = std::path::Path::new("script.py");
    assert_eq!(get_file_category(path), FileCategory::Script);
}

#[test]
fn test_get_file_category_document() {
    let path = std::path::Path::new("doc.pdf");
    assert_eq!(get_file_category(path), FileCategory::Document);

    let path = std::path::Path::new("spreadsheet.xlsx");
    assert_eq!(get_file_category(path), FileCategory::Document);
}

#[test]
fn test_get_file_category_audio_video() {
    let path = std::path::Path::new("song.mp3");
    assert_eq!(get_file_category(path), FileCategory::Audio);

    let path = std::path::Path::new("video.mp4");
    assert_eq!(get_file_category(path), FileCategory::Video);
}

#[test]
fn test_get_file_category_special_names() {
    let path = std::path::Path::new("LICENSE");
    assert_eq!(get_file_category(path), FileCategory::Text);

    let path = std::path::Path::new("Dockerfile");
    assert_eq!(get_file_category(path), FileCategory::Text);

    let path = std::path::Path::new("Makefile");
    assert_eq!(get_file_category(path), FileCategory::Text);

    let path = std::path::Path::new(".gitignore");
    assert_eq!(get_file_category(path), FileCategory::Text);
}

#[test]
fn test_get_file_category_unknown_extension() {
    let path = std::path::Path::new("file.xyzabc");
    assert_eq!(get_file_category(path), FileCategory::Other);
}

#[test]
fn test_get_open_with_suggestions_text_file() {
    let suggestions = get_open_with_suggestions("rs");
    assert!(!suggestions.is_empty());
    assert!(suggestions.contains(&"code".to_string()));
}

#[test]
fn test_get_open_with_suggestions_image() {
    let suggestions = get_open_with_suggestions("png");
    assert!(!suggestions.is_empty());
}

#[test]
fn test_get_open_with_suggestions_unknown() {
    let suggestions = get_open_with_suggestions("xyz");
    assert!(!suggestions.is_empty());
    assert!(suggestions.contains(&"xdg-open".to_string()));
}

#[test]
fn test_squarify_preserves_displayable() {
    let input = "Hello World!";
    let result = squarify(input);
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_squarify_removes_control_chars() {
    let input = "Hello\x00World\n\x07Test";
    let result = squarify(input);
    assert_eq!(result, "HelloWorldTest");
}

#[test]
fn test_format_datetime_smart_today() {
    let now = SystemTime::now();
    let result = format_datetime_smart(now);
    assert!(result.contains(':'));
}

#[test]
fn test_format_datetime_smart_past_date() {
    let older = SystemTime::now() - Duration::from_secs(86400 * 30);
    let result = format_datetime_smart(older);
    assert!(result.contains('-'));
}

#[test]
fn test_guess_icon_mode_returns_valid_enum() {
    std::env::remove_var("TERM");
    std::env::remove_var("TERM_PROGRAM");
    std::env::remove_var("COLORTERM");
    let mode = guess_icon_mode();
    match mode {
        IconMode::Nerd | IconMode::Unicode | IconMode::ASCII => {}
    }
}

#[test]
fn test_file_category_cyber_color() {
    let categories = [
        (FileCategory::Archive, 255, 50, 80),
        (FileCategory::Image, 255, 0, 255),
        (FileCategory::Script, 0, 255, 100),
        (FileCategory::Text, 255, 215, 0),
        (FileCategory::Document, 100, 200, 255),
        (FileCategory::Audio, 0, 150, 255),
        (FileCategory::Video, 180, 50, 255),
        (FileCategory::Other, 255, 255, 255),
    ];
    for (cat, r, g, b) in categories {
        let color = cat.cyber_color();
        assert_eq!(color, RatatuiColor::Rgb(r, g, b));
    }
}

#[test]
fn test_selection_state_is_empty() {
    let mut state = SelectionState::new();
    assert!(state.is_empty());
    state.add(1);
    assert!(!state.is_empty());
}

#[test]
fn test_selection_state_clear_multi() {
    let mut state = SelectionState::new();
    state.handle_click(1, false, false, false);
    state.add(2);
    state.clear_multi();
    assert!(state.multi.is_empty());
    assert!(state.selected.is_some());
}

#[test]
fn test_selection_state_multi_selected_indices() {
    let mut state = SelectionState::new();
    state.add(1);
    state.add(2);
    let indices = state.multi_selected_indices();
    assert!(indices.contains(&1));
    assert!(indices.contains(&2));
}