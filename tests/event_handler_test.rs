//! Integration tests for widget event handling (handle_key, handle_mouse).

use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::framework::widgets::{List, SearchInput, Slider};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers: Default::default(),
    }
}

fn make_key_repeat(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Repeat,
        code,
        modifiers: Default::default(),
    }
}

// ========== List Event Tests ==========

#[test]
fn test_list_handle_key_down() {
    let items = vec!["a", "b", "c", "d", "e"];
    let mut list = List::new(items);
    assert_eq!(list.selected_index(), 0);

    list.handle_key(make_key(KeyCode::Down));
    assert_eq!(list.selected_index(), 1);

    list.handle_key(make_key(KeyCode::Down));
    assert_eq!(list.selected_index(), 2);
}

#[test]
fn test_list_handle_key_up() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    list.handle_key(make_key(KeyCode::Down));
    list.handle_key(make_key(KeyCode::Down));
    assert_eq!(list.selected_index(), 2);

    list.handle_key(make_key(KeyCode::Up));
    assert_eq!(list.selected_index(), 1);
}

#[test]
fn test_list_handle_key_home() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    list.handle_key(make_key(KeyCode::Down));
    list.handle_key(make_key(KeyCode::Down));
    assert_eq!(list.selected_index(), 2);

    list.handle_key(make_key(KeyCode::Home));
    assert_eq!(list.selected_index(), 0);
}

#[test]
fn test_list_handle_key_end() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    list.handle_key(make_key(KeyCode::End));
    assert_eq!(list.selected_index(), 2);
}

#[test]
fn test_list_handle_key_repeat_suppressed() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    assert_eq!(list.selected_index(), 0);

    list.handle_key(make_key_repeat(KeyCode::Down));
    assert_eq!(list.selected_index(), 0);
}

#[test]
fn test_list_handle_mouse_scroll() {
    let items: Vec<String> = (0..20).map(|i| format!("item{}", i)).collect();
    let mut list = List::new(items);
    list.set_visible_count(5);

    list.handle_mouse(MouseEventKind::ScrollDown, 0, 0);
    assert_eq!(list.selected_index(), 0);

    list.handle_mouse(MouseEventKind::ScrollUp, 0, 0);
    assert_eq!(list.selected_index(), 0);
}

// ========== SearchInput Event Tests ==========

#[test]
fn test_search_input_handle_key_char() {
    let mut si = SearchInput::new(WidgetId::default_id());
    assert_eq!(si.query(), "");

    si.handle_key(make_key(KeyCode::Char('h')));
    si.handle_key(make_key(KeyCode::Char('i')));
    assert_eq!(si.query(), "hi");
}

#[test]
fn test_search_input_handle_key_backspace() {
    let mut si = SearchInput::new(WidgetId::default_id());
    si.handle_key(make_key(KeyCode::Char('h')));
    si.handle_key(make_key(KeyCode::Char('i')));
    assert_eq!(si.query(), "hi");

    si.handle_key(make_key(KeyCode::Backspace));
    assert_eq!(si.query(), "h");
}

#[test]
fn test_search_input_handle_key_enter() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let mut si = SearchInput::new(WidgetId::default_id());
    let submitted = Rc::new(RefCell::new(String::new()));
    let submitted_clone = submitted.clone();
    si = si.on_submit(move |q| *submitted_clone.borrow_mut() = q.to_string());

    si.handle_key(make_key(KeyCode::Char('t')));
    si.handle_key(make_key(KeyCode::Char('e')));
    si.handle_key(make_key(KeyCode::Char('s')));
    si.handle_key(make_key(KeyCode::Char('t')));
    si.handle_key(make_key(KeyCode::Enter));
    assert_eq!(*submitted.borrow(), "test");
}

#[test]
fn test_search_input_handle_key_repeat_suppressed() {
    let mut si = SearchInput::new(WidgetId::default_id());
    si.handle_key(make_key(KeyCode::Char('a')));
    si.handle_key(make_key_repeat(KeyCode::Char('b')));
    assert_eq!(si.query(), "a");
}

