// Snapshot tests for framework widgets using the insta crate.
#[cfg(test)]
mod widget_snapshots {
    use dracon_terminal_engine::compositor::Plane;
    use dracon_terminal_engine::framework::theme::Theme;
    use dracon_terminal_engine::framework::widget::Widget;
    use dracon_terminal_engine::framework::widgets::form::Form;
    use dracon_terminal_engine::framework::widgets::list::List;
    use dracon_terminal_engine::framework::widgets::table::{Column, Table};
    use dracon_terminal_engine::framework::widgets::tree::{Tree, TreeNode};
    use ratatui::layout::Rect;

    /// Helper to render a widget and serialize its Plane to a string for comparison.
    fn render_widget_to_string<W: Widget>(widget: &W, width: u16, height: u16) -> String {
        let area = Rect::new(0, 0, width, height);
        let plane = widget.render(area);
        render_plane_to_string(&plane)
    }

    /// Renders a Plane to a simple string representation for snapshot comparison.
    fn render_plane_to_string(plane: &Plane) -> String {
        let mut result = String::new();
        for row in 0..plane.height {
            for col in 0..plane.width {
                let idx = (row * plane.width + col) as usize;
                if idx < plane.cells.len() {
                    let cell = &plane.cells[idx];
                    let ch = if cell.transparent { ' ' } else { cell.char };
                    result.push(ch);
                } else {
                    result.push(' ');
                }
            }
            result.push('\n');
        }
        result
    }

    #[test]
    fn test_list_snapshot() {
        let theme = Theme::default();
        let items = vec![
            "Item One",
            "Item Two",
            "Item Three",
            "Item Four",
            "Item Five",
        ];
        let list = List::new(items).with_theme(theme);
        let output = render_widget_to_string(&list, 20, 5);
        insta::assert_snapshot!("list_widget", output);
    }

    #[test]
    fn test_table_snapshot() {
        let theme = Theme::default();
        let columns = vec![
            Column {
                header: "Name".to_string(),
                width: 10,
            },
            Column {
                header: "Value".to_string(),
                width: 8,
            },
        ];
        let rows = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let table = Table::new(columns).with_rows(rows).with_theme(theme);
        let output = render_widget_to_string(&table, 20, 5);
        insta::assert_snapshot!("table_widget", output);
    }

    #[test]
    fn test_tree_snapshot() {
        let theme = Theme::default();
        let mut root = TreeNode::new("Root");
        {
            let mut child1 = TreeNode::new("Child 1");
            child1.add_child(TreeNode::new("Grandchild 1"));
            child1.add_child(TreeNode::new("Grandchild 2"));
            root.add_child(child1);
        }
        {
            let mut child2 = TreeNode::new("Child 2");
            child2.add_child(TreeNode::new("Grandchild 3"));
            root.add_child(child2);
        }
        let tree = Tree::new(Default::default())
            .with_root(vec![root])
            .with_theme(theme);
        let output = render_widget_to_string(&tree, 30, 10);
        insta::assert_snapshot!("tree_widget", output);
    }

    #[test]
    fn test_form_snapshot() {
        let theme = Theme::default();
        let form = Form::new(Default::default())
            .add_field("Name")
            .add_field("Email")
            .add_field("Password")
            .with_theme(theme);
        let output = render_widget_to_string(&form, 40, 8);
        insta::assert_snapshot!("form_widget", output);
    }
}
