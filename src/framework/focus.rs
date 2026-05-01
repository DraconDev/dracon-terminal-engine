//! Focus management for widget focus ordering and navigation.
//!
//! Provides `FocusManager` which maintains a tab-order ring of widgets,
//! handles Tab/Shift+Tab cycling, and supports focus trapping (for modals).

use crate::framework::widget::WidgetId;
use std::collections::HashMap;
use std::sync::Arc;

/// Callback invoked when focus changes to a new widget.
pub type FocusCallback = Box<dyn Fn(WidgetId, Option<WidgetId>) + Send + Sync>;

/// Callback invoked when focus trap is entered/exited.
pub type TrapCallback = Box<dyn Fn(bool) + Send + Sync>;

/// Callback invoked when focus changes, providing old and new widget IDs.
pub type FocusChangeCallback = Arc<dyn Fn(Option<WidgetId>, Option<WidgetId>) + Send + Sync>;

/// Manages widget focus ordering, tab navigation, and focus trapping.
pub struct FocusManager {
    tab_order: Vec<WidgetId>,
    focused: Option<WidgetId>,
    focusable: HashMap<WidgetId, bool>,
    on_focus_change: Vec<Arc<FocusCallback>>,
    on_trap_change: Vec<Arc<TrapCallback>>,
    on_focus_change_internal: Vec<FocusChangeCallback>,
    trapped: bool,
    trap_exit_disabled: bool,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Creates a new empty FocusManager.
    pub fn new() -> Self {
        Self {
            tab_order: Vec::new(),
            focused: None,
            focusable: HashMap::new(),
            on_focus_change: Vec::new(),
            on_trap_change: Vec::new(),
            on_focus_change_internal: Vec::new(),
            trapped: false,
            trap_exit_disabled: false,
        }
    }

    /// Registers a widget in the focus ring.
    pub fn register(&mut self, id: WidgetId, focusable: bool) {
        if !self.tab_order.contains(&id) {
            self.tab_order.push(id);
        }
        self.focusable.insert(id, focusable);
    }

    /// Unregisters a widget from the focus ring.
    pub fn unregister(&mut self, id: WidgetId) {
        self.tab_order.retain(|&i| i != id);
        self.focusable.remove(&id);
        if self.focused == Some(id) {
            self.focused = None;
        }
    }

    /// Sets whether a widget can receive focus.
    pub fn set_focusable(&mut self, id: WidgetId, focusable: bool) {
        self.focusable.insert(id, focusable);
    }

    /// Returns the currently focused widget ID, if any.
    pub fn focused(&self) -> Option<WidgetId> {
        self.focused
    }

    /// Sets focus to a specific widget.
    /// Returns `true` if focus was successfully set.
    pub fn set_focus(&mut self, id: WidgetId) -> bool {
        if !self.focusable.get(&id).copied().unwrap_or(false) {
            return false;
        }
        let old = self.focused;
        if old != Some(id) {
            self.focused = Some(id);
            self.notify_focus_change(Some(id), old);
        }
        true
    }

    /// Clears focus (no widget is focused).
    pub fn clear_focus(&mut self) {
        let old = self.focused.take();
        self.notify_focus_change(None, old);
    }

    /// Advances focus to the next widget in tab order (Tab key).
    /// Wraps around to the start if at the end.
    /// Returns the new focused widget ID, or None if no focusable widgets exist.
    pub fn tab_next(&mut self) -> Option<WidgetId> {
        self.navigate(1)
    }

    /// Advances focus to the previous widget in tab order (Shift+Tab).
    /// Wraps around to the end if at the start.
    /// Returns the new focused widget ID, or None if no focusable widgets exist.
    pub fn tab_prev(&mut self) -> Option<WidgetId> {
        self.navigate(-1)
    }

    fn navigate(&mut self, delta: isize) -> Option<WidgetId> {
        let focusable_ids: Vec<WidgetId> = self
            .tab_order
            .iter()
            .filter(|id| self.focusable.get(id).copied().unwrap_or(false))
            .cloned()
            .collect();

        if focusable_ids.is_empty() {
            return None;
        }

        let current = self
            .focused
            .and_then(|f| focusable_ids.iter().position(|&id| id == f));
        let next = match current {
            Some(idx) => {
                let new_idx = ((idx as isize + delta + focusable_ids.len() as isize)
                    % focusable_ids.len() as isize) as usize;
                focusable_ids[new_idx]
            }
            None => focusable_ids[0],
        };

        let old = self.focused;
        self.focused = Some(next);
        self.notify_focus_change(Some(next), old);
        Some(next)
    }

