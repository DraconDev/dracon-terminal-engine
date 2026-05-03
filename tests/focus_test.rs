//! Integration tests for FocusManager and focus cycling between widgets.
//!
//! These tests verify:
//! - FocusManager basic operations (new, register, unregister, set_focus, focused)
//! - Tab cycling (tab_next, tab_prev, wrapping)
//! - Focus traps (enter_trap, exit_trap)
//! - on_focus/on_blur callbacks
//! - Focus persistence
//! - FocusManager integration with App's widget registry

use dracon_terminal_engine::framework::focus::FocusManager;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use std::sync::{Arc, Mutex};

mod common;

use common::TrackingWidget;

/// Test 1: FocusManager basic operations
///
/// Verifies:
/// - new() creates an empty manager with no focused widget
/// - register() adds widgets to the focus ring
/// - set_focus() and focused() work correctly
/// - Multiple widgets can be registered and focused
mod focus_manager_basic {
    use super::*;

    #[test]
    fn test_new_creates_empty_manager() {
        let fm = FocusManager::new();
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn test_register_adds_widget_to_ring() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);
        fm.register(id, true);
        assert!(fm.set_focus(id));
        assert_eq!(fm.focused(), Some(id));
    }

    #[test]
    fn test_unregister_removes_widget() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);
        fm.register(id, true);
        fm.set_focus(id);
        fm.unregister(id);
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn test_register_multiple_widgets() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);
        let id3 = WidgetId::new(3);

        fm.register(id1, true);
        fm.register(id2, true);
        fm.register(id3, true);

        assert!(fm.set_focus(id1));
        assert_eq!(fm.focused(), Some(id1));

        fm.set_focus(id3);
        assert_eq!(fm.focused(), Some(id3));
    }

    #[test]
    fn test_set_focus_to_unfocusable_widget_returns_false() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);
        fm.register(id, false);
        assert!(!fm.set_focus(id));
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn test_clear_focus() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);
        fm.register(id, true);
        fm.set_focus(id);
        fm.clear_focus();
        assert_eq!(fm.focused(), None);
    }
}

/// Test 2: Tab cycling
///
/// Verifies:
/// - tab_next() cycles forward through registered widgets
/// - tab_prev() cycles backward
/// - Cycling wraps around (last -> first, first -> last)
/// - Empty registry doesn't crash
mod tab_cycling {
    use super::*;

    #[test]
    fn test_tab_next_cycles_forward() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);
        let id3 = WidgetId::new(3);

        fm.register(id1, true);
        fm.register(id2, true);
        fm.register(id3, true);

        fm.set_focus(id1);
        assert_eq!(fm.tab_next(), Some(id2));
        assert_eq!(fm.tab_next(), Some(id3));
        assert_eq!(fm.tab_next(), Some(id1)); // wraps around
    }

    #[test]
    fn test_tab_prev_cycles_backward() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);
        let id3 = WidgetId::new(3);

        fm.register(id1, true);
        fm.register(id2, true);
        fm.register(id3, true);

        fm.set_focus(id3);
        assert_eq!(fm.tab_prev(), Some(id2));
        assert_eq!(fm.tab_prev(), Some(id1));
        assert_eq!(fm.tab_prev(), Some(id3)); // wraps around
    }

    #[test]
    fn test_tab_next_wraps_last_to_first() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);

        fm.register(id1, true);
        fm.register(id2, true);

        fm.set_focus(id2);
        assert_eq!(fm.tab_next(), Some(id1)); // wraps to first
    }

    #[test]
    fn test_tab_prev_wraps_first_to_last() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);

        fm.register(id1, true);
        fm.register(id2, true);

        fm.set_focus(id1);
        assert_eq!(fm.tab_prev(), Some(id2)); // wraps to last
    }

    #[test]
    fn test_empty_registry_tab_next_returns_none() {
        let mut fm = FocusManager::new();
        assert_eq!(fm.tab_next(), None);
    }

    #[test]
    fn test_empty_registry_tab_prev_returns_none() {
        let mut fm = FocusManager::new();
        assert_eq!(fm.tab_prev(), None);
    }

    #[test]
    fn test_tab_next_with_no_initial_focus_goes_to_first() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);

        fm.register(id1, true);
        fm.register(id2, true);

        assert_eq!(fm.tab_next(), Some(id1));
    }

    #[test]
    fn test_only_focusable_widgets_appear_in_cycle() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);
        let id3 = WidgetId::new(3);

        fm.register(id1, true);
        fm.register(id2, false);
        fm.register(id3, true);

        fm.set_focus(id1);
        assert_eq!(fm.tab_next(), Some(id3));
        assert_eq!(fm.tab_next(), Some(id1));
    }
}

/// Test 3: Focus traps
///
/// Verifies:
/// - enter_trap() prevents focus from leaving a widget
/// - exit_trap() releases the trap
/// - Tab cycling respects trap
mod focus_traps {
    use super::*;

