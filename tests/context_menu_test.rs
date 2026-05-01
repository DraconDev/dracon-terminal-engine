//! Tests for ContextMenuAction enum.

mod common;

use dracon_terminal_engine::utils::FileColumn;
use dracon_terminal_engine::widgets::context_menu::ContextMenuAction;

#[test]
fn test_context_menu_action_variants() {
    use ContextMenuAction::*;
    let actions = [
        Open,
        OpenNewTab,
        OpenWith,
        Edit,
        Run,
        RunTerminal,
        ExtractHere,
        NewFolder,
        NewFile,
        Cut,
        Copy,
        CopyPath,
        CopyName,
        Paste,
        Rename,
        Duplicate,
        Compress,
        Delete,
        TerminalWindow,
        TerminalTab,
        Properties,
        GitStatus,
        AddToFavorites,
        RemoveFromFavorites,
        Refresh,
        SelectAll,
        ToggleHidden,
        ConnectRemote,
        DeleteRemote,
        Mount,
        Unmount,
        SetWallpaper,
        GitInit,
        SystemMonitor,
        Drag,
        Separator,
        SortBy(FileColumn::Name),
        SortBy(FileColumn::Size),
        SortBy(FileColumn::Modified),
        SortBy(FileColumn::Permissions),
        SetColor(None),
        SetColor(Some(128)),
    ];
    assert_eq!(actions.len(), 42);
}

#[test]
fn test_context_menu_action_clone() {
    use ContextMenuAction::*;
    let action = Open;
    let cloned = action.clone();
    assert_eq!(cloned, action);
}

#[test]
fn test_context_menu_action_debug() {
    use ContextMenuAction::*;
    let action = Delete;
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("Delete"));
}

#[test]
fn test_context_menu_action_partial_eq() {
    use ContextMenuAction::*;
    assert_eq!(Open, Open);
    assert_eq!(OpenNewTab, OpenNewTab);
    assert_ne!(Open, OpenNewTab);
}

#[test]
fn test_context_menu_action_set_color() {
    use ContextMenuAction::*;
    assert_eq!(SetColor(None), SetColor(None));
    assert_eq!(SetColor(Some(42)), SetColor(Some(42)));
    assert_ne!(SetColor(None), SetColor(Some(42)));
}

#[test]
fn test_context_menu_action_sort_by() {
    use ContextMenuAction::*;
    assert_eq!(SortBy(FileColumn::Name), SortBy(FileColumn::Name));
    assert_ne!(SortBy(FileColumn::Name), SortBy(FileColumn::Size));
}

#[test]
fn test_context_menu_action_serialize_deserialize() {
    use ContextMenuAction::*;
    let action = Open;
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: ContextMenuAction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, action);
}

#[test]
fn test_context_menu_action_serialize_sort_by() {
    let action = ContextMenuAction::SortBy(FileColumn::Size);
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: ContextMenuAction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, action);
}

#[test]
fn test_context_menu_action_serialize_set_color() {
    let action = ContextMenuAction::SetColor(Some(200));
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: ContextMenuAction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, action);
}