#[test]
fn test_search_input_handle_key_home_end() {
    let mut si = SearchInput::new(WidgetId::default_id());
    si.handle_key(make_key(KeyCode::Char('a')));
    si.handle_key(make_key(KeyCode::Char('b')));
    si.handle_key(make_key(KeyCode::Char('c')));
    assert_eq!(si.query(), "abc");

    si.handle_key(make_key(KeyCode::Home));
    si.handle_key(make_key(KeyCode::End));
    assert_eq!(si.query(), "abc");
}

// ========== Slider Event Tests ==========

#[test]
fn test_slider_handle_mouse_in_bounds() {
    let mut slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    slider.set_area(Rect::new(0, 0, 40, 3));
    assert_eq!(slider.value(), 50.0);

    slider.handle_mouse(MouseEventKind::Down(MouseButton::Left), 2, 0);
    let v = slider.value();
    assert!(v < 50.0);
}

#[test]
fn test_slider_handle_mouse_out_of_bounds_returns_false() {
    let mut slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    slider.set_area(Rect::new(0, 0, 40, 3));
    let initial = slider.value();

    let consumed = slider.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 0);
    assert!(!consumed);
    assert_eq!(slider.value(), initial);
}

#[test]
fn test_slider_handle_key_not_implemented() {
    let mut slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    assert_eq!(slider.value(), 50.0);

    let result = slider.handle_key(make_key(KeyCode::Left));
    assert!(!result);
    assert_eq!(slider.value(), 50.0);
}

// ========== Checkbox Event Tests ==========

