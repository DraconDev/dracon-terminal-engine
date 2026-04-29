use dracon_terminal_engine::framework::widgets::List;
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