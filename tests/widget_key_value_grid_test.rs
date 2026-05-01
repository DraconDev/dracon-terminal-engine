mod common;

use dracon_terminal_engine::framework::command::ParsedOutput;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::key_value_grid::KeyValueGrid;
use ratatui::layout::Rect;

#[test]
fn test_key_value_grid_new() {
    let grid = KeyValueGrid::new();
    assert!(grid.pairs.is_empty());
    assert_eq!(grid.separator, "  ");
}

#[test]
fn test_key_value_grid_with_id() {
    let grid = KeyValueGrid::with_id(dracon_terminal_engine::framework::widget::WidgetId::new(3));
    assert_eq!(grid.id, dracon_terminal_engine::framework::widget::WidgetId::new(3));
}

#[test]
fn test_key_value_grid_separator() {
    let grid = KeyValueGrid::new().separator(" : ");
    assert_eq!(grid.separator, " : ");
}

#[test]
fn test_key_value_grid_bind_command() {
    use dracon_terminal_engine::framework::command::BoundCommand;
    let cmd = BoundCommand::new("sysinfo").label("info");
    let grid = KeyValueGrid::new().bind_command(cmd);
    assert_eq!(grid.commands().len(), 1);
}

#[test]
fn test_key_value_grid_set_pairs() {
    use std::collections::BTreeMap;
    let mut grid = KeyValueGrid::new();
    let mut pairs = BTreeMap::new();
    pairs.insert("CPU".to_string(), "i9".to_string());
    pairs.insert("RAM".to_string(), "64GB".to_string());
    grid.set_pairs(pairs);
    assert_eq!(grid.pairs.len(), 2);
}

#[test]
fn test_key_value_grid_update_from_scalar() {
    let mut grid = KeyValueGrid::new();
    grid.update_from_output(ParsedOutput::Scalar("active".to_string()));
    assert_eq!(grid.pairs.get("value").unwrap(), "active");
}

#[test]
fn test_key_value_grid_update_from_text() {
    let mut grid = KeyValueGrid::new();
    grid.update_from_output(ParsedOutput::Text("CPU: i9\nRAM: 64GB".to_string()));
    assert_eq!(grid.pairs.get("CPU").unwrap(), "i9");
    assert_eq!(grid.pairs.get("RAM").unwrap(), "64GB");
}

#[test]
fn test_key_value_grid_update_from_text_with_colon_in_value() {
    let mut grid = KeyValueGrid::new();
    grid.update_from_output(ParsedOutput::Text("path: /usr/local/bin:stuff".to_string()));
    assert_eq!(grid.pairs.get("path").unwrap(), "/usr/local/bin:stuff");
}

#[test]
fn test_key_value_grid_render() {
    use std::collections::BTreeMap;
    let mut grid = KeyValueGrid::new();
    let mut pairs = BTreeMap::new();
    pairs.insert("Name".to_string(), "Test".to_string());
    grid.set_pairs(pairs);
    let plane = grid.render(Rect::new(0, 0, 40, 5));
    assert_eq!(plane.cells[0].char, 'N');
}

#[test]
fn test_key_value_grid_render_empty() {
    let grid = KeyValueGrid::new();
    let plane = grid.render(Rect::new(0, 0, 30, 5));
    assert!(plane.cells.iter().any(|c| c.char == '('));
}

#[test]
fn test_key_value_grid_dirty_lifecycle() {
    let mut grid = KeyValueGrid::new();
    assert!(grid.needs_render());
    grid.clear_dirty();
    assert!(!grid.needs_render());
    grid.set_pairs(std::collections::BTreeMap::new());
    assert!(grid.needs_render());
}

#[test]
fn test_key_value_grid_with_theme() {
    let theme = Theme::gruvbox_dark();
    let grid = KeyValueGrid::new().with_theme(theme);
    assert_eq!(grid.theme.name, "gruvbox-dark");
}

#[test]
fn test_key_value_grid_sorted_keys() {
    use std::collections::BTreeMap;
    let mut grid = KeyValueGrid::new();
    let mut pairs = BTreeMap::new();
    pairs.insert("zebra".to_string(), "last".to_string());
    pairs.insert("apple".to_string(), "first".to_string());
    grid.set_pairs(pairs);
    let keys: Vec<&String> = grid.pairs.keys().collect();
    assert_eq!(keys[0], "apple");
    assert_eq!(keys[1], "zebra");
}

#[test]
fn test_key_value_grid_apply_command_output() {
    let mut grid = KeyValueGrid::new();
    grid.apply_command_output(&ParsedOutput::Text("CPU: i9\nRAM: 64GB".to_string()));
    assert_eq!(grid.pairs.get("CPU").unwrap(), "i9");
    assert_eq!(grid.pairs.get("RAM").unwrap(), "64GB");
}