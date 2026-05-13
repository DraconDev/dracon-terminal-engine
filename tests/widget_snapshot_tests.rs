//! Widget snapshot tests using the `insta` crate.
//!
//! These tests render widgets and compare their output to stored snapshots.
//! Run with `cargo test` to verify widgets render correctly.
//! Run with `INSTA_UPDATE=1 cargo test` to update all snapshots.
//!
//! ## Adding New Snapshots
//!
//! 1. Create a new test function for the widget
//! 2. Render the widget to a Plane
//! 3. Serialize the plane data using insta::assert_debug_snapshot!
//!
//! ## Example
//!
//! ```ignore
//! #[test]
//! fn test_my_widget_snapshot() {
//!     let widget = MyWidget::new();
//!     let plane = widget.render(Rect::new(0, 0, 40, 10));
//!     assert_debug_snapshot!("my_widget", plane);
//! }
//! ```

mod common;

use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widgets::{
    Form, FormField, FormFieldType, Label, List, Radio, Table, TableRow, Tree, TreeNode,
};
use dracon_terminal_engine::framework::widget::Widget;
use ratatui::layout::Rect;

// Helper function to convert Plane to a serializable representation
#[derive(Debug, serde::Serialize)]
struct PlaneSnapshot<'a> {
    width: u16,
    height: u16,
    cells: Vec<CellSnapshot<'a>>,
}

#[derive(Debug, serde::Serialize)]
struct CellSnapshot<'a> {
    char: &'a str,
    fg: u8,
    bg: u8,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    inverse: bool,
    transparent: bool,
}

fn plane_to_snapshot<'a>(plane: &'a Plane) -> PlaneSnapshot<'a> {
    let cells = plane
        .cells
        .iter()
        .map(|c| CellSnapshot {
            char: if c.char.is_empty() || c.char.is_ascii_control() {
                " "
            } else {
                &c.char
            },
            fg: c.fg.0,
            bg: c.bg.0,
            bold: c.style.contains(Styles::BOLD),
            italic: c.style.contains(Styles::ITALIC),
            underline: c.style.contains(Styles::UNDERLINE),
            strikethrough: c.style.contains(Styles::STRIKETHROUGH),
            inverse: c.style.contains(Styles::INVERSE),
            transparent: c.transparent,
        })
        .collect();

    PlaneSnapshot {
        width: plane.width,
        height: plane.height,
        cells,
    }
}

// ── List Widget Snapshots ────────────────────────────────────────────────────

#[test]
fn test_list_snapshot_basic() {
    let items = vec!["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"];
    let list = List::new(items);
    let area = Rect::new(0, 0, 30, 10);
    let plane = list.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("list_basic", snapshot);
}

#[test]
fn test_list_snapshot_selected() {
    let items = vec!["Apple", "Banana", "Cherry", "Date"];
    let mut list = List::new(items);
    list.select(1); // Select "Banana"
    let area = Rect::new(0, 0, 30, 10);
    let plane = list.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("list_selected", snapshot);
}

#[test]
fn test_list_snapshot_empty() {
    let items: Vec<&str> = vec![];
    let list = List::new(items);
    let area = Rect::new(0, 0, 30, 10);
    let plane = list.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("list_empty", snapshot);
}

#[test]
fn test_list_snapshot_custom_theme() {
    let items = vec!["Red", "Green", "Blue"];
    let list = List::new(items).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 30, 10);
    let plane = list.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("list_cyberpunk", snapshot);
}

// ── Table Widget Snapshots ───────────────────────────────────────────────────

#[test]
fn test_table_snapshot_basic() {
    let headers = vec!["Name", "Age", "City"];
    let data = vec![
        vec!["Alice", "30", "NYC".to_string()],
        vec!["Bob", "25", "LA".to_string()],
        vec!["Charlie", "35", "CHI".to_string()],
    ];
    let rows: Vec<TableRow> = data
        .into_iter()
        .map(|cols| TableRow::new(cols.iter().map(|s| s.as_str()).collect()))
        .collect();
    let table = Table::new(headers, rows);
    let area = Rect::new(0, 0, 40, 8);
    let plane = table.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("table_basic", snapshot);
}

#[test]
fn test_table_snapshot_with_sort() {
    let headers = vec!["Name", "Value"];
    let data = vec![
        vec!["Alpha", "100".to_string()],
        vec!["Beta", "200".to_string()],
        vec!["Gamma", "50".to_string()],
    ];
    let rows: Vec<TableRow> = data
        .into_iter()
        .map(|cols| TableRow::new(cols.iter().map(|s| s.as_str()).collect()))
        .collect();
    let mut table = Table::new(headers, rows);
    table.set_sort(1, false); // Sort by Value, descending
    let area = Rect::new(0, 0, 30, 8);
    let plane = table.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("table_sorted", snapshot);
}

#[test]
fn test_table_snapshot_empty() {
    let headers = vec!["Col1", "Col2"];
    let rows: Vec<TableRow> = vec![];
    let table = Table::new(headers, rows);
    let area = Rect::new(0, 0, 30, 5);
    let plane = table.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("table_empty", snapshot);
}

// ── Tree Widget Snapshots ────────────────────────────────────────────────────