#[test]
fn test_checkbox_handle_key_enter() {
    use dracon_terminal_engine::framework::widgets::Checkbox;
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(!cb.is_checked());

    cb.handle_key(make_key(KeyCode::Enter));
    assert!(cb.is_checked());

    cb.handle_key(make_key(KeyCode::Enter));
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_handle_key_repeat_suppressed() {
    use dracon_terminal_engine::framework::widgets::Checkbox;
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(!cb.is_checked());

    cb.handle_key(make_key_repeat(KeyCode::Enter));
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_handle_mouse_click() {
    use dracon_terminal_engine::framework::widgets::Checkbox;
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(!cb.is_checked());

    cb.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(cb.is_checked());
}

// ========== SplitPane Event Tests ==========

#[test]
fn test_split_pane_z_index() {
    use dracon_terminal_engine::framework::widgets::SplitPane;
    let split = SplitPane::new(Orientation::Horizontal);
    assert_eq!(split.z_index(), 5);
}

#[test]
fn test_split_pane_mouse_returns_false_for_non_drag() {
    use dracon_terminal_engine::framework::widgets::SplitPane;
    let mut split = SplitPane::new(Orientation::Horizontal);
    split.set_area(Rect::new(0, 0, 80, 24));

    let result = split.handle_mouse(MouseEventKind::Down(MouseButton::Left), 40, 12);
    assert!(!result);
}

#[test]
fn test_split_pane_mouse_drag_updates_ratio() {
    use dracon_terminal_engine::framework::widgets::SplitPane;
    let mut split = SplitPane::new(Orientation::Horizontal);
    split.set_area(Rect::new(0, 0, 80, 24));
    let initial_ratio = split.get_ratio();

    split.handle_mouse(MouseEventKind::Drag(MouseButton::Left), 60, 12);
    let new_ratio = split.get_ratio();
    assert_ne!(initial_ratio, new_ratio);
}

// ========== Toggle Event Tests ==========

#[test]
fn test_toggle_handle_key_enter() {
    use dracon_terminal_engine::framework::widgets::Toggle;
    let mut t = Toggle::new(WidgetId::default_id(), "Enable");
    assert!(!t.is_on());

    t.handle_key(make_key(KeyCode::Enter));
    assert!(t.is_on());

    t.handle_key(make_key(KeyCode::Enter));
    assert!(!t.is_on());
}

#[test]
fn test_toggle_handle_mouse_click() {
    use dracon_terminal_engine::framework::widgets::Toggle;
    let mut t = Toggle::new(WidgetId::default_id(), "Enable");
    assert!(!t.is_on());

    t.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(t.is_on());
}

// ========== Radio Event Tests ==========

#[test]
fn test_radio_handle_key_enter() {
    use dracon_terminal_engine::framework::widgets::Radio;
    let mut r = Radio::new(WidgetId::default_id(), "Option A");
    assert!(!r.is_selected());

    r.handle_key(make_key(KeyCode::Enter));
    assert!(r.is_selected());
}

#[test]
fn test_radio_handle_mouse_click() {
    use dracon_terminal_engine::framework::widgets::Radio;
    let mut r = Radio::new(WidgetId::default_id(), "Option A");
    assert!(!r.is_selected());

    r.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(r.is_selected());
}

// ========== Widget Trait Tests ==========

#[test]
fn test_widget_id_default() {
    let id = WidgetId::default_id();
    assert_eq!(id, WidgetId::default_id());
}

#[test]
fn test_widget_id_equality() {
    let id1 = WidgetId::new(1);
    let id2 = WidgetId::new(1);
    let id3 = WidgetId::new(2);
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_widget_id_uniqueness() {
    let ids: Vec<_> = (0..100).map(WidgetId::new).collect();
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(id.0, i);
    }
}

#[test]
fn test_list_z_index() {
    let items = vec!["a", "b", "c"];
    let list = List::new(items);
    assert_eq!(list.z_index(), 10);
}

#[test]
fn test_list_area_set_and_get() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    let area = Rect::new(5, 10, 30, 15);
    list.set_area(area);
    assert_eq!(list.area(), area);
}

// ========== Table Event Tests ==========

#[test]
fn test_table_handle_key_down() {
    use dracon_terminal_engine::framework::widgets::table::TableBuilder;
    let mut table = TableBuilder::new(vec![
        Column::new("Name", 10),
        Column::new("Age", 5),
    ])
    .with_row_data("Alice", 25)
    .with_row_data("Bob", 30)
    .with_row_data("Carol", 35)
    .build();
    table.set_area(Rect::new(0, 0, 20, 10));
    assert_eq!(table.selected_index(), 0);

    table.handle_key(make_key(KeyCode::Down));
    assert_eq!(table.selected_index(), 1);

    table.handle_key(make_key(KeyCode::Down));
    assert_eq!(table.selected_index(), 2);
}

#[test]
fn test_table_handle_key_up() {
    use dracon_terminal_engine::framework::widgets::table::TableBuilder;
    let mut table = TableBuilder::new(vec![
        Column::new("Name", 10),
        Column::new("Age", 5),
    ])
    .with_row_data("Alice", 25)
    .with_row_data("Bob", 30)
    .with_row_data("Carol", 35)
    .build();
    table.set_area(Rect::new(0, 0, 20, 10));

    table.handle_key(make_key(KeyCode::Down));
    table.handle_key(make_key(KeyCode::Down));
    assert_eq!(table.selected_index(), 2);

    table.handle_key(make_key(KeyCode::Up));
    assert_eq!(table.selected_index(), 1);
}

#[test]
fn test_table_handle_key_home() {
    use dracon_terminal_engine::framework::widgets::table::TableBuilder;
    let mut table = TableBuilder::new(vec![
        Column::new("Name", 10),
        Column::new("Age", 5),
    ])
    .with_row_data("Alice", 25)
    .with_row_data("Bob", 30)
    .with_row_data("Carol", 35)
    .build();
    table.set_area(Rect::new(0, 0, 20, 10));

    table.handle_key(make_key(KeyCode::Down));
    table.handle_key(make_key(KeyCode::Down));
    assert_eq!(table.selected_index(), 2);

    table.handle_key(make_key(KeyCode::Home));
    assert_eq!(table.selected_index(), 0);
}

#[test]
fn test_table_handle_key_end() {
    use dracon_terminal_engine::framework::widgets::table::TableBuilder;
    let mut table = TableBuilder::new(vec![
        Column::new("Name", 10),
        Column::new("Age", 5),
    ])
    .with_row_data("Alice", 25)
    .with_row_data("Bob", 30)
    .with_row_data("Carol", 35)
    .build();
    table.set_area(Rect::new(0, 0, 20, 10));

    table.handle_key(make_key(KeyCode::End));
    assert_eq!(table.selected_index(), 2);
}

#[test]
fn test_table_handle_key_repeat_suppressed() {
    use dracon_terminal_engine::framework::widgets::table::TableBuilder;
    let mut table = TableBuilder::new(vec![
        Column::new("Name", 10),
        Column::new("Age", 5),
    ])
    .with_row_data("Alice", 25)
    .with_row_data("Bob", 30)
    .build();
    table.set_area(Rect::new(0, 0, 20, 10));
    assert_eq!(table.selected_index(), 0);

    table.handle_key(make_key_repeat(KeyCode::Down));
    assert_eq!(table.selected_index(), 0);
}

#[test]
fn test_table_handle_mouse_down_selects_row() {
    use dracon_terminal_engine::framework::widgets::table::TableBuilder;
    let mut table = TableBuilder::new(vec![
        Column::new("Name", 10),
        Column::new("Age", 5),
    ])
    .with_row_data("Alice", 25)
    .with_row_data("Bob", 30)
    .build();
    table.set_area(Rect::new(0, 0, 20, 10));
    assert_eq!(table.selected_index(), 0);

    table.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 2);
    assert_eq!(table.selected_index(), 0);

    table.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 3);
    assert_eq!(table.selected_index(), 1);
}