    /// Enables focus trapping — Tab/Shift+Tab cycle within the trap and Esc is disabled.
    /// Used when a modal dialog is open.
    pub fn enter_trap(&mut self) {
        if !self.trapped {
            self.trapped = true;
            self.trap_exit_disabled = true;
            for cb in &self.on_trap_change {
                cb(true);
            }
        }
    }

    /// Disables focus trapping — normal Tab cycling resumes.
    pub fn exit_trap(&mut self) {
        if self.trapped && !self.trap_exit_disabled {
            self.trapped = false;
            for cb in &self.on_trap_change {
                cb(false);
            }
        }
    }

    /// Temporarily allows exiting the trap (call before exit_trap to re-enable it).
    pub fn enable_trap_exit(&mut self) {
        self.trap_exit_disabled = false;
    }

    /// Returns true if focus is currently trapped (inside a modal).
    pub fn is_trapped(&self) -> bool {
        self.trapped
    }

    /// Registers a callback invoked whenever focus changes.
    pub fn on_focus_change<F>(&mut self, f: F)
    where
        F: Fn(WidgetId, Option<WidgetId>) + Send + Sync + 'static,
    {
        self.on_focus_change.push(Arc::new(Box::new(f)));
    }

    /// Registers a callback invoked when focus trapping state changes.
    pub fn on_trap_change<F>(&mut self, f: F)
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_trap_change.push(Arc::new(Box::new(f)));
    }

    #[allow(dead_code)]
    pub(crate) fn on_focus_change_internal<F>(&mut self, f: F)
    where
        F: Fn(Option<WidgetId>, Option<WidgetId>) + Send + Sync + 'static,
    {
        self.on_focus_change_internal.push(Arc::new(f));
    }

    fn notify_focus_change(&self, new: Option<WidgetId>, old: Option<WidgetId>) {
        for cb in &self.on_focus_change {
            if let Some(n) = new {
                cb(n, old);
            }
        }
        for cb in &self.on_focus_change_internal {
            cb(new, old);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_focus() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);
        fm.register(id, true);

        assert!(fm.set_focus(id));
        assert_eq!(fm.focused(), Some(id));
    }

    #[test]
    fn test_tab_next_cycles() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);
        fm.register(id1, true);
        fm.register(id2, true);

        assert!(fm.set_focus(id1));
        let next = fm.tab_next().unwrap();
        assert_eq!(next, id2);

        let next2 = fm.tab_next().unwrap();
        assert_eq!(next2, id1);
    }

    #[test]
    fn test_tab_prev_cycles_reverse() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);
        fm.register(id1, true);
        fm.register(id2, true);

        assert!(fm.set_focus(id2));
        let prev = fm.tab_prev().unwrap();
        assert_eq!(prev, id1);
    }

    #[test]
    fn test_non_focusable_not_focused() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);
        fm.register(id, false);

        assert!(!fm.set_focus(id));
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn test_trap_prevents_exit() {
        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);
        fm.register(id1, true);
        fm.register(id2, true);

        assert!(fm.set_focus(id1));
        fm.enter_trap();
        assert!(fm.is_trapped());

        fm.enable_trap_exit();
        fm.exit_trap();
        assert!(!fm.is_trapped());
    }

    #[test]
    fn test_unregister_clears_focus() {
        let mut fm = FocusManager::new();
        let id = WidgetId::new(1);
        fm.register(id, true);
        assert!(fm.set_focus(id));
        fm.unregister(id);

        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn test_focus_change_callback() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let mut fm = FocusManager::new();
        let id1 = WidgetId::new(1);
        let id2 = WidgetId::new(2);
        fm.register(id1, true);
        fm.register(id2, true);

        let changes = Arc::new(Mutex::new(Vec::new()));
        let changes_clone = changes.clone();
        fm.on_focus_change(move |new, old| {
            changes_clone.lock().unwrap().push((new, old));
        });

        assert!(fm.set_focus(id1));
        fm.tab_next();

        assert_eq!(changes.lock().unwrap().len(), 2);
    }
}