#[test]
fn test_tree_snapshot_basic() {
    let nodes = vec![
        TreeNode::new("Root", vec![
            TreeNode::new("Child 1", vec![]),
            TreeNode::new("Child 2", vec![]),
        ]),
        TreeNode::new("Another Root", vec![]),
    ];
    let tree = Tree::new(WidgetId::default_id()).with_root(nodes);
    let area = Rect::new(0, 0, 30, 10);
    let plane = tree.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("tree_basic", snapshot);
}

#[test]
fn test_tree_snapshot_expanded() {
    let mut nodes = vec![
        TreeNode::new("Documents", vec![
            TreeNode::new("Work", vec![]),
            TreeNode::new("Personal", vec![]),
        ]),
    ];
    let mut tree = Tree::new(WidgetId::default_id()).with_root(nodes);
    tree.expand(0); // Expand first node
    let area = Rect::new(0, 0, 30, 10);
    let plane = tree.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("tree_expanded", snapshot);
}

#[test]
fn test_tree_snapshot_deep() {
    let nodes = vec![
        TreeNode::new("Level 1", vec![
            TreeNode::new("Level 2", vec![
                TreeNode::new("Level 3", vec![]),
            ]),
        ]),
    ];
    let tree = Tree::new(WidgetId::default_id()).with_root(nodes);
    let area = Rect::new(0, 0, 40, 10);
    let plane = tree.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("tree_deep", snapshot);
}

// ── Form Widget Snapshots ───────────────────────────────────────────────────

#[test]
fn test_form_snapshot_basic() {
    let fields = vec![
        FormField::new("username", "Username", FormFieldType::Text),
        FormField::new("email", "Email", FormFieldType::Email),
        FormField::new("age", "Age", FormFieldType::Number),
    ];
    let form = Form::new(fields);
    let area = Rect::new(0, 0, 40, 10);
    let plane = form.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("form_basic", snapshot);
}

#[test]
fn test_form_snapshot_with_values() {
    let fields = vec![
        FormField::new("name", "Name", FormFieldType::Text),
        FormField::new("password", "Password", FormFieldType::Password),
    ];
    let form = Form::new(fields);
    let area = Rect::new(0, 0, 40, 8);
    let plane = form.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("form_password", snapshot);
}

#[test]
fn test_form_snapshot_empty() {
    let fields: Vec<FormField> = vec![];
    let form = Form::new(fields);
    let area = Rect::new(0, 0, 30, 5);
    let plane = form.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("form_empty", snapshot);
}

// ── Label Widget Snapshots ──────────────────────────────────────────────────

#[test]
fn test_label_snapshot_basic() {
    let label = Label::new("Hello, World!");
    let area = Rect::new(0, 0, 20, 1);
    let plane = label.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("label_basic", snapshot);
}

#[test]
fn test_label_snapshot_long_text() {
    let label = Label::new("This is a very long label that should be truncated");
    let area = Rect::new(0, 0, 25, 1);
    let plane = label.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("label_long", snapshot);
}

// ── Radio Widget Snapshots ──────────────────────────────────────────────────

#[test]
fn test_radio_snapshot_basic() {
    let options = vec!["Option A", "Option B", "Option C"];
    let radio = Radio::new(
        dracon_terminal_engine::framework::widget::WidgetId::default_id(),
        options[0],
    );
    let area = Rect::new(0, 0, 20, 3);
    let plane = radio.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("radio_basic", snapshot);
}

// ── Theme Variations ────────────────────────────────────────────────────────

#[test]
fn test_list_nord_theme() {
    let items = vec!["One", "Two", "Three"];
    let list = List::new(items).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 30, 8);
    let plane = list.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("list_nord", snapshot);
}

#[test]
fn test_list_dracula_theme() {
    let items = vec!["Alpha", "Beta", "Gamma"];
    let list = List::new(items).with_theme(Theme::dracula());
    let area = Rect::new(0, 0, 30, 8);
    let plane = list.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("list_dracula", snapshot);
}

#[test]
fn test_table_catppuccin_theme() {
    let headers = vec!["ID", "Name", "Status"];
    let data = vec![
        vec!["1".to_string(), "Task 1".to_string(), "Done".to_string()],
        vec!["2".to_string(), "Task 2".to_string(), "Pending".to_string()],
    ];
    let rows: Vec<TableRow> = data
        .into_iter()
        .map(|cols| TableRow::new(cols.iter().map(|s| s.as_str()).collect()))
        .collect();
    let table = Table::new(headers, rows).with_theme(Theme::catppuccin_mocha());
    let area = Rect::new(0, 0, 35, 6);
    let plane = table.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("table_catppuccin", snapshot);
}

// ── Size Variations ──────────────────────────────────────────────────────────

#[test]
fn test_list_narrow() {
    let items = vec!["Short", "Item"];
    let list = List::new(items);
    let area = Rect::new(0, 0, 10, 5);
    let plane = list.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("list_narrow", snapshot);
}

#[test]
fn test_list_wide() {
    let items = vec!["Item 1", "Item 2"];
    let list = List::new(items);
    let area = Rect::new(0, 0, 60, 5);
    let plane = list.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("list_wide", snapshot);
}

#[test]
fn test_tree_wide() {
    let nodes = vec![
        TreeNode::new("Root Node", vec![
            TreeNode::new("Very Long Child Name", vec![]),
        ]),
    ];
    let tree = Tree::new(WidgetId::default_id()).with_root(nodes);
    let area = Rect::new(0, 0, 50, 8);
    let plane = tree.render(area);
    let snapshot = plane_to_snapshot(&plane);
    assert_debug_snapshot!("tree_wide", snapshot);
}