#[test]
fn test_table_handle_mouse_moved_tracks_hover() {
    use dracon_terminal_engine::framework::widgets::table::TableBuilder;
    let mut table = TableBuilder::new(vec![
        Column::new("Name", 10),
        Column::new("Age", 5),
    ])
    .with_row_data("Alice", 25)
    .with_row_data("Bob", 30)
    .build();
    table.set_area(Rect::new(0, 0, 20, 10));

    table.handle_mouse(MouseEventKind::Moved, 0, 2);
    assert!(table.hovered_row().is_some());

    table.handle_mouse(MouseEventKind::Moved, 0, 0);
    assert!(table.hovered_row().is_none());
}

// ========== Tree Event Tests ==========

#[test]
fn test_tree_handle_key_down_navigates_to_child() {
    use dracon_terminal_engine::framework::widgets::TreeNode;
    let mut tree = Tree::new(TreeNode::new("root", vec![
        TreeNode::new("child1", vec![]),
        TreeNode::new("child2", vec![]),
    ]));
    tree.set_area(Rect::new(0, 0, 20, 10));

    assert_eq!(tree.selected_path(), vec![0]);

    tree.handle_key(make_key(KeyCode::Down));
    assert_eq!(tree.selected_path(), vec![0, 0]);

    tree.handle_key(make_key(KeyCode::Up));
    assert_eq!(tree.selected_path(), vec![0]);
}

#[test]
fn test_tree_handle_key_left_collapse() {
    use dracon_terminal_engine::framework::widgets::TreeNode;
    let mut tree = Tree::new(TreeNode::new("root", vec![
        TreeNode::new("child1", vec![]),
    ]));
    tree.set_area(Rect::new(0, 0, 20, 10));

    tree.handle_key(make_key(KeyCode::Down));
    assert_eq!(tree.selected_path(), vec![0, 0]);

    tree.handle_key(make_key(KeyCode::Left));
    assert_eq!(tree.selected_path(), vec![0]);

    tree.handle_key(make_key(KeyCode::Left));
    assert!(!tree.is_expanded_at(&vec![0]));
}

#[test]
fn test_tree_handle_key_right_expands() {
    use dracon_terminal_engine::framework::widgets::TreeNode;
    let mut tree = Tree::new(TreeNode::new("root", vec![
        TreeNode::new("child1", vec![]),
    ]));
    tree.set_area(Rect::new(0, 0, 20, 10));

    assert!(!tree.is_expanded_at(&vec![0]));

    tree.handle_key(make_key(KeyCode::Right));
    assert!(tree.is_expanded_at(&vec![0]));
}

