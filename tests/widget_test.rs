use dracon_terminal_engine::framework::widgets::{List, Table, TabBar, Breadcrumbs, SplitPane, Hud, Modal, ContextMenu};
use dracon_terminal_engine::framework::widgets::context_menu::ContextAction;
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::framework::widget::Widget;
use ratatui::layout::Rect;

#[test]
fn test_list_new() {
    let items = vec!["a", "b", "c"];
    let list = List::new(items.clone());
    assert_eq!(list.len(), 3);
}

#[test]
fn test_list_render() {
    let items = vec!["Item 1", "Item 2", "Item 3"];
    let list = List::new(items);
    let area = Rect::new(0, 0, 40, 10);
    let plane = list.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_list_selected_index() {
    let items = vec!["a", "b", "c"];
    let list = List::new(items);
    assert_eq!(list.selected_index(), 0);
}

#[test]
fn test_list_with_theme() {
    use dracon_terminal_engine::framework::theme::Theme;
    let items = vec!["a", "b"];
    let list = List::new(items).with_theme(Theme::cyberpunk());
    let area = Rect::new(0, 0, 20, 5);
    let plane = list.render(area);
    assert_eq!(plane.width, 20);
}

#[test]
fn test_list_visible_count() {
    let items = vec!["a"; 100];
    let mut list = List::new(items);
    list.set_visible_count(20);
    let area = Rect::new(0, 0, 20, 50);
    let plane = list.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 50);
}

#[test]
fn test_table_new() {
    let table: Table<String> = Table::new(vec![]);
    assert_eq!(table.len(), 0);
}

#[test]
fn test_table_render() {
    use dracon_terminal_engine::framework::widgets::table::Column;
    let cols = vec![
        Column { header: "Name".to_string(), width: 20 },
        Column { header: "Age".to_string(), width: 10 },
    ];
    let table: Table<String> = Table::new(cols);
    let area = Rect::new(0, 0, 80, 20);
    let plane = table.render(area);
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_tabbar_new() {
    let tabs = vec!["Tab A", "Tab B", "Tab C"];
    let tabbar = TabBar::new(tabs.clone());
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_render() {
    let tabs = vec!["Tab A", "Tab B"];
    let tabbar = TabBar::new(tabs);
    let area = Rect::new(0, 0, 80, 3);
    let plane = tabbar.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_breadcrumbs_new() {
    let crumbs = vec!["home".to_string(), "user".to_string()];
    let bc = Breadcrumbs::new(crumbs);
    let area = Rect::new(0, 0, 80, 1);
    let plane = bc.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_split_pane_new() {
    let split = SplitPane::new(Orientation::Horizontal);
    assert_eq!(split.get_ratio(), 0.5);
}

#[test]
fn test_split_pane_split() {
    let split = SplitPane::new(Orientation::Horizontal);
    let area = Rect::new(0, 0, 80, 24);
    let (left, right) = split.split(area);
    assert!(left.width > 0);
    assert!(right.width > 0);
    assert_eq!(left.width + right.width, 80);
}

#[test]
fn test_hud_new() {
    let hud = Hud::new(50);
    assert!(hud.is_visible());
}

#[test]
fn test_hud_render() {
    let hud = Hud::new(50);
    let area = Rect::new(0, 0, 30, 10);
    let plane = hud.render(area);
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height, 10);
}

#[test]
fn test_modal_new() {
    let modal = Modal::new("Hello");
    let area = Rect::new(0, 0, 40, 5);
    let plane = modal.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 5);
}

#[test]
fn test_context_menu_new() {
    let menu = ContextMenu::new(vec![("Open", ContextAction::Open)]);
    let area = Rect::new(0, 0, 20, 10);
    let plane = menu.render(area);
    assert!(plane.width > 0);
}