    #[test]
    fn test_enter_trap_sets_trapped_flag() {
        let mut fm = FocusManager::new();
        assert!(!fm.is_trapped());
        fm.enter_trap();
        assert!(fm.is_trapped());
    }

    #[test]
    fn test_exit_trap_clears_trapped_flag() {
        let mut fm = FocusManager::new();
        fm.enter_trap();
        fm.enable_trap_exit();
        fm.exit_trap();
        assert!(!fm.is_trapped());
    }

    #[test]
    fn test_exit_trap_without_enable_does_nothing() {
        let mut fm = FocusManager::new();
        fm.enter_trap();
        fm.exit_trap();
        assert!(fm.is_trapped());
    }

    #[test]
    fn test_trap_callback_called_on_enter() {
        let mut fm = FocusManager::new();
        let trap_changes = Arc::new(Mutex::new(Vec::new()));
        let trap_changes_clone = trap_changes.clone();

        fm.on_trap_change(move |entered| {
            trap_changes_clone.lock().unwrap().push(entered);
        });

        fm.enter_trap();
        assert_eq!(trap_changes.lock().unwrap().as_slice(), &[true]);
    }

    #[test]
    fn test_trap_callback_called_on_exit() {
        let mut fm = FocusManager::new();
        let trap_changes = Arc::new(Mutex::new(Vec::new()));
        let trap_changes_clone = trap_changes.clone();

        fm.on_trap_change(move |entered| {
            trap_changes_clone.lock().unwrap().push(entered);
        });

        fm.enter_trap();
        fm.enable_trap_exit();
        fm.exit_trap();
        assert_eq!(trap_changes.lock().unwrap().as_slice(), &[true, false]);
    }

    #[test]
    fn test_multiple_enter_trap_calls_only_one_callback() {
        let mut fm = FocusManager::new();
        let trap_changes = Arc::new(Mutex::new(Vec::new()));
        let trap_changes_clone = trap_changes.clone();

        fm.on_trap_change(move |entered| {
            trap_changes_clone.lock().unwrap().push(entered);
        });

        fm.enter_trap();
        fm.enter_trap();
        fm.enter_trap();
        assert_eq!(trap_changes.lock().unwrap().as_slice(), &[true]);
    }
}

/// Test 4: on_focus/on_blur callbacks
///
/// Verifies:
/// - Widgets receive on_focus() when gaining focus
/// - Widgets receive on_blur() when losing focus
/// - Multiple focus changes in sequence
mod focus_callbacks {
    use super::*;

    #[test]
    fn test_widget_receives_on_focus_when_gaining_focus() {
        let mut w = TrackingWidget::new(1);
        assert_eq!(w.focus_count(), 0);

        Widget::on_focus(&mut w);
        assert_eq!(w.focus_count(), 1);
    }

    #[test]
    fn test_widget_receives_on_blur_when_losing_focus() {
        let mut w = TrackingWidget::new(1);
        assert_eq!(w.blur_count(), 0);

        Widget::on_blur(&mut w);
        assert_eq!(w.blur_count(), 1);
    }

    #[test]
    fn test_focus_manager_notifies_callbacks_on_focus_change() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);

        fm.register(id1, true);
        fm.register(id2, true);

        let changes = Arc::new(Mutex::new(Vec::new()));
        let changes_clone = changes.clone();

        fm.on_focus_change(move |new_id, old_id| {
            changes_clone.lock().unwrap().push((new_id, old_id));
        });

        fm.set_focus(id1);
        fm.set_focus(id2);
        fm.set_focus(id1);

        let recorded = changes.lock().unwrap();
        assert_eq!(recorded.len(), 3);
        assert_eq!(recorded[0], (id1, None));
        assert_eq!(recorded[1], (id2, Some(id1)));
        assert_eq!(recorded[2], (id1, Some(id2)));
    }

    #[test]
    fn test_focus_change_callback_receives_old_and_new_ids() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);

        fm.register(id1, true);
        fm.register(id2, true);

        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured.clone();

        fm.on_focus_change(move |new_id, old_id| {
            captured_clone.lock().unwrap().push((Some(new_id), old_id));
        });

        fm.set_focus(id1);
        fm.set_focus(id2);

        let result = captured.lock().unwrap();
        assert_eq!(result[0], (Some(id1), None));
        assert_eq!(result[1], (Some(id2), Some(id1)));
    }
}

/// Test 5: Focus persistence
///
/// Verifies:
/// - Setting focus to same widget doesn't re-trigger callback
/// - Unregistering focused widget transfers focus appropriately
mod focus_persistence {
    use super::*;