#[test]
fn test_tree_handle_key_enter_toggles_expand() {
    use dracon_terminal_engine::framework::widgets::TreeNode;
    let mut tree = Tree::new(TreeNode::new("root", vec![
        TreeNode::new("child1", vec![]),
    ]));
    tree.set_area(Rect::new(0, 0, 20, 10));

    tree.handle_key(make_key(KeyCode::Enter));
    assert!(tree.is_expanded_at(&vec![0]));

    tree.handle_key(make_key(KeyCode::Enter));
    assert!(!tree.is_expanded_at(&vec![0]));
}

#[test]
fn test_tree_handle_key_repeat_suppressed() {
    use dracon_terminal_engine::framework::widgets::TreeNode;
    let mut tree = Tree::new(TreeNode::new("root", vec![
        TreeNode::new("child1", vec![]),
    ]));
    tree.set_area(Rect::new(0, 0, 20, 10));

    tree.handle_key(make_key_repeat(KeyCode::Down));
    assert_eq!(tree.selected_path(), vec![0]);
}

// ========== TabBar Event Tests ==========

#[test]
fn test_tabbar_handle_key_left() {
    use dracon_terminal_engine::framework::widgets::TabBar;
    let tabs = vec!["Tab1", "Tab2", "Tab3"];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_area(Rect::new(0, 0, 30, 1));
    assert_eq!(tabbar.active_tab(), 0);

    tabbar.handle_key(make_key(KeyCode::Right));
    assert_eq!(tabbar.active_tab(), 1);

    tabbar.handle_key(make_key(KeyCode::Left));
    assert_eq!(tabbar.active_tab(), 0);
}

#[test]
fn test_tabbar_handle_key_right() {
    use dracon_terminal_engine::framework::widgets::TabBar;
    let tabs = vec!["Tab1", "Tab2", "Tab3"];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_area(Rect::new(0, 0, 30, 1));

    tabbar.handle_key(make_key(KeyCode::Right));
    assert_eq!(tabbar.active_tab(), 1);

    tabbar.handle_key(make_key(KeyCode::Right));
    assert_eq!(tabbar.active_tab(), 2);
}

#[test]
fn test_tabbar_handle_key_repeat_suppressed() {
    use dracon_terminal_engine::framework::widgets::TabBar;
    let tabs = vec!["Tab1", "Tab2"];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_area(Rect::new(0, 0, 30, 1));

    tabbar.handle_key(make_key_repeat(KeyCode::Right));
    assert_eq!(tabbar.active_tab(), 0);
}

#[test]
fn test_tabbar_handle_mouse_click_selects_tab() {
    use dracon_terminal_engine::framework::widgets::TabBar;
    let tabs = vec!["Tab1", "Tab2", "Tab3"];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_area(Rect::new(0, 0, 30, 1));
    assert_eq!(tabbar.active_tab(), 0);

    tabbar.handle_mouse(MouseEventKind::Down(MouseButton::Left), 12, 0);
    assert_eq!(tabbar.active_tab(), 1);
}

#[test]
fn test_tabbar_handle_mouse_moved_tracks_hover() {
    use dracon_terminal_engine::framework::widgets::TabBar;
    let tabs = vec!["Tab1", "Tab2"];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_area(Rect::new(0, 0, 30, 1));

    tabbar.handle_mouse(MouseEventKind::Moved, 5, 0);
    assert_eq!(tabbar.hovered_tab(), Some(0));

    tabbar.handle_mouse(MouseEventKind::Moved, 20, 0);
    assert_eq!(tabbar.hovered_tab(), Some(1));
}

// ========== Select Event Tests ==========

