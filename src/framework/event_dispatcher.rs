//! Event dispatcher for routing input events to widgets.
//!
//! Provides `EventDispatcher` which routes keyboard/mouse events to
//! widgets via HitZone groups, with capture/bubble phases and focus-aware
//! routing.

use crate::framework::focus::FocusManager;
use crate::framework::hitzone::{HitZone, HitZoneGroup};
use crate::framework::widget::WidgetId;
use crate::input::event::{KeyEvent, KeyModifiers, MouseEventKind};

struct DispatchEntry {
    zone: HitZone<WidgetId>,
    capture: bool,
}

/// Routes input events to widgets based on hit zones and focus.
pub struct EventDispatcher {
    groups: Vec<HitZoneGroup<WidgetId>>,
    entries: Vec<DispatchEntry>,
    focus_manager: Option<FocusManager>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventDispatcher {
    /// Creates a new `EventDispatcher` without focus management.
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            entries: Vec::new(),
            focus_manager: None,
        }
    }

    /// Creates a new `EventDispatcher` with focus management.
    pub fn with_focus(fm: FocusManager) -> Self {
        Self {
            groups: Vec::new(),
            entries: Vec::new(),
            focus_manager: Some(fm),
        }
    }

    /// Adds a hit zone to the dispatcher with capture or bubble behavior.
    pub fn add_zone(&mut self, zone: HitZone<WidgetId>, capture: bool) {
        self.entries.push(DispatchEntry { zone, capture });
    }

    /// Builds the capture and bubble groups from added zones.
    pub fn build_groups(&mut self) {
        self.groups.clear();
        let mut capture_group = HitZoneGroup::new();
        let mut bubble_group = HitZoneGroup::new();

        for entry in self.entries.drain(..) {
            if entry.capture {
                capture_group.zones_mut().push(entry.zone);
            } else {
                bubble_group.zones_mut().push(entry.zone);
            }
        }

        self.groups.push(capture_group);
        self.groups.push(bubble_group);
    }

    /// Dispatches a mouse event to the first matching zone.
    pub fn dispatch_mouse(
        &mut self,
        kind: MouseEventKind,
        col: u16,
        row: u16,
        modifiers: KeyModifiers,
        handler: &mut dyn FnMut(WidgetId, MouseEventKind, u16, u16, KeyModifiers) -> bool,
    ) {
        for group in self.groups.iter_mut() {
            if let Some(id) = group.dispatch_mouse(kind, col, row, modifiers) {
                if handler(id, kind, col, row, modifiers) {
                    return;
                }
            }
        }
    }

    /// Dispatches a keyboard event to the focused widget or tab navigation.
    pub fn dispatch_key<F>(&mut self, key: KeyEvent, handler: &mut F) -> bool
    where
        F: FnMut(WidgetId, KeyEvent) -> bool,
    {
        if let Some(ref mut fm) = self.focus_manager {
            if key.code == crate::input::event::KeyCode::Tab {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    if let Some(id) = fm.tab_prev() {
                        return handler(id, key);
                    }
                } else {
                    if let Some(id) = fm.tab_next() {
                        return handler(id, key);
                    }
                }
            }

            if let Some(focused) = fm.focused() {
                return handler(focused, key);
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_zone(x: u16, y: u16, w: u16, h: u16) -> HitZone<WidgetId> {
        HitZone::new(WidgetId::new(1), x, y, w, h)
    }

    #[test]
    fn test_add_zone_and_build() {
        let mut dispatcher = EventDispatcher::new();
        let zone = make_zone(0, 0, 10, 10);
        dispatcher.add_zone(zone, true);
        dispatcher.build_groups();
    }

    #[test]
    fn test_dispatch_key_tabs() {
        let mut fm = FocusManager::new();
        fm.register(WidgetId::new(1), true);
        fm.register(WidgetId::new(2), true);

        let mut dispatcher = EventDispatcher::with_focus(fm);

        let key = KeyEvent {
            code: crate::input::event::KeyCode::Tab,
            modifiers: KeyModifiers::empty(),
            kind: crate::input::event::KeyEventKind::Press,
        };

        let mut handled = false;
        dispatcher.dispatch_key(key, &mut |id, _| {
            handled = true;
            assert_eq!(id, WidgetId::new(1));
            true
        });

        assert!(handled);
    }

    #[test]
    fn test_dispatch_mouse_bubble() {
        let mut dispatcher = EventDispatcher::new();
        let zone = make_zone(0, 0, 20, 10);
        dispatcher.add_zone(zone, false);
        dispatcher.build_groups();

        let mut hit_id = None;
        dispatcher.dispatch_mouse(
            MouseEventKind::Down(crate::input::event::MouseButton::Left),
            5,
            5,
            KeyModifiers::empty(),
            &mut |id, _, _, _, _| {
                hit_id = Some(id);
                true
            },
        );

        assert_eq!(hit_id, Some(WidgetId::new(1)));
    }
}