    #[test]
    fn test_set_focus_to_same_widget_does_not_trigger_callback() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);

        fm.register(id, true);

        let changes = Arc::new(Mutex::new(Vec::new()));
        let changes_clone = changes.clone();

        fm.on_focus_change(move |new_id, old_id| {
            changes_clone.lock().unwrap().push((new_id, old_id));
        });

        fm.set_focus(id);
        fm.set_focus(id);
        fm.set_focus(id);

        assert_eq!(changes.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_unregistering_focused_widget_clears_focus() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);

        fm.register(id, true);
        fm.set_focus(id);
        fm.unregister(id);

        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn test_unregistering_non_focused_widget_preserves_focus() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);

        fm.register(id1, true);
        fm.register(id2, true);

        fm.set_focus(id1);
        fm.unregister(id2);

        assert_eq!(fm.focused(), Some(id1));
    }

    #[test]
    fn test_registering_same_widget_twice_does_not_duplicate() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);

        fm.register(id, true);
        fm.register(id, true);

        fm.set_focus(id);
        fm.tab_next();

        assert_eq!(fm.tab_next(), Some(id));
    }
}

/// Test 6: FocusManager with App integration
///
/// Verifies:
/// - FocusManager integrates with App's widget registry
/// - Focus cycling through App's registered widgets works correctly
mod app_integration {
    use super::*;
    use dracon_terminal_engine::framework::app::App;
    use dracon_terminal_engine::framework::widgets::Label;
    use ratatui::layout::Rect;

    #[test]
    fn test_app_adds_widgets_to_focus_ring() {
        let mut app = App::new().unwrap();
        let label = Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));

        assert_eq!(app.widget_count(), 1);

        let mut fm = FocusManager::new();
        fm.register(id, true);
        fm.set_focus(id);
        assert_eq!(fm.focused(), Some(id));
    }

    #[test]
    fn test_app_tab_cycles_through_widgets() {
        let mut app = App::new().unwrap();

        let label1 = Label::new("widget1");
        let id1 = app.add_widget(Box::new(label1), Rect::new(0, 0, 10, 1));

        let label2 = Label::new("widget2");
        let id2 = app.add_widget(Box::new(label2), Rect::new(10, 0, 10, 1));

        let mut fm = FocusManager::new();
        fm.register(id1, true);
        fm.register(id2, true);
        fm.set_focus(id1);
        assert_eq!(fm.focused(), Some(id1));

        let next = fm.tab_next().unwrap();
        assert_eq!(next, id2);
    }

    #[test]
    fn test_app_remove_widget_clears_focus() {
        let mut app = App::new().unwrap();

        let label = Label::new("test");
        let id = app.add_widget(Box::new(label), Rect::new(0, 0, 10, 1));

        let mut fm = FocusManager::new();
        fm.register(id, true);
        fm.set_focus(id);
        assert_eq!(fm.focused(), Some(id));

        app.remove_widget(id);
        assert_eq!(app.widget_count(), 0);
        assert_eq!(fm.focused(), Some(id)); // fm still has old focus
    }

    #[test]
    fn test_app_focus_cycles_through_multiple_widgets() {
        let mut app = App::new().unwrap();

        let label1 = Label::new("w1");
        let id1 = app.add_widget(Box::new(label1), Rect::new(0, 0, 10, 1));

        let label2 = Label::new("w2");
        let id2 = app.add_widget(Box::new(label2), Rect::new(10, 0, 10, 1));

        let label3 = Label::new("w3");
        let id3 = app.add_widget(Box::new(label3), Rect::new(20, 0, 10, 1));

        let mut fm = FocusManager::new();
        fm.register(id1, true);
        fm.register(id2, true);
        fm.register(id3, true);
        fm.set_focus(id1);

        assert_eq!(fm.tab_next(), Some(id2));
        assert_eq!(fm.tab_next(), Some(id3));
        assert_eq!(fm.tab_next(), Some(id1));
    }

    #[test]
    fn test_app_tab_prev_cycles_backward_through_widgets() {
        let mut app = App::new().unwrap();

        let label1 = Label::new("w1");
        let id1 = app.add_widget(Box::new(label1), Rect::new(0, 0, 10, 1));

        let label2 = Label::new("w2");
        let id2 = app.add_widget(Box::new(label2), Rect::new(10, 0, 10, 1));

        let mut fm = FocusManager::new();
        fm.register(id1, true);
        fm.register(id2, true);
        fm.set_focus(id2);

        assert_eq!(fm.tab_prev(), Some(id1));
        assert_eq!(fm.tab_prev(), Some(id2));
    }

    #[test]
    fn test_focus_manager_trap_integration() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);

        fm.register(id1, true);
        fm.register(id2, true);
        fm.set_focus(id1);
        fm.enter_trap();

        assert!(fm.is_trapped());

        fm.enable_trap_exit();
        fm.exit_trap();

        assert!(!fm.is_trapped());
    }
}