#[test]
fn test_select_handle_key_enter_toggles() {
    use dracon_terminal_engine::framework::widgets::Select;
    let options = vec!["Option A".to_string(), "Option B".to_string()];
    let mut select = Select::new(WidgetId::default_id(), options);
    assert!(!select.is_expanded());

    select.handle_key(make_key(KeyCode::Enter));
    assert!(select.is_expanded());

    select.handle_key(make_key(KeyCode::Enter));
    assert!(!select.is_expanded());
}

#[test]
fn test_select_handle_key_down_when_expanded() {
    use dracon_terminal_engine::framework::widgets::Select;
    let options = vec!["Option A".to_string(), "Option B".to_string()];
    let mut select = Select::new(WidgetId::default_id(), options);
    select.handle_key(make_key(KeyCode::Enter));
    assert!(select.is_expanded());
    assert_eq!(select.selected_index(), 0);

    select.handle_key(make_key(KeyCode::Down));
    assert_eq!(select.selected_index(), 1);
}

#[test]
fn test_select_handle_key_up_when_expanded() {
    use dracon_terminal_engine::framework::widgets::Select;
    let options = vec!["Option A".to_string(), "Option B".to_string()];
    let mut select = Select::new(WidgetId::default_id(), options);
    select.handle_key(make_key(KeyCode::Enter));
    select.handle_key(make_key(KeyCode::Down));
    assert_eq!(select.selected_index(), 1);

    select.handle_key(make_key(KeyCode::Up));
    assert_eq!(select.selected_index(), 0);
}

#[test]
fn test_select_handle_mouse_click_toggles_expanded() {
    use dracon_terminal_engine::framework::widgets::Select;
    let options = vec!["Option A".to_string()];
    let mut select = Select::new(WidgetId::default_id(), options);
    assert!(!select.is_expanded());

    select.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(select.is_expanded());
}

#[test]
fn test_select_handle_mouse_click_on_option_selects() {
    use dracon_terminal_engine::framework::widgets::Select;
    let options = vec!["Option A".to_string(), "Option B".to_string()];
    let mut select = Select::new(WidgetId::default_id(), options);
    select.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(select.is_expanded());

    select.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 2);
    assert_eq!(select.selected_index(), 1);
    assert!(!select.is_expanded());
}

// ========== CommandPalette Event Tests ==========

#[test]
fn test_command_palette_handle_key_up() {
    let cmds = vec![
        CommandItem::new("cmd1", "Command One"),
        CommandItem::new("cmd2", "Command Two"),
    ];
    let mut palette = CommandPalette::new(cmds);
    palette.show();
    assert_eq!(palette.selected_index(), 0);

    palette.handle_key(make_key(KeyCode::Up));
    assert_eq!(palette.selected_index(), 1);
}

#[test]
fn test_command_palette_handle_key_down() {
    let cmds = vec![
        CommandItem::new("cmd1", "Command One"),
        CommandItem::new("cmd2", "Command Two"),
    ];
    let mut palette = CommandPalette::new(cmds);
    palette.show();
    palette.handle_key(make_key(KeyCode::Up));
    assert_eq!(palette.selected_index(), 1);

    palette.handle_key(make_key(KeyCode::Down));
    assert_eq!(palette.selected_index(), 0);
}

#[test]
fn test_command_palette_handle_key_enter_executes() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let cmds = vec![
        CommandItem::new("cmd1", "Command One"),
        CommandItem::new("cmd2", "Command Two"),
    ];
    let mut palette = CommandPalette::new(cmds);
    let executed = Rc::new(RefCell::new(None));
    let executed_clone = executed.clone();
    palette = palette.on_execute(move |id| *executed_clone.borrow_mut() = Some(id.to_string()));
    palette.show();

    palette.handle_key(make_key(KeyCode::Enter));
    assert_eq!(*executed.borrow(), Some("cmd1".to_string()));
}

#[test]
fn test_command_palette_handle_key_esc_hides() {
    let cmds = vec![CommandItem::new("cmd1", "Command One")];
    let mut palette = CommandPalette::new(cmds);
    palette.show();
    assert!(palette.is_visible());

    palette.handle_key(make_key(KeyCode::Esc));
    assert!(!palette.is_visible());
}

#[test]
fn test_command_palette_handle_key_typing_filters() {
    let cmds = vec![
        CommandItem::new("cmd1", "Alpha"),
        CommandItem::new("cmd2", "Beta"),
        CommandItem::new("cmd3", "Gamma"),
    ];
    let mut palette = CommandPalette::new(cmds);
    palette.show();

    palette.handle_key(make_key(KeyCode::Char('a')));
    assert_eq!(palette.selected_index(), 0);

    palette.handle_key(make_key(KeyCode::Char('l')));
}

#[test]
fn test_command_palette_handle_key_backspace() {
    let cmds = vec![
        CommandItem::new("cmd1", "Alpha"),
        CommandItem::new("cmd2", "Beta"),
    ];
    let mut palette = CommandPalette::new(cmds);
    palette.show();

    palette.handle_key(make_key(KeyCode::Char('A')));
    palette.handle_key(make_key(KeyCode::Backspace));
}

#[test]
fn test_command_palette_handle_key_when_not_visible_returns_false() {
    let cmds = vec![CommandItem::new("cmd1", "Command One")];
    let mut palette = CommandPalette::new(cmds);
    assert!(!palette.is_visible());

    let result = palette.handle_key(make_key(KeyCode::Down));
    assert!(!result);
}

// ========== Modal Event Tests ==========

#[test]
fn test_modal_handle_key_tab_cycles_focus() {
    use dracon_terminal_engine::framework::widgets::{Modal, ModalButton, ModalResult};
    let mut modal = Modal::new("Test?", vec![
        ModalButton::new("OK", ModalResult::Confirm),
        ModalButton::new("Cancel", ModalResult::Cancel),
    ]);
    modal.set_area(Rect::new(0, 0, 40, 10));
    assert_eq!(modal.focused_button(), 0);

    modal.handle_key(make_key(KeyCode::Tab));
    assert_eq!(modal.focused_button(), 1);

    modal.handle_key(make_key(KeyCode::Tab));
    assert_eq!(modal.focused_button(), 0);
}

#[test]
fn test_modal_handle_key_backtab() {
    use dracon_terminal_engine::framework::widgets::{Modal, ModalButton, ModalResult};
    let mut modal = Modal::new("Test?", vec![
        ModalButton::new("OK", ModalResult::Confirm),
        ModalButton::new("Cancel", ModalResult::Cancel),
    ]);
    modal.set_area(Rect::new(0, 0, 40, 10));
    assert_eq!(modal.focused_button(), 0);

    modal.handle_key(make_key(KeyCode::BackTab));
    assert_eq!(modal.focused_button(), 1);
}

#[test]
fn test_modal_handle_key_enter_triggers_focused() {
    use std::cell::RefCell;
    use std::rc::Rc;
    use dracon_terminal_engine::framework::widgets::{Modal, ModalButton, ModalResult};
    let mut modal = Modal::new("Test?", vec![
        ModalButton::new("OK", ModalResult::Confirm),
    ]);
    modal.set_area(Rect::new(0, 0, 40, 10));
    let confirmed = Rc::new(RefCell::new(false));
    let confirmed_clone = confirmed.clone();
    modal = modal.on_confirm(move || *confirmed_clone.borrow_mut() = true);

    modal.handle_key(make_key(KeyCode::Enter));
    assert!(modal.result().is_some());
}

#[test]
fn test_modal_handle_key_esc_triggers_cancel() {
    use dracon_terminal_engine::framework::widgets::{Modal, ModalButton, ModalResult};
    let mut modal = Modal::new("Test?", vec![
        ModalButton::new("OK", ModalResult::Confirm),
        ModalButton::new("Cancel", ModalResult::Cancel),
    ]);
    modal.set_area(Rect::new(0, 0, 40, 10));

    modal.handle_key(make_key(KeyCode::Esc));
    assert_eq!(modal.result(), Some(ModalResult::Cancel));
}

#[test]
fn test_modal_handle_mouse_click_button() {
    use dracon_terminal_engine::framework::widgets::{Modal, ModalButton, ModalResult};
    let mut modal = Modal::new("Test?", vec![
        ModalButton::new("OK", ModalResult::Confirm),
        ModalButton::new("Cancel", ModalResult::Cancel),
    ]);
    modal.set_area(Rect::new(0, 0, 40, 10));

    let cx = 20;
    let cy = 8;
    let result = modal.handle_mouse(MouseEventKind::Down(MouseButton::Left), cx, cy);
    assert!(result);
}

// ========== Spinner Event Tests ==========

#[test]
fn test_spinner_handle_key_returns_false() {
    use dracon_terminal_engine::framework::widgets::Spinner;
    let mut spinner = Spinner::new();
    spinner.set_area(Rect::new(0, 0, 10, 1));

    let result = spinner.handle_key(make_key(KeyCode::Enter));
    assert!(!result);
}

#[test]
fn test_spinner_handle_key_repeat_returns_false() {
    use dracon_terminal_engine::framework::widgets::Spinner;
    let mut spinner = Spinner::new();
    spinner.set_area(Rect::new(0, 0, 10, 1));

    let result = spinner.handle_key(make_key_repeat(KeyCode::Enter));
    assert!(!result);
}

// ========== MenuBar Event Tests ==========

#[test]
fn test_menubar_handle_mouse_click_toggles_entry() {
    use dracon_terminal_engine::framework::widgets::{MenuBar, MenuEntry};
    let entries = vec![
        MenuEntry::new("File", vec![]),
        MenuEntry::new("Edit", vec![]),
    ];
    let mut menubar = MenuBar::new(entries);
    menubar.set_area(Rect::new(0, 0, 40, 1));
    assert_eq!(menubar.active_entry(), None);

    menubar.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 0);
    assert_eq!(menubar.active_entry(), Some(0));

    menubar.handle_mouse(MouseEventKind::Down(MouseButton::Left), 25, 0);
    assert_eq!(menubar.active_entry(), Some(1));
}

#[test]
fn test_menubar_handle_mouse_click_same_entry_toggles_off() {
    use dracon_terminal_engine::framework::widgets::{MenuBar, MenuEntry};
    let entries = vec![
        MenuEntry::new("File", vec![]),
        MenuEntry::new("Edit", vec![]),
    ];
    let mut menubar = MenuBar::new(entries);
    menubar.set_area(Rect::new(0, 0, 40, 1));

    menubar.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 0);
    assert_eq!(menubar.active_entry(), Some(0));

    menubar.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 0);
    assert_eq!(menubar.active_entry(), None);
}

#[test]
fn test_menubar_handle_mouse_row_nonzero_returns_false() {
    use dracon_terminal_engine::framework::widgets::{MenuBar, MenuEntry};
    let entries = vec![MenuEntry::new("File", vec![])];
    let mut menubar = MenuBar::new(entries);
    menubar.set_area(Rect::new(0, 0, 40, 1));

    let result = menubar.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 1);
    assert!(!result);
}

// ========== Form Event Tests ==========

#[test]
fn test_form_handle_key_tab_navigates() {
    use dracon_terminal_engine::framework::widgets::{Form, FormField};
    let mut form = Form::new(vec![
        FormField::text_input(WidgetId::new(1), "Name"),
        FormField::text_input(WidgetId::new(2), "Email"),
    ]);
    form.set_area(Rect::new(0, 0, 40, 10));
    assert_eq!(form.focused_field(), 0);

    form.handle_key(make_key(KeyCode::Tab));
    assert_eq!(form.focused_field(), 1);

    form.handle_key(make_key(KeyCode::BackTab));
    assert_eq!(form.focused_field(), 0);
}

#[test]
fn test_form_handle_key_enter_in_field() {
    use dracon_terminal_engine::framework::widgets::SearchInput;
    let mut form = Form::new(vec![]);
    form.set_area(Rect::new(0, 0, 40, 10));
    let result = form.handle_key(make_key(KeyCode::Enter));
    assert!(!result);
}
